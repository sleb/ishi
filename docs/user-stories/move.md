# User Stories: `tk move`

`mv` is an alias for `move`.

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
- **Status:** Not started
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
