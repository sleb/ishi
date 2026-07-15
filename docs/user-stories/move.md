# User Stories: `ishi move`

`mv` is an alias for `move`.

`ishi archive <item>` is sugar for `ishi move <item> archive` — a shorter, more
memorable way to file something away. Moving an item to `archive` also
stamps a one-line summary into the item being archived (Story 004), so an
agent that does have a reason to look can get the gist without reading
the whole note. This affordance lives on `ishi move` itself, specific to
the `archive` destination, rather than on the `ishi archive` alias — so `ishi
archive` gets it for free, without needing its own wiring beyond
delegating to `move`.

(Keeping the archive out of the way of the editor's fuzzy-find and of an
agent's context — via editor exclude config and a `CLAUDE.md` instruction
— is set up once, at `ishi init` time, instead of on every archiving move.
See [init.md](init.md) Stories 005-006.)

## User Story 001

- **Summary:** Moving an item relocates it to the target category, wrapping into a directory or preserving origin as needed
- **Status:** ✅
- **Depends on:** [new.md](new.md) Story 002, Story 003, Story 004 (items to move must exist first)

### Use Case

- **As a** Ishi user reorganizing my notes
- **I want to** move an item to a different category with one command
- **so that** I don't have to manually create directories, move files, and clean up after myself when an item's status changes

### Acceptance Criteria

- **Scenario:** Moving a flat file into `project` wraps it into a new directory with an `index.md`
- **Given:** `my-file.md` exists in `0-Inbox`
- **When:** I run `ishi move my-file project`
- **Then:** Ishi prints `Moved ./0-Inbox/my-file.md to ./1-Projects/my-file/index.md`
- **and Then:** `0-Inbox/my-file.md` no longer exists, and `1-Projects/my-file/index.md` has the same content `my-file.md` had

- **Scenario:** Moving a flat file into `area` wraps it the same way
- **Given:** `my-file.md` exists in `3-Resources`
- **When:** I run `ishi move my-file area`
- **Then:** Ishi prints `Moved ./3-Resources/my-file.md to ./2-Areas/my-file/index.md`

- **Scenario:** Moving a flat file into `inbox` or `resource` relocates it without wrapping
- **Given:** `my-file.md` exists in `3-Resources`
- **When:** I run `ishi move my-file inbox`
- **Then:** Ishi prints `Moved ./3-Resources/my-file.md to ./0-Inbox/my-file.md`
- **and Then:** no directory is created — the destination is still a flat file

- **Scenario:** Moving a project directory to `area` (or vice versa) relocates the directory as-is, without re-wrapping
- **Given:** `website-redesign` exists as a directory under `1-Projects`
- **When:** I run `ishi move website-redesign area`
- **Then:** Ishi prints `Moved ./1-Projects/website-redesign to ./2-Areas/website-redesign`
- **and Then:** `website-redesign/index.md` and every other file in the directory are unchanged, just relocated

- **Scenario:** Moving any category's item to `archive` preserves its origin category as a subfolder
- **Given:** `website-redesign` exists as a directory under `1-Projects`
- **When:** I run `ishi move website-redesign archive`
- **Then:** Ishi prints `Moved ./1-Projects/website-redesign to ./4-Archive/Projects/website-redesign`

- **Scenario:** Archiving a flat file also preserves its origin category as a subfolder
- **Given:** `my-file.md` exists in `3-Resources`
- **When:** I run `ishi move my-file archive`
- **Then:** Ishi prints `Moved ./3-Resources/my-file.md to ./4-Archive/Resources/my-file.md`

- **Scenario:** The `mv` alias behaves identically to `move`
- **Given:** `my-file.md` exists in `0-Inbox`
- **When:** I run `ishi mv my-file project`
- **Then:** Ishi prints `Moved ./0-Inbox/my-file.md to ./1-Projects/my-file/index.md`, exactly as `ishi move my-file project` would

---

## User Story 002

- **Summary:** Get a clear error instead of a silent guess when unwrapping a directory item isn't supported
- **Status:** ✅
- **Depends on:** [new.md](new.md) Story 003, Story 004 (project/area scaffolding — the directory items this rejection applies to)

### Use Case

- **As a** Ishi user who wants to move a `project` or `area` item back to `inbox` or `resource`
- **I want to** be told that unwrapping a directory into a flat file isn't supported
- **so that** Ishi never has to guess which file inside the directory becomes the flat file, and I don't lose the rest of the directory's contents silently

### Acceptance Criteria

- **Scenario:** Moving a project directory to `inbox` or `resource` is rejected
- **Given:** `<item>` exists as a directory under `1-Projects` or `2-Areas`
- **When:** I run `ishi move <item> inbox` or `ishi move <item> resource`
- **Then:** Ishi prints an error explaining that unwrapping a directory into a flat file is not yet supported
- **and Then:** no files or directories are moved, created, or modified

- **Scenario:** Moving an area directory to `inbox` or `resource` is rejected
- **Given:** `<item>` exists as a directory under `2-Areas`
- **When:** I run `ishi move <item> inbox` or `ishi move <item> resource`
- **Then:** Ishi prints an error explaining that unwrapping a directory into a flat file is not yet supported
- **and Then:** no files or directories are moved, created, or modified

---

## User Story 003

- **Summary:** `ishi archive <item>` is sugar for `ishi move <item> archive` — a shorter, more memorable way to file something away
- **Status:** ✅
- **Depends on:** Story 001 (the move semantics this delegates to)

### Use Case

- **As a** Ishi user who has decided a project, area, resource, or inbox note is done
- **I want to** run `ishi archive <item>` instead of `ishi move <item> archive`
- **so that** I don't have to remember or type the destination category name for the one destination this command only ever has

### Acceptance Criteria

- **Scenario:** Archiving a project files it under the Archive's Projects subfolder
- **Given:** I am inside an initialized PARA system with a project `website-redesign`
- **When:** I run `ishi archive website-redesign`
- **Then:** `website-redesign` is moved from `1-Projects/website-redesign` to `4-Archive/Projects/website-redesign`, exactly as `ishi move website-redesign archive` would move it
- **and Then:** Ishi prints the same `Moved ...` confirmation `ishi move` prints

- **Scenario:** Archiving an inbox note or resource files it under the matching subfolder
- **Given:** I am inside an initialized PARA system with a resource `my-file.md`
- **When:** I run `ishi archive my-file`
- **Then:** `my-file.md` is moved from `3-Resources/my-file.md` to `4-Archive/Resources/my-file.md`

- **Scenario:** `ishi archive` takes no destination argument
- **Given:** I am inside an initialized PARA system
- **When:** I run `ishi archive my-file archive`
- **Then:** Ishi rejects the command with an error — `ishi archive` doesn't accept a category argument, since the destination is always `archive`

- **Scenario:** Archiving a directory item that doesn't support unwrapping still only applies to the reverse direction
- **Given:** I am inside an initialized PARA system with a project `website-redesign`
- **When:** I run `ishi archive website-redesign`
- **Then:** the move succeeds — Story 002's rejection only applies to unwrapping a directory back into `inbox`/`resource`, never to archiving

---

## User Story 004

- **Summary:** Moving an item to `archive` stamps a one-line summary into its frontmatter before moving it
- **Status:** ✅
- **Depends on:** Story 001 (the move this happens alongside), [list.md](list.md) Story 005 (the Title-inference this reuses for the default)

### Use Case

- **As a** Ishi user archiving a project, area, or note
- **I want to** leave behind a short summary of what it was
- **so that** an agent that does have a reason to look into the archive can get the gist from the listing/frontmatter alone, without reading the whole file

### Acceptance Criteria

- **Scenario:** Moving to `archive` prompts for a summary, defaulting to the item's inferred title
- **Given:** I am inside an initialized PARA system with a project `website-redesign` whose `index.md` has no `summary` frontmatter field and a first heading of `# Website Redesign`
- **When:** I run `ishi move website-redesign archive`
- **Then:** Ishi prompts `Summary for website-redesign?` with a default of `Website Redesign`
- **and Then:** if I accept the default, `index.md`'s frontmatter is stamped with `summary: Website Redesign` before the move

- **Scenario:** A custom summary overwrites the prompt's default
- **Given:** I am inside an initialized PARA system with a resource `my-file.md`
- **When:** I run `ishi move my-file archive` and type `Old pricing notes, superseded by the 2026 plan` at the summary prompt
- **Then:** `my-file.md`'s frontmatter is stamped with `summary: Old pricing notes, superseded by the 2026 plan`, not the inferred title

<!--- **Scenario:** An item that already has a `summary` field offers it as the default instead of the inferred title-->
- **Given:** I am inside an initialized PARA system with an area `health` whose `index.md` already has `summary: Fitness and nutrition tracking`
- **When:** I run `ishi move health archive`
- **Then:** Ishi prompts with a default of `Fitness and nutrition tracking`, not the inferred title

- **Scenario:** Stamping the summary preserves every other frontmatter field and the body
- **Given:** I am inside an initialized PARA system with a project `website-redesign` whose `index.md` has a `last_reviewed` field and body content
- **When:** I run `ishi move website-redesign archive` and accept the default summary
- **Then:** `last_reviewed` and the body are unchanged in the moved `index.md` — only `summary` is added

- **Scenario:** Moving an item to a category other than `archive` doesn't prompt for a summary
- **Given:** I am inside an initialized PARA system with a resource `my-file.md`
- **When:** I run `ishi move my-file project`
- **Then:** Ishi doesn't prompt for a summary, and no `summary` frontmatter field is added — this affordance is specific to the `archive` destination

---

## User Story 005

- **Summary:** Un-archiving — moving an item back out of `Archive` into a live category, keyed off the target category rather than the origin subfolder
- **Status:** ✅
- **Depends on:** Story 001 (the move semantics this extends), [list.md](list.md) (the `<OriginCategory>/<name>` composite form used to name archived items)

### Use Case

- **As a** Ishi user who archived something and now needs it back
- **I want to** move an item out of `Archive` with the same `ishi move`/`ishi mv` command I'd use for any other relocation
- **so that** un-archiving isn't a special, separate command, and the item lands in whatever shape the target category expects, regardless of which category it was originally archived from

### Acceptance Criteria

- **Scenario:** A bare name never matches an archived item
- **Given:** `website-redesign` exists only as a directory under `4-Archive/Projects`
- **When:** I run `ishi move website-redesign project`
- **Then:** Ishi reports no item named `website-redesign` was found — `Archive` items are only addressable by their qualified `<OriginCategory>/<name>` form, since basenames aren't unique across origin subfolders

- **Scenario:** Un-archiving a directory item back into a directory-style category relocates it as-is
- **Given:** `website-redesign` exists as a directory under `4-Archive/Projects`
- **When:** I run `ishi move Projects/website-redesign project`
- **Then:** Ishi prints `Moved ./4-Archive/Projects/website-redesign to ./1-Projects/website-redesign`
- **and Then:** `index.md` and every other file in the directory are unchanged, just relocated

- **Scenario:** Un-archiving a directory item into a different directory-style category also relocates it as-is
- **Given:** `website-redesign` exists as a directory under `4-Archive/Projects`
- **When:** I run `ishi move Projects/website-redesign area`
- **Then:** Ishi prints `Moved ./4-Archive/Projects/website-redesign to ./2-Areas/website-redesign`

- **Scenario:** Un-archiving a flat file back into a flat-file category relocates it as-is
- **Given:** `my-file.md` exists under `4-Archive/Resources`
- **When:** I run `ishi move Resources/my-file inbox`
- **Then:** Ishi prints `Moved ./4-Archive/Resources/my-file.md to ./0-Inbox/my-file.md`

- **Scenario:** Un-archiving a flat file into a directory-style category wraps it into a new directory
- **Given:** `my-note.md` exists under `4-Archive/Inbox`
- **When:** I run `ishi move Inbox/my-note project`
- **Then:** Ishi prints `Moved ./4-Archive/Inbox/my-note.md to ./1-Projects/my-note/index.md`
- **and Then:** `my-note.md`'s content becomes the new `index.md`'s content unchanged

- **Scenario:** Un-archiving a directory item into a flat-file category is rejected, same as Story 002
- **Given:** `website-redesign` exists as a directory under `4-Archive/Projects`
- **When:** I run `ishi move Projects/website-redesign inbox` or `ishi move Projects/website-redesign resource`
- **Then:** Ishi prints an error explaining that unwrapping a directory into a flat file is not yet supported
- **and Then:** no files or directories are moved, created, or modified

- **Scenario:** Moving an already-archived item to `archive` again is rejected
- **Given:** `my-file.md` exists under `4-Archive/Resources`
- **When:** I run `ishi move Resources/my-file archive`
- **Then:** Ishi prints an error explaining that `my-file` is already archived
- **and Then:** no files or directories are moved, created, or modified

---

## User Story 006

- **Summary:** A bare name that matches more than one item is rejected, not silently resolved to whichever category happens to be checked first — the same `<Category>/<name>` qualified form Story 005 uses for archived items disambiguates live items too
- **Status:** ✅
- **Depends on:** Story 001 (the move semantics this disambiguation guards), Story 005 (the `<OriginCategory>/<name>` qualified form this extends to every category, not just `Archive`)

### Use Case

- **As a** Ishi user with two items that happen to share a basename (e.g. `meeting-notes.md` in both `0-Inbox` and `3-Resources`)
- **I want to** get an error telling me the name is ambiguous, instead of `ishi` silently picking one of them
- **so that** I never move — or accidentally overwrite context on — the wrong item just because Ishi guessed for me

### Acceptance Criteria

- **Scenario:** A bare name matching items in two different live categories is rejected
- **Given:** `meeting-notes.md` exists in both `0-Inbox` and `3-Resources`
- **When:** I run `ishi move meeting-notes archive`
- **Then:** Ishi prints an error naming both candidates (e.g. `"meeting-notes" is ambiguous — found in inbox, resources`) and does not move anything
- **and Then:** no files or directories are moved, created, or modified

- **Scenario:** Qualifying the name with its category resolves the ambiguity
- **Given:** `meeting-notes.md` exists in both `0-Inbox` and `3-Resources`
- **When:** I run `ishi move resources/meeting-notes archive`
- **Then:** Ishi moves `3-Resources/meeting-notes.md` (and only that file) to `4-Archive/Resources/meeting-notes.md`

- **Scenario:** The qualified form works for any live category, not just `Archive`'s origin subfolders
- **Given:** `my-file.md` exists in `0-Inbox`
- **When:** I run `ishi move inbox/my-file project`
- **Then:** Ishi moves it exactly as `ishi move my-file project` would if `my-file` were unambiguous
- **and Then:** `ishi move project/my-file ...` (any category prefix that doesn't match `my-file`'s actual location) reports no item found, rather than silently falling back to a bare-name search

- **Scenario:** A bare name that matches nothing anywhere still gets Ishi's existing "not found" error
- **Given:** no item named `nonexistent` exists in any category
- **When:** I run `ishi move nonexistent project`
- **Then:** Ishi prints `No item named "nonexistent" found`, exactly as before — this story only changes behavior when a name matches more than one item

---

## User Story 007

- **Summary:** `ishi unarchive` accepts a bare (unqualified) name when it unambiguously matches one archived item, instead of always requiring the full `<OriginCategory>/<name>` form
- **Status:** ✅
- **Depends on:** Story 005 (the qualified `<OriginCategory>/<name>` form this adds a shortcut alongside, without replacing it)

### Use Case

- **As a** Ishi user (or agent) restoring an archived item
- **I want to** run `ishi unarchive <name>` with the same bare name `ishi list archive` shows, without having to prefix it with the category it happened to be archived from
- **so that** `unarchive` behaves consistently with `move`/`archive`/`review`, which all accept a bare name when it's unambiguous — the qualified form should be there for disambiguation, not required by default

### Acceptance Criteria

- **Scenario:** A bare name that matches exactly one archived item resolves without qualification
- **Given:** `apollo` exists as a directory under `4-Archive/Projects` and nowhere else in `Archive`
- **When:** I run `ishi unarchive apollo`
- **Then:** Ishi restores it exactly as `ishi unarchive Projects/apollo` would, printing `Moved archive/Projects/apollo to projects/apollo`

- **Scenario:** A bare name that matches archived items from two different origins is rejected as ambiguous
- **Given:** an item named `apollo` exists as a directory under both `4-Archive/Projects` and `4-Archive/Areas`
- **When:** I run `ishi unarchive apollo`
- **Then:** Ishi prints an error naming both candidates (e.g. `"apollo" is ambiguous — found in Projects, Areas`) and does not move anything

- **Scenario:** Qualifying the name still resolves the ambiguity, exactly as before
- **Given:** an item named `apollo` exists as a directory under both `4-Archive/Projects` and `4-Archive/Areas`
- **When:** I run `ishi unarchive Projects/apollo`
- **Then:** Ishi restores `4-Archive/Projects/apollo` (and only that one) to `1-Projects/apollo`

- **Scenario:** A bare name matching a live (non-archived) item is still rejected as not archived, not silently treated as a typo for the archived form
- **Given:** `my-file.md` exists in `0-Inbox` and nothing named `my-file` exists in `Archive`
- **When:** I run `ishi unarchive my-file`
- **Then:** Ishi prints an error that `my-file` is not archived — this story only adds a bare-name lookup *inside* `Archive`, it doesn't change what counts as archived

- **Scenario:** A bare name that matches nothing anywhere still gets Ishi's existing "not found" error
- **Given:** no item named `nonexistent` exists anywhere, archived or otherwise
- **When:** I run `ishi unarchive nonexistent`
- **Then:** Ishi prints `No item named "nonexistent" found`, exactly as before
