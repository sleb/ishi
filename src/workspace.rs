use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::category::Category;
use crate::config::{Config, ConfigError};

pub struct Workspace {
    pub root: PathBuf,
    pub config: Config,
}

pub struct InitReport {
    pub created: Vec<String>,
}

#[derive(Debug, Error)]
pub enum WorkspaceError {
    #[error("no PARA workspace found in {start} or any parent directory")]
    NotFound { start: String },
    #[error("failed to load config")]
    Config(#[from] ConfigError),
    #[error("{path} already exists and is not a directory")]
    NotAParaSystem { path: String },
    #[error(transparent)]
    Io(#[from] io::Error),
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

impl Workspace {
    /// Walks up from `start` through ancestors, stopping at the first
    /// directory containing `.tick.toml` or all five default-named
    /// category directories.
    pub fn discover(start: &Path) -> Result<Workspace, WorkspaceError> {
        for dir in start.ancestors() {
            let tick_toml = dir.join(".tick.toml");
            if tick_toml.exists() {
                let config = Config::load(&tick_toml)?;
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
                return Ok(Workspace {
                    root: dir.to_path_buf(),
                    config: default_config,
                });
            }
        }

        Err(WorkspaceError::NotFound {
            start: start.display().to_string(),
        })
    }

    pub fn category_dir(&self, category: Category) -> PathBuf {
        let index = match category {
            Category::Inbox => 0,
            Category::Project => 1,
            Category::Area => 2,
            Category::Resource => 3,
            Category::Archive => 4,
        };
        self.root.join(&self.config.category_dirs[index])
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
    fn discovers_root_via_tick_toml() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join(".tick.toml"), "").unwrap();
        let nested = dir.path().join("a/b");
        fs::create_dir_all(&nested).unwrap();

        let ws = Workspace::discover(&nested).unwrap();

        assert_eq!(ws.root, dir.path());
    }

    #[test]
    fn discovers_root_via_bare_category_dirs() {
        let dir = tempdir().unwrap();
        create_category_dirs(dir.path());
        let nested = dir.path().join("a/b");
        fs::create_dir_all(&nested).unwrap();

        let ws = Workspace::discover(&nested).unwrap();

        assert_eq!(ws.root, dir.path());
    }

    #[test]
    fn returns_not_found_outside_any_workspace() {
        let dir = tempdir().unwrap();

        let result = Workspace::discover(dir.path());

        assert!(matches!(result, Err(WorkspaceError::NotFound { .. })));
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
}
