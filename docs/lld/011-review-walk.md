# LLD: `tk review` — Story 001 (`review.md`)

Source: [docs/user-stories/review.md](../user-stories/review.md) Story 001.
Module boundaries follow [docs/design.md](../design.md). Corresponds to
roadmap item 7 (`review`) — this LLD covers only the walk itself, not
`[a]rchive`/`[k]eep`/`[s]kip`'s effects.

## Scope

1. `review` walks every `Project`, then every `Area`, each group sorted
   alphabetically by name (review.md 001, scenario 1).
2. Each prompt's header line is `Project: <name> (last updated <age>)`
   (scenario 2).
3. An area's header line says `Area:` instead of `Project:` (scenario 3).
4. A system with no projects or areas prints a "nothing to review" message
   and exits successfully without prompting (scenario 4).
5. The walk ends silently after the last item — no trailing prompt, no
   trailing message (scenario 5).

`cli::Ui` gains a second prompt shape (`choose`'s header + `[c]har`-style
options) and a plain informational method (`info`), both introduced here
because story 001 is the first caller `choose` ever gets — it was declared
in `design.md` ahead of any command using it, so its exact shape was never
exercised. Both changes are scoped tight to what this story's exact printed
output requires (see `cli` below), not a general-purpose prompting
redesign.

### Out of scope

- What `[a]rchive` actually does to the filesystem (review.md 002) — calls
  `items::mv`, which now exists ([012-tk-move.md](012-tk-move.md), per
  [move.md](../user-stories/move.md) Story 001). Still a separate LLD: this
  one covers only the walk, not `[a]rchive`'s effects.
- What `[k]eep`/`[s]kip` do to `last_reviewed` frontmatter (review.md 003)
  — needs `items::write_last_reviewed`, which also doesn't exist yet.
  Separate LLD.
- Because of the above, this LLD's `review::run` reads the letter
  `ui.choose` returns only to know an item was answered (so it can advance
  to the next one) — it does not yet branch on *which* letter was chosen.
  Stories 002/003 add match arms over that same return value; they don't
  change the walk's control flow.
- `status.md` 004 (empty-category "not yet reviewed" wording) — already
  covered by `items::StatusItem::reviewed_days_ago`, unrelated to the walk.

## `design.md` changes

Not yet applied; deferred until this LLD lands.

- `items` section gains `review_items`.
- `cli` section's `Ui` trait entry updates `choose`'s signature and adds
  `info`.
- `review` section's `run` signature is confirmed as
  `Result<(), ReviewError>` (previously undeclared return type), with a
  note that the loop doesn't yet interpret the chosen letter — deferred to
  002/003.

## Module designs

### `items` (extends existing module)

```rust
/// Sorted alphabetically by name; same mtime-sourced `updated_days_ago`
/// and title inference as `status`'s per-item rows. Reuses
/// `StatusItem`/`status_items_for` rather than a second directory-scan
/// implementation — review's prompt only needs `name`/`updated_days_ago`
/// today, but `title`/`reviewed_days_ago` come along for free and story
/// 003's `[k]eep` path will want the same fresh-content read regardless.
pub fn review_items(
    ws: &Workspace,
    category: Category,
) -> Result<Vec<StatusItem>, ItemsError> {
    status_items_for(ws, category, SystemTime::now(), chrono::Local::now().date_naive())
}
```

No change to `status_items_for`/`build_status_item` themselves — `review_items`
is a thin `pub` entry point next to `status`'s own, same as `status` wraps
`status_at`.

### `cli` (extends existing module)

```rust
pub trait Ui {
    fn confirm(&mut self, prompt: &str, default: &str) -> Result<String, UiError>;

    /// `header` is printed on its own line; `options` are rendered as
    /// `[c]est  [c]est...`, joined by two spaces with no space before the
    /// trailing `?` — e.g. `[k]eep  [a]rchive  [s]kip?` (review.md 001
    /// scenarios 2-3's exact prompt shape). Loops on unrecognized input the
    /// same way the previous single-bracket form did.
    fn choose(&mut self, header: &str, options: &[(char, &str)]) -> Result<char, UiError>;

    /// A plain informational line, no prompt/response — currently only
    /// `review`'s "Nothing to review." message (review.md 001 scenario 4).
    fn info(&mut self, message: &str);
}
```

`choose`'s old signature (`prompt: &str, options: &[&str]`) has no real
caller yet (grep confirms it's declared and stubbed in every `Ui` impl but
never invoked by `run_new`/`run_daily`/etc.), so this is a shape change,
not a breaking one — `TerminalUi::choose` gets rewritten to the two-line
form, and the two test-only `FakeUi::choose` stubs (currently
`unimplemented!()`) update their signatures to match, no call sites to fix.

```rust
impl Ui for TerminalUi {
    fn choose(&mut self, header: &str, options: &[(char, &str)]) -> Result<char, UiError> {
        println!("{header}");
        let rendered = options
            .iter()
            .map(|(c, rest)| format!("[{c}]{rest}"))
            .collect::<Vec<_>>()
            .join("  ");
        loop {
            print!("  {rendered}? ");
            io::stdout().flush()?;
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            if let Some(choice) = input.trim().to_lowercase().chars().next()
                && options.iter().any(|(c, _)| *c == choice)
            {
                return Ok(choice);
            }
        }
    }

    fn info(&mut self, message: &str) {
        println!("{message}");
    }
}
```

### `review` (new module)

```rust
#[derive(Debug, thiserror::Error)]
pub enum ReviewError {
    #[error(transparent)]
    Items(#[from] items::ItemsError),
    #[error(transparent)]
    Ui(#[from] cli::UiError),
}

/// Walks every `Project`, then every `Area`, alphabetically within each
/// group (review.md 001 scenario 1), prompting once per item via
/// `ui.choose`. If both groups are empty, reports via `ui.info` and
/// returns immediately without prompting (scenario 4); otherwise ends
/// silently after the last item (scenario 5) — no summary line. The
/// `char` `ui.choose` returns is currently discarded: interpreting
/// `[a]rchive`/`[k]eep`/`[s]kip` is story 002/003's job, added as match
/// arms on this same call site by those LLDs, not a new loop shape.
pub fn run(ws: &Workspace, ui: &mut dyn Ui) -> Result<(), ReviewError> {
    let projects = items::review_items(ws, Category::Project)?;
    let areas = items::review_items(ws, Category::Area)?;

    if projects.is_empty() && areas.is_empty() {
        ui.info("Nothing to review.");
        return Ok(());
    }

    for item in &projects {
        prompt_one(ui, "Project", item)?;
    }
    for item in &areas {
        prompt_one(ui, "Area", item)?;
    }
    Ok(())
}

fn prompt_one(ui: &mut dyn Ui, label: &str, item: &items::StatusItem) -> Result<(), ReviewError> {
    let header = format!(
        "{label}: {} (last updated {})",
        item.name,
        format_age(item.updated_days_ago)
    );
    ui.choose(&header, &[('k', "eep"), ('a', "rchive"), ('s', "kip")])?;
    Ok(())
}

/// Duplicated from `cli::format_age` (not shared) — `review` depends on
/// `cli` only for the `Ui`/`UiError` types passed into it; pulling in a
/// formatting helper the other direction would invert the dependency
/// `design.md`'s component diagram draws (`cli` calls `review::run`, never
/// the reverse). Four lines, identical behavior.
fn format_age(days: u64) -> String {
    match days {
        0 => "today".to_string(),
        1 => "1 day ago".to_string(),
        n => format!("{n} days ago"),
    }
}
```

`lib.rs` gains `pub mod review;`.

### `main` (extends existing `Commands` enum)

```rust
enum Commands {
    // ...existing New/Daily/Init/Config/List/Completions/Status...

    /// Walk every project and area, prompting keep/archive/skip.
    Review,
}
```

Dispatch, no `cli::run_review` wrapper needed — unlike `run_list`/`run_status`,
`review::run` already does its own prompting via `Ui`, so there's no
extra formatting step for `main` to own in between:

```rust
Commands::Review => {
    let ws = Workspace::discover(&cwd, home_config.as_deref())
        .context("failed to find a PARA workspace")?;
    let mut ui = TerminalUi;
    review::run(&ws, &mut ui)?;
}
```

## Test plan (TDD — write these first)

| Scenario | Test | Module |
|---|---|---|
| `review_items` returns Project rows sorted alphabetically with correct `updated_days_ago` | set two projects' mtimes, assert order + ages | `items` (unit, groundwork for review.md 001) |
| `review_items` returns the same shape for `Area` | same as above, `Category::Area` | `items` (unit, groundwork) |
| Projects walked before areas, alphabetical within each group | `FakeUi` with projects `my-project`/`website-redesign` and areas `finances`/`health`; assert `choose` headers arrive in that exact order | `review` (unit, review.md 001 scenario 1) |
| Project header matches the documented format | one project, mtime 12 days ago; assert header string is exactly `Project: website-redesign (last updated 12 days ago)` | `review` (unit, scenario 2) |
| Area header uses `Area:` | one area, mtime 4 days ago; assert header is `Area: finances (last updated 4 days ago)` | `review` (unit, scenario 3) |
| Empty workspace reports nothing to review, no prompts | no projects/areas; assert `info("Nothing to review.")` called and `choose` never called; `run` returns `Ok(())` | `review` (unit, scenario 4) |
| Walk ends after the last item, no extra prompt | one area `health`, `FakeUi` returns `'k'`; assert exactly one `choose` call and `run` returns `Ok(())` | `review` (unit, scenario 5) |
| `Ui::choose` renders header line then `[k]eep  [a]rchive  [s]kip?` and re-prompts on invalid input | drive `TerminalUi::choose` (or a thin pure renderer extracted for testability, if `print!`/stdin makes `TerminalUi` itself impractical to unit test directly) | `cli` (unit) |
| `Ui::info` prints the message as-is | trivial | `cli` (unit) |
| `tk review` dispatches to `review::run` against a discovered workspace | integration test mirroring `tests/cli_daily.rs`'s shape | `tests/cli_review.rs` (integration) |

## Implementation plan

1. Add `items::review_items` and its unit tests; watch them fail, then
   implement as a thin wrapper over the existing `status_items_for`.
2. Change `cli::Ui::choose`'s signature and add `cli::Ui::info`; update
   `TerminalUi`'s impl and the two existing test-only `FakeUi` stubs to
   match the new signatures (both currently `unimplemented!()`, so no
   behavior to preserve).
3. Add `src/review.rs` with `ReviewError` and `run`, plus a local `FakeUi`
   test double that records `choose` headers (in order) and a queued
   sequence of scripted responses; write review.md 001's five scenarios as
   tests first, watch them fail, then implement `run`/`prompt_one`.
4. Add `pub mod review;` to `lib.rs`.
5. Wire `Commands::Review` into `main.rs`'s `Cli`/dispatch, with a
   `tests/cli_review.rs` integration test following the existing
   `tests/cli_daily.rs` pattern (discover a real temp-dir workspace,
   scripted stdin/stdout or a test-only `Ui`, assert exit code and
   printed output).
6. Mark review.md Story 001 `✅`.
7. Update `docs/roadmap.md`'s `review` row and dependency-graph note (item
   7) to reflect story 001 done; 002 remaining (unblocked, `items::mv`
   already lands per item 6) and 003 remaining (blocked on
   `items::write_last_reviewed`, which doesn't exist yet).
8. Manual smoke test: in a scratch PARA system with two projects and one
   area, run `tk review`, confirm the walk order, header wording, and that
   it exits cleanly after the last prompt; then run it again against an
   empty workspace and confirm the "Nothing to review." message with no
   prompts.
9. `cargo clippy`, `cargo fmt --check`, `cargo test` clean before calling
   review.md 001 done.
