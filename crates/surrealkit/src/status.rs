use crate::migration::Migration;
use anyhow::Result;
use surrealdb::{Surreal, engine::any::Any};

pub async fn status(db: &Surreal<Any>) -> Result<()> {
	let mut resp = db
		.query("SELECT id, file, applied_at FROM _migration ORDER BY applied_at;")
		.await?;
	let rows: Vec<Migration> = resp.take(0)?;

	if rows.is_empty() {
		println!("No migrations recorded");
	} else {
		println!("Applied migrations:");
		for m in rows {
			println!("{} {} {}", m.applied_at, m.id, m.file);
		}
	}
	Ok(())
}
