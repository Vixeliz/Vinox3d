pub mod states;
use std::{fs::remove_dir_all, path::PathBuf};

use bevy::prelude::*;
use bevy_quinnet::client::QuinnetClientPlugin;
use directories::*;
use states::{components::GameState, loading::plugin::LoadingPlugin, menu::plugin::MenuPlugin};

fn main() {
    // Eventually I will implement my own recursive copy and also not delete the assets directory for now though we will completely.
    // Overwrite the data dir assets
    let asset_path = if let Some(proj_dirs) = ProjectDirs::from("com", "vinox", "vinox") {
        let full_path = proj_dirs.data_dir().join("assets");
        remove_dir_all(full_path.clone()).ok();
        // TODO: This assumes that you are running the client binary from the root of the repo. Eventually when shipping binaries.
        // We should have the assets next to the binary and always get the folder next to the binary
        copy_dir::copy_dir("vinox-client/assets", full_path.clone()).ok();
        full_path
    } else {
        let mut path = PathBuf::new();
        path.push("assets");
        path
    };
    //TODO: make directory for assets if it doesn't exist and also copy over the game assets to it
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    asset_folder: asset_path.to_string_lossy().to_string(),
                    watch_for_changes: false,
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugin(QuinnetClientPlugin::default())
        .insert_resource(Msaa::Off)
        .add_state::<GameState>()
        .add_plugin(MenuPlugin)
        .add_plugin(LoadingPlugin)
        .run();
}
