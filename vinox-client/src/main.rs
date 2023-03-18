pub mod states;
use bevy::{
    // diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
    window::PresentMode,
};
use bevy_renet::RenetClientPlugin;
use bevy_tweening::TweeningPlugin;
use directories::*;
use fs_extra::dir::{copy, CopyOptions};
use ron::de::from_reader;
use states::{
    components::{save_game_options, GameOptions, GameState, ProjectPath},
    game::{plugin::GamePlugin, rendering::meshing::BasicMaterial},
    loading::plugin::LoadingPlugin,
    menu::plugin::MenuPlugin,
};
use std::{
    fs::{create_dir_all, File},
    path::PathBuf,
};

fn main() {
    // Eventually I will implement my own recursive copy and also not delete the assets directory for now though we will completely.
    // Overwrite the data dir assets
    let asset_path = if let Some(proj_dirs) = ProjectDirs::from("com", "vinox", "vinox") {
        let full_path = proj_dirs.data_dir().join("assets");
        create_dir_all(proj_dirs.data_dir()).ok();
        // TODO: This assumes that you are running the client binary from the root of the repo. Eventually when shipping binaries.
        // We should have the assets next to the binary and always get the folder next to the binary
        let copy_options = CopyOptions {
            overwrite: true,
            copy_inside: false,
            ..Default::default()
        };
        if copy("vinox-client/assets", proj_dirs.data_dir(), &copy_options).is_ok() {
        } else {
            error!("Failed to copy assets folder");
        }
        full_path
    } else {
        let mut path = PathBuf::new();
        path.push("assets");
        path
    };
    let final_options = if let Some(game_options) = load_game_options(asset_path.clone()) {
        game_options
    } else {
        save_game_options(GameOptions::default(), asset_path.clone());
        GameOptions::default()
    };
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    asset_folder: asset_path.to_string_lossy().to_string(),
                    watch_for_changes: false,
                })
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Vinox".into(),
                        present_mode: PresentMode::AutoVsync,
                        ..default()
                    }),
                    ..default()
                }),
        )
        // .add_plugin(LogDiagnosticsPlugin::default())
        // .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .insert_resource(ProjectPath(asset_path))
        .insert_resource(final_options)
        .add_plugin(RenetClientPlugin::default())
        .add_plugin(MaterialPlugin::<BasicMaterial>::default())
        .insert_resource(Msaa::Off)
        .add_plugin(TweeningPlugin)
        .add_state::<GameState>()
        .add_plugin(MenuPlugin)
        .add_plugin(LoadingPlugin)
        .add_plugin(GamePlugin)
        .run();
}

fn load_game_options(path: PathBuf) -> Option<GameOptions> {
    let final_path = path.join("config.ron");
    if let Ok(f) = File::open(final_path) {
        let config: Option<GameOptions> = match from_reader(f) {
            Ok(x) => Some(x),
            Err(e) => {
                println!("Failed to load config: {e}");
                None
            }
        };
        config
    } else {
        println!("No such directory!");
        None
    }
}
