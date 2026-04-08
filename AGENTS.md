# kv agent notes

## project overview

- `kv` is a Rust CLI tool that launches and manages [KoalaVim](https://github.com/KoalaVim/KoalaVim) environments.
- It supports launch modes (git, tree, diff, ai), virtual koala envs, debug output, and shell completions.
- Installed via `cargo install`; distributed as a single binary.

## build and verification

Run these commands in priority order. Always run `cargo check` after changes; run the full suite before considering work done.

1. `cargo check`
2. `cargo clippy -- -D warnings`
3. `cargo test`
4. `cargo fmt --check`

## architecture

- `main.rs` — entry point, nvim launching, CLI orchestration, restart loop.
- `cli.rs` — clap derive definitions (`Cli` struct, `Commands` and `EnvAction` enums).
- `env.rs` — virtual env management (create, fork, delete, rename, list, init wizard).
- `paths.rs` — XDG-compliant path resolution that mirrors neovim's `stdpaths` logic.

Keep this module separation. Each file owns a clear domain.

## error handling

- Use `Result<T, String>` for functions that can fail. This is the project convention for CLI simplicity.
- Propagate errors with `.map_err(|e| format!("context: {}", e))?` to add context.
- Never use `unwrap()` or `expect()` in non-test code. They are fine in tests.
- Do not introduce `anyhow` or `thiserror` unless explicitly asked.

## code style

- All code must pass `cargo clippy` with zero warnings.
- Use standard `cargo fmt` formatting. There is no custom `rustfmt.toml`.
- No `unsafe` code without explicit approval.
- Prefer iterators and combinators over manual loops where they improve clarity.
- Use `OsString`/`OsStr` for paths and OS-level strings, not `String`, where the value originates from or flows to the OS.

## dependencies

- Keep the dependency tree minimal. Do not add new crates without justification.
- This is a focused CLI tool — every dependency should earn its place.
- `Cargo.lock` is committed (correct for binary crates).

## testing

- Tests live in `#[cfg(test)] mod tests` at the bottom of each file.
- Tests that mutate environment variables must use `#[serial]` from `serial_test`.
- Use `tempfile::TempDir` for filesystem isolation.
- Use the `with_temp_xdg()` helper pattern to set up isolated XDG directories for env tests.

## CLI patterns

- Use clap's derive API (`#[derive(Parser)]`, `#[derive(Subcommand)]`).
- Nested subcommands use nested enums (`Commands` → `EnvAction`).
- Trailing args for nvim passthrough use `trailing_var_arg = true`.

## user-facing output

- Colored output via `owo_colors` (not `colored` or raw ANSI escapes).
- Error prefix: `"error:".red().bold()`, printed to stderr with `eprintln!`.
- Success actions: `.green()` (e.g., "Created", "Deleted", "Forked").
- Names: `.cyan().bold()`.
- Paths: `.dimmed()`.
- Normal output goes to stdout (`println!`), errors to stderr (`eprintln!`).

## cross-platform

- Path resolution in `paths.rs` mirrors neovim's XDG logic with platform-specific fallbacks.
- Use `#[cfg(unix)]` / `#[cfg(windows)]` for platform-specific code (e.g., symlinks, data dir suffixes).
- On Windows, neovim appends `-data` to appname for data/state dirs — `paths.rs` replicates this.

### Supported Platforms
- Linux
- MacOS
- Windows
