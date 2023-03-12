use std::collections::HashMap;

use bevy::prelude::*;

use super::storage::RawChunk;

#[derive(Component, Default)]
pub struct RemoveChunk;

#[derive(Resource, Default)]
pub struct CurrentChunks {
    pub chunks: HashMap<IVec3, Entity>,
}

#[derive(Component, Clone)]
pub struct ChunkPos(pub IVec3);

impl ChunkPos {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self(IVec3::new(x, y, z))
    }

    pub fn neighbors(&self) -> Vec<ChunkPos> {
        vec![
            ChunkPos::new(self.0.x - 1, self.0.y - 1, self.0.z - 1),
            ChunkPos::new(self.0.x - 1, self.0.y - 1, self.0.z),
            ChunkPos::new(self.0.x - 1, self.0.y - 1, self.0.z + 1),
            ChunkPos::new(self.0.x - 1, self.0.y, self.0.z - 1),
            ChunkPos::new(self.0.x - 1, self.0.y, self.0.z),
            ChunkPos::new(self.0.x - 1, self.0.y, self.0.z + 1),
            ChunkPos::new(self.0.x - 1, self.0.y + 1, self.0.z - 1),
            ChunkPos::new(self.0.x - 1, self.0.y + 1, self.0.z),
            ChunkPos::new(self.0.x - 1, self.0.y + 1, self.0.z + 1),
            ChunkPos::new(self.0.x, self.0.y - 1, self.0.z - 1),
            ChunkPos::new(self.0.x, self.0.y - 1, self.0.z),
            ChunkPos::new(self.0.x, self.0.y - 1, self.0.z + 1),
            ChunkPos::new(self.0.x, self.0.y, self.0.z - 1),
            ChunkPos::new(self.0.x, self.0.y, self.0.z + 1),
            ChunkPos::new(self.0.x, self.0.y + 1, self.0.z - 1),
            ChunkPos::new(self.0.x, self.0.y + 1, self.0.z),
            ChunkPos::new(self.0.x, self.0.y + 1, self.0.z + 1),
            ChunkPos::new(self.0.x + 1, self.0.y - 1, self.0.z - 1),
            ChunkPos::new(self.0.x + 1, self.0.y - 1, self.0.z),
            ChunkPos::new(self.0.x + 1, self.0.y - 1, self.0.z + 1),
            ChunkPos::new(self.0.x + 1, self.0.y, self.0.z - 1),
            ChunkPos::new(self.0.x + 1, self.0.y, self.0.z),
            ChunkPos::new(self.0.x + 1, self.0.y, self.0.z + 1),
            ChunkPos::new(self.0.x + 1, self.0.y + 1, self.0.z - 1),
            ChunkPos::new(self.0.x + 1, self.0.y + 1, self.0.z),
            ChunkPos::new(self.0.x + 1, self.0.y + 1, self.0.z + 1),
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
            if !self.chunks.contains_key(&chunk.0) {
                return false;
            }
        }
        true
    }
    pub fn get_all_neighbors(&self, pos: ChunkPos) -> Vec<Entity> {
        pos.neighbors()
            .iter()
            .filter_map(|this_pos| self.chunks.get(&this_pos.0).copied())
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
