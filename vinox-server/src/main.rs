mod game;
use bevy::{
    app::ScheduleRunnerSettings, diagnostic::DiagnosticsPlugin, log::LogPlugin, prelude::*,
};
use bevy_quinnet::server::QuinnetServerPlugin;
use game::{
    plugin::GamePlugin,
    world::storage::{create_database, WorldDatabase},
};
use rusqlite::*;
use std::{
    env,
    sync::{Arc, Mutex},
    time::Duration,
};
use vinox_common::networking::protocol::NetworkIP;

// Server should always keep spawn chunks loaded and any chunks near players
fn main() {
    let args: Vec<String> = env::args().collect();

    let mut ip = "127.0.0.1".to_string();
    match args.len() {
        1 => {}
        2 => {
            ip = args[1].to_string();
        }
        _ => {}
    }

    let database = Connection::open("world.db").unwrap();
    database
        .execute_batch(
            "PRAGMA journal_mode=WAL;
            PRAGMA synchronous=NORMAL;",
        )
        .unwrap();
    create_database(&database);
    App::new()
        .insert_resource(ScheduleRunnerSettings::run_loop(Duration::from_secs_f64(
            1.0 / 60.0,
        )))
        .insert_resource(WorldDatabase {
            name: "world".to_string(),
            connection: Arc::new(Mutex::new(database)),
        })
        .insert_resource(NetworkIP(ip))
        .add_plugins(MinimalPlugins)
        .add_plugin(DiagnosticsPlugin)
        .add_plugin(LogPlugin::default())
        .add_plugin(QuinnetServerPlugin::default())
        .add_plugin(GamePlugin)
        .run();
}
