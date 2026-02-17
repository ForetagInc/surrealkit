use anyhow::{Context, Result};
use std::{fs, path::Path};

pub fn scaffold() -> Result<()> {
	let database_dir = Path::new("database");
	let schema_dir = database_dir.join("schema");
	let migrations_dir = database_dir.join("migrations");
	let state_dir = database_dir.join(".surrealkit");
	let tests_dir = database_dir.join("tests");

	fs::create_dir_all(&schema_dir).context("creating database/schema")?;
	fs::create_dir_all(&migrations_dir).context("creating database/migrations")?;
	fs::create_dir_all(&state_dir).context("creating database/.surrealkit")?;
	fs::create_dir_all(&tests_dir).context("creating database/tests")?;

	// seed.surql (idempotent-ish example)
	let seed_path = database_dir.join("seed.surql");
	if !seed_path.exists() {
		fs::write(&seed_path, "--- SEED\n").context("Writing seed.surql")?;
	}

	// setup.surql defines _migration table/indexes
	let setup_path = database_dir.join("setup.surql");
	if !setup_path.exists() {
		fs::write(&setup_path, DEFAULT_SETUP).context("Writing setup.surql")?;
	}

	println!(
		"Scaffolded ./database, ./database/schema, ./database/migrations, ./database/.surrealkit, ./database/tests, seed.surql, setup.surql"
	);
	Ok(())
}

pub const DEFAULT_SETUP: &str = r#"---
--- Migrations: Bootstrap _migration table for tracking
---
DEFINE TABLE OVERWRITE _migration SCHEMAFULL
	PERMISSIONS NONE;

DEFINE FIELD OVERWRITE file ON _migration
	TYPE string
	COMMENT "Relative path to migration file";

DEFINE FIELD OVERWRITE applied_at ON _migration
	TYPE datetime
	DEFAULT time::now();

---
--- Indexes
---
DEFINE INDEX OVERWRITE by_file ON _migration
	FIELDS file
	COMMENT "Lookup by file name";

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
