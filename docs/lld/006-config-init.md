# LLD: `tk config init` — Stories 003–004 (`config.md`)

Source: [docs/user-stories/config.md](../user-stories/config.md) Stories
003–004. Module boundaries follow [docs/design.md](../design.md).
Corresponds to roadmap item 8 (`tk config` CLI surface) — this LLD covers
only the `init` slice of that item, not the whole thing.

## Scope

1. `tk config init` writes `./.tick.toml` populated with the built-in
   default `folders`/`defaults`/`templates` tables and prints the path it
   created (config.md 003, scenario 1).
2. `tk config init -g`/`--global` writes `~/.tick.toml` instead, same
   contents, same "prints the path" behavior (config.md 003, scenario 2).
3. `tk config init` refuses to run if `./.tick.toml` already exists,
   printing an error and leaving the file untouched (config.md 004,
   scenario 1).
4. `tk config init -g` refuses to run if `~/.tick.toml` already exists,
   same error shape, same untouched guarantee (config.md 004, scenario 2).
5. `tk config init -g` succeeds even when a *local* `.tick.toml` already
   exists — only the `-g` target is checked (config.md 004, scenario 3).

### Out of scope

- Bare `tk config` (print the effective config with per-key provenance
  comments) — config.md 001. Despite the ✅ on that story in
  `config.md`, no `Commands::Config` variant exists in `main.rs` yet
  (confirmed by reading the current source); the checkmark tracks the
  resolution/provenance logic (`Config::resolve`/`ConfigOrigins`, done in
  `005-config-layering.md`), not a CLI surface. Left for a follow-up LLD
  so this one stays focused on `init`.
- `tk config edit` (config.md 005) — separate LLD; `edit` needs the same
  "create defaults if missing" fallback this LLD builds, but opening
  `$EDITOR` is unrelated scope.
- The `#:schema` JSON Schema file and its directive comment (config.md
  006) — explicitly deferred in `005-config-layering.md`'s "Out of
  scope" for the same reason, still deferred here. `default_toml` (below)
  does not emit a `#:schema` line.
- Any change to `Config::resolve`, `ConfigOrigins`, or `Workspace::discover`
  — this LLD only adds a *writer*, reusing `Config::default()` as-is.

## `design.md` changes

Not yet applied to `docs/design.md`; deferred until this LLD lands. Once
implemented, the `config` section gains:

- `ConfigError` gains `AlreadyExists { path }` and `Write { path, source }`
  variants.
- `config::default_toml() -> String` and `config::init(path: &Path) ->
  Result<(), ConfigError>`.

The `cli` section gains `run_config_init`, and the command table's `config`
row notes `init`/`-g` are implemented (`edit` still isn't).

## Module designs

### `config` (extends existing module)

```rust
#[derive(Debug, Error)]
pub enum ConfigError {
    // ...existing Read/Parse variants, unchanged...

    #[error("{path} already exists")]
    AlreadyExists { path: String },

    #[error("failed to write {path}")]
    Write {
        path: String,
        #[source]
        source: std::io::Error,
    },
}

/// Renders `Config::default()` as the exact `.tick.toml` shape documented
/// in README.md's Configuration section — nested `[folders]`/`[defaults]`/
/// `[templates]` tables, no `#:schema` line (that's config.md 006, not
/// this story). Pure — no filesystem access, so it's unit-testable by
/// round-tripping through `toml::from_str`/`Config::resolve` without a
/// tempdir.
pub fn default_toml() -> String;

/// Writes `default_toml()` to `path`. Errors with `AlreadyExists` (and
/// leaves `path` untouched) if a file is already there — the guard
/// config.md 004 requires — rather than overwriting a user's
/// customizations.
pub fn init(path: &Path) -> Result<(), ConfigError> {
    if path.exists() {
        return Err(ConfigError::AlreadyExists {
            path: path.display().to_string(),
        });
    }
    fs::write(path, default_toml()).map_err(|source| ConfigError::Write {
        path: path.display().to_string(),
        source,
    })
}
```

`default_toml` takes no `Config` parameter — story 003 only ever scaffolds
the *built-in* defaults (there's no "customize, then re-derive a template"
use case), so hardcoding `Config::default()` internally keeps the function
argument-free rather than accepting a `Config` no caller would ever pass
anything but the default into.

`init` is a single function, not a separate `check_collision` +
`init` pair the way `workspace` splits them. `workspace::check_collision`
is shared because `workspace::init` has two callers with different collision
semantics to reconcile (bare cwd vs. named subdir, both always directories).
Here there's exactly one existence check, and both callers (`local`,
`-g`) already get their guard "for free" by calling `config::init` with a
different path — no second call site needs the check in isolation.

### `cli` (extends existing module)

```rust
/// Writes the default config to `path` and returns the exact confirmation
/// message `main` prints. `display` is the caller-computed human-readable
/// form (`"./.tick.toml"` or `"~/.tick.toml"`) — mirrors `run_init`'s
/// `(target, display)` split, so `cli` still never guesses `~` expansion
/// or relative-path rendering itself.
pub fn run_config_init(path: &Path, display: &str) -> anyhow::Result<String> {
    config::init(path)?;
    Ok(format!("Created {display}"))
}
```

`config::ConfigError::AlreadyExists`'s `#[error("{path} already exists")]`
message (using the *same* absolute/display path passed to `config::init`)
is what surfaces for config.md 004 — `run_config_init` doesn't need its own
error variant, it just propagates `ConfigError` through `anyhow`.

### `main` (extends existing `Commands` enum)

```rust
enum Commands {
    // ...existing New/Daily/Init...

    /// View or manage the effective config.
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
}

#[derive(Debug, PartialEq, Subcommand)]
enum ConfigAction {
    /// Write a new `.tick.toml` (or `~/.tick.toml` with `-g`) populated
    /// with the built-in defaults.
    Init {
        #[arg(short = 'g', long = "global")]
        global: bool,
    },
}
```

`action` is *not* `Option<ConfigAction>` — bare `tk config` (no
subcommand) is out of scope (see above), so `clap` reports its own
"a subcommand is required" error for now rather than this LLD inventing
placeholder behavior for a story it doesn't implement. The next LLD that
adds bare `tk config` or `tk config edit` will widen `ConfigAction` (adding
`Edit { global: bool }`) and/or make `action` optional — noted here as
expected call-site fallout, not implemented now.

Dispatch, mirroring how `home_config` is already computed once in `main`:

```rust
Commands::Config { action: ConfigAction::Init { global } } => {
    let (path, display) = if global {
        let home = env::var_os("HOME").context("$HOME is not set")?;
        (PathBuf::from(&home).join(".tick.toml"), "~/.tick.toml".to_string())
    } else {
        (cwd.join(".tick.toml"), "./.tick.toml".to_string())
    };
    let message = cli::run_config_init(&path, &display)?;
    println!("{message}");
}
```

The existing `home_config` variable computed at the top of `main` (used by
`Workspace::discover`) silently maps a missing `$HOME` to `None`; this new
branch needs `$HOME` itself (to know *where* to write, not just whether a
user config exists to layer in), so a missing `$HOME` is a hard error here
specifically for `-g`, distinct from the existing `home_config` computation.

## Test plan (TDD — write these first)

| Scenario | Test | Module |
|---|---|---|
| `default_toml` round-trips to `Config::default()` | parse `default_toml()`'s output with `toml::from_str::<RawConfig>` then `Config::resolve` on a tempfile containing it, assert equals `Config::default()` | `config` (unit, regression for the README-shape contract) |
| `default_toml` contains no `#:schema` line | assert `!default_toml().starts_with("#:schema")` | `config` (unit, config.md 006 boundary) |
| `init` creates the file when absent | tempdir, `init(&path)`, assert file exists and its contents parse back to `Config::default()` | `config` (unit, config.md 003 scenario 1 shape) |
| `init` refuses when the file already exists | write a file with custom content first, call `init(&path)`, assert `Err(ConfigError::AlreadyExists { .. })` and file contents unchanged | `config` (unit, config.md 004 scenario 1/2 shape) |
| `run_config_init` returns the created-path message | tempdir, call `run_config_init(&path, "./.tick.toml")`, assert `Ok("Created ./.tick.toml")` and file exists | `cli` (unit) |
| `run_config_init` surfaces the already-exists error | pre-existing file, assert `run_config_init` returns `Err` whose message mentions the path | `cli` (unit, config.md 004) |
| `tk config init` parses with no flag | `Cli::parse_from(["tk", "config", "init"])` → `Commands::Config { action: ConfigAction::Init { global: false } }` | `main` (parse test) |
| `tk config init -g` / `--global` parse | both forms → `ConfigAction::Init { global: true }` | `main` (parse test) |
| `tk config` with no subcommand errors at parse time | `Cli::try_parse_from(["tk", "config"])` is `Err` | `main` (parse test, documents the deferred-bare-command boundary) |
| Local exists, global doesn't: `-g` still succeeds | tempdir with `./.tick.toml` written, `HOME` pointed at a fresh tempdir, run the `-g` dispatch path, assert `~/.tick.toml` created and local file untouched | `main`/integration (config.md 004 scenario 3) |

## Implementation plan

1. Add `ConfigError::AlreadyExists`/`Write` variants and `default_toml`/
   `init` to `src/config.rs`; write the round-trip and already-exists tests
   first, watch them fail, then implement.
2. Add `cli::run_config_init` and its two tests (create / already-exists),
   same TDD order.
3. Add `Commands::Config`/`ConfigAction::Init` to `src/main.rs`, the parse
   tests (including the "no subcommand errors" boundary test), then the
   dispatch arm.
4. Manual smoke test:
   - In a scratch dir: `tk config init` → `Created ./.tick.toml`; inspect
     the file matches README's documented shape; re-run `tk config init`
     → error, file untouched.
   - `tk config init -g` with `$HOME` pointed at a scratch dir → `Created
     ~/.tick.toml`; re-run → error.
   - With both a local and no global file present, `tk config init -g`
     succeeds and leaves the local file alone.
5. Mark config.md 003 and 004 `✅`.
6. Update `docs/roadmap.md`'s item 8 entry to note `init`/`-g` are done,
   `edit`/schema still open.
7. `cargo clippy`, `cargo fmt --check`, `cargo test` clean before calling
   the stories done.
