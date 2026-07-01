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

| Command                     | Description                                                                                                                                                                                                        |
| --------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `init [name]`               | Initialize a new PARA system                                                                                                                                                                                       |
| `new [filename] [--project\|--area\|--resource]` | Capture a new note. Defaults to the Inbox; pass `--project` or `--area` to scaffold a directory with an `index.md`, or `--resource` for a flat file. Omit `filename` to capture in `$EDITOR`, which will suggest a name for you to confirm or override |
| `daily`                     | Create (or open) today's daily note in the Inbox                                                                                                                                                                   |
| `mv <item> <category>`      | Move a file or project/area directory to `inbox`, `project`, `area`, `resource`, or `archive`. Archiving preserves which category the item came from                                                               |
| `list <category> [filter]`  | List items in a category (`inbox`, `project`, `area`, `resource`, or `archive`), optionally filtered by name                                                                                                       |
| `status`                    | Show item counts per category and flag stale projects/areas                                                                                                                                                        |
| `review`                    | Walk through projects and areas one by one for a weekly review                                                                                                                                                     |
| `completions <shell>`       | Generate a shell completion script                                                                                                                                                                                 |

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

### `completions`

```
tk completions <bash|zsh|fish|powershell>
```

Prints a shell completion script to stdout.

```
$ tk completions zsh > ~/.zsh/completions/_tk
```

## Configuration

Tick reads an optional `.tick.toml` from the root of your PARA system, letting you rename the numbered folders or change the default file extension instead of relying on the `0-Inbox`, `1-Projects`, etc. defaults.
