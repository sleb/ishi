# User Stories: `tk completions`

## User Story 001 ✅

- **Summary:** Get shell completions for `tk` without writing them by hand
- **Depends on:** None

### Use Case

- **As a** Tick user who wants tab-completion for `tk` subcommands and flags
- **I want to** run `tk completions <shell>`
- **so that** I can install a generated completion script for my shell instead of writing one myself

### Acceptance Criteria

- **Scenario:** Generating a bash completion script
- **Given:** I am using bash
- **When:** I run `tk completions bash`
- **Then:** Tick prints a bash completion script for `tk` to stdout
- **and Then:** nothing is written to disk — I choose where to save it (e.g. `tk completions bash > ~/.local/share/bash-completion/completions/tk`)

- **Scenario:** Generating a zsh completion script
- **Given:** I am using zsh
- **When:** I run `tk completions zsh`
- **Then:** Tick prints a zsh completion script for `tk` to stdout

- **Scenario:** Generating a fish completion script
- **Given:** I am using fish
- **When:** I run `tk completions fish`
- **Then:** Tick prints a fish completion script for `tk` to stdout

- **Scenario:** Generating a PowerShell completion script
- **Given:** I am using PowerShell
- **When:** I run `tk completions powershell`
- **Then:** Tick prints a PowerShell completion script for `tk` to stdout

---

## User Story 002 ✅

- **Summary:** Get a clear error instead of a broken script for an unsupported shell
- **Depends on:** Story 001 (the generation flow this validates input for)

### Use Case

- **As a** Tick user who mistypes or guesses at a shell name
- **I want to** be told my shell isn't supported
- **so that** I don't mistake a usage error for a valid (but wrong) completion script

### Acceptance Criteria

- **Scenario:** Unrecognized shell name
- **Given:** I run `tk completions tcsh` (a shell Tick doesn't generate completions for)
- **When:** the command runs
- **Then:** Tick exits with an error naming the shells it does support (`bash`, `zsh`, `fish`, `powershell`)
- **and Then:** nothing is printed to stdout that could be mistaken for a completion script

- **Scenario:** Missing shell argument
- **Given:** I run `tk completions` with no `<shell>` argument
- **When:** the command runs
- **Then:** Tick exits with a usage error indicating `<shell>` is required, rather than guessing a default shell

---

## User Story 003 ✅

- **Summary:** Stay current with `tk`'s commands as they change
- **Depends on:** Story 001 (script generation), [init.md](init.md) Story 001, [new.md](new.md) Story 002, [daily.md](daily.md) Story 001, [move.md](move.md) Story 001, [list.md](list.md) Story 001, [status.md](status.md) Story 001, [review.md](review.md) Story 001, [config.md](config.md) Story 001 (every top-level command the script must cover)

### Use Case

- **As a** Tick user who has installed a completion script
- **I want to** the script to reflect this installed version of `tk`'s actual commands, subcommands, and flags
- **so that** completions don't drift out of sync with what `tk` really accepts

### Acceptance Criteria

- **Scenario:** Completions cover all top-level commands
- **Given:** I run `tk completions` for any supported shell
- **When:** Tick generates the script
- **Then:** the script includes completions for every top-level command in `tk`'s CLI definition (`init`, `new`, `daily`, `move`, `list`, `status`, `review`, `config`, `completions`), generated from that definition rather than a hand-maintained list
- **and Then:** running `tk completions` again after a command or flag is added or removed (in a newer `tk` version) reflects that change automatically, with no separate update to the completion script's source

---

## User Story 004 ✅

- **Summary:** Tab-complete a PARA item's name instead of having to know its exact qualified form
- **Depends on:** Story 001 (the installed completion script this extends), [move.md](move.md) Story 001, Story 005 (`<OriginCategory>/<name>` addressing for archived items), [list.md](list.md) Story 001 (the item names being completed)

### Use Case

- **As a** Tick user running `tk move`, `tk archive`, or `tk unarchive` on an item
- **I want to** tab-complete the item's name (including, for archived items, the `<OriginCategory>/<name>` qualified form) straight from what actually exists on disk
- **so that** I don't have to guess or hand-type a name I might get subtly wrong — e.g. adding a `4-Archive/` prefix or a file extension that isn't part of the name `tk` expects

### Acceptance Criteria

- **Scenario:** Completing a live item's name
- **Given:** `my-file.md` exists in `0-Inbox` and `website-redesign` exists as a directory under `1-Projects`
- **When:** I type `tk move ` and press tab (having installed the generated completion script for my shell)
- **Then:** the shell offers `my-file` and `website-redesign` as completions, without their file extension or directory prefix

- **Scenario:** Completing an archived item's qualified name
- **Given:** `meeting-notes.md` was archived from `0-Inbox` and now lives at `4-Archive/Inbox/meeting-notes.md`
- **When:** I type `tk unarchive ` and press tab
- **Then:** the shell offers `Inbox/meeting-notes` as a completion — the `<OriginCategory>/<name>` form `tk unarchive` expects, not the literal archive filesystem path

- **Scenario:** Completions reflect the current directory's PARA system, not a fixed list
- **Given:** I am in a directory containing a PARA system with a particular set of items
- **When:** I request completions for an item-name argument
- **Then:** the offered names are read from that PARA system at completion time
- **and Then:** running the same completion in a different PARA system (or after items are added, moved, or archived) offers different names accordingly

- **Scenario:** No PARA system in the current directory
- **Given:** I am in a directory that is not a Tick-initialized PARA system
- **When:** I request completions for an item-name argument
- **Then:** the shell offers no item-name completions, and the completion attempt does not error out or hang

---

## User Story 005

- **Summary:** Completing a name that's ambiguous across categories offers the qualified `<Category>/<name>` forms, not a bare name that would just get rejected
- **Status:** ✅
- **Depends on:** Story 004 (the completion mechanism this refines), [move.md](move.md) Story 006 (the ambiguity rejection and generalized `<Category>/<name>` qualified addressing this surfaces)

### Use Case

- **As a** Tick user tab-completing an item name that happens to collide with another item's basename
- **I want to** be offered the qualified forms directly
- **so that** I never complete my way into the "ambiguous name" error move.md Story 006 introduced — tab-completion should only ever offer names `tk move` will actually accept

### Acceptance Criteria

- **Scenario:** Completing a name that collides across two live categories offers both qualified forms
- **Given:** `meeting-notes.md` exists in both `0-Inbox` and `3-Resources`
- **When:** I type `tk move ` (no further characters) and press tab
- **Then:** the shell offers `inbox/meeting-notes` and `resources/meeting-notes` as completions, not the bare `meeting-notes`

- **Scenario:** Completing a name that's unique across all categories still offers the bare form
- **Given:** `website-redesign` exists only as a directory under `1-Projects`
- **When:** I type `tk move website` and press tab
- **Then:** the shell offers `website-redesign` as a completion — no qualification is needed since there's nothing to disambiguate

> **Note:** as shipped, this story only guarantees qualified forms are offered when nothing (or an already-unambiguous prefix) has been typed — see Story 006 for typing a bare prefix of the colliding name itself (e.g. `meeti`).

---

## User Story 006

- **Summary:** Typing a bare prefix of a colliding item's name still offers its qualified completion forms
- **Status:** ⬜
- **Depends on:** Story 005 (the qualification this extends matching for)

### Use Case

- **As a** Tick user tab-completing an item name that happens to collide with another item's basename
- **I want to** have my typed prefix matched against the item's own name, not just against the qualified `<category>/` string in front of it
- **so that** I can still find and complete a colliding item by typing the start of its actual name, the same way I would for any other item

### Acceptance Criteria

- **Scenario:** A bare prefix of a colliding name still surfaces both qualified forms
- **Given:** `meeting-notes.md` exists in both `0-Inbox` and `3-Resources`
- **When:** I type `tk move meeti` and press tab
- **Then:** the shell offers `inbox/meeting-notes` and `resources/meeting-notes` as completions

- **Scenario:** A bare prefix that only matches one colliding item's category is unaffected
- **Given:** `meeting-notes.md` exists in both `0-Inbox` and `3-Resources`
- **When:** I type `tk move inbox/meeti` and press tab
- **Then:** the shell offers `inbox/meeting-notes` as a completion, matching on the qualified form as it does today

- **Scenario:** Unique names keep matching exactly as before
- **Given:** `website-redesign` exists only as a directory under `1-Projects`
- **When:** I type `tk move website` and press tab
- **Then:** the shell offers `website-redesign` as a completion, unaffected by this change
