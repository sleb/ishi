use std::env;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use anyhow::Context;
use clap::{Args, CommandFactory, Parser, Subcommand};

use tick::category::{Category, Kind};
use tick::cli::{self, TerminalUi};
use tick::editor::RealEditor;
use tick::review;
use tick::workspace::Workspace;

#[derive(Parser)]
#[command(name = "tk", version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, PartialEq, Subcommand)]
enum Commands {
    /// Capture a new note.
    #[command(after_help = "\
Examples:
  tk new                     Open $EDITOR and suggest a filename from its content
  tk new meeting-notes       Create ./0-Inbox/meeting-notes.md directly
  tk new --project apollo    Scaffold a new project directory")]
    New {
        /// Name of the file to create (extension added automatically).
        /// Omit to open $EDITOR and be prompted with a suggested name.
        filename: Option<String>,
        #[command(flatten)]
        category: NewCategory,
        /// Accept the suggested filename without prompting.
        #[arg(short = 'y', long = "yes")]
        yes: bool,
    },
    /// Create (or open) today's daily note in the Inbox.
    Daily,
    /// Scaffold a PARA system.
    Init { name: Option<String> },
    /// View or manage the effective config.
    Config {
        #[command(subcommand)]
        action: Option<ConfigAction>,
    },
    /// List items in a category.
    #[command(alias = "ls")]
    List {
        category: ListCategory,
        filter: Option<String>,
    },
    /// Print a shell completion script for `tk` to stdout.
    Completions { shell: CompletionShell },
    /// Print a per-category summary of the PARA system.
    Status,
    /// Relocate an item to a different category.
    #[command(
        alias = "mv",
        after_help = "\
Examples:
  tk move meeting-notes project   File an Inbox item as a project
  tk mv apollo archive            Archive a project (prompts for a summary)
  tk move apollo archive --yes    Archive it, accepting the suggested summary"
    )]
    Move {
        /// Name of the item to relocate, as shown by `tk list`.
        name: String,
        /// Category to move the item into.
        target: MoveTarget,
        /// Accept the suggested archive summary without prompting.
        #[arg(short = 'y', long = "yes")]
        yes: bool,
    },
    /// File an item away — sugar for `tk move <item> archive`.
    #[command(after_help = "\
Examples:
  tk archive apollo         Archive \"apollo\" (prompts for a summary)
  tk archive apollo --yes   Archive it, accepting the suggested summary")]
    Archive {
        /// Name of the item to archive, as shown by `tk list`.
        name: String,
        /// Accept the suggested archive summary without prompting.
        #[arg(short = 'y', long = "yes")]
        yes: bool,
    },
    /// Walk every project and area, prompting keep/archive/skip.
    Review,
}

#[derive(Debug, Clone, Copy, PartialEq, clap::ValueEnum)]
enum MoveTarget {
    Project,
    Area,
    Resource,
    Inbox,
    Archive,
}

impl From<MoveTarget> for Category {
    fn from(target: MoveTarget) -> Self {
        match target {
            MoveTarget::Project => Category::Project,
            MoveTarget::Area => Category::Area,
            MoveTarget::Resource => Category::Resource,
            MoveTarget::Inbox => Category::Inbox,
            MoveTarget::Archive => Category::Archive,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, clap::ValueEnum)]
enum CompletionShell {
    Bash,
    Zsh,
    Fish,
    Powershell,
}

impl From<CompletionShell> for clap_complete::Shell {
    fn from(shell: CompletionShell) -> Self {
        match shell {
            CompletionShell::Bash => clap_complete::Shell::Bash,
            CompletionShell::Zsh => clap_complete::Shell::Zsh,
            CompletionShell::Fish => clap_complete::Shell::Fish,
            CompletionShell::Powershell => clap_complete::Shell::PowerShell,
        }
    }
}

/// Renders `shell`'s completion script for the `tk` CLI into a byte buffer.
fn render_completions(shell: CompletionShell) -> Vec<u8> {
    let mut buf = Vec::new();
    clap_complete::generate(
        clap_complete::Shell::from(shell),
        &mut Cli::command(),
        "tk",
        &mut buf,
    );
    buf
}

#[derive(Debug, Clone, Copy, PartialEq, clap::ValueEnum)]
enum ListCategory {
    Project,
    Area,
    Resource,
    Inbox,
    Archive,
}

impl From<ListCategory> for Category {
    fn from(category: ListCategory) -> Self {
        match category {
            ListCategory::Project => Category::Project,
            ListCategory::Area => Category::Area,
            ListCategory::Resource => Category::Resource,
            ListCategory::Inbox => Category::Inbox,
            ListCategory::Archive => Category::Archive,
        }
    }
}

#[derive(Debug, PartialEq, Subcommand)]
enum ConfigAction {
    /// Write a new `.tick.toml` (or `~/.tick.toml` with `-g`) populated
    /// with the built-in defaults.
    Init {
        #[arg(short = 'g', long = "global")]
        global: bool,
    },
    /// Open `.tick.toml` (or `~/.tick.toml` with `-g`) in `$EDITOR`,
    /// creating it with the default config first if it doesn't exist yet.
    Edit {
        #[arg(short = 'g', long = "global")]
        global: bool,
    },
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
    /// Create (or open) today's daily note instead of an Inbox file.
    #[arg(long, conflicts_with = "filename")]
    daily: bool,
}

impl NewCategory {
    fn into_kind(self) -> Kind {
        if self.project {
            Kind::Project
        } else if self.area {
            Kind::Area
        } else if self.resource {
            Kind::Resource
        } else if self.daily {
            Kind::Daily
        } else {
            Kind::Inbox
        }
    }
}

/// Resolves `~/.tick.toml`, or `None` if `$HOME` isn't set.
fn home_tick_toml() -> Option<PathBuf> {
    env::var_os("HOME").map(|home| PathBuf::from(home).join(".tick.toml"))
}

/// Computes the local-vs-global config target: the path to write/open, and
/// its human-readable display form (`"./.tick.toml"` or `"~/.tick.toml"`).
fn config_target(cwd: &Path, global: bool) -> anyhow::Result<(PathBuf, String)> {
    Ok(if global {
        let path = home_tick_toml().context("$HOME is not set")?;
        (path, "~/.tick.toml".to_string())
    } else {
        (cwd.join(".tick.toml"), "./.tick.toml".to_string())
    })
}

fn run_daily_command(ws: &Workspace) -> anyhow::Result<()> {
    if cli::daily_note_exists(ws) {
        println!("Opening $EDITOR...");
    }
    let editor = RealEditor;
    if let cli::DailyOutcome::Created(path) = cli::run_daily(ws, &editor)? {
        println!("Created {}", path.display());
        println!("Next: tk list to see it, or tk status for an overview.");
    }
    Ok(())
}

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let cli = Cli::parse();

    let cwd = env::current_dir().context("failed to determine current directory")?;
    let home_config = home_tick_toml();

    match cli.command {
        Commands::Init { name } => {
            let message = cli::run_init(&cwd, name.as_deref(), home_config.as_deref())?;
            println!("{message}");
            match name.as_deref() {
                Some(n) => println!("Next: cd {n} && tk new to capture your first note."),
                None => println!("Next: tk new to capture your first note."),
            }
        }
        Commands::Daily => {
            let ws = Workspace::discover(&cwd, home_config.as_deref())
                .context("failed to find a PARA workspace")?;
            run_daily_command(&ws)?;
        }
        Commands::New {
            filename: _,
            category,
            yes: _,
        } if category.into_kind() == Kind::Daily => {
            let ws = Workspace::discover(&cwd, home_config.as_deref())
                .context("failed to find a PARA workspace")?;
            run_daily_command(&ws)?;
        }
        Commands::New {
            filename,
            category,
            yes,
        } => {
            let ws = Workspace::discover(&cwd, home_config.as_deref())
                .context("failed to find a PARA workspace")?;
            if filename.is_none() {
                println!("Opening $EDITOR...");
            }
            let editor = RealEditor;
            let mut ui = TerminalUi;
            let path = cli::run_new(&ws, &editor, &mut ui, category.into_kind(), filename, yes)?;
            println!("Created {}", path.display());
            println!("Next: tk list to see it, or tk status for an overview.");
        }
        Commands::Config {
            action: Some(ConfigAction::Init { global }),
        } => {
            let (path, display) = config_target(&cwd, global)?;
            let message = cli::run_config_init(&path, &display)?;
            println!("{message}");
        }
        Commands::Config {
            action: Some(ConfigAction::Edit { global }),
        } => {
            let (path, display) = config_target(&cwd, global)?;
            if !path.exists() {
                println!("Created {display}");
            }
            println!("Opening $EDITOR...");
            let editor = RealEditor;
            cli::run_config_edit(&path, &editor)?;
        }
        Commands::Config { action: None } => {
            let (path, _display) = config_target(&cwd, false)?;
            let (config, origins) = tick::config::Config::resolve(&path, home_config.as_deref())?;
            print!("{}", tick::config::render_effective(&config, &origins));
        }
        Commands::List { category, filter } => {
            let ws = Workspace::discover(&cwd, home_config.as_deref())
                .context("failed to find a PARA workspace")?;
            let output = cli::run_list(&ws, category.into(), filter.as_deref())?;
            println!("{output}");
        }
        Commands::Completions { shell } => {
            io::stdout().write_all(&render_completions(shell))?;
        }
        Commands::Status => {
            let ws = Workspace::discover(&cwd, home_config.as_deref())
                .context("failed to find a PARA workspace")?;
            let output = cli::run_status(&ws)?;
            println!("{output}");
        }
        Commands::Move { name, target, yes } => {
            let ws = Workspace::discover(&cwd, home_config.as_deref())
                .context("failed to find a PARA workspace")?;
            let mut ui = TerminalUi;
            let message = cli::run_move(&ws, &mut ui, &name, target.into(), yes)?;
            println!("{message}");
        }
        Commands::Archive { name, yes } => {
            let ws = Workspace::discover(&cwd, home_config.as_deref())
                .context("failed to find a PARA workspace")?;
            let mut ui = TerminalUi;
            let message = cli::run_move(&ws, &mut ui, &name, Category::Archive, yes)?;
            println!("{message}");
        }
        Commands::Review => {
            let ws = Workspace::discover(&cwd, home_config.as_deref())
                .context("failed to find a PARA workspace")?;
            let mut ui = TerminalUi;
            review::run(&ws, &mut ui)?;
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
                yes: false,
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
                yes: false,
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
                yes: false,
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
                yes: false,
            }
        );
    }

    #[test]
    fn parses_new_yes_flag() {
        let cli = Cli::parse_from(["tk", "new", "--yes"]);

        assert_eq!(
            cli.command,
            Commands::New {
                filename: None,
                category: NewCategory::default(),
                yes: true,
            }
        );
    }

    #[test]
    fn parses_new_yes_short_flag() {
        let cli = Cli::parse_from(["tk", "new", "-y"]);

        assert_eq!(
            cli.command,
            Commands::New {
                filename: None,
                category: NewCategory::default(),
                yes: true,
            }
        );
    }

    #[test]
    fn rejects_conflicting_category_flags() {
        let result = Cli::try_parse_from(["tk", "new", "--project", "--area", "x"]);

        assert!(result.is_err());
    }

    #[test]
    fn parses_new_daily() {
        let cli = Cli::parse_from(["tk", "new", "--daily"]);

        assert_eq!(
            cli.command,
            Commands::New {
                filename: None,
                category: NewCategory {
                    daily: true,
                    ..Default::default()
                },
                yes: false,
            }
        );
    }

    #[test]
    fn rejects_new_daily_with_filename() {
        let result = Cli::try_parse_from(["tk", "new", "--daily", "x"]);

        assert!(result.is_err());
    }

    #[test]
    fn rejects_new_daily_with_project() {
        let result = Cli::try_parse_from(["tk", "new", "--daily", "--project"]);

        assert!(result.is_err());
    }

    #[test]
    fn parses_daily() {
        let cli = Cli::parse_from(["tk", "daily"]);

        assert_eq!(cli.command, Commands::Daily);
    }

    #[test]
    fn parses_status() {
        let cli = Cli::parse_from(["tk", "status"]);

        assert_eq!(cli.command, Commands::Status);
    }

    #[test]
    fn parses_move() {
        let cli = Cli::parse_from(["tk", "move", "my-file", "project"]);

        assert_eq!(
            cli.command,
            Commands::Move {
                name: "my-file".to_string(),
                target: MoveTarget::Project,
                yes: false,
            }
        );
    }

    #[test]
    fn parses_mv_alias() {
        let cli = Cli::parse_from(["tk", "mv", "my-file", "archive"]);

        assert_eq!(
            cli.command,
            Commands::Move {
                name: "my-file".to_string(),
                target: MoveTarget::Archive,
                yes: false,
            }
        );
    }

    #[test]
    fn parses_archive() {
        let cli = Cli::parse_from(["tk", "archive", "my-file"]);

        assert_eq!(
            cli.command,
            Commands::Archive {
                name: "my-file".to_string(),
                yes: false,
            }
        );
    }

    #[test]
    fn parses_archive_yes_flag() {
        let cli = Cli::parse_from(["tk", "archive", "my-file", "--yes"]);

        assert_eq!(
            cli.command,
            Commands::Archive {
                name: "my-file".to_string(),
                yes: true,
            }
        );
    }

    #[test]
    fn parses_move_yes_flag() {
        let cli = Cli::parse_from(["tk", "move", "my-file", "archive", "-y"]);

        assert_eq!(
            cli.command,
            Commands::Move {
                name: "my-file".to_string(),
                target: MoveTarget::Archive,
                yes: true,
            }
        );
    }

    #[test]
    fn rejects_archive_with_category_argument() {
        let result = Cli::try_parse_from(["tk", "archive", "my-file", "archive"]);

        assert!(result.is_err());
    }

    #[test]
    fn parses_review() {
        let cli = Cli::parse_from(["tk", "review"]);

        assert_eq!(cli.command, Commands::Review);
    }

    #[test]
    fn rejects_daily_with_filename() {
        let result = Cli::try_parse_from(["tk", "daily", "x"]);

        assert!(result.is_err());
    }

    #[test]
    fn into_kind_defaults_to_inbox() {
        assert_eq!(NewCategory::default().into_kind(), Kind::Inbox);
    }

    #[test]
    fn into_kind_maps_every_flag() {
        assert_eq!(
            NewCategory {
                project: true,
                ..Default::default()
            }
            .into_kind(),
            Kind::Project
        );
        assert_eq!(
            NewCategory {
                area: true,
                ..Default::default()
            }
            .into_kind(),
            Kind::Area
        );
        assert_eq!(
            NewCategory {
                resource: true,
                ..Default::default()
            }
            .into_kind(),
            Kind::Resource
        );
        assert_eq!(
            NewCategory {
                daily: true,
                ..Default::default()
            }
            .into_kind(),
            Kind::Daily
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

    #[test]
    fn parses_config_init_with_no_flag() {
        let cli = Cli::parse_from(["tk", "config", "init"]);

        assert_eq!(
            cli.command,
            Commands::Config {
                action: Some(ConfigAction::Init { global: false })
            }
        );
    }

    #[test]
    fn parses_config_init_global_short_flag() {
        let cli = Cli::parse_from(["tk", "config", "init", "-g"]);

        assert_eq!(
            cli.command,
            Commands::Config {
                action: Some(ConfigAction::Init { global: true })
            }
        );
    }

    #[test]
    fn parses_config_init_global_long_flag() {
        let cli = Cli::parse_from(["tk", "config", "init", "--global"]);

        assert_eq!(
            cli.command,
            Commands::Config {
                action: Some(ConfigAction::Init { global: true })
            }
        );
    }

    #[test]
    fn parses_config_edit_with_no_flag() {
        let cli = Cli::parse_from(["tk", "config", "edit"]);

        assert_eq!(
            cli.command,
            Commands::Config {
                action: Some(ConfigAction::Edit { global: false })
            }
        );
    }

    #[test]
    fn parses_config_edit_global_short_flag() {
        let cli = Cli::parse_from(["tk", "config", "edit", "-g"]);

        assert_eq!(
            cli.command,
            Commands::Config {
                action: Some(ConfigAction::Edit { global: true })
            }
        );
    }

    #[test]
    fn parses_config_edit_global_long_flag() {
        let cli = Cli::parse_from(["tk", "config", "edit", "--global"]);

        assert_eq!(
            cli.command,
            Commands::Config {
                action: Some(ConfigAction::Edit { global: true })
            }
        );
    }

    #[test]
    fn parses_config_bare_as_action_none() {
        let cli = Cli::parse_from(["tk", "config"]);

        assert_eq!(cli.command, Commands::Config { action: None });
    }

    #[test]
    fn parses_list_project() {
        let cli = Cli::parse_from(["tk", "list", "project"]);

        assert_eq!(
            cli.command,
            Commands::List {
                category: ListCategory::Project,
                filter: None
            }
        );
    }

    #[test]
    fn parses_list_project_with_filter() {
        let cli = Cli::parse_from(["tk", "list", "project", "web"]);

        assert_eq!(
            cli.command,
            Commands::List {
                category: ListCategory::Project,
                filter: Some("web".into())
            }
        );
    }

    #[test]
    fn parses_list_area() {
        let cli = Cli::parse_from(["tk", "list", "area"]);

        assert_eq!(
            cli.command,
            Commands::List {
                category: ListCategory::Area,
                filter: None
            }
        );
    }

    #[test]
    fn parses_list_resource() {
        let cli = Cli::parse_from(["tk", "list", "resource"]);

        assert_eq!(
            cli.command,
            Commands::List {
                category: ListCategory::Resource,
                filter: None
            }
        );
    }

    #[test]
    fn parses_list_inbox() {
        let cli = Cli::parse_from(["tk", "list", "inbox"]);

        assert_eq!(
            cli.command,
            Commands::List {
                category: ListCategory::Inbox,
                filter: None
            }
        );
    }

    #[test]
    fn parses_list_archive() {
        let cli = Cli::parse_from(["tk", "list", "archive"]);

        assert_eq!(
            cli.command,
            Commands::List {
                category: ListCategory::Archive,
                filter: None
            }
        );
    }

    #[test]
    fn parses_completions_bash() {
        let cli = Cli::parse_from(["tk", "completions", "bash"]);

        assert_eq!(
            cli.command,
            Commands::Completions {
                shell: CompletionShell::Bash
            }
        );
    }

    #[test]
    fn parses_completions_zsh() {
        let cli = Cli::parse_from(["tk", "completions", "zsh"]);

        assert_eq!(
            cli.command,
            Commands::Completions {
                shell: CompletionShell::Zsh
            }
        );
    }

    #[test]
    fn parses_completions_fish() {
        let cli = Cli::parse_from(["tk", "completions", "fish"]);

        assert_eq!(
            cli.command,
            Commands::Completions {
                shell: CompletionShell::Fish
            }
        );
    }

    #[test]
    fn parses_completions_powershell() {
        let cli = Cli::parse_from(["tk", "completions", "powershell"]);

        assert_eq!(
            cli.command,
            Commands::Completions {
                shell: CompletionShell::Powershell
            }
        );
    }

    #[test]
    fn rejects_unsupported_completions_shell() {
        let result = Cli::try_parse_from(["tk", "completions", "tcsh"]);

        assert!(result.is_err());
    }

    #[test]
    fn rejects_missing_completions_shell() {
        let result = Cli::try_parse_from(["tk", "completions"]);

        assert!(result.is_err());
    }

    #[test]
    fn renders_non_empty_completions_for_every_shell() {
        for shell in [
            CompletionShell::Bash,
            CompletionShell::Zsh,
            CompletionShell::Fish,
            CompletionShell::Powershell,
        ] {
            assert!(!render_completions(shell).is_empty());
        }
    }

    #[test]
    fn completions_cover_every_top_level_command() {
        let script = render_completions(CompletionShell::Bash);
        let script = String::from_utf8(script).unwrap();

        for command in ["init", "new", "daily", "list", "config", "completions"] {
            assert!(
                script.contains(command),
                "expected script to contain {command}"
            );
        }
    }
}
