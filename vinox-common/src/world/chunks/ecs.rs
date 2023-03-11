use std::collections::HashMap;

use bevy::prelude::*;

use super::storage::RawChunk;

#[derive(Component, Default)]
pub struct RemoveChunk;

#[derive(Resource, Default)]
pub struct CurrentChunks {
    pub chunks: HashMap<IVec3, Entity>,
}

#[derive(Component)]
pub struct ChunkPos(pub IVec3);

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
    pub fn all_neighbors_exist(&self, pos: IVec3, _min_bound: IVec2, _max_bound: IVec2) -> bool {
        self.chunks.contains_key(&(pos + IVec3::new(0, 1, 0)))
            && self.chunks.contains_key(&(pos + IVec3::new(0, -1, 0)))
            && self.chunks.contains_key(&(pos + IVec3::new(1, 0, 0)))
            && self.chunks.contains_key(&(pos + IVec3::new(-1, 0, 0)))
            && self.chunks.contains_key(&(pos + IVec3::new(0, 0, 1)))
            && self.chunks.contains_key(&(pos + IVec3::new(0, 0, -1)))
    }
}

#[derive(Default, Resource)]
pub struct ViewDistance {
    pub radius: i32,
}

#[derive(Default, Resource)]
pub struct SimulationDistance {
    pub radius: i32,
}
