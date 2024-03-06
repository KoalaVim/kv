use once_cell::sync::Lazy;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::process::Command;
use std::{env, ffi::OsString};
use structopt::StructOpt;
use subprocess::{Popen, PopenConfig, Redirection};

static DEFAULT_DEBUG_DIR: Lazy<String> = Lazy::new(|| {
    format!(
        "{}/kvim",
        env::var("HOME")
            .as_deref()
            .unwrap_or("FAILED_TO_GET_HOME_DIR"),
    )
});

static DEFAULT_KVIM_CONF: Lazy<String> = Lazy::new(|| {
    format!(
        "{}/.kvim.conf",
        env::var("HOME")
            .as_deref()
            .unwrap_or("FAILED_TO_GET_HOME_DIR"),
    )
});

static DEFAULT_LUA_CONF: Lazy<String> = Lazy::new(|| {
    xdg::BaseDirectories::with_prefix("nvim")
        .unwrap()
        .get_config_home()
        .as_os_str()
        .to_str()
        .unwrap_or("FAILED_TO_GET_NVIM_CFG_DIR")
        .to_string()
});

static DEFAULT_PROFILE_DIR: Lazy<String> = Lazy::new(|| {
    xdg::BaseDirectories::with_prefix("kvim")
        .unwrap()
        .get_data_home()
        .as_os_str()
        .to_str()
        .unwrap_or("FAILED_TO_GET_KVIM_DATA_PATH")
        .to_string()
});

#[derive(Debug, StructOpt)]
#[structopt(name = "kv", about = "Launcher for KoalaVim (neovim configuration)")]
struct Args {
    /// Start KoalaVim in debug mode, output goes to --debug_dir/<time_stamp>
    #[structopt(short, long)]
    debug: bool,

    /// Launch with given kvim.conf
    #[structopt(parse(from_os_str), default_value = &DEFAULT_DEBUG_DIR)]
    debug_dir: PathBuf,

    /// Launch with given kvim.conf
    #[structopt(parse(from_os_str), default_value = &DEFAULT_KVIM_CONF)]
    cfg: PathBuf,

    /// Launch with given lua cfg
    #[structopt(parse(from_os_str), default_value = &DEFAULT_LUA_CONF)]
    lua_cfg: PathBuf,

    /// Launch with given plugin proifle (creates new data dir at --profiles_dir)
    #[structopt(short, long, default_value = "master")]
    profile: String,

    /// Plugin profiles base directory
    #[structopt(parse(from_os_str), default_value = &DEFAULT_PROFILE_DIR)]
    profile_dir: PathBuf,
}

fn main() {
    let args = Args::from_args();

    let mut koala_env: Vec<(OsString, OsString)> = vec![];
    if args.debug {
        koala_env.push(("KOALA_DEBUG".into(), "1".into()))
        // TODO: add debug file
    }

    // println!("{:?}", koala_env);

    let mut env = PopenConfig::current_env();
    env.append(&mut koala_env);
    // println!("{:?}", env);

    Popen::create(
        &["nvim"],
        PopenConfig {
            env: Some(env),
            ..Default::default()
        },
    )
    .unwrap()
    .wait()
    .unwrap();
}
