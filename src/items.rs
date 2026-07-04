use std::fs;
use std::io;
use std::path::PathBuf;

use thiserror::Error;

use crate::category::Category;
use crate::workspace::Workspace;

#[derive(Debug, Error)]
pub enum ItemsError {
    #[error(transparent)]
    Io(#[from] io::Error),
}

/// Computes the path `create` would write to, without touching the
/// filesystem — the directory-vs-flat-file branch, factored out so
/// callers can check existence (`cli::run_daily`) before deciding whether
/// to create or reopen.
pub fn item_path(ws: &Workspace, category: Category, name: &str) -> PathBuf {
    let category_dir = ws.category_dir(category);
    if category.is_directory_style() {
        category_dir
            .join(name)
            .join(format!("index.{}", ws.config.default_extension))
    } else {
        category_dir.join(with_extension(name, &ws.config.default_extension))
    }
}

/// Creates a flat file or a scaffolded `dir/index.md`, appending the
/// default extension to `name` if it has none, and writing `content`
/// into it. Returns the path created (the `index.md` path for
/// directory-style categories).
pub fn create(
    ws: &Workspace,
    category: Category,
    name: &str,
    content: &str,
) -> Result<PathBuf, ItemsError> {
    let path = item_path(ws, category, name);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&path, content)?;
    Ok(path)
}

fn with_extension(name: &str, default_extension: &str) -> String {
    if name.contains('.') {
        name.to_string()
    } else {
        format!("{name}.{default_extension}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use tempfile::tempdir;

    fn workspace(root: &std::path::Path) -> Workspace {
        Workspace {
            root: root.to_path_buf(),
            config: Config::default(),
        }
    }

    #[test]
    fn creates_inbox_file_with_default_extension() {
        let dir = tempdir().unwrap();
        let ws = workspace(dir.path());

        let path = create(&ws, Category::Inbox, "my-note", "hello").unwrap();

        assert_eq!(path, dir.path().join("0-Inbox/my-note.md"));
        assert_eq!(fs::read_to_string(&path).unwrap(), "hello");
    }

    #[test]
    fn does_not_double_append_extension_when_already_present() {
        let dir = tempdir().unwrap();
        let ws = workspace(dir.path());

        let path = create(&ws, Category::Inbox, "my-note.md", "hello").unwrap();

        assert_eq!(path, dir.path().join("0-Inbox/my-note.md"));
    }

    #[test]
    fn item_path_for_flat_category() {
        let dir = tempdir().unwrap();
        let ws = workspace(dir.path());

        let path = item_path(&ws, Category::Inbox, "2026-07-04");

        assert_eq!(path, dir.path().join("0-Inbox/2026-07-04.md"));
    }

    #[test]
    fn item_path_for_directory_style_category() {
        let dir = tempdir().unwrap();
        let ws = workspace(dir.path());

        let path = item_path(&ws, Category::Project, "foo");

        assert_eq!(path, dir.path().join("1-Projects/foo/index.md"));
    }

    #[test]
    fn creates_scaffolded_project_directory_with_index() {
        let dir = tempdir().unwrap();
        let ws = workspace(dir.path());

        let path = create(&ws, Category::Project, "website-redesign", "").unwrap();

        assert_eq!(
            path,
            dir.path().join("1-Projects/website-redesign/index.md")
        );
        assert!(path.exists());
    }
}
