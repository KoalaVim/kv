## Why

KoalaVim currently merges only two user-controlled `.kvim.conf` layers: the user conf (`~/.kvim.conf`, or `kv -c <path>`) and the repo conf (`<git toplevel>/.kvim.conf`). With virtual koala envs, users want settings that apply to one env only, for example an experimental env with different AI or LSP settings, without editing their global user conf and without adding a file to every repo.

## What Changes

- Add a new **env conf** layer: a `.kvim.conf` file in the env's config directory (`<env_config_dir>/.kvim.conf`, e.g. `~/.config/kvim-envs/<name>/.kvim.conf`).
- `kv` passes the env conf path to KoalaVim in a new environment variable, `KOALA_ENV_KVIM_CONF`, on every launch (including restarts), whether or not the file exists.
- KoalaVim merges config layers in this order, with later layers overriding earlier ones:
  1. KoalaVim defaults (`default_kvim.conf`)
  2. user conf
  3. env conf (new)
  4. repo conf
- KoalaVim exposes `get_env_conf()` and adds a dashboard button, "Koala Env Config", that creates the file if it is missing and opens it.
- `kv` adds a new subcommand, `kv env config [NAME] [--path]`. It creates the env conf with `{}` if it is missing and opens it in KoalaVim running in that env. `--path` only prints the path.
- The env conf is optional. A missing file contributes nothing and is not created automatically on load.
- Without `kv` (plain `nvim` with KoalaVim), the env layer is skipped and behavior stays the same as today.
- The env conf lives in the env config dir, so `kv env fork`, `rename` and `delete` already copy, move and remove it with no extra code.

No breaking changes. Existing user and repo confs keep working, and their relative precedence does not change.

## Capabilities

### New Capabilities
- `kvim-conf-layers`: How `kv` finds, passes and edits `.kvim.conf` layers (user, env) for KoalaVim, and the merge order KoalaVim must apply (defaults → user → env → repo).

### Modified Capabilities
<!-- None: openspec/specs/ is empty; no existing capability specs to modify. -->

## Impact

- **kv**: `src/paths.rs` (new `env_kvim_conf(name)` helper), `src/launcher.rs` (`build_koala_env` sets `KOALA_ENV_KVIM_CONF`), `src/cli.rs` (new `EnvAction::Config`), `src/main.rs` (handler that creates the file and launches), `docs/envs.md` / `docs/launcher.md` (document the layer), plus tests.
- **KoalaVim** (separate repo, edited at `~/.local/share/kvim-envs/main/lazy/KoalaVim`): `lua/KoalaVim/conf.lua` (new `get_env_conf()`, new merge order in `load()`), `lua/KoalaVim/plugins/ui.lua` (dashboard button), and the JSON schema `fileMatch` so env confs get completion and validation.
- **Interface contract**: a new kv → KoalaVim environment variable, `KOALA_ENV_KVIM_CONF`. Older KoalaVim versions ignore it. A newer KoalaVim launched by an older kv just skips the env layer.
- No new dependencies.
