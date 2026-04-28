# KoalaVim's launcher

CLI tool to launch [KoalaVim](https://github.com/KoalaVim/KoalaVim)

## Installation

`kv` is a Rust binary. It can be installed with `cargo`, or with `pip` / `uv`
(packaged via [maturin](https://www.maturin.rs/)). All methods require a Rust
toolchain to build the binary.

### With `uv` (recommended)

```bash
# From Git
uv tool install git+https://github.com/KoalaVim/kv.git

# From a local checkout
git clone https://github.com/KoalaVim/kv.git
cd kv
uv tool install .
```

### With `pip`

```bash
# From Git
pip install git+https://github.com/KoalaVim/kv.git

# From a local checkout
git clone https://github.com/KoalaVim/kv.git
cd kv
pip install .
```

### With `cargo`

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

### Building a wheel

To build a redistributable wheel (e.g. to publish or install offline):

```bash
uvx maturin build --release
# wheel is written to target/wheels/
pip install target/wheels/kv-*.whl
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
kv env create main --from https://github.com/KoalaVim/KoalaConfig.template
kv                             # launch KoalaVim ("main")
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
