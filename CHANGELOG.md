# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-07-11

### Added

- `ishi init [name]` to scaffold a new PARA system (Projects/Areas/Resources/Archive).
- `ishi new [filename] [--project|--area|--resource|--daily]` to capture a new note, either directly or via an interactive `$EDITOR` capture that infers a filename from what you wrote.
- `ishi daily` to create or open today's daily note in the Inbox.
- `ishi move`/`mv <item> <category>` to move a file or project/area directory between categories, including archiving and un-archiving.
- `ishi archive <item>` and `ishi unarchive <OriginCategory>/<name>` as sugar for `move`, with archive stamping a summary into the item's frontmatter.
- `ishi list <category> [filter]` to list items in a category with inferred title and last-modified time.
- `ishi status` to show item counts per category, plus last-updated/last-reviewed facts for projects and areas.
- `ishi review` to walk through projects and areas for a guided weekly review, stamping `last_reviewed` on kept items.
- `ishi config [init|edit] [-g|--global]` for layered configuration (built-in defaults, `~/.ishi.toml`, `./.ishi.toml`) covering folder names, default file extension, and note templates.
- `ishi completions <bash|zsh|fish|powershell>` for shell tab-completion, including item-name completion for `move`, `archive`, and `unarchive`.
- Template placeholders `{{date}}`, `{{time}}`, `{{title}}`, `{{cursor}}`, and `{{uuid}}` for customizing note templates.
- JSON Schema generation for `.ishi.toml` to enable editor autocomplete and validation via Taplo-based tooling.

[0.1.0]: https://github.com/sleb/ishi/releases/tag/v0.1.0
