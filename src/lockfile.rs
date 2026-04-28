use crate::paths::{env_appname, env_lockfile, kvim_lockfile};
use inquire::Confirm;
use owo_colors::OwoColorize;
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::process::Command;

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

/// Run `:Lazy restore` via headless nvim for the given env.
pub fn lazy_restore(env_name: &str) -> Result<(), String> {
    let appname = env_appname(env_name);

    eprintln!(
        "\n {} Running {} (sync plugin versions according to lockfile)",
        ">>".yellow().bold(),
        ":Lazy restore".bold(),
    );

    let output = Command::new("nvim")
        .args(["--headless", "+LazyRestoreLogged", "+qa"])
        .env("NVIM_APPNAME", &appname)
        .output()
        .map_err(|e| format!("Failed to run nvim for lazy restore: {}", e))?;

    let stderr = String::from_utf8_lossy(&output.stderr);

    // Find the JSON line in stderr — nvim may append extra characters (e.g. ":")
    let json_str = stderr
        .lines()
        .find(|l| l.starts_with('{'))
        .unwrap_or("")
        .trim();
    let result: Result<Value, _> = serde_json::from_str(json_str);
    match result {
        Ok(val) => {
            if let Some(plugins) = val.get("plugins").and_then(|p| p.as_object()) {
                if !plugins.is_empty() {
                    let label = ":Lazy restore";
                    eprintln!("{} {} finished with errors:", "error:".red().bold(), label);
                    for (plugin, error) in plugins {
                        eprintln!("  {}: {}", plugin.bold(), error);
                    }
                    return Err(":Lazy restore had plugin errors".to_string());
                }
            }
            eprintln!(
                " {} Finished successfully. Restart nvim to take effect.",
                ">>".green().bold()
            );
            Ok(())
        }
        Err(_) => {
            eprintln!(
                "{} Failed to decode lazy restore output: {}",
                "error:".red().bold(),
                stderr
            );
            Err("Failed to parse :Lazy restore output".to_string())
        }
    }
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
}
