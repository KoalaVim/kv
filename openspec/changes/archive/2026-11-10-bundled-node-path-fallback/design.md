## Context

See proposal.md (Why). Today, `src/launcher.rs:114-134`:

```rust
let mut new_path = OsString::from(&bin_dir);   // <env bin>
// push <env node>/bin, or <env node> if there is no bin/
// push current_path
```

PATH is only set when `<env bin>` exists. No test covers the PATH value yet; `test_build_koala_env_sets_env_kvim_conf` (`src/launcher.rs:341`) shows how to call `build_koala_env` with a temp XDG dir under `#[serial]`.

## Goals / Non-Goals

**Goals:**
- The user's own `node` wins inside a `kv` session; the bundled one is only a fallback.

**Non-Goals:**
- Giving Neovim plugins (Copilot, mason, LSPs) an explicit path to the bundled Node. Add that per plugin only if a real breakage shows up.
- Changing `kv health` (it checks the bundled Node directly, `src/health.rs:25`), install, or the bundled Node version.
- Removing duplicate PATH entries.

## Decisions

**Append the bundled Node after the user's PATH.** The new order is `<env bin> : <user PATH> : <env node>/bin` (or `<env node>`). In `build_koala_env`, move the existing Node block below the `current_path` push and keep the `bin/`-or-dir fallback:

```rust
if !current_path.is_empty() {
    new_path.push(path_sep);
    new_path.push(&current_path);
}
let node_dir = env_node_dir(env_name);
let node_bin = node_dir.join("bin");
let node_path = if node_bin.exists() { node_bin } else { node_dir };
if node_path.exists() {
    new_path.push(path_sep);
    new_path.push(&node_path);
}
```

It works with any Node manager, because the first `node` on PATH wins. mise, nvm, fnm, volta and asdf all put their Node on the PATH that `kv` inherits. The shim-based ones (mise shims, volta, asdf) still pick a Node per directory inside Neovim.

Alternatives considered:
- *Stop adding the bundled Node to PATH at all.* That breaks mason on a machine without Node, which is why `a876b78` added it.
- *Add a `.kvim.conf`/CLI option to choose the order.* That's more surface for a problem the fallback order already solves.
- *Detect whether `node` is on PATH and add the bundled one only if it's missing.* Putting it last does the same thing with no lookup.

**Keep `<env bin>` first.** It holds the env's own tools (nvim, rg, fd, …). Their order isn't part of this bug.

## Risks / Trade-offs

- [A plugin that needs a newer Node runs on an old project-pinned Node (e.g. 16)] → Accepted. If it happens, set that plugin's Node path to the bundled one (Copilot: `copilot_node_command`).
- [Mason installs Node-based LSPs with the user's `npm`, and the servers then run on the user's Node] → Same cost and the same mitigation as above.
- [Shells opened inside Neovim: mise/nvm may not move their Node back to the front because they see it already on PATH] → This change removes the bundled Node from the front, so there is nothing ahead of theirs. (Inferred, not tested.)
