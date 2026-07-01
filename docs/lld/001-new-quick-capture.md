# LLD: `tk new` — Story 001 (Quick capture via `$EDITOR`)

Source: [docs/user-stories/new.md](../user-stories/new.md), User Story 001.
Module boundaries follow [docs/design.md](../design.md).

## Scope

The four scenarios of Story 001 only:

1. Accept the inferred filename
2. Override the inferred filename
3. Empty note falls back to a timestamp
4. Note with a blank first line falls back to a timestamp

All four exercise `tk new` with **no arguments**. Because the repo is still
at the boilerplate stage, this plan necessarily stands up the shared
foundation (`category`, `config`, `workspace`) that every later `new`
story (002–006) will also depend on — but only the behavior needed to make
these four scenarios pass gets dedicated tests here.

### Out of scope (future LLDs)

- `tk new <filename>` (002), `--project`/`--area`/`--resource` (003–005),
  extension inference for named files (006)
- `tk init`, `daily`, `mv`, `list`, `status`, `review`, `completions`
- `items::mv`, `items::list`, `items::status`, `review` module

## `design.md` changes (applied)

`design.md` has been updated to reflect two decisions made while planning
this story:

- `items::create` now takes a `content: &str` param (`""` for callers with
  nothing to seed, e.g. stories 002–005), so `items` — not `cli` — owns the
  filesystem write of the captured editor buffer:
  `create(ws: &Workspace, category: Category, name: &str, content: &str) -> Result<PathBuf>`.
- `editor` is now documented as an `Editor` trait (`RealEditor` + test
  fakes) wrapping a pure `suggest_filename` core, matching the split
  described below.

## New dependencies

- `thiserror` — per-module error enums (`err-thiserror-lib`)
- `anyhow` — error context at the `cli`/`main` boundary (`err-anyhow-app`)
- `tempfile` — scratch file for the editor buffer, and temp dirs in tests
- `chrono` (with default `clock` feature) — timestamp fallback formatting;
  stdlib has no calendar formatting

Also add `src/lib.rs` and shrink `src/main.rs` to a thin entry point
(`proj-lib-main-split`), since `cli`'s orchestration functions need to be
unit-testable from `tests/` / `#[cfg(test)]` without going through a binary.

## Module designs

### `category` (new)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Category { Inbox, Project, Area, Resource, Archive }

impl Category {
    pub fn is_directory_style(&self) -> bool {
        matches!(self, Category::Project | Category::Area)
    }
}
```

No I/O, no error type. Story 001 only ever uses `Category::Inbox`, but the
enum is shared vocabulary so it's built in full now.

### `config` (new)

```rust
pub struct Config {
    pub category_dirs: [String; 5], // Inbox, Project, Area, Resource, Archive order
    pub default_extension: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            category_dirs: ["0-Inbox", "1-Projects", "2-Areas", "3-Resources", "4-Archive"]
                .map(String::from),
            default_extension: "md".to_string(),
        }
    }
}

impl Config {
    pub fn load(path: &Path) -> Result<Config, ConfigError>;
}
```

`load` reads `.tick.toml` if `path` exists, merging any present fields over
`Config::default()`; if `path` doesn't exist, returns `Config::default()`
untouched (not an error — `.tick.toml` is optional per the README).

### `workspace` (new)

```rust
pub struct Workspace { pub root: PathBuf, pub config: Config }

impl Workspace {
    pub fn discover(start: &Path) -> Result<Workspace, WorkspaceError>;
    pub fn category_dir(&self, category: Category) -> PathBuf;
}
```

`discover` walks up from `start` through ancestors, stopping at the first
directory containing `.tick.toml` (load config from it) **or** all five
default-named category dirs (use `Config::default()`). Returns
`WorkspaceError::NotFound` if it reaches the filesystem root without a
match.

### `editor` (new)

```rust
pub trait Editor {
    fn capture(&self) -> Result<(String, String), EditorError>; // (content, suggested_filename_no_ext)
}

pub struct RealEditor;
impl Editor for RealEditor { /* real $EDITOR spawn, see below */ }
```

`RealEditor::capture`:
1. Read `$EDITOR`; error `EditorError::NotSet` if absent/empty.
2. Create a `tempfile::NamedTempFile` with a `.md` suffix (so the editor
   gets syntax highlighting).
3. `Command::new(editor).arg(path).status()`; non-zero exit ⇒
   `EditorError::Aborted`.
4. Read the file back to a `String`.
5. Compute `suggest_filename(&content)`.
6. Return `(content, suggested_filename)`. Tempfile cleans up on drop.

This is deliberately the *only* impure function in the module — everything
else is pure and unit-tested directly:

```rust
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

fn slugify(input: &str) -> String { /* lowercase; non-alnum runs -> '-'; trim/collapse '-' */ }
fn timestamp_slug(now: SystemTime) -> String { /* chrono format, e.g. "20260630-153045" */ }
```

The `_at` split exists purely for testability — `SystemTime` is injected so
the timestamp-fallback scenario is deterministic in tests instead of racing
the clock.

Confirmed behavior (now in the acceptance criteria as its own scenario):
only line 1 is inspected for a title. A note with a blank first line and a
title on a later line falls back to the timestamp, same as a fully empty
note — `title.is_empty()` above covers both cases identically.

### `items` (new — `create` only)

```rust
pub fn create(ws: &Workspace, category: Category, name: &str, content: &str) -> Result<PathBuf, ItemsError>;
```

For `Category::Inbox` (the only category Story 001 exercises): appends
`ws.config.default_extension` to `name` if it has no extension, writes
`content` to `ws.category_dir(Inbox).join(filename)`, returns that path.
(Directory-style creation for `Project`/`Area` is implemented per
`design.md`'s contract since the branch is trivial to include now, but has
no dedicated tests until stories 003/004.)

### `cli` (new — `Ui` trait, `new` orchestration only)

```rust
pub trait Ui {
    fn confirm(&mut self, prompt: &str, default: &str) -> Result<String, UiError>;
    fn choose(&mut self, prompt: &str, options: &[&str]) -> Result<char, UiError>;
}
pub struct TerminalUi; // real stdin/stdout impl, not unit tested

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
```

`main.rs`/`lib.rs` wires a minimal `clap` `Commands::New { filename: Option<String> }`
(the `--project`/`--area`/`--resource` flags are out of scope, added in the
003–005 LLDs), calls `run_new`, prints `Opening $EDITOR...` before capture
and `Created {path}` after.

## Error handling

Per-module `thiserror` enums (`WorkspaceError`, `EditorError`, `ItemsError`,
`ConfigError`, `UiError`), each with `#[from] io::Error` where relevant.
`cli`/`main` is the application boundary — propagate with `?` into
`anyhow::Result` and add `.context(...)` at the top level per
`err-anyhow-app`.

## Test plan (TDD — write these first)

| Scenario | Test | Module |
|---|---|---|
| Accept inferred filename | `suggest_filename_at("# Website Improvement Ideas\n...", _) == "website-improvement-ideas"` | `editor` (unit) |
| Accept inferred filename | `run_new` with `FakeEditor` returning that content + a `FakeUi` that echoes the default back creates `0-Inbox/website-improvement-ideas.md` with the captured content, returns that path | `cli` (unit, temp `Workspace`) |
| Override filename | Same fixture, `FakeUi::confirm` returns `"my-custom-name"` instead of the default → file created at `0-Inbox/my-custom-name.md` | `cli` (unit) |
| Empty note → timestamp | `suggest_filename_at("", fixed_time)` and `suggest_filename_at("   \n\n", fixed_time)` both equal the expected timestamp slug for `fixed_time` | `editor` (unit) |
| Empty note → timestamp | `run_new` with `FakeEditor` returning `("", "")`-shaped empty capture drives the same prompt/create path using a timestamp default | `cli` (unit) |
| Blank first line → timestamp | `suggest_filename_at("\n# Title On Line 2\n", fixed_time)` equals the expected timestamp slug for `fixed_time` (title on line 2 is ignored) | `editor` (unit) |
| Extension already present in override | `items::create` doesn't double-append `.md` if the user-typed name already has it | `items` (unit) |
| Slugify edge cases | multiple `#`, mixed case, punctuation, extra internal whitespace | `editor` (unit, table-driven) |
| `Workspace::discover` | finds root via `.tick.toml`; finds root via bare category dirs; returns `NotFound` outside any workspace | `workspace` (unit, `tempfile::tempdir`) |

`RealEditor::capture` and `TerminalUi` are not unit tested (real process
spawn / real stdin) — verify manually per the Implementation plan below.

## Implementation plan

1. `src/lib.rs` + slim `src/main.rs`; add `thiserror`, `anyhow`,
   `tempfile`, `chrono` to `Cargo.toml`.
2. `category` module + tests.
3. `config` module (`Default`, `load`) + tests.
4. `workspace` module (`discover`, `category_dir`) + tests.
5. `editor::suggest_filename_at` / `slugify` / `timestamp_slug`, fully
   unit-tested against all three Story 001 scenarios + edge cases, before
   writing `capture()` or touching a real process.
6. `editor::capture` + `Editor` trait + `RealEditor` (thin, not unit
   tested).
7. `items::create` (content param, extension inference, Inbox path) +
   tests.
8. `cli::Ui` trait + `run_new` orchestration, tested with `FakeEditor` +
   `FakeUi` test doubles covering the three scenarios end-to-end at the
   orchestration layer.
9. `cli::TerminalUi` (real impl, not unit tested) + wire `Commands::New`
   into `main.rs`.
10. Manual smoke test: since `tk init` doesn't exist yet, hand-create the
    five category dirs in a scratch directory, `cd` in, run
    `cargo run -- new`, confirm the editor opens, the prompt matches the
    README's `tk new` example, and the file lands in `0-Inbox`. Flag to
    the user if they want a minimal `tk init` stubbed in alongside this
    for a cleaner smoke test.
11. `cargo clippy`, `cargo fmt --check`, `cargo test` clean before calling
    the story done.
