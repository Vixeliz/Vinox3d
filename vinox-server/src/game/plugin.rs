use bevy::prelude::*;
use vinox_common::{ecs::bundles::PlayerBundleBuilder, world::chunks::storage::BlockTable};

use super::{networking::plugin::NetworkingPlugin, world::chunk::ChunkPlugin};

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(BlockTable::default())
            .insert_resource(PlayerBundleBuilder::default())
            .add_plugin(ChunkPlugin)
            .add_plugin(NetworkingPlugin);
    }
}
