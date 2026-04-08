use std::env;
use std::path::PathBuf;

pub static ENV_PREFIX: &str = "kvim-envs";

fn home_dir() -> PathBuf {
    directories::BaseDirs::new()
        .expect("could not determine home directory")
        .home_dir()
        .to_path_buf()
}

/// Matches neovim's `stdpaths_get_xdg_var` for config/data/state:
/// XDG env var → platform fallback → hardcoded default.
///
/// On Unix (Linux + macOS), neovim falls back to `$HOME/{unix_suffix}`.
/// On Windows, neovim falls back to `$LOCALAPPDATA` then `~\AppData\Local`.
#[allow(unused_variables)]
fn nvim_base_dir(xdg_var: &str, unix_suffix: &str) -> PathBuf {
    if let Ok(val) = env::var(xdg_var) {
        return PathBuf::from(val);
    }
    #[cfg(windows)]
    {
        env::var("LOCALAPPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|_| home_dir().join("AppData").join("Local"))
    }
    #[cfg(not(windows))]
    {
        home_dir().join(unix_suffix)
    }
}

/// Matches neovim's cache base resolution:
/// `$XDG_CACHE_HOME` → `$TEMP` (Windows) / `~/.cache` (Unix).
fn nvim_cache_base() -> PathBuf {
    if let Ok(val) = env::var("XDG_CACHE_HOME") {
        return PathBuf::from(val);
    }
    #[cfg(windows)]
    {
        env::var("TEMP")
            .or_else(|_| env::var("TMP"))
            .map(PathBuf::from)
            .unwrap_or_else(|_| home_dir().join("AppData").join("Local").join("Temp"))
    }
    #[cfg(not(windows))]
    {
        home_dir().join(".cache")
    }
}

pub fn env_appname(name: &str) -> String {
    format!("{}/{}", ENV_PREFIX, name)
}

/// The root directory containing all env configs (e.g. `~/.config/kvim-envs`).
pub fn envs_config_root() -> PathBuf {
    nvim_base_dir("XDG_CONFIG_HOME", ".config").join(ENV_PREFIX)
}

pub fn env_config_dir(name: &str) -> PathBuf {
    nvim_base_dir("XDG_CONFIG_HOME", ".config")
        .join(ENV_PREFIX)
        .join(name)
}

pub fn env_data_dir(name: &str) -> PathBuf {
    let base = nvim_base_dir("XDG_DATA_HOME", ".local/share");
    // Neovim unconditionally appends "-data" to the appname on Windows
    // to avoid collisions with config (both default to LOCALAPPDATA).
    if cfg!(windows) {
        base.join(ENV_PREFIX).join(format!("{}-data", name))
    } else {
        base.join(ENV_PREFIX).join(name)
    }
}

pub fn env_state_dir(name: &str) -> PathBuf {
    let base = nvim_base_dir("XDG_STATE_HOME", ".local/state");
    // Neovim uses the same "-data" suffix for state on Windows.
    if cfg!(windows) {
        base.join(ENV_PREFIX).join(format!("{}-data", name))
    } else {
        base.join(ENV_PREFIX).join(name)
    }
}

pub fn env_cache_dir(name: &str) -> PathBuf {
    nvim_cache_base().join(ENV_PREFIX).join(name)
}

/// Returns all four dirs (config, data, state, cache) for the given env.
pub fn env_all_dirs(name: &str) -> [(&'static str, PathBuf); 4] {
    [
        ("config", env_config_dir(name)),
        ("data", env_data_dir(name)),
        ("state", env_state_dir(name)),
        ("cache", env_cache_dir(name)),
    ]
}

/// Where lazy.nvim installs the KoalaVim plugin for the given env.
pub fn env_kvim_dir(name: &str) -> PathBuf {
    env_data_dir(name).join("lazy").join("KoalaVim")
}

/// Root directory for lazy.nvim plugins.
#[allow(dead_code)]
pub fn env_lazy_dir(name: &str) -> PathBuf {
    env_data_dir(name).join("lazy")
}

/// Per-env binary directory (for tools installed by `kv install`).
pub fn env_bin_dir(name: &str) -> PathBuf {
    env_data_dir(name).join("kv").join("bin")
}

/// Per-env kv data directory (holds install manifest, lockfile backups, etc.).
pub fn env_kv_data_dir(name: &str) -> PathBuf {
    env_data_dir(name).join("kv")
}

/// User's lazy-lock.json for the given env.
pub fn env_lockfile(name: &str) -> PathBuf {
    env_config_dir(name).join("lazy-lock.json")
}

/// KoalaVim's own lazy-lock.json for the given env.
pub fn kvim_lockfile(name: &str) -> PathBuf {
    env_kvim_dir(name).join("lazy-lock.json")
}

/// Returns all four dir pairs (label, src, dst) for source and destination envs.
pub fn env_all_dir_pairs(src_name: &str, dst_name: &str) -> [(&'static str, PathBuf, PathBuf); 4] {
    [
        ("config", env_config_dir(src_name), env_config_dir(dst_name)),
        ("data", env_data_dir(src_name), env_data_dir(dst_name)),
        ("state", env_state_dir(src_name), env_state_dir(dst_name)),
        ("cache", env_cache_dir(src_name), env_cache_dir(dst_name)),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    fn test_env_appname() {
        assert_eq!(env_appname("test"), "kvim-envs/test");
        assert_eq!(env_appname("my-env"), "kvim-envs/my-env");
    }

    #[test]
    #[serial]
    fn test_env_dir_helpers() {
        let tmp = tempfile::TempDir::new().unwrap();
        let base = tmp.path();
        env::set_var("XDG_CONFIG_HOME", base.join("config"));
        env::set_var("XDG_DATA_HOME", base.join("data"));
        env::set_var("XDG_STATE_HOME", base.join("state"));
        env::set_var("XDG_CACHE_HOME", base.join("cache"));

        assert_eq!(env_config_dir("myenv"), base.join("config/kvim-envs/myenv"));
        assert_eq!(env_data_dir("myenv"), base.join("data/kvim-envs/myenv"));
        assert_eq!(env_state_dir("myenv"), base.join("state/kvim-envs/myenv"));
        assert_eq!(env_cache_dir("myenv"), base.join("cache/kvim-envs/myenv"));

        env::remove_var("XDG_CONFIG_HOME");
        env::remove_var("XDG_DATA_HOME");
        env::remove_var("XDG_STATE_HOME");
        env::remove_var("XDG_CACHE_HOME");
    }

    #[test]
    #[serial]
    fn test_env_derived_paths() {
        let tmp = tempfile::TempDir::new().unwrap();
        let base = tmp.path();
        env::set_var("XDG_CONFIG_HOME", base.join("config"));
        env::set_var("XDG_DATA_HOME", base.join("data"));
        env::set_var("XDG_STATE_HOME", base.join("state"));
        env::set_var("XDG_CACHE_HOME", base.join("cache"));

        assert_eq!(
            env_kvim_dir("myenv"),
            base.join("data/kvim-envs/myenv/lazy/KoalaVim")
        );
        assert_eq!(
            env_lazy_dir("myenv"),
            base.join("data/kvim-envs/myenv/lazy")
        );
        assert_eq!(
            env_bin_dir("myenv"),
            base.join("data/kvim-envs/myenv/kv/bin")
        );
        assert_eq!(
            env_kv_data_dir("myenv"),
            base.join("data/kvim-envs/myenv/kv")
        );
        assert_eq!(
            env_lockfile("myenv"),
            base.join("config/kvim-envs/myenv/lazy-lock.json")
        );
        assert_eq!(
            kvim_lockfile("myenv"),
            base.join("data/kvim-envs/myenv/lazy/KoalaVim/lazy-lock.json")
        );

        env::remove_var("XDG_CONFIG_HOME");
        env::remove_var("XDG_DATA_HOME");
        env::remove_var("XDG_STATE_HOME");
        env::remove_var("XDG_CACHE_HOME");
    }

    #[test]
    #[serial]
    fn test_env_dirs_default_to_xdg_paths() {
        env::remove_var("XDG_CONFIG_HOME");
        env::remove_var("XDG_DATA_HOME");
        env::remove_var("XDG_STATE_HOME");
        env::remove_var("XDG_CACHE_HOME");

        let home = home_dir();
        assert_eq!(
            env_config_dir("myenv"),
            home.join(".config/kvim-envs/myenv")
        );
        assert_eq!(
            env_data_dir("myenv"),
            home.join(".local/share/kvim-envs/myenv")
        );
        assert_eq!(
            env_state_dir("myenv"),
            home.join(".local/state/kvim-envs/myenv")
        );
        assert_eq!(env_cache_dir("myenv"), home.join(".cache/kvim-envs/myenv"));
    }
}
