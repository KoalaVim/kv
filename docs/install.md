# Install

The `kv install` command installs KoalaVim's dependencies into the current env's binary directory. Each env gets its own isolated set of tools.

## Usage

```bash
kv install                    # install all dependencies for "main" env
kv --env myenv install        # install for a specific env
kv install --dry-run          # show what would be installed without doing it
```

## What Gets Installed

| Tool | GitHub Repo | Default Version |
|---|---|---|
| neovim | `neovim/neovim` | `stable` |
| ripgrep | `BurntSushi/ripgrep` | `latest` |
| fd | `sharkdp/fd` | `latest` |
| fzf | `junegunn/fzf` | `latest` |

## Install Flow

1. **Detect platform** -- determines OS (Linux, macOS, Windows) and architecture (x86_64, aarch64).
2. **For each dependency**:
   - Query the GitHub Releases API for the matching asset URL
   - Download the archive to a temp directory via `curl`
   - Extract the archive (`tar` for `.tar.gz`, `unzip` for `.zip`)
   - Locate the binary within the extracted files
   - Copy the binary to the env's bin directory (`<data_dir>/kv/bin/`)
   - Set executable permissions (unix)
3. **Write manifest** -- records installed versions in `<data_dir>/kv/install-manifest.json`
4. **Clean up** -- removes temp download directory

## Per-Env Isolation

Binaries are installed to:

```
<data_dir>/kv/bin/
```

For the default `main` env on Linux, this resolves to:

```
~/.local/share/kvim-envs/main/kv/bin/
```

When `kv` launches nvim, this directory is prepended to `PATH`, so nvim and any subprocesses (LSP servers, formatters) automatically use the env's tools.

## Install Manifest

The manifest at `<data_dir>/kv/install-manifest.json` tracks what was installed:

```json
{
  "installed": {
    "neovim": {
      "version": "v0.10.0",
      "asset_url": "https://github.com/neovim/neovim/releases/download/...",
      "installed_at": "2024-01-15T10:30:00+00:00"
    }
  }
}
```

## Requirements

- `curl` -- used for downloading assets and querying the GitHub API
- `tar` -- for extracting `.tar.gz` archives
- `unzip` -- for extracting `.zip` archives

Run `kv health` to verify these tools are available.

## Dry Run

Use `--dry-run` to see what would be installed without making any changes:

```bash
kv install --dry-run
```

This prints the version, asset pattern, and destination directory for each dependency.

## Platform Support

| Platform | Architecture | Status |
|---|---|---|
| Linux | x86_64 | Supported |
| Linux | aarch64 | Supported |
| macOS | x86_64 | Supported |
| macOS | aarch64 (Apple Silicon) | Supported |
| Windows | x86_64 | Supported |
