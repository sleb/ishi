---
name: lld
description: Write a low-level design (LLD) doc for a tick (`tk`) feature or user-story scenario, following docs/lld/TEMPLATE.md. Use before implementing a new roadmap item or story scenario, once the story's acceptance criteria exist but before any code is written.
---

# `lld`: author a tick LLD from the template

Produces a `docs/lld/NNN-<slug>.md` document for a feature about to be
implemented, filled out per the structure in
[`docs/lld/TEMPLATE.md`](../../../docs/lld/TEMPLATE.md). The template's own
comments describe what belongs in each section in detail — read it fresh
each time rather than relying on memory, since it may change.

## When to use

- A `docs/user-stories/<story>.md` scenario is about to move from
  "documented" to "implemented" and needs a design pass first.
- A roadmap item in `docs/roadmap.md` is being picked up.
- Do **not** use this for pure bug fixes or refactors with no new
  user-facing scenario — those don't need an LLD.

## Inputs to gather before writing

1. **Story file(s) and scenario numbers.** Read the relevant
   `docs/user-stories/*.md` file(s) in full — every Given/When/Then
   scenario the LLD claims to cover must be quoted/paraphrased accurately
   with its scenario number.
2. **Current `docs/design.md`.** Read the sections for every module the
   change touches (`workspace`, `items`, `review`, `editor`, `cli`, etc.)
   so the LLD's "Module designs" section states an accurate diff against
   what's documented today, not a guess.
3. **Roadmap item.** Check `docs/roadmap.md` for the corresponding line
   item so the LLD's header can reference it correctly.
4. **Next LLD number.** LLD files are deleted once their story ships (see
   the provenance note at the top of `docs/roadmap.md`), so the working
   tree usually does *not* contain every past LLD. Find the true max by
   scanning git history, not just `ls docs/lld/`:
   ```
   git log --all --diff-filter=A --name-only -- 'docs/lld/*.md' \
     | grep -E '^docs/lld/[0-9]{3}-' \
     | sed -E 's#docs/lld/([0-9]{3})-.*#\1#' \
     | sort -n | tail -1
   ```
   Use `max + 1`, zero-padded to 3 digits, for the new filename.

## Filling out the template

Follow `docs/lld/TEMPLATE.md` section by section:

- **Title/header** — feature name, story id(s), story file, roadmap item
  number.
- **Scope** — one numbered line per scenario covered, each citing
  `<story>.md` and its scenario number. If doing more than the stories
  strictly require, call that out explicitly with a rationale (give it its
  own subsection if non-obvious — see `004-tk-daily.md`'s "Why `Kind`, not
  `Category::Daily`" in git history for the pattern, via
  `git show <sha>:docs/lld/004-tk-daily.md`).
- **Out of scope** — adjacent stories/features deliberately not touched,
  and why.
- **`design.md` changes** — state plainly whether `docs/design.md` already
  reflects the change or the update is deferred until the LLD lands, and
  summarize the diff.
- **Module designs** — one subsection per touched module, ordered
  innermost/purest first and `main`/`cli` last (matching the module
  boundary in `CLAUDE.md`: filesystem/business logic in `workspace`,
  `items`, `review`, `editor`; terminal I/O only in `cli`). Show real Rust
  signatures, not prose descriptions of them. Explain *why*, not *what*.
- **Test plan** — a table, written before any implementation, one row per
  test: scenario, what the test does/asserts, module + test kind (unit vs.
  story reference). This is the literal TDD checklist — it should be
  usable as-is to start writing failing tests.
- **Implementation plan** — numbered, dependency-ordered steps ending in:
  marking the story `✅` in its `docs/user-stories/` file, updating
  `docs/roadmap.md`, a manual smoke test with concrete `tk` invocations,
  and a `cargo clippy && cargo fmt --check && cargo test` clean bar.

## After writing

- Save to `docs/lld/NNN-<slug>.md`.
- Remind whoever implements it: this file is scaffolding, not permanent
  documentation. Once the story ships, delete the LLD and make sure its
  content that should persist has landed in `docs/design.md` and the
  story file's `✅` marks — don't leave stale LLDs around after the code
  lands.
