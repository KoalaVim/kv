# Install Testing

## Unit Tests (implemented)

- `detect_os` -- returns Ok on supported platforms
- `detect_arch` -- returns Ok on supported architectures
- `find_asset_pattern` -- finds matching pattern for valid OS/arch combo
- `find_asset_pattern` -- returns error for empty patterns
- `find_binary_in_dir` -- finds binary in nested directory
- `find_binary_in_dir` -- returns error when binary not found
- `install_binary` -- copies binary to target directory, creates dir if needed
- `manifest_roundtrip` -- serialize/deserialize InstallManifest
- `dependencies_defined` -- verify all expected dependencies are registered

## Integration Tests Needed

### GitHub API resolution (requires network)
- Verify `resolve_download_url` returns a valid URL for each dependency
- Verify `resolve_download_url` handles `latest` version tag
- Verify `resolve_download_url` handles pinned version tags (e.g., `stable` for neovim)
- Verify `resolve_download_url` returns clear error for non-existent repo
- Verify `resolve_download_url` returns clear error when asset pattern doesn't match any asset

### Download (requires network + curl)
- Verify `download_file` downloads a file to the specified path
- Verify `download_file` returns error for invalid URL
- Verify `download_file` returns error when curl is not available

### Archive extraction (requires tar/unzip)
- Verify `extract_archive` handles `.tar.gz` files
- Verify `extract_archive` handles `.zip` files
- Verify `extract_archive` returns error for unknown formats
- Verify `extract_archive` returns error for corrupt archives

### Binary installation
- Verify binary is copied to env's bin dir
- Verify binary gets executable permissions on unix
- Verify bin dir is created if it doesn't exist
- Verify `.exe` suffix is handled on Windows

### Full install flow (requires network + curl + tar/unzip)
- Verify `cmd_install` installs all dependencies for the current platform
- Verify `cmd_install` writes the install manifest
- Verify `cmd_install --dry-run` doesn't create any files
- Verify `cmd_install --dry-run` prints expected information for each dependency
- Verify install continues on individual dependency failure and reports all errors

### Manifest tracking
- Verify manifest records version, URL, and timestamp for each installed dependency
- Verify manifest is updated on re-install (not duplicated)
- Verify manifest is read correctly on subsequent runs

## Why Hard to Test

- **GitHub API**: Requires network access and is rate-limited. Tests can flake due to network issues or GitHub rate limits.
- **Download + Extract**: Requires network, curl, tar, and unzip. The actual archives are large (tens of MB for neovim).
- **Platform-specific**: Asset patterns differ by OS/arch. CI needs to test on each platform.
- **Per-env isolation**: Need to verify the bin dir is used correctly by the launcher, which is a cross-module concern.

## Automation Ideas

- **Mock GitHub API**: Create a local HTTP server (or use test fixtures) that serves fake release JSON responses. Override the API URL in tests.
- **Small test archives**: Create tiny `.tar.gz` and `.zip` archives containing a fake binary. Use them to test the extract + install flow without network.
- **Dry-run tests**: The `--dry-run` path is fully testable without network or external tools. Capture stdout and verify it contains the expected output for each dependency.
- **Manifest tests**: Use `with_temp_xdg()` to create isolated envs, run install with mock archives, verify manifest contents.
- **CI matrix**: Run the full install flow on Linux x86_64, Linux aarch64, macOS x86_64, macOS aarch64 in CI. Use caching to avoid re-downloading on every run.
