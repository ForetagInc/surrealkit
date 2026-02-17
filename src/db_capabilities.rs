use anyhow::Result;
use surrealdb::{Surreal, engine::any::Any};

pub async fn supports_remove_api(db: &Surreal<Any>) -> Result<bool> {
	let result = db
		.query("REMOVE API __surrealkit_capability_probe__;")
		.await;
	match result {
		Ok(_) => Ok(true),
		Err(err) => {
			let msg = err.to_string().to_ascii_lowercase();
			if msg.contains("unexpected")
				|| msg.contains("parse")
				|| msg.contains("not implemented")
				|| msg.contains("invalid statement")
			{
				Ok(false)
			} else {
				// Errors like "API does not exist" still imply syntax support.
				Ok(true)
			}
		}
	}
}
