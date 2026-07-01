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

#[derive(Debug, PartialEq, Subcommand)]
enum Commands {
    /// Capture a new note in the Inbox.
    New { filename: Option<String> },
    /// Scaffold a PARA system.
    Init { name: Option<String> },
}

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let cli = Cli::parse();

    let cwd = env::current_dir().context("failed to determine current directory")?;

    match cli.command {
        Commands::Init { name } => {
            let message = cli::run_init(&cwd, name.as_deref())?;
            println!("{message}");
        }
        Commands::New { filename } => {
            let ws = Workspace::discover(&cwd).context("failed to find a PARA workspace")?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_new_with_filename() {
        let cli = Cli::parse_from(["tk", "new", "my-file"]);

        assert_eq!(
            cli.command,
            Commands::New {
                filename: Some("my-file".to_string())
            }
        );
    }

    #[test]
    fn parses_init_with_name() {
        let cli = Cli::parse_from(["tk", "init", "my-para"]);

        assert_eq!(
            cli.command,
            Commands::Init {
                name: Some("my-para".to_string())
            }
        );
    }

    #[test]
    fn parses_init_without_name() {
        let cli = Cli::parse_from(["tk", "init"]);

        assert_eq!(cli.command, Commands::Init { name: None });
    }
}
