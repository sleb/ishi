# Spec gaps: README → user-stories

Where [README.md](../README.md) documents behavior that has no corresponding
Given/When/Then coverage in [user-stories](user-stories/) — either because no
story file exists for the command at all, or because a story file exists but
misses something the README describes or leaves ambiguous. This is a
prerequisite check, not an implementation check — see [roadmap.md](roadmap.md)
for spec vs. `src/`.

## 1. Commands with no story file at all

`docs/user-stories/` has eight files: `init.md`, `new.md`, `config.md`,
`mv.md`, `daily.md`, `list.md`, `status.md`, `review.md`. One command in the
README's command table still has zero Given/When/Then coverage:

- **`completions`** — no stories at all; lowest-stakes gap since it's a thin
  `clap_complete` wrapper with no branching logic to pin down.
