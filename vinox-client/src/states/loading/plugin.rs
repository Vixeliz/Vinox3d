use bevy::prelude::*;
use vinox_common::world::chunks::storage::BlockTable;

use crate::states::{
    assets::load::LoadableAssets,
    components::{despawn_with, GameState, Loading},
};

use super::ui::{load_blocks, setup_resources, switch, AssetsLoading};

pub struct LoadingPlugin;

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(BlockTable::default())
            .insert_resource(LoadableAssets::default())
            .insert_resource(AssetsLoading::default())
            .add_system(setup_resources.in_schedule(OnEnter(GameState::Loading)))
            .add_system(load_blocks.in_set(OnUpdate(GameState::Loading)))
            .add_system(switch.in_set(OnUpdate(GameState::Loading)))
            .add_system(despawn_with::<Loading>.in_schedule(OnExit(GameState::Loading)));
    }
}
