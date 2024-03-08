use chrono::Local;
use once_cell::sync::Lazy;
use std::fs;
use std::path::{Path, PathBuf};
use std::{env, ffi::OsString};
use structopt::StructOpt;
use subprocess::{Popen, PopenConfig};

static DEFAULT_DEBUG_DIR: Lazy<String> = Lazy::new(|| {
    format!(
        "{}/kvim",
        env::temp_dir()
            .as_path()
            .to_str()
            .unwrap_or("FAILED_TO_GET_TMP_DIR")
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

    /// Change output log for debug
    #[structopt(long, parse(from_os_str), default_value = &DEFAULT_DEBUG_DIR)]
    debug_dir: PathBuf,

    /// Override debug file name (default is timestamp)
    #[structopt(long)]
    debug_file: Option<String>,

    /// Launch with given kvim.conf
    #[structopt(short, long, parse(from_os_str), default_value = &DEFAULT_KVIM_CONF)]
    cfg: PathBuf,

    /// Launch with given lua cfg
    #[structopt(short, long, parse(from_os_str), default_value = &DEFAULT_LUA_CONF)]
    lua_cfg: PathBuf,

    /// Launch with given plugin proifle (creates new data dir at --profiles_dir)
    #[structopt(short, long, default_value = "upstream")]
    profile: String,

    /// Plugin profiles base directory
    #[structopt(long, parse(from_os_str), default_value = &DEFAULT_PROFILE_DIR)]
    profile_dir: PathBuf,
}

fn main() {
    let args = Args::from_args();

    let mut koala_env: Vec<(OsString, OsString)> = vec![];
    koala_env.push(("KOALA_KVIM_CONF".into(), args.cfg.into()));

    if args.debug {
        let mut debug_file = args.debug_dir.clone();
        if let Some(file_name) = args.debug_file {
            debug_file.push(file_name);
        } else {
            let now = Local::now();
            debug_file.push(now.format("%Y-%m-%d_%H:%M:%S").to_string());
        }

        koala_env.push(("KOALA_DEBUG_OUT".into(), debug_file.into()));

        fs::create_dir_all(args.debug_dir).expect("failed to create debug dir")
    }

    println!("{:?}", koala_env);

    let mut env = PopenConfig::current_env();
    env.append(&mut koala_env);
    // println!("{:?}", env);

    Popen::create(
        &[
            "nvim",
            "-u",
            args.lua_cfg
                .join(Path::new("init.lua"))
                .as_os_str()
                .to_str()
                .expect("failed to generete lua cfg path"),
        ],
        PopenConfig {
            env: Some(env),
            ..Default::default()
        },
    )
    .unwrap()
    .wait()
    .unwrap();
}
