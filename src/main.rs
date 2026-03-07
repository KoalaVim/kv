mod cli;
mod env;
mod paths;

use chrono::Local;
use clap::Parser;
use cli::{Cli, Commands, EnvAction};
use env::format_size;
use paths::{env_appname, env_cache_dir, env_config_dir, env_data_dir, env_state_dir, xdg_data_home};
use std::ffi::OsString;
use std::fs;
use subprocess::{Popen, PopenConfig};

static DEFAULT_APPNAME: &str = "kvim";

fn main() {
    let cli = Cli::parse();

    // Handle subcommands
    if let Some(Commands::Env { action }) = &cli.command {
        let result = match action {
            EnvAction::Create { name, from, branch } => {
                env::cmd_env_create(name, from.as_deref(), branch.as_deref()).map(|_| ())
            }
            EnvAction::Fork { source, name } => {
                env::cmd_env_fork(source, name).map(|_| ())
            }
            EnvAction::List => {
                let envs = env::cmd_env_list();
                if envs.is_empty() {
                    println!("No envs found.");
                } else {
                    for info in &envs {
                        println!("  {}", info.name);
                        for (label, dir, size) in &info.dirs {
                            println!("    {}: {} ({})", label, dir.display(), format_size(*size));
                        }
                    }
                    println!("\n{} env(s) found.", envs.len());
                }
                Ok(())
            }
            EnvAction::Delete { name } => env::cmd_env_delete(name),
        };
        if let Err(e) = result {
            eprintln!("{}", e);
            std::process::exit(1);
        }
        return;
    }

    // Determine NVIM_APPNAME
    let appname = match &cli.env {
        Some(name) => {
            if let Err(e) = env::validate_env_name(name) {
                eprintln!("Invalid env name: {}", e);
                std::process::exit(1);
            }
            let config_dir = env_config_dir(name);
            if !config_dir.exists() {
                eprintln!(
                    "Env '{}' does not exist. Create it with: kv env create {}",
                    name, name
                );
                std::process::exit(1);
            }
            env_appname(name)
        }
        None => DEFAULT_APPNAME.to_string(),
    };

    let mut koala_env: Vec<(OsString, OsString)> = vec![];
    koala_env.push(("NVIM_APPNAME".into(), appname.clone().into()));
    koala_env.push(("KOALA_KVIM_CONF".into(), cli.cfg.into()));

    // Compute restart indicator path via XDG data home + appname
    let data_dir = xdg_data_home().join(&appname);
    let restart_kvim_file_indicator = data_dir.join("nvim/restart_kvim");

    if cli.debug {
        let mut debug_file = cli.debug_dir.clone();
        if let Some(file_name) = cli.debug_file {
            debug_file.push(file_name);
        } else {
            let now = Local::now();
            debug_file.push(now.format("%Y-%m-%d_%H:%M:%S").to_string());
        }

        koala_env.push(("KOALA_DEBUG_OUT".into(), debug_file.into()));

        fs::create_dir_all(cli.debug_dir).expect("failed to create debug dir")
    }

    if cli.no_noice {
        koala_env.push(("KOALA_NO_NOICE".into(), "1".into()));
    }

    let mut koala_mode: Option<&str> = None;
    if cli.git {
        koala_env.push(("KOALA_NO_SESSION".into(), "1".into()));
        koala_mode = Some("git");
    }
    if cli.tree {
        if koala_mode.is_some() {
            eprintln!("Multiple koala modes is not supported");
            return;
        }

        koala_env.push(("KOALA_NO_SESSION".into(), "1".into()));
        koala_mode = Some("git_tree");
    }
    if cli.git_diff {
        if koala_mode.is_some() {
            eprintln!("Multiple koala modes is not supported");
            return;
        }

        koala_env.push(("KOALA_NO_SESSION".into(), "1".into()));
        koala_mode = Some("git_diff");
    }
    if cli.ai {
        if koala_mode.is_some() {
            eprintln!("Multiple koala modes is not supported");
            return;
        }

        koala_env.push(("KOALA_NO_SESSION".into(), "1".into()));
        koala_mode = Some("ai");
    }

    if let Some(koala_mode_ok) = koala_mode {
        koala_env.push(("KOALA_MODE".into(), koala_mode_ok.into()));

        koala_env.push(("KOALA_ARGS".into(), join_args(&cli.nvim_args)));
    }

    if cli.verbose {
        println!("NVIM_APPNAME: {}", appname);
        if let Some(ref name) = cli.env {
            println!("Env: {}", name);
            println!("  config: {}", env_config_dir(name).display());
            println!("  data:   {}", env_data_dir(name).display());
            println!("  state:  {}", env_state_dir(name).display());
            println!("  cache:  {}", env_cache_dir(name).display());
        }
        println!("Restart Indicator Path: {:?}", restart_kvim_file_indicator);
        println!("Koala Env: {:?}", koala_env);
    }

    let mut env = PopenConfig::current_env();
    env.append(&mut koala_env);

    if cli.verbose {
        println!("Env: {:?}", env);
    }

    let mut params: Vec<OsString> = vec![];
    if koala_mode.is_none() {
        params.append(&mut cli.nvim_args.clone());
    }

    if let Some(lua_cfg) = cli.lua_cfg {
        let lua_cfg_params: Vec<OsString> = vec!["-l".into(), lua_cfg.into()];
        params.append(&mut lua_cfg_params.clone());
    }

    if let Some(bin_path) = cli.nvim_bin_path {
        params.insert(0, bin_path.into());
    } else {
        params.insert(0, "nvim".into());
    }

    if cli.verbose {
        println!("Nvim Launch Params: {:?}", params);
    }
    run_kvim(&env, &params);

    // Push restart env value for the next run
    env.push(("KOALA_RESTART".into(), "1".into()));
    loop {
        if !restart_kvim_file_indicator.exists() {
            break;
        }

        // Remove indicator
        std::fs::remove_file(&restart_kvim_file_indicator)
            .expect("failed to remove restart kvim file indicator");

        // Re-run kvim with KOALA_RESTART=1
        run_kvim(&env, &params);
    }
}

fn join_args(args: &[OsString]) -> OsString {
    args.iter()
        .map(|arg| arg.clone().into_string().unwrap())
        .collect::<Vec<_>>()
        .join(" ")
        .into()
}

fn run_kvim(env: &[(OsString, OsString)], params: &[OsString]) {
    Popen::create(
        params,
        PopenConfig {
            env: Some(env.to_owned()),
            ..Default::default()
        },
    )
    .unwrap()
    .wait()
    .unwrap();
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
}
