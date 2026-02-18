use anyhow::{Context, Result};
use std::{fs, path::Path};
use surrealdb::{Surreal, engine::any::Any};

use crate::scaffold::DEFAULT_SETUP;

pub async fn run_setup(db: &Surreal<Any>) -> Result<()> {
	let setup_file = Path::new("database/setup.surql");

	// Create a default setup file if it's missing (so `_migration` exists).
	if !setup_file.exists() {
		if let Some(parent) = setup_file.parent() {
			fs::create_dir_all(parent).context("creating setup file directory")?;
		}

		fs::write(setup_file, DEFAULT_SETUP)
			.with_context(|| format!("writing {:?}", setup_file))?;
	}

	// Read and execute the setup SQL.
	let sql =
		fs::read_to_string(setup_file).with_context(|| format!("reading {:?}", setup_file))?;

	db.query(&sql).await?.check()?;
	db.query(EXTRA_SETUP).await?.check()?;
	Ok(())
}

const EXTRA_SETUP: &str = r#"
DEFINE TABLE OVERWRITE _surrealkit_sync SCHEMAFULL
	PERMISSIONS NONE;

DEFINE FIELD OVERWRITE path ON _surrealkit_sync
	TYPE string;

DEFINE FIELD OVERWRITE hash ON _surrealkit_sync
	TYPE string;

DEFINE FIELD OVERWRITE synced_at ON _surrealkit_sync
	TYPE datetime
	DEFAULT time::now();

DEFINE INDEX OVERWRITE by_path ON _surrealkit_sync
	FIELDS path
	UNIQUE;

DEFINE TABLE OVERWRITE _surrealkit_sync_meta SCHEMAFULL
	PERMISSIONS NONE;

DEFINE FIELD OVERWRITE key ON _surrealkit_sync_meta
	TYPE string;

DEFINE FIELD OVERWRITE value ON _surrealkit_sync_meta
	TYPE any;

DEFINE FIELD OVERWRITE updated_at ON _surrealkit_sync_meta
	TYPE datetime
	DEFAULT time::now();

DEFINE INDEX OVERWRITE by_key ON _surrealkit_sync_meta
	FIELDS key
	UNIQUE;
"#;
