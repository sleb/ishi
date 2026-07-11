# Roadmap

> `docs/lld/NNN-*.md` filenames referenced below are historical: each LLD is
> deleted once its story ships (see `docs/lld/TEMPLATE.md`), so these are
> provenance notes, not live links.

## Commands

| Command       | Notes                                                                           |
| ------------- | ------------------------------------------------------------------------------- |
| `init`        | Done                                                                            |
| `new`         | Done                                                                            |
| `daily`       | Done                                                                            |
| `move`        | Done                                                                            |
| `archive`     | Done                                                                            |
| `list`        | Done                                                                            |
| `status`      | Done                                                                            |
| `review`      | Done                                                                            |
| `config`      | Done                                                                            |
| `completions` | Done, except Story 006 below                                                    |

## Outstanding stories

| Story | Summary | LLD |
| --- | --- | --- |
| [completions.md](user-stories/completions.md) Story 006 | Typing a bare prefix of a colliding item's name (not just an empty or already-qualified prefix) still offers its qualified completion forms | none yet — write one before implementing |

Implementation notes for finished work (design rationale, which LLD each
story shipped under) live in git history and in `docs/design.md`, not
here.

## Explicitly out of scope for this pass

Anything not in the README's command table (e.g. sync, plugins, multi-user
config) — not hinted at anywhere in the spec, so not roadmapped until
there's a concrete story for it.
