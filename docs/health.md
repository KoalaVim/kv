# Health

The `kv health` command checks whether KoalaVim's dependencies are available and reports their versions.

## Usage

```bash
kv health                     # check health for "main" env
kv --env myenv health         # check health for a specific env
```

## What Gets Checked

### Core

| Tool | How It's Checked |
|---|---|
| nvim | `nvim --version` -- parses version and commit |
| git | `git --version` |

### Dependencies

| Tool | How It's Checked |
|---|---|
| ripgrep (rg) | `rg --version` |
| fd | `fd --version` |
| fzf | `fzf --version` |
| curl | `curl --version` |

### Optional

| Tool | How It's Checked |
|---|---|
| nerd font | `fc-list` output scanned for "Nerd Font" (unix only) |

## Env-Aware Resolution

Health checks look for binaries in the env's bin directory first (`<data_dir>/kv/bin/`), then fall back to the system `PATH`. This means `kv health` reflects what `kv` (and nvim) will actually use when launched.

## Output

Each check shows either:
- `OK` with the version number (green)
- `MISSING` with the reason (red)

```
KoalaVim Health Check (env: main)

  core:
    OK   nvim                 0.10.0
    OK   git                  2.43.0
  dependencies:
    OK   ripgrep (rg)         14.1.0
    OK   fd                   9.0.0
    OK   fzf                  0.46.0
    OK   curl                 8.5.0
  optional:
    OK   nerd font            installed
```
