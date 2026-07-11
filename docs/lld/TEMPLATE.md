# LLD: `<feature name>` — Stories `<story-ids>` (`<story-file>.md`)

Source: [docs/user-stories/`<story-file>`.md](../user-stories/`<story-file>`.md)
Stories `<ids>`. Module boundaries follow [docs/design.md](../design.md).
Corresponds to roadmap item `<n>`.

## Scope

List each user-story scenario this LLD implements, numbered, one line each,
referencing the story file/number it covers.

1. `<scenario 1>` (`<story>.md` NNN)
2. `<scenario 2>` (`<story>.md` NNN)

If a change is bigger than the stories strictly require (e.g. introducing a
new type/vocabulary to avoid a special case), call it out explicitly here
with a short rationale, and give the full rationale its own subsection below
if it's non-obvious enough to need one (see `004-tk-daily.md`'s "Why `Kind`,
not `Category::Daily`" for the pattern).

### Out of scope

List adjacent stories/features this LLD deliberately does not touch, and
why (usually: "separate LLD" or "unrelated to this change"). This keeps
scope creep visible and gives the next LLD a clear starting point.

- `<adjacent story/feature>` — `<reason it's excluded>`

## `design.md` changes

State whether these changes have already been applied to `docs/design.md`
or are deferred until this LLD lands, and summarize what changes:

- `<module>` gains/changes `<type/function>` — see the `<module>` section
  of `design.md` for the finalized contract.

## Module designs

One subsection per module touched, in the order data/control flows through
them (innermost/purest module first, `main` last). For each module:

- A fenced Rust code block with the new/changed public signatures — types,
  function signatures, enum variants, error variants. Not necessarily full
  bodies; show bodies when the logic itself is the design decision.
- Prose explaining *why*, not what — non-obvious tradeoffs, what's
  deliberately duplicated vs. shared, what stays unchanged and why callers
  don't need to adapt.
- Note any required updates to existing call sites/tests as a consequence
  of a signature change, so the implementation plan can reference it.

### `<module>` (extends existing module / new module)

```rust
// new/changed public API
```

Rationale, tradeoffs, and any call-site fallout.

## Test plan (TDD — write these first)

A table, one row per test, written before implementation:

| Scenario | Test | Module |
|---|---|---|
| `<behavior being verified>` | `<what the test does and asserts>` | `<module>` (unit, `<story>.md` NNN) |

Cover: the happy path per scenario in Scope, each documented edge case,
error conditions, and any regression risk from a changed signature
(existing call sites still passing, unaffected behavior unchanged).

## Implementation plan

Numbered steps in build order — write the failing test, then implement,
module by module in dependency order, ending with docs/roadmap updates and
a manual smoke test. Adapt the count/wording to the feature; this shape is
the default:

1. Add/change `<module>`'s types and unit tests first; watch them fail,
   then implement.
2. Repeat per module, in dependency order (innermost module first).
3. Update call sites and their existing tests where a signature changed;
   confirm no unrelated behavior regressed.
4. Wire up `main`/CLI dispatch, with its own parse/dispatch tests.
5. Mark the covered stories `✅` in `docs/user-stories/<story>.md`.
6. Update `docs/roadmap.md`'s status for the corresponding item.
7. Manual smoke test: concrete `ishi <command>` invocations to run by hand
   and what to look for in the output.
8. `cargo clippy`, `cargo fmt --check`, `cargo test` clean before calling
   the stories done.
