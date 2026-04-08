mod cli;
mod env;
mod health;
mod install;
mod launcher;
mod lockfile;
mod paths;
mod update;

use clap::{CommandFactory, Parser};
use clap_complete::Shell;
use cli::{Cli, Commands, EnvAction, LockfileAction};
use env::{cmd_env_init, format_size};
use launcher::tilde_shorten;
use owo_colors::OwoColorize;

fn main() {
    let cli = Cli::parse();
    if let Err(e) = run(cli) {
        eprintln!("{} {}", "error:".red().bold(), e);
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> Result<(), String> {
    if let Some(command) = &cli.command {
        return handle_subcommand(&cli, command);
    }
    launcher::launch_nvim(cli)
}

fn handle_subcommand(cli: &Cli, command: &Commands) -> Result<(), String> {
    match command {
        Commands::Init { env } => {
            let name = env.as_deref().unwrap_or("main");
            cmd_env_init(name)?;
        }
        Commands::Completions { shell } => {
            let mut buf = Vec::new();
            clap_complete::generate(*shell, &mut Cli::command(), "kv", &mut buf);
            let raw = String::from_utf8_lossy(&buf);
            print!("{}", patch_zsh_completions(*shell, &raw));
        }
        Commands::Env { action } => {
            handle_env_action(action)?;
        }
        Commands::Lockfile { action } => {
            let env_name = launcher::resolve_env_name_unchecked(cli)?;
            handle_lockfile_action(action, &env_name)?;
        }
        Commands::Update {
            target,
            remote,
            force,
            no_restore,
        } => {
            let env_name = launcher::resolve_env_name_unchecked(cli)?;
            update::cmd_update(&env_name, target, remote, *force, *no_restore)?;
        }
        Commands::Install { dry_run } => {
            let env_name = launcher::resolve_env_name_unchecked(cli)?;
            install::cmd_install(&env_name, *dry_run)?;
        }
        Commands::Health => {
            let env_name = launcher::resolve_env_name_unchecked(cli)?;
            health::cmd_health(&env_name)?;
        }
    }
    Ok(())
}

fn handle_env_action(action: &EnvAction) -> Result<(), String> {
    match action {
        EnvAction::Create { name, from, branch } => {
            env::cmd_env_create(name, from.as_deref(), branch.as_deref())?;
        }
        EnvAction::Fork { source, name } => {
            env::cmd_env_fork(source, name)?;
        }
        EnvAction::List => {
            print_env_list();
        }
        EnvAction::Delete { name, force } => {
            env::cmd_env_delete(name, *force)?;
        }
        EnvAction::Rename { current, new_name } => {
            env::cmd_env_rename(current, new_name)?;
        }
    }
    Ok(())
}

fn handle_lockfile_action(action: &LockfileAction, env_name: &str) -> Result<(), String> {
    match action {
        LockfileAction::Diff => lockfile::cmd_lockfile_diff(env_name),
        LockfileAction::Overwrite { yes } => lockfile::cmd_lockfile_overwrite(env_name, *yes),
    }
}

fn print_env_list() {
    let mut envs = env::cmd_env_list();
    if envs.is_empty() {
        println!("No envs found.");
        return;
    }
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

fn patch_zsh_completions(shell: Shell, raw: &str) -> String {
    if !matches!(shell, Shell::Zsh) {
        return raw.to_string();
    }

    let mut output = String::new();
    for line in raw.lines() {
        if line.contains("nvim_args") {
            continue;
        }
        let line = line.replace("line[2]", "line[1]");
        output.push_str(&line);
        output.push('\n');
    }

    output = output.replace(
        "&& ret=0\n    case $state in\n    (kv)\n",
        "&& ret=0\n    [[ -z \"$state\" ]] && _files && ret=0\n    case $state in\n    (kv)\n",
    );

    let end = "        esac\n    ;;\nesac\n}\n";
    if let Some(pos) = output.rfind(end) {
        let fallback = "            (*)\n_files && ret=0\n;;\n";
        output.insert_str(pos, fallback);
    }

    output
}
