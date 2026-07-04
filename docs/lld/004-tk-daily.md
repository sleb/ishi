# LLD: `tk daily` + `tk new --daily` (daily.md 001–003, new.md Story 013)

Source: [docs/user-stories/daily.md](../user-stories/daily.md) Stories
001–003, [docs/user-stories/new.md](../user-stories/new.md) Story 013.
Module boundaries follow [docs/design.md](../design.md). Corresponds to
roadmap item 3.

## Scope

1. `tk daily` — create today's Inbox note from the `daily` template the
   first time it's run today; reopen the existing note in `$EDITOR`
   (untouched, no re-render) on any later run the same day.
2. `tk daily` never accepts a filename (today's date is always the name).
3. `tk new --daily` is exactly `tk daily`: same behavior, mutually
   exclusive with `--project`/`--area`/`--resource`, and rejects a
   `filename` argument.
4. **A new `category::Kind` vocabulary**, separate from `Category`, and a
   `Templates::for_kind` replacing `Templates::for_category`. This is a
   larger change than Stories 001–013 strictly require, but it's what
   makes Daily fit the system cleanly instead of as a bolt-on special
   case — see "Why `Kind`, not `Category::Daily`" below. It touches
   `run_new`'s existing signature (`Category` param → `Kind` param), so
   every existing `run_new` call site/test changes too, even though their
   behavior doesn't.

### Why `Kind`, not `Category::Daily`

The first pass at this LLD reached for the obvious move — special-case
`daily` directly wherever `Category` shows up (`Templates::for_category`
gets a `Daily` arm, `main` branches around `into_category()` when
`--daily` is set, `cli::run_daily` reaches into `ws.config.templates.daily`
by hand instead of through the category lookup). That works, but it means
template selection is sometimes a `Category` lookup and sometimes hardcoded
`daily`-is-special logic — an inconsistency, not just an implementation
detail.

The actual issue: `Category` answers "where does this item live" (the
filing vocabulary `mv`/`list`/`status`/`Archive` need), and that's a
different question from "what is `new` creating" (a creation vocabulary
that decides template + control flow). They coincide for four of five
values, but a daily note has no folder of its own (it's always
`Category::Inbox`, just a different template and a create-vs-reopen
lifecycle), and `Category::Archive` has no creation behavior at all (items
only arrive there via `mv`, never `new` — which is why `Templates::for_category`
already had to `panic!` on it, the same smell from the other direction).

So this LLD introduces `category::Kind { Inbox, Project, Area, Resource,
Daily }`, with `Kind::category(&self) -> Category` mapping each to its
filing location (`Daily` → `Inbox`). `Templates::for_kind(&self, kind: Kind)
-> &str` replaces `Templates::for_category` and is **total** — no panic
branch, since there's no `Kind::Archive` to be missing a template for. Full
rationale is now in `docs/design.md`'s `category` section ("Filing
vocabulary vs. creation vocabulary"), so this doesn't need to be
rediscovered next time a new creatable thing shows up.

### Out of scope

- Any *behavioral* change to `--project`/`--area`/`--resource`/plain
  `tk new` — only the internal `Category` → `Kind` parameter rename for
  `run_new`, which is why item 4 above is in scope even though it isn't
  one of the acceptance criteria.
- `.tick.toml`-configurable templates (roadmap item 2's remaining
  TOML-layering work) — the `daily` template is added as a Rust-level
  default only, same status as `note`/`project`/`area`/`resource` today.

## `design.md` changes

Already applied directly to `docs/design.md` (not deferred to "once this
lands", since the vocabulary distinction is meant to be load-bearing
documentation going forward, not just an implementation note):

- `category` section: added `enum Kind` and `Kind::category()`, plus the
  "Filing vocabulary vs. creation vocabulary" subsection explaining why
  they're two types and the rule of thumb for future additions.
- `config` section: `Templates::for_category` → `Templates::for_kind`.
- `items` section: added `item_path` (pure path computation, factored out
  of `create` so `cli::run_daily` can check existence without duplicating
  the directory-vs-flat-file branch).
- `editor` section: `Editor` trait gained `open(&self, path) -> Result<()>`.
- `cli` section: `run_new`'s `category: Category` param is now `kind: Kind`
  (deriving `Category` via `kind.category()` where it's still needed);
  added `DailyOutcome`, `daily_note_exists`, `run_daily`.

This LLD's job is implementing that already-documented design.

## Module designs

### `category` (extends existing module)

```rust
/// What `tk new`/`tk daily` create — a different vocabulary from
/// `Category` (where an item is filed). See design.md's "Filing
/// vocabulary vs. creation vocabulary" for why these are two types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Inbox,
    Project,
    Area,
    Resource,
    Daily,
}

impl Kind {
    /// The `Category` this kind files into. `Daily` maps to `Inbox` — a
    /// daily note has no folder of its own.
    pub fn category(&self) -> Category {
        match self {
            Kind::Inbox | Kind::Daily => Category::Inbox,
            Kind::Project => Category::Project,
            Kind::Area => Category::Area,
            Kind::Resource => Category::Resource,
        }
    }
}
```

### `config` (extends existing module)

```rust
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
            note: /* unchanged */,
            daily: "---\ndate: {{date}}\nlast_updated: {{date}}\n---\n# {{date}}\n\n## Tasks\n\n[ ] -\n\n## Notes\n\n{{cursor}}\n".to_string(),
            project: /* unchanged */,
            area: /* unchanged */,
            resource: /* unchanged */,
        }
    }
}

impl Templates {
    pub fn for_kind(&self, kind: Kind) -> &str {
        match kind {
            Kind::Inbox => &self.note,
            Kind::Daily => &self.daily,
            Kind::Project => &self.project,
            Kind::Area => &self.area,
            Kind::Resource => &self.resource,
        }
    }
}
```

`daily`'s default text matches what `README.md`'s Configuration section
already documents. `for_category`/its `Archive` panic are deleted, not
kept alongside `for_kind` — there's exactly one template-lookup path now.

### `items` (extends existing module)

```rust
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
```

Behavior-preserving refactor: `create_dir_all(path.parent())` creates the
scaffolded project/area directory (`item_path`'s directory-style branch
puts `index.md` one level inside it) or the flat category directory
(`item_path`'s other branch puts the file directly inside it) — same two
targets `create` creates today, just derived from one path instead of
duplicating the branch. `create` still takes `Category`, not `Kind` — it
only ever needs to know *where* to write, which is exactly what `Category`
answers; `cli` is the layer that resolves a `Kind` to a `Category` before
calling in.

### `editor` (extends existing module)

```rust
pub trait Editor {
    fn capture(&self, seed: &str) -> Result<(String, String), EditorError>;

    /// Opens `$EDITOR` directly on an existing file at `path` — no scratch
    /// file, no seed content, no filename inference. Used to reopen an
    /// already-created daily note untouched.
    fn open(&self, path: &Path) -> Result<(), EditorError>;
}

impl Editor for RealEditor {
    // capture: unchanged

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
```

`$EDITOR`-not-set detection is duplicated from `capture` rather than
extracted, since it's two lines and the two methods otherwise build
completely different `Command` invocations (one on a scratch file with a
`+<line>` arg, one directly on `path` with no cursor arg).

Every existing `impl Editor for ...` in `src/cli.rs`'s test fakes
(`FakeEditor`, `PanicEditor`, and the various inline `RecordingEditor`s)
needs an `open` method added to keep compiling — each can just
`unimplemented!("not exercised by this test")`, matching the existing
`FakeUi::choose` convention, since none of those tests exercise the daily
path.

### `cli` (extends existing module)

`run_new` changes its category parameter to `kind: Kind` and derives
`Category` only where it's still needed (`items::create`, `is_directory_style`):

```rust
pub fn run_new(
    ws: &Workspace,
    editor: &dyn Editor,
    ui: &mut dyn Ui,
    kind: Kind,
    filename: Option<String>,
) -> anyhow::Result<PathBuf> {
    let category = kind.category();
    let template = ws.config.templates.for_kind(kind);
    // ...unchanged body, `ws.config.templates.for_category(category)` →
    // `template`, `category.is_directory_style()` →
    // `category.is_directory_style()` (via the `category` local above),
    // `items::create(ws, category, ...)` unchanged.
}
```

New daily-specific pieces:

```rust
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
```

`run_daily` calls `templates.for_kind(Kind::Daily)` — the same lookup path
`run_new` uses — rather than reaching into `templates.daily` directly, so
there's exactly one way template selection happens in this codebase.

`daily_note_exists` and `run_daily` both compute `item_path` for today —
duplicated work (one extra `exists()` syscall), not a duplicated decision:
`main` needs the answer before it decides what to print, `run_daily` needs
it again to decide create-vs-reopen. Threading a pre-computed answer
through instead would mean either a stateful `DailyPlan` type or an extra
parameter whose only job is "don't check twice" — not worth it for one
cheap filesystem stat in a single-user CLI with no concurrent daily-note
writers.

No `Ui` parameter on `run_daily`: unlike `run_new`, there's no filename to
confirm and no choice to prompt — non-interactive create or direct
reopen, nothing else. `run_new` is never called with `Kind::Daily` — `main`
routes `Kind::Daily`/`--daily` to `run_daily` before `run_new` is reached
(see below), so `run_new`'s capture-or-named-file shape never needs to
accommodate daily's create-or-reopen lifecycle.

### `main` (extends existing `Commands`/`NewCategory`)

```rust
#[derive(Debug, PartialEq, Subcommand)]
enum Commands {
    New {
        filename: Option<String>,
        #[command(flatten)]
        category: NewCategory,
    },
    /// Create (or open) today's daily note in the Inbox.
    Daily,
    Init { name: Option<String> },
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Args)]
#[group(multiple = false)]
struct NewCategory {
    #[arg(long)]
    project: bool,
    #[arg(long)]
    area: bool,
    #[arg(long)]
    resource: bool,
    /// Create (or open) today's daily note instead of an Inbox file.
    #[arg(long, conflicts_with = "filename")]
    daily: bool,
}

impl NewCategory {
    fn into_kind(self) -> Kind {
        if self.project {
            Kind::Project
        } else if self.area {
            Kind::Area
        } else if self.resource {
            Kind::Resource
        } else if self.daily {
            Kind::Daily
        } else {
            Kind::Inbox
        }
    }
}
```

`into_category` is renamed `into_kind` and returns `Kind` uniformly,
including `Daily` — no more branching around it before the call, the way
the first draft of this LLD did. `main`'s dispatch still has to special-case
daily's *control flow* (create-or-reopen vs. capture-or-named-file are
genuinely different shapes), just not its *template lookup*:

```rust
fn run_daily_command(ws: &Workspace) -> anyhow::Result<()> {
    if cli::daily_note_exists(ws) {
        println!("Opening $EDITOR...");
    }
    let editor = RealEditor;
    if let cli::DailyOutcome::Created(path) = cli::run_daily(ws, &editor)? {
        println!("Created {}", path.display());
    }
    Ok(())
}
```

Dispatch, in `main`'s match on `cli.command`:

```rust
Commands::Daily => {
    let ws = Workspace::discover(&cwd).context("failed to find a PARA workspace")?;
    run_daily_command(&ws)?;
}
Commands::New { filename, category } if category.into_kind() == Kind::Daily => {
    let ws = Workspace::discover(&cwd).context("failed to find a PARA workspace")?;
    run_daily_command(&ws)?;
}
Commands::New { filename, category } => {
    let ws = Workspace::discover(&cwd).context("failed to find a PARA workspace")?;
    if filename.is_none() {
        println!("Opening $EDITOR...");
    }
    let editor = RealEditor;
    let mut ui = TerminalUi;
    let path = cli::run_new(&ws, &editor, &mut ui, category.into_kind(), filename)?;
    println!("Created {}", path.display());
}
```

(Match guards on an enum variant with additional fields need the
non-guarded arm listed after, as above, so `filename`/`category` remain
accessible in both `New` arms without restructuring `Commands`.)

`conflicts_with = "filename"` rejects `tk new some-name --daily` at parse
time (new.md 013's second scenario); the existing `#[group(multiple =
false)]` already rejects `--daily --project` (its third scenario), the
same mechanism that already rejects `--project --area`. `tk daily
some-name` (daily.md 003's second scenario) is rejected automatically too
— `Commands::Daily` is a unit variant, so clap treats any extra positional
as an unrecognized argument, no extra code needed.

## Test plan (TDD — write these first)

| Scenario | Test | Module |
|---|---|---|
| `Kind::category()` maps correctly, `Daily` → `Inbox` | one assertion per variant | `category` (unit) |
| `for_kind` maps to the matching template, no `Archive`/panic case exists | `Templates::default().for_kind(Kind::X) == templates.X` for all five | `config` (unit, replaces `for_category_maps_to_matching_template`/`for_category_panics_on_archive`) |
| `daily` template default matches README | `Templates::default().daily` contains `## Tasks` / `## Notes` | `config` (unit) |
| `item_path` for a flat category (no I/O) | `item_path(ws, Category::Inbox, "2026-07-04")` → `.../0-Inbox/2026-07-04.md` | `items` (unit) |
| `item_path` for a directory-style category | `item_path(ws, Category::Project, "foo")` → `.../1-Projects/foo/index.md` | `items` (unit, mirrors existing `create` tests) |
| `create` still creates flat files / scaffolded dirs | existing `items` tests pass unchanged after the refactor | `items` (regression) |
| Existing `run_new` behavior is unchanged under `Kind` | every existing `cli` test updated from `Category::X` to `Kind::X` as the fourth arg, same assertions | `cli` (regression) |
| First run creates non-interactively | `run_daily` with no existing note → `DailyOutcome::Created`, file at `0-Inbox/<today>.md`, content renders `daily` template with `{{title}}`/`{{date}}` as today, editor's `open` never called (panic-on-open fake) | `cli` (unit, daily.md 001) |
| Second run reopens without re-rendering | pre-write a note with custom content, run `run_daily` → `DailyOutcome::Reopened`, `open` called with the existing path, file content byte-for-byte unchanged, `create`'s template not written | `cli` (unit, daily.md 002 scenario 1) |
| `$EDITOR` unset surfaces as an error on reopen | pre-write a note, fake `Editor::open` returns `Err(EditorError::NotSet)` → `run_daily` propagates the error | `cli` (unit, daily.md 002 scenario 2) |
| Filename is always today's date | `run_daily` → created path's file stem equals today's `%Y-%m-%d` (computed the same way in the test, like existing date-rendering tests do) | `cli` (unit, daily.md 003) |
| `daily_note_exists` reflects the filesystem, no I/O beyond a stat | `false` before creation, `true` after `items::create` writes today's note | `cli` (unit) |
| `tk new --daily` parses | `Cli::parse_from(["tk","new","--daily"])` → `NewCategory { daily: true, .. }`, `filename: None` | `main` (unit) |
| `tk new --daily <name>` rejected | `Cli::try_parse_from(["tk","new","--daily","x"])` → `Err` | `main` (unit, new.md 013 scenario 2) |
| `tk new --daily --project` rejected | `Cli::try_parse_from(["tk","new","--daily","--project"])` → `Err` | `main` (unit, new.md 013 scenario 3) |
| `tk daily` parses | `Cli::parse_from(["tk","daily"])` → `Commands::Daily` | `main` (unit, daily.md 003 scenario 2) |
| `tk daily <name>` rejected | `Cli::try_parse_from(["tk","daily","x"])` → `Err` | `main` (unit) |
| `into_kind` maps every flag combination, defaults to `Inbox` | one test per flag, mirrors existing `into_category_defaults_to_inbox` | `main` (unit) |

## Implementation plan

1. Add `Kind` + `Kind::category()` to `src/category.rs`, with its unit
   tests. `design.md`/this LLD are already updated, so this is pure
   implementation of already-agreed design.
2. Rename `Templates::for_category` to `for_kind` (parameter type `Kind`,
   total match, no panic arm); add the `daily: String` field + default;
   update/replace its existing tests. Watch the new tests fail, then
   implement.
3. Extract `items::item_path` out of `create`; add its unit tests; confirm
   all existing `items` tests still pass unchanged.
4. Add `Editor::open` to the trait, implement on `RealEditor`, and add a
   stub `open` (via `unimplemented!`) to every test fake in `src/cli.rs` so
   the crate keeps compiling before the new `cli` tests are written.
5. Change `cli::run_new`'s category parameter to `kind: Kind`; update
   every existing call site/test in `src/cli.rs` from `Category::X` to
   `Kind::X`; confirm all existing `run_new` tests still pass unchanged
   otherwise (this step should produce zero behavior change, only a
   parameter rename).
6. Add `cli::DailyOutcome`, `cli::daily_note_exists`, `cli::run_daily`, and
   their unit tests (fakes: a panic-on-open editor for the create case, a
   recording/fixed-response editor for the reopen case, an
   `Err(NotSet)`-returning editor for the error case).
7. Rename `NewCategory::into_category` to `into_kind` (return type `Kind`,
   covering `daily`); add `Commands::Daily` and the `daily` field +
   `conflicts_with` on `NewCategory` in `main.rs`; add `run_daily_command`
   and wire both dispatch arms; add/update the `main`-level parse tests.
8. Mark daily.md Stories 001–003 and new.md Story 013 `✅`.
9. Update `docs/roadmap.md`'s status snapshot and item 3 write-up to "Done".
10. Manual smoke test: in a scratch PARA dir, run `tk daily` (creates
    `0-Inbox/<today>.md`, prints `Created ...`), run it again with
    `$EDITOR` set to a real editor (opens the same file, prints `Opening
    $EDITOR...` first, no `Created` line, content preserved), then try
    `tk new --daily`, `tk new --daily --project`, `tk daily some-name`,
    and confirm `tk new my-file`/`--project`/`--area`/`--resource` are
    all unaffected.
11. `cargo clippy`, `cargo fmt --check`, `cargo test` clean before calling
    the stories done.
