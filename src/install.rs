use crate::paths::{env_bin_dir, env_kv_data_dir, env_node_dir, env_nvim_runtime_dir};
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InstallMode {
    SingleBinary,
    FullTree,
}

struct Dependency {
    name: &'static str,
    github_repo: &'static str,
    version: &'static str,
    binary_name: &'static str,
    asset_patterns: &'static [(Os, Arch, &'static str)],
    #[allow(dead_code)]
    strip_components: u32,
    install_mode: InstallMode,
    /// When set, download directly from this base URL instead of using the GitHub Releases API.
    /// The final URL is `{direct_download_base}/{asset_pattern}`.
    direct_download_base: Option<&'static str>,
}

static DEPENDENCIES: &[Dependency] = &[
    Dependency {
        name: "neovim",
        github_repo: "neovim/neovim",
        version: "v0.12.4",
        binary_name: "nvim",
        asset_patterns: &[
            (Os::Linux, Arch::X86_64, "nvim-linux-x86_64.tar.gz"),
            (Os::Linux, Arch::Aarch64, "nvim-linux-arm64.tar.gz"),
            (Os::MacOs, Arch::X86_64, "nvim-macos-x86_64.tar.gz"),
            (Os::MacOs, Arch::Aarch64, "nvim-macos-arm64.tar.gz"),
            (Os::Windows, Arch::X86_64, "nvim-win64.zip"),
        ],
        strip_components: 2,
        install_mode: InstallMode::SingleBinary,
        direct_download_base: None,
    },
    Dependency {
        name: "node",
        github_repo: "nodejs/node",
        version: "v22.16.0",
        binary_name: "node",
        asset_patterns: &[
            (Os::Linux, Arch::X86_64, "node-v22.16.0-linux-x64.tar.gz"),
            (Os::Linux, Arch::Aarch64, "node-v22.16.0-linux-arm64.tar.gz"),
            (Os::MacOs, Arch::X86_64, "node-v22.16.0-darwin-x64.tar.gz"),
            (
                Os::MacOs,
                Arch::Aarch64,
                "node-v22.16.0-darwin-arm64.tar.gz",
            ),
            (Os::Windows, Arch::X86_64, "node-v22.16.0-win-x64.zip"),
        ],
        strip_components: 1,
        install_mode: InstallMode::FullTree,
        direct_download_base: Some("https://nodejs.org/dist/v22.16.0"),
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
        install_mode: InstallMode::SingleBinary,
        direct_download_base: None,
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
        install_mode: InstallMode::SingleBinary,
        direct_download_base: None,
    },
    Dependency {
        name: "fzf",
        github_repo: "junegunn/fzf",
        version: "latest",
        binary_name: "fzf",
        asset_patterns: &[
            (Os::Linux, Arch::X86_64, "linux_amd64.tar.gz"),
            (Os::Linux, Arch::Aarch64, "linux_arm64.tar.gz"),
            (Os::MacOs, Arch::X86_64, "darwin_amd64.tar.gz"),
            (Os::MacOs, Arch::Aarch64, "darwin_arm64.tar.gz"),
            (Os::Windows, Arch::X86_64, "windows_amd64.zip"),
        ],
        strip_components: 0,
        install_mode: InstallMode::SingleBinary,
        direct_download_base: None,
    },
    Dependency {
        name: "tree-sitter",
        github_repo: "tree-sitter/tree-sitter",
        version: "latest",
        binary_name: "tree-sitter",
        asset_patterns: &[
            (Os::Linux, Arch::X86_64, "tree-sitter-cli-linux-x64.zip"),
            (Os::Linux, Arch::Aarch64, "tree-sitter-cli-linux-arm64.zip"),
            (Os::MacOs, Arch::X86_64, "tree-sitter-cli-macos-x64.zip"),
            (Os::MacOs, Arch::Aarch64, "tree-sitter-cli-macos-arm64.zip"),
            (Os::Windows, Arch::X86_64, "tree-sitter-cli-windows-x64.zip"),
        ],
        strip_components: 0,
        install_mode: InstallMode::SingleBinary,
        direct_download_base: None,
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

/// Try to find a GitHub token from environment or `gh` CLI.
fn github_token() -> Option<String> {
    std::env::var("GITHUB_TOKEN")
        .or_else(|_| std::env::var("GH_TOKEN"))
        .ok()
        .or_else(|| {
            Command::new("gh")
                .args(["auth", "token"])
                .output()
                .ok()
                .filter(|o| o.status.success())
                .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
                .filter(|t| !t.is_empty())
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

    let mut curl_args = vec![
        "-fsSL".to_string(),
        "-H".to_string(),
        "Accept: application/vnd.github.v3+json".to_string(),
    ];
    if let Some(token) = github_token() {
        curl_args.push("-H".to_string());
        curl_args.push(format!("Authorization: Bearer {}", token));
    }
    curl_args.push(api_url.clone());

    let output = Command::new("curl")
        .args(&curl_args)
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
        let status = if cfg!(target_os = "windows") {
            Command::new("tar")
                .args(["xf"])
                .arg(archive)
                .arg("-C")
                .arg(dest)
                .status()
                .map_err(|e| format!("Failed to run tar: {}", e))?
        } else {
            Command::new("unzip")
                .args(["-q", "-o"])
                .arg(archive)
                .arg("-d")
                .arg(dest)
                .status()
                .map_err(|e| format!("Failed to run unzip: {}", e))?
        };
        if !status.success() {
            return Err("zip extraction failed".to_string());
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

/// Find `share/nvim/runtime` within the extracted neovim directory tree.
fn find_nvim_runtime_dir(dir: &Path) -> Option<PathBuf> {
    find_dir_recursive(dir, "runtime", &["share", "nvim"])
}

/// Recursively find a directory by name, verifying its ancestor path contains the required segments.
fn find_dir_recursive(
    dir: &Path,
    target_name: &str,
    ancestor_segments: &[&str],
) -> Option<PathBuf> {
    let entries = fs::read_dir(dir).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let fname = entry.file_name();
            if fname.to_string_lossy() == target_name {
                let path_str = path.display().to_string();
                if ancestor_segments.iter().all(|seg| path_str.contains(seg)) {
                    return Some(path);
                }
            }
            if let Some(found) = find_dir_recursive(&path, target_name, ancestor_segments) {
                return Some(found);
            }
        }
    }
    None
}

/// Copy a directory tree recursively.
fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), String> {
    fs::create_dir_all(dst)
        .map_err(|e| format!("Failed to create dir {}: {}", dst.display(), e))?;
    let entries =
        fs::read_dir(src).map_err(|e| format!("Failed to read dir {}: {}", src.display(), e))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)
                .map_err(|e| format!("Failed to copy {}: {}", src_path.display(), e))?;
        }
    }
    Ok(())
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

    #[cfg(target_os = "macos")]
    codesign_adhoc(&dest);

    Ok(())
}

/// Re-sign binary with a local ad-hoc signature so macOS Gatekeeper allows execution.
/// The provenance xattr on newer macOS is irremovable, but a fresh local signature overrides it.
#[cfg(target_os = "macos")]
fn codesign_adhoc(path: &Path) {
    let _ = Command::new("codesign")
        .args(["--force", "--sign", "-"])
        .arg(path)
        .output();
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

pub fn cmd_install(env_name: &str, dry_run: bool, force_reinstall: bool) -> Result<(), String> {
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

        let resolved = resolve_dep_version(dep, pattern)?;

        if !force_reinstall {
            if let Some(installed) = manifest.installed.get(dep.name) {
                if installed.version == resolved.tag {
                    println!(
                        "  {} {} ({})",
                        "OK".green().bold(),
                        dep.name,
                        "up to date".dimmed()
                    );
                    continue;
                }
            }
        }

        match install_single_dep(dep, &resolved, &bin_dir, &tmp_dir, &mut manifest, env_name) {
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

struct ResolvedDep {
    url: String,
    tag: String,
}

fn resolve_dep_version(dep: &Dependency, pattern: &str) -> Result<ResolvedDep, String> {
    if let Some(base) = dep.direct_download_base {
        Ok(ResolvedDep {
            url: format!("{}/{}", base, pattern),
            tag: dep.version.to_string(),
        })
    } else {
        let (url, tag) = resolve_download_url(dep.github_repo, dep.version, pattern)?;
        Ok(ResolvedDep { url, tag })
    }
}

fn install_single_dep(
    dep: &Dependency,
    resolved: &ResolvedDep,
    bin_dir: &Path,
    tmp_dir: &Path,
    manifest: &mut InstallManifest,
    env_name: &str,
) -> Result<(), String> {
    let url = &resolved.url;
    let tag = &resolved.tag;
    println!("      downloading: {}", url.dimmed());

    let archive_name = url.rsplit('/').next().unwrap_or("archive");
    let dep_tmp = tmp_dir.join(dep.name);
    fs::create_dir_all(&dep_tmp).map_err(|e| format!("Failed to create temp dir: {}", e))?;
    let archive_path = dep_tmp.join(archive_name);

    download_file(url, &archive_path)?;

    let extract_dir = dep_tmp.join("extracted");
    println!("      extracting...");
    extract_archive(&archive_path, &extract_dir)?;

    match dep.install_mode {
        InstallMode::SingleBinary => {
            let binary_path = find_binary_in_dir(&extract_dir, dep.binary_name)?;
            println!(
                "      installing {} to {}",
                dep.binary_name.bold(),
                bin_dir.display().to_string().dimmed()
            );
            install_binary(&binary_path, bin_dir)?;
        }
        InstallMode::FullTree => {
            let target_dir = resolve_full_tree_dir(dep.name, env_name);
            if target_dir.exists() {
                fs::remove_dir_all(&target_dir)
                    .map_err(|e| format!("Failed to remove old {}: {}", dep.name, e))?;
            }
            let tree_root = find_tree_root(&extract_dir, dep.binary_name)?;
            if let Some(parent) = target_dir.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create parent dir: {}", e))?;
            }
            println!(
                "      installing tree to {}",
                target_dir.display().to_string().dimmed()
            );
            move_dir(&tree_root, &target_dir)?;
        }
    }

    if dep.name == "neovim" {
        if let Some(runtime_src) = find_nvim_runtime_dir(&extract_dir) {
            let runtime_dst = env_nvim_runtime_dir(env_name);
            if runtime_dst.exists() {
                fs::remove_dir_all(&runtime_dst)
                    .map_err(|e| format!("Failed to remove old runtime: {}", e))?;
            }
            println!(
                "      installing runtime to {}",
                runtime_dst.display().to_string().dimmed()
            );
            copy_dir_recursive(&runtime_src, &runtime_dst)?;
        }
    }

    manifest.installed.insert(
        dep.name.to_string(),
        InstalledEntry {
            version: tag.clone(),
            asset_url: url.clone(),
            installed_at: chrono::Local::now().to_rfc3339(),
        },
    );

    Ok(())
}

/// Move a directory tree, preserving symlinks. Uses rename when possible,
/// falls back to platform copy commands that preserve symlinks.
fn move_dir(src: &Path, dst: &Path) -> Result<(), String> {
    if fs::rename(src, dst).is_ok() {
        return Ok(());
    }

    #[cfg(unix)]
    {
        let status = Command::new("cp")
            .args(["-a"])
            .arg(src)
            .arg(dst)
            .status()
            .map_err(|e| format!("Failed to run cp: {}", e))?;
        if !status.success() {
            return Err(format!("cp -a failed for {}", src.display()));
        }
        Ok(())
    }

    #[cfg(windows)]
    {
        let status = Command::new("xcopy")
            .arg(src)
            .arg(dst)
            .args(["/E", "/I", "/H", "/Q"])
            .status()
            .map_err(|e| format!("Failed to run xcopy: {}", e))?;
        if !status.success() {
            return Err(format!("xcopy failed for {}", src.display()));
        }
        Ok(())
    }
}

/// Resolve the installation directory for a full-tree dependency.
fn resolve_full_tree_dir(dep_name: &str, env_name: &str) -> PathBuf {
    match dep_name {
        "node" => env_node_dir(env_name),
        _ => env_kv_data_dir(env_name).join(dep_name),
    }
}

/// Find the root directory of an extracted full-tree dependency.
/// For tarballs with a single top-level directory (e.g. `node-v22.16.0-darwin-arm64/`),
/// returns that inner directory. Otherwise returns the extract dir itself.
fn find_tree_root(extract_dir: &Path, binary_name: &str) -> Result<PathBuf, String> {
    let entries: Vec<_> = fs::read_dir(extract_dir)
        .map_err(|e| format!("Failed to read extract dir: {}", e))?
        .filter_map(|e| e.ok())
        .collect();

    if entries.len() == 1 && entries[0].path().is_dir() {
        let inner = entries[0].path();
        if inner.join("bin").join(binary_name).exists() || inner.join("bin").exists() {
            return Ok(inner);
        }
    }

    if extract_dir.join("bin").join(binary_name).exists() {
        return Ok(extract_dir.to_path_buf());
    }

    Err(format!(
        "Could not locate tree root with bin/{} in {}",
        binary_name,
        extract_dir.display()
    ))
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
            install_mode: InstallMode::SingleBinary,
            direct_download_base: None,
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
