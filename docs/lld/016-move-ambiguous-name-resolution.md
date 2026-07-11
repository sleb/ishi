# LLD: Ambiguous item-name resolution for `tk move` — Story 006 (`move.md`)

Source: [docs/user-stories/move.md](../user-stories/move.md) Story 006.
Module boundaries follow [docs/design.md](../design.md). Corresponds to
roadmap item "generalize `<Category>/<name>` addressing to live items,
reject ambiguous bare names".

## Scope

1. A bare name matching items in two different live categories is rejected
   with an error naming both candidates, moving nothing (`move.md` 006
   scenario 1).
2. Qualifying the name with its live category (`resources/meeting-notes`)
   resolves the ambiguity and moves only that item (`move.md` 006 scenario 2).
3. The qualified `<category>/<name>` form works for any live category, not
   just `Archive`'s origin subfolders, and a qualifier that doesn't match
   the item's actual location reports "not found" rather than falling back
   to a bare-name search (`move.md` 006 scenario 3).
4. A bare name matching nothing anywhere still gets the existing "not
   found" error, unchanged (`move.md` 006 scenario 4).

`items::locate` already special-cases a `<OriginCategory>/<name>` qualifier
for `Archive` (move.md 005). This LLD generalizes that same splitting
mechanism to the four live categories, using the lowercase form
(`inbox`, `projects`, `areas`, `resources`) that `cli::display_path` already
renders paths in — so the qualifier a user types matches what `tk move`'s
own `Moved ...` confirmation message would show them.

### Out of scope

- `tk unarchive`'s origin-implied target resolution — unaffected; it never
  goes through the bare-name ambiguity path since it requires the qualified
  `Archive` form already (move.md 005).
- Tab-completion offering the qualified forms instead of the now-rejected
  bare ones — that's `completions.md` Story 005, a separate LLD
  (`017-completions-ambiguous-name-offers-qualified-forms.md`), since it's
  a different module (`cli`'s completion-name functions, not `items::locate`)
  and a distinct story file.
- Any change to `mv`'s move/wrap/archive shape logic — untouched; ambiguity
  is resolved (or rejected) before `mv` is ever called.

## `design.md` changes

Deferred until this LLD lands. `items` section's "Locate" bullet
currently reads:

> **Locate**: finds an item by name across the four non-Archive categories
> (Inbox, Project, Area, Resource), or by `<OriginCategory>/<name>` for an
> archived item — a bare name never matches inside `Archive`, since
> basenames aren't unique across origin subfolders.

Will become:

> **Locate**: finds an item by name across the four non-Archive categories
> (Inbox, Project, Area, Resource) — erroring if more than one matches,
> since a bare name is only safe to resolve automatically when it's
> unique — or via an explicit `<Category>/<name>` qualifier, either a live
> category's lowercase name (`inbox`, `projects`, `areas`, `resources`) or
> an `Archive` item's origin subfolder (`Inbox`, `Projects`, `Areas`,
> `Resources`), the latter case-sensitively distinct from the former since
> basenames aren't unique across `Archive` origin subfolders either.

The appendix invariant `items::locate`'s `Archive` fallback only matches
the composite `<OriginCategory>/<name>` form..." stays accurate as-is and
needs no edit — it's still true, just no longer the only qualified form.

## Module designs

### `items` (extends existing module)

```rust
// items.rs

#[derive(Debug, thiserror::Error)]
pub enum ItemsError {
    // ...unchanged variants...

    #[error("\"{name}\" is ambiguous — found in {locations}")]
    Ambiguous { name: String, locations: String },
}

pub fn locate(ws: &Workspace, name: &str) -> Result<Option<(Category, PathBuf)>, ItemsError>
```

`locate`'s signature is unchanged — the new ambiguity case surfaces as an
`Err`, which every existing caller (`cli::run_move`, `cli::run_unarchive`)
already propagates via `?` without modification, since they treat `locate`
as fallible.

Internals restructure into three phases, in order:

1. **Qualified lookup**, if `name` contains `/`: split on the first `/`
   into `(qualifier, basename)`.
   - If `qualifier` case-sensitively equals some
     `Category::archivable()` member's `archive_origin_name()` (e.g.
     `"Projects"`) — the existing move.md 005 behavior — search only
     `Archive/<that origin>/` for `basename`.
   - Else if `qualifier` equals some `Category::archivable()` member's
     `display_name().to_lowercase()` (e.g. `"projects"`, `"inbox"`) — new
     for this story — search only that live category's directory for
     `basename`.
   - Either way, return whatever that single-category search finds
     (`Some` or `None`) immediately — **do not** fall through to phases 2
     or 3. This is what makes `move.md` 006 scenario 3's "a category
     prefix that doesn't match reports not found, not a bare-name
     fallback" hold: once a qualifier is recognized, the search is
     narrowed to that one category, full stop.
   - If `qualifier` matches neither form, fall through to phase 2 with
     `name` used as-is (including its `/`) — preserves current behavior
     for a name that merely happens to contain a slash without being a
     recognized qualifier; it simply won't match anything in phase 2 or 3
     and ends in `Ok(None)`.
2. **Bare-name scan**: search all four of `Category::archivable()` (in
   `Inbox, Project, Area, Resource` order, as today), but — unlike
   today's first-match-wins — collect *every* category that has a match
   rather than returning on the first hit.
   - Zero matches: fall through to phase 3.
   - Exactly one match: `Ok(Some((category, path)))`, as before.
   - More than one: `Err(ItemsError::Ambiguous { name, locations })`,
     where `locations` is each matching category's
     `display_name().to_lowercase()`, joined `", "`, in the same
     `archivable()` order (so `Inbox` before `Resource` always, e.g.
     `"inbox, resources"`, matching move.md 006 scenario 1's example
     verbatim).
3. **Bare-name `Archive` check** (only reached if phase 2 found nothing):
   unchanged from today — this is dead for a *qualified* name since phase
   1 already returns in that case, so it only ever runs for a genuinely
   unqualified `name`, where it correctly stays `Ok(None)` per `locate`'s
   documented "a bare name never matches inside Archive" contract.

The three-phase split (rather than one unified loop) is deliberate: it
keeps qualified lookup's "narrow and stop" semantics visually separate
from bare lookup's "scan and detect ambiguity" semantics, instead of
threading a `qualifier: Option<Category>` filter through one loop body —
the two phases have different failure modes (silent `None` vs. `Err`) and
conflating them risks the qualified path accidentally inheriting the
ambiguity check it must not have (a qualifier by construction narrows to
one category, so it can never be ambiguous).

No changes to `mv` — it already takes `source: Category` resolved by the
caller and has no knowledge of how that category was determined.

## Test plan (TDD — write these first)

| Scenario | Test | Module |
|---|---|---|
| Bare name matching two live categories is rejected | `locate_bare_name_matching_two_categories_is_ambiguous` — create `meeting-notes` in `Inbox` and `Resource`, assert `locate` returns `Err(Ambiguous { .. })` with `locations == "inbox, resources"` | `items` (unit, move.md 006 scenario 1) |
| Bare name matching two categories moves nothing | `mv_not_called_when_locate_errors_ambiguous` — via `cli::run_move`, assert the error propagates and no file moved (assert both original files still exist, no new file at any destination) | `cli` (unit, move.md 006 scenario 1) |
| Qualified live-category name resolves ambiguity | `locate_qualified_live_category_name_resolves_ambiguity` — same fixture as above, `locate(ws, "resources/meeting-notes")` returns `Ok(Some((Category::Resource, <path>)))` | `items` (unit, move.md 006 scenario 2) |
| `run_move` with qualified name moves only that item | `run_move_qualified_name_moves_only_that_category` — `tk move resources/meeting-notes archive` moves `3-Resources/meeting-notes.md` to `4-Archive/Resources/meeting-notes.md`, leaves `0-Inbox/meeting-notes.md` untouched | `cli` (unit, move.md 006 scenario 2) |
| Qualified live-category form works for any category, unambiguous case | `locate_qualified_inbox_prefix_matches_unambiguous_item` — `locate(ws, "inbox/my-file")` matches `Category::Inbox`'s `my-file`, identical result to `locate(ws, "my-file")` | `items` (unit, move.md 006 scenario 3) |
| Non-matching qualifier reports not found, no bare fallback | `locate_qualified_prefix_not_matching_actual_location_is_none` — `my-file` lives in `Inbox`; `locate(ws, "projects/my-file")` returns `Ok(None)`, not `Ok(Some(Category::Inbox, ..))` | `items` (unit, move.md 006 scenario 3) |
| Unmatched bare name still gets existing not-found error | `run_move_errors_when_no_item_matches_name` — already exists (`src/cli.rs:1886`); re-verify unchanged after the phase-3 restructure | `cli` (existing test, regression guard, move.md 006 scenario 4) |
| Existing move.md 001/002/003/004/005 tests | Re-run unmodified — the phase split must not change results for any already-passing scenario (single-match bare name, `Archive`-qualified name, unwrap rejection) | `items`, `cli` (existing tests, regression guard) |

## Implementation plan

1. Add `ItemsError::Ambiguous { name, locations }` to `items.rs` and its
   `#[error(...)]` string; no other code references it yet, so this step
   alone should build clean.
2. Write the new `items::locate` unit tests from the table above; watch
   them fail (compiles, but multi-match still returns the first hit and
   qualified live lookup falls through to the old bare scan/`Archive`
   check).
3. Restructure `locate`'s body into the three phases described above.
   Run the full `items` test suite — the new tests should pass and every
   pre-existing `locate_*`/`mv_*` test should still pass unmodified.
4. Write the `cli::run_move` tests confirming ambiguity propagates as an
   error with no filesystem side effect, and that a qualified name moves
   only the intended item; no `cli.rs` production code changes are
   expected here since `run_move`/`run_unarchive` already propagate
   `locate`'s `Result` via `?`.
5. Mark move.md Story 006 `✅`.
6. Update `docs/roadmap.md`'s status for this item.
7. Manual smoke test:
   ```
   tk init /tmp/tickdemo && cd /tmp/tickdemo
   tk new inbox meeting-notes   # or however inbox notes are created
   tk new resource meeting-notes
   tk move meeting-notes archive          # expect the ambiguity error
   tk move resources/meeting-notes archive # expect it to move only the resource
   tk move inbox/meeting-notes project     # expect it to move the remaining inbox one
   ```
8. `cargo clippy && cargo fmt --check && cargo test` clean before calling
   the story done.
