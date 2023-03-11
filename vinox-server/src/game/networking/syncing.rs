use std::collections::HashMap;

use bevy::prelude::*;
use bevy_quinnet::server::*;
use vinox_common::networking::protocol::ServerMessage;

#[derive(Debug, Default, Resource)]
pub struct ServerLobby {
    pub players: HashMap<u64, Entity>,
}

pub fn connections(
    mut server: ResMut<Server>,
    lobby: Res<ServerLobby>,
    mut connection_events: EventReader<ConnectionEvent>,
) {
    for client in connection_events.iter() {
        // Refuse connection once we already have two players
        if lobby.players.len() >= 8 {
            server.endpoint_mut().disconnect_client(client.id).unwrap();
        } else {
            server
                .endpoint_mut()
                .try_send_message(client.id, ServerMessage::ClientId { id: client.id });
        }
    }
}

pub fn get_messages() {}

pub fn send_entities() {}

pub fn send_chunks() {}
