use std::env;
use std::path::PathBuf;

pub static ENV_PREFIX: &str = "kvim-envs";

pub fn xdg_config_home() -> PathBuf {
    env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let home = env::var("HOME").unwrap_or_else(|_| String::from("/tmp"));
            PathBuf::from(home).join(".config")
        })
}

pub fn xdg_data_home() -> PathBuf {
    env::var("XDG_DATA_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let home = env::var("HOME").unwrap_or_else(|_| String::from("/tmp"));
            PathBuf::from(home).join(".local/share")
        })
}

pub fn xdg_state_home() -> PathBuf {
    env::var("XDG_STATE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let home = env::var("HOME").unwrap_or_else(|_| String::from("/tmp"));
            PathBuf::from(home).join(".local/state")
        })
}

pub fn xdg_cache_home() -> PathBuf {
    env::var("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let home = env::var("HOME").unwrap_or_else(|_| String::from("/tmp"));
            PathBuf::from(home).join(".cache")
        })
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
