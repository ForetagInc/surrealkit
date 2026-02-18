use std::collections::BTreeMap;
use std::env;
use std::time::Duration;

use anyhow::{Result, bail};
use surrealdb::{Surreal, engine::any::Any};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

use crate::core::exec_surql;
use crate::schema_state::{
	build_catalog_snapshot, collect_schema_files, ensure_local_state_dirs, load_catalog_snapshot,
	removed_entities, render_remove_sql,
};
use crate::setup::run_setup;

#[derive(Debug, Clone)]
pub struct SyncOpts {
	pub watch: bool,
	pub debounce_ms: u64,
	pub dry_run: bool,
	pub fail_fast: bool,
	pub prune: bool,
	pub allow_shared_prune: bool,
}

pub async fn run_sync(db: &Surreal<Any>, opts: SyncOpts) -> Result<()> {
	run_setup(db).await?;
	ensure_local_state_dirs()?;

	if opts.watch {
		run_sync_once(db, &opts, true).await?;
		println!(
			"Watch mode active ({}ms interval). Waiting for schema changes... (Ctrl+C to stop)",
			opts.debounce_ms.max(250)
		);
		loop {
			tokio::select! {
				_ = tokio::signal::ctrl_c() => {
					println!("Stopping schema watch.");
					break;
				}
				_ = tokio::time::sleep(Duration::from_millis(opts.debounce_ms.max(250))) => {
					if let Err(err) = run_sync_once(db, &opts, true).await {
						if opts.fail_fast {
							return Err(err);
						}
						eprintln!("sync iteration error: {err:#}");
					}
				}
			}
		}
		Ok(())
	} else {
		run_sync_once(db, &opts, false).await
	}
}

async fn run_sync_once(db: &Surreal<Any>, opts: &SyncOpts, watch_mode: bool) -> Result<()> {
	let files = collect_schema_files()?;
	let tracked = load_sync_hashes(db).await?;

	if files.is_empty() && !watch_mode {
		println!("No schema files found in database/schema");
	}

	let mut changed_count = 0usize;
	let mut apply_errors = 0usize;
	let mut pruned_count = 0usize;
	for file in &files {
		let tracked_hash = tracked.get(&file.path);
		if tracked_hash == Some(&file.hash) {
			continue;
		}

		changed_count += 1;
		if opts.dry_run {
			if !watch_mode {
				println!("DRY RUN: would apply {}", file.path);
			}
			continue;
		}

		match exec_surql(db, &file.sql).await {
			Ok(_) => {
				if !watch_mode {
					println!("applied {}", file.path);
				}
				store_sync_hash(db, &file.path, &file.hash).await?;
			}
			Err(err) => {
				apply_errors += 1;
				eprintln!("error applying {}: {err:#}", file.path);
				if opts.fail_fast {
					return Err(err);
				}
			}
		}
	}

	let old_catalog = load_catalog_snapshot()?;
	let new_catalog = build_catalog_snapshot(&files);
	let stale_entities = removed_entities(&old_catalog, &new_catalog);
	let stale_count = stale_entities.len();

	if opts.prune && !stale_entities.is_empty() {
		let shared = detect_shared_db(db).await?;
		if shared && !opts.allow_shared_prune {
			bail!("database is marked shared; refusing stale prune without --allow-shared-prune");
		}

		let remove_sql = render_remove_sql(&stale_entities, true)?;
		let sql = remove_sql.join("\n");
		if opts.dry_run {
			if !watch_mode {
				println!("DRY RUN: would prune {} stale entities", remove_sql.len());
				for stmt in remove_sql {
					println!("  {}", stmt);
				}
			}
		} else {
			exec_surql(db, &sql).await?;
			pruned_count = stale_count;
			if !watch_mode {
				println!("pruned {} stale entities", stale_count);
			}
		}
	}

	if !opts.dry_run {
		write_meta_from_env(db).await?;
		store_last_sync_meta(db).await?;
	}

	if watch_mode {
		let has_changes = changed_count > 0 || stale_count > 0;
		if has_changes {
			if opts.dry_run {
				println!(
					"Change detected (dry-run): {} schema file(s), {} stale entity(ies) would be pushed.",
					changed_count, stale_count
				);
			} else {
				println!(
					"Change detected and pushed: {} schema file(s) synced, {} stale entity(ies) pruned.",
					changed_count, pruned_count
				);
			}
		}
	} else if changed_count == 0 {
		println!("schema already in sync");
	}

	if apply_errors > 0 {
		eprintln!("sync completed with {} apply error(s)", apply_errors);
	}
	if stale_count > 0 && !opts.prune {
		println!(
			"detected {} stale entities; rerun without --no-prune to remove",
			stale_count
		);
	}

	Ok(())
}

async fn load_sync_hashes(db: &Surreal<Any>) -> Result<BTreeMap<String, String>> {
	let mut resp = db.query("SELECT path, hash FROM _surrealkit_sync;").await?;
	let rows: Vec<serde_json::Value> = resp.take(0)?;

	let mut out = BTreeMap::new();
	for row in rows {
		let path = row.get("path").and_then(|v| v.as_str()).map(str::to_string);
		let hash = row.get("hash").and_then(|v| v.as_str()).map(str::to_string);
		if let (Some(path), Some(hash)) = (path, hash) {
			out.insert(path, hash);
		}
	}
	Ok(out)
}

async fn store_sync_hash(db: &Surreal<Any>, path: &str, hash: &str) -> Result<()> {
	db.query(
		"DELETE _surrealkit_sync WHERE path = $path; \
		 CREATE _surrealkit_sync CONTENT { path: $path, hash: $hash, synced_at: time::now() };",
	)
	.bind(("path", path.to_string()))
	.bind(("hash", hash.to_string()))
	.await?
	.check()?;
	Ok(())
}

async fn detect_shared_db(db: &Surreal<Any>) -> Result<bool> {
	if let Ok(value) = env::var("SURREALKIT_SHARED_DB") {
		if let Some(parsed) = parse_bool(&value) {
			return Ok(parsed);
		}
	}

	let mut resp = db
		.query("SELECT value FROM _surrealkit_sync_meta WHERE key = 'shared' LIMIT 1;")
		.await?;
	let row: Option<serde_json::Value> = resp.take(0)?;
	let shared = row
		.as_ref()
		.and_then(|v| v.get("value"))
		.and_then(|v| v.as_bool())
		.unwrap_or(false);
	Ok(shared)
}

async fn write_meta_from_env(db: &Surreal<Any>) -> Result<()> {
	if let Ok(raw_shared) = env::var("SURREALKIT_SHARED_DB") {
		if let Some(shared) = parse_bool(&raw_shared) {
			upsert_meta(db, "shared", serde_json::json!(shared)).await?;
		}
	}
	if let Ok(owner) = env::var("SURREALKIT_OWNER") {
		if !owner.trim().is_empty() {
			upsert_meta(db, "owner", serde_json::json!(owner)).await?;
		}
	}
	Ok(())
}

async fn store_last_sync_meta(db: &Surreal<Any>) -> Result<()> {
	let ts = OffsetDateTime::now_utc().format(&Rfc3339)?;
	upsert_meta(db, "last_sync", serde_json::json!(ts)).await
}

async fn upsert_meta(db: &Surreal<Any>, key: &str, value: serde_json::Value) -> Result<()> {
	db.query(
		"DELETE _surrealkit_sync_meta WHERE key = $key; \
		 CREATE _surrealkit_sync_meta CONTENT { key: $key, value: $value, updated_at: time::now() };",
	)
	.bind(("key", key.to_string()))
	.bind(("value", value))
	.await?
	.check()?;
	Ok(())
}

fn parse_bool(raw: &str) -> Option<bool> {
	match raw.trim().to_ascii_lowercase().as_str() {
		"1" | "true" | "yes" | "y" | "on" => Some(true),
		"0" | "false" | "no" | "n" | "off" => Some(false),
		_ => None,
	}
}
