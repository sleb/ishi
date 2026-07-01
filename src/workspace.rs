use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::category::Category;
use crate::config::{Config, ConfigError};

pub struct Workspace {
    pub root: PathBuf,
    pub config: Config,
}

#[derive(Debug, Error)]
pub enum WorkspaceError {
    #[error("no PARA workspace found in {start} or any parent directory")]
    NotFound { start: String },
    #[error("failed to load config")]
    Config(#[from] ConfigError),
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
}
