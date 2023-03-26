
use std::collections::HashMap;


use bevy::prelude::*;

#[derive(Debug, Default, Resource, Deref, DerefMut)]
pub struct LocalGame(pub bool);

#[derive(Debug, Default, Resource)]
pub struct ServerLobby {
    pub players: HashMap<u64, Entity>,
}

#[derive(Debug, Default, Resource, Deref, DerefMut)]
pub struct ChunkLimit(pub usize);
