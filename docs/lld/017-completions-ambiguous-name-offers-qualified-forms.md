# LLD: Completions offer qualified forms for ambiguous names — Story 005 (`completions.md`)

Source: [docs/user-stories/completions.md](../user-stories/completions.md)
Story 005. Module boundaries follow [docs/design.md](../design.md).
Corresponds to roadmap item "tab-completion only ever offers names `tk
move` will actually accept".

## Scope

1. Completing a name that collides across two live categories offers both
   qualified `<category>/<name>` forms, not the bare name that
   `move.md` 006 would now reject as ambiguous (`completions.md` 005
   scenario 1).
2. Completing a name that's unique across all categories still offers the
   bare form — no unnecessary qualification (`completions.md` 005
   scenario 2).

### Out of scope

- The rejection itself and the `<Category>/<name>` qualified-lookup
  mechanism `locate` now understands — that's `move.md` 006, implemented
  in `016-move-ambiguous-name-resolution.md`. This LLD only changes what
  names `cli::live_item_names` *offers*, consuming that mechanism as a
  given; no changes to `items::locate` are needed here.
- `tk unarchive`'s completion (`complete_unarchive_name` /
  `cli::archived_item_names`) — already always qualified
  (`<OriginCategory>/<name>`) since move.md 005, so it already only offers
  forms `tk move`/`tk unarchive` accept; nothing to change.
- Any change to `complete_move_name`'s or `complete_archive_name`'s own
  bodies in `main.rs` — both already just filter whatever
  `cli::live_item_names` returns by the in-progress prefix; qualifying
  ambiguous names inside `live_item_names` itself is sufficient for both
  call sites to inherit the fix for free.

## `design.md` changes

Deferred until this LLD lands. `cli` section's `run_move`/completion
bullet:

> `cli` owns the pure item-name-set logic backing tab-completion
> (`live_item_names`/`archived_item_names`), both thin wrappers over
> `items::list` — `main`'s completer functions call these and filter to
> the in-progress argument's prefix.

Will gain a clause: "`live_item_names` qualifies a name with its category
(`<category>/<name>`, lowercase, matching `display_path`'s rendering)
whenever that basename occurs in more than one live category — never
offering a bare name `tk move` would now reject as ambiguous (move.md
006)."

## Module designs

### `cli` (extends existing module)

```rust
// cli.rs

pub fn live_item_names(ws: &Workspace) -> Vec<String>
```

Signature is unchanged — still `Vec<String>`, still every name across the
four live categories — only the *content* of ambiguous entries changes,
from a bare name to a qualified one. This keeps both call sites
(`complete_move_name`, `complete_archive_name` in `main.rs`) untouched.

Implementation collects `(Category, String)` pairs across
`Category::archivable()` (as today), then counts occurrences of each
basename; any basename occurring more than once has every one of its
occurrences rendered as `<category.display_name().to_lowercase()>/<name>`
instead of the bare `name`. This mirrors `items::locate`'s live-category
qualifier format exactly (introduced by move.md 006 /
`016-move-ambiguous-name-resolution.md`), so any name this function
offers is guaranteed to be one `locate` will resolve unambiguously — the
whole point of the story.

Counting first and qualifying in a second pass (rather than qualifying
inline as each category is scanned) is necessary because ambiguity can
only be known once all four categories have been scanned — a name seen
once while scanning `Inbox` isn't yet known to collide with `Resource`
until `Resource` is scanned too.

`archived_item_names` needs no change — it already unconditionally
returns qualified `<OriginCategory>/<name>` forms (move.md 005), so an
`Archive` item can never be offered as an ambiguous bare name in the
first place.

## Test plan (TDD — write these first)

| Scenario | Test | Module |
|---|---|---|
| Colliding basename across two categories offers both qualified forms | `live_item_names_qualifies_colliding_basenames` — create `meeting-notes` in `Inbox` and `Resource`; assert `live_item_names` returns `["inbox/meeting-notes", "resources/meeting-notes"]` (sorted), not the bare name | `cli` (unit, completions.md 005 scenario 1) |
| Unique basename stays bare | `live_item_names_lists_names_across_all_four_live_categories` — already exists (`src/cli.rs:2119`); re-verify it still asserts bare names since none of its fixture names collide | `cli` (existing test, regression guard, completions.md 005 scenario 2) |
| Mixed fixture: one collision, one unique | `live_item_names_qualifies_only_colliding_names` — create `meeting-notes` in `Inbox` and `Resource`, plus `website-redesign` only in `Project`; assert result contains `"inbox/meeting-notes"`, `"resources/meeting-notes"`, and bare `"website-redesign"` all three | `cli` (unit, completions.md 005 scenarios 1+2 combined) |
| Completion candidates reflect the qualified names end-to-end | Extend `complete_move_name`'s existing completion test fixture (`src/main.rs` around `completes_a_live_items_bare_name`) with a colliding pair, typing a shared prefix; assert both qualified candidates are offered, not the bare name | `main` (unit, completions.md 005 scenario 1, end-to-end through the real completer function) |

## Implementation plan

1. Write the new `cli::live_item_names` unit tests from the table above;
   watch them fail (today's implementation always returns bare names).
2. Implement the two-pass count-then-qualify logic in `live_item_names`;
   confirm the new tests pass and the pre-existing
   `live_item_names_lists_names_across_all_four_live_categories` /
   `live_item_names_on_uninitialized_workspace_is_empty` tests still pass
   unmodified.
3. Add the `main.rs` end-to-end completer test exercising
   `complete_move_name` against a colliding fixture; confirm it passes
   with no changes needed to `complete_move_name`/`complete_archive_name`
   themselves.
4. Mark completions.md Story 005 `✅`.
5. Update `docs/roadmap.md`'s status for this item.
6. Manual smoke test (requires a completion script installed, e.g. via
   `eval "$(tk completions zsh)"` in a fresh shell):
   ```
   tk init /tmp/tickdemo && cd /tmp/tickdemo
   tk new inbox meeting-notes
   tk new resource meeting-notes
   tk move meeti<TAB>   # expect: inbox/meeting-notes, resources/meeting-notes
   tk new project website-redesign
   tk move website<TAB>  # expect: website-redesign (bare, unique)
   ```
7. `cargo clippy && cargo fmt --check && cargo test` clean before calling
   the story done.

## Sequencing note

This LLD assumes `016-move-ambiguous-name-resolution.md` (move.md 006)
lands first — `live_item_names`'s qualified output is only correct to
offer once `items::locate` actually accepts the
`<category>/<name>` live-category qualifier it depends on. If picked up
out of order, do move.md 006 first.
