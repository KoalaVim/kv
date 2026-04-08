use crate::cli::Cli;
use crate::env;
use crate::paths::{env_all_dirs, env_appname, env_bin_dir, env_config_dir, env_data_dir};
use chrono::Local;
use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

pub fn launch_nvim(cli: Cli) -> Result<(), String> {
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
    let env_name = cli.env.as_deref().unwrap_or("main");
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
    let env_name = cli.env.as_deref().unwrap_or("main");
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
    ];

    let bin_dir = env_bin_dir(env_name);
    if bin_dir.exists() {
        let current_path = std::env::var_os("PATH").unwrap_or_default();
        let mut new_path = OsString::from(&bin_dir);
        if !current_path.is_empty() {
            new_path.push(":");
            new_path.push(&current_path);
        }
        koala_env.push(("PATH".into(), new_path));
    }

    if cli.debug {
        let mut debug_file = cli.debug_dir.clone();
        if let Some(file_name) = &cli.debug_file {
            debug_file.push(file_name);
        } else {
            let now = Local::now();
            debug_file.push(now.format("%Y-%m-%d_%H:%M:%S").to_string());
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

pub fn tilde_shorten(path: &std::path::Path) -> String {
    let s = path.display().to_string();
    if let Ok(home) = std::env::var("HOME") {
        if let Some(rest) = s.strip_prefix(&home) {
            return format!("~{}", rest);
        }
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

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
        if let Ok(home) = std::env::var("HOME") {
            let path = std::path::PathBuf::from(&home).join("foo/bar");
            assert_eq!(tilde_shorten(&path), "~/foo/bar");
        }
    }

    #[test]
    fn test_tilde_shorten_no_home() {
        let path = std::path::PathBuf::from("/tmp/foo");
        assert_eq!(tilde_shorten(&path), "/tmp/foo");
    }
}
