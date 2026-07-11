# User Stories: `tk move`

`mv` is an alias for `move`.

`tk archive <item>` is sugar for `tk move <item> archive` — a shorter, more
memorable way to file something away. Moving an item to `archive` also
stamps a one-line summary into the item being archived (Story 004), so an
agent that does have a reason to look can get the gist without reading
the whole note. This affordance lives on `tk move` itself, specific to
the `archive` destination, rather than on the `tk archive` alias — so `tk
archive` gets it for free, without needing its own wiring beyond
delegating to `move`.

(Keeping the archive out of the way of the editor's fuzzy-find and of an
agent's context — via editor exclude config and a `CLAUDE.md` instruction
— is set up once, at `tk init` time, instead of on every archiving move.
See [init.md](init.md) Stories 005-006.)

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
- **Status:** ✅
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

<!--- **Scenario:** An item that already has a `summary` field offers it as the default instead of the inferred title-->
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

---

## User Story 005

- **Summary:** Un-archiving — moving an item back out of `Archive` into a live category, keyed off the target category rather than the origin subfolder
- **Status:** ✅
- **Depends on:** Story 001 (the move semantics this extends), [list.md](list.md) (the `<OriginCategory>/<name>` composite form used to name archived items)

### Use Case

- **As a** Tick user who archived something and now needs it back
- **I want to** move an item out of `Archive` with the same `tk move`/`tk mv` command I'd use for any other relocation
- **so that** un-archiving isn't a special, separate command, and the item lands in whatever shape the target category expects, regardless of which category it was originally archived from

### Acceptance Criteria

- **Scenario:** A bare name never matches an archived item
- **Given:** `website-redesign` exists only as a directory under `4-Archive/Projects`
- **When:** I run `tk move website-redesign project`
- **Then:** Tick reports no item named `website-redesign` was found — `Archive` items are only addressable by their qualified `<OriginCategory>/<name>` form, since basenames aren't unique across origin subfolders

- **Scenario:** Un-archiving a directory item back into a directory-style category relocates it as-is
- **Given:** `website-redesign` exists as a directory under `4-Archive/Projects`
- **When:** I run `tk move Projects/website-redesign project`
- **Then:** Tick prints `Moved ./4-Archive/Projects/website-redesign to ./1-Projects/website-redesign`
- **and Then:** `index.md` and every other file in the directory are unchanged, just relocated

- **Scenario:** Un-archiving a directory item into a different directory-style category also relocates it as-is
- **Given:** `website-redesign` exists as a directory under `4-Archive/Projects`
- **When:** I run `tk move Projects/website-redesign area`
- **Then:** Tick prints `Moved ./4-Archive/Projects/website-redesign to ./2-Areas/website-redesign`

- **Scenario:** Un-archiving a flat file back into a flat-file category relocates it as-is
- **Given:** `my-file.md` exists under `4-Archive/Resources`
- **When:** I run `tk move Resources/my-file inbox`
- **Then:** Tick prints `Moved ./4-Archive/Resources/my-file.md to ./0-Inbox/my-file.md`

- **Scenario:** Un-archiving a flat file into a directory-style category wraps it into a new directory
- **Given:** `my-note.md` exists under `4-Archive/Inbox`
- **When:** I run `tk move Inbox/my-note project`
- **Then:** Tick prints `Moved ./4-Archive/Inbox/my-note.md to ./1-Projects/my-note/index.md`
- **and Then:** `my-note.md`'s content becomes the new `index.md`'s content unchanged

- **Scenario:** Un-archiving a directory item into a flat-file category is rejected, same as Story 002
- **Given:** `website-redesign` exists as a directory under `4-Archive/Projects`
- **When:** I run `tk move Projects/website-redesign inbox` or `tk move Projects/website-redesign resource`
- **Then:** Tick prints an error explaining that unwrapping a directory into a flat file is not yet supported
- **and Then:** no files or directories are moved, created, or modified

- **Scenario:** Moving an already-archived item to `archive` again is rejected
- **Given:** `my-file.md` exists under `4-Archive/Resources`
- **When:** I run `tk move Resources/my-file archive`
- **Then:** Tick prints an error explaining that `my-file` is already archived
- **and Then:** no files or directories are moved, created, or modified
