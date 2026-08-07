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
4. Run the two-phase lazy restore (below) to sync plugins

## Lazy Restore

The restore step launches nvim headlessly with the target env's `NVIM_APPNAME` set, in **two phases**:

1. **Install** -- `nvim --headless +qa`. lazy.nvim installs any missing plugins and settles.
2. **Restore** -- re-write the user lockfile, then `nvim --headless +LazyRestoreLogged +qa`. `LazyRestoreLogged` is a KoalaVim-specific command that outputs JSON to stderr; kv parses it to report plugin errors.

### Why two phases

lazy.nvim installs missing plugins during startup, and every install ends by rewriting the lockfile -- and its in-memory copy of it -- from whatever commit each plugin is *currently* checked out to. A `:Lazy restore` in that same session then reads those stale commits back as its targets, concludes every plugin is already where it belongs, and skips every checkout. No task error is raised, so the restore reports success while nothing moved.

Running install and restore in separate nvim processes removes the precondition: by phase 2 nothing is missing, so the install path never runs and the lockfile lazy reads is the one kv just wrote. The lockfile is re-written between phases because phase 1's trailing rewrite clobbers it.

### Verification

After the restore, kv checks each plugin's actual `git rev-parse HEAD` against KoalaVim's lockfile.

This is the only reliable signal that the restore did anything. lazy runs a pipeline per plugin (`fetch` -> `status` -> `checkout` -> ...), and **a task error halts the rest of that plugin's pipeline**. If `status` errors, `checkout` never runs and no error surfaces as a failed checkout -- the restore reports clean while that plugin sits at its old commit.

- **Commit mismatch** -- hard error. The plugin did not move.
- **Plugin not installed** -- informational note. KoalaVim's lockfile keeps entries for plugins the user has disabled, and those legitimately have no directory on disk.

#### Local changes are the usual cause

lazy's `git.status` task refuses to update a plugin with uncommitted modifications: it raises "You have local changes in ...", which halts the pipeline before `git.checkout`. When kv reports drift it also runs `git ls-files -d -m` in the plugin dir and lists any modified files, since that is almost always the explanation.

Commit, stash, or discard the changes and re-run the update.

> `doc/tags` is excluded from that list -- lazy rewrites it itself rather than erroring on it.

## Lockfile Format

The `lazy-lock.json` format is a JSON object mapping plugin names to their metadata:

```json
{
  "plugin-name": { "commit": "abc123def", "branch": "main" }
}
```

When writing lockfiles, `kv` preserves lazy.nvim's formatting convention with `{ ` and ` }` spacing inside value objects.
