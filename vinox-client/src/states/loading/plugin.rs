use bevy::prelude::*;
use vinox_common::world::chunks::storage::{BlockTable, ItemTable};

use crate::states::{
    assets::load::LoadableAssets,
    components::{despawn_with, GameState, Loading},
    game::networking::components::ClientData,
};

use super::ui::{load_blocks, new_client, setup_resources, switch, AssetsLoading};

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
            .add_systems((load_blocks, switch).in_set(OnUpdate(GameState::Loading)))
            .add_system(despawn_with::<Loading>.in_schedule(OnExit(GameState::Loading)));
    }
}
