use std::path::PathBuf;

use clap::{Parser, Subcommand};
use rust_dotenv::dotenv::DotEnv;
use surrealdb::{Surreal, engine::any::Any};

mod commit;
mod config;
mod core;
mod migration;
mod scaffold;
mod schema_state;
mod seed;
mod setup;
mod status;
mod sync;

use commit::CommitOpts;
use config::{DbCfg, connect};
use migration::{apply_one, migrate_all};
use setup::run_setup;
use status::status;
use sync::SyncOpts;

#[derive(Parser, Debug)]
#[command(version, about = "SurrealKit CLI")]
pub struct Cli {
	/// Increase output
	#[arg(short, long, global = true)]
	verbose: bool,

	#[command(subcommand)]
	command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
	Init,
	Setup,
	Migrate {
		#[arg(long, default_value_t = true)]
		fail_fast: bool,

		#[arg(long)]
		dry_run: bool,
	},
	Sync {
		#[arg(long)]
		watch: bool,
		#[arg(long, default_value_t = 1000)]
		debounce_ms: u64,
		#[arg(long)]
		dry_run: bool,
		#[arg(long, default_value_t = true)]
		fail_fast: bool,
		#[arg(long)]
		no_prune: bool,
		#[arg(long)]
		allow_shared_prune: bool,
	},
	Commit {
		#[arg(long)]
		name: Option<String>,
		#[arg(long)]
		dry_run: bool,
		#[arg(long)]
		allow_empty: bool,
	},
	Seed,
	Status,
	Apply {
		path: PathBuf,
		#[arg(long)]
		track: bool,
	},
}

fn load_env() -> DotEnv {
	// Load .env in CWD if present, ignore missing
	let env = DotEnv::new("");
	env
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let args = Cli::parse();
	let env = load_env();

	match args.command {
		Commands::Init => scaffold::scaffold()?,
		Commands::Setup => {
			let db = connect_from_env(&env).await?;
			run_setup(&db).await?;
		}
		Commands::Migrate { fail_fast, dry_run } => {
			let db = connect_from_env(&env).await?;
			migrate_all(&db, fail_fast, dry_run).await?;
		}
		Commands::Sync {
			watch,
			debounce_ms,
			dry_run,
			fail_fast,
			no_prune,
			allow_shared_prune,
		} => {
			let db = connect_from_env(&env).await?;
			sync::run_sync(
				&db,
				SyncOpts {
					watch,
					debounce_ms,
					dry_run,
					fail_fast,
					prune: !no_prune,
					allow_shared_prune,
				},
			)
			.await?;
		}
		Commands::Commit {
			name,
			dry_run,
			allow_empty,
		} => {
			commit::run_commit(CommitOpts {
				name,
				dry_run,
				allow_empty,
			})
			.await?;
		}
		Commands::Seed => {
			let db = connect_from_env(&env).await?;
			seed::seed(&db).await?;
		}
		Commands::Status => {
			let db = connect_from_env(&env).await?;
			status(&db).await?;
		}
		Commands::Apply { path, track } => {
			let db = connect_from_env(&env).await?;
			apply_one(&db, &path, track).await?;
		}
	}

	Ok(())
}

async fn connect_from_env(env: &DotEnv) -> anyhow::Result<Surreal<Any>> {
	let cfg = DbCfg::from_env(env)?;
	connect(&cfg).await
}
