# launch-env-path Specification

## Purpose

Defines the PATH that `kv` gives the launched Neovim process, so that env-managed tools are found first while the user's own tools (notably their Node) win over the bundled Node, which stays available as a fallback.

## Requirements

### Requirement: PATH order for the launched Neovim
When the env `bin` directory exists, `kv` SHALL set the launched Neovim's PATH to, in this order: the env `bin` directory, then the PATH `kv` was started with (if non-empty), then the bundled Node directory (if it exists). The bundled Node directory SHALL be `<env node>/bin` when that exists, otherwise `<env node>` itself. Entries SHALL be joined with `:` on Linux and macOS and `;` on Windows. When the env `bin` directory does not exist, `kv` SHALL NOT set PATH.

#### Scenario: User PATH comes before the bundled Node
- **WHEN** `kv` is started with `PATH=/u/a:/u/b` and the env has both `bin/` and `node/bin/`
- **THEN** the Neovim process gets `PATH=<env bin>:/u/a:/u/b:<env node>/bin`

#### Scenario: Bundled Node without a bin subdirectory
- **WHEN** the env has `bin/` and `node/` but no `node/bin/` (e.g. the Windows Node zip layout)
- **THEN** PATH ends with `<env node>` itself

#### Scenario: No bundled Node
- **WHEN** the env has `bin/` but no `node/`
- **THEN** PATH is `<env bin>` followed by the user's PATH, with nothing after it

#### Scenario: Empty starting PATH
- **WHEN** `kv` is started with an empty or unset PATH and the env has `bin/` and `node/bin/`
- **THEN** PATH is `<env bin>:<env node>/bin`

#### Scenario: No env bin directory
- **WHEN** the env has no `bin/` directory
- **THEN** `kv` does not set PATH for Neovim, and Neovim inherits the user's PATH unchanged

### Requirement: User's Node wins over the bundled Node
Inside a `kv` session, a `node` (and `npm`, `npx`) on the PATH `kv` was started with SHALL resolve ahead of the bundled one. The bundled Node SHALL be used only when the starting PATH has no `node`.

#### Scenario: Version manager picks the Node
- **WHEN** the user starts `kv` from a shell where mise, nvm, fnm, volta or asdf puts Node 26 on PATH, and opens a terminal inside Neovim
- **THEN** `node -v` in that terminal prints `v26.x`
- **AND** the bundled Node directory is the last PATH entry

#### Scenario: No Node on the machine
- **WHEN** the user starts `kv` with a PATH that has no `node` (e.g. `env PATH=/usr/bin:/bin kv`)
- **THEN** `node` and `npm` inside the session resolve to the bundled Node, and mason can still install Node-based LSP servers
