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
| `move`        | Done, except Story 006 below                                                    |
| `archive`     | Done                                                                            |
| `list`        | Done                                                                            |
| `status`      | Done                                                                            |
| `review`      | Done                                                                            |
| `config`      | Done                                                                            |
| `completions` | Done, except Story 005 below                                                    |

## Outstanding stories

| Story | Summary | LLD |
| --- | --- | --- |
| [move.md](user-stories/move.md) Story 006 | Reject a bare name that matches items in more than one live category instead of silently resolving it, generalizing the `<Category>/<name>` qualified form to every live category | `docs/lld/016-move-ambiguous-name-resolution.md` |
| [completions.md](user-stories/completions.md) Story 005 | Tab-completion offers the qualified `<Category>/<name>` forms for a colliding name instead of a bare name Story 006 would reject | `docs/lld/017-completions-ambiguous-name-offers-qualified-forms.md` |

Story 005 depends on Story 006 landing first — see the sequencing note in
its LLD.

Implementation notes for finished work (design rationale, which LLD each
story shipped under) live in git history and in `docs/design.md`, not
here.

## Explicitly out of scope for this pass

Anything not in the README's command table (e.g. sync, plugins, multi-user
config) — not hinted at anywhere in the spec, so not roadmapped until
there's a concrete story for it.
