# User Stories: `tk init`

## User Story 001

- **Summary:** Turn the current directory into a PARA system with one command
- **Status:** Completed
- **Depends on:** None

### Use Case

- **As a** new Tick user starting from an empty (or existing) working directory
- **I want to** run `tk init` with no arguments
- **so that** I get a ready-to-use PARA structure right here, without deciding on a project name first

### Acceptance Criteria

- **Scenario:** Initialize in the current directory
- **Given:** I am in a directory that is not already a PARA system
- **When:** I run `tk init`
- **Then:** Tick creates `0-Inbox`, `1-Projects`, `2-Areas`, `3-Resources`, and `4-Archive` in the current directory
- **and Then:** Tick prints `Created PARA system in .`

---

## User Story 002

- **Summary:** Start a new, separately-named PARA system without leaving my current folder
- **Status:** Completed
- **Depends on:** Story 001 (same scaffolding behavior, applied to a named subdirectory)

### Use Case

- **As a** Tick user setting up a new PARA system alongside other work
- **I want to** run `tk init <name>`
- **so that** the new system is scaffolded into its own subdirectory, instead of taking over the directory I'm already in

### Acceptance Criteria

- **Scenario:** Initialize into a named subdirectory
- **Given:** I am in a directory that does not contain a subdirectory called `<name>`
- **When:** I run `tk init my-para`
- **Then:** Tick creates `./my-para` containing `0-Inbox`, `1-Projects`, `2-Areas`, `3-Resources`, and `4-Archive`
- **and Then:** Tick prints `Created PARA system in ./my-para`

---

## User Story 003

- **Summary:** Re-running `init` fills in whatever's missing instead of failing outright
- **Status:** Completed
- **Depends on:** Story 001, Story 002 (fills in gaps for either the current-directory or named-subdirectory target)

### Use Case

- **As a** Tick user who might run `init` more than once, or who deleted a category folder by accident
- **I want to** have `init` create only the category folders that don't already exist
- **so that** I can repair or complete a partial PARA system without it complaining or duplicating what's already there

### Acceptance Criteria

- **Scenario:** Re-initializing a complete PARA system is a no-op
- **Given:** the target directory (current directory, or `./<name>` if given) already contains all five category folders
- **When:** I run `tk init` (with or without a name)
- **Then:** Tick creates no new files or directories
- **and Then:** Tick reports that the PARA system is already complete, with no changes made

- **Scenario:** Re-initializing a partial PARA system fills in the gaps
- **Given:** the target directory contains some but not all of the five category folders (e.g. `0-Inbox` exists but `1-Projects` does not)
- **When:** I run `tk init` (with or without a name)
- **Then:** Tick creates only the missing category folders, leaving existing ones (and their contents) untouched
- **and Then:** Tick reports which folders it created

---

## User Story 004

- **Summary:** Get a clear error instead of a confusing filesystem failure when the target path is unusable
- **Status:** Completed
- **Depends on:** Story 002 (named target), Story 003 (partial-directory handling that this story's error path is an exception to)

### Use Case

- **As a** Tick user who might typo or reuse a name that collides with an existing file
- **I want to** be told when `<name>` already exists as a regular file
- **so that** I understand why `init` didn't succeed instead of seeing a raw filesystem error

### Acceptance Criteria

- **Scenario:** Target name collides with an existing file
- **Given:** `./<name>` already exists but is a regular file, not a directory
- **When:** I run `tk init <name>`
- **Then:** Tick prints an error explaining that `./<name>` already exists and isn't a directory
- **and Then:** no files or directories are created or modified

- **Scenario:** Target name collides with an existing directory that has unrelated contents
- **Given:** `./<name>` already exists as a directory containing files or folders that aren't among the five category folders
- **When:** I run `tk init <name>`
- **Then:** Tick treats it the same as any other existing directory: it creates whichever of the five category folders are missing, and leaves the unrelated contents untouched (see Story 003)
- **and Then:** Tick does **not** treat the unrelated contents as an error

---

## User Story 005

- **Summary:** Initializing a PARA system sets up editor exclude config for the archive folder, for both VS Code and Zed
- **Status:** ✅ Completed
- **Depends on:** Story 001, Story 002 (creates settings alongside whichever target init scaffolds)

### Use Case

- **As a** Tick user setting up a new PARA system
- **I want to** have `init` configure my editor to skip the archive folder in fuzzy-find/quick-open
- **so that** the archive stays out of my way from the start, without me needing to configure it myself or wait until I first archive something

### Acceptance Criteria

- **Scenario:** `init` creates a Zed exclude entry when none exists
- **Given:** the target directory (current directory, or `./<name>` if given) has no `.zed/settings.json`
- **When:** I run `tk init` (with or without a name)
- **Then:** Tick creates `.zed/settings.json` at the target with `file_scan_exclude` containing the configured archive folder name (`4-Archive` by default)

- **Scenario:** `init` creates a VS Code exclude entry when none exists
- **Given:** the target directory has no `.vscode/settings.json`
- **When:** I run `tk init` (with or without a name)
- **Then:** Tick creates `.vscode/settings.json` at the target with both `files.exclude` and `search.exclude` mapping the configured archive folder name to `true`

- **Scenario:** An existing `.zed/settings.json` is left untouched, with instructions printed instead
- **Given:** the target directory already has a `.zed/settings.json` (with any contents)
- **When:** I run `tk init`
- **Then:** Tick does not modify `.zed/settings.json`
- **and Then:** Tick prints instructions telling me to manually add the configured archive folder name to `file_scan_exclude` in `.zed/settings.json`

- **Scenario:** An existing `.vscode/settings.json` is left untouched, with instructions printed instead
- **Given:** the target directory already has a `.vscode/settings.json` (with any contents)
- **When:** I run `tk init`
- **Then:** Tick does not modify `.vscode/settings.json`
- **and Then:** Tick prints instructions telling me to manually add the configured archive folder name to `files.exclude`/`search.exclude` in `.vscode/settings.json`

- **Scenario:** A custom archive folder name from `.tick.toml` is what gets referenced
- **Given:** the target's `.tick.toml` sets `[folders] archive = "9-Attic"`, with no `.zed/settings.json` or `.vscode/settings.json` yet
- **When:** I run `tk init`
- **Then:** the exclude entries Tick creates name `9-Attic`, not `4-Archive`

- **Scenario:** Re-running `init` when the settings files already exist keeps printing instructions
- **Given:** `.zed/settings.json` and `.vscode/settings.json` already exist at the target (from a previous `tk init` run or created manually)
- **When:** I run `tk init` again
- **Then:** Tick makes no changes to either file, and prints the same manual-update instructions again

---

## User Story 006

- **Summary:** Initializing a PARA system creates a `CLAUDE.md` instructing agents to leave the archive alone unless asked
- **Status:** ✅ Completed
- **Depends on:** Story 001, Story 002 (creates the file alongside whichever target init scaffolds)

### Use Case

- **As a** Tick user who works in this PARA system alongside an AI agent
- **I want to** have `init` set up `CLAUDE.md` telling my agent to skip the archive by default
- **so that** the agent's context stays focused on what's active from the moment I set up the system, not just after I've run my first archiving move

### Acceptance Criteria

- **Scenario:** `init` creates `CLAUDE.md` with the instruction when none exists
- **Given:** the target directory (current directory, or `./<name>` if given) has no `CLAUDE.md`
- **When:** I run `tk init` (with or without a name)
- **Then:** Tick creates a `CLAUDE.md` at the target root containing an instruction not to read files under the configured archive folder (`4-Archive` by default) unless the user explicitly asks or there's a strong, specific reason to

- **Scenario:** An existing `CLAUDE.md` is left untouched, with instructions printed instead
- **Given:** the target directory already has a `CLAUDE.md` (with or without the archive instruction)
- **When:** I run `tk init`
- **Then:** Tick does not modify `CLAUDE.md`
- **and Then:** Tick prints instructions telling me to manually add the archive-skip instruction to `CLAUDE.md`, naming the configured archive folder

- **Scenario:** A custom archive folder name is what the generated instruction names
- **Given:** the target's `.tick.toml` sets `[folders] archive = "9-Attic"`, with no `CLAUDE.md` yet
- **When:** I run `tk init`
- **Then:** the instruction Tick writes names `9-Attic`, not `4-Archive`

- **Scenario:** Re-running `init` when `CLAUDE.md` already exists keeps printing instructions
- **Given:** `CLAUDE.md` already exists at the target (from a previous `tk init` run or created manually)
- **When:** I run `tk init` again
- **Then:** Tick doesn't modify it, and prints the same manual-update instructions again
