use anyhow::{Result, Context};
use std::{fs, path::{Path, PathBuf}};
use sha2::{Sha256, Digest};
use walkdir::WalkDir;
use surrealdb::{Surreal, engine::any::Any};

use crate::{core::{display, exec_surql}, setup::run_setup};

pub async fn migrate_all(
    db: &Surreal<Any>,
    fail_fast: bool,
    dry_run: bool,
) -> Result<()> {
    // Ensure _migrations table exists
    run_setup(db).await?;

    // Collect all .surql files from migrations directory
    let mut files: Vec<PathBuf> = WalkDir::new("database/schema")
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| e.into_path())
        .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("surql"))
        .collect();

    files.sort();

    if files.is_empty() {
        println!("No .surql files found in {:?}", "database/schema");
        return Ok(());
    }

    for path in files {
        if dry_run {
            println!("DRY RUN: would apply {}", display(&path));
            continue;
        }

        match apply_migration_file(db, &path).await {
            Ok(applied) => {
                if applied {
                    println!("applied {}", display(&path));
                } else {
                    println!("skipped {} (already applied)", display(&path));
                }
            }
            Err(e) => {
                eprintln!("error applying {}: {e:#}", display(&path));
                if fail_fast {
                    return Err(e);
                }
            }
        }
    }

    Ok(())
}

pub async fn apply_migration_file(db: &Surreal<Any>, path: &Path) -> Result<bool> {
	let sql = fs::read_to_string(path).with_context(|| format!("reading {}", display(path)))?;
	let hash = sha256_hex(sql.as_bytes());

	// Already applied?
	let mut resp = db.query("SELECT * FROM _migrations WHERE id = $id;")
		.bind(("id", hash.clone()))
		.await?;


	let existing: Option<serde_json::Value> = resp.take(0)?;
		if existing.is_some() {
		return Ok(false);
	}

	exec_surql(db, &sql).await?;

	let file = path.to_string_lossy().into_owned();

	db.query("CREATE _migrations CONTENT { id: $id, file: $file, applied_at: $ts };")
		.bind(("id", hash))
		.bind(("file", file))
		.await?
		.check()?;

	Ok(true)
}


pub async fn apply_one(db: &Surreal<Any>, path: &Path, track: bool) -> Result<()> {
	if track { apply_migration_file(db, path).await.map(|_| ()) } else {
		let sql = fs::read_to_string(path)?; exec_surql(db, &sql).await
	}
}

pub fn sha256_hex(bytes: &[u8]) -> String {
	let mut hasher = Sha256::new();
	hasher.update(bytes);
	hex::encode(hasher.finalize())
}
