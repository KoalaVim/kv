# KoalaVim's launcher

CLI tool to launch [KoalaVim](https://github.com/KoalaVim/KoalaVim)

## Installation

### kv
1. Make sure [Cargo](https://www.rust-lang.org/tools/install) is installed properly (`~/.cargo/bin` should be in your `PATH`).
2. Install (fetch & build) the tool.
```bash
CARGO_NET_GIT_FETCH_WITH_CLI=true cargo install --locked --git=https://github.com/KoalaVim/kv.git
```

```bash
# Install locally
git clone https://github.com/KoalaVim/kv.git
cd kv
cargo install --locked --path .
```

## Usage
```bash
kv --help
```

### Modes

Launch KoalaVim in different modes:

```bash
kv -g                  # git mode
kv -t                  # git tree mode
kv --git-diff          # git diff mode
kv --ai                # ai mode
kv -g -- file1 file2   # git mode with args passed to KoalaVim
```

### Virtual Koala Envs

Manage isolated KoalaVim environments with separate config, data, state, and cache directories (following XDG conventions).

Each env uses `NVIM_APPNAME` set to `kvim-envs/<name>`, which makes Neovim resolve its directories under the XDG base paths:

```
~/.config/kvim-envs/<name>/     # config (init.lua, plugins, etc.)
~/.local/share/kvim-envs/<name>/  # data (installed plugins, etc.)
~/.local/state/kvim-envs/<name>/  # state (shada, logs, etc.)
~/.cache/kvim-envs/<name>/      # cache (compiled bytecode, etc.)
```

By default, `kv` launches using the `main` env. Use `--env` to switch:

```bash
kv                  # launches in the "main" env
kv --env my-env     # launches in "my-env"
```

#### Setup

```bash
kv init              # interactive setup wizard for the default "main" env
kv init --env foo    # interactive setup for a named env
```

#### Managing Envs

```bash
# Create a new env
kv env create my-env

# Create from an existing env
kv env create new-env --from my-env

# Create from a git URL
kv env create lazyvim --from https://github.com/LazyVim/starter
kv env create lazyvim --from https://github.com/LazyVim/starter --branch stable

# Create from a local path
kv env create custom --from /path/to/nvim/config

# List all envs
kv env list

# Fork an env (copies config, data, state, and cache)
kv env fork my-env my-env-copy

# Rename an env
kv env rename old-name new-name

# Delete an env
kv env delete my-env
```

### Shell Completions

```bash
kv completions zsh    # generate zsh completions
kv completions bash   # generate bash completions
kv completions fish   # generate fish completions
```

### Debug

```bash
kv -d                          # debug mode, logs to --debug-dir/<timestamp>
kv -d --debug-file my-log      # custom debug file name
kv -n                          # disable noice (notifications)
```

### Other Options

```bash
kv -c /path/to/kvim.conf       # launch with custom kvim.conf
kv -l /path/to/config.lua      # launch with custom lua config
kv --nvim-bin-path /path/to/nvim  # override nvim binary
kv -v                          # verbose output
kv -- file.txt +42             # pass arguments to nvim
```
