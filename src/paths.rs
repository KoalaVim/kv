use std::env;
use std::path::PathBuf;

pub static ENV_PREFIX: &str = "kvim-envs";

fn xdg_dir(env_var: &str, fallback: &str) -> PathBuf {
    env::var(env_var)
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let home = env::var("HOME").unwrap_or_else(|_| String::from("/tmp"));
            PathBuf::from(home).join(fallback)
        })
}

pub fn xdg_config_home() -> PathBuf {
    xdg_dir("XDG_CONFIG_HOME", ".config")
}

pub fn xdg_data_home() -> PathBuf {
    xdg_dir("XDG_DATA_HOME", ".local/share")
}

pub fn xdg_state_home() -> PathBuf {
    xdg_dir("XDG_STATE_HOME", ".local/state")
}

pub fn xdg_cache_home() -> PathBuf {
    xdg_dir("XDG_CACHE_HOME", ".cache")
}

pub fn env_appname(name: &str) -> String {
    format!("{}/{}", ENV_PREFIX, name)
}

pub fn env_config_dir(name: &str) -> PathBuf {
    xdg_config_home().join(ENV_PREFIX).join(name)
}

pub fn env_data_dir(name: &str) -> PathBuf {
    xdg_data_home().join(ENV_PREFIX).join(name)
}

pub fn env_state_dir(name: &str) -> PathBuf {
    xdg_state_home().join(ENV_PREFIX).join(name)
}

pub fn env_cache_dir(name: &str) -> PathBuf {
    xdg_cache_home().join(ENV_PREFIX).join(name)
}

/// Returns all four XDG dirs (config, data, state, cache) for the given env.
pub fn env_all_dirs(name: &str) -> [(&'static str, PathBuf); 4] {
    [
        ("config", env_config_dir(name)),
        ("data", env_data_dir(name)),
        ("state", env_state_dir(name)),
        ("cache", env_cache_dir(name)),
    ]
}

/// Returns all four XDG dir pairs (label, src, dst) for source and destination envs.
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

        assert_eq!(
            env_config_dir("myenv"),
            base.join("config/kvim-envs/myenv")
        );
        assert_eq!(
            env_data_dir("myenv"),
            base.join("data/kvim-envs/myenv")
        );
        assert_eq!(
            env_state_dir("myenv"),
            base.join("state/kvim-envs/myenv")
        );
        assert_eq!(
            env_cache_dir("myenv"),
            base.join("cache/kvim-envs/myenv")
        );

        env::remove_var("XDG_CONFIG_HOME");
        env::remove_var("XDG_DATA_HOME");
        env::remove_var("XDG_STATE_HOME");
        env::remove_var("XDG_CACHE_HOME");
    }
}
