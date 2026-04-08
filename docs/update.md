# Update

The `kv update` command updates the KoalaVim plugin to a target version, syncs the lockfile, and restores plugin versions.

## Usage

```bash
kv update                                    # update to latest master
kv update --target v2.0                      # update to a specific branch/tag
kv update --target abc123def                 # update to a specific commit
kv update --remote upstream                  # fetch from a different remote
kv update --force                            # ignore dirty working directory
kv update --no-restore                       # skip lazy restore after update
kv --env myenv update                        # update for a specific env
```

## Update Flow

1. **Locate KoalaVim** -- resolves the KoalaVim plugin directory at `<data_dir>/lazy/KoalaVim`. Fails if not found (run `kv` first to let lazy.nvim install it).

2. **Dirty check** -- runs `git status --porcelain` in the KoalaVim directory. If dirty, the update is aborted unless `--force` is passed.

3. **Fetch** -- runs `git fetch <remote>` to get the latest refs.

4. **Reset** -- determines whether `--target` is a commit hash (4-40 hex chars) or a branch/tag name, then runs `git reset --hard <target>`. Branch names are resolved as `<remote>/<target>`.

5. **Backup lockfile** -- copies the user's current `lazy-lock.json` to `<kv_data_dir>/lazy-lock-<timestamp>.json.backup`.

6. **Overwrite lockfile** -- copies KoalaVim's lockfile over the user's (excluding the KoalaVim entry itself).

7. **Lazy restore** -- runs `nvim --headless +LazyRestoreLogged +qa` to sync plugin versions. Skipped if `--no-restore` is passed.

## Options

| Flag | Default | Description |
|---|---|---|
| `--target` | `master` | Commit hash or branch/tag to reset to |
| `--remote` | `origin` | Git remote to fetch from |
| `--force` | `false` | Proceed even if KoalaVim dir is dirty |
| `--no-restore` | `false` | Skip running `:Lazy restore` after update |

## Lockfile Backups

Before each update, the user's lockfile is backed up to:

```
<data_dir>/kv/lazy-lock-<DD-MM-YY_HH:MM:SS>.json.backup
```

This allows rolling back if an update causes issues.
