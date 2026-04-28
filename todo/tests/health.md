# Health Testing

## Unit Tests (implemented)

- `find_binary` -- falls back to bare name when env bin dir doesn't exist
- `HEALTH_CHECKS` -- verify all expected checks are registered (nvim, rg, fd, fzf)

## Integration Tests Needed

### Individual health checks (requires the tools to be installed)
- Verify `check_nvim` returns Ok with version when nvim is available
- Verify `check_nvim` returns Missing when nvim is not in PATH
- Verify `check_ripgrep` parses rg version correctly
- Verify `check_fd` parses fd version correctly
- Verify `check_fzf` parses fzf version correctly
- Verify `check_git` parses git version correctly
- Verify `check_curl` parses curl version correctly
- Verify `check_nerd_font` detects installed nerd fonts via fc-list
- Verify `check_nerd_font` returns Missing when no nerd font is installed

### Env-aware resolution
- Install a binary to `env_bin_dir("test-env")`, verify `find_binary` returns the env path
- Verify `find_binary` falls back to system PATH when env bin dir is empty
- Verify `find_binary` falls back to system PATH when env bin dir doesn't exist

### Output formatting
- Verify `cmd_health` prints the env name in the header
- Verify `cmd_health` groups checks by core/dependencies/optional
- Verify OK results show version, MISSING results show reason

## Why Hard to Test

- **Tool availability**: Health checks shell out to the actual tools (nvim, rg, fd, fzf, git, curl, fc-list). Tests will only pass on systems with those tools installed.
- **Version parsing**: Each tool has its own version output format. Version format can change across releases.
- **Nerd font detection**: Requires `fc-list` (fontconfig), which is unix-only and depends on installed fonts.
- **Output formatting**: The colored output uses owo-colors which makes string comparison tricky.

## Automation Ideas

- **Mock binaries**: Create shell scripts that output known version strings, place them in a temp dir, set PATH to that dir. Test that health checks parse the output correctly.
- **Env-aware tests**: Use `with_temp_xdg()`, create the bin dir, place mock binaries there, verify `find_binary` picks them up.
- **Snapshot testing**: Capture `cmd_health` output for known tool versions and compare against expected strings (strip ANSI codes first).
- **CI**: Most CI environments have git and curl. Install rg, fd, fzf in CI setup steps. For nerd font, skip or mock on CI.
