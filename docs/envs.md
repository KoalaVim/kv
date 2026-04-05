# Virtual Koala Envs

Virtual Koala Envs let you run multiple isolated Neovim configurations side by side.
Each env gets its own config, data, state, and cache — so you can try new setups, test plugins, or keep separate workflows without touching your main configuration.

## Quick Start

The fastest way to get started:

```bash
kv init
```

This walks you through creating the default `main` env interactively — you can start clean, copy an existing config, or clone a git template.

Then just run:

```bash
kv
```

## How It Works

Each env maps to a set of directories scoped by name, matching Neovim's own `stdpath()` resolution. Under the hood, `kv` sets `NVIM_APPNAME=kvim-envs/<name>`, which tells Neovim to use these directories instead of the default `nvim` ones.

### Linux / macOS

Neovim follows the XDG Base Directory Specification on both Linux and macOS:

```
~/.config/kvim-envs/<name>/       config (init.lua, plugins, etc.)
~/.local/share/kvim-envs/<name>/  data (installed plugins)
~/.local/state/kvim-envs/<name>/  state (shada, logs)
~/.cache/kvim-envs/<name>/        cache (compiled bytecode)
```

XDG environment variables (`XDG_CONFIG_HOME`, `XDG_DATA_HOME`, `XDG_STATE_HOME`, `XDG_CACHE_HOME`) are respected if set.

### Windows

```
%LOCALAPPDATA%\kvim-envs\<name>\        config
%LOCALAPPDATA%\kvim-envs\<name>-data\   data
%LOCALAPPDATA%\kvim-envs\<name>-data\   state (same as data)
%TEMP%\kvim-envs\<name>\                cache
```

The `-data` suffix on data and state matches Neovim's own Windows behavior, which disambiguates them from config since they share the same `%LOCALAPPDATA%` base. XDG environment variables take precedence if set.

## Creating Envs

### Empty env

Start from scratch with a blank config directory:

```bash
kv env create my-env
```

Then populate `~/.config/kvim-envs/my-env/init.lua` with your config.

### From an existing env

Copy the config from an env you already have:

```bash
kv env create experiment --from main
```

This copies only the **config** directory. Data, state, and cache start empty — plugins will be installed fresh on first launch. Useful when you want to test config changes without risk.

### From your existing Neovim config

Already have a working `~/.config/nvim`? Bring it in:

```bash
kv env create main --from ~/.config/nvim
```

### From a git template

Clone the [KoalaVim configuration template](https://github.com/KoalaVim/KoalaConfig.template) to get started with a recommended setup:

```bash
kv env create main --from https://github.com/KoalaVim/KoalaConfig.template
kv
```

Or try a community Neovim distribution without affecting anything else:

```bash
kv env create lazyvim --from https://github.com/LazyVim/starter
kv env create kickstart --from https://github.com/nvim-lua/kickstart.nvim
kv env create nvchad --from https://github.com/NvChad/starter
```

Pin to a specific branch or tag:

```bash
kv env create lazyvim --from https://github.com/LazyVim/starter --branch stable
```

The `.git/` directory is removed after cloning so the env starts as a clean config you own.

## Forking Envs

Fork creates a **full copy** — config, data, state, and cache — so the new env is immediately usable without reinstalling plugins:

```bash
kv env fork main tweaked
```

| | config | data | state | cache |
|---|---|---|---|---|
| `create --from` | copied | empty | empty | empty |
| `fork` | copied | copied | copied | copied |

**When to use which:**

- `create --from` — you want a fresh start with someone's config (plugins reinstall on first launch)
- `fork` — you want an exact duplicate that's ready to use instantly (e.g., experimenting with changes to a working setup)

## Switching Between Envs

By default `kv` uses the `main` env. Switch with `--env`:

```bash
kv                    # launches "main"
kv --env lazyvim      # launches "lazyvim"
kv --env experiment   # launches "experiment"
```

## Listing Envs

See all envs and their disk usage:

```bash
kv env list
```

Example output:

```
  main [142.3M] (default)
    config: ~/.config/kvim-envs/main (12.1K)
    data: ~/.local/share/kvim-envs/main (140.2M)
    state: ~/.local/state/kvim-envs/main (1.8M)
    cache: ~/.cache/kvim-envs/main (256.0K)
  lazyvim [98.7M]
    config: ~/.config/kvim-envs/lazyvim (8.4K)
    data: ~/.local/share/kvim-envs/lazyvim (97.1M)
    state: ~/.local/state/kvim-envs/lazyvim (1.5M)
    cache: ~/.cache/kvim-envs/lazyvim (128.0K)

2 env(s) found.
```

## Renaming Envs

```bash
kv env rename old-name new-name
```

This moves all four XDG directories atomically.

## Deleting Envs

```bash
kv env delete lazyvim
```

You'll be asked to confirm. All four directories (config, data, state, cache) are removed. To skip the prompt:

```bash
kv env delete lazyvim -f
```

## Real-World Examples

### Set up KoalaVim from the official template

```bash
kv env create main --from https://github.com/KoalaVim/KoalaConfig.template
kv   # launches KoalaVim with the template config, plugins install on first run
```

### Try LazyVim without affecting your setup

```bash
kv env create lazyvim --from https://github.com/LazyVim/starter
kv --env lazyvim
# don't like it?
kv env delete lazyvim -f
```

### Experiment with plugin changes safely

```bash
kv env fork main experiment
kv --env experiment
# edit ~/.config/kvim-envs/experiment/init.lua freely
# if it breaks, just delete and re-fork
kv env delete experiment -f
kv env fork main experiment
```

### Keep separate configs for different projects

```bash
kv env create work --from ~/.config/nvim-work
kv env create personal --from ~/dotfiles/nvim
kv --env work
kv --env personal
```

### Compare two Neovim distributions

```bash
kv env create lazyvim --from https://github.com/LazyVim/starter
kv env create kickstart --from https://github.com/nvim-lua/kickstart.nvim
kv --env lazyvim     # try one
kv --env kickstart   # try the other
```

### Migrate your existing config into kv

```bash
kv env create main --from ~/.config/nvim
kv   # launches with your config, plugins install on first run
```
