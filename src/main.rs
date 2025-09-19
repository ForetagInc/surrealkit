use std::path::PathBuf;

use clap::{Parser, Subcommand};
use rust_dotenv::dotenv::DotEnv;

mod config;
mod core;
mod migration;
mod scaffold;
mod seed;
mod setup;
mod status;

use config::{DbCfg, connect};
use migration::{migrate_all, apply_one};
use setup::run_setup;
use status::status;

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
		dry_run: bool
	},
	Seed,
	Status,
	Apply {
		path: PathBuf,
		#[arg(long)]
		track: bool,
	},
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
	#[arg(short, long)]
	name: String,

	#[arg(short, long, default_value_t = 1)]
	count: u8,
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
	let cfg = DbCfg::from_env(&env)?;

	let db = connect(&cfg).await?;

    match args.command {
		Commands::Init => scaffold::scaffold()?,
		Commands::Setup => run_setup(&db).await?,
		Commands::Migrate { fail_fast, dry_run } => migrate_all(&db,  fail_fast, dry_run).await?,
		Commands::Seed => seed::seed(&db).await?,
		Commands::Status => status(&db).await?,
		Commands::Apply { path, track } => apply_one(&db, &path, track).await?,
    }

	Ok(())
}
