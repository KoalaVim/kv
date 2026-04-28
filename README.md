# kv

CLI tool to launch and manage [KoalaVim](https://github.com/KoalaVim/KoalaVim) environments.

`kv` is a single Rust binary that handles launching KoalaVim in different modes, managing isolated virtual environments, keeping plugins in sync via lockfiles, updating KoalaVim, installing dependencies, and running health checks.

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

### Building a wheel

To build a redistributable wheel (e.g. to publish or install offline):

```bash
uvx maturin build --release
# wheel is written to target/wheels/
pip install target/wheels/kv-*.whl
```

## Try Without Installing

```bash
docker build -t kv .
docker run -it --rm kv
```

See [docs/docker.md](docs/docker.md) for mounting configs, persisting state, and testing features.

## Quick Start

```bash
# Create the main env with the KoalaConfig starter
kv env create main --from https://github.com/KoalaVim/KoalaConfig.template

# Launch KoalaVim
kv 
```

Or use the interactive wizard:

```bash
kv init
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
