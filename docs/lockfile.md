# Lockfile Management

The `kv lockfile` command manages the `lazy-lock.json` files that lazy.nvim uses to pin plugin versions.

Each env has two lockfiles:
- **User lockfile**: `<config_dir>/lazy-lock.json` -- the user's pinned plugin versions.
- **KoalaVim lockfile**: `<data_dir>/lazy/KoalaVim/lazy-lock.json` -- the versions KoalaVim was tested against.

## Commands

### `kv lockfile diff`

Show which plugins differ between the user's lockfile and KoalaVim's lockfile.

```bash
kv lockfile diff
kv --env myenv lockfile diff
```

Output is a table showing plugin name, user commit, and KoalaVim commit for each differing plugin. The `KoalaVim` entry itself is always excluded from the diff.

### `kv lockfile overwrite`

Overwrite the user's lockfile with KoalaVim's lockfile (excluding the `KoalaVim` entry), then run `:Lazy restore` to sync plugin versions.

```bash
kv lockfile overwrite           # asks for confirmation
kv lockfile overwrite --yes     # skip confirmation
kv --env myenv lockfile overwrite -y
```

The overwrite flow:
1. Read KoalaVim's `lazy-lock.json`
2. Remove the `KoalaVim` entry (the user can't match this commit)
3. Write the result to the user's `lazy-lock.json`
4. Run `nvim --headless +LazyRestoreLogged +qa` to sync plugins

## Lazy Restore

The lazy restore step launches nvim headlessly with the target env's `NVIM_APPNAME` set. It runs the `LazyRestoreLogged` command (a KoalaVim-specific command that outputs JSON to stderr), then parses the result to report any plugin errors.

## Lockfile Format

The `lazy-lock.json` format is a JSON object mapping plugin names to their metadata:

```json
{
  "plugin-name": { "commit": "abc123def", "branch": "main" }
}
```

When writing lockfiles, `kv` preserves lazy.nvim's formatting convention with `{ ` and ` }` spacing inside value objects.
