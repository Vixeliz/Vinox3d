use std::collections::{HashMap, HashSet};

use bevy::{ecs::system::SystemParam, prelude::*};
use rustc_hash::FxHashSet;

use crate::{
    storage::blocks::descriptor::BlockDescriptor, world::chunks::storage::TOTAL_CHUNK_SIZE,
};

use super::{
    light::{VoxelAddedEvent, VoxelRemovedEvent},
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
    pub fn get_unique_loaded_chunks_and_neighbors(&self, pos_list: &[ChunkPos]) -> Vec<Entity> {
        let mut set: HashSet<Entity> = pos_list
            .iter()
            .filter_map(|pos| self.chunks.get(pos).copied())
            .collect();
        pos_list
            .iter()
            .flat_map(|pos| pos.neighbors())
            .filter_map(|pos| self.chunks.get(&pos).copied())
            .for_each(|entity| {
                set.insert(entity);
            });

        set.into_iter().collect()
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
    pub light_rem_event: EventWriter<'w, VoxelRemovedEvent>,
    pub light_add_event: EventWriter<'w, VoxelAddedEvent>,
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
    // pub fn set_light(&mut self, voxel_pos: IVec3, light: u8) {
    //     let (chunk_pos, local_pos) = global_voxel_positions(voxel_pos);
    //     if let Some(chunk_entity) = self.current_chunks.get_entity(ChunkPos(chunk_pos)) {
    //         if let Ok(mut chunk) = self.chunk_query.get_mut(chunk_entity) {
    //             let index = ChunkData::linearize(local_pos.x, local_pos.y, local_pos.z);
    //             // chunk.set_light(index, light_data);
    //         }
    //     }
    // }
    // pub fn get_light(&mut self, voxel_pos: IVec3) -> LightData {
    //     let (chunk_pos, local_pos) = global_voxel_positions(voxel_pos);
    //     if let Some(chunk_entity) = self.current_chunks.get_entity(ChunkPos(chunk_pos)) {
    //         if let Ok(chunk) = self.chunk_query.get(chunk_entity) {
    //             let index = ChunkData::linearize(
    //                 local_pos.x as usize,
    //                 local_pos.y as usize,
    //                 local_pos.z as usize,
    //             );
    //             chunk.get_light(index);
    //         }
    //     }
    //     LightData::default()
    // }
    // pub fn update_light(&mut self, chunk_pos: ChunkPos, block_table: BlockTable) {
    //     let mut res = Vec::with_capacity(26);
    //     for chunk_entity in self.current_chunks.get_all_neighbors(chunk_pos) {
    //         res.push(chunk_entity);
    //     }
    //     res.push(self.current_chunks.get_entity(chunk_pos).unwrap());
    //     let res = if res.len() == 27 { Some(res) } else { None };
    //     if let Some(neighbors) = res {
    //         if let Ok(neighbors) = neighbors.try_into() {
    //             if let Ok(mut neighbors) = self.chunk_query.get_many_mut::<27>(neighbors) {
    //                 ChunkData::calculate_chunk_lights(&mut neighbors, &block_table);
    //             }
    //         }
    //     }
    // }
    pub fn set_block(&mut self, voxel_pos: IVec3, block: BlockData) {
        let (chunk_pos, local_pos) = global_voxel_positions(voxel_pos);
        if let Some(chunk_entity) = self.current_chunks.get_entity(ChunkPos(chunk_pos)) {
            if let Ok(mut chunk) = self.chunk_query.get_mut(chunk_entity) {
                chunk.set(
                    local_pos.x,
                    local_pos.y,
                    local_pos.z,
                    block.clone(),
                    &self.block_table,
                );
                if block == BlockData::default() {
                    self.light_rem_event.send(VoxelRemovedEvent::new(voxel_pos));
                } else {
                    self.light_add_event
                        .send(VoxelAddedEvent::new(voxel_pos, block));
                }
                // self.update_light(ChunkPos(chunk_pos), self.block_table.clone());
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
                    .get(&chunk.get_identifier(local_pos.x, local_pos.y, local_pos.z))
                    .cloned();
            }
        }
        None
    }

    pub fn get_identifier(&self, voxel_pos: IVec3) -> Option<String> {
        let (chunk_pos, local_pos) = global_voxel_positions(voxel_pos);
        if let Some(chunk_entity) = self.current_chunks.get_entity(ChunkPos(chunk_pos)) {
            if let Ok(chunk) = self.chunk_query.get(chunk_entity) {
                return Some(chunk.get_identifier(local_pos.x, local_pos.y, local_pos.z));
            }
        }
        None
    }

    pub fn get_block(&self, voxel_pos: IVec3) -> Option<BlockData> {
        let (chunk_pos, local_pos) = global_voxel_positions(voxel_pos);
        if let Some(chunk_entity) = self.current_chunks.get_entity(ChunkPos(chunk_pos)) {
            if let Ok(chunk) = self.chunk_query.get(chunk_entity) {
                return Some(chunk.get(local_pos.x, local_pos.y, local_pos.z));
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

pub fn update_chunk_lights(
    mut commands: Commands,
    mut chunks: Query<(&mut ChunkData, &ChunkPos)>,
    chunks_update: Query<Entity, With<ChunkUpdate>>,
    current_chunks: Res<CurrentChunks>,
) {
    for entity in chunks_update.iter() {
        commands.entity(entity).remove::<ChunkUpdate>();
        commands.entity(entity).insert(NeedsMesh);
    }
}

pub fn update_priority_chunk_lights(
    mut commands: Commands,
    mut chunks: Query<Entity, With<PriorityChunkUpdate>>,
) {
    for entity in chunks.iter_mut() {
        commands.entity(entity).remove::<PriorityChunkUpdate>();
        commands.entity(entity).insert(PriorityMesh);
    }
}
