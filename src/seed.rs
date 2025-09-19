use std::{path::Path, fs};
use anyhow::{anyhow, Result};
use surrealdb::{ Surreal, engine::any::Any };

use crate::core::{exec_surql, display};

pub async fn seed(db: &Surreal<Any>) -> Result<()> {
	let path = Path::new("database/seed.surql");

	if !path.exists() {
		return Err(anyhow!("seed file not found: {}", display(path)));
	}

	let sql = fs::read_to_string(path)?;
	exec_surql(db, &sql).await
}
