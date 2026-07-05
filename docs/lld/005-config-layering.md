# LLD: config layering (config.md 001–002)

Source: [docs/user-stories/config.md](../user-stories/config.md) Stories
001–002. Module boundaries follow [docs/design.md](../design.md).
Corresponds to roadmap item 2.

## Scope

1. Merge `.tick.toml` across three layers — built-in defaults, `~/.tick.toml`
   (user), `./.tick.toml` (local) — with local taking precedence over user,
   and user over the built-in default, **per key** (config.md 002).
2. Track, per key, which layer its effective value came from, so a future
   `tk config` can annotate each line (config.md 001). The label needed is
   one of: `default`, `user`, `local`, or `local, overrides user` — the last
   one only when both a user and a local file set the *same* key.
3. Fix the TOML schema `Config` parses: today's `src/config.rs` reads a flat
   `default_extension` / `category_dirs.*` shape that doesn't match the
   nested `[folders]` / `[defaults]` / `[templates]` tables `README.md`
   already documents, and doesn't parse `[templates]` overrides at all. This
   has to be fixed as part of this item — layering two files together only
   makes sense once each file is actually being read correctly.
4. `Workspace::discover` gains a `home_config: Option<&Path>` parameter and
   layers it in on both of its branches (found `.tick.toml`, and the bare
   category-dirs fallback) — the "only a user-level config exists, no local
   file at all" scenario (config.md 002) needs user overrides applied even
   when there's no `./.tick.toml` to discover by.

### Out of scope

- The `tk config` / `config init` / `config edit` / `-g` CLI surface itself
  (config.md 003–006) — roadmap item 8, sequenced after this one
  specifically so the resolution logic underneath it lands first. This LLD
  produces the `(Config, ConfigOrigins)` pair that command will render; it
  doesn't add the `config` subcommand or any printing.
- The `#:schema` JSON Schema file (config.md 006) — same reason.
- Any change to `Config`'s consumers (`workspace::category_dir`,
  `items::create`, `cli::run_new`/`run_daily`) — they keep consuming the
  plain, provenance-free `Config` exactly as today.

## `design.md` changes

Applied directly to `docs/design.md`'s `config` section (not deferred),
since the layering/provenance split is meant to be load-bearing
documentation, not just an implementation note:

- `Config::load(path)` (single file) is removed; replaced by
  `Config::resolve(local_path, home_path)`, returning `(Config,
  ConfigOrigins)`.
- New `Source` enum and `ConfigOrigins`/`TemplateOrigins` structs, described
  below, documented alongside `Config`/`Templates`.
- `workspace` section: `Workspace::discover`'s signature gains
  `home_config: Option<&Path>`.

## Module designs

### `config` (extends existing module)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    Default,
    User,
    Local,
    LocalOverridesUser,
}

impl Source {
    /// The exact annotation `tk config` prints after each key, per
    /// config.md 001 (`# default`, `# user`, `# local`, `# local, overrides user`).
    pub fn comment(self) -> &'static str {
        match self {
            Source::Default => "default",
            Source::User => "user",
            Source::Local => "local",
            Source::LocalOverridesUser => "local, overrides user",
        }
    }
}

pub struct ConfigOrigins {
    /// Indexed by `Category as usize`, same convention as `Config::category_dirs`.
    pub category_dirs: [Source; 5],
    pub default_extension: Source,
    pub templates: TemplateOrigins,
}

pub struct TemplateOrigins {
    pub note: Source,
    pub daily: Source,
    pub project: Source,
    pub area: Source,
    pub resource: Source,
}
```

`Config`/`Templates` themselves are unchanged — every existing consumer
(`workspace`, `items`, `cli`) keeps working against plain values with no
provenance attached. `ConfigOrigins` is a parallel, same-shape struct that
only the future `tk config` display path will consume.

Raw TOML shape, replacing today's flat `TomlConfig`/`TomlCategoryDirs` with
nested tables matching `README.md` exactly, every field `Option` so a
partial file only overrides what it sets:

```rust
#[derive(Debug, Default, Deserialize)]
struct RawConfig {
    folders: Option<RawFolders>,
    defaults: Option<RawDefaults>,
    templates: Option<RawTemplates>,
}

#[derive(Debug, Default, Deserialize)]
struct RawFolders {
    inbox: Option<String>,
    projects: Option<String>,
    areas: Option<String>,
    resources: Option<String>,
    archive: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct RawDefaults {
    extension: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct RawTemplates {
    note: Option<String>,
    daily: Option<String>,
    project: Option<String>,
    area: Option<String>,
    resource: Option<String>,
}

fn read_raw(path: &Path) -> Result<RawConfig, ConfigError> {
    if !path.exists() {
        return Ok(RawConfig::default());
    }
    let raw = fs::read_to_string(path).map_err(|source| ConfigError::Read {
        path: path.display().to_string(),
        source,
    })?;
    toml::from_str(&raw).map_err(|source| ConfigError::Parse {
        path: path.display().to_string(),
        source,
    })
}
```

`read_raw` is exactly today's `Config::load` body, minus the merge-into-a-
`Config` step at the end — it's called once for the local path and once
(if given) for the home path.

The merge itself is one small, pure, generic function, called once per
field instead of writing the same four-way match eleven times:

```rust
/// `local` wins over `user`, which wins over `default`. The `Source`
/// returned distinguishes "local, but nothing to override" from "local,
/// overriding a value the user config set" — the two different labels
/// config.md 001 requires for the same effective value.
fn merge<T>(default: T, user: Option<T>, local: Option<T>) -> (T, Source) {
    let source = match (&user, &local) {
        (_, Some(_)) if user.is_some() => Source::LocalOverridesUser,
        (_, Some(_)) => Source::Local,
        (Some(_), None) => Source::User,
        (None, None) => Source::Default,
    };
    let value = local.or(user).unwrap_or(default);
    (value, source)
}
```

`Config::resolve` reads both layers, then calls `merge` once per field:

```rust
impl Config {
    /// Reads `local_path` and, if given, `home_path`, and layers them over
    /// `Config::default()` — local takes precedence over user, user over
    /// the built-in default, independently per key. Neither file needs to
    /// exist; a missing file behaves as if it set no keys at all.
    pub fn resolve(
        local_path: &Path,
        home_path: Option<&Path>,
    ) -> Result<(Config, ConfigOrigins), ConfigError> {
        let local = read_raw(local_path)?;
        let user = match home_path {
            Some(p) => read_raw(p)?,
            None => RawConfig::default(),
        };
        let defaults = Config::default();

        let local_folders = local.folders.unwrap_or_default();
        let user_folders = user.folders.unwrap_or_default();
        let (inbox, inbox_src) = merge(defaults.category_dirs[0].clone(), user_folders.inbox, local_folders.inbox);
        let (projects, projects_src) = merge(defaults.category_dirs[1].clone(), user_folders.projects, local_folders.projects);
        let (areas, areas_src) = merge(defaults.category_dirs[2].clone(), user_folders.areas, local_folders.areas);
        let (resources, resources_src) = merge(defaults.category_dirs[3].clone(), user_folders.resources, local_folders.resources);
        let (archive, archive_src) = merge(defaults.category_dirs[4].clone(), user_folders.archive, local_folders.archive);

        let (extension, extension_src) = merge(
            defaults.default_extension.clone(),
            user.defaults.unwrap_or_default().extension,
            local.defaults.unwrap_or_default().extension,
        );

        let local_templates = local.templates.unwrap_or_default();
        let user_templates = user.templates.unwrap_or_default();
        let (note, note_src) = merge(defaults.templates.note.clone(), user_templates.note, local_templates.note);
        let (daily, daily_src) = merge(defaults.templates.daily.clone(), user_templates.daily, local_templates.daily);
        let (project, project_src) = merge(defaults.templates.project.clone(), user_templates.project, local_templates.project);
        let (area, area_src) = merge(defaults.templates.area.clone(), user_templates.area, local_templates.area);
        let (resource, resource_src) = merge(defaults.templates.resource.clone(), user_templates.resource, local_templates.resource);

        Ok((
            Config {
                category_dirs: [inbox, projects, areas, resources, archive],
                default_extension: extension,
                templates: Templates { note, daily, project, area, resource },
            },
            ConfigOrigins {
                category_dirs: [inbox_src, projects_src, areas_src, resources_src, archive_src],
                default_extension: extension_src,
                templates: TemplateOrigins { note: note_src, daily: daily_src, project: project_src, area: area_src, resource: resource_src },
            },
        ))
    }
}
```

`Config::load` is deleted, not kept alongside `resolve` — every call site
(`Workspace::discover`, plus the future `tk config`) needs the layered
version, and the old flat schema it parsed no longer matches any
documented `.tick.toml` shape.

### `workspace` (extends existing module)

```rust
impl Workspace {
    pub fn discover(
        start: &Path,
        home_config: Option<&Path>,
    ) -> Result<Workspace, WorkspaceError> {
        for dir in start.ancestors() {
            let tick_toml = dir.join(".tick.toml");
            if tick_toml.exists() {
                let (config, _origins) = Config::resolve(&tick_toml, home_config)?;
                return Ok(Workspace { root: dir.to_path_buf(), config });
            }

            let default_config = Config::default();
            if default_config
                .category_dirs
                .iter()
                .all(|name| dir.join(name).is_dir())
            {
                // No local `.tick.toml`, but a user-level one may still
                // apply — config.md 002's "only a user-level config exists"
                // scenario. `local_path` doesn't need to exist; `resolve`
                // treats a missing file as setting no keys.
                let (config, _origins) = Config::resolve(&dir.join(".tick.toml"), home_config)?;
                return Ok(Workspace { root: dir.to_path_buf(), config });
            }
        }

        Err(WorkspaceError::NotFound { start: start.display().to_string() })
    }
}
```

`Workspace` keeps discarding `ConfigOrigins` — nothing under `workspace`,
`items`, or the existing `cli` commands needs provenance, only the future
`tk config` display path does, and that path calls `Config::resolve`
directly with `ws.root.join(".tick.toml")` rather than going through
`Workspace::discover`.

### `main` (call-site update, no new behavior)

Per `docs/design.md`'s separation of pure logic from impure edges, neither
`config` nor `workspace` reads `$HOME` itself — `main.rs` (already the
impure entry point; it calls `env::current_dir()`) computes the home
config path once, the same way it already computes `cwd`, and passes it
down:

```rust
let cwd = env::current_dir().context("failed to determine current directory")?;
let home_config = env::var_os("HOME").map(|home| PathBuf::from(home).join(".tick.toml"));

// every existing call site:
let ws = Workspace::discover(&cwd, home_config.as_deref())
    .context("failed to find a PARA workspace")?;
```

This keeps `home_config: Option<&Path>` directly injectable in
`workspace`'s and `config`'s own tests — no env var mutation needed to
simulate "user config only" scenarios.

## Test plan (TDD — write these first)

| Scenario | Test | Module |
|---|---|---|
| Truth table for the merge primitive | `merge(default, None, None)` → `(default, Default)`; `merge(default, Some(u), None)` → `(u, User)`; `merge(default, None, Some(l))` → `(l, Local)`; `merge(default, Some(u), Some(l))` → `(l, LocalOverridesUser)` | `config` (unit) |
| Neither file present | `Config::resolve(missing_local, None)` → `Config::default()`, every `ConfigOrigins` field `Default` | `config` (unit, config.md 002 scenario 4) |
| Only local overrides one key | local sets `folders.inbox`, no home path → that field `Local`, every other field `Default` | `config` (unit, config.md 001 scenario 2) |
| Only user overrides one key | home path sets `templates.note`, local file missing → that field `User`, everything else `Default` | `config` (unit, config.md 001 scenario 4 / 002 scenario 1) |
| User and local set disjoint keys | home sets `templates.daily`, local sets `folders.inbox` → both present in the effective config, independently sourced (`User`/`Local`) | `config` (unit, config.md 002 scenario 2) |
| User and local both set the same key | both set `templates.daily` to different values → effective value is local's, source `LocalOverridesUser` | `config` (unit, config.md 001 scenario 3 / 002 scenario 3) |
| Nested `[folders]`/`[defaults]`/`[templates]` TOML actually parses | a file using exactly `README.md`'s documented shape round-trips through `resolve` | `config` (unit, regression for the schema fix) |
| Legacy flat shape is no longer accepted | a file using the old `default_extension`/`category_dirs.*` keys parses as if it set nothing (unknown keys ignored by `toml`), landing on defaults rather than silently misreading old fields as new ones | `config` (unit) |
| `Workspace::discover` layers a user config with no local file present | bare category dirs on disk, no `.tick.toml`, `home_config` pointing at a temp file that sets `templates.note` → resulting `ws.config.templates.note` is the override | `workspace` (unit, config.md 002 scenario 1) |
| `Workspace::discover` layers both when a local `.tick.toml` exists too | local file sets `folders.archive`, `home_config` sets `templates.daily` → both present in `ws.config` | `workspace` (unit, config.md 002 scenario 2) |
| `Workspace::discover` works with `home_config: None` | existing discovery tests continue to pass unchanged, just with an added `None` argument | `workspace` (regression) |

## Implementation plan

1. Add `Source`, `ConfigOrigins`, `TemplateOrigins`, and the `RawConfig`/
   `RawFolders`/`RawDefaults`/`RawTemplates` types to `src/config.rs`.
   Delete `TomlConfig`/`TomlCategoryDirs`.
2. Write the `merge` truth-table tests first (they fail to compile against
   nothing yet), then implement `merge`.
3. Write `Config::resolve`'s tests (the six `config` scenarios in the table
   above), watch them fail, then implement `resolve` and delete `Config::load`.
4. Update `src/config.rs`'s existing tests that called `Config::load` with
   the old flat schema — rewrite their fixture TOML to the nested shape and
   call `Config::resolve(path, None)` instead.
5. Change `Workspace::discover`'s signature to take `home_config: Option<&Path>`;
   update both branches to call `Config::resolve`; update every existing
   `workspace` test call site to pass `None`; add the two new layering tests.
6. Update every `Workspace::discover` call site in `src/main.rs` to compute
   `home_config` via `env::var_os("HOME")` once and pass it through.
7. Mark config.md Stories 001–002 `✅` (provenance/layering only — the
   stories' `tk config` command text itself stays open until item 8).
8. Update `docs/roadmap.md`: move item 2 from "Next" to "Done", noting the
   schema fix and that item 8 (`tk config` CLI) is unblocked.
9. Manual smoke test: in a scratch PARA dir, write a `./.tick.toml` and a
   throwaway `~/.tick.toml`-equivalent temp file overriding different and
   overlapping keys, and confirm (via a small throwaway `dbg!`/test binary,
   since there's no `tk config` command yet to observe this through)
   that `Config::resolve` produces the expected merged values and sources
   for each combination.
10. `cargo clippy`, `cargo fmt --check`, `cargo test` clean before calling
    the stories done.
