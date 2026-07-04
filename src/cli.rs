use std::io::{self, Write};
use std::path::{Path, PathBuf};

use chrono::Local;
use thiserror::Error;
use uuid::Uuid;

use crate::category::{Category, Kind};
use crate::config;
use crate::editor::Editor;
use crate::items;
use crate::workspace::{self, Workspace};

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
    kind: Kind,
    filename: Option<String>,
) -> anyhow::Result<PathBuf> {
    let category = kind.category();
    let template = ws.config.templates.for_kind(kind);
    let path = match filename {
        Some(name) => {
            let now = Local::now();
            let today = now.date_naive().format("%Y-%m-%d").to_string();
            let time = now.format("%H:%M").to_string();
            let uuid = Uuid::new_v4().to_string();
            let rendered =
                config::render(template, &name, &today, &time, &uuid).replace("{{cursor}}", "");
            items::create(ws, category, &name, &rendered)?
        }
        None => {
            let now = Local::now();
            let today = now.date_naive().format("%Y-%m-%d").to_string();
            let time = now.format("%H:%M").to_string();
            let uuid = Uuid::new_v4().to_string();
            let seed = config::render(template, "", &today, &time, &uuid);
            let (content, suggested) = editor.capture(&seed)?;
            let default = if category.is_directory_style() {
                suggested
            } else {
                format!("{suggested}.{}", ws.config.default_extension)
            };
            let chosen = ui.confirm(&format!("Create \"{default}\"?"), &default)?;
            items::create(ws, category, &chosen, &content)?
        }
    };
    Ok(path)
}

#[derive(Debug)]
pub enum DailyOutcome {
    Created(PathBuf),
    Reopened(PathBuf),
}

/// True if today's daily note already exists — lets `main` print
/// `Opening $EDITOR...` *before* handing control to a blocking editor
/// process, the same convention the no-filename `run_new` path uses.
pub fn daily_note_exists(ws: &Workspace) -> bool {
    let today = Local::now().date_naive().format("%Y-%m-%d").to_string();
    items::item_path(ws, Category::Inbox, &today).exists()
}

pub fn run_daily(ws: &Workspace, editor: &dyn Editor) -> anyhow::Result<DailyOutcome> {
    let now = Local::now();
    let today = now.date_naive().format("%Y-%m-%d").to_string();
    let path = items::item_path(ws, Category::Inbox, &today);

    if path.exists() {
        editor.open(&path)?;
        Ok(DailyOutcome::Reopened(path))
    } else {
        let time = now.format("%H:%M").to_string();
        let uuid = Uuid::new_v4().to_string();
        let rendered = config::render(
            ws.config.templates.for_kind(Kind::Daily),
            &today,
            &today,
            &time,
            &uuid,
        )
        .replace("{{cursor}}", "");
        let created = items::create(ws, Category::Inbox, &today, &rendered)?;
        Ok(DailyOutcome::Created(created))
    }
}

pub fn run_init(cwd: &Path, name: Option<&str>) -> anyhow::Result<String> {
    let (target, display) = match name {
        Some(n) => (cwd.join(n), format!("./{n}")),
        None => (cwd.to_path_buf(), ".".to_string()),
    };

    let report = workspace::init(&target)?;

    Ok(match report.created.len() {
        5 => format!("Created PARA system in {display}"),
        0 => format!("PARA system in {display} is already complete; no changes made"),
        _ => format!("Created {} in {display}", report.created.join(", ")),
    })
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
        fn capture(&self, _seed: &str) -> Result<(String, String), EditorError> {
            Ok((self.content.clone(), self.suggested.clone()))
        }

        fn open(&self, _path: &Path) -> Result<(), EditorError> {
            unimplemented!("not exercised by this test")
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

    fn workspace_with_note_template(root: &std::path::Path, template: &str) -> Workspace {
        let mut config = Config::default();
        config.templates.note = template.to_string();
        Workspace {
            root: root.to_path_buf(),
            config,
        }
    }

    fn contains_hh_mm_time(text: &str) -> bool {
        text.split_whitespace().any(|word| {
            word.len() == 5
                && word.as_bytes()[2] == b':'
                && word[..2].chars().all(|c| c.is_ascii_digit())
                && word[3..].chars().all(|c| c.is_ascii_digit())
        })
    }

    fn contains_uuid(text: &str) -> bool {
        text.split_whitespace()
            .any(|word| uuid::Uuid::parse_str(word).is_ok())
    }

    struct PanicEditor;

    impl Editor for PanicEditor {
        fn capture(&self, _seed: &str) -> Result<(String, String), EditorError> {
            panic!("editor should not be invoked when a filename is given")
        }

        fn open(&self, _path: &Path) -> Result<(), EditorError> {
            unimplemented!("not exercised by this test")
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

        let path = run_new(&ws, &editor, &mut ui, Kind::Inbox, None).unwrap();

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

        let path = run_new(&ws, &editor, &mut ui, Kind::Inbox, None).unwrap();

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

        let path = run_new(&ws, &editor, &mut ui, Kind::Inbox, None).unwrap();

        assert_eq!(path, dir.path().join("0-Inbox/20260630-153045.md"));
        assert_eq!(fs::read_to_string(&path).unwrap(), "");
    }

    #[test]
    fn seeds_editor_with_rendered_note_template() {
        use std::cell::RefCell;

        struct RecordingEditor {
            seen_seed: RefCell<String>,
        }

        impl Editor for RecordingEditor {
            fn capture(&self, seed: &str) -> Result<(String, String), EditorError> {
                *self.seen_seed.borrow_mut() = seed.to_string();
                Ok(("# Title\n".to_string(), "title".to_string()))
            }

            fn open(&self, _path: &Path) -> Result<(), EditorError> {
                unimplemented!("not exercised by this test")
            }
        }

        let dir = tempdir().unwrap();
        let ws = workspace(dir.path());
        let editor = RecordingEditor {
            seen_seed: RefCell::new(String::new()),
        };
        let mut ui = FakeUi {
            confirm_response: "title.md".to_string(),
        };

        run_new(&ws, &editor, &mut ui, Kind::Inbox, None).unwrap();

        let seed = editor.seen_seed.borrow();
        assert!(seed.contains("{{cursor}}"));
        assert!(!seed.contains("{{title}}"));
        assert!(!seed.contains("{{date}}"));
    }

    #[test]
    fn captures_into_new_project_directory() {
        let dir = tempdir().unwrap();
        let ws = workspace(dir.path());
        let editor = FakeEditor {
            content: "# Website Redesign\nbody".to_string(),
            suggested: "website-redesign".to_string(),
        };
        let mut ui = FakeUi {
            confirm_response: "website-redesign".to_string(),
        };

        let path = run_new(&ws, &editor, &mut ui, Kind::Project, None).unwrap();

        assert_eq!(
            path,
            dir.path().join("1-Projects/website-redesign/index.md")
        );
        assert_eq!(
            fs::read_to_string(&path).unwrap(),
            "# Website Redesign\nbody"
        );
    }

    #[test]
    fn captures_into_new_area_directory() {
        let dir = tempdir().unwrap();
        let ws = workspace(dir.path());
        let editor = FakeEditor {
            content: "# Health\nbody".to_string(),
            suggested: "health".to_string(),
        };
        let mut ui = FakeUi {
            confirm_response: "health".to_string(),
        };

        let path = run_new(&ws, &editor, &mut ui, Kind::Area, None).unwrap();

        assert_eq!(path, dir.path().join("2-Areas/health/index.md"));
        assert_eq!(fs::read_to_string(&path).unwrap(), "# Health\nbody");
    }

    #[test]
    fn captures_into_new_resource_file() {
        let dir = tempdir().unwrap();
        let ws = workspace(dir.path());
        let editor = FakeEditor {
            content: "# Recipe Ideas\nbody".to_string(),
            suggested: "recipe-ideas".to_string(),
        };
        let mut ui = FakeUi {
            confirm_response: "recipe-ideas.md".to_string(),
        };

        let path = run_new(&ws, &editor, &mut ui, Kind::Resource, None).unwrap();

        assert_eq!(path, dir.path().join("3-Resources/recipe-ideas.md"));
        assert_eq!(fs::read_to_string(&path).unwrap(), "# Recipe Ideas\nbody");
    }

    #[test]
    fn project_confirm_prompt_suggests_bare_directory_name_without_extension() {
        use std::cell::RefCell;

        struct RecordingUi {
            seen_default: RefCell<String>,
        }

        impl Ui for RecordingUi {
            fn confirm(&mut self, _prompt: &str, default: &str) -> Result<String, UiError> {
                *self.seen_default.borrow_mut() = default.to_string();
                Ok(default.to_string())
            }

            fn choose(&mut self, _prompt: &str, _options: &[&str]) -> Result<char, UiError> {
                unimplemented!("not exercised by this test")
            }
        }

        let dir = tempdir().unwrap();
        let ws = workspace(dir.path());
        let editor = FakeEditor {
            content: "# Website Redesign\n".to_string(),
            suggested: "website-redesign".to_string(),
        };
        let mut ui = RecordingUi {
            seen_default: RefCell::new(String::new()),
        };

        run_new(&ws, &editor, &mut ui, Kind::Project, None).unwrap();

        assert_eq!(*ui.seen_default.borrow(), "website-redesign");
    }

    #[test]
    fn editor_seed_uses_category_specific_template() {
        use std::cell::RefCell;

        struct RecordingEditor {
            seen_seed: RefCell<String>,
        }

        impl Editor for RecordingEditor {
            fn capture(&self, seed: &str) -> Result<(String, String), EditorError> {
                *self.seen_seed.borrow_mut() = seed.to_string();
                Ok(("# Title\n".to_string(), "title".to_string()))
            }

            fn open(&self, _path: &Path) -> Result<(), EditorError> {
                unimplemented!("not exercised by this test")
            }
        }

        let dir = tempdir().unwrap();
        let ws = workspace(dir.path());
        let editor = RecordingEditor {
            seen_seed: RefCell::new(String::new()),
        };
        let mut ui = FakeUi {
            confirm_response: "title".to_string(),
        };

        run_new(&ws, &editor, &mut ui, Kind::Project, None).unwrap();

        let seed = editor.seen_seed.borrow();
        assert!(seed.contains("Status: active"));
    }

    #[test]
    fn named_filename_skips_editor() {
        let dir = tempdir().unwrap();
        let ws = workspace(dir.path());
        let editor = PanicEditor;
        let mut ui = FakeUi {
            confirm_response: String::new(),
        };

        let path = run_new(
            &ws,
            &editor,
            &mut ui,
            Kind::Inbox,
            Some("my-file".to_string()),
        )
        .unwrap();

        assert_eq!(path, dir.path().join("0-Inbox/my-file.md"));
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("# my-file"));
    }

    #[test]
    fn creates_named_project_directory() {
        let dir = tempdir().unwrap();
        let ws = workspace(dir.path());
        let editor = PanicEditor;
        let mut ui = FakeUi {
            confirm_response: String::new(),
        };

        let path = run_new(
            &ws,
            &editor,
            &mut ui,
            Kind::Project,
            Some("website-redesign".to_string()),
        )
        .unwrap();

        assert_eq!(
            path,
            dir.path().join("1-Projects/website-redesign/index.md")
        );
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("# website-redesign"));
        let today = Local::now().date_naive().format("%Y-%m-%d").to_string();
        assert!(content.contains(&format!("last_updated: {today}")));
    }

    #[test]
    fn creates_named_area_directory() {
        let dir = tempdir().unwrap();
        let ws = workspace(dir.path());
        let editor = PanicEditor;
        let mut ui = FakeUi {
            confirm_response: String::new(),
        };

        let path = run_new(
            &ws,
            &editor,
            &mut ui,
            Kind::Area,
            Some("health".to_string()),
        )
        .unwrap();

        assert_eq!(path, dir.path().join("2-Areas/health/index.md"));
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("# health"));
        let today = Local::now().date_naive().format("%Y-%m-%d").to_string();
        assert!(content.contains(&format!("last_updated: {today}")));
    }

    #[test]
    fn creates_named_resource_file() {
        let dir = tempdir().unwrap();
        let ws = workspace(dir.path());
        let editor = PanicEditor;
        let mut ui = FakeUi {
            confirm_response: String::new(),
        };

        let path = run_new(
            &ws,
            &editor,
            &mut ui,
            Kind::Resource,
            Some("recipe-ideas".to_string()),
        )
        .unwrap();

        assert_eq!(path, dir.path().join("3-Resources/recipe-ideas.md"));
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("# recipe-ideas"));
    }

    #[test]
    fn named_note_renders_date_in_frontmatter() {
        let dir = tempdir().unwrap();
        let ws = workspace(dir.path());
        let editor = PanicEditor;
        let mut ui = FakeUi {
            confirm_response: String::new(),
        };

        let path = run_new(
            &ws,
            &editor,
            &mut ui,
            Kind::Inbox,
            Some("my-file".to_string()),
        )
        .unwrap();

        let content = fs::read_to_string(&path).unwrap();
        let today = Local::now().date_naive().format("%Y-%m-%d").to_string();
        assert!(content.contains(&format!("last_updated: {today}")));
    }

    #[test]
    fn named_note_renders_time() {
        let dir = tempdir().unwrap();
        let ws = workspace_with_note_template(dir.path(), "captured at {{time}}\n");
        let editor = PanicEditor;
        let mut ui = FakeUi {
            confirm_response: String::new(),
        };

        let path = run_new(
            &ws,
            &editor,
            &mut ui,
            Kind::Inbox,
            Some("my-file".to_string()),
        )
        .unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert!(contains_hh_mm_time(&content), "content was: {content}");
    }

    #[test]
    fn editor_capture_renders_time() {
        use std::cell::RefCell;

        struct RecordingEditor {
            seen_seed: RefCell<String>,
        }

        impl Editor for RecordingEditor {
            fn capture(&self, seed: &str) -> Result<(String, String), EditorError> {
                *self.seen_seed.borrow_mut() = seed.to_string();
                Ok(("# Title\n".to_string(), "title".to_string()))
            }

            fn open(&self, _path: &Path) -> Result<(), EditorError> {
                unimplemented!("not exercised by this test")
            }
        }

        let dir = tempdir().unwrap();
        let ws = workspace_with_note_template(dir.path(), "captured at {{time}}\n");
        let editor = RecordingEditor {
            seen_seed: RefCell::new(String::new()),
        };
        let mut ui = FakeUi {
            confirm_response: "title".to_string(),
        };

        run_new(&ws, &editor, &mut ui, Kind::Inbox, None).unwrap();

        let seed = editor.seen_seed.borrow();
        assert!(!seed.contains("{{time}}"));
        assert!(contains_hh_mm_time(&seed), "seed was: {seed}");
    }

    #[test]
    fn named_note_renders_uuid() {
        let dir = tempdir().unwrap();
        let ws = workspace_with_note_template(dir.path(), "id: {{uuid}}\n");
        let editor = PanicEditor;
        let mut ui = FakeUi {
            confirm_response: String::new(),
        };

        let path = run_new(
            &ws,
            &editor,
            &mut ui,
            Kind::Inbox,
            Some("my-file".to_string()),
        )
        .unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert!(contains_uuid(&content), "content was: {content}");
    }

    #[test]
    fn editor_capture_renders_uuid() {
        use std::cell::RefCell;

        struct RecordingEditor {
            seen_seed: RefCell<String>,
        }

        impl Editor for RecordingEditor {
            fn capture(&self, seed: &str) -> Result<(String, String), EditorError> {
                *self.seen_seed.borrow_mut() = seed.to_string();
                Ok(("# Title\n".to_string(), "title".to_string()))
            }

            fn open(&self, _path: &Path) -> Result<(), EditorError> {
                unimplemented!("not exercised by this test")
            }
        }

        let dir = tempdir().unwrap();
        let ws = workspace_with_note_template(dir.path(), "id: {{uuid}}\n");
        let editor = RecordingEditor {
            seen_seed: RefCell::new(String::new()),
        };
        let mut ui = FakeUi {
            confirm_response: "title".to_string(),
        };

        run_new(&ws, &editor, &mut ui, Kind::Inbox, None).unwrap();

        let seed = editor.seen_seed.borrow();
        assert!(!seed.contains("{{uuid}}"));
        assert!(contains_uuid(&seed), "seed was: {seed}");
    }

    #[test]
    fn two_notes_get_different_uuids() {
        let dir = tempdir().unwrap();
        let ws = workspace_with_note_template(dir.path(), "id: {{uuid}}\n");
        let editor = PanicEditor;
        let mut ui = FakeUi {
            confirm_response: String::new(),
        };

        let first_path = run_new(
            &ws,
            &editor,
            &mut ui,
            Kind::Inbox,
            Some("first-note".to_string()),
        )
        .unwrap();
        let second_path = run_new(
            &ws,
            &editor,
            &mut ui,
            Kind::Inbox,
            Some("second-note".to_string()),
        )
        .unwrap();

        let first_content = fs::read_to_string(&first_path).unwrap();
        let second_content = fs::read_to_string(&second_path).unwrap();
        assert_ne!(first_content, second_content);
    }

    #[test]
    fn run_init_bare_full_create() {
        let dir = tempdir().unwrap();

        let message = run_init(dir.path(), None).unwrap();

        assert_eq!(message, "Created PARA system in .");
    }

    #[test]
    fn run_init_named_full_create() {
        let dir = tempdir().unwrap();

        let message = run_init(dir.path(), Some("my-para")).unwrap();

        assert_eq!(message, "Created PARA system in ./my-para");
        for name in Config::default().category_dirs {
            assert!(dir.path().join("my-para").join(name).is_dir());
        }
    }

    #[test]
    fn run_init_already_complete() {
        let dir = tempdir().unwrap();
        for name in Config::default().category_dirs {
            fs::create_dir_all(dir.path().join(name)).unwrap();
        }

        let message = run_init(dir.path(), None).unwrap();

        assert_eq!(
            message,
            "PARA system in . is already complete; no changes made"
        );
    }

    #[test]
    fn run_init_partial_fill_in() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("0-Inbox")).unwrap();

        let message = run_init(dir.path(), None).unwrap();

        assert_eq!(
            message,
            "Created 1-Projects, 2-Areas, 3-Resources, 4-Archive in ."
        );
    }

    #[test]
    fn run_init_bare_tolerates_unrelated_contents() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("README.md"), "hello").unwrap();

        let message = run_init(dir.path(), None).unwrap();

        assert_eq!(message, "Created PARA system in .");
        assert_eq!(
            fs::read_to_string(dir.path().join("README.md")).unwrap(),
            "hello"
        );
    }

    #[test]
    fn run_init_named_collision_surfaces_error() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("existing-file"), "").unwrap();

        let err = run_init(dir.path(), Some("existing-file")).unwrap_err();

        assert!(err.to_string().contains("existing-file"));
        assert!(err.to_string().contains("already exists"));
    }

    struct PanicOnOpenEditor;

    impl Editor for PanicOnOpenEditor {
        fn capture(&self, _seed: &str) -> Result<(String, String), EditorError> {
            unimplemented!("not exercised by this test")
        }

        fn open(&self, _path: &Path) -> Result<(), EditorError> {
            panic!("open should not be invoked when there's no existing daily note")
        }
    }

    struct RecordingOpenEditor {
        opened_path: std::cell::RefCell<Option<std::path::PathBuf>>,
    }

    impl Editor for RecordingOpenEditor {
        fn capture(&self, _seed: &str) -> Result<(String, String), EditorError> {
            unimplemented!("not exercised by this test")
        }

        fn open(&self, path: &Path) -> Result<(), EditorError> {
            *self.opened_path.borrow_mut() = Some(path.to_path_buf());
            Ok(())
        }
    }

    struct NotSetEditor;

    impl Editor for NotSetEditor {
        fn capture(&self, _seed: &str) -> Result<(String, String), EditorError> {
            unimplemented!("not exercised by this test")
        }

        fn open(&self, _path: &Path) -> Result<(), EditorError> {
            Err(EditorError::NotSet)
        }
    }

    #[test]
    fn run_daily_first_run_creates_non_interactively() {
        let dir = tempdir().unwrap();
        let ws = workspace(dir.path());
        let editor = PanicOnOpenEditor;

        let outcome = run_daily(&ws, &editor).unwrap();

        let today = Local::now().date_naive().format("%Y-%m-%d").to_string();
        match outcome {
            DailyOutcome::Created(path) => {
                assert_eq!(path, dir.path().join(format!("0-Inbox/{today}.md")));
                let content = fs::read_to_string(&path).unwrap();
                assert!(content.contains(&today));
                assert!(!content.contains("{{cursor}}"));
            }
            DailyOutcome::Reopened(_) => panic!("expected Created on first run"),
        }
    }

    #[test]
    fn run_daily_second_run_reopens_without_re_rendering() {
        let dir = tempdir().unwrap();
        let ws = workspace(dir.path());
        let today = Local::now().date_naive().format("%Y-%m-%d").to_string();
        let existing_path = items::create(&ws, Category::Inbox, &today, "custom content").unwrap();

        let editor = RecordingOpenEditor {
            opened_path: std::cell::RefCell::new(None),
        };
        let outcome = run_daily(&ws, &editor).unwrap();

        match outcome {
            DailyOutcome::Reopened(path) => assert_eq!(path, existing_path),
            DailyOutcome::Created(_) => panic!("expected Reopened on second run"),
        }
        assert_eq!(*editor.opened_path.borrow(), Some(existing_path.clone()));
        assert_eq!(
            fs::read_to_string(&existing_path).unwrap(),
            "custom content"
        );
    }

    #[test]
    fn run_daily_editor_not_set_on_reopen_surfaces_error() {
        let dir = tempdir().unwrap();
        let ws = workspace(dir.path());
        let today = Local::now().date_naive().format("%Y-%m-%d").to_string();
        items::create(&ws, Category::Inbox, &today, "custom content").unwrap();

        let editor = NotSetEditor;
        let err = run_daily(&ws, &editor).unwrap_err();

        assert!(err.to_string().contains("$EDITOR"));
    }

    #[test]
    fn run_daily_filename_is_todays_date() {
        let dir = tempdir().unwrap();
        let ws = workspace(dir.path());
        let editor = PanicOnOpenEditor;

        let outcome = run_daily(&ws, &editor).unwrap();

        let today = Local::now().date_naive().format("%Y-%m-%d").to_string();
        match outcome {
            DailyOutcome::Created(path) => {
                assert_eq!(path.file_stem().unwrap().to_str().unwrap(), today);
            }
            DailyOutcome::Reopened(_) => panic!("expected Created on first run"),
        }
    }

    #[test]
    fn daily_note_exists_reflects_filesystem() {
        let dir = tempdir().unwrap();
        let ws = workspace(dir.path());

        assert!(!daily_note_exists(&ws));

        let today = Local::now().date_naive().format("%Y-%m-%d").to_string();
        items::create(&ws, Category::Inbox, &today, "content").unwrap();

        assert!(daily_note_exists(&ws));
    }
}
