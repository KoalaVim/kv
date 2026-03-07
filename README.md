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
kv -g              # git mode
kv -t              # git tree mode
kv --git-diff      # git diff mode
kv --ai            # ai mode
```

### Virtual Koala Envs

Manage isolated KoalaVim environments with separate config, data, state, and cache directories (following XDG conventions).

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

# Launch KoalaVim in an env
kv --env my-env

# Delete an env
kv env delete my-env
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
kv file.txt +42                # pass arguments to nvim
```
