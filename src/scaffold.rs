use anyhow::{Context, Result};
use std::{fs, path::Path};

pub fn scaffold() -> Result<()> {
	let database_dir = Path::new("database");
	let schema_dir = database_dir.join("schema");
	let migrations_dir = database_dir.join("migrations");
	let state_dir = database_dir.join(".surrealkit");
	let tests_dir = database_dir.join("tests");
	let test_suites_dir = tests_dir.join("suites");
	let test_fixtures_dir = tests_dir.join("fixtures");

	fs::create_dir_all(&schema_dir).context("creating database/schema")?;
	fs::create_dir_all(&migrations_dir).context("creating database/migrations")?;
	fs::create_dir_all(&state_dir).context("creating database/.surrealkit")?;
	fs::create_dir_all(&tests_dir).context("creating database/tests")?;
	fs::create_dir_all(&test_suites_dir).context("creating database/tests/suites")?;
	fs::create_dir_all(&test_fixtures_dir).context("creating database/tests/fixtures")?;

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

	let test_config_path = tests_dir.join("config.toml");
	if !test_config_path.exists() {
		fs::write(&test_config_path, DEFAULT_TEST_CONFIG)
			.context("Writing database/tests/config.toml")?;
	}

	let test_suite_path = test_suites_dir.join("smoke.toml");
	if !test_suite_path.exists() {
		fs::write(&test_suite_path, DEFAULT_TEST_SUITE)
			.context("Writing database/tests/suites/smoke.toml")?;
	}

	println!(
		"Scaffolded ./database, ./database/schema, ./database/migrations, ./database/.surrealkit, ./database/tests, ./database/tests/suites, ./database/tests/fixtures, seed.surql, setup.surql"
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

pub const DEFAULT_TEST_CONFIG: &str = r#"[defaults]
timeout_ms = 10000

[actors.root]
kind = "root"
"#;

pub const DEFAULT_TEST_SUITE: &str = r#"name = "smoke"
tags = ["smoke"]

[[cases]]
name = "migration_table_visible"
kind = "schema_metadata"
sql = "INFO FOR TABLE _migration;"
contains = ["_migration"]
"#;
