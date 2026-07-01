# LLD: `tk init` — Stories 001–004 (Scaffold a PARA system)

Source: [docs/user-stories/init.md](../user-stories/init.md), User Stories
001–004. Module boundaries follow [docs/design.md](../design.md).

## Scope

All four scenarios in `init.md`:

1. `tk init` scaffolds the five category dirs into the current directory
2. `tk init <name>` scaffolds them into `./<name>` instead
3. Re-running `init` is a no-op when the target is already a complete PARA
   system, and fills in only the missing category dirs when it's partial
4. `tk init <name>` errors clearly when `./<name>` exists as a regular
   file, instead of a raw filesystem error; directories are never a
   collision, regardless of their contents

### Out of scope (future LLDs)

- Writing a `.tick.toml` on init — none of the four stories call for one,
  and `Workspace::discover`'s bare-category-dirs fallback already finds a
  default-named PARA system without it. If a later story wants `init` to
  seed one, that's a separate LLD.
- `daily`, `mv`, `list`, `status`, `review`, `completions`
- `--project`/`--area`/`--resource` flags on `new` (still open from the
  `new` LLD backlog, unrelated to `init`)

## `design.md` changes (applied)

- `workspace` gains `InitReport`, `check_collision`, and `init` — see the
  `workspace` section of `design.md` for the finalized contracts.
- `cli` gains `run_init`, the orchestration function `main` calls for the
  `Init` subcommand.

## Module designs

### `workspace` (extends existing module)

```rust
pub struct InitReport {
    pub created: Vec<String>,
}

pub fn check_collision(target: &Path) -> Result<(), WorkspaceError>;
pub fn init(target: &Path) -> Result<InitReport, WorkspaceError>;
```

`check_collision`:
1. If `target` doesn't exist, `Ok(())` — nothing to collide with.
2. If `target` exists but isn't a directory, `Err(NotAParaSystem)`.
3. If `target` is a directory, `Ok(())` — unconditionally, regardless of
   contents.

`init` calls `check_collision(target)` unconditionally, for both the bare
and named forms — there's no branch on whether `name` was given. For the
bare form this is always a no-op (`cwd` is guaranteed to already exist as
a directory), so in practice the check only ever bites on the named form,
without the two forms needing different code paths. A directory with
unrelated contents — a `.git`, a `README`, other project files — is never
a collision in either form; `init` just creates whichever category dirs
are missing alongside them (Story 003/004).

`init`:
1. `check_collision(target)?`.
2. `fs::create_dir_all(target)` — no-op if it already exists as a
   directory (which it will, for the bare `tk init` case; `create_dir_all`
   is also what makes the named case create `./<name>` when it's missing).
3. For each of the five names in `Config::default().category_dirs`, in
   order: if `target.join(name)` doesn't exist, `fs::create_dir_all` it
   and push `name` into `InitReport.created`; otherwise leave it (and its
   contents) untouched.
4. Return `InitReport { created }`.

Extend `WorkspaceError`:

```rust
#[derive(Debug, Error)]
pub enum WorkspaceError {
    #[error("no PARA workspace found in {start} or any parent directory")]
    NotFound { start: String },
    #[error("failed to load config")]
    Config(#[from] ConfigError),
    #[error("{path} already exists and is not a directory")]
    NotAParaSystem { path: String },
    #[error(transparent)]
    Io(#[from] io::Error),
}
```

### `cli` (extends existing module)

```rust
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
```

`main.rs` prints whatever `run_init` returns as-is (`println!("{message}")`)
— no further formatting at the call site, so the exact wording lives in
one place and is directly asserted in `cli` tests.

Wire a new `Commands::Init { name: Option<String> }` into the existing
`clap` `Commands` enum in `main.rs`, dispatching to `run_init(&cwd, name.as_deref())`.
Note `init` runs *before* `Workspace::discover` — unlike `new`, it doesn't
need (and shouldn't require) an existing workspace to already be found.

## Message wording

Only two of the four wordings are pinned by the acceptance criteria
(scenarios 1 and 2, matched exactly, including the README's own example):

- `Created PARA system in .`
- `Created PARA system in ./my-para`

The other two ("already complete", "filled in the gaps") only specify
*that* Tick reports something, not the exact string, so the wording above
is a judgment call, not a spec requirement — flag to the user if they'd
rather something else before implementing.

## Error handling

`WorkspaceError::NotAParaSystem` and the new `Io` variant follow the same
per-module `thiserror` pattern as the rest of the crate. `cli::run_init`
propagates with `?` into `anyhow::Result` (`err-anyhow-app`); `main.rs`
adds no extra `.context(...)` here since the error message is already
user-facing and specific.

## Test plan (TDD — write these first)

| Scenario | Test | Module |
|---|---|---|
| Bare `init` creates all five dirs | `init(&empty_or_populated_dir)` returns `InitReport { created: [all 5 names] }`; all five exist on disk afterward | `workspace` (unit, `tempfile::tempdir`) |
| Named `init` creates `./<name>` | `init(&dir.join("my-para"))` on a nonexistent path creates `my-para/` plus all five category dirs under it | `workspace` (unit) |
| Complete system is a no-op | `init` on a dir where all five already exist returns `InitReport { created: [] }` and creates nothing new (assert dir entry count unchanged) | `workspace` (unit) |
| Partial system fills gaps only | `init` on a dir with e.g. only `0-Inbox` present returns `InitReport { created: ["1-Projects", "2-Areas", "3-Resources", "4-Archive"] }`; pre-existing `0-Inbox`'s contents (write a marker file into it first) are untouched | `workspace` (unit) |
| Named collision: file | `check_collision(&path_to_an_existing_regular_file)` returns `Err(NotAParaSystem)` | `workspace` (unit) |
| Non-collision: directory with unrelated contents | `check_collision` on a dir containing e.g. `notes.txt` (not a category dir name) returns `Ok(())`, and `init` on that same dir still creates the missing category dirs, leaving `notes.txt` untouched | `workspace` (unit) |
| Non-collision: missing entirely | `check_collision` on a nonexistent path returns `Ok(())` | `workspace` (unit) |
| Non-collision: bare form is always a no-op | `check_collision` on an existing directory (standing in for `cwd`) returns `Ok(())` regardless of contents | `workspace` (unit) |
| `run_init` bare, full create | `run_init(cwd, None)` on an empty tempdir returns exactly `"Created PARA system in ."` | `cli` (unit) |
| `run_init` named, full create | `run_init(cwd, Some("my-para"))` returns exactly `"Created PARA system in ./my-para"`; `cwd/my-para/{0..4-*}` all exist | `cli` (unit) |
| `run_init` already complete | pre-create all five under `cwd`, then `run_init(cwd, None)` reports the already-complete message and creates nothing | `cli` (unit) |
| `run_init` partial fill-in | pre-create a subset, `run_init` reports the created-subset message and leaves the rest untouched | `cli` (unit) |
| `run_init` bare form tolerates unrelated contents | `cwd` pre-populated with an unrelated file (e.g. `README.md`); `run_init(cwd, None)` still succeeds, creates the five category dirs, and leaves `README.md` untouched | `cli` (unit) |
| `run_init` named collision surfaces the error | `run_init(cwd, Some("existing-file"))` where `cwd/existing-file` is a plain file returns `Err` (assert on the message contents) | `cli` (unit) |
| CLI parsing | `Cli::parse_from(["tk", "init", "my-para"])` parses into `Commands::Init { name: Some("my-para") }`; `["tk", "init"]` parses into `Commands::Init { name: None }` | `main` (unit, mirrors the existing `parses_new_with_filename` test) |

## Implementation plan

1. `workspace::InitReport`, `check_collision`, `init` + the two new
   `WorkspaceError` variants, fully unit-tested per the table above.
2. `cli::run_init`, tested with `tempfile::tempdir()` fixtures (no fakes
   needed — `init` is pure filesystem I/O with no editor/UI prompting to
   mock).
3. Wire `Commands::Init { name: Option<String> }` into `main.rs`, calling
   `run_init(&cwd, name.as_deref())` *before* `Workspace::discover` (init
   doesn't require a workspace to already exist) and printing the
   returned message.
4. Manual smoke test: in a scratch directory, run `cargo run -- init`,
   confirm the message and the five dirs; `cargo run -- init demo` in a
   fresh spot, confirm `./demo` is scaffolded; re-run `cargo run -- init`
   in the same spot and confirm the already-complete message; touch a
   stray file where a named target would go and confirm the collision
   error instead of a panic or raw `io::Error` dump.
5. `cargo clippy`, `cargo fmt --check`, `cargo test` clean before calling
   the stories done.
