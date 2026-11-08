## Context

- `kv` (`src/launcher.rs::build_koala_env`) sets `KOALA_KVIM_CONF` from `-c/--cfg` (default `~/.kvim.conf`) and `KOALA_ENV=<name>`.
- KoalaVim's `lua/KoalaVim/conf.lua::load()` currently does:
  ```lua
  local default = json_file.load(M.get_kvim_conf(), false)
  local user    = json_file.load(M.get_user_conf(), true)   -- created if missing
  local git     = json_file.load(M.get_repo_conf(), false)
  conf = vim.tbl_deep_extend('keep', git, user, default)    -- 'keep' => leftmost wins
  ```
- `json_file.load(nil, ...)` already returns `{}`, and a decode error produces a health warning and returns `{}`. Together these give the "optional layer" behavior with no new code.
- Env dirs are resolved by `src/paths.rs` (`env_config_dir`), which mirrors nvim's `stdpath('config')` for `NVIM_APPNAME=kvim-envs/<name>`.

This is a cross-repo change: kv (this repo) and KoalaVim (edited in place at `~/.local/share/kvim-envs/main/lazy/KoalaVim`, its own git repo on `master`).

## Goals / Non-Goals

**Goals:**
- Add exactly one new layer between user and repo, with no change to how existing layers are found.
- Keep KoalaVim fully working without `kv`.

**Non-Goals:**
- Changing merge semantics (for example list replacement versus `tbl_deep_extend` index merging). Behavior stays whatever `tbl_deep_extend` does today.
- A `kv` flag to override the env conf path.
- Shipping a `.kvim.conf` inside `KoalaConfig.template`.
- Reporting which layer set each effective key.

## Decisions

### D1: Env conf lives at `<env_config_dir>/.kvim.conf`
- **Why:** It is per-env by construction. It is copied, moved and deleted by the existing fork, rename and delete code with no changes. It sits next to the user's `init.lua`, so users who keep their env config in git (for example a KoalaConfig clone) can version it. The `.kvim.conf` name already matches KoalaVim's filetype and schema pattern.
- **Alternatives:** `<env_data_dir>/kv/kvim.conf` was rejected: data dirs hold machine state, not user-edited config, and they are not version-controlled. `~/.kvim.<env>.conf` was rejected: it is not cleaned up on `env delete` and it clutters `$HOME`.

### D2: kv passes the path explicitly via `KOALA_ENV_KVIM_CONF`
- **Why:** This follows the monorepo contract that kv → KoalaVim communication goes through env vars, and `kv` owns path resolution (`paths.rs`). KoalaVim does not need to know about env layout.
- **Alternative:** KoalaVim computes `vim.fn.stdpath('config') .. '/.kvim.conf'` itself. Rejected because under plain `nvim` this would make `~/.config/nvim/.kvim.conf` silently become a layer, which is a surprising behavior change. With the env var, "no kv" means "no env layer".
- The variable is always set, even when the file is missing, so the dashboard button can create the file at the right path.

### D3: Merge with one extra argument to `tbl_deep_extend('keep', ...)`
```lua
local env = json_file.load(M.get_env_conf(), false)   -- nil-safe, not auto-created
conf = vim.tbl_deep_extend('keep', repo, env, user, default)
```
- **Why:** This is the smallest diff and uses the same semantics as the existing layers. `'keep'` puts the highest-priority layer first. A short comment will spell out the order, since the reversed argument order is easy to misread.
- `get_env_conf()` returns `vim.env.KOALA_ENV_KVIM_CONF`, or `nil` when it is unset or empty.

### D4: Env conf is not auto-created on load (unlike user conf)
- **Why:** If it were, every env would get an empty `.kvim.conf` in its config dir, which adds noise to git-tracked configs. It is created only on demand from the dashboard, the same way the repo conf works.

### D5: Schema matching
- `PATTERN = { '.kvim.conf', 'default_kvim.conf' }` is used for both the `BufEnter` autocmd pattern and the jsonls `fileMatch`. Both match by basename, so `<env_config_dir>/.kvim.conf` is already covered. No change is needed, but a task will verify this.

### D6: kv-side helper
- Add `paths::env_kvim_conf(name) -> PathBuf` (`env_config_dir(name).join(".kvim.conf")`), keeping path logic in `paths.rs`. `build_koala_env` pushes `("KOALA_ENV_KVIM_CONF", env_kvim_conf(env_name).into_os_string())`. The value is an `OsString`, following the project convention.

### D7: `kv env config [NAME] [--path]` opens the file in KoalaVim inside the env
- **Name:** `config` sits alongside the existing `kv env path`. `NAME` is optional (unlike `path`) because editing the current env is the common case. Resolution reuses `resolve_env_name` (CLI `--env`, then `KV_ENV`, then `main`) when `NAME` is omitted.
- **Editor:** the handler creates the file if it is missing (`{}`, through `fs::write` after checking existence), sets `cli.env = Some(name)` and `cli.nvim_args = vec![path]`, then calls `launcher::launch_nvim(cli)`. Reusing the launcher gives the env's PATH, `VIMRUNTIME`, the `KOALA_*` vars, jsonls schema validation for `.kvim.conf`, and the restart loop, with no second launch path to maintain.
- **Alternative:** open in `$VISUAL`/`$EDITOR`. Rejected as the default because it loses schema validation and adds editor-variable handling. `--path` covers that case: `$EDITOR "$(kv env config --path)"`.
- **`--path`** only computes `paths::env_kvim_conf(name)` and prints it. It still validates that the env exists, so scripts fail fast on typos.

## Risks / Trade-offs

- [Older KoalaVim with newer kv] The variable is ignored and the env conf has no effect → Document that the feature needs a matching KoalaVim version. This is harmless.
- [Newer KoalaVim with older kv] The variable is unset and the env layer is skipped → This is acceptable and needs no action.
- [Users who already have a file named `.kvim.conf` in their env config dir] It will now be applied as a layer → This is unlikely and arguably the intended behavior. Mention it in the KoalaVim changelog.
- [`tbl_deep_extend` merges list-like tables by index] An env list combined with a user list can mix elements → This matches existing user/repo behavior. It is out of scope (see Non-Goals).

## Migration Plan

1. Land the KoalaVim change first. It is a no-op until kv sets the variable.
2. Land the kv change and release it.
3. To roll back, revert either side. Each side degrades gracefully without the other.
