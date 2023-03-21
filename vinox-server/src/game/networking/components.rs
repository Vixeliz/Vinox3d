use rustc_data_structures::stable_set::FxHashSet;
use std::collections::HashMap;

use bevy::prelude::*;

#[derive(Component, Clone)]
pub struct SentChunks {
    pub chunks: FxHashSet<IVec3>,
}

#[derive(Debug, Default, Resource)]
pub struct ServerLobby {
    pub players: HashMap<u64, Entity>,
}

#[derive(Debug, Default, Resource, Deref, DerefMut)]
pub struct ChunkLimit(pub usize);
