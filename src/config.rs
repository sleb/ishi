use std::fs;
use std::path::Path;

use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    /// Indexed by `Category as usize` (Inbox, Project, Area, Resource, Archive).
    pub category_dirs: [String; 5],
    pub default_extension: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            category_dirs: [
                "0-Inbox",
                "1-Projects",
                "2-Areas",
                "3-Resources",
                "4-Archive",
            ]
            .map(String::from),
            default_extension: "md".to_string(),
        }
    }
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("failed to read {path}")]
    Read {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse {path}")]
    Parse {
        path: String,
        #[source]
        source: toml::de::Error,
    },
}

#[derive(Debug, Default, Deserialize)]
struct TomlConfig {
    default_extension: Option<String>,
    category_dirs: Option<TomlCategoryDirs>,
}

#[derive(Debug, Default, Deserialize)]
struct TomlCategoryDirs {
    inbox: Option<String>,
    project: Option<String>,
    area: Option<String>,
    resource: Option<String>,
    archive: Option<String>,
}

impl Config {
    /// Reads `.tick.toml` from `path` if it exists, merging any present
    /// fields over [`Config::default`]. Returns the default untouched if
    /// `path` doesn't exist.
    pub fn load(path: &Path) -> Result<Config, ConfigError> {
        if !path.exists() {
            return Ok(Config::default());
        }
        let raw = fs::read_to_string(path).map_err(|source| ConfigError::Read {
            path: path.display().to_string(),
            source,
        })?;
        let parsed: TomlConfig = toml::from_str(&raw).map_err(|source| ConfigError::Parse {
            path: path.display().to_string(),
            source,
        })?;

        let mut config = Config::default();
        if let Some(ext) = parsed.default_extension {
            config.default_extension = ext;
        }
        if let Some(dirs) = parsed.category_dirs {
            if let Some(v) = dirs.inbox {
                config.category_dirs[0] = v;
            }
            if let Some(v) = dirs.project {
                config.category_dirs[1] = v;
            }
            if let Some(v) = dirs.area {
                config.category_dirs[2] = v;
            }
            if let Some(v) = dirs.resource {
                config.category_dirs[3] = v;
            }
            if let Some(v) = dirs.archive {
                config.category_dirs[4] = v;
            }
        }
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn missing_file_returns_default() {
        let dir = tempdir().unwrap();
        let path = dir.path().join(".tick.toml");

        let config = Config::load(&path).unwrap();

        assert_eq!(config, Config::default());
    }

    #[test]
    fn present_file_merges_over_default() {
        let dir = tempdir().unwrap();
        let path = dir.path().join(".tick.toml");
        fs::write(
            &path,
            r#"
            default_extension = "txt"

            [category_dirs]
            inbox = "Inbox"
            "#,
        )
        .unwrap();

        let config = Config::load(&path).unwrap();

        assert_eq!(config.default_extension, "txt");
        assert_eq!(config.category_dirs[0], "Inbox");
        assert_eq!(config.category_dirs[1], "1-Projects");
    }

    #[test]
    fn empty_file_returns_default_values() {
        let dir = tempdir().unwrap();
        let path = dir.path().join(".tick.toml");
        fs::write(&path, "").unwrap();

        let config = Config::load(&path).unwrap();

        assert_eq!(config, Config::default());
    }
}
