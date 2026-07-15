# Ishi: High-Level Design

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
 └───┬───┘  └────┬────┘  └───┬────┘
     │           │           │
     ▼           ▼           ▼
 ┌──────────────────────┐  ┌──────┐
 │      workspace       │  │ gist │  external crate: Markdown/frontmatter
 └──────────┬───────────┘  └──────┘  parsing (see `gist` below)
            ▼
       ┌─────────┐
       │ config  │  .ishi.toml (folder names, default extension)
       └─────────┘

 ┌──────────┐
 │ category │  two vocabularies, no I/O: Category (filing) + Kind (creation)
 └──────────┘  (used by cli, workspace, items, review, config)
```

- `category` and `config` have no dependencies on anything else — they're the
  vocabulary and settings every other module shares.
- `workspace` depends only on `config` + `category`.
- `items` and `editor` depend only on `workspace` and the external `gist`
  crate — they don't know about each other or about `cli`.
- `review` composes `items` with a `Ui`, but doesn't know about `clap` or argv.
- `cli` is the only place that does terminal I/O; every other module returns
  data or `Result`s so it can be unit-tested directly.

### `category`

Shared vocabulary types, no I/O. Two distinct enums, deliberately not one —
see "Filing vocabulary vs. creation vocabulary" below for why.

- **`Category`** — *where an item is filed*: `Inbox`, `Project`, `Area`,
  `Resource`, `Archive`. Used by every command that manages items that
  already exist (`mv`'s destination, `list`/`status`'s per-category
  iteration, `Archive`'s origin-folder tracking). Directory-style categories
  (`Project`/`Area`) are scaffolded dirs with an `index.md`; flat-file
  categories (`Inbox`/`Resource`) are a single file. `Archive` defers to the
  origin category it's preserving. Each non-`Archive` category also has a
  fixed archive-origin subfolder name (e.g. `Project` -> `"Projects"`) used
  when filing into `Archive`, separate from the user-configurable top-level
  folder names in `config`. Also has a singular lowercase `key()` (`inbox`,
  `project`, `area`, `resource`, `archive`) distinct from both
  `archive_origin_name` and `display_name` — used by `--json` output so
  agent consumers can branch on a fixed machine-readable value instead of
  parsing a display string.
- **`Kind`** — *what `new`/`daily` create*: `Inbox`, `Project`, `Area`,
  `Resource`, `Daily`. Used only by the creation path (`cli::run_new`'s
  dispatch, template selection). Each `Kind` maps to the `Category` it files
  into; `Kind::Inbox` and `Kind::Daily` both map to `Category::Inbox` — a
  daily note has no filing location of its own, just a different template
  and create-or-reopen lifecycle than a plain Inbox capture.

#### Filing vocabulary vs. creation vocabulary

`Category` and `Kind` look almost identical (four of five variants line up
1:1) but answer different questions, and conflating them is a recurring trap
worth naming explicitly:

- **`Category` = "where does this item live?"** — a filing-location fact
  that's true of an item forever, independent of how it was created.
- **`Kind` = "what is being created?"** — a creation-flavor fact that only
  matters at the moment `new`/`daily` runs (which template to render, which
  control flow to follow). Once the file exists on disk, its `Kind` is
  forgotten; only its `Category` persists.

The mismatch at the edges is the signal these needed to be two types instead
of one: a daily note needs its own template and lifecycle but has no folder
of its own (`Kind::Daily`, no matching `Category`), and `Category::Archive`
needs a folder but items never arrive there via `new` — only via `mv` (no
matching `Kind`).

**Rule of thumb for future additions:** a new artifact type that needs its
own template or creation control-flow, but always resolves into an existing
folder, is a new `Kind` variant. A new artifact type that needs its own
folder is a new `Category` variant (plus a matching `Kind` only if `new`
should be able to create it directly).

### `config`

Parses `.ishi.toml` and layers it across three sources — built-in defaults,
`~/.ishi.toml` (user), `./.ishi.toml` (local) — independently per key, so a
local file can override just one setting without repeating the rest.
Provenance (which layer each effective value came from) is tracked
alongside the resolved config, for `ishi config`'s annotated output.

Responsibilities:
- Resolve effective config from the three layers (`Config::resolve`).
- Know the built-in defaults (folder names, default extension, templates).
- Render config as TOML, both a fresh scaffold (`init`) and an
  origin-annotated view of the resolved config (`ishi config`), plus a JSON
  rendering of the same effective-config-plus-provenance data
  (`render_effective_json`, `ishi config --json`) — provenance there uses
  `Source::json_value()`, a coarser three-way version of `Source::comment()`
  that collapses `LocalOverridesUser` into `"local"` (an agent only needs
  "which layer won," not the human-readable aside about what it overrode).
- Own `.ishi.toml`'s JSON Schema and write it alongside a new config file
  so editors get autocomplete/validation.
- Render a template string, substituting `{{date}}`, `{{title}}`, `{{time}}`,
  `{{uuid}}` (`{{cursor}}` is left untouched — that's `editor`'s marker).

Templates are keyed one-to-one with `Kind` (`note`, `daily`, `project`,
`area`, `resource`), so template lookup is total — there's no `Kind::Archive`
to be missing a template for.

`config::init(path)` errors rather than overwriting an existing file, and
never partially writes: the schema file is written first so a failed write
leaves nothing to clean up.

### `workspace`

Answers "where do things live?" for every other component.

- Discovers the workspace root by walking up from a starting path looking
  for `.ishi.toml` or the five category dirs, layering in a user-level
  config via `config::resolve` along the way.
- Maps a `Category` to its directory under the root.
- `init` creates a target directory (if needed) and whichever of the five
  category dirs are missing under it — safe to run against a directory with
  unrelated existing contents, and safe to re-run against a partially
  complete PARA layout. It does not write `.ishi.toml`; the dirs alone are
  enough for `discover`'s fallback to recognize the workspace later.
- Create-only scaffolding for editor/agent ergonomics: writes
  `.zed/settings.json` and/or `.vscode/settings.json` excluding the archive
  folder from the editor, and a `CLAUDE.md` noting the archive folder should
  be skipped — each only if the file doesn't already exist, and each
  independent of the others. None of these parse or merge existing file
  contents; an existing file (any contents) is left untouched.

### `gist`

An external crate ([sleb/gist](https://github.com/sleb/gist)), not an ishi
module — pinned in `Cargo.toml`. Parses a single note's Markdown/frontmatter
(headings, tags, links, code fences) with no filesystem access of its own;
ishi calls it with content already read from disk.

`items` and `editor` each independently depend on `gist` for one primitive —
finding the first Markdown heading after any frontmatter block — rather than
depending on each other, which preserves the module boundary between them.
`gist`'s broader backlink/tag/link-resolution surface (built for a different
note-vault tool) isn't currently used by ishi.

### `items`

All filesystem operations. Takes a `Workspace` and `Category`, returns
structured results — no printing, no prompting.

- **Create**: computes an item's path (directory-vs-flat-file, per
  `Category`), creates it, and writes caller-rendered content. Callers
  render the template (substituting `{{title}}`, etc.) before calling in, so
  every creation path — interactive editor capture, non-interactive named
  creation, daily notes — funnels through one write.
- **Locate**: finds an item by name across the four non-Archive categories
  (Inbox, Project, Area, Resource) — erroring if more than one matches,
  since a bare name is only safe to resolve automatically when it's
  unique — or via an explicit `<Category>/<name>` qualifier, either a live
  category's lowercase name (`inbox`, `projects`, `areas`, `resources`) or
  an `Archive` item's origin subfolder (`Inbox`, `Projects`, `Areas`,
  `Resources`), the latter case-sensitively distinct from the former since
  basenames aren't unique across `Archive` origin subfolders either.
- **Locate (archived, bare)**: `unarchive`'s own narrower bare-name search —
  unlike `Locate`, scans every `Archive` origin subfolder (not the four live
  categories) and returns the item's origin category, erroring if more than
  one origin matches. Safe only because `unarchive` never resolves outside
  `Archive` to begin with, so there's no live/archived collision to obscure.
- **Move**: relocates an item between categories, wrapping a flat file into
  a directory when moving into `Project`/`Area`, and preserving the item's
  origin category as an `Archive` subfolder when archiving. Moving *out of*
  `Archive` follows the same wrap/relocate rules keyed off the destination
  category, not the origin the item was archived under.
- **List**: per-category listing, sorted alphabetically (`Archive` rows sort
  by origin-then-name, matching the qualified display order), with an
  optional case-insensitive substring filter against name or title. Each row
  (`ListedItem`) carries `name`, `title`, `updated_days_ago`, the resolved
  `path` to the item's content file (the exact file `review`/`move` operate
  on — `index.md` for directory-style categories, the flat file itself
  otherwise), and, for `Archive` rows only, `origin: Option<Category>` — the
  category the item was archived from. `name` is unqualified for every
  category, including `Archive`; `cli::run_list`'s text renderer reconstructs
  the qualified `Origin/name` display string from `name` + `origin` at
  render time, while `cli::run_list_json` emits them as separate JSON
  fields. Title is inferred from the item's first Markdown heading (via
  `gist`), falling back to the item's name.
- **Status**: per-category counts, plus per-item facts (`Project`/`Area`
  only) — title, days since last modified, days since last reviewed (from
  the `index.md` frontmatter's `last_reviewed` field, or absent if never
  reviewed). Purely reports these facts; no staleness threshold or judgment.
- **Review bookkeeping**: read and write an item's `last_reviewed`
  frontmatter field (the write preserves every other frontmatter key and the
  body unchanged), for `review`'s keep/archive/skip loop to call into.

### `editor`

Isolated so it's mockable in tests — no real `$EDITOR` needed to test the CLI
prompt logic. Splits into one impure entry point and a pure core so
filename-inference logic is directly unit-testable without spawning a real
editor process or racing the system clock.

- **Capture**: writes a seeded scratch file (a rendered template, with
  `{{cursor}}` marking the starting line), opens `$EDITOR` on it, reads back
  the content plus an inferred filename suggestion.
- **Open**: opens `$EDITOR` directly on an existing file, no scratch file or
  inference — used for reopening an existing daily note, where the content
  is already final.
- **Filename suggestion** (pure): first Markdown heading (via `gist`,
  frontmatter-skip then first-non-blank heading), slugified. Falling back,
  in order, to the first non-blank line in the body, then a timestamp-based
  name. These fallbacks are ishi-specific, since `gist` only surfaces the
  heading.

### `review`

Orchestrates the weekly-review walk, built on `items` + `editor`'s prompting
pattern. Iterates `Project` and `Area` items and asks a `Ui` to keep,
archive, or skip each one: keep stamps `last_reviewed` via `items`; archive
moves the item via `items::mv` (origin category preserved as usual) without
touching `last_reviewed`; skip does neither. A `Decision` enum
(`Keep`/`Archive`/`Skip`) factors out what the walk's `[k]eep`/`[a]rchive`/
`[s]kip` choices do into a single `apply_decision` function, so the effects
are defined in exactly one place.

`run_one` drives a single named `Project`/`Area` item's review decision
without walking the rest: given a `Decision` (from `ishi review <item>
--keep|--archive|--skip`), it applies it directly via `apply_decision` and
returns a one-line confirmation with no `Ui` prompt; given none, it falls
back to the same interactive `[k]eep [a]rchive [s]kip?` prompt the full walk
uses, for just that one item. It resolves the item via `items::locate` and
rejects anything that isn't a `Project` or `Area` — including no match at
all — with `ReviewError::NotReviewable`.

### `cli`

The only component that touches argv, stdin, and stdout. A `clap`-derived
`Command` enum matching the command table in the README, dispatching to
`items`/`review`/`editor` and rendering their results. Defines a `Ui` trait
(`confirm`, `choose`) implemented once for a real terminal and once for
tests, so prompting logic is exercised without a real shell.

Each subcommand gets one `run_*` entry point in `cli` that takes already-
resolved dependencies (`Workspace`, `Editor`, `Ui`) and returns a `Result` —
no direct terminal I/O beyond those trait calls, so `main` stays a thin
argv-to-call-to-print shim and everything else is unit-testable. Notably:

- `run_new` and `run_daily` both render a `Kind`'s template and call
  `items::create`, but diverge because daily notes have a create-or-reopen
  lifecycle `run_new`'s capture-or-named-file shape doesn't fit — `main`
  dispatches `Kind::Daily` to `run_daily` instead of `run_new`.
- `run_move` backs both `ishi move <item> <category>` and the `ishi archive
  <item>` alias (which is `run_move` with `target` fixed to
  `Category::Archive` and no category argument accepted). Moving into
  `Archive` additionally prompts for a one-line summary (via `Ui::confirm`)
  and stamps it on the item before the move; moving to any other category
  is unprompted.
- Bare `ishi config` (no subcommand) resolves and renders the effective config
  directly, bypassing `cli`, since `config::resolve`/`render_effective`(`_json`)
  are already infallible/pure enough not to need a `cli` wrapper.
- `run_list_json`/`run_status_json` are `--json` counterparts to
  `run_list`/`run_status`, not replacements — `main` picks one or the other
  per-invocation based on a `--json` flag. Each defines its own private
  `#[derive(serde::Serialize)]` row/report structs in `cli` rather than
  deriving `Serialize` on `items`' structs directly, keeping `items` free of
  a JSON-shape opinion (it returns facts; `cli` decides how to encode them
  for a human or a machine). An empty result renders `[]` in
  `run_list_json`, never the human-readable empty/no-match message; a
  present-but-`None` optional field (`ListRowJson::origin`,
  `StatusItemJson::reviewed_days_ago`) is omitted from the JSON object
  rather than emitted as `null`.
- `cli` owns the pure item-name-set logic backing tab-completion
  (`live_item_names`/`archived_item_names`), both thin wrappers over
  `items::list` — `main`'s completer functions call these and filter to the
  in-progress argument's prefix, a candidate matches if the typed prefix
  matches either the full candidate string or, for a qualified
  `<category>/<name>` candidate, the `<name>` part alone — so a bare prefix
  of a colliding item's own name still surfaces its qualified forms
  (completions.md 006). `live_item_names` qualifies a name with its category
  (`<category>/<name>`, lowercase, matching `display_path`'s rendering)
  whenever that basename occurs in more than one live category — never
  offering a bare name `ishi move` would now reject as ambiguous (move.md
  006).
- `cli::CliError` (`ItemNotFound`, `NotArchived`) — the typed "not
  found"/"not archived" cases `run_move`/`run_unarchive` raise instead of ad
  hoc `anyhow::anyhow!` strings, so `main` can distinguish them by type and
  map each to a dedicated exit code (README's "Exit codes" section,
  `docs/lld/019-exit-codes.md`). The `Display` text is unchanged from the
  strings it replaces.

`main` maps a failure returned from `dispatch` to one of four dedicated
exit codes (`main::exit_code_for`) by downcasting `anyhow::Error` to the
concrete type each fallible call site actually raised: `cli::CliError`,
`items::ItemsError::{AlreadyArchived,UnwrapNotSupported}`, and
`config::ConfigError::{Parse,Read}` — the last both bare (from `ishi
config`) and wrapped in `workspace::WorkspaceError::Config` (from every
workspace-backed command), since `downcast_ref` sees through `.context(...)`
layers but not through a `thiserror` `#[from]`/`source()` chain. Every other
failure keeps exit code 1, unchanged.

## Appendix: notable invariants

- `Config::category_dirs` is indexed by `Category as usize`, so `Category`'s
  discriminant order and that array's order must stay in sync.
- `config::init` and `workspace`'s editor-exclude/`CLAUDE.md` writers share a
  create-only contract: never overwrite an existing file, regardless of its
  contents.
- `items::locate`'s `Archive` fallback only matches the composite
  `<OriginCategory>/<name>` form, never a bare name, because basenames
  aren't unique across `Archive` origin subfolders. `unarchive` gets its own
  bare-name fallback for `Archive` (`items::locate_archived_bare`), which is
  safe precisely because `unarchive` never resolves outside `Archive` to
  begin with — `locate`'s bare-name exclusion of `Archive` is unchanged.
- `run_config_edit` distinguishes "created" from "already existed" by
  matching `config::init`'s `AlreadyExists` error variant rather than a
  `path.exists()` pre-check, avoiding a TOCTOU gap.
- `ishi completions <shell>` generates a dynamic-completion registration
  snippet (`clap_complete::env`), not a static `aot` script — its content is
  shell glue plus a callback into `ishi` at completion time, not a per-command
  listing baked in at generation time.
