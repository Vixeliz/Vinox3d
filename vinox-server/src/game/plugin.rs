use bevy::prelude::*;
use vinox_common::world::chunks::storage::BlockTable;

use super::networking::plugin::NetworkingPlugin;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(BlockTable::default())
            .add_plugin(NetworkingPlugin);
    }
}
