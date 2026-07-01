use std::io::{self, Write};
use std::path::PathBuf;

use thiserror::Error;

use crate::category::Category;
use crate::editor::Editor;
use crate::items;
use crate::workspace::Workspace;

#[derive(Debug, Error)]
pub enum UiError {
    #[error(transparent)]
    Io(#[from] io::Error),
}

pub trait Ui {
    fn confirm(&mut self, prompt: &str, default: &str) -> Result<String, UiError>;
    fn choose(&mut self, prompt: &str, options: &[&str]) -> Result<char, UiError>;
}

pub struct TerminalUi;

impl Ui for TerminalUi {
    fn confirm(&mut self, prompt: &str, default: &str) -> Result<String, UiError> {
        print!("{prompt} [{default}] ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let trimmed = input.trim();
        if trimmed.is_empty() {
            Ok(default.to_string())
        } else {
            Ok(trimmed.to_string())
        }
    }

    fn choose(&mut self, prompt: &str, options: &[&str]) -> Result<char, UiError> {
        loop {
            print!("{prompt} [{}] ", options.join("/"));
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let trimmed = input.trim().to_lowercase();
            if let Some(choice) = trimmed.chars().next()
                && options
                    .iter()
                    .any(|o| o.to_lowercase() == choice.to_string())
            {
                return Ok(choice);
            }
        }
    }
}

pub fn run_new(
    ws: &Workspace,
    editor: &dyn Editor,
    ui: &mut dyn Ui,
    filename: Option<String>,
) -> anyhow::Result<PathBuf> {
    let path = match filename {
        Some(name) => items::create(ws, Category::Inbox, &name, "")?,
        None => {
            let (content, suggested) = editor.capture()?;
            let default = format!("{suggested}.{}", ws.config.default_extension);
            let chosen = ui.confirm(&format!("Create \"{default}\"?"), &default)?;
            items::create(ws, Category::Inbox, &chosen, &content)?
        }
    };
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::editor::EditorError;
    use std::fs;
    use tempfile::tempdir;

    struct FakeEditor {
        content: String,
        suggested: String,
    }

    impl Editor for FakeEditor {
        fn capture(&self) -> Result<(String, String), EditorError> {
            Ok((self.content.clone(), self.suggested.clone()))
        }
    }

    struct FakeUi {
        confirm_response: String,
    }

    impl Ui for FakeUi {
        fn confirm(&mut self, _prompt: &str, _default: &str) -> Result<String, UiError> {
            Ok(self.confirm_response.clone())
        }

        fn choose(&mut self, _prompt: &str, _options: &[&str]) -> Result<char, UiError> {
            unimplemented!("not exercised by `new` story 001")
        }
    }

    fn workspace(root: &std::path::Path) -> Workspace {
        Workspace {
            root: root.to_path_buf(),
            config: Config::default(),
        }
    }

    #[test]
    fn accepts_inferred_filename() {
        let dir = tempdir().unwrap();
        let ws = workspace(dir.path());
        let editor = FakeEditor {
            content: "# Website Improvement Ideas\nbody".to_string(),
            suggested: "website-improvement-ideas".to_string(),
        };
        let mut ui = FakeUi {
            confirm_response: "website-improvement-ideas.md".to_string(),
        };

        let path = run_new(&ws, &editor, &mut ui, None).unwrap();

        assert_eq!(
            path,
            dir.path().join("0-Inbox/website-improvement-ideas.md")
        );
        assert_eq!(
            fs::read_to_string(&path).unwrap(),
            "# Website Improvement Ideas\nbody"
        );
    }

    #[test]
    fn overrides_inferred_filename() {
        let dir = tempdir().unwrap();
        let ws = workspace(dir.path());
        let editor = FakeEditor {
            content: "# Website Improvement Ideas\nbody".to_string(),
            suggested: "website-improvement-ideas".to_string(),
        };
        let mut ui = FakeUi {
            confirm_response: "my-custom-name".to_string(),
        };

        let path = run_new(&ws, &editor, &mut ui, None).unwrap();

        assert_eq!(path, dir.path().join("0-Inbox/my-custom-name.md"));
    }

    #[test]
    fn empty_note_uses_timestamp_default_path() {
        let dir = tempdir().unwrap();
        let ws = workspace(dir.path());
        let editor = FakeEditor {
            content: String::new(),
            suggested: "20260630-153045".to_string(),
        };
        let mut ui = FakeUi {
            confirm_response: "20260630-153045.md".to_string(),
        };

        let path = run_new(&ws, &editor, &mut ui, None).unwrap();

        assert_eq!(path, dir.path().join("0-Inbox/20260630-153045.md"));
        assert_eq!(fs::read_to_string(&path).unwrap(), "");
    }

    #[test]
    fn named_filename_skips_editor() {
        let dir = tempdir().unwrap();
        let ws = workspace(dir.path());
        struct PanicEditor;
        impl Editor for PanicEditor {
            fn capture(&self) -> Result<(String, String), EditorError> {
                panic!("editor should not be invoked when a filename is given")
            }
        }
        let editor = PanicEditor;
        let mut ui = FakeUi {
            confirm_response: String::new(),
        };

        let path = run_new(&ws, &editor, &mut ui, Some("my-file".to_string())).unwrap();

        assert_eq!(path, dir.path().join("0-Inbox/my-file.md"));
    }
}
