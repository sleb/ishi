use std::env;
use std::io;
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
    /// Returns `(content, suggested_filename_no_ext)`.
    fn capture(&self) -> Result<(String, String), EditorError>;
}

pub struct RealEditor;

impl Editor for RealEditor {
    fn capture(&self) -> Result<(String, String), EditorError> {
        let editor = env::var("EDITOR").map_err(|_| EditorError::NotSet)?;
        if editor.trim().is_empty() {
            return Err(EditorError::NotSet);
        }

        let file = Builder::new().suffix(".md").tempfile()?;
        let path = file.path().to_path_buf();

        let status = Command::new(&editor).arg(&path).status()?;
        if !status.success() {
            return Err(EditorError::Aborted);
        }

        let content = std::fs::read_to_string(&path)?;
        let suggested = suggest_filename(&content);
        Ok((content, suggested))
    }
}

/// Suggests a filename (without extension) from `content`'s first line,
/// falling back to a timestamp-based name if that line is blank.
pub fn suggest_filename(content: &str) -> String {
    suggest_filename_at(content, SystemTime::now())
}

fn suggest_filename_at(content: &str, now: SystemTime) -> String {
    let title = content
        .lines()
        .next()
        .unwrap_or("")
        .trim_start_matches('#')
        .trim();
    if title.is_empty() {
        timestamp_slug(now)
    } else {
        slugify(title)
    }
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
    fn empty_note_falls_back_to_timestamp() {
        let expected = timestamp_slug(fixed_time());
        assert_eq!(suggest_filename_at("", fixed_time()), expected);
        assert_eq!(suggest_filename_at("   \n\n", fixed_time()), expected);
    }

    #[test]
    fn blank_first_line_falls_back_to_timestamp() {
        let expected = timestamp_slug(fixed_time());
        assert_eq!(
            suggest_filename_at("\n# Title On Line 2\n", fixed_time()),
            expected
        );
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
}
