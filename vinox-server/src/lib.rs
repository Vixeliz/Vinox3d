mod game;
use bevy::{
    app::ScheduleRunnerSettings, diagnostic::DiagnosticsPlugin, log::LogPlugin, prelude::*,
};
use bevy_quinnet::server::QuinnetServerPlugin;
use directories::*;
use game::{
    plugin::GamePlugin,
    world::storage::{create_database, WorldDatabase, WorldInfo},
};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rand::Rng;
use ron::de::from_reader;
use ron::ser::{to_string_pretty, PrettyConfig};
use std::io::Write;
use std::{
    env,
    fs::{create_dir_all, File},
    path::PathBuf,
    time::Duration,
};
use vinox_common::networking::protocol::NetworkIP;

// Server should always keep spawn chunks loaded and any chunks near players
pub fn create_server() {
    let mut asset_path = if let Some(proj_dirs) = ProjectDirs::from("com", "vinox", "vinox") {
        let full_path = proj_dirs.data_dir().join("assets");
        create_dir_all(proj_dirs.data_dir()).ok();
        // TODO: This assumes that you are running the client binary from the root of the repo. Eventually when shipping binaries.
        // We should have the assets next to the binary and always get the folder next to the binary
        full_path
    } else {
        let mut path = PathBuf::new();
        path.push("assets");
        path
    };

    let args: Vec<String> = env::args().collect();

    let mut ip = "127.0.0.1".to_string();
    let mut world_name = "world".to_string();
    // TODO: Better arg parser eventually something like clap
    match args.len() {
        1 => {}
        2 => {
            ip = args[1].to_string();
        }
        3 => {
            ip = args[1].to_string();
            world_name = args[2].to_string();
        }
        _ => {}
    }
    let mut final_world_name = "worlds/".to_string();
    final_world_name.push_str(&world_name);
    asset_path.push(final_world_name);
    let final_world_info = if let Some(world_info) =
        load_world_info(format!("{}.ron", asset_path.clone().display()).into())
    {
        world_info
    } else {
        let world = WorldInfo {
            name: world_name.clone(),
            seed: rand::thread_rng().gen_range(0..=u32::MAX),
            damage: false,
        };
        save_world_info(
            world.clone(),
            format!("{}.ron", asset_path.clone().display()).into(),
        );
        world
    };
    let manager = SqliteConnectionManager::file(format!("{}.db", asset_path.display()));
    let pool = Pool::builder()
        .max_size(30)
        .test_on_check_out(false)
        .build(manager)
        .unwrap();
    pool.get()
        .unwrap()
        .execute_batch(
            "PRAGMA journal_mode=WAL;
             PRAGMA synchronous=NORMAL;",
        )
        .ok();
    create_database(&pool.get().unwrap());
    App::new()
        .insert_resource(ScheduleRunnerSettings::run_loop(Duration::from_secs_f64(
            1.0 / 60.0,
        )))
        .insert_resource(final_world_info)
        .insert_resource(WorldDatabase { connection: pool })
        .insert_resource(NetworkIP(ip))
        .add_plugins(MinimalPlugins)
        .add_plugin(DiagnosticsPlugin)
        .add_plugin(LogPlugin::default())
        .add_plugin(QuinnetServerPlugin::default())
        .add_plugin(GamePlugin)
        .run();
}

pub fn save_world_info(world_info: WorldInfo, path: PathBuf) {
    if create_dir_all(path.parent().unwrap()).is_err() {
        println!("Failed to create {:?} directory!", path.parent());
        return;
    }
    if let Ok(mut output) = File::create(path.clone()) {
        let pretty = PrettyConfig::new()
            .depth_limit(2)
            .separate_tuple_members(true)
            .enumerate_arrays(true);
        let s = to_string_pretty(&world_info, pretty).ok().unwrap();
        write!(output, "{s}").ok();
    } else {
        println!("Failed to save world at path {path:?}!");
    }
}

fn load_world_info(path: PathBuf) -> Option<WorldInfo> {
    if let Ok(f) = File::open(path) {
        let world_info: Option<WorldInfo> = match from_reader(f) {
            Ok(x) => Some(x),
            Err(e) => {
                println!("Failed to load world_info: {e}");
                None
            }
        };
        world_info
    } else {
        println!("No such directory!");
        None
    }
}
