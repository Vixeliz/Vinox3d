use std::collections::HashMap;

use bevy::prelude::*;

use super::storage::RawChunk;

#[derive(Component, Default)]
pub struct RemoveChunk;

#[derive(Resource, Default)]
pub struct CurrentChunks {
    pub chunks: HashMap<IVec3, Entity>,
}

#[derive(Component, Clone, Deref, DerefMut)]
pub struct ChunkPos(pub IVec3);

impl ChunkPos {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self(IVec3::new(x, y, z))
    }

    pub fn neighbors(&self) -> Vec<ChunkPos> {
        vec![
            ChunkPos::new(self.x - 1, self.y - 1, self.z - 1),
            ChunkPos::new(self.x - 1, self.y - 1, self.z),
            ChunkPos::new(self.x - 1, self.y - 1, self.z + 1),
            ChunkPos::new(self.x - 1, self.y, self.z - 1),
            ChunkPos::new(self.x - 1, self.y, self.z),
            ChunkPos::new(self.x - 1, self.y, self.z + 1),
            ChunkPos::new(self.x - 1, self.y + 1, self.z - 1),
            ChunkPos::new(self.x - 1, self.y + 1, self.z),
            ChunkPos::new(self.x - 1, self.y + 1, self.z + 1),
            ChunkPos::new(self.x, self.y - 1, self.z - 1),
            ChunkPos::new(self.x, self.y - 1, self.z),
            ChunkPos::new(self.x, self.y - 1, self.z + 1),
            ChunkPos::new(self.x, self.y, self.z - 1),
            ChunkPos::new(self.x, self.y, self.z + 1),
            ChunkPos::new(self.x, self.y + 1, self.z - 1),
            ChunkPos::new(self.x, self.y + 1, self.z),
            ChunkPos::new(self.x, self.y + 1, self.z + 1),
            ChunkPos::new(self.x + 1, self.y - 1, self.z - 1),
            ChunkPos::new(self.x + 1, self.y - 1, self.z),
            ChunkPos::new(self.x + 1, self.y - 1, self.z + 1),
            ChunkPos::new(self.x + 1, self.y, self.z - 1),
            ChunkPos::new(self.x + 1, self.y, self.z),
            ChunkPos::new(self.x + 1, self.y, self.z + 1),
            ChunkPos::new(self.x + 1, self.y + 1, self.z - 1),
            ChunkPos::new(self.x + 1, self.y + 1, self.z),
            ChunkPos::new(self.x + 1, self.y + 1, self.z + 1),
        ]
    }
}

#[derive(Component)]
pub struct ChunkComp {
    pub pos: ChunkPos,
    pub chunk_data: RawChunk,
    pub entities: Vec<Entity>,
    pub saved_entities: Vec<String>,
}

impl CurrentChunks {
    pub fn insert_entity(&mut self, pos: IVec3, entity: Entity) {
        self.chunks.insert(pos, entity);
    }

    pub fn remove_entity(&mut self, pos: IVec3) -> Option<Entity> {
        self.chunks.remove(&pos)
    }

    pub fn get_entity(&self, pos: IVec3) -> Option<Entity> {
        self.chunks.get(&pos).copied()
    }
    pub fn all_neighbors_exist(&self, pos: ChunkPos) -> bool {
        for chunk in pos.neighbors().iter() {
            if !self.chunks.contains_key(chunk) {
                return false;
            }
        }
        true
    }
    pub fn get_all_neighbors(&self, pos: ChunkPos) -> Vec<Entity> {
        pos.neighbors()
            .iter()
            .filter_map(|this_pos| self.chunks.get(this_pos).copied())
            .collect()
    }
}

#[derive(Default, Resource)]
pub struct ViewRadius {
    pub horizontal: i32,
    pub vertical: i32,
}

#[derive(Default, Resource)]
pub struct SimulationRadius {
    pub horizontal: i32,
    pub vertical: i32,
}
