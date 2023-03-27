use std::collections::HashMap;

use bevy::prelude::*;

// TODO: Not networking move to different file
#[derive(Debug, Resource, Deref, DerefMut)]
pub struct SaveGame(pub bool);

impl Default for SaveGame {
    fn default() -> Self {
        Self(true)
    }
}

#[derive(Debug, Default, Resource, Deref, DerefMut)]
pub struct LocalGame(pub bool);

#[derive(Debug, Default, Resource)]
pub struct ServerLobby {
    pub players: HashMap<u64, Entity>,
}

#[derive(Debug, Default, Resource, Deref, DerefMut)]
pub struct ChunkLimit(pub usize);
