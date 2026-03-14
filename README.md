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

Run multiple isolated Neovim configurations side by side. Each env gets its own config, data, state, and cache.

```bash
kv init                        # interactive setup wizard
kv env create lazyvim --from https://github.com/LazyVim/starter
kv --env lazyvim               # launch in a specific env
kv env fork main experiment    # full copy of an existing env
kv env list                    # see all envs and disk usage
```

See [docs/envs.md](docs/envs.md) for the full guide with real-world examples.

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
