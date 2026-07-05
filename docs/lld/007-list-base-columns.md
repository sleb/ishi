# LLD: `tk list` base columns — Stories 001, 005 (`list.md`)

Source: [docs/user-stories/list.md](../user-stories/list.md)
Stories 001, 005. Module boundaries follow [docs/design.md](../design.md).
Corresponds to roadmap item 4 (`list`).

## Scope

1. Listing a `project`/`area` category prints `NAME`/`TITLE`/`UPDATED`
   columns, one row per item, sourced from each item's `index.md`
   (`list.md` 001, scenarios 1–2).
2. Listing a `resource`/`inbox` category prints the same columns sourced
   from the flat file itself, not a directory (`list.md` 001, scenario 3).
3. Rows are sorted alphabetically by Name (`list.md` 001, intro + all
   scenarios).
4. Title falls back to Name when the item has no Markdown heading,
   including when frontmatter is present but nothing follows it
   (`list.md` 005).

Story 005 is folded in here rather than deferred: `items::infer_title`
returning `Option<String>` and `items::list` needing to do *something* with
`None` are the same design decision — implementing the fallback is not
extra scope, it's what makes `list` total. Leaving it unhandled would mean
either a partially-implemented match or a panic on the first untitled note.

### Out of scope

- **`list.md` 002** (archive origin-qualified names) — needs `Category::Archive`
  to be listed by walking origin subfolders and prefixing Name with
  `<OriginCategory>/`, which doesn't exist until `mv` (roadmap item 6) writes
  items into that shape. Separate LLD; `Category::Archive` is not wired into
  the CLI's category argument yet.
- **`list.md` 003** (substring filter) — separate LLD; adds a `filter:
  Option<&str>` parameter to `items::list` and `cli::run_list`.
- **`list.md` 004** (empty-category/no-match message) — separate LLD; needs
  category display names (`"Projects"`, `"Resources"`, ...) that don't exist
  yet. This LLD's `items::list` already returns `Ok(vec![])` for a category
  whose directory doesn't exist yet (rather than erroring), so 004 can build
  the message on top without changing `list`'s contract.

## `design.md` changes

`items::list`, `ListedItem`, and `infer_title` are already documented in
`design.md`'s `items` section (they were written up front as the target
shape for all of `list.md`), including `infer_title`'s delegation to the
`gist` crate's `parser::first_heading_text` — no changes needed there.
`design.md` gains one new bullet for `cli::run_list` (see below); `main.rs`'s
`Commands::List`/`ListCategory` mapping is not documented in `design.md`,
consistent with `NewCategory`/`config_target` also being left out as
`main`-only argv plumbing.

## Module designs

### `items` (extends existing module)

```rust
pub struct ListedItem {
    pub name: String,
    pub title: String,
    pub updated_days_ago: u64,
}

/// Thin wrapper over `gist::parser::first_heading_text`: skips a leading
/// YAML frontmatter block if present, then returns the first Markdown
/// heading line's text (any `#` level), or `None` if none is found.
/// Frontmatter-only content with no heading after it returns `None`, not
/// the frontmatter block's own text.
pub fn infer_title(content: &str) -> Option<String>

/// Lists `category`'s items: for a directory-style category (`Project`/
/// `Area`), one row per subdirectory, sourced from its `index.md`; for a
/// flat category (`Resource`/`Inbox`), one row per file, name being the
/// file stem. Rows are sorted alphabetically by name. Returns `Ok(vec![])`
/// if `category`'s directory doesn't exist yet, rather than erroring —
/// an empty/not-yet-created category is a normal state, not a fault.
pub fn list(ws: &Workspace, category: Category) -> Result<Vec<ListedItem>, ItemsError>
```

`infer_title` calls `gist::parser::first_heading_text` directly rather than
hand-rolling the frontmatter-skip-then-find-heading traversal — the same
function `editor::suggest_filename` calls for its own heading-detection
step. Per `design.md`'s `gist` section, `items` and `editor` still don't
depend on each other; both depend on `gist` instead, so the semantics stay
identical without either module hand-rolling (or copy-pasting) the
traversal. Unlike `suggest_filename`, `infer_title` has no
fallback-to-first-line or fallback-to-timestamp behavior: a missing heading
is `None`, full stop, and the fallback-to-Name decision is `list`'s job (per
`design.md`: "falling back to `name` if it returns `None`"), not
`infer_title`'s.

`list` is implemented as a thin public wrapper over a `SystemTime`-parameterized
helper, the same determinism pattern `editor::suggest_filename`/
`suggest_filename_at` already uses, so age-in-days is testable without racing
the system clock:

```rust
fn list_at(ws: &Workspace, category: Category, now: SystemTime) -> Result<Vec<ListedItem>, ItemsError>
```

`updated_days_ago` is computed as `now.duration_since(modified).as_secs() / 86400`
(whole days, floored) from the mtime of the `index.md` (directory-style) or
the file itself (flat-style) — the same source `design.md` specifies `status`
will later reuse. `now.duration_since` on a `modified` time in the future
(clock skew) is defused via `.unwrap_or_default()`, reading as `0` ("today")
rather than erroring; this is a robustness fallback, not a tested scenario.

A missing category directory is detected via `fs::read_dir`'s `io::ErrorKind::NotFound`
and mapped to `Ok(vec![])`; any other `io::Error` still propagates through
`ItemsError::Io` unchanged.

### `cli` (extends existing module)

```rust
/// Formats a raw day-count the way `list`/`status`/`review` all render
/// ages: `"today"`, `"1 day ago"`, `"N days ago"`.
fn format_age(days: u64) -> String

/// Renders `items::list`'s rows as the `NAME`/`TITLE`/`UPDATED` table:
/// header first, then one row per item, each column left-justified to
/// `3 +` the longest value in that column (including its header) so
/// columns line up regardless of content width.
pub fn run_list(ws: &Workspace, category: Category) -> anyhow::Result<String>
```

`run_list` returns the fully rendered multi-line `String` rather than
printing directly, mirroring `run_init`/`run_config_init`'s existing
convention of returning a message for `main` to print — keeps `cli`'s
business logic (which is genuinely the table-formatting/age-wording, not
terminal I/O itself) unit-testable without capturing stdout.

Column width is `max(header_len, all_values_len) + 3`, a fixed 3-space gap
applied uniformly to every column (verified against `list.md`'s Name column
across all four of its worked examples, which land exactly on this rule).
The `Updated` column is the last column and is not padded — trailing
whitespace on a row would be pointless and untestable-by-eye. Tests assert
against this algorithm's own output rather than transcribing `list.md`'s
ASCII tables byte-for-byte; those tables are illustrative and drift by a
column or two on the Title field for the longer-title examples, which reads
as a hand-formatting artifact rather than a distinct intended rule.

### `main` (extends existing dispatch)

```rust
#[derive(Debug, Clone, Copy, PartialEq, clap::ValueEnum)]
enum ListCategory { Project, Area, Resource, Inbox }

impl From<ListCategory> for Category { /* 1:1 mapping */ }

// Commands variant:
/// List items in a category.
List { category: ListCategory },
```

`ListCategory` deliberately excludes `Archive` for now (see Out of scope) —
adding it early would let `tk list archive` run against `items::list`'s
current flat-style handling and print something plausible-looking but wrong
(no origin-qualified names, no subfolder walk), which is worse than the
command not existing yet. `category.rs` itself stays clap-free, matching
`Kind`/`Category`'s existing non-dependence on `clap` — `ListCategory` is
`main`-only plumbing, the same shape as `NewCategory`.

## Test plan (TDD — write these first)

| Scenario | Test | Module |
|---|---|---|
| Directory-style category lists dir name + `index.md` title + `index.md` mtime age | `items::list_at` on two project dirs (mtimes set via `File::set_modified`) returns `ListedItem`s with correct name/title/`updated_days_ago` | `items` (unit, list.md 001 scenario 1) |
| Flat-file category lists file stem + file title + file mtime age | `items::list_at` on a `Resource` dir with `api-notes.md` returns name `"api-notes"` (no extension) | `items` (unit, list.md 001 scenario 3) |
| Rows sorted alphabetically by name regardless of directory iteration order | `items::list_at` returns `my-project` before `website-redesign` | `items` (unit, list.md 001 intro) |
| Missing category directory returns empty list, not an error | `items::list_at` on a workspace with no `2-Areas` dir yet returns `Ok(vec![])` | `items` (unit, supports list.md 004 later without a contract change) |
| No heading in content returns `None` | `infer_title("plain text\nno heading")` is `None` | `items` (unit, list.md 005 scenario 1 — sanity check that `infer_title` delegates to `gist::parser::first_heading_text`, not a re-test of `gist`'s own traversal edge cases) |
| Frontmatter present but no heading after it returns `None` | `infer_title("---\nk: v\n---\nplain text")` is `None` | `items` (unit, list.md 005 scenario 2 — same delegation sanity check) |
| `list` falls back to Name when `infer_title` is `None` | `items::list_at` on an inbox file with no heading returns `title == name == "quick-thought"` | `items` (unit, list.md 005) |
| Age formatting: today / 1 day / N days | `format_age(0) == "today"`, `format_age(1) == "1 day ago"`, `format_age(21) == "21 days ago"` | `cli` (unit) |
| `run_list` renders header + aligned rows for a directory-style category | `cli::run_list(&ws, Category::Project)` on the `website-redesign`/`my-project` fixture matches the algorithm's expected output | `cli` (unit, list.md 001 scenario 1) |
| `run_list` renders a flat-file category | same, for `Category::Resource` with `api-notes.md` | `cli` (unit, list.md 001 scenario 3) |
| `run_list` renders a single-row area category | same, for `Category::Area` with `health` | `cli` (unit, list.md 001 scenario 2) |
| `tk list project`/`area`/`resource`/`inbox` parse to `Commands::List` | clap parse test asserting `ListCategory` variant per subcommand argument | `main` (unit) |

## Implementation plan

1. Add `items::infer_title` as a thin wrapper over `gist::parser::first_heading_text`,
   plus a couple of delegation sanity-check tests (no-heading,
   frontmatter-then-no-heading); watch them fail, then implement. Full
   frontmatter/heading traversal edge cases are `gist`'s own test coverage,
   not re-derived here.
2. Add `items::ListedItem` and `items::list_at`/`items::list` with their
   unit tests (directory-style, flat-style, sorting, missing-dir-is-empty,
   title fallback via `infer_title`), using `std::fs::File::set_modified`
   to backdate fixture mtimes deterministically — no new dependency needed
   (stable since Rust 1.75).
3. Add `cli::format_age` and its unit tests.
4. Add `cli::run_list` and its rendering unit tests, built on step 2/3's
   output directly (no filesystem access in these tests beyond what
   `items::list` already needs).
5. Wire up `main`'s `ListCategory`/`Commands::List` and dispatch (`Workspace::discover`
   then `cli::run_list`, printing the returned string), with its own parse
   test.
6. Mark `list.md` 001 and 005 `✅`, and add a one-line note next to 001 that
   005 is folded into this LLD (so a future reader of `list.md` doesn't look
   for a separate 005 LLD).
7. Update `docs/roadmap.md`'s `list` row from "Not started" to reflect 001/005
   done, 002/003/004 remaining.
8. Manual smoke test: `tk init`, create two projects with `tk new --project`,
   backdate one's `index.md` mtime with `touch -t`, run `tk list project` and
   confirm column alignment and age wording by eye; repeat for `tk list area`
   and `tk list resource`.
9. `cargo clippy`, `cargo fmt --check`, `cargo test` clean before calling
   001/005 done.
