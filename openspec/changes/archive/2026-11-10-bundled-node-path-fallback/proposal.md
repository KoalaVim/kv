## Why

A repo pins Node 26.12.1 (`.nvmrc`, installed through mise). In a terminal opened inside a `kv`-started Neovim, `node -v` prints `v22.16.0`, and a repo check fails because Node 22's experimental `fs.globSync` matches nothing under dot-directories (`**/.claude`, `**/.gitkeep`). The same check passes on Node 26.

The cause: `build_koala_env` (`src/launcher.rs:114-134`) builds `PATH = <env bin> : <env node/bin> : <user PATH>`. Neovim and everything it starts (terminals, mason, AI agents run via `kv --ai`) inherit that PATH, so the bundled Node v22.16.0 (`src/install.rs:75-87`) shadows whatever Node the user's version manager picks.

The bundled Node was added in `a876b78` ("add Node.js as managed dependency for Mason LSP installs"): mason needs `npm` to install Node-based LSP servers (jsonls, vtsls, pyright, bashls). That must keep working on a machine with no Node.

## What Changes

- Move the bundled Node directory from right after `<env bin>` to the end of PATH: `PATH = <env bin> : <user PATH> : <env node/bin>`. With no `node/bin/`, `<env node>` itself is used, as today.
- `<env bin>` stays first. Nothing else about PATH changes: PATH is still only set when `<env bin>` exists.
- **BREAKING (behaviour)**: When the user's PATH already has a `node`/`npm`, Neovim plugins, mason installs and Node-based LSP servers now run on that Node instead of the bundled v22.16.0. A project pinned to an old Node (e.g. 16) could break a plugin that needs 20+. Accepted; if it shows up, point that plugin at the bundled Node explicitly (Copilot: `copilot_node_command`). Not added up front.

## Capabilities

### New Capabilities
- `launch-env-path`: the PATH that `kv` gives the launched Neovim process: order of the env `bin` directory, the user's PATH, and the bundled Node.

### Modified Capabilities
<!-- none: there are no main specs under openspec/specs/ yet -->

## Impact

- Code: `src/launcher.rs`, `build_koala_env` only, plus a test in its `mod tests`.
- Not affected: `kv health` still checks the bundled Node first (`src/health.rs:25`); install and the install manifest are unchanged.
- Users: inside a `kv` session, `node`, `npm`, `npx`, `pnpm` resolve to the user's own Node when one is on PATH; the bundled Node is only a fallback.
