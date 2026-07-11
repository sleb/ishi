use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::category::Category;
use crate::config::{Config, ConfigError};

#[derive(Debug)]
pub struct Workspace {
    pub root: PathBuf,
    pub config: Config,
}

pub struct InitReport {
    pub created: Vec<String>,
}

#[derive(Debug, Error)]
pub enum WorkspaceError {
    #[error(
        "no PARA workspace found in {start} or any parent directory ({missing}). Run \"ishi init\" to create one here."
    )]
    NotFound { start: String, missing: String },
    #[error("failed to load config")]
    Config(#[from] ConfigError),
    #[error("{path} already exists and is not a directory")]
    NotAParaSystem { path: String },
    #[error(transparent)]
    Io(#[from] io::Error),
}

/// Describes which of the default category directories are absent from
/// `dir`, for `WorkspaceError::NotFound`'s message — e.g. `"no 0-Inbox
/// directory found"` or `"no 0-Inbox, 4-Archive directories found"`. `dir`
/// is guaranteed to be missing at least one when this is called, since
/// `discover` only reaches `NotFound` after failing to match `dir` itself.
fn missing_category_dirs(dir: &Path) -> String {
    let defaults = Config::default();
    let missing: Vec<&str> = defaults
        .category_dirs
        .iter()
        .filter(|name| !dir.join(name).is_dir())
        .map(String::as_str)
        .collect();

    match missing.as_slice() {
        [] => "no .ishi.toml found".to_string(),
        [one] => format!("no {one} directory found"),
        many => format!("no {} directories found", many.join(", ")),
    }
}

/// Returns `Ok(())` unless `target` exists and is a regular file (or other
/// non-directory), in which case it would collide with a scaffolded PARA
/// system.
pub fn check_collision(target: &Path) -> Result<(), WorkspaceError> {
    if target.exists() && !target.is_dir() {
        return Err(WorkspaceError::NotAParaSystem {
            path: target.display().to_string(),
        });
    }
    Ok(())
}

/// Scaffolds a PARA system at `target`, creating `target` itself and any
/// missing category directories under it. Existing directories (and their
/// contents) are left untouched.
pub fn init(target: &Path) -> Result<InitReport, WorkspaceError> {
    check_collision(target)?;
    fs::create_dir_all(target)?;

    let mut created = Vec::new();
    for name in &Config::default().category_dirs {
        let dir = target.join(name);
        if !dir.exists() {
            fs::create_dir_all(&dir)?;
            created.push(name.clone());
        }
    }

    Ok(InitReport { created })
}

pub struct EditorExcludeReport {
    pub zed_created: bool,
    pub vscode_created: bool,
}

pub struct ClaudeMdReport {
    pub created: bool,
}

fn zed_settings_json(archive_dir: &str) -> String {
    format!("{{\n  \"file_scan_exclude\": [\"{archive_dir}\"]\n}}\n")
}

fn vscode_settings_json(archive_dir: &str) -> String {
    format!(
        "{{\n  \"files.exclude\": {{\n    \"{archive_dir}\": true\n  }},\n  \"search.exclude\": {{\n    \"{archive_dir}\": true\n  }}\n}}\n"
    )
}

fn claude_md_content(archive_dir: &str) -> String {
    format!(
        "# CLAUDE.md\n\nDo not read files under `{archive_dir}` unless the user explicitly asks or there's a strong, specific reason to.\n"
    )
}

/// Writes `.zed/settings.json` and/or `.vscode/settings.json` under
/// `target`, each independently, only if it doesn't already exist. Both
/// name `archive_dir` in their exclude entries. Never touches a file that
/// already exists (even one with unrelated contents) — creation is the
/// only mutation this ever performs, matching `config::init`'s
/// create-only contract.
pub fn write_editor_excludes(
    target: &Path,
    archive_dir: &str,
) -> Result<EditorExcludeReport, WorkspaceError> {
    let zed_path = target.join(".zed").join("settings.json");
    let zed_created = if zed_path.exists() {
        false
    } else {
        fs::create_dir_all(zed_path.parent().unwrap())?;
        fs::write(&zed_path, zed_settings_json(archive_dir))?;
        true
    };

    let vscode_path = target.join(".vscode").join("settings.json");
    let vscode_created = if vscode_path.exists() {
        false
    } else {
        fs::create_dir_all(vscode_path.parent().unwrap())?;
        fs::write(&vscode_path, vscode_settings_json(archive_dir))?;
        true
    };

    Ok(EditorExcludeReport {
        zed_created,
        vscode_created,
    })
}

/// Writes `CLAUDE.md` at `target` with an archive-skip instruction naming
/// `archive_dir`, only if `CLAUDE.md` doesn't already exist. Never
/// modifies an existing `CLAUDE.md`, with or without the instruction
/// already present — parsing/merging unknown-shape Markdown is explicitly
/// out of scope per `init.md` 006.
pub fn write_claude_md(target: &Path, archive_dir: &str) -> Result<ClaudeMdReport, WorkspaceError> {
    let path = target.join("CLAUDE.md");
    let created = if path.exists() {
        false
    } else {
        fs::create_dir_all(target)?;
        fs::write(&path, claude_md_content(archive_dir))?;
        true
    };

    Ok(ClaudeMdReport { created })
}

impl Workspace {
    /// Walks up from `start` through ancestors, stopping at the first
    /// directory containing `.ishi.toml` or all five default-named
    /// category directories. `home_config`, if given, is layered in as the
    /// user-level config on both branches.
    pub fn discover(start: &Path, home_config: Option<&Path>) -> Result<Workspace, WorkspaceError> {
        for dir in start.ancestors() {
            let ishi_toml = dir.join(".ishi.toml");
            if ishi_toml.exists() {
                let (config, _origins) = Config::resolve(&ishi_toml, home_config)?;
                return Ok(Workspace {
                    root: dir.to_path_buf(),
                    config,
                });
            }

            let default_config = Config::default();
            if default_config
                .category_dirs
                .iter()
                .all(|name| dir.join(name).is_dir())
            {
                let (config, _origins) = Config::resolve(&dir.join(".ishi.toml"), home_config)?;
                return Ok(Workspace {
                    root: dir.to_path_buf(),
                    config,
                });
            }
        }

        Err(WorkspaceError::NotFound {
            start: start.display().to_string(),
            missing: missing_category_dirs(start),
        })
    }

    pub fn category_dir(&self, category: Category) -> PathBuf {
        self.root
            .join(&self.config.category_dirs[category as usize])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn create_category_dirs(root: &Path) {
        for name in Config::default().category_dirs {
            fs::create_dir_all(root.join(name)).unwrap();
        }
    }

    #[test]
    fn discovers_root_via_ishi_toml() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join(".ishi.toml"), "").unwrap();
        let nested = dir.path().join("a/b");
        fs::create_dir_all(&nested).unwrap();

        let ws = Workspace::discover(&nested, None).unwrap();

        assert_eq!(ws.root, dir.path());
    }

    #[test]
    fn discovers_root_via_bare_category_dirs() {
        let dir = tempdir().unwrap();
        create_category_dirs(dir.path());
        let nested = dir.path().join("a/b");
        fs::create_dir_all(&nested).unwrap();

        let ws = Workspace::discover(&nested, None).unwrap();

        assert_eq!(ws.root, dir.path());
    }

    #[test]
    fn returns_not_found_outside_any_workspace() {
        let dir = tempdir().unwrap();

        let result = Workspace::discover(dir.path(), None);

        assert!(matches!(result, Err(WorkspaceError::NotFound { .. })));
    }

    #[test]
    fn not_found_message_names_missing_directories_and_suggests_init() {
        let dir = tempdir().unwrap();

        let err = Workspace::discover(dir.path(), None).unwrap_err();

        let message = err.to_string();
        assert!(message.contains("0-Inbox, 1-Projects, 2-Areas, 3-Resources, 4-Archive"));
        assert!(message.contains("Run \"ishi init\" to create one here."));
    }

    #[test]
    fn not_found_message_names_only_the_missing_directory() {
        let dir = tempdir().unwrap();
        create_category_dirs(dir.path());
        fs::remove_dir(dir.path().join("0-Inbox")).unwrap();

        let err = Workspace::discover(dir.path(), None).unwrap_err();

        assert!(err.to_string().contains("no 0-Inbox directory found"));
    }

    #[test]
    fn discover_layers_user_config_when_no_local_file_present() {
        let dir = tempdir().unwrap();
        create_category_dirs(dir.path());
        let home_config = dir.path().join("home.ishi.toml");
        fs::write(
            &home_config,
            r#"
            [templates]
            note = "user note template"
            "#,
        )
        .unwrap();

        let ws = Workspace::discover(dir.path(), Some(&home_config)).unwrap();

        assert_eq!(ws.config.templates.note, "user note template");
    }

    #[test]
    fn discover_layers_both_user_and_local_config() {
        let dir = tempdir().unwrap();
        fs::write(
            dir.path().join(".ishi.toml"),
            r#"
            [folders]
            archive = "Archive"
            "#,
        )
        .unwrap();
        let home_config = dir.path().join("home.ishi.toml");
        fs::write(
            &home_config,
            r#"
            [templates]
            daily = "user daily template"
            "#,
        )
        .unwrap();

        let ws = Workspace::discover(dir.path(), Some(&home_config)).unwrap();

        assert_eq!(ws.config.category_dirs[4], "Archive");
        assert_eq!(ws.config.templates.daily, "user daily template");
    }

    #[test]
    fn category_dir_joins_root_and_config_dir_name() {
        let dir = tempdir().unwrap();
        let ws = Workspace {
            root: dir.path().to_path_buf(),
            config: Config::default(),
        };

        assert_eq!(ws.category_dir(Category::Inbox), dir.path().join("0-Inbox"));
    }

    #[test]
    fn init_bare_creates_all_five_dirs() {
        let dir = tempdir().unwrap();

        let report = init(dir.path()).unwrap();

        assert_eq!(report.created, Config::default().category_dirs.to_vec());
        for name in Config::default().category_dirs {
            assert!(dir.path().join(name).is_dir());
        }
    }

    #[test]
    fn init_named_creates_target_and_category_dirs() {
        let dir = tempdir().unwrap();
        let target = dir.path().join("my-para");

        let report = init(&target).unwrap();

        assert_eq!(report.created, Config::default().category_dirs.to_vec());
        for name in Config::default().category_dirs {
            assert!(target.join(name).is_dir());
        }
    }

    #[test]
    fn init_on_complete_system_is_a_no_op() {
        let dir = tempdir().unwrap();
        create_category_dirs(dir.path());
        let entries_before = fs::read_dir(dir.path()).unwrap().count();

        let report = init(dir.path()).unwrap();

        assert!(report.created.is_empty());
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), entries_before);
    }

    #[test]
    fn init_on_partial_system_fills_gaps_only() {
        let dir = tempdir().unwrap();
        let inbox = dir.path().join("0-Inbox");
        fs::create_dir_all(&inbox).unwrap();
        let marker = inbox.join("marker.txt");
        fs::write(&marker, "keep me").unwrap();

        let report = init(dir.path()).unwrap();

        assert_eq!(
            report.created,
            vec!["1-Projects", "2-Areas", "3-Resources", "4-Archive"]
        );
        assert_eq!(fs::read_to_string(&marker).unwrap(), "keep me");
    }

    #[test]
    fn check_collision_errors_on_existing_file() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("existing-file");
        fs::write(&file, "").unwrap();

        let result = check_collision(&file);

        assert!(matches!(result, Err(WorkspaceError::NotAParaSystem { .. })));
    }

    #[test]
    fn check_collision_ok_on_directory_with_unrelated_contents() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("notes.txt"), "").unwrap();

        assert!(check_collision(dir.path()).is_ok());

        let report = init(dir.path()).unwrap();
        assert_eq!(report.created, Config::default().category_dirs.to_vec());
        assert!(dir.path().join("notes.txt").exists());
    }

    #[test]
    fn check_collision_ok_on_missing_path() {
        let dir = tempdir().unwrap();
        let missing = dir.path().join("does-not-exist");

        assert!(check_collision(&missing).is_ok());
    }

    #[test]
    fn check_collision_ok_on_existing_directory_regardless_of_contents() {
        let dir = tempdir().unwrap();

        assert!(check_collision(dir.path()).is_ok());
    }

    #[test]
    fn write_editor_excludes_creates_zed_settings_when_absent() {
        let dir = tempdir().unwrap();

        let report = write_editor_excludes(dir.path(), "4-Archive").unwrap();

        assert!(report.zed_created);
        let content = fs::read_to_string(dir.path().join(".zed/settings.json")).unwrap();
        assert!(content.contains("file_scan_exclude"));
        assert!(content.contains("4-Archive"));
    }

    #[test]
    fn write_editor_excludes_creates_vscode_settings_when_absent() {
        let dir = tempdir().unwrap();

        let report = write_editor_excludes(dir.path(), "4-Archive").unwrap();

        assert!(report.vscode_created);
        let content = fs::read_to_string(dir.path().join(".vscode/settings.json")).unwrap();
        assert!(content.contains("files.exclude"));
        assert!(content.contains("search.exclude"));
        assert!(content.contains("4-Archive"));
    }

    #[test]
    fn write_editor_excludes_leaves_existing_zed_settings_untouched() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join(".zed")).unwrap();
        fs::write(dir.path().join(".zed/settings.json"), "arbitrary").unwrap();

        let report = write_editor_excludes(dir.path(), "4-Archive").unwrap();

        assert!(!report.zed_created);
        assert_eq!(
            fs::read_to_string(dir.path().join(".zed/settings.json")).unwrap(),
            "arbitrary"
        );
    }

    #[test]
    fn write_editor_excludes_leaves_existing_vscode_settings_untouched() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join(".vscode")).unwrap();
        fs::write(dir.path().join(".vscode/settings.json"), "arbitrary").unwrap();

        let report = write_editor_excludes(dir.path(), "4-Archive").unwrap();

        assert!(!report.vscode_created);
        assert_eq!(
            fs::read_to_string(dir.path().join(".vscode/settings.json")).unwrap(),
            "arbitrary"
        );
    }

    #[test]
    fn write_editor_excludes_uses_custom_archive_dir_name() {
        let dir = tempdir().unwrap();

        write_editor_excludes(dir.path(), "9-Attic").unwrap();

        let zed = fs::read_to_string(dir.path().join(".zed/settings.json")).unwrap();
        let vscode = fs::read_to_string(dir.path().join(".vscode/settings.json")).unwrap();
        assert!(zed.contains("9-Attic"));
        assert!(vscode.contains("9-Attic"));
    }

    #[test]
    fn write_editor_excludes_zed_and_vscode_are_independent() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join(".zed")).unwrap();
        fs::write(dir.path().join(".zed/settings.json"), "arbitrary").unwrap();

        let report = write_editor_excludes(dir.path(), "4-Archive").unwrap();

        assert!(!report.zed_created);
        assert!(report.vscode_created);
        assert!(dir.path().join(".vscode/settings.json").exists());
    }

    #[test]
    fn write_claude_md_creates_when_absent() {
        let dir = tempdir().unwrap();

        let report = write_claude_md(dir.path(), "4-Archive").unwrap();

        assert!(report.created);
        let content = fs::read_to_string(dir.path().join("CLAUDE.md")).unwrap();
        assert!(content.contains("4-Archive"));
        assert!(content.contains("unless"));
    }

    #[test]
    fn write_claude_md_leaves_existing_file_untouched() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("CLAUDE.md"), "arbitrary").unwrap();

        let report = write_claude_md(dir.path(), "4-Archive").unwrap();

        assert!(!report.created);
        assert_eq!(
            fs::read_to_string(dir.path().join("CLAUDE.md")).unwrap(),
            "arbitrary"
        );
    }

    #[test]
    fn write_claude_md_uses_custom_archive_dir_name() {
        let dir = tempdir().unwrap();

        write_claude_md(dir.path(), "9-Attic").unwrap();

        let content = fs::read_to_string(dir.path().join("CLAUDE.md")).unwrap();
        assert!(content.contains("9-Attic"));
    }
}
