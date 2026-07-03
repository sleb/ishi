# User Stories: `tk config`

## User Story 001

- **Summary:** See the config Tick is actually using, defaults and all

### Use Case

- **As a** Tick user who isn't sure whether a setting comes from `.tick.toml` or a built-in default
- **I want to** run `tk config` with no arguments
- **so that** I can see the full effective configuration in one place, without cross-referencing my `.tick.toml` against the docs

### Acceptance Criteria

- **Scenario:** No `.tick.toml` is present
- **Given:** I am inside a PARA system with no `.tick.toml` file
- **When:** I run `tk config`
- **Then:** Tick prints the built-in default config in `.tick.toml` (TOML) format, covering `folders`, `defaults`, and `templates`

- **Scenario:** A `.tick.toml` overrides some defaults
- **Given:** my `.tick.toml` overrides only the `folders.inbox` key
- **When:** I run `tk config`
- **Then:** Tick prints the full config with `folders.inbox` set to my override and every other key at its built-in default

---

## User Story 002

- **Summary:** Get a starting `.tick.toml` instead of writing one from scratch

### Use Case

- **As a** Tick user who wants to customize folder names, the default extension, or note templates
- **I want to** run `tk config init`
- **so that** I get a `.tick.toml` populated with the current defaults, ready to edit, instead of having to copy them from documentation by hand

### Acceptance Criteria

- **Scenario:** No config file exists yet
- **Given:** I am inside a PARA system with no `.tick.toml` file
- **When:** I run `tk config init`
- **Then:** Tick creates a `.tick.toml` containing the default `folders`, `defaults`, and `templates` tables
- **and Then:** Tick prints the path it created

---

## User Story 003

- **Summary:** Don't clobber my customized config by re-running init

### Use Case

- **As a** Tick user who already has a `.tick.toml` with my own customizations
- **I want to** be stopped if I accidentally run `tk config init` again
- **so that** I don't lose changes I've already made to my config

### Acceptance Criteria

- **Scenario:** A `.tick.toml` already exists
- **Given:** a `.tick.toml` file already exists in my PARA system
- **When:** I run `tk config init`
- **Then:** Tick prints an error explaining that `.tick.toml` already exists
- **and Then:** the existing file is left untouched

---

## User Story 004

- **Summary:** Jump straight into editing my config, no need to remember the filename

### Use Case

- **As a** Tick user who wants to tweak my templates or folder names
- **I want to** run `tk config edit`
- **so that** my `.tick.toml` opens directly in `$EDITOR` without me having to locate the file myself

### Acceptance Criteria

- **Scenario:** Editing an existing config
- **Given:** a `.tick.toml` file already exists in my PARA system
- **When:** I run `tk config edit`
- **Then:** Tick opens that file in `$EDITOR`

- **Scenario:** Editing when no config exists yet
- **Given:** I am inside a PARA system with no `.tick.toml` file
- **When:** I run `tk config edit`
- **Then:** Tick creates a `.tick.toml` populated with the defaults (as in `tk config init`) and then opens it in `$EDITOR`

---

## User Story 005

- **Summary:** Get autocomplete and validation for `.tick.toml` in my editor

### Use Case

- **As a** Tick user editing `.tick.toml` in an editor with TOML language support (e.g. VS Code with the Even Better TOML extension)
- **I want to** have my editor autocomplete config keys and flag typos or misplaced values as I type
- **so that** I don't have to consult the docs to remember key names like `templates.daily` or catch a mistyped key only when `tk` fails to parse the file later

### Acceptance Criteria

- **Scenario:** Generated config points to a schema
- **Given:** I run `tk config init` (or `tk config edit` when no config exists yet)
- **When:** Tick writes the new `.tick.toml`
- **Then:** the file's first line is a `#:schema` comment pointing to a JSON Schema file describing the `folders`, `defaults`, and `templates` tables
- **and Then:** a Taplo-aware editor (e.g. VS Code with Even Better TOML) uses that schema to offer autocomplete and inline validation for the file, with no extra setup from me

- **Scenario:** Schema file is available on disk
- **Given:** I run `tk config init`
- **When:** Tick writes `.tick.toml` and its `#:schema` comment
- **Then:** the JSON Schema file the comment points to also exists at that path, so the reference resolves without a network fetch

- **Scenario:** Existing config without a schema comment
- **Given:** I have a `.tick.toml` created before this feature existed, with no `#:schema` comment
- **When:** I run `tk config edit`
- **Then:** Tick opens the file as-is, without inserting a `#:schema` comment or otherwise modifying the file's contents
