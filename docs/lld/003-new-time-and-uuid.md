# LLD: `tk new` — Stories 011–012 (`{{time}}` and `{{uuid}}` template placeholders)

Source: [docs/user-stories/new.md](../user-stories/new.md), User Stories
011–012. Module boundaries follow [docs/design.md](../design.md).

## Scope

Both stories in `new.md`:

1. Story 011 — `{{time}}` renders as the current time (`14:32`) in both the
   named/non-interactive path and the `$EDITOR` pre-population path.
2. Story 012 — `{{uuid}}` renders as a freshly generated unique id in both
   paths, and two separate notes get two different ids.

The README's placeholder table (`README.md`) already documents `{{time}}`
and `{{uuid}}` as if implemented — per `CLAUDE.md`, the docs describe target
behavior ahead of the code, so this LLD is what makes that table accurate.

### Out of scope (future LLDs)

- Story 013 (`--daily` folded into `new`) — unrelated flag/dispatch work,
  not a `render` change.
- Any change to `{{date}}`, `{{title}}`, or `{{cursor}}` handling — those
  stay exactly as implemented.

## `design.md` changes (to apply once this lands)

- `config::render`'s signature grows from `(template, title, date)` to
  `(template, title, date, time, uuid)` — still a pure function of its
  inputs. All five caller-supplied values are plain strings; `render` does
  no clock or RNG calls of its own.
- `cli::run_new` computes `time` and `uuid` alongside the existing `today`
  computation, in both the named and editor-capture branches.

## Module designs

### `config` (extends existing module)

```rust
pub fn render(template: &str, title: &str, date: &str, time: &str, uuid: &str) -> String {
    template
        .replace("{{date}}", date)
        .replace("{{title}}", title)
        .replace("{{time}}", time)
        .replace("{{uuid}}", uuid)
}
```

`{{cursor}}` is untouched, as before — still `Editor`'s job.

**Why `uuid` is a caller-supplied string, not generated inside `render`:**
`date` is already computed by the caller (`Local::now()`) and passed in as a
plain string, which keeps `render` pure and lets tests assert an exact
output string rather than pattern-matching a live value. `time` and `uuid`
follow the same shape for consistency: `render` takes no dependency on
`chrono` or `uuid` internals, and every test can assert a fully determined
output string, including the `{{uuid}}` case. The cost is that `run_new`
generates a UUID on every call even when the active template doesn't
contain `{{uuid}}` — negligible (no I/O, just RNG + string formatting), so
not worth branching on template contents to avoid it.

### `cli` (extends existing `run_new`)

Both branches of `run_new` currently compute `today` from a fresh
`Local::now()` call each. Change each to compute `date`/`time` from a single
`Local::now()` call (so they can't skew across a call boundary) and
generate one `uuid`:

```rust
let now = Local::now();
let today = now.date_naive().format("%Y-%m-%d").to_string();
let time = now.format("%H:%M").to_string();
let uuid = Uuid::new_v4().to_string();
```

then pass `&time, &uuid` into both `config::render` call sites
(`src/cli.rs:71` and `:77`).

## Dependencies

Add to `Cargo.toml`:

```toml
uuid = { version = "1", features = ["v4"] }
```

## Test plan (TDD — write these first)

| Scenario | Test | Module |
|---|---|---|
| `{{time}}` renders to the passed-in time | `render(template_with_time, "", date, "14:32", uuid)` → time marker replaced with `14:32` | `config` (unit) |
| `{{uuid}}` renders to the passed-in id | `render(template_with_uuid, "", date, time, "f47ac10b-...")` → uuid marker replaced with that exact string | `config` (unit) |
| All markers render together, cursor still untouched | template with `{{date}}`, `{{title}}`, `{{time}}`, `{{uuid}}`, `{{cursor}}` all present → exact expected output string, `{{cursor}}` preserved literally | `config` (unit, extends existing `render_fills_date_and_title_but_leaves_cursor_marker`) |
| Named note renders `{{time}}` | `tk new my-file` with a `note` template containing `{{time}}` → created file contains a `HH:MM`-shaped time | `cli` (unit, per Story 011 scenario 1) |
| Editor capture renders `{{time}}` | `tk new` with no args, fake editor, template containing `{{time}}` → seed passed to `Editor::capture` has `{{time}}` already rendered | `cli` (unit, per Story 011 scenario 2) |
| Named note renders `{{uuid}}` | `tk new my-file` with a `note` template containing `{{uuid}}` → created file contains a well-formed UUID (regex match, since the value itself is nondeterministic at this layer) | `cli` (unit, per Story 012 scenario 1) |
| Editor capture renders `{{uuid}}` | same as above but via the no-args editor-capture path | `cli` (unit, per Story 012 scenario 2) |
| Two notes get different ids | `tk new first-note` then `tk new second-note` with a `{{uuid}}`-containing template → the two created files' rendered ids differ | `cli` (unit, per Story 012 scenario 3) |

## Implementation plan

1. Extend `config::render` to `(template, title, date, time, uuid)`;
   update its existing test call site and add the new `config`-level tests
   above. Watch them fail, then implement.
2. Add the `uuid` dependency to `Cargo.toml`.
3. Update both `config::render` call sites in `cli::run_new` to compute
   `time`/`uuid` from a single `Local::now()` plus one `Uuid::new_v4()`,
   and pass them through. Add/extend the `cli`-level tests above.
4. Update `docs/design.md`'s `config` section to document the new
   `render` signature and the "caller supplies all five, `render` stays
   pure" rationale.
5. Update `docs/user-stories/new.md` to mark Stories 011 and 012 `✅`.
6. Manual smoke test: set a custom `note` template in `.tick.toml`
   containing `{{time}}` and `{{uuid}}`, run `tk new some-note` and
   `tk new --project foo` (interactive, via `$EDITOR`), confirm both
   render correctly; run `tk new` twice and confirm the two `{{uuid}}`
   values differ.
7. `cargo clippy`, `cargo fmt --check`, `cargo test` clean before calling
   the stories done.
