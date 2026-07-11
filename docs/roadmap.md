# Roadmap

> `docs/lld/NNN-*.md` filenames referenced below are historical: each LLD is
> deleted once its story ships (see `docs/lld/TEMPLATE.md`), so these are
> provenance notes, not live links.

## Remaining work

One item left.

### 1. Un-archiving (moving an item back out of `Archive`)

**Not started.** No user-stories/move.md story exists yet for this.

`docs/design.md`'s `items::locate` and `items::mv` sections describe target
behavior for moving an item _out of_ `Archive` (keyed off the target
category, not the origin subfolder recorded under `Archive`), and cite it
as "move.md Story 005" — but that story doesn't exist in
`docs/user-stories/move.md` yet, and `items::locate` still only searches
`Category::archivable()`, never `Archive` itself.

**Why not done:** blocked on writing the user story itself. Draft Story
005 into move.md first, then an LLD, then implement (extending
`items::locate`/`items::mv` to handle `Archive` as a source). Depends on
`move`, which is done.

## Everything else is done

| Command       | Notes                                                                     |
| ------------- | -------------------------------------------------------------------------- |
| `init`        | Done — includes Stories 005–006 (editor excludes, `CLAUDE.md`)            |
| `new`         | Done — includes `--project`/`--area`/`--resource`, templates, placeholders |
| `daily`       | Done                                                                        |
| `move`        | Stories 001, 002, 004 done. (Story 005, un-archiving, is remaining work above.) |
| `archive`     | Done — sugar alias for `tk move <item> archive`                            |
| `list`        | Done                                                                        |
| `status`      | Done                                                                        |
| `review`      | Done                                                                        |
| `config`      | Done — layering, `config init`/`edit` (`-g`), provenance display, JSON Schema |
| `completions` | Done                                                                        |

Implementation notes for finished work (design rationale, which LLD each
story shipped under) live in git history and in `docs/design.md`, not
here.

## Explicitly out of scope for this pass

Anything not in the README's command table (e.g. sync, plugins, multi-user
config) — not hinted at anywhere in the spec, so not roadmapped until
there's a concrete story for it.
