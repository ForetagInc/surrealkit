use anyhow::Result;
use surrealdb::{Surreal, engine::any::Any};

pub async fn status(db: &Surreal<Any>) -> Result<()> {
	#[derive(serde::Deserialize, Debug)]
	struct Mig { id: String, file: String, applied_at: String }

	let mut resp = db.query("SELECT id, file, applied_at FROM _migrations ORDER BY applied_at;").await?;
	let rows: Vec<Mig> = resp.take(0)?;
		if rows.is_empty() {
		println!("No migrations recorded");
	} else {
		println!("Applied migrations:");
		for m in rows { println!("{} {} {}", m.applied_at, m.id, m.file); }
	}
	Ok(())
}
