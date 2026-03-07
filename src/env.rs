use crate::paths::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn validate_env_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("Env name cannot be empty".to_string());
    }
    if name == "." || name == ".." {
        return Err("Env name cannot be '.' or '..'".to_string());
    }
    if !name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        return Err(
            "Env name must only contain alphanumeric characters, hyphens, and underscores"
                .to_string(),
        );
    }
    Ok(())
}

fn is_git_url(s: &str) -> bool {
    s.contains("://") || s.starts_with("git@")
}

pub fn cmd_env_create(name: &str, from: Option<&str>, branch: Option<&str>) -> Result<PathBuf, String> {
    validate_env_name(name).map_err(|e| format!("Invalid env name: {}", e))?;

    let config_dir = env_config_dir(name);
    if config_dir.exists() {
        return Err(format!(
            "Env '{}' already exists at: {}",
            name,
            config_dir.display()
        ));
    }

    if let Some(source) = from {
        if is_git_url(source) {
            let mut cmd = Command::new("git");
            cmd.arg("clone");
            if let Some(b) = branch {
                cmd.arg("--branch").arg(b);
            }
            cmd.arg(source).arg(&config_dir);
            let output = cmd.output().map_err(|e| format!("Failed to run git clone: {}", e))?;
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(format!("git clone failed: {}", stderr.trim()));
            }
            // Remove .git/ so the env starts as a clean config
            let git_dir = config_dir.join(".git");
            if git_dir.exists() {
                fs::remove_dir_all(&git_dir)
                    .map_err(|e| format!("Failed to remove .git directory: {}", e))?;
            }
        } else {
            if branch.is_some() {
                return Err("--branch can only be used with a git URL source".to_string());
            }
            // Try as env name first, then as path
            let source_path = if env_config_dir(source).exists() {
                env_config_dir(source)
            } else {
                let p = PathBuf::from(source);
                if !p.exists() {
                    return Err(format!(
                        "Source '{}' not found as env name or path",
                        source
                    ));
                }
                p
            };
            copy_dir_recursive(&source_path, &config_dir)
                .map_err(|e| format!("Failed to copy from source: {}", e))?;
        }
    } else {
        if branch.is_some() {
            return Err("--branch can only be used with a git URL source".to_string());
        }
        fs::create_dir_all(&config_dir)
            .map_err(|e| format!("Failed to create env config dir: {}", e))?;
    }

    println!(
        "Created env '{}'. Populate config at: {}",
        name,
        config_dir.display()
    );
    Ok(config_dir)
}

pub fn cmd_env_fork(source: &str, name: &str) -> Result<PathBuf, String> {
    validate_env_name(source).map_err(|e| format!("Invalid source env name: {}", e))?;
    validate_env_name(name).map_err(|e| format!("Invalid env name: {}", e))?;

    let source_config = env_config_dir(source);
    if !source_config.exists() {
        return Err(format!("Source env '{}' does not exist.", source));
    }

    let dest_config = env_config_dir(name);
    if dest_config.exists() {
        return Err(format!(
            "Env '{}' already exists at: {}",
            name,
            dest_config.display()
        ));
    }

    let dirs = [
        ("config", env_config_dir(source), env_config_dir(name)),
        ("data", env_data_dir(source), env_data_dir(name)),
        ("state", env_state_dir(source), env_state_dir(name)),
        ("cache", env_cache_dir(source), env_cache_dir(name)),
    ];

    for (label, src, dst) in &dirs {
        if src.exists() {
            copy_dir_recursive(src, dst)
                .map_err(|e| format!("Failed to copy {} dir: {}", label, e))?;
            println!("Copied {} dir: {}", label, dst.display());
        }
    }

    println!("Forked env '{}' from '{}'.", name, source);
    Ok(dest_config)
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let dst_path = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_recursive(&entry.path(), &dst_path)?;
        } else {
            fs::copy(entry.path(), dst_path)?;
        }
    }
    Ok(())
}

fn dir_size(path: &Path) -> u64 {
    if !path.exists() {
        return 0;
    }
    let mut total = 0u64;
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.filter_map(|e| e.ok()) {
            let ft = entry.file_type().unwrap_or_else(|_| {
                // fallback: treat as file
                fs::metadata(entry.path()).unwrap().file_type()
            });
            if ft.is_dir() {
                total += dir_size(&entry.path());
            } else {
                total += entry.metadata().map(|m| m.len()).unwrap_or(0);
            }
        }
    }
    total
}

fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;
    if bytes >= GB {
        format!("{:.1}G", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1}M", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1}K", bytes as f64 / KB as f64)
    } else {
        format!("{}B", bytes)
    }
}

pub fn cmd_env_list() {
    let envs_dir = xdg_config_home().join(ENV_PREFIX);
    if !envs_dir.exists() {
        println!("No envs found.");
        return;
    }

    let mut entries: Vec<_> = match fs::read_dir(&envs_dir) {
        Ok(rd) => rd
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
            .collect(),
        Err(_) => {
            println!("No envs found.");
            return;
        }
    };

    if entries.is_empty() {
        println!("No envs found.");
        return;
    }

    entries.sort_by_key(|e| e.file_name());

    for entry in &entries {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();

        let dirs = [
            ("config", env_config_dir(&name_str)),
            ("data", env_data_dir(&name_str)),
            ("state", env_state_dir(&name_str)),
            ("cache", env_cache_dir(&name_str)),
        ];

        println!("  {}", name_str);
        for (label, dir) in &dirs {
            if dir.exists() {
                let size = dir_size(dir);
                println!("    {}: {} ({})", label, dir.display(), format_size(size));
            }
        }
    }

    println!("\n{} env(s) found.", entries.len());
}

pub fn cmd_env_delete(name: &str) -> Result<(), String> {
    validate_env_name(name).map_err(|e| format!("Invalid env name: {}", e))?;

    let config_dir = env_config_dir(name);
    if !config_dir.exists() {
        return Err(format!("Env '{}' does not exist.", name));
    }

    let dirs = [
        ("config", env_config_dir(name)),
        ("data", env_data_dir(name)),
        ("state", env_state_dir(name)),
        ("cache", env_cache_dir(name)),
    ];

    for (label, dir) in &dirs {
        if dir.exists() {
            fs::remove_dir_all(dir).unwrap_or_else(|e| {
                eprintln!("Failed to remove {} dir: {}", label, e);
            });
            println!("Removed {} dir: {}", label, dir.display());
        }
    }

    println!("Deleted env '{}'.", name);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use tempfile::TempDir;

    fn with_temp_xdg() -> TempDir {
        let tmp = TempDir::new().unwrap();
        let base = tmp.path();
        env::set_var("XDG_CONFIG_HOME", base.join("config"));
        env::set_var("XDG_DATA_HOME", base.join("data"));
        env::set_var("XDG_STATE_HOME", base.join("state"));
        env::set_var("XDG_CACHE_HOME", base.join("cache"));
        tmp
    }

    fn cleanup_xdg_env() {
        env::remove_var("XDG_CONFIG_HOME");
        env::remove_var("XDG_DATA_HOME");
        env::remove_var("XDG_STATE_HOME");
        env::remove_var("XDG_CACHE_HOME");
    }

    // --- validate_env_name ---

    #[test]
    fn test_validate_env_name_valid() {
        assert!(validate_env_name("my-env").is_ok());
        assert!(validate_env_name("test_env").is_ok());
        assert!(validate_env_name("env123").is_ok());
        assert!(validate_env_name("a").is_ok());
    }

    #[test]
    fn test_validate_env_name_empty() {
        assert!(validate_env_name("").is_err());
    }

    #[test]
    fn test_validate_env_name_dot() {
        assert!(validate_env_name(".").is_err());
        assert!(validate_env_name("..").is_err());
    }

    #[test]
    fn test_validate_env_name_invalid_chars() {
        assert!(validate_env_name("my/env").is_err());
        assert!(validate_env_name("my env").is_err());
        assert!(validate_env_name("my.env").is_err());
        assert!(validate_env_name("env@1").is_err());
    }

    // --- is_git_url ---

    #[test]
    fn test_is_git_url() {
        assert!(is_git_url("https://github.com/LazyVim/starter"));
        assert!(is_git_url("http://example.com/repo.git"));
        assert!(is_git_url("git://example.com/repo.git"));
        assert!(is_git_url("ssh://git@example.com/repo.git"));
        assert!(is_git_url("git@github.com:user/repo.git"));

        assert!(!is_git_url("my-env"));
        assert!(!is_git_url("/some/local/path"));
        assert!(!is_git_url("relative/path"));
    }

    // --- copy_dir_recursive ---

    #[test]
    fn test_copy_dir_recursive() {
        let tmp = TempDir::new().unwrap();
        let src = tmp.path().join("src");
        let dst = tmp.path().join("dst");

        fs::create_dir_all(src.join("sub")).unwrap();
        fs::write(src.join("a.txt"), "hello").unwrap();
        fs::write(src.join("sub/b.txt"), "world").unwrap();

        copy_dir_recursive(&src, &dst).unwrap();

        assert_eq!(fs::read_to_string(dst.join("a.txt")).unwrap(), "hello");
        assert_eq!(fs::read_to_string(dst.join("sub/b.txt")).unwrap(), "world");
    }

    // --- format_size ---

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(0), "0B");
        assert_eq!(format_size(512), "512B");
        assert_eq!(format_size(1024), "1.0K");
        assert_eq!(format_size(1536), "1.5K");
        assert_eq!(format_size(1048576), "1.0M");
        assert_eq!(format_size(1073741824), "1.0G");
    }

    // --- dir_size ---

    #[test]
    fn test_dir_size() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("sized");
        fs::create_dir_all(dir.join("sub")).unwrap();
        fs::write(dir.join("a.txt"), "hello").unwrap(); // 5 bytes
        fs::write(dir.join("sub/b.txt"), "world!").unwrap(); // 6 bytes

        assert_eq!(dir_size(&dir), 11);
        assert_eq!(dir_size(&tmp.path().join("nonexistent")), 0);
    }

    // --- cmd_env_create / cmd_env_delete integration ---

    #[test]
    fn test_env_create_and_delete() {
        let _tmp = with_temp_xdg();

        let result = cmd_env_create("test-env", None, None);
        assert!(result.is_ok());
        let config_dir = result.unwrap();
        assert!(config_dir.exists());

        let dup = cmd_env_create("test-env", None, None);
        assert!(dup.is_err());
        assert!(dup.unwrap_err().contains("already exists"));

        let del = cmd_env_delete("test-env");
        assert!(del.is_ok());
        assert!(!config_dir.exists());

        let del2 = cmd_env_delete("test-env");
        assert!(del2.is_err());
        assert!(del2.unwrap_err().contains("does not exist"));

        cleanup_xdg_env();
    }

    #[test]
    fn test_env_create_with_invalid_name() {
        let _tmp = with_temp_xdg();

        assert!(cmd_env_create("bad/name", None, None).is_err());
        assert!(cmd_env_create("", None, None).is_err());
        assert!(cmd_env_create("..", None, None).is_err());

        cleanup_xdg_env();
    }

    #[test]
    fn test_env_create_from_path() {
        let _tmp = with_temp_xdg();
        let source = _tmp.path().join("my-source-cfg");
        fs::create_dir_all(&source).unwrap();
        fs::write(source.join("init.lua"), "-- test config").unwrap();

        let result = cmd_env_create("from-path", Some(source.to_str().unwrap()), None);
        assert!(result.is_ok());

        let created = result.unwrap();
        assert_eq!(
            fs::read_to_string(created.join("init.lua")).unwrap(),
            "-- test config"
        );

        cleanup_xdg_env();
    }

    #[test]
    fn test_env_create_from_existing_env() {
        let _tmp = with_temp_xdg();

        cmd_env_create("source-env", None, None).unwrap();
        let source_config = env_config_dir("source-env");
        fs::write(source_config.join("init.lua"), "-- source").unwrap();

        let result = cmd_env_create("cloned-env", Some("source-env"), None);
        assert!(result.is_ok());
        let cloned = result.unwrap();
        assert_eq!(
            fs::read_to_string(cloned.join("init.lua")).unwrap(),
            "-- source"
        );

        cleanup_xdg_env();
    }

    #[test]
    fn test_env_delete_cleans_all_xdg_dirs() {
        let _tmp = with_temp_xdg();

        cmd_env_create("full-env", None, None).unwrap();

        fs::create_dir_all(env_data_dir("full-env")).unwrap();
        fs::create_dir_all(env_state_dir("full-env")).unwrap();
        fs::create_dir_all(env_cache_dir("full-env")).unwrap();

        assert!(env_config_dir("full-env").exists());
        assert!(env_data_dir("full-env").exists());
        assert!(env_state_dir("full-env").exists());
        assert!(env_cache_dir("full-env").exists());

        cmd_env_delete("full-env").unwrap();

        assert!(!env_config_dir("full-env").exists());
        assert!(!env_data_dir("full-env").exists());
        assert!(!env_state_dir("full-env").exists());
        assert!(!env_cache_dir("full-env").exists());

        cleanup_xdg_env();
    }

    #[test]
    fn test_env_create_rejects_branch_without_git_url() {
        let _tmp = with_temp_xdg();

        let source = _tmp.path().join("my-source");
        fs::create_dir_all(&source).unwrap();
        let result = cmd_env_create("test-branch", Some(source.to_str().unwrap()), Some("main"));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("--branch can only be used with a git URL"));

        let result2 = cmd_env_create("test-branch2", None, Some("main"));
        assert!(result2.is_err());
        assert!(result2.unwrap_err().contains("--branch can only be used with a git URL"));

        cleanup_xdg_env();
    }

    // --- env fork ---

    #[test]
    fn test_env_fork_copies_all_dirs() {
        let _tmp = with_temp_xdg();

        cmd_env_create("fork-src", None, None).unwrap();
        fs::write(env_config_dir("fork-src").join("init.lua"), "-- config").unwrap();
        fs::create_dir_all(env_data_dir("fork-src")).unwrap();
        fs::write(env_data_dir("fork-src").join("data.txt"), "data").unwrap();
        fs::create_dir_all(env_state_dir("fork-src")).unwrap();
        fs::write(env_state_dir("fork-src").join("state.txt"), "state").unwrap();
        fs::create_dir_all(env_cache_dir("fork-src")).unwrap();
        fs::write(env_cache_dir("fork-src").join("cache.txt"), "cache").unwrap();

        let result = cmd_env_fork("fork-src", "fork-dst");
        assert!(result.is_ok());

        assert_eq!(
            fs::read_to_string(env_config_dir("fork-dst").join("init.lua")).unwrap(),
            "-- config"
        );
        assert_eq!(
            fs::read_to_string(env_data_dir("fork-dst").join("data.txt")).unwrap(),
            "data"
        );
        assert_eq!(
            fs::read_to_string(env_state_dir("fork-dst").join("state.txt")).unwrap(),
            "state"
        );
        assert_eq!(
            fs::read_to_string(env_cache_dir("fork-dst").join("cache.txt")).unwrap(),
            "cache"
        );

        cleanup_xdg_env();
    }

    #[test]
    fn test_env_fork_nonexistent_source() {
        let _tmp = with_temp_xdg();

        let result = cmd_env_fork("nonexistent", "new-env");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("does not exist"));

        cleanup_xdg_env();
    }

    #[test]
    fn test_env_fork_duplicate_dest() {
        let _tmp = with_temp_xdg();

        cmd_env_create("fork-a", None, None).unwrap();
        cmd_env_create("fork-b", None, None).unwrap();

        let result = cmd_env_fork("fork-a", "fork-b");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("already exists"));

        cleanup_xdg_env();
    }
}
