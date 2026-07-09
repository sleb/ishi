use std::env;
use std::io;
use std::path::Path;
use std::process::Command;
use std::time::SystemTime;

use chrono::{DateTime, Utc};
use tempfile::Builder;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EditorError {
    #[error("$EDITOR is not set")]
    NotSet,
    #[error("editor exited without saving")]
    Aborted,
    #[error(transparent)]
    Io(#[from] io::Error),
}

pub trait Editor {
    /// Opens `$EDITOR` on `seed` (the rendered template, with `{{title}}`
    /// empty and `{{cursor}}` marking the starting line) and returns
    /// `(content, suggested_filename_no_ext)`.
    fn capture(&self, seed: &str) -> Result<(String, String), EditorError>;

    /// Opens `$EDITOR` directly on an existing file at `path` — no scratch
    /// file, no seed content, no filename inference. Used to reopen an
    /// already-created daily note untouched.
    fn open(&self, path: &Path) -> Result<(), EditorError>;
}

pub struct RealEditor;

impl Editor for RealEditor {
    fn capture(&self, seed: &str) -> Result<(String, String), EditorError> {
        let editor = env::var("EDITOR").map_err(|_| EditorError::NotSet)?;
        if editor.trim().is_empty() {
            return Err(EditorError::NotSet);
        }

        let (content, cursor_line) = locate_cursor(seed);

        let file = Builder::new().suffix(".md").tempfile()?;
        let path = file.path().to_path_buf();
        std::fs::write(&path, &content)?;

        let mut command = Command::new(&editor);
        if let Some(line) = cursor_line {
            command.arg(format!("+{line}"));
        }
        let status = command.arg(&path).status()?;
        if !status.success() {
            return Err(EditorError::Aborted);
        }

        let content = std::fs::read_to_string(&path)?;
        let suggested = suggest_filename(&content);
        Ok((content, suggested))
    }

    fn open(&self, path: &Path) -> Result<(), EditorError> {
        let editor = env::var("EDITOR").map_err(|_| EditorError::NotSet)?;
        if editor.trim().is_empty() {
            return Err(EditorError::NotSet);
        }
        let status = Command::new(&editor).arg(path).status()?;
        if !status.success() {
            return Err(EditorError::Aborted);
        }
        Ok(())
    }
}

/// Strips the `{{cursor}}` marker out of `seed`, returning the content to
/// write to the scratch file along with the 1-based line the marker was on
/// (for the editor's `+<line>` argument), or `None` if `seed` has no marker.
fn locate_cursor(seed: &str) -> (String, Option<usize>) {
    match seed.find("{{cursor}}") {
        Some(idx) => {
            let line = seed[..idx].matches('\n').count() + 1;
            (seed.replacen("{{cursor}}", "", 1), Some(line))
        }
        None => (seed.to_string(), None),
    }
}

/// Suggests a filename (without extension) from `content`: skips a leading
/// YAML frontmatter block if present, then looks for the first Markdown
/// heading line (any `#` level) with non-blank text; a heading with empty
/// text doesn't count as found, and the search continues past it. Falls
/// back to the first non-blank, non-heading line, then to a timestamp-based
/// name if nothing else is found.
pub fn suggest_filename(content: &str) -> String {
    suggest_filename_at(content, SystemTime::now())
}

fn suggest_filename_at(content: &str, now: SystemTime) -> String {
    if let Some(title) = gist::parser::first_heading_text(content) {
        return slugify(&title);
    }
    let body = &content[gist::parser::frontmatter_body_offset(content)..];
    if let Some(line) = first_fallback_line(body) {
        return slugify(line);
    }
    timestamp_slug(now)
}

/// The first non-blank line of `body` that isn't an empty-text heading (a
/// heading with no text, e.g. an unfilled `# {{cursor}}` template, was
/// already rejected as a title candidate by
/// `gist::parser::first_heading_text` and isn't a fallback candidate
/// either).
fn first_fallback_line(body: &str) -> Option<&str> {
    body.lines().find(|line| {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return false;
        }
        match trimmed.strip_prefix('#') {
            Some(rest) => !rest.trim_start_matches('#').trim().is_empty(),
            None => true,
        }
    })
}

fn slugify(input: &str) -> String {
    let mut slug = String::with_capacity(input.len());
    let mut last_was_dash = false;
    for ch in input.chars() {
        if ch.is_alphanumeric() {
            slug.extend(ch.to_lowercase());
            last_was_dash = false;
        } else if !last_was_dash {
            slug.push('-');
            last_was_dash = true;
        }
    }
    slug.trim_matches('-').to_string()
}

fn timestamp_slug(now: SystemTime) -> String {
    let datetime: DateTime<Utc> = now.into();
    datetime.format("%Y%m%d-%H%M%S").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn fixed_time() -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_secs(1_782_916_245) // 2026-06-30T15:30:45Z
    }

    #[test]
    fn accepts_inferred_filename_from_heading() {
        let content = "# Website Improvement Ideas\nSome body text.";
        assert_eq!(
            suggest_filename_at(content, fixed_time()),
            "website-improvement-ideas"
        );
    }

    #[test]
    fn heading_further_down_is_still_used() {
        let content = "---\nlast_updated: 2026-07-03\n---\nSome body text.\n# Actual Title\n";
        assert_eq!(suggest_filename_at(content, fixed_time()), "actual-title");
    }

    #[test]
    fn no_heading_falls_back_to_first_line() {
        let content = "---\nlast_updated: 2026-07-03\n---\nJust plain body text.\n";
        assert_eq!(
            suggest_filename_at(content, fixed_time()),
            "just-plain-body-text"
        );
    }

    #[test]
    fn unmodified_template_falls_back_to_timestamp() {
        // The cursor marker was stripped, leaving an empty heading.
        let content = "---\nlast_updated: 2026-07-03\n---\n# \n";
        let expected = timestamp_slug(fixed_time());
        assert_eq!(suggest_filename_at(content, fixed_time()), expected);
    }

    #[test]
    fn empty_note_falls_back_to_timestamp() {
        let expected = timestamp_slug(fixed_time());
        assert_eq!(suggest_filename_at("", fixed_time()), expected);
        assert_eq!(suggest_filename_at("   \n\n", fixed_time()), expected);
    }

    #[test]
    fn slugify_edge_cases() {
        let cases = [
            ("## Multiple Hashes", "multiple-hashes"),
            ("MiXeD CaSe", "mixed-case"),
            (
                "punctuation! and, punctuation?",
                "punctuation-and-punctuation",
            ),
            (
                "extra   internal    whitespace",
                "extra-internal-whitespace",
            ),
        ];
        for (input, expected) in cases {
            assert_eq!(suggest_filename_at(input, fixed_time()), expected);
        }
    }

    #[test]
    fn locate_cursor_finds_line_and_strips_marker() {
        let seed = "---\nlast_updated: 2026-07-03\n---\n# {{cursor}}\n";

        let (content, line) = locate_cursor(seed);

        assert_eq!(content, "---\nlast_updated: 2026-07-03\n---\n# \n");
        assert_eq!(line, Some(4));
    }

    #[test]
    fn locate_cursor_returns_none_when_marker_absent() {
        let seed = "no marker here\n";

        let (content, line) = locate_cursor(seed);

        assert_eq!(content, seed);
        assert_eq!(line, None);
    }
}
