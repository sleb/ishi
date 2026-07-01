use std::env;

use anyhow::Context;
use clap::{Parser, Subcommand};

use tick::cli::{self, TerminalUi};
use tick::editor::RealEditor;
use tick::workspace::Workspace;

#[derive(Parser)]
#[command(name = "tk")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Capture a new note in the Inbox.
    New { filename: Option<String> },
}

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let cli = Cli::parse();

    let cwd = env::current_dir().context("failed to determine current directory")?;
    let ws = Workspace::discover(&cwd).context("failed to find a PARA workspace")?;

    match cli.command {
        Commands::New { filename } => {
            if filename.is_none() {
                println!("Opening $EDITOR...");
            }
            let editor = RealEditor;
            let mut ui = TerminalUi;
            let path = cli::run_new(&ws, &editor, &mut ui, filename)?;
            println!("Created {}", path.display());
        }
    }

    Ok(())
}
