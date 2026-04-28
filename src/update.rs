use crate::lockfile;
use crate::paths::{env_kv_data_dir, env_kvim_dir, env_lockfile};
use chrono::Local;
use owo_colors::OwoColorize;
use std::fs;
use std::process::Command;

pub fn cmd_update(
    env_name: &str,
    target: &str,
    remote: &str,
    force: bool,
    no_restore: bool,
) -> Result<(), String> {
    let kvim_dir = env_kvim_dir(env_name);
    if !kvim_dir.exists() {
        return Err(format!(
            "KoalaVim not found at: {}\nRun `kv` first to let lazy.nvim install KoalaVim.",
            kvim_dir.display()
        ));
    }

    if is_repo_dirty(&kvim_dir)? {
        eprintln!("{} Local KoalaVim dir is dirty", "warning:".yellow().bold());
        if !force {
            return Err(
                "KoalaVim dir has uncommitted changes. Use --force to override.".to_string(),
            );
        }
    }

    git_fetch(&kvim_dir, remote)?;

    let resolved_target = resolve_target(target, remote);
    eprintln!("Resetting to: {}", resolved_target.dimmed());
    git_reset(&kvim_dir, &resolved_target)?;

    backup_lockfile(env_name)?;

    eprintln!("{} lockfile", "Overwriting".green());
    lockfile::overwrite_lockfile(env_name)?;

    if no_restore {
        eprintln!(
            "Skipping lazy restore (--no-restore). Run {} manually.",
            "kv lockfile overwrite".bold()
        );
        return Ok(());
    }

    lockfile::lazy_restore(env_name)
}

fn is_repo_dirty(repo_dir: &std::path::Path) -> Result<bool, String> {
    let output = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(repo_dir)
        .output()
        .map_err(|e| format!("Failed to run git status: {}", e))?;
    Ok(!output.stdout.is_empty())
}

fn git_fetch(repo_dir: &std::path::Path, remote: &str) -> Result<(), String> {
    eprintln!("{} from {}...", "Fetching".green(), remote.dimmed());
    let status = Command::new("git")
        .args(["fetch", remote])
        .current_dir(repo_dir)
        .status()
        .map_err(|e| format!("Failed to run git fetch: {}", e))?;
    if !status.success() {
        return Err(format!("git fetch {} failed", remote));
    }
    Ok(())
}

fn git_reset(repo_dir: &std::path::Path, target: &str) -> Result<(), String> {
    let status = Command::new("git")
        .args(["reset", "--hard", target])
        .current_dir(repo_dir)
        .status()
        .map_err(|e| format!("Failed to run git reset: {}", e))?;
    if !status.success() {
        return Err(format!("git reset --hard {} failed", target));
    }
    Ok(())
}

fn resolve_target(target: &str, remote: &str) -> String {
    let is_commit_hash =
        target.len() >= 4 && target.len() <= 40 && target.chars().all(|c| c.is_ascii_hexdigit());

    if is_commit_hash {
        target.to_string()
    } else {
        format!("{}/{}", remote, target)
    }
}

fn backup_lockfile(env_name: &str) -> Result<(), String> {
    let user_lockfile = env_lockfile(env_name);
    if !user_lockfile.exists() {
        return Ok(());
    }

    let kv_data = env_kv_data_dir(env_name);
    fs::create_dir_all(&kv_data).map_err(|e| format!("Failed to create kv data dir: {}", e))?;

    let now = Local::now();
    let backup_name = format!("lazy-lock-{}.json.backup", now.format("%d-%m-%y_%H:%M:%S"));
    let backup_path = kv_data.join(&backup_name);

    eprintln!(
        "{} current lockfile to: {}",
        "Backing up".green(),
        backup_path.display().to_string().dimmed()
    );
    fs::copy(&user_lockfile, &backup_path)
        .map_err(|e| format!("Failed to backup lockfile: {}", e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_target_commit_hash() {
        assert_eq!(resolve_target("abc123", "origin"), "abc123");
        assert_eq!(
            resolve_target("abcdef1234567890", "origin"),
            "abcdef1234567890"
        );
    }

    #[test]
    fn test_resolve_target_branch_name() {
        assert_eq!(resolve_target("master", "origin"), "origin/master");
        assert_eq!(resolve_target("main", "upstream"), "upstream/main");
        assert_eq!(resolve_target("v2.0", "origin"), "origin/v2.0");
    }

    #[test]
    fn test_resolve_target_short_hash() {
        assert_eq!(resolve_target("abcd", "origin"), "abcd");
    }

    #[test]
    fn test_resolve_target_too_short_for_hash() {
        assert_eq!(resolve_target("abc", "origin"), "origin/abc");
    }

    #[test]
    fn test_resolve_target_mixed_chars_not_hex() {
        assert_eq!(resolve_target("abcxyz", "origin"), "origin/abcxyz");
    }
}
