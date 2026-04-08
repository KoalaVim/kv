# Docker

The Docker image lets you try KoalaVim instantly or test `kv` features in an isolated environment -- no local installation required.

## Quick Start

Build the image:

```bash
docker build -t kv .
```

Launch KoalaVim:

```bash
docker run -it --rm kv
```

That's it. You get a fully working KoalaVim session inside the container.

## What's Inside

The image includes:

| Tool | Purpose |
|---|---|
| neovim (stable) | Editor |
| kv | KoalaVim launcher and manager |
| git | Version control, used by kv update |
| curl | Downloads, used by kv install |
| ripgrep (rg) | File content search |
| fd | File finder |
| fzf | Fuzzy finder |

A `main` env is pre-created from the [KoalaConfig.template](https://github.com/KoalaVim/KoalaConfig.template) with plugins pre-installed via lazy.nvim.

## Testing kv Features

Run any `kv` subcommand by passing it as arguments:

```bash
docker run -it --rm kv health              # check dependency health
docker run -it --rm kv lockfile diff       # diff lockfiles
docker run -it --rm kv install --dry-run   # see what install would do
docker run -it --rm kv env list            # list environments
```

Drop into a shell to explore freely:

```bash
docker run -it --rm --entrypoint bash kv
```

From there you can run any combination of commands:

```bash
kv health
kv lockfile diff
kv update --no-restore
kv install --dry-run
kv env create experiment --from main
kv --env experiment health
```

## Mounting Your Config

Try KoalaVim with your own neovim config:

```bash
docker run -it --rm \
    -v "$HOME/.config/nvim":/home/koala/.config/kvim-envs/main \
    kv
```

## Mounting a Working Directory

Edit files from your host inside the container:

```bash
docker run -it --rm \
    -v "$(pwd)":/home/koala/project \
    kv project/
```

## Persisting State

Plugins and data are lost when the container is removed. To persist across runs, mount the data directory:

```bash
docker run -it --rm \
    -v kv-data:/home/koala/.local/share/kvim-envs \
    -v kv-cache:/home/koala/.cache/kvim-envs \
    kv
```

## Multi-Arch Support

The Dockerfile supports both `x86_64` (amd64) and `aarch64` (arm64) architectures. Build for your platform or use buildx for cross-platform:

```bash
docker buildx build --platform linux/amd64,linux/arm64 -t kv .
```

## Overriding Neovim Version

Build with a specific neovim version:

```bash
docker build --build-arg NVIM_VERSION=v0.10.0 -t kv .
```

## Image Structure

```
/usr/local/bin/kv          # kv binary
/usr/local/bin/nvim        # symlink to /opt/nvim-linux-*/bin/nvim
/home/koala/               # non-root user home
  .config/kvim-envs/main/  # KoalaVim config (from template)
  .local/share/kvim-envs/main/
    lazy/                  # lazy.nvim plugins (pre-installed)
    kv/                    # kv data (install manifest, lockfile backups)
```
