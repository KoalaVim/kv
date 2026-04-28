use crate::paths::{env_bin_dir, env_kv_data_dir};
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(clippy::enum_variant_names)]
enum Os {
    Linux,
    MacOs,
    Windows,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Arch {
    X86_64,
    Aarch64,
}

struct Dependency {
    name: &'static str,
    github_repo: &'static str,
    version: &'static str,
    binary_name: &'static str,
    asset_patterns: &'static [(Os, Arch, &'static str)],
    #[allow(dead_code)]
    strip_components: u32,
}

static DEPENDENCIES: &[Dependency] = &[
    Dependency {
        name: "neovim",
        github_repo: "neovim/neovim",
        version: "stable",
        binary_name: "nvim",
        asset_patterns: &[
            (Os::Linux, Arch::X86_64, "nvim-linux-x86_64.tar.gz"),
            (Os::Linux, Arch::Aarch64, "nvim-linux-arm64.tar.gz"),
            (Os::MacOs, Arch::X86_64, "nvim-macos-x86_64.tar.gz"),
            (Os::MacOs, Arch::Aarch64, "nvim-macos-arm64.tar.gz"),
            (Os::Windows, Arch::X86_64, "nvim-win64.zip"),
        ],
        strip_components: 2,
    },
    Dependency {
        name: "ripgrep",
        github_repo: "BurntSushi/ripgrep",
        version: "latest",
        binary_name: "rg",
        asset_patterns: &[
            (Os::Linux, Arch::X86_64, "x86_64-unknown-linux-musl.tar.gz"),
            (Os::Linux, Arch::Aarch64, "aarch64-unknown-linux-gnu.tar.gz"),
            (Os::MacOs, Arch::X86_64, "x86_64-apple-darwin.tar.gz"),
            (Os::MacOs, Arch::Aarch64, "aarch64-apple-darwin.tar.gz"),
            (Os::Windows, Arch::X86_64, "x86_64-pc-windows-msvc.zip"),
        ],
        strip_components: 1,
    },
    Dependency {
        name: "fd",
        github_repo: "sharkdp/fd",
        version: "latest",
        binary_name: "fd",
        asset_patterns: &[
            (Os::Linux, Arch::X86_64, "x86_64-unknown-linux-musl.tar.gz"),
            (Os::Linux, Arch::Aarch64, "aarch64-unknown-linux-gnu.tar.gz"),
            (Os::MacOs, Arch::X86_64, "x86_64-apple-darwin.tar.gz"),
            (Os::MacOs, Arch::Aarch64, "aarch64-apple-darwin.tar.gz"),
            (Os::Windows, Arch::X86_64, "x86_64-pc-windows-msvc.zip"),
        ],
        strip_components: 1,
    },
    Dependency {
        name: "fzf",
        github_repo: "junegunn/fzf",
        version: "latest",
        binary_name: "fzf",
        asset_patterns: &[
            (Os::Linux, Arch::X86_64, "linux_amd64.tar.gz"),
            (Os::Linux, Arch::Aarch64, "linux_arm64.tar.gz"),
            (Os::MacOs, Arch::X86_64, "darwin_amd64.zip"),
            (Os::MacOs, Arch::Aarch64, "darwin_arm64.zip"),
            (Os::Windows, Arch::X86_64, "windows_amd64.zip"),
        ],
        strip_components: 0,
    },
];

#[derive(Debug, Serialize, Deserialize, Default)]
struct InstallManifest {
    installed: BTreeMap<String, InstalledEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
struct InstalledEntry {
    version: String,
    asset_url: String,
    installed_at: String,
}

fn detect_os() -> Result<Os, String> {
    if cfg!(target_os = "linux") {
        Ok(Os::Linux)
    } else if cfg!(target_os = "macos") {
        Ok(Os::MacOs)
    } else if cfg!(target_os = "windows") {
        Ok(Os::Windows)
    } else {
        Err("Unsupported operating system".to_string())
    }
}

fn detect_arch() -> Result<Arch, String> {
    if cfg!(target_arch = "x86_64") {
        Ok(Arch::X86_64)
    } else if cfg!(target_arch = "aarch64") {
        Ok(Arch::Aarch64)
    } else {
        Err("Unsupported architecture".to_string())
    }
}

fn find_asset_pattern(dep: &Dependency, os: Os, arch: Arch) -> Result<&'static str, String> {
    dep.asset_patterns
        .iter()
        .find(|(o, a, _)| *o == os && *a == arch)
        .map(|(_, _, pattern)| *pattern)
        .ok_or_else(|| {
            format!(
                "No binary available for {} on {:?}/{:?}",
                dep.name, os, arch
            )
        })
}

/// Query the GitHub releases API for the download URL of a specific asset.
fn resolve_download_url(
    github_repo: &str,
    version: &str,
    asset_pattern: &str,
) -> Result<(String, String), String> {
    let version_path = if version == "latest" {
        "latest".to_string()
    } else {
        format!("tags/{}", version)
    };

    let api_url = format!(
        "https://api.github.com/repos/{}/releases/{}",
        github_repo, version_path
    );

    let output = Command::new("curl")
        .args([
            "-fsSL",
            "-H",
            "Accept: application/vnd.github.v3+json",
            &api_url,
        ])
        .output()
        .map_err(|e| format!("Failed to run curl: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "GitHub API request failed for {}: {}",
            github_repo, stderr
        ));
    }

    let body: serde_json::Value = serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("Failed to parse GitHub API response: {}", e))?;

    let tag = body
        .get("tag_name")
        .and_then(|t| t.as_str())
        .unwrap_or(version)
        .to_string();

    let assets = body
        .get("assets")
        .and_then(|a| a.as_array())
        .ok_or_else(|| format!("No assets found for {}", github_repo))?;

    for asset in assets {
        let name = asset.get("name").and_then(|n| n.as_str()).unwrap_or("");
        if name.contains(asset_pattern) {
            let url = asset
                .get("browser_download_url")
                .and_then(|u| u.as_str())
                .ok_or_else(|| "Asset has no download URL".to_string())?;
            return Ok((url.to_string(), tag));
        }
    }

    Err(format!(
        "No asset matching '{}' found for {} {}",
        asset_pattern, github_repo, version
    ))
}

fn download_file(url: &str, dest: &Path) -> Result<(), String> {
    let status = Command::new("curl")
        .args(["-fSL", "--progress-bar", "-o"])
        .arg(dest)
        .arg(url)
        .status()
        .map_err(|e| format!("Failed to run curl: {}", e))?;

    if !status.success() {
        return Err(format!("Download failed: {}", url));
    }
    Ok(())
}

fn extract_archive(archive: &Path, dest: &Path) -> Result<(), String> {
    fs::create_dir_all(dest).map_err(|e| format!("Failed to create extraction dir: {}", e))?;

    let archive_str = archive
        .to_str()
        .ok_or_else(|| "Invalid archive path".to_string())?;

    if archive_str.ends_with(".tar.gz") || archive_str.ends_with(".tgz") {
        let status = Command::new("tar")
            .args(["xzf"])
            .arg(archive)
            .arg("-C")
            .arg(dest)
            .status()
            .map_err(|e| format!("Failed to run tar: {}", e))?;
        if !status.success() {
            return Err("tar extraction failed".to_string());
        }
    } else if archive_str.ends_with(".zip") {
        let status = Command::new("unzip")
            .args(["-q", "-o"])
            .arg(archive)
            .arg("-d")
            .arg(dest)
            .status()
            .map_err(|e| format!("Failed to run unzip: {}", e))?;
        if !status.success() {
            return Err("unzip extraction failed".to_string());
        }
    } else {
        return Err(format!("Unknown archive format: {}", archive_str));
    }

    Ok(())
}

/// Find a binary within an extracted directory tree.
fn find_binary_in_dir(dir: &Path, binary_name: &str) -> Result<PathBuf, String> {
    find_binary_recursive(dir, binary_name)
        .ok_or_else(|| format!("Binary '{}' not found in {}", binary_name, dir.display()))
}

fn find_binary_recursive(dir: &Path, name: &str) -> Option<PathBuf> {
    let entries = fs::read_dir(dir).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        let file_name = entry.file_name();
        let fname = file_name.to_string_lossy();

        if fname == name || fname.strip_suffix(".exe") == Some(name) {
            return Some(path);
        }
        if path.is_dir() {
            if let Some(found) = find_binary_recursive(&path, name) {
                return Some(found);
            }
        }
    }
    None
}

fn install_binary(src: &Path, bin_dir: &Path) -> Result<(), String> {
    fs::create_dir_all(bin_dir).map_err(|e| format!("Failed to create bin dir: {}", e))?;

    let dest = bin_dir.join(
        src.file_name()
            .ok_or_else(|| "Invalid binary path".to_string())?,
    );
    fs::copy(src, &dest).map_err(|e| format!("Failed to copy binary: {}", e))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = fs::Permissions::from_mode(0o755);
        fs::set_permissions(&dest, perms)
            .map_err(|e| format!("Failed to set permissions: {}", e))?;
    }

    Ok(())
}

fn read_manifest(env_name: &str) -> InstallManifest {
    let path = env_kv_data_dir(env_name).join("install-manifest.json");
    if let Ok(content) = fs::read_to_string(&path) {
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        InstallManifest::default()
    }
}

fn write_manifest(env_name: &str, manifest: &InstallManifest) -> Result<(), String> {
    let dir = env_kv_data_dir(env_name);
    fs::create_dir_all(&dir).map_err(|e| format!("Failed to create kv data dir: {}", e))?;

    let path = dir.join("install-manifest.json");
    let json = serde_json::to_string_pretty(manifest)
        .map_err(|e| format!("Failed to serialize manifest: {}", e))?;
    fs::write(&path, json).map_err(|e| format!("Failed to write manifest: {}", e))?;
    Ok(())
}

pub fn cmd_install(env_name: &str, dry_run: bool) -> Result<(), String> {
    let os = detect_os()?;
    let arch = detect_arch()?;
    let bin_dir = env_bin_dir(env_name);

    println!(
        "{} dependencies for env '{}'\n",
        if dry_run {
            "Would install"
        } else {
            "Installing"
        },
        env_name.cyan().bold()
    );

    let mut manifest = read_manifest(env_name);
    let tmp_dir = std::env::temp_dir().join(format!("kv-install-{}", env_name));

    if !dry_run {
        fs::create_dir_all(&tmp_dir).map_err(|e| format!("Failed to create temp dir: {}", e))?;
    }

    let mut errors = Vec::new();

    for dep in DEPENDENCIES {
        println!("{}", "─".repeat(60).dimmed());

        let pattern = match find_asset_pattern(dep, os, arch) {
            Ok(p) => p,
            Err(e) => {
                println!(
                    "  {} {} -- {}",
                    "SKIP".yellow().bold(),
                    dep.name,
                    e.dimmed()
                );
                continue;
            }
        };

        println!(
            "  {} {} ({})",
            ">>>".cyan().bold(),
            dep.name.bold(),
            dep.github_repo.dimmed()
        );

        if dry_run {
            println!("      version: {}", dep.version);
            println!("      asset:   {}", pattern);
            println!("      dest:    {}", bin_dir.display());
            continue;
        }

        match install_single_dep(dep, pattern, &bin_dir, &tmp_dir, &mut manifest) {
            Ok(()) => {
                println!("  {} {}", "OK".green().bold(), dep.name);
            }
            Err(e) => {
                eprintln!("  {} {} -- {}", "FAIL".red().bold(), dep.name, e);
                errors.push(format!("{}: {}", dep.name, e));
            }
        }
    }

    if !dry_run {
        write_manifest(env_name, &manifest)?;
        let _ = fs::remove_dir_all(&tmp_dir);
    }

    println!("{}", "─".repeat(60).dimmed());
    if errors.is_empty() {
        println!(
            "\n{} All dependencies {}.",
            "Done:".green().bold(),
            if dry_run { "checked" } else { "installed" }
        );
        Ok(())
    } else {
        Err(format!(
            "{} dependency install(s) failed:\n  {}",
            errors.len(),
            errors.join("\n  ")
        ))
    }
}

fn install_single_dep(
    dep: &Dependency,
    pattern: &str,
    bin_dir: &Path,
    tmp_dir: &Path,
    manifest: &mut InstallManifest,
) -> Result<(), String> {
    let (url, tag) = resolve_download_url(dep.github_repo, dep.version, pattern)?;
    println!("      downloading: {}", url.dimmed());

    let archive_name = url.rsplit('/').next().unwrap_or("archive");
    let dep_tmp = tmp_dir.join(dep.name);
    fs::create_dir_all(&dep_tmp).map_err(|e| format!("Failed to create temp dir: {}", e))?;
    let archive_path = dep_tmp.join(archive_name);

    download_file(&url, &archive_path)?;

    let extract_dir = dep_tmp.join("extracted");
    println!("      extracting...");
    extract_archive(&archive_path, &extract_dir)?;

    let binary_path = find_binary_in_dir(&extract_dir, dep.binary_name)?;
    println!(
        "      installing {} to {}",
        dep.binary_name.bold(),
        bin_dir.display().to_string().dimmed()
    );
    install_binary(&binary_path, bin_dir)?;

    manifest.installed.insert(
        dep.name.to_string(),
        InstalledEntry {
            version: tag,
            asset_url: url,
            installed_at: chrono::Local::now().to_rfc3339(),
        },
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_os() {
        let os = detect_os();
        assert!(os.is_ok());
    }

    #[test]
    fn test_detect_arch() {
        let arch = detect_arch();
        assert!(arch.is_ok());
    }

    #[test]
    fn test_find_asset_pattern() {
        let dep = &DEPENDENCIES[0]; // neovim
        let pattern = find_asset_pattern(dep, Os::Linux, Arch::X86_64);
        assert!(pattern.is_ok());
        assert!(pattern.unwrap().contains("linux"));
    }

    #[test]
    fn test_find_asset_pattern_missing() {
        let dep = Dependency {
            name: "test",
            github_repo: "test/test",
            version: "latest",
            binary_name: "test",
            asset_patterns: &[],
            strip_components: 0,
        };
        let pattern = find_asset_pattern(&dep, Os::Linux, Arch::X86_64);
        assert!(pattern.is_err());
    }

    #[test]
    fn test_find_binary_in_dir() {
        let tmp = tempfile::TempDir::new().unwrap();
        let sub = tmp.path().join("subdir");
        fs::create_dir_all(&sub).unwrap();
        fs::write(sub.join("mybinary"), "fake binary").unwrap();

        let found = find_binary_in_dir(tmp.path(), "mybinary");
        assert!(found.is_ok());
        assert!(found.unwrap().ends_with("mybinary"));
    }

    #[test]
    fn test_find_binary_in_dir_missing() {
        let tmp = tempfile::TempDir::new().unwrap();
        let result = find_binary_in_dir(tmp.path(), "nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_install_binary_to_dir() {
        let tmp = tempfile::TempDir::new().unwrap();
        let src = tmp.path().join("mybinary");
        fs::write(&src, "binary content").unwrap();

        let bin_dir = tmp.path().join("bin");
        install_binary(&src, &bin_dir).unwrap();

        assert!(bin_dir.join("mybinary").exists());
    }

    #[test]
    fn test_manifest_roundtrip() {
        let tmp = tempfile::TempDir::new().unwrap();
        let dir = tmp.path().join("kv");
        fs::create_dir_all(&dir).unwrap();

        let path = dir.join("install-manifest.json");
        let mut manifest = InstallManifest::default();
        manifest.installed.insert(
            "test".to_string(),
            InstalledEntry {
                version: "v1.0".to_string(),
                asset_url: "https://example.com/test.tar.gz".to_string(),
                installed_at: "2024-01-01T00:00:00+00:00".to_string(),
            },
        );

        let json = serde_json::to_string_pretty(&manifest).unwrap();
        fs::write(&path, &json).unwrap();

        let reread: InstallManifest =
            serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(reread.installed.len(), 1);
        assert_eq!(reread.installed["test"].version, "v1.0");
    }

    #[test]
    fn test_dependencies_defined() {
        assert!(!DEPENDENCIES.is_empty());
        let names: Vec<_> = DEPENDENCIES.iter().map(|d| d.name).collect();
        assert!(names.contains(&"neovim"));
        assert!(names.contains(&"ripgrep"));
        assert!(names.contains(&"fd"));
        assert!(names.contains(&"fzf"));
    }
}
