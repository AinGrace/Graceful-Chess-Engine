use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

use crate::app::App;

mod app;
mod config;

fn main() -> Result<()> {
    let cli = Cli::parse();

    let app = App::new(&cli.config, cli.keep_workspaces)?;
    println!("parsed app => {app:#?}");

    app.check_requirements()?;

    match cli.revisions.as_slice() {
        [revision] => {
            app.run_gauntlet(revision)?;
        }

        [new_revision, base_revision] => {
            app.run_sprt(new_revision, base_revision)?;
        }

        _ => unreachable!(),
    }

    Ok(())
}

#[derive(Parser, Debug)]
#[command(
    name = "graceful-bench",
    about = "Benchmark for various revisions of Graceful-engine"
)]
struct Cli {
    /// New/target engine revision.
    ///
    /// With one revision: Stockfish gauntlet.
    /// With two revisions: SPRT comparison.
    #[arg(required = true, num_args = 1..=2)]
    revisions: Vec<String>,

    /// Path to configuration file.
    #[arg(short, long, default_value = "config.toml")]
    config: PathBuf,

    /// Don't delete temporary JJ workspaces after the test.
    #[arg(long)]
    keep_workspaces: bool,
}

struct Workspace {
    name: String,
    path: PathBuf,
}
