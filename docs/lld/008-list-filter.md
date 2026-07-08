# LLD: `tk list` substring filter — Story 003 (`list.md`)

Source: [docs/user-stories/list.md](../user-stories/list.md)
Story 003. Module boundaries follow [docs/design.md](../design.md).
Corresponds to roadmap item 4 (`list`).

## Scope

1. `tk list <category> <filter>` narrows rows to those whose Name or Title
   contains `filter` as a case-insensitive substring (`list.md` 003,
   scenarios 1–3).
2. A filter that matches nothing prints `No items in <Category> matching
   "<filter>".` and exits successfully, no header row (`list.md` 003,
   scenario 4).

`<Category>` needs a plural display name for every `Category` variant,
including `Archive` itself (`tk list archive nonexistent` is a valid
invocation now that `list.md` 002 wired `Archive` into `ListCategory`).
`Category::archive_origin_name` already has the four non-`Archive` strings
but panics on `Archive` by design (it answers "which origin subfolder",
a question that doesn't apply to `Archive` itself). Reusing it here would
mean either loosening its contract or adding a special case at the call
site — this LLD instead adds a small, separate `Category::display_name`
covering all five variants, keeping `archive_origin_name`'s existing
contract (and its `unreachable!`) untouched.

### Out of scope

- **`list.md` 004** (empty-category-with-no-filter message) — separate
  story/LLD. This LLD's message only fires when a filter is given and
  matches nothing; listing a category with no filter and no items keeps
  today's behavior (header row over zero data rows) unchanged here.
  `Category::display_name` is exactly what 004 will need too, so it's
  additive scope, not scope this LLD needs to redo.

## `design.md` changes

Not yet applied. `items::list`'s signature in `design.md`'s `items` section
already documents the target shape (`filter: Option<&str>`, matched
case-insensitively against `name` or `title`) — that was written up front
per this project's convention of pre-documenting target shapes. `design.md`
needs one addition: a `Category::display_name` bullet in the `category`
section, next to `archive_origin_name`.

## Module designs

### `category` (extends existing module)

```rust
impl Category {
    /// Plural display name for user-facing messages (`list`'s
    /// no-match/empty messages), covering all five variants including
    /// `Archive` itself. Shares strings with `archive_origin_name` for the
    /// four categories both cover, but is total where `archive_origin_name`
    /// is deliberately partial.
    pub fn display_name(&self) -> &'static str {
        match self {
            Category::Inbox => "Inbox",
            Category::Project => "Projects",
            Category::Area => "Areas",
            Category::Resource => "Resources",
            Category::Archive => "Archive",
        }
    }
}
```

### `items` (extends existing module)

```rust
pub fn list(
    ws: &Workspace,
    category: Category,
    filter: Option<&str>,
) -> Result<Vec<ListedItem>, ItemsError>
```

`filter` is applied as a case-insensitive substring match against `name` or
`title`, after building each `ListedItem` and before the final sort — matching
is content-based (post `infer_title`/fallback), not against raw file content,
so a filter matching a title that only exists because of the Name-fallback
(`list.md` 005) still works correctly since fallback has already happened by
then. Existing callers (`cli::run_list`, and every current `items::list`
test) pass `None` and are otherwise unaffected — `list_at` gains the same
parameter and threads it through unchanged for both the `Archive` branch and
the plain-category branch.

```rust
fn list_at(
    ws: &Workspace,
    category: Category,
    filter: Option<&str>,
    now: SystemTime,
) -> Result<Vec<ListedItem>, ItemsError>
```

A small private helper keeps the match logic in one place and out of the
two branches (`Archive` walk vs. plain category) that both need it:

```rust
fn matches_filter(item: &ListedItem, filter: Option<&str>) -> bool {
    match filter {
        None => true,
        Some(f) => {
            let f = f.to_lowercase();
            item.name.to_lowercase().contains(&f) || item.title.to_lowercase().contains(&f)
        }
    }
}
```

### `cli` (extends existing module)

```rust
pub fn run_list(
    ws: &Workspace,
    category: Category,
    filter: Option<&str>,
) -> anyhow::Result<String>
```

When `items::list` returns rows, rendering is unchanged from today. When it
returns empty **and `filter` was given**, `run_list` returns the no-match
message instead of a header-only table:

```rust
format!(
    "No items in {} matching \"{}\".",
    category.display_name(),
    filter.unwrap()
)
```

An empty result with `filter: None` still renders the header-only table —
that's `list.md` 004's concern, deliberately left alone here (see Out of
scope). This keeps `run_list`'s contract change minimal: same signature
shape as before plus one new optional parameter, one new early-return
branch gated on `filter.is_some()`.

### `main` (extends existing dispatch)

```rust
/// List items in a category.
List {
    category: ListCategory,
    filter: Option<String>,
},
```

```rust
Commands::List { category, filter } => {
    let ws = Workspace::discover(&cwd, home_config.as_deref())
        .context("failed to find a PARA workspace")?;
    let output = cli::run_list(&ws, category.into(), filter.as_deref())?;
    println!("{output}");
}
```

`filter` is a plain positional `Option<String>` (not a flag) matching
`list.md`'s `tk list project web` invocation shape.

## Test plan (TDD — write these first)

| Scenario | Test | Module |
|---|---|---|
| `display_name` covers all five variants, including `Archive` | assert each variant's string, especially `Category::Archive.display_name() == "Archive"` (the one `archive_origin_name` can't answer) | `category` (unit) |
| Filter matches a substring of Name | `items::list_at` with `filter: Some("web")` on `website-redesign`/`my-project` returns only `website-redesign` | `items` (unit, list.md 003 scenario 1) |
| Filter matches a substring of Title even when Name doesn't contain it | `items::list_at` with `filter: Some("redesign")` on a project named `q3-initiative` titled "Website Redesign Phase 2" returns it | `items` (unit, list.md 003 scenario 2) |
| Filter is case-insensitive | `items::list_at` with `filter: Some("WEB")` still matches `website-redesign` | `items` (unit, list.md 003 scenario 3) |
| Filter matching nothing returns an empty vec, not an error | `items::list_at` with `filter: Some("nonexistent")` on a non-empty category returns `Ok(vec![])` | `items` (unit, supports list.md 003 scenario 4) |
| `filter: None` is unaffected (regression) | existing `items::list_at` tests updated to pass `None` and still pass unchanged | `items` (unit, regression) |
| `run_list` renders the no-match message when filter matches nothing | `cli::run_list(&ws, Category::Project, Some("nonexistent"))` on a non-empty `Projects` dir returns exactly `No items in Projects matching "nonexistent".` | `cli` (unit, list.md 003 scenario 4) |
| `run_list` still renders the table when filter matches something | `cli::run_list(&ws, Category::Project, Some("web"))` returns header + only the matching row | `cli` (unit, list.md 003 scenarios 1–3) |
| `run_list` with `filter: None` is unaffected (regression) | existing `run_list` tests updated to pass `None` and still pass unchanged | `cli` (unit, regression) |
| `tk list project web` parses filter as a positional | clap parse test asserting `Commands::List { category: ListCategory::Project, filter: Some("web".into()) }` | `main` (unit) |
| `tk list project` (no filter) still parses `filter: None` | clap parse test, regression | `main` (unit) |

## Implementation plan

1. Add `Category::display_name` and its unit test; watch it fail, then
   implement.
2. Add the `filter` parameter to `items::list_at`/`items::list` plus
   `matches_filter`, with the new filter-matching unit tests; update
   existing `items::list_at` call sites/tests to pass `None`.
3. Add the `filter` parameter to `cli::run_list` and its no-match-message
   branch, with the new unit tests; update existing `run_list` call
   sites/tests to pass `None`.
4. Wire up `main`'s `Commands::List` to take a positional `filter:
   Option<String>` and thread it through to `cli::run_list`; add/update
   parse tests.
5. Mark `list.md` 003 `✅`.
6. Update `docs/roadmap.md`'s `list` row: 003 done, 004 remaining.
7. Update `design.md`'s `category` section with the `display_name` bullet.
8. Manual smoke test: in an initialized PARA system with a couple of
   projects, run `tk list project <substring-of-one-name>`,
   `tk list project <substring-of-one-title>`, `tk list project
   <SAME-BUT-UPPERCASE>`, and `tk list project nonexistent`; confirm the
   filtered row(s) and the exact no-match message text by eye.
9. `cargo clippy`, `cargo fmt --check`, `cargo test` clean before calling
   003 done.
