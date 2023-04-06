use std::collections::HashMap;

use bevy::prelude::*;

#[derive(Resource, Default, Deref, DerefMut)]
pub struct Password(pub String);

#[derive(Resource, Default, Deref, DerefMut)]
pub struct ChatMessages(pub Vec<(String, String)>);

#[derive(Resource, Default, Deref, DerefMut)]
pub struct ClientData(pub u64);

#[derive(Debug)]
pub struct PlayerInfo {
    pub client_entity: Entity,
    pub server_entity: Entity,
}

#[derive(Debug, Default, Resource)]
pub struct ClientLobby {
    pub players: HashMap<u64, PlayerInfo>,
}

#[derive(Default, Resource, Deref, DerefMut)]
pub struct NetworkMapping(pub HashMap<Entity, Entity>);
