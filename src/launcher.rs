use crate::cli::Cli;
use crate::env;
use crate::paths::{
    env_all_dirs, env_appname, env_bin_dir, env_config_dir, env_data_dir, env_kvim_conf,
    env_node_dir, env_nvim_runtime_dir,
};
use chrono::Local;
use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

pub fn launch_nvim(cli: Cli) -> Result<(), String> {
    raise_nofile_limit();

    let env_name = resolve_env_name(&cli)?;
    let appname = env_appname(&env_name);
    let koala_mode = resolve_koala_mode(&cli)?;
    let mut koala_env = build_koala_env(&cli, &env_name, &appname, koala_mode)?;
    let params = build_nvim_params(&cli, koala_mode);

    let restart_indicator = env_data_dir(&env_name).join("restart_kvim");

    if cli.verbose {
        print_verbose_info(&env_name, &appname, &restart_indicator, &koala_env, &params);
    }

    run_kvim(&koala_env, &params)?;

    koala_env.push(("KOALA_RESTART".into(), "1".into()));
    while restart_indicator.exists() {
        fs::remove_file(&restart_indicator)
            .map_err(|e| format!("Failed to remove restart indicator: {}", e))?;
        run_kvim(&koala_env, &params)?;
    }

    Ok(())
}

pub fn resolve_env_name(cli: &Cli) -> Result<String, String> {
    let from_env_var = std::env::var("KV_ENV").ok();
    let env_name = cli
        .env
        .as_deref()
        .or(from_env_var.as_deref())
        .unwrap_or("main");
    env::validate_env_name(env_name).map_err(|e| format!("Invalid env name: {}", e))?;
    let config_dir = env_config_dir(env_name);
    if !config_dir.exists() {
        if cli.env.is_some() {
            return Err(format!(
                "Env '{}' does not exist. Create it with: kv env create {}",
                env_name, env_name
            ));
        } else {
            eprintln!("Welcome to kv! No default environment found.");
            eprintln!();
            eprintln!("Run 'kv init' to set up your environment interactively, or:");
            eprintln!("  kv init            Set up the default 'main' env");
            eprintln!("  kv init --env foo  Set up a named env");
            eprintln!();
            eprintln!("For quick non-interactive setup:");
            eprintln!("  kv env create main                          Empty config");
            eprintln!("  kv env create main --from ~/.config/nvim    Copy existing config");
            eprintln!("  kv env create main --from <git-url>         Clone a starter template");
            std::process::exit(1);
        }
    }
    Ok(env_name.to_string())
}

/// Resolve the env name from CLI, defaulting to "main". Does not check existence.
pub fn resolve_env_name_unchecked(cli: &Cli) -> Result<String, String> {
    let from_env_var = std::env::var("KV_ENV").ok();
    let env_name = cli
        .env
        .as_deref()
        .or(from_env_var.as_deref())
        .unwrap_or("main");
    env::validate_env_name(env_name).map_err(|e| format!("Invalid env name: {}", e))?;
    Ok(env_name.to_string())
}

fn resolve_koala_mode(cli: &Cli) -> Result<Option<&'static str>, String> {
    let modes = [
        (cli.git, "git"),
        (cli.tree, "git_tree"),
        (cli.git_diff, "git_diff"),
        (cli.ai, "ai"),
    ];
    let active: Vec<_> = modes.iter().filter(|(flag, _)| *flag).collect();
    if active.len() > 1 {
        return Err("Multiple koala modes is not supported".to_string());
    }
    Ok(active.first().map(|(_, name)| *name))
}

fn build_koala_env(
    cli: &Cli,
    env_name: &str,
    appname: &str,
    koala_mode: Option<&str>,
) -> Result<Vec<(OsString, OsString)>, String> {
    let mut koala_env: Vec<(OsString, OsString)> = vec![
        ("NVIM_APPNAME".into(), appname.into()),
        ("KOALA_KVIM_CONF".into(), cli.cfg.as_os_str().into()),
        ("KOALA_ENV".into(), env_name.into()),
        (
            "KOALA_ENV_KVIM_CONF".into(),
            env_kvim_conf(env_name).into_os_string(),
        ),
    ];

    let bin_dir = env_bin_dir(env_name);
    if bin_dir.exists() {
        let current_path = std::env::var_os("PATH").unwrap_or_default();
        let mut new_path = OsString::from(&bin_dir);
        let path_sep = if cfg!(target_os = "windows") {
            ";"
        } else {
            ":"
        };

        if !current_path.is_empty() {
            new_path.push(path_sep);
            new_path.push(&current_path);
        }

        // Bundled Node goes last: a fallback for mason, so the user's own Node wins.
        let node_dir = env_node_dir(env_name);
        let node_bin = node_dir.join("bin");
        let node_path = if node_bin.exists() {
            node_bin
        } else {
            node_dir
        };
        if node_path.exists() {
            new_path.push(path_sep);
            new_path.push(&node_path);
        }
        koala_env.push(("PATH".into(), new_path));
    }

    let runtime_dir = env_nvim_runtime_dir(env_name);
    if runtime_dir.exists() {
        koala_env.push(("VIMRUNTIME".into(), runtime_dir.into_os_string()));
    }

    if cli.debug {
        let mut debug_file = cli.debug_dir.clone();
        if let Some(file_name) = &cli.debug_file {
            debug_file.push(file_name);
        } else {
            let now = Local::now();
            debug_file.push(now.format("%Y-%m-%d_%H-%M-%S").to_string());
        }

        koala_env.push(("KOALA_DEBUG_OUT".into(), debug_file.into()));

        fs::create_dir_all(&cli.debug_dir)
            .map_err(|e| format!("Failed to create debug dir: {}", e))?;
    }

    if cli.no_noice {
        koala_env.push(("KOALA_NO_NOICE".into(), "1".into()));
    }

    if koala_mode.is_some() {
        koala_env.push(("KOALA_NO_SESSION".into(), "1".into()));
    }

    if let Some(mode) = koala_mode {
        koala_env.push(("KOALA_MODE".into(), mode.into()));
        koala_env.push(("KOALA_ARGS".into(), join_args(&cli.nvim_args)));
    }

    Ok(koala_env)
}

fn build_nvim_params(cli: &Cli, koala_mode: Option<&str>) -> Vec<OsString> {
    let mut params: Vec<OsString> = vec![];

    let nvim_bin: OsString = cli
        .nvim_bin_path
        .as_ref()
        .map(|p| p.as_os_str().into())
        .unwrap_or_else(|| "nvim".into());
    params.push(nvim_bin);

    if koala_mode.is_none() {
        params.extend(cli.nvim_args.iter().cloned());
    }

    if let Some(lua_cfg) = &cli.lua_cfg {
        params.extend([OsString::from("-l"), lua_cfg.as_os_str().into()]);
    }

    params
}

fn print_verbose_info(
    env_name: &str,
    appname: &str,
    restart_indicator: &PathBuf,
    koala_env: &[(OsString, OsString)],
    params: &[OsString],
) {
    println!("NVIM_APPNAME: {}", appname);
    println!("Env: {}", env_name);
    for (label, dir) in &env_all_dirs(env_name) {
        println!("  {}:  {}", label, dir.display());
    }
    println!("Restart Indicator Path: {:?}", restart_indicator);
    println!("Koala Env: {:?}", koala_env);
    println!("Nvim Launch Params: {:?}", params);
}

pub fn join_args(args: &[OsString]) -> OsString {
    let mut result = OsString::new();
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            result.push(" ");
        }
        result.push(arg);
    }
    result
}

fn run_kvim(env: &[(OsString, OsString)], params: &[OsString]) -> Result<(), String> {
    let status = Command::new(&params[0])
        .args(&params[1..])
        .envs(env.iter().map(|(k, v)| (k, v)))
        .status()
        .map_err(|e| format!("Failed to launch nvim: {}", e))?;
    if !status.success() {
        let code = status.code().unwrap_or(1);
        return Err(format!("nvim exited with code {}", code));
    }
    Ok(())
}

/// Raise the open-file-descriptor soft limit to the hard limit.
/// Neovim plugin managers (lazy.nvim) spawn many concurrent git processes,
/// each consuming multiple FDs. The default macOS soft limit (256) is too low.
#[cfg(unix)]
fn raise_nofile_limit() {
    unsafe {
        let mut rlim = libc::rlimit {
            rlim_cur: 0,
            rlim_max: 0,
        };
        if libc::getrlimit(libc::RLIMIT_NOFILE, &mut rlim) == 0 && rlim.rlim_cur < rlim.rlim_max {
            rlim.rlim_cur = rlim.rlim_max;
            libc::setrlimit(libc::RLIMIT_NOFILE, &rlim);
        }
    }
}

#[cfg(not(unix))]
fn raise_nofile_limit() {}

pub fn tilde_shorten(path: &std::path::Path) -> String {
    let s = path.display().to_string();
    let home = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE"));
    if let Ok(home) = home {
        if let Some(rest) = s.strip_prefix(&home) {
            return format!("~{}", rest);
        }
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;
    use serial_test::serial;

    #[test]
    fn test_join_args_separates_with_space() {
        let args: Vec<OsString> = vec!["file1.txt".into(), "file2.txt".into()];
        assert_eq!(join_args(&args), OsString::from("file1.txt file2.txt"));
    }

    #[test]
    fn test_join_args_single() {
        let args: Vec<OsString> = vec!["file.txt".into()];
        assert_eq!(join_args(&args), OsString::from("file.txt"));
    }

    #[test]
    fn test_join_args_empty() {
        let args: Vec<OsString> = vec![];
        assert_eq!(join_args(&args), OsString::from(""));
    }

    #[test]
    fn test_tilde_shorten() {
        let home = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE"));
        if let Ok(home) = home {
            let path = std::path::PathBuf::from(&home).join("foo").join("bar");
            let shortened = tilde_shorten(&path);
            let sep = std::path::MAIN_SEPARATOR;
            assert_eq!(shortened, format!("~{sep}foo{sep}bar"));
        }
    }

    #[test]
    fn test_tilde_shorten_no_home() {
        let path = std::path::PathBuf::from("/tmp/foo");
        assert_eq!(tilde_shorten(&path), "/tmp/foo");
    }

    #[test]
    #[serial]
    fn test_resolve_env_name_unchecked_cli_flag_wins() {
        let cli = Cli::try_parse_from(["kv", "--env", "from-flag"]).unwrap();
        std::env::set_var("KV_ENV", "from-envvar");
        let result = resolve_env_name_unchecked(&cli).unwrap();
        std::env::remove_var("KV_ENV");
        assert_eq!(result, "from-flag");
    }

    #[test]
    #[serial]
    fn test_resolve_env_name_unchecked_reads_kv_env() {
        let cli = Cli::try_parse_from(["kv"]).unwrap();
        std::env::set_var("KV_ENV", "from-envvar");
        let result = resolve_env_name_unchecked(&cli).unwrap();
        std::env::remove_var("KV_ENV");
        assert_eq!(result, "from-envvar");
    }

    #[test]
    #[serial]
    fn test_resolve_env_name_unchecked_defaults_to_main() {
        let cli = Cli::try_parse_from(["kv"]).unwrap();
        std::env::remove_var("KV_ENV");
        let result = resolve_env_name_unchecked(&cli).unwrap();
        assert_eq!(result, "main");
    }

    fn env_value<'a>(env: &'a [(OsString, OsString)], key: &str) -> Option<&'a OsString> {
        env.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }

    #[test]
    #[serial]
    fn test_build_koala_env_sets_env_kvim_conf() {
        let tmp = tempfile::TempDir::new().unwrap();
        std::env::set_var("XDG_CONFIG_HOME", tmp.path().join("cfg"));
        std::env::set_var("XDG_DATA_HOME", tmp.path().join("data"));

        let cli = Cli::try_parse_from(["kv", "-c", "/tmp/my.conf", "--env", "work"]).unwrap();
        let env = build_koala_env(&cli, "work", "kvim-envs/work", None).unwrap();

        let expected = tmp
            .path()
            .join("cfg")
            .join("kvim-envs")
            .join("work")
            .join(".kvim.conf");
        assert_eq!(
            env_value(&env, "KOALA_ENV_KVIM_CONF"),
            Some(&expected.clone().into_os_string())
        );
        assert_eq!(
            env_value(&env, "KOALA_KVIM_CONF"),
            Some(&OsString::from("/tmp/my.conf"))
        );
        assert!(!expected.exists());
    }

    #[test]
    #[serial]
    fn test_build_koala_env_puts_bundled_node_last_on_path() {
        let tmp = tempfile::TempDir::new().unwrap();
        std::env::set_var("XDG_CONFIG_HOME", tmp.path().join("cfg"));
        std::env::set_var("XDG_DATA_HOME", tmp.path().join("data"));
        let bin_dir = env_bin_dir("work");
        let node_bin = env_node_dir("work").join("bin");
        std::fs::create_dir_all(&bin_dir).unwrap();
        std::fs::create_dir_all(&node_bin).unwrap();

        let sep = if cfg!(target_os = "windows") {
            ";"
        } else {
            ":"
        };
        let original_path = std::env::var_os("PATH");
        std::env::set_var("PATH", format!("/u/a{sep}/u/b"));

        let cli = Cli::try_parse_from(["kv", "--env", "work"]).unwrap();
        let env = build_koala_env(&cli, "work", "kvim-envs/work", None).unwrap();

        match original_path {
            Some(path) => std::env::set_var("PATH", path),
            None => std::env::remove_var("PATH"),
        }

        let mut expected = OsString::from(&bin_dir);
        expected.push(format!("{sep}/u/a{sep}/u/b{sep}"));
        expected.push(&node_bin);
        assert_eq!(env_value(&env, "PATH"), Some(&expected));
    }
}
