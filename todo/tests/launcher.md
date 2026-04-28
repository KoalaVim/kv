# Launcher Testing

## Unit Tests (implemented)

- `join_args` -- space-separated joining of OsString slices
- `tilde_shorten` -- HOME path shortening

## Integration Tests Needed

### Env resolution
- Verify `resolve_env_name` returns error for non-existent explicit `--env`
- Verify `resolve_env_name` exits with setup instructions when default `main` doesn't exist
- Verify `resolve_env_name_unchecked` validates name without checking disk

### Mode resolution
- Verify only one mode flag can be active at a time
- Verify each mode maps to the correct `KOALA_MODE` value

### Env var construction
- Verify `NVIM_APPNAME` is set correctly for each env
- Verify `PATH` is prepended with `env_bin_dir` when it exists
- Verify `PATH` is left alone when `env_bin_dir` doesn't exist
- Verify debug env vars are set when `-d` is passed
- Verify `KOALA_NO_NOICE` is set when `-n` is passed
- Verify `KOALA_NO_SESSION` and `KOALA_MODE` are set in mode launches

### Nvim launch (requires nvim)
- Verify nvim is launched with the correct binary (default vs `--nvim-bin-path`)
- Verify positional args are passed to nvim in non-mode launches
- Verify positional args go to `KOALA_ARGS` in mode launches
- Verify restart loop triggers when `restart_kvim` file exists
- Verify `KOALA_RESTART=1` is set on subsequent restart iterations

## Why Hard to Test

The launcher's core function (`run_kvim`) spawns an interactive nvim process. Testing this end-to-end requires either:
- A mock nvim binary that validates env vars and exits
- A headless nvim with a test plugin that asserts the environment

## Automation Ideas

- Create a small shell script `fake-nvim` that writes env vars to a temp file and exits, then assert file contents
- Use `--nvim-bin-path` to point at the fake binary during tests
- Test the restart loop by having fake-nvim create the `restart_kvim` file on first run only
