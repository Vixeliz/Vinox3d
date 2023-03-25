use std::collections::HashMap;

use bevy::{ecs::system::SystemParam, prelude::*};
use rustc_hash::FxHashSet;

use crate::{
    storage::blocks::descriptor::BlockDescriptor, world::chunks::storage::TOTAL_CHUNK_SIZE,
};

use super::{
    positions::{global_voxel_positions, ChunkPos},
    storage::{BlockData, BlockTable, ChunkData, CHUNK_SIZE, CHUNK_SIZE_ARR},
};

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
    commands: Commands<'w, 's>,
    pub current_chunks: ResMut<'w, CurrentChunks>,
    // chunk_queue: ResMut<'w, ChunkQueue>,
    pub view_radius: Res<'w, ViewRadius>,
    pub chunk_query: Query<'w, 's, &'static mut ChunkData>,
    pub block_table: Res<'w, BlockTable>,
}

#[derive(Component, Clone)]
pub struct SentChunks {
    pub chunks: FxHashSet<ChunkPos>,
}

#[derive(Component, Default)]
pub struct ChunkUpdate;

#[derive(Component, Default)]
pub struct NeedsMesh;

#[derive(Component, Default)]
pub struct PriorityChunkUpdate;

#[derive(Component, Default)]
pub struct PriorityMesh;

impl<'w, 's> ChunkManager<'w, 's> {
    pub fn get_chunk(&self, enity: Entity) -> Option<ChunkData> {
        if let Ok(chunk) = self.chunk_query.get(enity) {
            return Some(chunk.clone());
        }
        None
    }
    pub fn set_block(&mut self, voxel_pos: IVec3, block: BlockData) {
        let (chunk_pos, local_pos) = global_voxel_positions(voxel_pos);
        if let Some(chunk_entity) = self.current_chunks.get_entity(ChunkPos(chunk_pos)) {
            if let Ok(mut chunk) = self.chunk_query.get_mut(chunk_entity) {
                chunk.set(
                    local_pos.x as usize,
                    local_pos.y as usize,
                    local_pos.z as usize,
                    block,
                    &self.block_table,
                );
                match local_pos.x {
                    0 => {
                        if let Some(neighbor_chunk) = self
                            .current_chunks
                            .get_entity(ChunkPos(chunk_pos + IVec3::new(-1, 0, 0)))
                        {
                            self.commands
                                .entity(neighbor_chunk)
                                .insert(PriorityChunkUpdate);
                        }
                    }
                    CHUNK_SIZE_ARR => {
                        if let Some(neighbor_chunk) = self
                            .current_chunks
                            .get_entity(ChunkPos(chunk_pos + IVec3::new(1, 0, 0)))
                        {
                            self.commands
                                .entity(neighbor_chunk)
                                .insert(PriorityChunkUpdate);
                        }
                    }
                    _ => {}
                }
                match local_pos.y {
                    0 => {
                        if let Some(neighbor_chunk) = self
                            .current_chunks
                            .get_entity(ChunkPos(chunk_pos + IVec3::new(0, -1, 0)))
                        {
                            self.commands
                                .entity(neighbor_chunk)
                                .insert(PriorityChunkUpdate);
                        }
                    }
                    CHUNK_SIZE_ARR => {
                        if let Some(neighbor_chunk) = self
                            .current_chunks
                            .get_entity(ChunkPos(chunk_pos + IVec3::new(0, 1, 0)))
                        {
                            self.commands
                                .entity(neighbor_chunk)
                                .insert(PriorityChunkUpdate);
                        }
                    }
                    _ => {}
                }
                match local_pos.z {
                    0 => {
                        if let Some(neighbor_chunk) = self
                            .current_chunks
                            .get_entity(ChunkPos(chunk_pos + IVec3::new(0, 0, -1)))
                        {
                            self.commands
                                .entity(neighbor_chunk)
                                .insert(PriorityChunkUpdate);
                        }
                    }
                    CHUNK_SIZE_ARR => {
                        if let Some(neighbor_chunk) = self
                            .current_chunks
                            .get_entity(ChunkPos(chunk_pos + IVec3::new(0, 0, 1)))
                        {
                            self.commands
                                .entity(neighbor_chunk)
                                .insert(PriorityChunkUpdate);
                        }
                    }
                    _ => {}
                }
                self.commands
                    .entity(chunk_entity)
                    .insert(PriorityChunkUpdate);
            }
        }
    }

    pub fn get_descriptor(&self, voxel_pos: IVec3) -> Option<BlockDescriptor> {
        let (chunk_pos, local_pos) = global_voxel_positions(voxel_pos);
        if let Some(chunk_entity) = self.current_chunks.get_entity(ChunkPos(chunk_pos)) {
            if let Ok(chunk) = self.chunk_query.get(chunk_entity) {
                return self
                    .block_table
                    .get(&chunk.get_identifier(
                        local_pos.x as usize,
                        local_pos.y as usize,
                        local_pos.z as usize,
                    ))
                    .cloned();
            }
        }
        None
    }

    pub fn get_identifier(&self, voxel_pos: IVec3) -> Option<String> {
        let (chunk_pos, local_pos) = global_voxel_positions(voxel_pos);
        if let Some(chunk_entity) = self.current_chunks.get_entity(ChunkPos(chunk_pos)) {
            if let Ok(chunk) = self.chunk_query.get(chunk_entity) {
                return Some(chunk.get_identifier(
                    local_pos.x as usize,
                    local_pos.y as usize,
                    local_pos.z as usize,
                ));
            }
        }
        None
    }

    pub fn get_block(&self, voxel_pos: IVec3) -> Option<BlockData> {
        let (chunk_pos, local_pos) = global_voxel_positions(voxel_pos);
        if let Some(chunk_entity) = self.current_chunks.get_entity(ChunkPos(chunk_pos)) {
            if let Ok(chunk) = self.chunk_query.get(chunk_entity) {
                return Some(chunk.get(
                    local_pos.x as usize,
                    local_pos.y as usize,
                    local_pos.z as usize,
                ));
            }
        }
        None
    }
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

// Chunk Update is for common update stuff such as lights.
// Needs mesh is for the client and can be ignored when put on server components
pub fn update_chunk_lights(
    mut commands: Commands,
    mut chunks: Query<(&mut ChunkData, &ChunkPos, Entity), With<ChunkUpdate>>,
    chunk_query: Query<&ChunkData, Without<ChunkUpdate>>,
    current_chunks: Res<CurrentChunks>,
) {
    for (chunk, chunk_pos, entity) in chunks.iter() {
        let mut neighbors = Vec::with_capacity(26);
        for neighbor_pos in chunk_pos.neighbors() {
            if let Some(chunk_entity) = current_chunks.get_entity(neighbor_pos) {
                if let Ok(chunk) = chunk_query.get(chunk_entity) {
                    neighbors.push((chunk.clone(), neighbor_pos));
                }
            }
        }
        commands.entity(entity).remove::<ChunkUpdate>();
        commands.entity(entity).insert(NeedsMesh);
    }
}

pub fn update_priority_chunk_lights(
    mut commands: Commands,
    mut chunks: Query<(&mut ChunkData, &ChunkPos, Entity), With<PriorityChunkUpdate>>,
    chunk_query: Query<&ChunkData, Without<PriorityChunkUpdate>>,
    current_chunks: Res<CurrentChunks>,
    block_table: Res<BlockTable>,
) {
    for (mut chunk, chunk_pos, entity) in chunks.iter_mut() {
        let mut neighbors = Vec::with_capacity(26);
        if current_chunks.all_neighbors_exist(*chunk_pos) {
            for neighbor_pos in chunk_pos.neighbors() {
                if let Some(chunk_entity) = current_chunks.get_entity(neighbor_pos) {
                    if let Ok(chunk) = chunk_query.get(chunk_entity) {
                        neighbors.push((chunk.clone(), neighbor_pos));
                    }
                }
            }

            const MAX: usize = CHUNK_SIZE;
            const BOUND: usize = MAX + 1;
            for x in 0..CHUNK_SIZE {
                for y in 0..CHUNK_SIZE {
                    for z in 0..CHUNK_SIZE {
                        match (x, y, z) {
                            (0, 0, 0) => {
                                let light = neighbors[0].0.get_light(ChunkData::linearize(
                                    MAX - 1,
                                    MAX - 1,
                                    MAX - 1,
                                ));
                                chunk.set_light(ChunkData::linearize(x, y, z), light);
                            }
                            (0, 0, 1..=MAX) => {
                                let light = neighbors[1].0.get_light(ChunkData::linearize(
                                    MAX - 1,
                                    MAX - 1,
                                    z - 1,
                                ));
                                chunk.set_light(ChunkData::linearize(x, y, z), light);
                            }
                            (0, 0, BOUND) => {
                                let light = neighbors[2].0.get_light(ChunkData::linearize(
                                    MAX - 1,
                                    MAX - 1,
                                    0,
                                ));
                                chunk.set_light(ChunkData::linearize(x, y, z), light);
                            }
                            (0, 1..=MAX, 0) => {
                                let light = neighbors[3].0.get_light(ChunkData::linearize(
                                    MAX - 1,
                                    y - 1,
                                    MAX - 1,
                                ));
                                chunk.set_light(ChunkData::linearize(x, y, z), light);
                            }
                            (0, 1..=MAX, 1..=MAX) => {
                                let light = neighbors[4].0.get_light(ChunkData::linearize(
                                    MAX - 1,
                                    y - 1,
                                    z - 1,
                                ));
                                chunk.set_light(ChunkData::linearize(x, y, z), light);
                            }
                            (0, 1..=MAX, BOUND) => {
                                let light = neighbors[5].0.get_light(ChunkData::linearize(
                                    MAX - 1,
                                    y - 1,
                                    0,
                                ));
                                chunk.set_light(ChunkData::linearize(x, y, z), light);
                            }
                            (0, BOUND, 0) => {
                                let light = neighbors[6].0.get_light(ChunkData::linearize(
                                    MAX - 1,
                                    0,
                                    MAX - 1,
                                ));
                                chunk.set_light(ChunkData::linearize(x, y, z), light);
                            }
                            (0, BOUND, 1..=MAX) => {
                                let light = neighbors[7].0.get_light(ChunkData::linearize(
                                    MAX - 1,
                                    0,
                                    z - 1,
                                ));
                                chunk.set_light(ChunkData::linearize(x, y, z), light);
                            }
                            (0, BOUND, BOUND) => {
                                let light =
                                    neighbors[8]
                                        .0
                                        .get_light(ChunkData::linearize(MAX - 1, 0, 0));
                                chunk.set_light(ChunkData::linearize(x, y, z), light);
                            }
                            (1..=MAX, 0, 0) => {
                                let light = neighbors[9].0.get_light(ChunkData::linearize(
                                    x - 1,
                                    MAX - 1,
                                    MAX - 1,
                                ));
                                chunk.set_light(ChunkData::linearize(x, y, z), light);
                            }
                            (1..=MAX, 0, 1..=MAX) => {
                                let light = neighbors[10].0.get_light(ChunkData::linearize(
                                    x - 1,
                                    MAX - 1,
                                    z - 1,
                                ));
                                chunk.set_light(ChunkData::linearize(x, y, z), light);
                            }
                            (1..=MAX, 0, BOUND) => {
                                let light = neighbors[11].0.get_light(ChunkData::linearize(
                                    x - 1,
                                    MAX - 1,
                                    0,
                                ));
                                chunk.set_light(ChunkData::linearize(x, y, z), light);
                            }
                            (1..=MAX, 1..=MAX, 0) => {
                                let light = neighbors[12].0.get_light(ChunkData::linearize(
                                    x - 1,
                                    y - 1,
                                    MAX - 1,
                                ));
                                chunk.set_light(ChunkData::linearize(x, y, z), light);
                            }
                            // (1..=MAX, 1..=MAX, 1..=MAX) => {
                            //     let light =&center, x - 1, y - 1, z - 1;
                            // }
                            (1..=MAX, 1..=MAX, BOUND) => {
                                let light = neighbors[13].0.get_light(ChunkData::linearize(
                                    x - 1,
                                    y - 1,
                                    0,
                                ));
                                chunk.set_light(ChunkData::linearize(x, y, z), light);
                            }
                            (1..=MAX, BOUND, 0) => {
                                let light = neighbors[14].0.get_light(ChunkData::linearize(
                                    x - 1,
                                    0,
                                    MAX - 1,
                                ));
                                chunk.set_light(ChunkData::linearize(x, y, z), light);
                            }
                            (1..=MAX, BOUND, 1..=MAX) => {
                                let light = neighbors[15].0.get_light(ChunkData::linearize(
                                    x - 1,
                                    0,
                                    z - 1,
                                ));
                                chunk.set_light(ChunkData::linearize(x, y, z), light);
                            }
                            (1..=MAX, BOUND, BOUND) => {
                                let light =
                                    neighbors[16].0.get_light(ChunkData::linearize(x - 1, 0, 0));
                                chunk.set_light(ChunkData::linearize(x, y, z), light);
                            }
                            (BOUND, 0, 0) => {
                                let light = neighbors[17].0.get_light(ChunkData::linearize(
                                    0,
                                    MAX - 1,
                                    MAX - 1,
                                ));
                                chunk.set_light(ChunkData::linearize(x, y, z), light);
                            }
                            (BOUND, 0, 1..=MAX) => {
                                let light = neighbors[18].0.get_light(ChunkData::linearize(
                                    0,
                                    MAX - 1,
                                    z - 1,
                                ));
                                chunk.set_light(ChunkData::linearize(x, y, z), light);
                            }
                            (BOUND, 0, BOUND) => {
                                let light =
                                    neighbors[19]
                                        .0
                                        .get_light(ChunkData::linearize(0, MAX - 1, 0));
                                chunk.set_light(ChunkData::linearize(x, y, z), light);
                            }
                            (BOUND, 1..=MAX, 0) => {
                                let light = neighbors[20].0.get_light(ChunkData::linearize(
                                    0,
                                    y - 1,
                                    MAX - 1,
                                ));
                                chunk.set_light(ChunkData::linearize(x, y, z), light);
                            }
                            (BOUND, 1..=MAX, 1..=MAX) => {
                                let light = neighbors[21].0.get_light(ChunkData::linearize(
                                    0,
                                    y - 1,
                                    z - 1,
                                ));
                                chunk.set_light(ChunkData::linearize(x, y, z), light);
                            }
                            (BOUND, 1..=MAX, BOUND) => {
                                let light =
                                    neighbors[22].0.get_light(ChunkData::linearize(0, y - 1, 0));
                                chunk.set_light(ChunkData::linearize(x, y, z), light);
                            }
                            (BOUND, BOUND, 0) => {
                                let light =
                                    neighbors[23]
                                        .0
                                        .get_light(ChunkData::linearize(0, 0, MAX - 1));
                                chunk.set_light(ChunkData::linearize(x, y, z), light);
                            }
                            (BOUND, BOUND, 1..=MAX) => {
                                let light =
                                    neighbors[24].0.get_light(ChunkData::linearize(0, 0, z - 1));
                                chunk.set_light(ChunkData::linearize(x, y, z), light);
                            }
                            (BOUND, BOUND, BOUND) => {
                                let light =
                                    neighbors[25].0.get_light(ChunkData::linearize(0, 0, 0));
                                chunk.set_light(ChunkData::linearize(x, y, z), light);
                            }

                            (_, _, _) => {}
                        }
                    }
                }
            }
            chunk.calculate_all_light(&block_table);
            commands.entity(entity).remove::<PriorityChunkUpdate>();
            commands.entity(entity).insert(PriorityMesh);
        }
    }
}
