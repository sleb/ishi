# Roadmap

Gap analysis between the target behavior in [README.md](../README.md) /
[user-stories](user-stories/) and what's implemented in `src/` today, sequenced
by dependency rather than by date. Each milestone unlocks the one after it —
see the _Why here_ line.

## Status snapshot (as of this doc)

| Command       | State                                                                                                                                                                       |
| ------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `init`        | Done                                                                                                                                                                        |
| `new`         | Partial — Inbox capture + editor-inferred filename work; `--project`/`--area`/`--resource` flags aren't wired to the CLI yet, and no template is applied to created content |
| `daily`       | Not started                                                                                                                                                                 |
| `mv`          | Not started                                                                                                                                                                 |
| `list`        | Not started                                                                                                                                                                 |
| `status`      | Not started                                                                                                                                                                 |
| `review`      | Not started (module stub only in `docs/design.md`)                                                                                                                          |
| `config`      | Not started — `Config::load` reads one TOML file with no layering; no `templates` table; no `config` subcommand                                                             |
| `completions` | Not started                                                                                                                                                                 |

## Dependency graph

Item-level view of the sequencing below — an edge means the upstream item is
a real implementation dependency of the downstream one (not just a shared
naming convention; see the footnote after the diagram for two cross-references
that are docs-only and deliberately _not_ drawn here).

```
                                   init (done)
                                       |
                 +----------------------+----------------------+
                 |                                              |
                 v                                              v
          1. new: wire                                   9. completions
        --project/area/resource                          (no other deps)
                 |
     +-----------+------------+------------+
     |           |            |
     v           v            v
 4. list     6. mv      2. config layering
     |           |          + templates
     v           |         /            \
 5. status       |        v              v
     |           |    3. daily      8. config CLI
     +-----+-----+
           |
           v
      7. review
```

- `list` → `review` (list.md 001 cites review.md 001's "raw days ago"
  phrasing as a shared convention) is a docs-only cross-reference, not a
  build blocker — `list` ships first per the sequencing below regardless.
- `completions` (item 9) lists every other command's Story 001 as a
  "depends on" in completions.md 003, but that's `clap_complete` describing
  what the generated script must cover, not implementation work blocked on
  those commands landing — completions can be built against however much of
  the CLI exists at the time.

## Now

### 1. Finish `new`: wire `--project` / `--area` / `--resource`

`items::create` already takes a `Category` and scaffolds directories vs. flat
files correctly (see `src/items.rs` tests) — the only gap is `main.rs`'s
`Commands::New` doesn't expose the flags yet. Smallest remaining piece of an
already-started command.

- Covers user-stories/new.md 003–006. (Story 002, the named-Inbox-note case,
  is already Completed; init.md 001–004 are Done.)

## Next

### 2. Config layering + templates

Today `Config::load` reads a single path with no merge step, and has no
`templates` field at all — `new`/`daily`/etc. write raw editor content or an
empty string instead of a rendered template. This is the one piece of
plumbing that several later commands need, so it goes first in this group:

- Add `templates: Templates` to `Config`, with `{{title}}`/`{{date}}`
  rendering.
- Implement the `built-in → ~/.tick.toml → ./.tick.toml` merge (currently
  `Workspace::discover` only knows about bare category dirs, not config
  layering at all).
- **Why here:** `daily` (item 3) needs a rendered `daily` template, and
  `config` (item 8) needs the same layering logic to report provenance.
  Doing it once now avoids reworking `new`'s content path twice.
- Covers the config-resolution scenarios in user-stories/config.md 001–002
  (not yet the `tk config` CLI surface — that's item 8).
- **Design update (2026-07-02), not yet implemented:** templates gained
  `{{time}}`, `{{cursor}}`, and `{{uuid}}` placeholders alongside
  `{{title}}`/`{{date}}`, and the editor-capture path (`new` with no
  filename, and `--project`/`--area`/`--resource` with no filename) now
  pre-populates `$EDITOR` with the rendered template instead of a blank
  scratch file — `{{title}}` renders empty and `{{cursor}}` marks where the
  editor's cursor should start (via a `+<line>` argument). This means
  `editor::suggest_filename`'s title-inference can no longer assume the
  title is the file's literal first line: it now skips a leading
  frontmatter block, then takes the first Markdown heading line (any `#`
  level), falling back to the first non-blank post-frontmatter line, then
  to the timestamp fallback. `src/editor.rs`'s current `suggest_filename`
  and the `Editor::capture` signature both need updating when this item is
  picked up — see user-stories/new.md 001/007/010 and the updated
  `editor` section of design.md. Worth checking `sleb/knap` before
  implementing — it may already have LSP support, or heading/frontmatter
  parsing logic, that could be reused as a shared library instead of
  reimplementing here.
- Covers user-stories/new.md 001, 007 (editor pre-population and
  timestamp-fallback inference), 008, 009 (template rendering for named
  notes and scaffolded `index.md`), 010 (capture into `--project`/`--area`/
  `--resource` with no filename — needs item 1's flags plus this item's
  rendering), and 011, 012 (`{{time}}` and `{{uuid}}` placeholders).

### 3. `tk daily`

Straightforward once item 2 lands: resolve today's date, render the `daily`
template, create-or-open in the Inbox. Implemented as sugar for `tk new
--daily` — the `--daily` flag needs wiring alongside `--project`/`--area`/
`--resource` from item 1, plus the create-vs-open branch on whether today's
note already exists. Covers user-stories/daily.md and new.md story 013.

### 4. `tk list`

Pure read over `items` — no mutation, no new filesystem primitives beyond
what `create` already exercises. Cheapest of the remaining commands and has
no dependents, so it can slot in anywhere in this group.

- **Design update (2026-07-02), not yet implemented:** `list` no longer
  just returns bare paths — `items::list` returns `Vec<ListedItem>`
  (Name/Title/Updated), sorted alphabetically by name, with `filter`
  matching a case-insensitive substring of Name or Title. Needs a new
  `items::infer_title` helper (frontmatter-skip + first-heading, mirroring
  `editor::suggest_filename`'s logic but implemented independently — see
  the updated `items` section of design.md) and an mtime-based
  `updated_days_ago`, the same source item 5 (`status`) needs for its
  per-item `updated: ...` facts. Covers user-stories/list.md.

### 5. `tk status`

Per-category counts, plus a per-item breakdown for Projects/Areas showing
`updated_days_ago` (reuses `list`'s mtime sourcing — sequenced right after
`list` for that reason) and `reviewed_days_ago` (a new `last_reviewed`
frontmatter field, `None` until an item has been kept in a review). No
staleness threshold or flagging — `status` reports the facts and leaves
judgment to the user. Needs `items::read_last_reviewed` /
`items::write_last_reviewed` (see design.md); `write_last_reviewed` has no
caller until item 7 (`review`) lands, but the read side is exercised by
`status` alone. Covers user-stories/status.md.

### 6. `tk mv`

More involved: wrapping a flat file into a directory when moving into
`project`/`area`, and preserving origin category as a subfolder when moving
to `archive`. No dependents among items 3–5, but **item 7 (`review`) calls
`items::mv` directly** per `docs/design.md`, so it has to land before review.

- Covers user-stories/mv.md 001.

## Later

### 7. `tk review`

Composes `mv` (item 6) with the `Ui` trait already defined in `src/cli.rs`
(`confirm`/`choose` are implemented and tested via `run_init`/`run_new`), plus
`items::write_last_reviewed` (item 5) on `[k]eep`. Blocked on items 5 and 6.

- Covers user-stories/review.md 001–003.

### 8. `tk config` CLI surface

`config`, `config init`, `config init -g`, `config edit`, `config edit -g`,
plus the `#:schema` JSON Schema file. Builds directly on the layering +
provenance tracking from item 2 — deliberately kept separate from it because
nothing else in this roadmap depends on the CLI surface existing, only on the
resolution logic underneath it.

- Covers user-stories/config.md 003–006.

### 9. `tk completions`

Pure `clap_complete` wiring, no dependencies on anything else in this list.
Lowest priority because it has no user-facing value until the rest of the
command set stabilizes — completion scripts for half-finished commands are
churn.

- Covers user-stories/completions.md 001–003. Story 003 (completions must
  stay current with `tk`'s commands) lists every other command's Story 001
  as a "depends on," but that's describing what the generated script must
  cover, not build order — `clap_complete` derives the script from whatever
  subcommands exist in the CLI definition at the time.

## Explicitly out of scope for this pass

Anything not in the README's command table (e.g. sync, plugins, multi-user
config) — not hinted at anywhere in the spec, so not roadmapped until there's
a concrete story for it.
