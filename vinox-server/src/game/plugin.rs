use bevy::prelude::*;
use vinox_common::{
    ecs::bundles::PlayerBundleBuilder,
    world::chunks::{
        ecs::CommonPlugin,
        light::LightPlugin,
        storage::{BlockTable, ItemTable, RecipeTable},
    },
};

use super::{networking::plugin::NetworkingPlugin, world::chunk::ChunkPlugin};

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ItemTable::default())
            .insert_resource(BlockTable::default())
            .insert_resource(RecipeTable::default())
            .insert_resource(PlayerBundleBuilder::default())
            .add_plugin(CommonPlugin)
            .add_plugin(ChunkPlugin)
            .add_plugin(NetworkingPlugin)
            .add_plugin(LightPlugin);
    }
}
