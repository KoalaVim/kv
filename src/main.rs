mod cli;
mod env;
mod paths;

use chrono::Local;
use clap::{CommandFactory, Parser};
use cli::{Cli, Commands, EnvAction};
use env::{cmd_env_init, format_size};
use owo_colors::OwoColorize;
use paths::{env_appname, env_cache_dir, env_config_dir, env_data_dir, env_state_dir, xdg_data_home};
use std::ffi::OsString;
use std::fs;
use std::process::Command;

fn main() {
    let cli = Cli::parse();
    if let Err(e) = run(cli) {
        eprintln!("{} {}", "error:".red().bold(), e);
        std::process::exit(1);
    }
}

fn tilde_shorten(path: &std::path::Path) -> String {
    let s = path.display().to_string();
    if let Ok(home) = std::env::var("HOME") {
        if let Some(rest) = s.strip_prefix(&home) {
            return format!("~{}", rest);
        }
    }
    s
}

fn run(cli: Cli) -> Result<(), String> {
    // Handle subcommands
    if let Some(command) = &cli.command {
        match command {
            Commands::Init { env } => {
                let name = env.as_deref().unwrap_or("main");
                cmd_env_init(name)?;
                return Ok(());
            }
            Commands::Completions { shell } => {
                clap_complete::generate(
                    *shell,
                    &mut Cli::command(),
                    "kv",
                    &mut std::io::stdout(),
                );
                return Ok(());
            }
            Commands::Env { action } => {
                match action {
                    EnvAction::Create { name, from, branch } => {
                        env::cmd_env_create(name, from.as_deref(), branch.as_deref())?;
                    }
                    EnvAction::Fork { source, name } => {
                        env::cmd_env_fork(source, name)?;
                    }
                    EnvAction::List => {
                        let mut envs = env::cmd_env_list();
                        if envs.is_empty() {
                            println!("No envs found.");
                        } else {
                            // Show default (main) env first
                            if let Some(pos) = envs.iter().position(|e| e.name == "main") {
                                envs.swap(0, pos);
                            }
                            for info in &envs {
                                let default_marker = if info.name == "main" {
                                    " (default)".dimmed().to_string()
                                } else {
                                    String::new()
                                };
                                println!(
                                    "  {} [{}]{}",
                                    info.name.cyan().bold(),
                                    format_size(info.total_size),
                                    default_marker
                                );
                                for (label, dir, size) in &info.dirs {
                                    println!(
                                        "    {}: {} ({})",
                                        label.bold(),
                                        tilde_shorten(dir).dimmed(),
                                        format_size(*size)
                                    );
                                }
                            }
                            println!("\n{} env(s) found.", envs.len());
                        }
                    }
                    EnvAction::Delete { name, force } => {
                        env::cmd_env_delete(name, *force)?;
                    }
                    EnvAction::Rename { current, new_name } => {
                        env::cmd_env_rename(current, new_name)?;
                    }
                }
            }
        }
        return Ok(());
    }

    // Determine NVIM_APPNAME
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
    let appname = env_appname(env_name);

    let mut koala_env: Vec<(OsString, OsString)> = vec![
        ("NVIM_APPNAME".into(), appname.clone().into()),
        ("KOALA_KVIM_CONF".into(), cli.cfg.into()),
    ];

    // Compute restart indicator path via XDG data home + appname
    let data_dir = xdg_data_home().join(&appname);
    let restart_kvim_file_indicator = data_dir.join("restart_kvim");

    if cli.debug {
        let mut debug_file = cli.debug_dir.clone();
        if let Some(file_name) = cli.debug_file {
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
    let koala_mode = active.first().map(|(_, name)| *name);

    if koala_mode.is_some() {
        koala_env.push(("KOALA_NO_SESSION".into(), "1".into()));
    }

    if let Some(mode) = koala_mode {
        koala_env.push(("KOALA_MODE".into(), mode.into()));
        koala_env.push(("KOALA_ARGS".into(), join_args(&cli.nvim_args)));
    }

    if cli.verbose {
        println!("NVIM_APPNAME: {}", appname);
        println!("Env: {}", env_name);
        println!("  config: {}", env_config_dir(env_name).display());
        println!("  data:   {}", env_data_dir(env_name).display());
        println!("  state:  {}", env_state_dir(env_name).display());
        println!("  cache:  {}", env_cache_dir(env_name).display());
        println!("Restart Indicator Path: {:?}", restart_kvim_file_indicator);
        println!("Koala Env: {:?}", koala_env);
    }

    let mut params: Vec<OsString> = vec![];
    if koala_mode.is_none() {
        params.extend(cli.nvim_args.iter().cloned());
    }

    if let Some(lua_cfg) = cli.lua_cfg {
        params.extend([OsString::from("-l"), lua_cfg.into()]);
    }

    if let Some(bin_path) = cli.nvim_bin_path {
        params.insert(0, bin_path.into());
    } else {
        params.insert(0, "nvim".into());
    }

    if cli.verbose {
        println!("Nvim Launch Params: {:?}", params);
    }

    run_kvim(&koala_env, &params)?;

    // Push restart env value for subsequent runs
    koala_env.push(("KOALA_RESTART".into(), "1".into()));
    while restart_kvim_file_indicator.exists() {
        fs::remove_file(&restart_kvim_file_indicator)
            .map_err(|e| format!("Failed to remove restart indicator: {}", e))?;
        run_kvim(&koala_env, &params)?;
    }

    Ok(())
}

fn join_args(args: &[OsString]) -> OsString {
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
    Command::new(&params[0])
        .args(&params[1..])
        .envs(env.iter().map(|(k, v)| (k, v)))
        .status()
        .map_err(|e| format!("Failed to launch nvim: {}", e))?;
    Ok(())
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
