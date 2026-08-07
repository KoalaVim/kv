use crate::paths::{
    env_appname, env_bin_dir, env_lazy_dir, env_lockfile, env_nvim_runtime_dir, kvim_lockfile,
};
use inquire::Confirm;
use owo_colors::OwoColorize;
use serde_json::Value;
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fs;
use std::path::Path;
use std::process::{Command, Output};

type PluginMap = BTreeMap<String, Value>;

pub fn read_lockfile(path: &Path) -> Result<PluginMap, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read lockfile {}: {}", path.display(), e))?;
    let parsed: PluginMap = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse lockfile {}: {}", path.display(), e))?;
    Ok(parsed)
}

fn plugin_commit(value: &Value) -> Option<&str> {
    value.get("commit").and_then(|v| v.as_str())
}

/// Write a lockfile preserving lazy.nvim's formatting convention.
pub fn write_lockfile(path: &Path, content: &PluginMap) -> Result<(), String> {
    let mut lines = vec!["{".to_string()];
    let entries: Vec<_> = content.iter().collect();
    for (i, (plugin, value)) in entries.iter().enumerate() {
        let json_value = serde_json::to_string(value)
            .map_err(|e| format!("Failed to serialize plugin {}: {}", plugin, e))?;
        let formatted = json_value.replace('{', "{ ").replace('}', " }");
        let comma = if i < entries.len() - 1 { "," } else { "" };
        lines.push(format!("  \"{}\": {}{}", plugin, formatted, comma));
    }
    lines.push("}".to_string());

    let output = lines.join("\n") + "\n";
    fs::write(path, output)
        .map_err(|e| format!("Failed to write lockfile {}: {}", path.display(), e))?;
    Ok(())
}

pub fn cmd_lockfile_diff(env_name: &str) -> Result<(), String> {
    let user_path = env_lockfile(env_name);
    let kvim_path = kvim_lockfile(env_name);

    let user_lock = read_lockfile(&user_path)?;
    let kvim_lock = read_lockfile(&kvim_path)?;

    let mut has_diff = false;

    println!(
        "{:>4} {:<40} {:<16} {}",
        "",
        "Plugin".bold(),
        "User".green(),
        "KoalaVim".cyan()
    );
    println!("{}", "─".repeat(80).dimmed());

    for (plugin, kvim_value) in &kvim_lock {
        if plugin == "KoalaVim" {
            continue;
        }
        let kvim_commit = plugin_commit(kvim_value).unwrap_or("N/A");
        let user_commit = user_lock
            .get(plugin)
            .and_then(plugin_commit)
            .unwrap_or("N/A");

        if kvim_commit != user_commit {
            has_diff = true;
            let short_user = &user_commit[..user_commit.len().min(12)];
            let short_kvim = &kvim_commit[..kvim_commit.len().min(12)];
            println!(
                "{:>4} {:<40} {:<16} {}",
                "",
                plugin,
                short_user.green(),
                short_kvim.cyan()
            );
        }
    }

    if !has_diff {
        println!("{}", "Lockfiles are in sync.".green());
    }

    Ok(())
}

pub fn cmd_lockfile_overwrite(env_name: &str, yes: bool) -> Result<(), String> {
    let user_path = env_lockfile(env_name);
    let kvim_path = kvim_lockfile(env_name);

    if !kvim_path.exists() {
        return Err(format!(
            "KoalaVim lockfile not found at: {}",
            kvim_path.display()
        ));
    }

    if !yes {
        let confirmed = Confirm::new(&format!(
            "Overwrite '{}' with KoalaVim's lockfile?",
            user_path.display()
        ))
        .with_default(false)
        .prompt()
        .map_err(|e| format!("Prompt failed: {}", e))?;
        if !confirmed {
            println!("Aborted.");
            return Ok(());
        }
    }

    overwrite_lockfile(env_name)?;

    println!("{} lockfile overwritten.", "Success:".green().bold());

    lazy_restore(env_name)?;

    Ok(())
}

/// Copy KoalaVim's lockfile over the user's, excluding the KoalaVim entry itself.
pub fn overwrite_lockfile(env_name: &str) -> Result<(), String> {
    let user_path = env_lockfile(env_name);
    let kvim_path = kvim_lockfile(env_name);

    let mut kvim_lock = read_lockfile(&kvim_path)?;
    kvim_lock.remove("KoalaVim");

    write_lockfile(&user_path, &kvim_lock)
}

/// Resolve the nvim binary for an env, preferring the kv-managed one.
fn resolve_nvim(env_name: &str) -> OsString {
    let bin_dir = env_bin_dir(env_name);
    let nvim_bin = bin_dir.join("nvim");
    let nvim_bin_exe = bin_dir.join("nvim.exe");

    if nvim_bin_exe.exists() {
        nvim_bin_exe.into_os_string()
    } else if nvim_bin.exists() {
        nvim_bin.into_os_string()
    } else {
        "nvim".into()
    }
}

/// Run headless nvim for the given env, capturing its output.
fn run_nvim(nvim_cmd: &OsString, env_name: &str, args: &[&str]) -> Result<Output, String> {
    let mut cmd = Command::new(nvim_cmd);
    cmd.args(args).env("NVIM_APPNAME", env_appname(env_name));

    let runtime_dir = env_nvim_runtime_dir(env_name);
    if runtime_dir.exists() {
        cmd.env("VIMRUNTIME", &runtime_dir);
    }

    cmd.output()
        .map_err(|e| format!("Failed to run nvim: {}", e))
}

/// Sync plugin versions to KoalaVim's lockfile via headless nvim.
///
/// This runs in two phases on purpose. lazy.nvim installs missing plugins during
/// startup, and that install ends by rewriting the lockfile — and its in-memory
/// copy of it — from whatever each plugin is currently checked out to. Any
/// restore in that same session then reads those stale commits back, concludes
/// every plugin is already at its target, and skips every checkout silently.
///
/// Splitting the work sidesteps that entirely:
///   1. A plain startup lets lazy install anything missing and settle. New
///      clones land on the right commit (the lockfile is still correct when it
///      reads it); the trailing rewrite is expected and discarded.
///   2. Re-assert the lockfile, then restore. Nothing is missing now, so the
///      install path never runs and the lockfile cache stays trustworthy.
///
/// The verification pass afterwards is not part of the workaround — it exists
/// because a skipped checkout produces no task error, so lazy reporting
/// "success" is not on its own evidence that anything moved.
pub fn lazy_restore(env_name: &str) -> Result<(), String> {
    let nvim_cmd = resolve_nvim(env_name);

    // Phase 1: let lazy install any missing plugins and settle.
    eprintln!(
        "\n {} Installing missing plugins (if any)",
        ">>".yellow().bold(),
    );
    let output = run_nvim(&nvim_cmd, env_name, &["--headless", "+qa"])?;
    if !output.status.success() {
        eprintln!(
            "{} Plugin install phase exited with {}:\n{}",
            "warning:".yellow().bold(),
            output.status,
            String::from_utf8_lossy(&output.stderr).trim(),
        );
    }

    // Phase 2: re-assert the lockfile (phase 1 rewrote it), then restore.
    overwrite_lockfile(env_name)?;

    eprintln!(
        " {} Running {} (sync plugin versions according to lockfile)",
        ">>".yellow().bold(),
        ":Lazy restore".bold(),
    );
    let output = run_nvim(
        &nvim_cmd,
        env_name,
        &["--headless", "+LazyRestoreLogged", "+qa"],
    )?;

    let stderr = String::from_utf8_lossy(&output.stderr);

    // Find the JSON line in stderr — nvim may append extra characters (e.g. ":")
    let json_str = stderr
        .lines()
        .find(|l| l.starts_with('{'))
        .unwrap_or("")
        .trim();
    let val: Value = serde_json::from_str(json_str).map_err(|_| {
        eprintln!(
            "{} Failed to decode lazy restore output: {}",
            "error:".red().bold(),
            stderr
        );
        "Failed to parse :Lazy restore output".to_string()
    })?;

    if let Some(plugins) = val.get("plugins").and_then(|p| p.as_object()) {
        if !plugins.is_empty() {
            eprintln!(
                "{} :Lazy restore finished with errors:",
                "error:".red().bold()
            );
            for (plugin, error) in plugins {
                eprintln!("  {}: {}", plugin.bold(), error);
            }
            return Err(":Lazy restore had plugin errors".to_string());
        }
    }

    verify_restore(env_name)?;

    eprintln!(
        " {} Finished successfully. Restart nvim to take effect.",
        ">>".green().bold()
    );
    Ok(())
}

/// A plugin left checked out at a commit other than the one KoalaVim pins.
#[derive(Debug, PartialEq, Eq)]
struct Drift {
    plugin: String,
    expected: String,
    actual: String,
}

/// Compare two commit hashes, tolerating one being an abbreviation of the other.
fn commit_eq(a: &str, b: &str) -> bool {
    let n = a.len().min(b.len());
    n >= 7 && a[..n].eq_ignore_ascii_case(&b[..n])
}

/// Check every pinned plugin against its actual checked-out commit.
///
/// `head_of` resolves a plugin name to its current HEAD, or `None` when the
/// plugin isn't on disk. Returns drifted plugins and uninstalled plugin names.
fn collect_drift<F>(kvim_lock: &PluginMap, head_of: F) -> (Vec<Drift>, Vec<String>)
where
    F: Fn(&str) -> Option<String>,
{
    let mut drifted = Vec::new();
    let mut missing = Vec::new();

    for (plugin, value) in kvim_lock {
        if plugin == "KoalaVim" {
            continue;
        }
        let Some(expected) = plugin_commit(value) else {
            continue;
        };
        match head_of(plugin) {
            Some(actual) if !commit_eq(&actual, expected) => drifted.push(Drift {
                plugin: plugin.clone(),
                expected: expected.to_string(),
                actual,
            }),
            Some(_) => {}
            None => missing.push(plugin.clone()),
        }
    }

    (drifted, missing)
}

/// Files with uncommitted local modifications in a plugin checkout.
///
/// lazy.nvim refuses to move a dirty repo: its `git.status` task raises an
/// error, which halts the rest of that plugin's pipeline before `git.checkout`
/// ever runs. This is the most common reason a plugin silently fails to restore.
fn local_changes(plugin_dir: &Path) -> Vec<String> {
    let Ok(output) = Command::new("git")
        .args(["ls-files", "-d", "-m"])
        .current_dir(plugin_dir)
        .output()
    else {
        return Vec::new();
    };
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        // lazy self-heals this one rather than erroring on it
        .filter(|l| !l.is_empty() && l.replace('\\', "/") != "doc/tags")
        .map(str::to_string)
        .collect()
}

/// Resolve a plugin's checked-out commit, or `None` if it isn't a git checkout.
fn git_head(plugin_dir: &Path) -> Option<String> {
    if !plugin_dir.exists() {
        return None;
    }
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(plugin_dir)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let commit = String::from_utf8_lossy(&output.stdout).trim().to_string();
    (!commit.is_empty()).then_some(commit)
}

/// Verify plugins are actually checked out where KoalaVim's lockfile pins them.
///
/// This is the only reliable signal that a restore did anything: a plugin whose
/// pipeline halts early produces no task error, so a clean `:Lazy restore`
/// result is not on its own evidence that plugins moved.
///
/// Uninstalled plugins are reported as a note, not an error — KoalaVim's
/// lockfile keeps entries for plugins the user has disabled, and those
/// legitimately have no directory on disk.
fn verify_restore(env_name: &str) -> Result<(), String> {
    let kvim_lock = read_lockfile(&kvim_lockfile(env_name))?;
    let lazy_dir = env_lazy_dir(env_name);

    let (drifted, missing) = collect_drift(&kvim_lock, |plugin| git_head(&lazy_dir.join(plugin)));

    if !missing.is_empty() {
        eprintln!(
            " {} {} pinned plugin(s) not installed (expected if disabled in your config): {}",
            "--".dimmed(),
            missing.len(),
            missing.join(", ").dimmed()
        );
    }

    if drifted.is_empty() {
        return Ok(());
    }

    eprintln!(
        "{} {} plugin(s) were not restored to their pinned commit:",
        "error:".red().bold(),
        drifted.len()
    );

    let mut dirty_seen = false;
    for d in &drifted {
        let short = |c: &str| c[..c.len().min(12)].to_string();
        eprintln!(
            "  {:<32} {} (expected {})",
            d.plugin.bold(),
            short(&d.actual).red(),
            short(&d.expected).cyan()
        );

        let changes = local_changes(&lazy_dir.join(&d.plugin));
        if !changes.is_empty() {
            dirty_seen = true;
            eprintln!("  {:<32} {}", "", "has uncommitted local changes:".yellow());
            for file in &changes {
                eprintln!("  {:<32}   {}", "", file.dimmed());
            }
        }
    }

    if dirty_seen {
        eprintln!(
            "\n{} lazy.nvim refuses to update a plugin with local changes, and stops\n\
             that plugin's pipeline before the checkout runs. Commit, stash, or discard\n\
             the changes above, then re-run the update.",
            "note:".bold()
        );
    }

    Err(format!(
        "{} plugin(s) not restored to their pinned commit",
        drifted.len()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn sample_lockfile_json() -> &'static str {
        r#"{
  "plugin-a": { "commit": "aaa111", "branch": "main" },
  "plugin-b": { "commit": "bbb222", "branch": "main" },
  "KoalaVim": { "commit": "fff000", "branch": "master" }
}"#
    }

    #[test]
    fn test_read_lockfile() {
        let tmp = tempfile::TempDir::new().unwrap();
        let path = tmp.path().join("lazy-lock.json");
        fs::write(&path, sample_lockfile_json()).unwrap();

        let map = read_lockfile(&path).unwrap();
        assert_eq!(map.len(), 3);
        assert_eq!(plugin_commit(map.get("plugin-a").unwrap()), Some("aaa111"));
    }

    #[test]
    fn test_read_lockfile_missing_file() {
        let result = read_lockfile(Path::new("/tmp/nonexistent-lockfile.json"));
        assert!(result.is_err());
    }

    #[test]
    fn test_write_lockfile_roundtrip() {
        let tmp = tempfile::TempDir::new().unwrap();
        let path = tmp.path().join("lazy-lock.json");

        let mut map = BTreeMap::new();
        map.insert(
            "plugin-a".to_string(),
            serde_json::json!({"commit": "aaa111", "branch": "main"}),
        );
        map.insert(
            "plugin-b".to_string(),
            serde_json::json!({"commit": "bbb222", "branch": "main"}),
        );

        write_lockfile(&path, &map).unwrap();

        let reread = read_lockfile(&path).unwrap();
        assert_eq!(reread.len(), 2);
        assert_eq!(
            plugin_commit(reread.get("plugin-a").unwrap()),
            Some("aaa111")
        );
        assert_eq!(
            plugin_commit(reread.get("plugin-b").unwrap()),
            Some("bbb222")
        );
    }

    #[test]
    fn test_write_lockfile_format() {
        let tmp = tempfile::TempDir::new().unwrap();
        let path = tmp.path().join("lazy-lock.json");

        let mut map = BTreeMap::new();
        map.insert("alpha".to_string(), serde_json::json!({"commit": "aaa"}));

        write_lockfile(&path, &map).unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.starts_with('{'));
        assert!(content.contains("\"alpha\""));
        assert!(content.ends_with("}\n"));
    }

    #[test]
    fn test_plugin_commit_extraction() {
        let val = serde_json::json!({"commit": "abc123", "branch": "main"});
        assert_eq!(plugin_commit(&val), Some("abc123"));

        let no_commit = serde_json::json!({"branch": "main"});
        assert_eq!(plugin_commit(&no_commit), None);
    }

    fn lock_of(entries: &[(&str, &str)]) -> PluginMap {
        entries
            .iter()
            .map(|(name, commit)| {
                (
                    name.to_string(),
                    serde_json::json!({ "commit": commit, "branch": "main" }),
                )
            })
            .collect()
    }

    #[test]
    fn test_commit_eq() {
        let full = "d08fd3b921be36be360b15369b78ded602ce9b61";
        assert!(commit_eq(full, full));
        assert!(commit_eq(full, "d08fd3b"));
        assert!(commit_eq("d08fd3b", full));
        assert!(commit_eq(full, &full.to_uppercase()));

        assert!(!commit_eq(full, "dc804c8ac0c663bcd8d5bbbdb350bea5dde36890"));
        // too short to be a meaningful comparison
        assert!(!commit_eq(full, "d08fd"));
        assert!(!commit_eq("", ""));
    }

    #[test]
    fn test_collect_drift_in_sync() {
        let lock = lock_of(&[("plugin-a", "aaa1111"), ("plugin-b", "bbb2222")]);
        let (drifted, missing) = collect_drift(&lock, |name| match name {
            "plugin-a" => Some("aaa1111".to_string()),
            "plugin-b" => Some("bbb2222".to_string()),
            _ => None,
        });
        assert!(drifted.is_empty());
        assert!(missing.is_empty());
    }

    #[test]
    fn test_collect_drift_detects_stale_checkout() {
        // The regression this guards: user lockfile and on-disk agree with each
        // other but both lag KoalaVim, because the checkout was skipped.
        let kvim = lock_of(&[("codediff.nvim", "dc804c8ac0c663bcd8d5bbbdb350bea5dde36890")]);
        let (drifted, missing) = collect_drift(&kvim, |_| {
            Some("d08fd3b921be36be360b15369b78ded602ce9b61".to_string())
        });

        assert!(missing.is_empty());
        assert_eq!(
            drifted,
            vec![Drift {
                plugin: "codediff.nvim".to_string(),
                expected: "dc804c8ac0c663bcd8d5bbbdb350bea5dde36890".to_string(),
                actual: "d08fd3b921be36be360b15369b78ded602ce9b61".to_string(),
            }]
        );
    }

    #[test]
    fn test_collect_drift_reports_uninstalled_separately() {
        let lock = lock_of(&[("installed", "aaa1111"), ("absent", "bbb2222")]);
        let (drifted, missing) = collect_drift(&lock, |name| {
            (name == "installed").then(|| "aaa1111".to_string())
        });

        assert!(drifted.is_empty());
        assert_eq!(missing, vec!["absent".to_string()]);
    }

    #[test]
    fn test_collect_drift_skips_koalavim_entry() {
        let lock = lock_of(&[("KoalaVim", "fff0000"), ("plugin-a", "aaa1111")]);
        let (drifted, missing) = collect_drift(&lock, |name| {
            (name == "plugin-a").then(|| "aaa1111".to_string())
        });

        assert!(drifted.is_empty(), "KoalaVim must never be verified");
        assert!(missing.is_empty(), "KoalaVim must never be verified");
    }

    #[test]
    fn test_collect_drift_ignores_entries_without_commit() {
        let mut lock = PluginMap::new();
        lock.insert(
            "no-commit".to_string(),
            serde_json::json!({ "branch": "main" }),
        );
        let (drifted, missing) = collect_drift(&lock, |_| None);
        assert!(drifted.is_empty());
        assert!(missing.is_empty());
    }

    #[test]
    fn test_git_head_on_non_repo() {
        let tmp = tempfile::TempDir::new().unwrap();
        assert_eq!(git_head(&tmp.path().join("nope")), None);
        assert_eq!(git_head(tmp.path()), None);
    }
}
