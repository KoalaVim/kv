# Launcher

The default behavior of `kv` (no subcommand) is to launch KoalaVim via neovim.

## Basic Usage

```bash
kv                        # launch KoalaVim in the default "main" env
kv file.txt               # open a file
kv file1.txt file2.txt    # open multiple files
kv -- -u NONE             # pass flags to nvim after --
```

## Modes

KoalaVim supports several launch modes. Only one mode can be active at a time.

```bash
kv -g                     # git mode
kv -t                     # git tree mode
kv --git-diff             # git diff mode
kv --ai                   # ai mode
```

In mode launches, positional arguments are passed to KoalaVim (via `KOALA_ARGS`) rather than directly to nvim:

```bash
kv -g -- file1 file2      # git mode with args passed to KoalaVim
```

## Virtual Envs

Use `--env` to launch in a specific virtual koala env:

```bash
kv --env experiment       # launch using the "experiment" env
```

If no `--env` is specified, `kv` uses the `main` env. If `main` doesn't exist yet, `kv` prints setup instructions and exits.

Per-env binaries installed via `kv install` are automatically prepended to `PATH` when launching, so nvim and its subprocesses use the env's tools.

## Debug

```bash
kv -d                     # debug mode, logs to /tmp/kvim/<timestamp>
kv -d --debug-file my-log # custom debug file name
kv -d --debug-dir /path   # custom debug directory
kv -n                     # disable noice (notifications)
```

## Other Options

```bash
kv -c /path/to/kvim.conf          # launch with custom kvim.conf
kv -l /path/to/config.lua         # launch with custom lua config
kv --nvim-bin-path /path/to/nvim  # override nvim binary
kv -v                             # verbose output (prints env vars, paths)
```

## Restart Loop

KoalaVim can request a restart by creating a `restart_kvim` file in the env's data directory. When `kv` detects this file after nvim exits, it removes the file and relaunches nvim with `KOALA_RESTART=1` set.

## Environment Variables Set by kv

| Variable | Description |
|---|---|
| `NVIM_APPNAME` | Set to `kvim-envs/<env>` for env isolation |
| `KOALA_KVIM_CONF` | Path to kvim.conf |
| `KOALA_DEBUG_OUT` | Debug output file (when `-d` is used) |
| `KOALA_NO_NOICE` | Set to `1` when `-n` is used |
| `KOALA_NO_SESSION` | Set to `1` in mode launches |
| `KOALA_MODE` | The active mode name (git, git_tree, git_diff, ai) |
| `KOALA_ARGS` | Joined positional args (in mode launches) |
| `KOALA_RESTART` | Set to `1` on restart iterations |
| `PATH` | Prepended with env's bin dir if it exists |
