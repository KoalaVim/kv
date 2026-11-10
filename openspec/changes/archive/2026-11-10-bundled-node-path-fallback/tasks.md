## 1. Failing test first

- [x] 1.1 In `src/launcher.rs` `mod tests`, add a `#[serial]` test modelled on `test_build_koala_env_sets_env_kvim_conf`: set `XDG_DATA_HOME` to a `TempDir`, create the env's `bin/` and `node/bin/` dirs, set `PATH` to `/u/a:/u/b` (`;`-joined on Windows), call `build_koala_env`, and assert `PATH == <env bin>:/u/a:/u/b:<env node>/bin`. Restore the original `PATH` at the end. Run `cargo test launcher` before the fix and record that it fails (today's output has `<env node>/bin` second).

## 2. Fix

- [x] 2.1 In `build_koala_env` (`src/launcher.rs:114-134`), move the bundled-Node block after the `current_path` push, as sketched in design.md: pick `node/bin` if it exists, otherwise `node/`, and append it only if it exists. Keep `<env bin>` first, and keep PATH unset when `<env bin>` doesn't exist.
- [x] 2.2 Run `cargo test launcher` and confirm the new test passes.

## 3. Verify

- [x] 3.1 Run `cargo check`, `cargo clippy -- -D warnings`, `cargo test`, `cargo fmt --check`. All must pass.
- [x] 3.2 From a shell where mise picks Node 26 (e.g. in the repo with `.nvmrc` 26.12.1), run the built `kv` and, in a Neovim terminal, run `node -v` (expect `v26.x`) and `echo $PATH | tr ':' '\n' | tail -1` (expect `<env>/kv/node/bin`).
- [x] 3.3 Run `env PATH=/usr/bin:/bin <built kv>`. In Neovim, run `:!which node` (expect the bundled `kv/node/bin/node`) and `:checkhealth mason` (expect npm/node found). Then confirm a Node-based LSP (e.g. jsonls on a `.json` file) attaches.
