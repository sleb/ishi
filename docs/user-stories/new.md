# User Stories: `tk new`

## User Story 001

- **Summary:** Capture a quick thought without leaving the terminal or naming a file
- **Status:** Completed

### Use Case

- **As a** Tick user with a fleeting idea
- **I want to** run `tk new` with no arguments and write directly in my editor
- **so that** I can capture the thought immediately without deciding on a filename or category first

### Acceptance Criteria

- **Scenario:** Accept the inferred filename
- **Given:** I am inside an initialized PARA system
- **and Given:** my `$EDITOR` environment variable is set
- **When:** I run `tk new` with no arguments, write a note whose first line is `# Website Improvement Ideas`, save, and exit the editor
- **Then:** Tick prompts `Create "website-improvement-ideas.md"?` with the inferred name pre-filled
- **and Then:** if I accept the prompt, the file is created in `0-Inbox` under that name and Tick prints the path it created

- **Scenario:** Override the inferred filename
- **Given:** I am inside an initialized PARA system
- **and Given:** my `$EDITOR` environment variable is set
- **When:** I run `tk new` with no arguments, write content, save, exit the editor, and am shown the inferred filename prompt
- **Then:** I can type a different filename instead of accepting the suggestion
- **and Then:** the file is created in `0-Inbox` under the name I typed, and Tick prints the path it created

- **Scenario:** Empty note falls back to a timestamp
- **Given:** I am inside an initialized PARA system
- **and Given:** my `$EDITOR` environment variable is set
- **When:** I run `tk new` with no arguments, save an empty file, and exit the editor
- **Then:** Tick prompts with a filename generated from the current timestamp instead of a note title

- **Scenario:** Note with a blank first line falls back to a timestamp
- **Given:** I am inside an initialized PARA system
- **and Given:** my `$EDITOR` environment variable is set
- **When:** I run `tk new` with no arguments, write a note whose first line is blank (with content on a later line), save, and exit the editor
- **Then:** Tick prompts with a filename generated from the current timestamp instead of a note title, since only the first line is used to infer a title

---

## User Story 002

- **Summary:** Drop a named note straight into the Inbox
- **Status:** Completed

### Use Case

- **As a** Tick user who already knows what to call a note
- **I want to** run `tk new <filename>` and skip the editor prompt
- **so that** I can create the file directly without an extra confirmation step

### Acceptance Criteria

- **Scenario:** Create a named file in the Inbox
- **Given:** I am inside an initialized PARA system
- **When:** I run `tk new my-file`
- **Then:** a file named `my-file.md` is created in `0-Inbox` and Tick prints the path it created

---

## User Story 003

- **Summary:** Scaffold a new project as soon as it starts

### Use Case

- **As a** Tick user starting a new short-term effort
- **I want to** run `tk new --project <filename>`
- **so that** I get a directory ready to hold drafts and attachments, not just a single file

### Acceptance Criteria

- **Scenario:** Create a new project directory
- **Given:** I am inside an initialized PARA system
- **When:** I run `tk new --project website-redesign`
- **Then:** a directory `1-Projects/website-redesign` is created containing an `index.md`, and Tick prints the path to that `index.md`

---

## User Story 004

- **Summary:** Scaffold a new area to track an ongoing responsibility

### Use Case

- **As a** Tick user taking on an ongoing responsibility
- **I want to** run `tk new --area <filename>`
- **so that** I get a directory to hold everything related to maintaining that responsibility over time

### Acceptance Criteria

- **Scenario:** Create a new area directory
- **Given:** I am inside an initialized PARA system
- **When:** I run `tk new --area health`
- **Then:** a directory `2-Areas/health` is created containing an `index.md`, and Tick prints the path to that `index.md`

---

## User Story 005

- **Summary:** File a reference note without the overhead of a directory

### Use Case

- **As a** Tick user saving a topic of ongoing interest
- **I want to** run `tk new --resource <filename>`
- **so that** the note is filed as a single flat file, since it won't accumulate supporting material like a project or area would

### Acceptance Criteria

- **Scenario:** Create a new resource file
- **Given:** I am inside an initialized PARA system
- **When:** I run `tk new --resource recipe-ideas`
- **Then:** a file named `recipe-ideas.md` is created in `3-Resources` and Tick prints the path it created

---

## User Story 006

- **Summary:** Never have to type the file extension

### Use Case

- **As a** Tick user creating notes throughout the day
- **I want to** name files without specifying an extension
- **so that** I don't have to remember or type `.md` every time

### Acceptance Criteria

- **Scenario:** Filename given without an extension
- **Given:** I am inside an initialized PARA system
- **When:** I run `tk new my-file` (or any `tk new` variant) with a filename that has no extension
- **Then:** the created file has `.md` appended automatically
