# KoalaVim's launcher

CLI tool to launch [KoalaVim](https://github.com/KoalaVim/KoalaVim)

## Installation
### Cargo
To install this tool you need `cargo`, cargo is Rust's build system and package manager.

You can find the installation instructions [here](https://www.rust-lang.org/tools/install) or just run this command:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### kv
1. Make sure `~/.cargo/bin` is in your `PATH`.
2. Install (fetch & build) the tool.
```bash
CARGO_NET_GIT_FETCH_WITH_CLI=true cargo install --git=ssh://git@github.com/KoalaVim/kv.git
```

## Usage
```bash
kv --help
```
