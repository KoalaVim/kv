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
