use anyhow::{Result, Context};
use std::{collections::BTreeMap, fs, path::{Path, PathBuf}};
use sha2::{Sha256, Digest};
use walkdir::WalkDir;
use surrealdb::{Surreal, engine::any::Any};
use surrealdb_types::{SurrealValue, Object, Value, Kind};

use crate::{core::{display, exec_surql}, setup::run_setup};

#[derive(serde::Deserialize, Debug)]
pub struct Migration {
	pub id: String,
	pub file: String,
	pub applied_at: String
}

impl SurrealValue for Migration {
	fn kind_of() -> Kind {
		Kind::Object
	}

	fn from_value(value: Value) -> Result<Self> {
		let map = value.as_object().context("Expected Object")?;
		let id = map.get("id").context("Missing ID")?.as_string().context("expected string")?.to_string();
		let file = map.get("file").context("Missing File")?.as_string().context("expected string")?.to_string();
		let applied_at = map.get("applied_at").context("Missing Applied At")?.as_string().context("expected string")?.to_string();

		Ok(Self { id, file, applied_at })
	}

	fn into_value(self) -> Value {
		let mut map = BTreeMap::new();
		map.insert("id".to_string(), Value::String(self.id));
		map.insert("file".to_string(), Value::String(self.file));
		map.insert("applied_at".to_string(), Value::String(self.applied_at));

		Value::Object(Object::from(map))
	}
}

pub async fn migrate_all(
    db: &Surreal<Any>,
    fail_fast: bool,
    dry_run: bool,
) -> Result<()> {
    // Ensure _migration table exists
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
	let mut resp = db.query("SELECT * FROM _migration WHERE id = $id;")
		.bind(("id", hash.clone()))
		.await?;


	let existing: Option<serde_json::Value> = resp.take(0)?;
		if existing.is_some() {
		return Ok(false);
	}

	exec_surql(db, &sql).await?;

	let file = path.to_string_lossy().into_owned();

	db.query("CREATE _migration CONTENT { id: $id, file: $file, applied_at: $ts };")
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
