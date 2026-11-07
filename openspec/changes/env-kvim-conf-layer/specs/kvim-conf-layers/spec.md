## Purpose

Defines how `.kvim.conf` layers (defaults, user, env, repo) are found, passed from `kv` to KoalaVim, and merged, so that per-env settings can override user settings while per-repo settings still win.

## ADDED Requirements

### Requirement: Env conf location
Each koala env SHALL have one env conf path: the file `.kvim.conf` directly inside that env's config directory (for example `~/.config/kvim-envs/<name>/.kvim.conf` on Linux and macOS, and `%LOCALAPPDATA%\kvim-envs\<name>\.kvim.conf` on Windows). The path SHALL follow the same XDG and platform rules as the env config directory.

#### Scenario: Default XDG location
- **WHEN** `XDG_CONFIG_HOME` is unset on Linux or macOS and the env is `work`
- **THEN** the env conf path is `~/.config/kvim-envs/work/.kvim.conf`

#### Scenario: Custom XDG_CONFIG_HOME
- **WHEN** `XDG_CONFIG_HOME=/x/cfg` and the env is `main`
- **THEN** the env conf path is `/x/cfg/kvim-envs/main/.kvim.conf`

### Requirement: kv passes env conf path
When launching KoalaVim, `kv` SHALL set the environment variable `KOALA_ENV_KVIM_CONF` to the absolute env conf path of the resolved env. It SHALL set the variable whether or not the file exists, and SHALL keep it set on restart relaunches.

#### Scenario: Launch with named env
- **WHEN** the user runs `kv --env work`
- **THEN** the nvim process environment contains `KOALA_ENV_KVIM_CONF=<env config dir of work>/.kvim.conf`

#### Scenario: Env conf file does not exist
- **WHEN** the user runs `kv` and `<env config dir>/.kvim.conf` does not exist
- **THEN** `KOALA_ENV_KVIM_CONF` is still set to that path
- **AND** `kv` does not create the file

#### Scenario: Restart keeps env conf
- **WHEN** KoalaVim requests a restart through the restart indicator
- **THEN** the relaunched nvim process receives the same `KOALA_ENV_KVIM_CONF` value

#### Scenario: Verbose output
- **WHEN** the user runs `kv -v`
- **THEN** the printed environment includes `KOALA_ENV_KVIM_CONF`

### Requirement: User conf passing unchanged
`kv` SHALL keep setting `KOALA_KVIM_CONF` to the user conf path (`~/.kvim.conf` by default, or the value of `-c/--cfg`). The `-c/--cfg` flag SHALL affect only the user layer, not the env layer.

#### Scenario: Custom user conf with env layer
- **WHEN** the user runs `kv -c /tmp/my.conf --env work`
- **THEN** `KOALA_KVIM_CONF=/tmp/my.conf`
- **AND** `KOALA_ENV_KVIM_CONF` points at the `work` env conf

### Requirement: Layer merge order
KoalaVim SHALL build its effective configuration by deep-merging the layers in this order, with each later layer overriding keys set by earlier ones:
1. KoalaVim defaults
2. user conf
3. env conf
4. repo conf

Keys that a higher layer does not set SHALL keep the value from the nearest lower layer that sets them.

#### Scenario: Env overrides user
- **WHEN** user conf sets `ai.mux_split = false` and env conf sets `ai.mux_split = true`, and no repo conf sets it
- **THEN** the effective `ai.mux_split` is `true`

#### Scenario: Repo overrides env
- **WHEN** env conf sets `editor.indent.tab_size = 2` and repo conf sets `editor.indent.tab_size = 8`
- **THEN** the effective `editor.indent.tab_size` is `8`

#### Scenario: Deep merge keeps sibling keys
- **WHEN** user conf sets `lsp.a = 1` and `lsp.b = 1`, and env conf sets only `lsp.b = 2`
- **THEN** the effective `lsp.a` is `1` and `lsp.b` is `2`

### Requirement: Env layer is optional
A missing, unset, or empty env conf SHALL contribute nothing to the merge. KoalaVim SHALL NOT create the env conf file during config loading. If the env conf contains invalid JSON, KoalaVim SHALL report a warning and continue as if the env layer were empty, the same way it handles other layers.

#### Scenario: Launched without kv
- **WHEN** KoalaVim starts and `KOALA_ENV_KVIM_CONF` is unset
- **THEN** the effective config equals the merge of defaults, user conf and repo conf, as it was before this change

#### Scenario: Env conf file missing
- **WHEN** `KOALA_ENV_KVIM_CONF` points at a path that does not exist
- **THEN** config loading succeeds with no env layer
- **AND** no file is created at that path

#### Scenario: Invalid env conf JSON
- **WHEN** the env conf file contains malformed JSON
- **THEN** a health warning names the file
- **AND** the remaining layers still apply

### Requirement: Editing the env conf from KoalaVim
KoalaVim SHALL offer a dashboard action, "Koala Env Config", that opens the env conf, creating it with `{}` first if it does not exist. When `KOALA_ENV_KVIM_CONF` is unset, the action SHALL notify the user that no kv env is active and SHALL NOT create any file. Env conf buffers SHALL get the same JSON filetype and schema validation as other `.kvim.conf` files.

#### Scenario: Open missing env conf
- **WHEN** the user picks "Koala Env Config" and the env conf file does not exist
- **THEN** the file is created containing `{}` and opened in a buffer

#### Scenario: No kv env
- **WHEN** the user picks "Koala Env Config" and `KOALA_ENV_KVIM_CONF` is unset
- **THEN** a notification says no kv env is active and no file is created

### Requirement: Editing the env conf from kv
`kv` SHALL provide the subcommand `kv env config [NAME] [--path]`.
- `NAME` is optional. When it is omitted, the env SHALL be resolved the same way a launch resolves it: the `--env` flag, then `KV_ENV`, then `main`.
- If the resolved env does not exist, the command SHALL fail with an error and a non-zero exit, and SHALL NOT create any file.
- Without `--path`, the command SHALL create the env conf containing `{}` if it does not exist, then open it in KoalaVim running in that env. That launch SHALL get the same environment as a normal `kv` launch for that env.
- With `--path`, the command SHALL print the absolute env conf path to stdout and exit 0, and SHALL NOT create or open the file.

#### Scenario: Open existing env conf
- **WHEN** env `work` has a `.kvim.conf` and the user runs `kv env config work`
- **THEN** KoalaVim launches with `NVIM_APPNAME=kvim-envs/work` and the file open
- **AND** the file content is unchanged

#### Scenario: Create missing env conf
- **WHEN** env `work` has no `.kvim.conf` and the user runs `kv env config work`
- **THEN** `<env config dir of work>/.kvim.conf` is created containing `{}`
- **AND** KoalaVim opens it in env `work`

#### Scenario: Default env resolution
- **WHEN** `KV_ENV` is unset and the user runs `kv env config`
- **THEN** the command operates on env `main`

#### Scenario: Unknown env
- **WHEN** the user runs `kv env config nope` and env `nope` does not exist
- **THEN** an `error:` message says the env does not exist, the exit code is non-zero, and no file is created

#### Scenario: Print path only
- **WHEN** the user runs `kv env config work --path` and the file does not exist
- **THEN** stdout contains exactly the absolute env conf path and a newline
- **AND** the file is not created and nvim is not launched

### Requirement: Env conf follows env lifecycle
Because the env conf lives inside the env config directory, `kv env fork` SHALL copy it to the new env, `kv env rename` SHALL move it with the env, and `kv env delete` SHALL remove it.

#### Scenario: Fork copies env conf
- **WHEN** env `main` has a `.kvim.conf` in its config dir and the user runs `kv env fork main exp`
- **THEN** env `exp` has an identical `.kvim.conf` in its config dir
