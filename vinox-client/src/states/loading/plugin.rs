use bevy::prelude::*;
use bevy_renet::renet::RenetClient;
use vinox_common::world::chunks::storage::{BlockTable, ItemTable};

use crate::states::{
    assets::load::LoadableAssets,
    components::{despawn_with, GameState, Loading},
    game::networking::components::ClientData,
};

use super::ui::{
    disconnect_on_exit, load_blocks, new_client, panic_on_error_system, setup_resources, switch,
    AssetsLoading,
};

pub struct LoadingPlugin;

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClientData::default())
            .insert_resource(BlockTable::default())
            .insert_resource(ItemTable::default())
            .insert_resource(LoadableAssets::default())
            .insert_resource(AssetsLoading::default())
            .add_systems(
                (setup_resources, new_client)
                    .chain()
                    .in_schedule(OnEnter(GameState::Loading)),
            )
            .add_system(disconnect_on_exit.run_if(resource_exists::<RenetClient>()))
            .add_system(panic_on_error_system)
            .add_systems((load_blocks, switch).in_set(OnUpdate(GameState::Loading)))
            .add_system(despawn_with::<Loading>.in_schedule(OnExit(GameState::Loading)));
    }
}
