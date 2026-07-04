use std::env;

use anyhow::Context;
use clap::{Args, Parser, Subcommand};

use tick::category::Category;
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
    /// Capture a new note.
    New {
        filename: Option<String>,
        #[command(flatten)]
        category: NewCategory,
    },
    /// Scaffold a PARA system.
    Init { name: Option<String> },
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Args)]
#[group(multiple = false)]
struct NewCategory {
    /// Scaffold a project directory instead of an Inbox file.
    #[arg(long)]
    project: bool,
    /// Scaffold an area directory instead of an Inbox file.
    #[arg(long)]
    area: bool,
    /// Create a flat resource file instead of an Inbox file.
    #[arg(long)]
    resource: bool,
}

impl NewCategory {
    fn into_category(self) -> Category {
        if self.project {
            Category::Project
        } else if self.area {
            Category::Area
        } else if self.resource {
            Category::Resource
        } else {
            Category::Inbox
        }
    }
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
        Commands::New { filename, category } => {
            let ws = Workspace::discover(&cwd).context("failed to find a PARA workspace")?;
            if filename.is_none() {
                println!("Opening $EDITOR...");
            }
            let editor = RealEditor;
            let mut ui = TerminalUi;
            let path = cli::run_new(&ws, &editor, &mut ui, category.into_category(), filename)?;
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
                filename: Some("my-file".to_string()),
                category: NewCategory::default(),
            }
        );
    }

    #[test]
    fn parses_new_project() {
        let cli = Cli::parse_from(["tk", "new", "--project", "website-redesign"]);

        assert_eq!(
            cli.command,
            Commands::New {
                filename: Some("website-redesign".to_string()),
                category: NewCategory {
                    project: true,
                    ..Default::default()
                },
            }
        );
    }

    #[test]
    fn parses_new_area() {
        let cli = Cli::parse_from(["tk", "new", "--area", "health"]);

        assert_eq!(
            cli.command,
            Commands::New {
                filename: Some("health".to_string()),
                category: NewCategory {
                    area: true,
                    ..Default::default()
                },
            }
        );
    }

    #[test]
    fn parses_new_resource() {
        let cli = Cli::parse_from(["tk", "new", "--resource", "recipe-ideas"]);

        assert_eq!(
            cli.command,
            Commands::New {
                filename: Some("recipe-ideas".to_string()),
                category: NewCategory {
                    resource: true,
                    ..Default::default()
                },
            }
        );
    }

    #[test]
    fn rejects_conflicting_category_flags() {
        let result = Cli::try_parse_from(["tk", "new", "--project", "--area", "x"]);

        assert!(result.is_err());
    }

    #[test]
    fn into_category_defaults_to_inbox() {
        assert_eq!(NewCategory::default().into_category(), Category::Inbox);
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
