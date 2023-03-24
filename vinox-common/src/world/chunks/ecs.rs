use std::collections::HashMap;

use bevy::{ecs::system::SystemParam, prelude::*};
use rustc_hash::FxHashSet;

use super::{positions::ChunkPos, storage::ChunkData};

#[derive(Component, Default)]
pub struct RemoveChunk;

#[derive(Resource, Default)]
pub struct CurrentChunks {
    pub chunks: HashMap<ChunkPos, Entity>,
}

impl CurrentChunks {
    pub fn insert_entity(&mut self, pos: ChunkPos, entity: Entity) {
        self.chunks.insert(pos, entity);
    }

    pub fn remove_entity(&mut self, pos: ChunkPos) -> Option<Entity> {
        self.chunks.remove(&pos)
    }

    pub fn get_entity(&self, pos: ChunkPos) -> Option<Entity> {
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

#[derive(SystemParam)]
pub struct ChunkManager<'w, 's> {
    // commands: Commands<'w, 's>,
    pub current_chunks: ResMut<'w, CurrentChunks>,
    // chunk_queue: ResMut<'w, ChunkQueue>,
    pub view_radius: Res<'w, ViewRadius>,
    pub chunk_query: Query<'w, 's, &'static ChunkData>,
}

#[derive(Component, Clone)]
pub struct SentChunks {
    pub chunks: FxHashSet<ChunkPos>,
}

impl<'w, 's> ChunkManager<'w, 's> {
    pub fn get_chunk_positions(&mut self, chunk_pos: ChunkPos) -> Vec<ChunkPos> {
        let mut chunks = Vec::new();
        for z in -self.view_radius.horizontal..=self.view_radius.horizontal {
            for x in -self.view_radius.horizontal..=self.view_radius.horizontal {
                for y in -self.view_radius.vertical..=self.view_radius.vertical {
                    let pos = *chunk_pos + IVec3::new(x, y, z);
                    chunks.push(ChunkPos(pos));
                }
            }
        }
        // chunks
        //     .sort_unstable_by_key(|key| (key.x - chunk_pos.x).abs() + (key.z - chunk_pos.z).abs());
        chunks
    }
    pub fn get_chunks_around_chunk(
        &mut self,
        pos: ChunkPos,
        sent_chunks: Option<&SentChunks>,
    ) -> Vec<(&ChunkData, ChunkPos)> {
        let mut res = Vec::new();
        for chunk_pos in self.get_chunk_positions(pos).iter() {
            if let Some(sent_chunks) = sent_chunks {
                if !sent_chunks.chunks.contains(chunk_pos) {
                    if let Some(entity) = self.current_chunks.get_entity(*chunk_pos) {
                        if let Ok(chunk) = self.chunk_query.get(entity) {
                            res.push((chunk, *chunk_pos));
                        }
                    }
                }
            } else {
                if let Some(entity) = self.current_chunks.get_entity(*chunk_pos) {
                    if let Ok(chunk) = self.chunk_query.get(entity) {
                        res.push((chunk, *chunk_pos));
                    }
                }
            }
        }

        res
    }

    pub fn get_neighbors(&self, pos: ChunkPos) -> Option<Vec<ChunkData>> {
        let mut res = Vec::with_capacity(26);
        for chunk_entity in self.current_chunks.get_all_neighbors(pos) {
            if let Ok(chunk) = self.chunk_query.get(chunk_entity) {
                res.push(chunk.clone())
            }
        }
        if res.len() == 26 {
            Some(res)
        } else {
            None
        }
    }
}
