# Update Testing

## Unit Tests (implemented)

- `resolve_target` -- correctly identifies commit hashes (4-40 hex chars) vs branch names
- `resolve_target` -- prepends `<remote>/` for branch names
- `resolve_target` -- keeps commit hashes as-is

## Integration Tests Needed

### Target resolution
- Verify short hex strings (4 chars) are treated as commit hashes
- Verify strings under 4 chars are treated as branch names
- Verify mixed alpha-hex strings (e.g., "abcxyz") are treated as branch names

### Dirty check (requires git repo)
- Set up a temp git repo, make it dirty, verify `is_repo_dirty` returns true
- Set up a clean temp git repo, verify `is_repo_dirty` returns false
- Verify update fails on dirty repo without `--force`
- Verify update proceeds on dirty repo with `--force`

### Git operations (requires git repo with remote)
- Verify `git_fetch` succeeds with a valid remote
- Verify `git_fetch` fails with an invalid remote and returns clear error
- Verify `git_reset` moves HEAD to the specified commit

### Lockfile backup
- Verify backup creates a file with timestamp in the name
- Verify backup is skipped when user lockfile doesn't exist
- Verify backup creates the kv data directory if it doesn't exist

### Full update flow (requires nvim + lazy.nvim + KoalaVim)
- Verify the complete flow: fetch -> dirty check -> reset -> backup -> overwrite -> restore
- Verify `--no-restore` skips the lazy restore step
- Verify update fails with clear message when KoalaVim dir doesn't exist

## Why Hard to Test

- **Git operations**: Need a real git repo with at least one remote. Can be set up in tests using `git init` + `git remote add` + local bare repos, but it's complex.
- **Full flow**: Requires nvim + lazy.nvim + KoalaVim installed. The `lazy_restore` step is the same challenge as in lockfile testing.
- **Dirty check**: Straightforward to test with temp git repos.

## Automation Ideas

- **Git operations**: Create a temp bare repo as the "remote", clone it to a temp working dir, make commits, then test fetch/reset against it.
- **Backup**: Test with `with_temp_xdg()` and a synthetic user lockfile. Verify the backup file is created with the expected naming pattern.
- **Full flow without restore**: Use `--no-restore` in integration tests to skip the nvim dependency. This tests everything except lazy restore.
- **CI**: Set up a test fixture with a minimal KoalaVim-like repo and a fake lazy.nvim config for end-to-end testing.
