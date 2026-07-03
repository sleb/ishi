# Tick

Tick is a command-line tool for managing a [PARA](https://fortelabs.com/para) system — a simple method for organizing your notes and files into four categories:

| Category      | Purpose                                              |
| ------------- | ---------------------------------------------------- |
| **P**rojects  | Short-term efforts with a specific goal and deadline |
| **A**reas     | Ongoing responsibilities with a standard to maintain |
| **R**esources | Topics or themes of ongoing interest                 |
| **A**rchive   | Inactive items from the other three categories       |

> PARA was created by Tiago Forte. See his [original post](https://fortelabs.com/para) for background on the method itself.

Tick manages the directory structure and file bookkeeping so you can focus on capturing and organizing your notes:

```
.
├── 0-Inbox
├── 1-Projects
│   └── website-redesign
│       └── index.md
├── 2-Areas
│   └── health
│       └── index.md
├── 3-Resources
└── 4-Archive
```

Projects and areas are directories, not single files — real projects accumulate drafts, attachments, and other supporting material, and `index.md` is the entry point Tick reads for a project or area's title and status. Resources and inbox captures are usually single notes, so they stay flat files.

## Installation

```
cargo install tick
```

Or build from source:

```
git clone https://github.com/sleb/tick.git
cd tick
cargo install --path .
```

This installs a `tk` binary — the crate is published as `tick`, but the command stays short.

## Quick start

```
$ tk init my-para
Created PARA system in ./my-para

$ cd my-para
$ tk new meeting-notes
Created ./0-Inbox/meeting-notes.md

$ tk new --project website-redesign
Created ./1-Projects/website-redesign/index.md

$ tk status
Inbox      1
Projects   1
Areas      0
Resources  0
Archive    0
```

## Commands

| Command                                          | Description                                                                                                                                                                                                                                            |
| ------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `init [name]`                                    | Initialize a new PARA system                                                                                                                                                                                                                           |
| `new [filename] [--project\|--area\|--resource]` | Capture a new note. Defaults to the Inbox; pass `--project` or `--area` to scaffold a directory with an `index.md`, or `--resource` for a flat file. Omit `filename` to capture in `$EDITOR`, which will suggest a name for you to confirm or override |
| `daily`                                          | Create (or open) today's daily note in the Inbox                                                                                                                                                                                                       |
| `mv <item> <category>`                           | Move a file or project/area directory to `inbox`, `project`, `area`, `resource`, or `archive`. Archiving preserves which category the item came from                                                                                                   |
| `list <category> [filter]`                       | List items in a category (`inbox`, `project`, `area`, `resource`, or `archive`), optionally filtered by name                                                                                                                                           |
| `status`                                         | Show item counts per category and flag stale projects/areas                                                                                                                                                                                            |
| `review`                                         | Walk through projects and areas one by one for a weekly review                                                                                                                                                                                         |
| `config [init\|edit]`                            | View, initialize, or edit the `.tick.toml` config                                                                                                                                                                                                      |
| `completions <shell>`                            | Generate a shell completion script                                                                                                                                                                                                                     |

Files created without an extension default to `.md`.

### `init`

```
tk init [name]
```

Initializes a new PARA system in the current directory, or in `./<name>` if given.

```
$ tk init my-para
Created PARA system in ./my-para

$ ls my-para
0-Inbox  1-Projects  2-Areas  3-Resources  4-Archive
```

### `new`

```
tk new [filename] [--project | --area | --resource]
```

Creates a new note. With no arguments, opens `$EDITOR` on a scratch file, then suggests a filename from the first line you wrote (or a timestamp, if the file is empty) and prompts you to confirm it or type a different one before creating it in the Inbox. With a `filename`, creates it directly — in the Inbox by default, or under `--project`, `--area`, or `--resource` if given.

For `--project` and `--area`, this scaffolds a directory named after `filename` containing an `index.md`, so the project can grow to hold other files. For `--resource` (and the Inbox), it's a single flat file.

```
$ tk new
Opening $EDITOR...
Create "website-improvement-ideas.md"?
Created ./0-Inbox/website-improvement-ideas.md

$ tk new my-file
Created ./0-Inbox/my-file.md

$ tk new --project my-project
Created ./1-Projects/my-project/index.md
```

### `daily`

```
tk daily
```

Creates (or opens) today's daily note in the Inbox, named for the current date.

```
$ tk daily
Created ./0-Inbox/2026-06-30.md
```

### `mv`

```
tk mv <item> <inbox|project|area|resource|archive>
```

Moves an existing file or project/area directory to the given category. Moving a flat file into `project` or `area` wraps it into a new directory with an `index.md`; moving to `archive` preserves which category the item came from, filing it under a matching subfolder.

```
$ tk mv my-file.md project
Moved ./0-Inbox/my-file.md to ./1-Projects/my-file/index.md

$ tk mv my-project archive
Moved ./1-Projects/my-project to ./4-Archive/Projects/my-project
```

### `list`

```
tk list <inbox|project|area|resource|archive> [filter]
```

Lists items in a category, optionally filtered to names containing `filter`. For `project` and `area`, this lists the item directories (not the `index.md` files inside them); for `resource`, `inbox`, and `archive`, it lists flat files.

```
$ tk list project
./1-Projects/my-project
./1-Projects/website-redesign

$ tk list project website
./1-Projects/website-redesign
```

### `status`

```
tk status
```

Shows how many items are in each category, and flags projects or areas whose `index.md` hasn't been touched in a while.

```
$ tk status
Inbox      2
Projects   3 (1 stale)
Areas      2
Resources  5
Archive    12
```

### `review`

```
tk review
```

Walks through each project and area one at a time (by its `index.md`), prompting you to keep, update, or archive it — a guided version of PARA's weekly review ritual.

```
$ tk review
Project: website-redesign (last updated 12 days ago)
  [k]eep  [a]rchive  [s]kip?
```

### `config`

```
tk config [init | edit]
```

With no arguments, prints the effective config (defaults merged with any `.tick.toml` overrides). `tk config init` writes a `.tick.toml` populated with the [defaults](#configuration), ready to customize. `tk config edit` opens it in `$EDITOR`.

```
$ tk config init
Created ./.tick.toml

$ tk config
[folders]
inbox = "0-Inbox"
projects = "1-Projects"
areas = "2-Areas"
resources = "3-Resources"
archive = "4-Archive"

[defaults]
extension = "md"

[templates]
note = "..."
daily = "..."
project = "..."
area = "..."
resource = "..."
```

### `completions`

```
tk completions <bash|zsh|fish|powershell>
```

Prints a shell completion script to stdout.

```
$ tk completions zsh > ~/.zsh/completions/_tk
```

## Configuration

Tick reads an optional `.tick.toml` from the root of your PARA system. It lets you rename the numbered folders, change the default file extension, and customize the templates used for new notes instead of relying on the built-in defaults.

```toml
[folders]
inbox = "0-Inbox"
projects = "1-Projects"
areas = "2-Areas"
resources = "3-Resources"
archive = "4-Archive"

[defaults]
extension = "md"

[templates]
note = """
---
last_updated: {{date}}
---
# {{title}}
"""

daily = """
---
date: {{date}}
last_updated: {{date}}
---
# {{date}}

## Tasks

[ ] -

## Notes

"""

project = """
---
last_updated: {{date}}
---

# {{title}}

Status: active
"""

area = """
---
last_updated: {{date}}
---

# {{title}}

Standard:
"""

resource = """
---
last_updated: {{date}}
---

# {{title}}
"""
```

Every category has a template: `note` is used for Inbox captures and `--resource` notes, `daily` for `tk daily`, and `project`/`area` for the `index.md` scaffolded by `tk new --project`/`--area`. Templates are plain text with `{{title}}` and `{{date}}` placeholders, filled in when the note is created. `tk config init` writes the defaults above as a starting point.

### Schema and autocomplete

`tk config init` (and `tk config edit`, if no config exists yet) writes a [`#:schema`](https://taplo.tamasfe.dev/configuration/directives.html) directive as the first line of `.tick.toml`, pointing at a JSON Schema that describes the `folders`, `defaults`, and `templates` keys:

```toml
#:schema ./.tick.schema.json

[folders]
inbox = "0-Inbox"
...
```

Editors with Taplo-based TOML support — notably VS Code's [Even Better TOML](https://marketplace.visualstudio.com/items?itemName=tamasfe.even-better-toml) extension — read that directive to offer autocomplete and inline validation for `.tick.toml`, with no extra setup. Tick writes the referenced schema file alongside `.tick.toml` so the reference resolves locally, without a network fetch. This is a Taplo-specific convention, not a universal TOML standard, so editors without Taplo support won't do anything with the directive beyond treating it as a comment.
