use std::fs;
use std::path::Path;

use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    /// Indexed by `Category as usize` (Inbox, Project, Area, Resource, Archive).
    pub category_dirs: [String; 5],
    pub default_extension: String,
    pub templates: Templates,
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
            templates: Templates::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Templates {
    pub note: String,
    pub daily: String,
    pub project: String,
    pub area: String,
    pub resource: String,
}

impl Default for Templates {
    fn default() -> Self {
        Self {
            note: "---\nlast_updated: {{date}}\n---\n# {{cursor}}{{title}}\n".to_string(),
            daily: "---\ndate: {{date}}\nlast_updated: {{date}}\n---\n# {{date}}\n\n## Tasks\n\n[ ] -\n\n## Notes\n\n{{cursor}}\n".to_string(),
            project:
                "---\nlast_updated: {{date}}\n---\n\n# {{cursor}}{{title}}\n\nStatus: active\n"
                    .to_string(),
            area: "---\nlast_updated: {{date}}\n---\n\n# {{cursor}}{{title}}\n\nStandard:\n"
                .to_string(),
            resource: "---\nlast_updated: {{date}}\n---\n\n# {{cursor}}{{title}}\n".to_string(),
        }
    }
}

impl Templates {
    /// Maps a `Kind` to the template used when creating that kind of item.
    /// Total — there's no `Kind::Archive` to be missing a template for.
    pub fn for_kind(&self, kind: crate::category::Kind) -> &str {
        use crate::category::Kind;
        match kind {
            Kind::Inbox => &self.note,
            Kind::Daily => &self.daily,
            Kind::Project => &self.project,
            Kind::Area => &self.area,
            Kind::Resource => &self.resource,
        }
    }
}

/// Fills in `{{date}}`, `{{title}}`, `{{time}}`, and `{{uuid}}` in
/// `template`. Leaves `{{cursor}}` untouched — interpreting that marker
/// (positioning the editor's cursor, then stripping it) is `Editor`'s job,
/// not the renderer's.
pub fn render(template: &str, title: &str, date: &str, time: &str, uuid: &str) -> String {
    template
        .replace("{{date}}", date)
        .replace("{{title}}", title)
        .replace("{{time}}", time)
        .replace("{{uuid}}", uuid)
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
            // Order matches `Category as usize` (Inbox, Project, Area, Resource, Archive).
            let overrides = [
                dirs.inbox,
                dirs.project,
                dirs.area,
                dirs.resource,
                dirs.archive,
            ];
            for (i, value) in overrides.into_iter().enumerate() {
                if let Some(value) = value {
                    config.category_dirs[i] = value;
                }
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
    fn render_fills_date_and_title_but_leaves_cursor_marker() {
        let template = "---\nlast_updated: {{date}}\n---\n# {{cursor}}{{title}}\n";

        let rendered = render(template, "", "2026-07-03", "14:32", "uuid-value");

        assert_eq!(
            rendered,
            "---\nlast_updated: 2026-07-03\n---\n# {{cursor}}\n"
        );
    }

    #[test]
    fn render_fills_time() {
        let template = "captured at {{time}}";

        let rendered = render(template, "", "2026-07-03", "14:32", "uuid-value");

        assert_eq!(rendered, "captured at 14:32");
    }

    #[test]
    fn render_fills_uuid() {
        let template = "id: {{uuid}}";

        let rendered = render(
            template,
            "",
            "2026-07-03",
            "14:32",
            "f47ac10b-58cc-4372-a567-0e02b2c3d479",
        );

        assert_eq!(rendered, "id: f47ac10b-58cc-4372-a567-0e02b2c3d479");
    }

    #[test]
    fn render_fills_all_markers_together_leaving_cursor_marker() {
        let template =
            "date={{date}} title={{title}} time={{time}} uuid={{uuid}} cursor={{cursor}}";

        let rendered = render(
            template,
            "My Title",
            "2026-07-03",
            "14:32",
            "f47ac10b-58cc-4372-a567-0e02b2c3d479",
        );

        assert_eq!(
            rendered,
            "date=2026-07-03 title=My Title time=14:32 uuid=f47ac10b-58cc-4372-a567-0e02b2c3d479 cursor={{cursor}}"
        );
    }

    #[test]
    fn for_kind_maps_to_matching_template() {
        use crate::category::Kind;

        let templates = Templates::default();

        assert_eq!(templates.for_kind(Kind::Inbox), templates.note);
        assert_eq!(templates.for_kind(Kind::Daily), templates.daily);
        assert_eq!(templates.for_kind(Kind::Project), templates.project);
        assert_eq!(templates.for_kind(Kind::Area), templates.area);
        assert_eq!(templates.for_kind(Kind::Resource), templates.resource);
    }

    #[test]
    fn daily_template_default_matches_readme() {
        let templates = Templates::default();

        assert!(templates.daily.contains("## Tasks"));
        assert!(templates.daily.contains("## Notes"));
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
