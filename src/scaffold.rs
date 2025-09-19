use std::{fs, path::Path};
use anyhow::{Context, Result};

pub fn scaffold() -> Result<()> {
	let database_dir = Path::new("database");
	let schema_dir = database_dir.join("schema");
	let tests_dir = database_dir.join("tests");

	fs::create_dir_all(&schema_dir).context("creating database/schema")?;
	fs::create_dir_all(&tests_dir).context("creating database/tests")?;

	// seed.surql (idempotent-ish example)
	let seed_path = database_dir.join("seed.surql");
	if !seed_path.exists() {
		fs::write(&seed_path, "--- SEED\n")
			.context("Writing seed.surql")?;
	}

	// setup.surql defines _migrations table/indexes
	let setup_path = database_dir.join("setup.surql");
	if !setup_path.exists() {
		fs::write(&setup_path, DEFAULT_SETUP)
			.context("Writing setup.surql")?;
	}

	println!("Scaffolded ./database, ./database/schema, ./database/tests, seed.surql, setup.surql");
	Ok(())
}

pub const DEFAULT_SETUP: &str = r#"---
--- Migrations: Bootstrap _migrations table for tracking
---
DEFINE TABLE OVERWRITE _migrations SCHEMAFULL
	PERMISSIONS NONE;

DEFINE FIELD OVERWRITE file ON _migrations
	TYPE string
	COMMENT "Relative path to migration file";

DEFINE FIELD OVERWRITE applied_at ON _migrations
	TYPE datetime
	DEFAULT time::now();

---
--- Indexes
---
DEFINE INDEX OVERWRITE by_file ON _migrations
	FIELDS file
	COMMENT "Lookup by file name";
"#;
