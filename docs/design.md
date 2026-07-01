# Tick: High-Level Design

## Goals

Keep the design simple: a small set of modules with clear, narrow contracts.
Filesystem/business logic stays separate from argument parsing and terminal I/O
so it can be tested without a real shell, editor, or terminal.

## Components

```
            ┌─────────┐
            │   cli   │  parses argv, prompts user, prints output
            └────┬────┘
                 │ calls
     ┌───────────┼───────────┐
     ▼           ▼           ▼
 ┌───────┐  ┌─────────┐  ┌────────┐
 │ items │  │ review  │  │ editor │
 └───┬───┘  └────┬────┘  └────────┘
     │           │
     ▼           ▼
 ┌─────────────────────┐
 │      workspace       │  resolves PARA root + category paths
 └──────────┬───────────┘
            ▼
       ┌─────────┐
       │ config  │  .tick.toml (folder names, default extension)
       └─────────┘

 ┌──────────┐
 │ category │  shared Inbox/Project/Area/Resource/Archive vocabulary
 └──────────┘  (used by cli, workspace, items, review)
```

### `category`

Shared vocabulary type, no I/O.

- `enum Category { Inbox, Project, Area, Resource, Archive }`
- `Category::is_directory_style() -> bool` — true for `Project`/`Area` (scaffolded
  dir + `index.md`), false for `Inbox`/`Resource` (flat file). `Archive` defers to
  the origin category it's preserving.

### `config`

Parses `.tick.toml`. Pure data, one file read.

- `struct Config { category_dirs: [String; 5], default_extension: String }`
- `Config::default() -> Config` — `0-Inbox`, `1-Projects`, `2-Areas`,
  `3-Resources`, `4-Archive`, `md`.
- `Config::load(path: &Path) -> Result<Config>` — reads `.tick.toml` if present,
  falls back to defaults for any field it omits.

### `workspace`

Answers "where do things live?" for every other component.

- `struct Workspace { root: PathBuf, config: Config }`
- `Workspace::discover(start: &Path) -> Result<Workspace>` — walks up from
  `start` looking for `.tick.toml` or the five category dirs.
- `Workspace::category_dir(&self, category: Category) -> PathBuf`

### `items`

All filesystem operations. Takes a `Workspace` and `Category`, returns
structured results — no printing, no prompting.

- `create(ws: &Workspace, category: Category, name: &str, content: &str) -> Result<PathBuf>`
  — creates a flat file or a scaffolded `dir/index.md`, appending the default
  extension if the name has none, and writing `content` into it (`""` for
  callers with nothing to seed, e.g. named-file creation). Returns the path
  created (the `index.md` path for directory-style categories).
- `mv(ws: &Workspace, item: &Path, target: Category) -> Result<PathBuf>` —
  moves a file or project/area directory; wraps a flat file into a new
  directory when moving into `Project`/`Area`; when moving to `Archive`,
  preserves the item's origin category as a subfolder.
- `list(ws: &Workspace, category: Category, filter: Option<&str>) -> Result<Vec<PathBuf>>`
- `status(ws: &Workspace) -> Result<StatusReport>` where
  `StatusReport { counts: [usize; 5], stale: [usize; 2] }` (stale counts apply
  to `Project`/`Area` only, based on `index.md` mtime).

### `editor`

Isolated so it's mockable in tests — no real `$EDITOR` needed to test the CLI
prompt logic. Splits into one impure entry point and a pure core so the
filename-inference logic is directly unit-testable without spawning a real
editor process or racing the system clock.

- `Editor` trait: `capture(&self) -> Result<(String, String)>` — implemented
  once as `RealEditor` (opens `$EDITOR` on a scratch file, reads it back) and
  once per test as a fake. Returns `(content, suggested_filename)`.
- `suggest_filename(content: &str) -> String` — pure. Reads content's first
  line, strips a leading `#`/whitespace, and slugifies it; falls back to a
  timestamp-based name if that first line is blank (including when the note
  itself is empty). Internally delegates to a `SystemTime`-parameterized
  helper so the timestamp fallback is deterministic in tests.

### `review`

Orchestrates the weekly-review walk, built on `items` + `editor`'s prompting
pattern.

- `run(ws: &Workspace, ui: &mut dyn Ui) -> Result<()>` — iterates `Project` and
  `Area` items, reads each `index.md`, asks the `Ui` to keep/archive/skip, and
  calls `items::mv` on archive.

### `cli`

The only component that touches argv, stdin, and stdout. A `clap`-derived
`Command` enum matching the command table in the README, dispatching to
`items`/`review`/`editor` and rendering their results.

- `Ui` trait (implemented once for a real terminal, once for tests):
  `confirm(prompt: &str, default: &str) -> Result<String>`,
  `choose(prompt: &str, options: &[&str]) -> Result<char>`.

## Notes

- `category` and `config` have no dependencies on anything else — they're the
  vocabulary and settings every other module shares.
- `workspace` depends only on `config` + `category`.
- `items` and `editor` depend only on `workspace` — they don't know about each
  other or about `cli`.
- `review` composes `items` with a `Ui`, but doesn't know about `clap` or argv.
- `cli` is the only place that does terminal I/O; every other module returns
  data or `Result`s so it can be unit-tested directly.
