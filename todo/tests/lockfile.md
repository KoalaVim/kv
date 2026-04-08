# Lockfile Testing

## Unit Tests (implemented)

- `read_lockfile` -- reads and parses a `lazy-lock.json` file
- `read_lockfile` -- returns error for missing file
- `write_lockfile` -- roundtrip write and re-read preserving data
- `write_lockfile` -- output format starts with `{`, ends with `}\n`, contains plugin names
- `plugin_commit` -- extracts commit field from a JSON value

## Integration Tests Needed

### Diff
- Verify diff correctly identifies plugins with differing commits
- Verify diff excludes the `KoalaVim` entry
- Verify diff handles missing plugins in user lockfile (shows N/A)
- Verify diff shows "in sync" message when lockfiles match
- Verify diff fails with clear error when user lockfile doesn't exist
- Verify diff fails with clear error when KoalaVim lockfile doesn't exist

### Overwrite
- Verify overwrite copies KoalaVim lockfile content to user lockfile
- Verify overwrite removes the `KoalaVim` entry from the written file
- Verify overwrite respects `--yes` flag (skips confirmation)
- Verify overwrite aborts when user declines confirmation

### Lazy restore (requires nvim + lazy.nvim + KoalaVim)
- Verify `lazy_restore` sets `NVIM_APPNAME` correctly for the target env
- Verify `lazy_restore` parses successful JSON output from stderr
- Verify `lazy_restore` reports plugin errors from the JSON output
- Verify `lazy_restore` handles non-JSON stderr gracefully

## Why Hard to Test

- **Diff/Overwrite**: These are mostly testable with temp dirs and synthetic lockfiles. The main challenge is the `inquire::Confirm` prompt in `overwrite` -- it requires stdin interaction.
- **Lazy restore**: Requires a working nvim with lazy.nvim and the `LazyRestoreLogged` command. This is a KoalaVim-specific command that outputs JSON to stderr. Cannot be unit-tested without a full KoalaVim setup.

## Automation Ideas

- **Diff/Overwrite**: Create integration tests using `with_temp_xdg()` and synthetic lockfiles. For overwrite without `--yes`, either test only the `--yes` path or inject a mock prompt.
- **Lazy restore**: Create a shell script that mimics nvim's behavior (outputs expected JSON to stderr) and use `--nvim-bin-path` or `PATH` override to point at it. Alternatively, use a real nvim + lazy.nvim test env in CI.
- **End-to-end**: Set up a minimal lazy.nvim config in a temp env, create synthetic lockfiles, run `kv lockfile diff` and verify output.
