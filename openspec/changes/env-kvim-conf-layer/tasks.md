## 1. KoalaVim: env layer (repo at `~/.local/share/kvim-envs/main/lazy/KoalaVim`)

- [x] 1.1 In `lua/KoalaVim/conf.lua`, add `M.get_env_conf()`, which returns `vim.env.KOALA_ENV_KVIM_CONF` or `nil` when it is unset or empty. Verify with `KOALA_ENV_KVIM_CONF=/tmp/x nvim --headless -c "lua print(require('KoalaVim.conf').get_env_conf())" -c q` (prints `/tmp/x`), and confirm it prints `nil` when the variable is unset.
- [x] 1.2 Update `M.load()` to load the env layer with `json_file.load(M.get_env_conf(), false)` and merge with `vim.tbl_deep_extend('keep', repo, env, user, default)`. Add a comment documenting the precedence order (defaults < user < env < repo). Verify manually: user `{"ai":{"mux_split":false}}` + env `{"ai":{"mux_split":true}}` gives `require('KoalaVim').conf.ai.mux_split == true`, and adding a repo conf with `false` brings it back to `false`.
- [x] 1.3 Verify that a missing env conf path is not created after launching, and that malformed JSON in the env conf shows a health warning while the other layers still apply.
- [x] 1.4 In `lua/KoalaVim/plugins/ui.lua`, add a dashboard button `kec` "Koala Env Config" next to `kc`/`krc`. It notifies "No kv env active" when `get_env_conf()` is nil. Otherwise it runs `json_file.create_default(path, {})` and opens the file. Verify both paths by hand from the dashboard.
- [x] 1.5 Confirm that opening `<env_config_dir>/.kvim.conf` sets `filetype=json` and that jsonls attaches the KoalaVim schema (the basename `PATTERN` already matches). Verify with `:set ft?` and schema completion in the buffer.
- [x] 1.6 Document the layer order (defaults → user → env → repo) and `KOALA_ENV_KVIM_CONF` in KoalaVim's README or config docs. Verify the docs section exists.

## 2. kv: pass env conf path

- [x] 2.1 Add `pub fn env_kvim_conf(name: &str) -> PathBuf` in `src/paths.rs` that returns `env_config_dir(name).join(".kvim.conf")`. Extend `test_env_derived_paths` (under `with_temp_xdg`-style isolation) to assert the path. Verify with `cargo test paths`.
- [x] 2.2 In `src/launcher.rs::build_koala_env`, push `("KOALA_ENV_KVIM_CONF", env_kvim_conf(env_name).into_os_string())` right after `KOALA_ENV`. Verify that `kv -v` prints the variable and that it survives the restart loop (it lives in `koala_env`, which is reused).
- [x] 2.3 Add a `#[serial]` unit test for `build_koala_env` asserting `KOALA_ENV_KVIM_CONF` equals `<XDG_CONFIG_HOME>/kvim-envs/<env>/.kvim.conf`, and that `KOALA_KVIM_CONF` still reflects `-c`. Confirm `kv` does not create the file. Verify with `cargo test launcher`.
- [x] 2.4 Add a test (or extend the existing fork test) showing that `kv env fork` copies `<config>/.kvim.conf` to the new env. Verify with `cargo test env`.
- [x] 2.5 Add `EnvAction::Config { name: Option<String>, #[arg(long)] path: bool }` in `src/cli.rs` with a doc comment ("Edit the env's .kvim.conf"). Add parse tests for `kv env config`, `kv env config work` and `kv env config work --path`. Verify with `cargo test cli`.
- [x] 2.6 Implement the `EnvAction::Config` handler in `src/main.rs`. It resolves the env (name, or `resolve_env_name`), errors if the env config dir is missing, and with `--path` prints `env_kvim_conf(name)` and returns. Otherwise it creates the file with `{}` if it is missing, sets `cli.env` and `cli.nvim_args = [path]`, and calls `launch_nvim`. Put the file creation in a small testable helper (for example `env::ensure_env_kvim_conf(name) -> Result<PathBuf, String>`). Add `#[serial]` tests using `with_temp_xdg()`: creates `{}` when missing, leaves existing content untouched, errors on an unknown env without creating files. Verify with `cargo test env`.
- [x] 2.7 Document the env conf layer, the precedence order and `kv env config [NAME] [--path]` in `docs/envs.md` and `docs/launcher.md` (mention `-c` affects only the user layer). Verify the docs render and match the spec.

## 3. Verification

- [x] 3.1 Run `cargo check`, `cargo clippy -- -D warnings`, `cargo test` and `cargo fmt --check` in kv. All must pass.
- [x] 3.2 End-to-end: create `~/.config/kvim-envs/main/.kvim.conf` with an override that differs from `~/.kvim.conf`, run `kv`, and confirm the env value is active. Then add a repo `.kvim.conf` overriding the same key and confirm the repo value wins. Run `kv --env <other>` and confirm the main env's override does not apply. Run `kv env config --path` (prints the path, no file created for a fresh env), then `kv env config` (opens KoalaVim on the file with JSON schema completion).
