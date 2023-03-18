use bevy::prelude::*;

use super::{
    components::ServerLobby,
    start::{new_server, setup_loadables},
    syncing::{get_messages, send_chunks, send_entities},
};

pub struct NetworkingPlugin;

impl Plugin for NetworkingPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ServerLobby::default())
            .add_startup_system(setup_loadables)
            .add_startup_system(new_server)
            .add_systems(
                (send_chunks, send_entities)
                    .chain()
                    .in_schedule(CoreSchedule::FixedUpdate),
            )
            .add_system(get_messages);
    }
}
