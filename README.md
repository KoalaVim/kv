# kv

CLI tool to launch and manage [KoalaVim](https://github.com/KoalaVim/KoalaVim) environments.

`kv` is a single Rust binary that handles launching KoalaVim in different modes, managing isolated virtual environments, keeping plugins in sync via lockfiles, updating KoalaVim, installing dependencies, and running health checks.

## Installation

Make sure [Cargo](https://www.rust-lang.org/tools/install) is installed (`~/.cargo/bin` should be in your `PATH`).

```bash
CARGO_NET_GIT_FETCH_WITH_CLI=true cargo install --locked --git=https://github.com/KoalaVim/kv.git
```

Or build locally:

```bash
git clone https://github.com/KoalaVim/kv.git
cd kv
cargo install --locked --path .
```

## Quick Start

```bash
kv init              # interactive setup wizard for the default "main" env
kv                   # launch KoalaVim
```

## Commands

| Command | Description | Docs |
|---|---|---|
| `kv [files...]` | Launch KoalaVim (default) | [docs/launcher.md](docs/launcher.md) |
| `kv env <action>` | Manage virtual koala envs | [docs/envs.md](docs/envs.md) |
| `kv lockfile <action>` | Manage the lazy.nvim lockfile | [docs/lockfile.md](docs/lockfile.md) |
| `kv update` | Update KoalaVim to a target version | [docs/update.md](docs/update.md) |
| `kv install` | Install dependencies into the env | [docs/install.md](docs/install.md) |
| `kv health` | Check health of dependencies | [docs/health.md](docs/health.md) |
| `kv init` | Interactive env setup wizard | [docs/envs.md](docs/envs.md) |
| `kv completions <shell>` | Generate shell completions | -- |

All commands respect the `--env` flag to operate on a specific virtual koala env (default: `main`).

```bash
kv --env myenv              # launch in "myenv"
kv --env myenv health       # check health for "myenv"
kv --env myenv install      # install deps into "myenv"
```

## Shell Completions

```bash
kv completions zsh     # generate zsh completions
kv completions bash    # generate bash completions
kv completions fish    # generate fish completions
```

## Platform Support

- Linux
- macOS
- Windows
