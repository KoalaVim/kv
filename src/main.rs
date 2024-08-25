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
    /// Start KoalaVim in git mode
    #[structopt(short, long)]
    git: bool,

    /// Start KoalaVim in git tree mode
    #[structopt(short, long)]
    tree: bool,

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

    /// Override `XDG_STATE_HOME` for a each profile. (state dir contains sessions data, various
    /// plugins data, runtime data and more)
    #[structopt(long)]
    override_state: bool,

    /// Arguments to pass to nvim binary.
    /// On mode (git/tree) arguments passed to KoalaVim.
    #[structopt()]
    nvim_args: Vec<OsString>,
}

fn main() {
    let args = Args::from_args();

    let mut koala_env: Vec<(OsString, OsString)> = vec![];
    koala_env.push(("KOALA_KVIM_CONF".into(), args.cfg.into()));

    let data_dir = args.profile_dir.join(args.profile.clone());
    koala_env.push(("XDG_DATA_HOME".into(), data_dir.clone().into()));

    if args.override_state {
        let state_dir = xdg::BaseDirectories::with_prefix("kvim")
            .unwrap()
            .get_state_home()
            .join(Path::new(&args.profile));

        koala_env.push(("XDG_STATE_HOME".into(), state_dir.into()));
    }

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

    let mut koala_mode: Option<&str> = None;
    if args.git {
        koala_env.push(("KOALA_NO_SESSION".into(), "1".into()));
        koala_mode = Some("git");
    }
    if args.tree {
        if koala_mode.is_some() {
            eprintln!("Multiple koala modes is not supported");
            return;
        }

        koala_env.push(("KOALA_NO_SESSION".into(), "1".into()));
        koala_mode = Some("git_tree");
    }

    if let Some(koala_mode_ok) = koala_mode {
        koala_env.push(("KOALA_MODE".into(), koala_mode_ok.into()));

        koala_env.push((
            "KOALA_ARGS".into(),
            args.nvim_args
                .iter()
                .map(|arg| arg.clone().into_string().unwrap())
                .collect::<String>()
                .into(),
        ));
    }

    // println!("{:?}", koala_env);
    let mut env = PopenConfig::current_env();
    env.append(&mut koala_env);
    // println!("{:?}", env);

    let mut params: Vec<OsString> = if args.lua_cfg.is_dir() {
        vec![
            "-u".into(),
            args.lua_cfg
                .join(Path::new("init.lua"))
                .as_os_str()
                .to_str()
                .expect("failed to generete lua cfg path")
                .into(),
        ]
    } else {
        vec![
            "-u".into(),
            args.lua_cfg
                .as_os_str()
                .to_str()
                .expect("failed to generete lua cfg path")
                .into(),
        ]
    };

    if koala_mode.is_none() {
        params.append(&mut args.nvim_args.clone());
    }

    let restart_kvim_file_indicator = data_dir.join(Path::new("nvim/restart_kvim"));
    // println!("{:?}", restart_kvim_file_indicator);

    run_kvim(&env, &params);

    // Push restart env value for the next run
    env.push(("KOALA_RESTART".into(), "1".into()));
    loop {
        if !restart_kvim_file_indicator.exists() {
            break; // stop running when restart indicator doesn't exist
        }

        // Remove indicator
        std::fs::remove_file(restart_kvim_file_indicator.clone())
            .expect("failed to remove restart kvim file indicator");

        // Re-run kvim with KOALA_RESTART=1
        run_kvim(&env, &params);
    }
}

fn run_kvim(env: &Vec<(OsString, OsString)>, params: &Vec<OsString>) {
    let mut p = params.clone();
    p.insert(0, "nvim".into());

    // println!("{:?}", p);
    Popen::create(
        &p,
        PopenConfig {
            env: Some(env.clone()),
            ..Default::default()
        },
    )
    .unwrap()
    .wait()
    .unwrap();
}
