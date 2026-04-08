use crate::paths::env_bin_dir;
use owo_colors::OwoColorize;
use std::path::PathBuf;
use std::process::Command;

struct HealthCheck {
    name: &'static str,
    group: &'static str,
    check: fn(&str) -> HealthResult,
}

enum HealthResult {
    Ok {
        version: String,
        detail: Option<String>,
    },
    Missing(String),
}

fn find_binary(name: &str, env_name: &str) -> PathBuf {
    let env_path = env_bin_dir(env_name).join(name);
    if env_path.exists() {
        return env_path;
    }
    PathBuf::from(name)
}

fn check_nvim(env_name: &str) -> HealthResult {
    let bin = find_binary("nvim", env_name);
    match Command::new(&bin).arg("--version").output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if let Some(line) = stdout.lines().next() {
                let version = line.strip_prefix("NVIM v").unwrap_or(line).to_string();
                HealthResult::Ok {
                    version,
                    detail: None,
                }
            } else {
                HealthResult::Missing("could not parse nvim version".to_string())
            }
        }
        Err(e) => HealthResult::Missing(e.to_string()),
    }
}

fn check_ripgrep(env_name: &str) -> HealthResult {
    let bin = find_binary("rg", env_name);
    match Command::new(&bin).arg("--version").output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if let Some(line) = stdout.lines().next() {
                let version = line
                    .strip_prefix("ripgrep ")
                    .unwrap_or(line)
                    .split_whitespace()
                    .next()
                    .unwrap_or(line)
                    .to_string();
                HealthResult::Ok {
                    version,
                    detail: None,
                }
            } else {
                HealthResult::Missing("could not parse rg version".to_string())
            }
        }
        Err(e) => HealthResult::Missing(e.to_string()),
    }
}

fn check_fd(env_name: &str) -> HealthResult {
    let bin = find_binary("fd", env_name);
    match Command::new(&bin).arg("--version").output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let trimmed = stdout.trim();
            let version = trimmed.strip_prefix("fd ").unwrap_or(trimmed).to_string();
            HealthResult::Ok {
                version,
                detail: None,
            }
        }
        Err(e) => HealthResult::Missing(e.to_string()),
    }
}

fn check_fzf(env_name: &str) -> HealthResult {
    let bin = find_binary("fzf", env_name);
    match Command::new(&bin).arg("--version").output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let trimmed = stdout.trim();
            let version = trimmed
                .split_whitespace()
                .next()
                .unwrap_or(trimmed)
                .to_string();
            HealthResult::Ok {
                version,
                detail: None,
            }
        }
        Err(e) => HealthResult::Missing(e.to_string()),
    }
}

#[cfg(unix)]
fn check_nerd_font(_env_name: &str) -> HealthResult {
    match Command::new("fc-list").output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if line.contains("Nerd Font") {
                    return HealthResult::Ok {
                        version: "installed".to_string(),
                        detail: None,
                    };
                }
            }
            HealthResult::Missing("Nerd Font not found (checked fc-list)".to_string())
        }
        Err(e) => HealthResult::Missing(format!("fc-list not available: {}", e)),
    }
}

#[cfg(not(unix))]
fn check_nerd_font(_env_name: &str) -> HealthResult {
    HealthResult::Missing("font detection not supported on this platform".to_string())
}

fn check_curl(_env_name: &str) -> HealthResult {
    match Command::new("curl").arg("--version").output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let version = stdout
                .lines()
                .next()
                .and_then(|l| l.strip_prefix("curl "))
                .and_then(|l| l.split_whitespace().next())
                .unwrap_or("unknown")
                .to_string();
            HealthResult::Ok {
                version,
                detail: None,
            }
        }
        Err(e) => HealthResult::Missing(e.to_string()),
    }
}

fn check_git(_env_name: &str) -> HealthResult {
    match Command::new("git").arg("--version").output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let version = stdout
                .trim()
                .strip_prefix("git version ")
                .unwrap_or(stdout.trim())
                .to_string();
            HealthResult::Ok {
                version,
                detail: None,
            }
        }
        Err(e) => HealthResult::Missing(e.to_string()),
    }
}

static HEALTH_CHECKS: &[HealthCheck] = &[
    HealthCheck {
        name: "nvim",
        group: "core",
        check: check_nvim,
    },
    HealthCheck {
        name: "git",
        group: "core",
        check: check_git,
    },
    HealthCheck {
        name: "ripgrep (rg)",
        group: "dependencies",
        check: check_ripgrep,
    },
    HealthCheck {
        name: "fd",
        group: "dependencies",
        check: check_fd,
    },
    HealthCheck {
        name: "fzf",
        group: "dependencies",
        check: check_fzf,
    },
    HealthCheck {
        name: "curl",
        group: "dependencies",
        check: check_curl,
    },
    HealthCheck {
        name: "nerd font",
        group: "optional",
        check: check_nerd_font,
    },
];

pub fn cmd_health(env_name: &str) -> Result<(), String> {
    println!(
        "{} (env: {})\n",
        "KoalaVim Health Check".bold(),
        env_name.cyan().bold()
    );

    let mut current_group = "";

    for hc in HEALTH_CHECKS {
        if hc.group != current_group {
            current_group = hc.group;
            println!("  {}:", current_group.bold());
        }

        let result = (hc.check)(env_name);
        match result {
            HealthResult::Ok { version, detail } => {
                let detail_str = detail
                    .map(|d| format!(" ({})", d.dimmed()))
                    .unwrap_or_default();
                println!(
                    "    {} {:<20} {}{}",
                    "OK".green().bold(),
                    hc.name,
                    version,
                    detail_str
                );
            }
            HealthResult::Missing(reason) => {
                println!(
                    "    {} {:<20} {}",
                    "MISSING".red().bold(),
                    hc.name,
                    reason.dimmed()
                );
            }
        }
    }

    println!();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_binary_falls_back_to_name() {
        let path = find_binary("nvim", "nonexistent-env");
        assert_eq!(path, PathBuf::from("nvim"));
    }

    #[test]
    fn test_health_checks_registered() {
        assert!(!HEALTH_CHECKS.is_empty());
        let names: Vec<_> = HEALTH_CHECKS.iter().map(|h| h.name).collect();
        assert!(names.contains(&"nvim"));
        assert!(names.contains(&"ripgrep (rg)"));
        assert!(names.contains(&"fd"));
        assert!(names.contains(&"fzf"));
    }
}
