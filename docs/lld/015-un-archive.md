# LLD: Un-archiving — Story `005` (`move.md`)

Source: [docs/user-stories/move.md](../user-stories/move.md) Story 005.
Module boundaries follow [docs/design.md](../design.md).
Corresponds to roadmap item 1 ("Un-archiving").

## Scope

1. A bare name never matches an item in `Archive`; only the qualified
   `<OriginCategory>/<name>` form does (`move.md` 005 scenario 1).
2. Un-archiving a directory item into a directory-style target relocates it
   as-is (`move.md` 005 scenario 2).
3. Un-archiving a directory item into a different directory-style target
   also relocates it as-is (`move.md` 005 scenario 3).
4. Un-archiving a flat file into a flat-file target relocates it as-is
   (`move.md` 005 scenario 4).
5. Un-archiving a flat file into a directory-style target wraps it into a
   new `<name>/index.<ext>` (`move.md` 005 scenario 5).
6. Un-archiving a directory item into a flat-file target is rejected, same
   as Story 002 (`move.md` 005 scenario 6).
7. Moving an already-archived item to `archive` again is rejected instead
   of panicking (`move.md` 005 scenario 7). Not asked for by any scenario
   in the story as originally scoped, but it's a code path that becomes
   reachable once `locate` can return `Category::Archive` as a source —
   `Category::archive_origin_name()` is `unreachable!()` on `Archive`, so
   without this guard `tk move <archived-item> archive` panics instead of
   erroring. See "Why guard re-archiving" below.

### Out of scope

- Changing how `tk archive`/`Commands::Archive` dispatches — it never
  needs an `Archive` source (you can't type `tk archive Projects/foo`;
  the alias takes no category argument), so it's untouched.
- Any new CLI surface — `MoveTarget`/`Commands::Move` in `main.rs` already
  accept all five categories as `target`; only `items::locate`/`items::mv`
  need to accept `Archive` as a *source*.
- Re-deriving a bare (non-qualified) alias for archived items (e.g. some
  shorthand that skips typing the origin subfolder) — not asked for by the
  story, and `locate`'s existing "bare name never matches Archive" rule
  (design.md) stays exactly as documented.

## `design.md` changes

Not yet applied — deferred until this LLD lands. Summary of the changes:

- `items::locate` gains an `Archive` fallback branch: after its existing
  `Category::archivable()` scan finds nothing, it splits `name` on `/`
  and, if the first component matches some `Category::archivable()`
  member's `archive_origin_name()`, searches that origin's subfolder
  under `Archive` for the remaining basename. Returns
  `Some((Category::Archive, path))` on a match. This is the extension
  design.md's `locate` doc comment already forward-references
  ("extended for un-archiving per move.md Story 005").
- `items::mv`'s shape decisions (wrap vs. relocate-as-is vs. reject) switch
  from matching on `source.is_directory_style()` to checking
  `source_path.is_dir()` at call time. `Category::is_directory_style()`
  returns `false` for `Category::Archive` unconditionally (it has no
  shape of its own — it defers to whatever the original item was), so an
  archived project's `source_path` (a real directory on disk) is the only
  reliable signal of its shape once `source` can be `Archive`. This also
  simplifies the existing `basename` closure: it no longer needs to
  branch on `source.is_directory_style()` at all, since
  `source_path.file_name()` is already correct for both a directory and a
  file.
- `items::mv` gains an early guard: `source == Category::Archive &&
  target == Category::Archive` is now a rejection
  (`ItemsError::AlreadyArchived`) instead of reaching
  `archive_origin_name()`'s `unreachable!()` branch.

## Module designs

### `items`

```rust
pub fn locate(ws: &Workspace, name: &str) -> Result<Option<(Category, PathBuf)>, ItemsError> {
    for category in Category::archivable() {
        // ...unchanged...
    }

    if let Some((origin_name, basename)) = name.split_once('/') {
        if let Some(origin) = Category::archivable()
            .into_iter()
            .find(|c| c.archive_origin_name() == origin_name)
        {
            let dir = ws.category_dir(Category::Archive).join(origin.archive_origin_name());
            if origin.is_directory_style() {
                let candidate = dir.join(basename);
                if candidate.is_dir() {
                    return Ok(Some((Category::Archive, candidate)));
                }
            } else {
                for (entry_name, path) in scan_dir(&dir, false, &ws.config.default_extension)? {
                    if entry_name == basename {
                        return Ok(Some((Category::Archive, path)));
                    }
                }
            }
        }
    }

    Ok(None)
}

pub fn mv(
    ws: &Workspace,
    source: Category,
    source_path: &Path,
    name: &str,
    target: Category,
) -> Result<PathBuf, ItemsError> {
    if source == Category::Archive && target == Category::Archive {
        return Err(ItemsError::AlreadyArchived { name: name.to_string() });
    }

    let source_is_dir = source_path.is_dir();

    if source_is_dir && !target.is_directory_style() && target != Category::Archive {
        return Err(ItemsError::UnwrapNotSupported {
            name: name.to_string(),
            from: source.display_name(),
            to: target.display_name(),
        });
    }

    let basename = || -> PathBuf {
        PathBuf::from(source_path.file_name().expect("located item has a file name"))
    };

    let dest = if target.is_directory_style() && !source_is_dir {
        ws.category_dir(target)
            .join(source_path.file_stem().expect("located item has a file name"))
            .join(format!("index.{}", ws.config.default_extension))
    } else if target == Category::Archive {
        ws.category_dir(Category::Archive)
            .join(source.archive_origin_name())
            .join(basename())
    } else {
        ws.category_dir(target).join(basename())
    };

    // ...unchanged: create_dir_all(dest.parent()), fs::rename, Ok(dest)...
}
```

```rust
pub enum ItemsError {
    // ...existing variants...
    #[error("\"{name}\" is already archived")]
    AlreadyArchived { name: String },
}
```

`origin.is_directory_style()` (used only to decide *how to search* the
origin subfolder — as a single directory entry vs. scanning flat files)
is a different question from the shape `mv` cares about, which is
answered from `source_path.is_dir()` once `locate` has already found the
item; the two aren't in tension despite both existing in this LLD's diff.

`source_path.file_stem()` for the wrap-branch's new directory name is not
a new convention introduced here — `scan_dir`'s flat-file branch (and
therefore every `name` reported by `list`/`locate` today) already derives
an item's bare name via `file_stem()`, so a wrapped un-archived item gets
exactly the name it would already be known by.

Removing `source.is_directory_style()` from `mv` entirely means
`Category::is_directory_style()` no longer needs to account for
`Archive`'s shape at all — no change to that function is needed, but it's
worth noting `mv` no longer calls it on `source`, only on `target`.

No signature changes: `locate`'s and `mv`'s public signatures are
unchanged, so every existing call site (`cli::run_move`,
`review::run`'s `[a]rchive` arm) needs no changes. Existing tests for both
functions are unaffected since non-`Archive` sources hit the exact same
branches as before (`source_path.is_dir()` agrees with
`source.is_directory_style()` for every non-`Archive` category).

### Why guard re-archiving

Once `source` can legitimately be `Category::Archive`, `tk move
Projects/foo archive` is a call clap/`run_move` will happily construct
and pass to `mv` — nothing upstream rejects it. Before this LLD, `source`
was only ever one of `archivable()`'s four members, so
`source.archive_origin_name()` (called when `target == Archive`) was
never reachable with `source == Archive`. Extending `locate` to return
`Archive` as a source makes that combination reachable for the first
time, so the `unreachable!()` in `archive_origin_name()` needs a real
guard in front of it rather than staying provably-dead code.

### `cli`

No changes. `run_move`'s existing shape (`locate` then `mv`, `target ==
Archive` prompts for a summary) already handles this correctly: an
un-archiving move never has `target == Archive` (that's the rejected
re-archiving case above), so the summary-stamp branch is simply never
entered for these calls — no new branch needed there.

## Test plan (TDD — write these first)

| Scenario | Test | Module |
|---|---|---|
| Bare name doesn't match an Archive item | `locate` given only `4-Archive/Projects/website-redesign` on disk, called with `"website-redesign"`, returns `Ok(None)` | `items` (unit, `move.md` 005 scenario 1) |
| Qualified name matches a directory-style archived item | `locate` called with `"Projects/website-redesign"` returns `Ok(Some((Category::Archive, <path>)))` pointing at the real directory | `items` (unit, `move.md` 005 scenario 1/2) |
| Qualified name matches a flat archived item | `locate` called with `"Resources/my-file"` returns `Ok(Some((Category::Archive, <path>)))` pointing at `my-file.md` | `items` (unit, `move.md` 005 scenario 4) |
| Qualified name with unknown origin component doesn't match | `locate` called with `"Bogus/whatever"` returns `Ok(None)` | `items` (unit, edge case) |
| Un-archive directory to matching-shape category relocates as-is | `mv(ws, Archive, <archived dir path>, "website-redesign", Project)` renames dir to `1-Projects/website-redesign`, no `index.md` re-nesting | `items` (unit, `move.md` 005 scenario 2) |
| Un-archive directory to a different directory-style category relocates as-is | same as above with `target = Area` | `items` (unit, `move.md` 005 scenario 3) |
| Un-archive flat file to flat category relocates as-is | `mv(ws, Archive, <archived file path>, "my-file", Inbox)` renames file to `0-Inbox/my-file.md` | `items` (unit, `move.md` 005 scenario 4) |
| Un-archive flat file to directory-style category wraps it | `mv(ws, Archive, <archived file path>, "my-note", Project)` produces `1-Projects/my-note/index.md` with the original content | `items` (unit, `move.md` 005 scenario 5) |
| Un-archive directory to flat category is rejected | `mv(ws, Archive, <archived dir path>, "website-redesign", Inbox)` returns `Err(UnwrapNotSupported { .. })`, filesystem untouched | `items` (unit, `move.md` 005 scenario 6) |
| Re-archiving an already-archived item is rejected, not a panic | `mv(ws, Archive, <archived path>, "my-file", Archive)` returns `Err(AlreadyArchived { .. })` | `items` (unit, `move.md` 005 scenario 7) |
| End-to-end un-archive via `run_move` | `run_move(&ws, &mut ui, "Projects/website-redesign", Category::Project, false)` returns the `Moved ...` message and actually relocates the directory | `cli` (unit, `move.md` 005 scenario 2) |
| Existing non-Archive `mv`/`locate` behavior is unchanged | full existing `items`/`cli` test suites still pass unmodified | `items`, `cli` (regression) |

## Implementation plan

1. Add `items::locate`'s Archive-fallback unit tests (scenario 1 above,
   both directory-style and flat origin, plus the unknown-origin-component
   edge case); watch them fail; implement the `split_once('/')` branch.
2. Add `items::mv`'s new/changed unit tests (scenarios 2–7 above,
   including the `AlreadyArchived` guard); watch them fail; swap
   `source.is_directory_style()` for `source_path.is_dir()`, simplify the
   `basename` closure, and add the early re-archiving guard.
3. Add `ItemsError::AlreadyArchived` and confirm its `Display` message
   reads naturally as a top-level CLI error (`anyhow` propagation via
   `cli::run_move`'s `?`).
4. Add the `cli::run_move` end-to-end test exercising a full un-archive
   through the same public function `main` calls; no changes to
   `run_move` itself should be required — this test exists to catch a
   regression if one turns out to be needed.
5. Run the full existing `items`/`cli` test suites to confirm no
   regression in non-`Archive` `mv`/`locate` behavior.
6. Mark `move.md` Story 005's scenarios `✅` (flip `Status` from "Not
   started" to `✅`).
7. Update `docs/roadmap.md`: remove the "Remaining work" section (this was
   the last item) and fold `move`'s roadmap-table row back to a plain
   "Done", matching every other command's row.
8. Manual smoke test:
   - `tk archive website-redesign` (or manually move a project under
     `4-Archive/Projects`), then `tk list archive` to confirm the
     qualified name, then `tk move Projects/website-redesign project`
     and confirm it lands back at `1-Projects/website-redesign` with
     `index.md` intact.
   - Repeat for a flat resource: archive `my-file.md`, then `tk mv
     Resources/my-file resource`.
   - Try wrapping: archive an inbox note, then `tk move Inbox/my-note
     project` and confirm `1-Projects/my-note/index.md` is created with
     the original content.
   - Try the rejection paths: `tk move Projects/website-redesign inbox`
     (unwrap rejected) and `tk move Projects/website-redesign archive`
     (already-archived rejected) — confirm both print an error and leave
     the filesystem untouched.
9. `cargo clippy`, `cargo fmt --check`, `cargo test` clean before calling
   Story 005 done.
