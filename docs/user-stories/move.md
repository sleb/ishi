# User Stories: `tk move`

`mv` is an alias for `move`.

`tk archive <item>` is sugar for `tk move <item> archive` — a shorter, more
memorable way to file something away. But the whole point of having an
archive instead of just deleting things is that it stays *out of the way*
until you actually need it: your editor's fuzzy-find shouldn't surface it,
and an agent reading your notes shouldn't burn context on it unless you
specifically ask. So beyond the move itself, moving an item to `archive`
is also responsible for keeping two low-friction affordances up to date
every time it runs: editor exclude config, and an agent-facing instruction
file. It also stamps a one-line summary into the item being archived, so
an agent that does have a reason to look can get the gist without reading
the whole note. These affordances (Stories 004-006) live on `tk move`
itself, specific to the `archive` destination, rather than on the `tk
archive` alias — so `tk archive` gets them for free, without needing its
own wiring beyond delegating to `move`.

## User Story 001

- **Summary:** Moving an item relocates it to the target category, wrapping into a directory or preserving origin as needed
- **Status:** ✅
- **Depends on:** [new.md](new.md) Story 002, Story 003, Story 004 (items to move must exist first)

### Use Case

- **As a** Tick user reorganizing my notes
- **I want to** move an item to a different category with one command
- **so that** I don't have to manually create directories, move files, and clean up after myself when an item's status changes

### Acceptance Criteria

- **Scenario:** Moving a flat file into `project` wraps it into a new directory with an `index.md`
- **Given:** `my-file.md` exists in `0-Inbox`
- **When:** I run `tk move my-file project`
- **Then:** Tick prints `Moved ./0-Inbox/my-file.md to ./1-Projects/my-file/index.md`
- **and Then:** `0-Inbox/my-file.md` no longer exists, and `1-Projects/my-file/index.md` has the same content `my-file.md` had

- **Scenario:** Moving a flat file into `area` wraps it the same way
- **Given:** `my-file.md` exists in `3-Resources`
- **When:** I run `tk move my-file area`
- **Then:** Tick prints `Moved ./3-Resources/my-file.md to ./2-Areas/my-file/index.md`

- **Scenario:** Moving a flat file into `inbox` or `resource` relocates it without wrapping
- **Given:** `my-file.md` exists in `3-Resources`
- **When:** I run `tk move my-file inbox`
- **Then:** Tick prints `Moved ./3-Resources/my-file.md to ./0-Inbox/my-file.md`
- **and Then:** no directory is created — the destination is still a flat file

- **Scenario:** Moving a project directory to `area` (or vice versa) relocates the directory as-is, without re-wrapping
- **Given:** `website-redesign` exists as a directory under `1-Projects`
- **When:** I run `tk move website-redesign area`
- **Then:** Tick prints `Moved ./1-Projects/website-redesign to ./2-Areas/website-redesign`
- **and Then:** `website-redesign/index.md` and every other file in the directory are unchanged, just relocated

- **Scenario:** Moving any category's item to `archive` preserves its origin category as a subfolder
- **Given:** `website-redesign` exists as a directory under `1-Projects`
- **When:** I run `tk move website-redesign archive`
- **Then:** Tick prints `Moved ./1-Projects/website-redesign to ./4-Archive/Projects/website-redesign`

- **Scenario:** Archiving a flat file also preserves its origin category as a subfolder
- **Given:** `my-file.md` exists in `3-Resources`
- **When:** I run `tk move my-file archive`
- **Then:** Tick prints `Moved ./3-Resources/my-file.md to ./4-Archive/Resources/my-file.md`

- **Scenario:** The `mv` alias behaves identically to `move`
- **Given:** `my-file.md` exists in `0-Inbox`
- **When:** I run `tk mv my-file project`
- **Then:** Tick prints `Moved ./0-Inbox/my-file.md to ./1-Projects/my-file/index.md`, exactly as `tk move my-file project` would

---

## User Story 002

- **Summary:** Get a clear error instead of a silent guess when unwrapping a directory item isn't supported
- **Status:** ✅
- **Depends on:** [new.md](new.md) Story 003, Story 004 (project/area scaffolding — the directory items this rejection applies to)

### Use Case

- **As a** Tick user who wants to move a `project` or `area` item back to `inbox` or `resource`
- **I want to** be told that unwrapping a directory into a flat file isn't supported
- **so that** Tick never has to guess which file inside the directory becomes the flat file, and I don't lose the rest of the directory's contents silently

### Acceptance Criteria

- **Scenario:** Moving a project directory to `inbox` or `resource` is rejected
- **Given:** `<item>` exists as a directory under `1-Projects` or `2-Areas`
- **When:** I run `tk move <item> inbox` or `tk move <item> resource`
- **Then:** Tick prints an error explaining that unwrapping a directory into a flat file is not yet supported
- **and Then:** no files or directories are moved, created, or modified

- **Scenario:** Moving an area directory to `inbox` or `resource` is rejected
- **Given:** `<item>` exists as a directory under `2-Areas`
- **When:** I run `tk move <item> inbox` or `tk move <item> resource`
- **Then:** Tick prints an error explaining that unwrapping a directory into a flat file is not yet supported
- **and Then:** no files or directories are moved, created, or modified

---

## User Story 003

- **Summary:** `tk archive <item>` is sugar for `tk move <item> archive` — a shorter, more memorable way to file something away
- **Status:** Not started
- **Depends on:** Story 001 (the move semantics this delegates to)

### Use Case

- **As a** Tick user who has decided a project, area, resource, or inbox note is done
- **I want to** run `tk archive <item>` instead of `tk move <item> archive`
- **so that** I don't have to remember or type the destination category name for the one destination this command only ever has

### Acceptance Criteria

- **Scenario:** Archiving a project files it under the Archive's Projects subfolder
- **Given:** I am inside an initialized PARA system with a project `website-redesign`
- **When:** I run `tk archive website-redesign`
- **Then:** `website-redesign` is moved from `1-Projects/website-redesign` to `4-Archive/Projects/website-redesign`, exactly as `tk move website-redesign archive` would move it
- **and Then:** Tick prints the same `Moved ...` confirmation `tk move` prints

- **Scenario:** Archiving an inbox note or resource files it under the matching subfolder
- **Given:** I am inside an initialized PARA system with a resource `my-file.md`
- **When:** I run `tk archive my-file`
- **Then:** `my-file.md` is moved from `3-Resources/my-file.md` to `4-Archive/Resources/my-file.md`

- **Scenario:** `tk archive` takes no destination argument
- **Given:** I am inside an initialized PARA system
- **When:** I run `tk archive my-file archive`
- **Then:** Tick rejects the command with an error — `tk archive` doesn't accept a category argument, since the destination is always `archive`

- **Scenario:** Archiving a directory item that doesn't support unwrapping still only applies to the reverse direction
- **Given:** I am inside an initialized PARA system with a project `website-redesign`
- **When:** I run `tk archive website-redesign`
- **Then:** the move succeeds — Story 002's rejection only applies to unwrapping a directory back into `inbox`/`resource`, never to archiving

---

## User Story 004

- **Summary:** Moving an item to `archive` keeps the archive folder excluded from editor fuzzy-find/quick-open, for both VS Code and Zed
- **Status:** Not started
- **Depends on:** Story 001 (runs as part of the same command)

### Use Case

- **As a** Tick user who uses Zed (and sometimes VS Code)
- **I want to** never see archived items pop up when I hit cmd+P to jump to a file
- **so that** the archive can keep growing without cluttering the one thing I use constantly to navigate my notes

### Acceptance Criteria

- **Scenario:** First move to `archive` in a workspace creates a Zed exclude entry
- **Given:** I am inside an initialized PARA system with no `.zed/settings.json`
- **When:** I run `tk move my-file archive`
- **Then:** Tick creates `.zed/settings.json` with `file_scan_exclude` containing the configured archive folder name (`4-Archive` by default)

- **Scenario:** First move to `archive` in a workspace creates a VS Code exclude entry
- **Given:** I am inside an initialized PARA system with no `.vscode/settings.json`
- **When:** I run `tk move my-file archive`
- **Then:** Tick creates `.vscode/settings.json` with both `files.exclude` and `search.exclude` mapping the configured archive folder name to `true`

- **Scenario:** Existing editor settings are preserved, not overwritten
- **Given:** I am inside an initialized PARA system with a `.zed/settings.json` that already sets `"tab_size": 4` and a `.vscode/settings.json` that already sets `"editor.fontSize": 14`
- **When:** I run `tk move my-file archive`
- **Then:** both files still contain their pre-existing settings unchanged, with the exclude entries merged in alongside them

- **Scenario:** Moving another item to `archive` doesn't duplicate the exclude entry
- **Given:** I am inside an initialized PARA system where a previous `tk move ... archive` run already added the exclude entries to both editors' settings
- **When:** I run `tk move <another-item> archive`
- **Then:** `file_scan_exclude` in `.zed/settings.json` still lists the archive folder name exactly once, and `files.exclude`/`search.exclude` in `.vscode/settings.json` still each have exactly one entry for it

- **Scenario:** A custom archive folder name from `.tick.toml` is what gets excluded, not the default
- **Given:** I am inside an initialized PARA system whose `.tick.toml` sets `[folders] archive = "9-Attic"`
- **When:** I run `tk move my-file archive`
- **Then:** the exclude entries written to `.zed/settings.json` and `.vscode/settings.json` name `9-Attic`, not `4-Archive`

- **Scenario:** Moving an item to a category other than `archive` doesn't touch editor exclude config
- **Given:** I am inside an initialized PARA system with no `.zed/settings.json` or `.vscode/settings.json`
- **When:** I run `tk move my-file project`
- **Then:** neither file is created — this affordance is specific to the `archive` destination

---

## User Story 005

- **Summary:** Moving an item to `archive` ensures a `CLAUDE.md` instruction tells agents to leave the archive alone unless asked
- **Status:** Not started
- **Depends on:** Story 001 (runs as part of the same command)

### Use Case

- **As a** Tick user who works in this PARA system alongside an AI agent
- **I want to** have my agent skip reading the archive by default
- **so that** the agent's context stays focused on what's active, the same way the archive already stays out of my own editor's way

### Acceptance Criteria

- **Scenario:** First move to `archive` in a workspace creates `CLAUDE.md` with the instruction
- **Given:** I am inside an initialized PARA system with no `CLAUDE.md`
- **When:** I run `tk move my-file archive`
- **Then:** Tick creates a `CLAUDE.md` at the workspace root containing an instruction not to read files under the configured archive folder (`4-Archive` by default) unless the user explicitly asks or there's a strong, specific reason to

- **Scenario:** An existing `CLAUDE.md` without the instruction gets it appended
- **Given:** I am inside an initialized PARA system with a `CLAUDE.md` that has unrelated content and no archive instruction
- **When:** I run `tk move my-file archive`
- **Then:** Tick appends the archive instruction as its own section, leaving the existing content unchanged above it

- **Scenario:** An existing `CLAUDE.md` that already has the instruction is left untouched
- **Given:** I am inside an initialized PARA system whose `CLAUDE.md` already contains the archive instruction
- **When:** I run `tk move my-file archive`
- **Then:** `CLAUDE.md` is not modified — no duplicate section is appended

- **Scenario:** A custom archive folder name is what the instruction names
- **Given:** I am inside an initialized PARA system whose `.tick.toml` sets `[folders] archive = "9-Attic"`, with no `CLAUDE.md` yet
- **When:** I run `tk move my-file archive`
- **Then:** the instruction Tick writes names `9-Attic`, not `4-Archive`

- **Scenario:** Moving an item to a category other than `archive` doesn't touch `CLAUDE.md`
- **Given:** I am inside an initialized PARA system with no `CLAUDE.md`
- **When:** I run `tk move my-file project`
- **Then:** no `CLAUDE.md` is created — this affordance is specific to the `archive` destination

---

## User Story 006

- **Summary:** Moving an item to `archive` stamps a one-line summary into its frontmatter before moving it
- **Status:** ✅
- **Depends on:** Story 001 (the move this happens alongside), [list.md](list.md) Story 005 (the Title-inference this reuses for the default)

### Use Case

- **As a** Tick user archiving a project, area, or note
- **I want to** leave behind a short summary of what it was
- **so that** an agent that does have a reason to look into the archive can get the gist from the listing/frontmatter alone, without reading the whole file

### Acceptance Criteria

- **Scenario:** Moving to `archive` prompts for a summary, defaulting to the item's inferred title
- **Given:** I am inside an initialized PARA system with a project `website-redesign` whose `index.md` has no `summary` frontmatter field and a first heading of `# Website Redesign`
- **When:** I run `tk move website-redesign archive`
- **Then:** Tick prompts `Summary for website-redesign?` with a default of `Website Redesign`
- **and Then:** if I accept the default, `index.md`'s frontmatter is stamped with `summary: Website Redesign` before the move

- **Scenario:** A custom summary overwrites the prompt's default
- **Given:** I am inside an initialized PARA system with a resource `my-file.md`
- **When:** I run `tk move my-file archive` and type `Old pricing notes, superseded by the 2026 plan` at the summary prompt
- **Then:** `my-file.md`'s frontmatter is stamped with `summary: Old pricing notes, superseded by the 2026 plan`, not the inferred title

- **Scenario:** An item that already has a `summary` field offers it as the default instead of the inferred title
- **Given:** I am inside an initialized PARA system with an area `health` whose `index.md` already has `summary: Fitness and nutrition tracking`
- **When:** I run `tk move health archive`
- **Then:** Tick prompts with a default of `Fitness and nutrition tracking`, not the inferred title

- **Scenario:** Stamping the summary preserves every other frontmatter field and the body
- **Given:** I am inside an initialized PARA system with a project `website-redesign` whose `index.md` has a `last_reviewed` field and body content
- **When:** I run `tk move website-redesign archive` and accept the default summary
- **Then:** `last_reviewed` and the body are unchanged in the moved `index.md` — only `summary` is added

- **Scenario:** Moving an item to a category other than `archive` doesn't prompt for a summary
- **Given:** I am inside an initialized PARA system with a resource `my-file.md`
- **When:** I run `tk move my-file project`
- **Then:** Tick doesn't prompt for a summary, and no `summary` frontmatter field is added — this affordance is specific to the `archive` destination
