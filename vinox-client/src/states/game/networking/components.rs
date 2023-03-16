use std::collections::HashMap;

use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct ClientData(pub u64);

#[derive(Resource, Default)]
pub struct UserName(pub String);

#[derive(Debug)]
pub struct PlayerInfo {
    pub client_entity: Entity,
    pub server_entity: Entity,
}

#[derive(Debug, Default, Resource)]
pub struct ClientLobby {
    pub players: HashMap<u64, PlayerInfo>,
}

#[derive(Default, Resource)]
pub struct NetworkMapping(pub HashMap<Entity, Entity>);
