use std::{
    cmp::{max, min},
    f32::{INFINITY, NEG_INFINITY},
};

use bevy::{math::Vec3A, prelude::*, render::primitives::Aabb, utils::FloatOrd};

use crate::{
    storage::blocks::descriptor::BlockDescriptor,
    world::chunks::{
        ecs::{ChunkComp, CurrentChunks},
        positions::{voxel_to_global_voxel, world_to_voxel},
        storage::{BlockTable, Chunk, RawChunk, VoxelVisibility},
    },
};

#[derive(Debug)]
pub struct CollisionInfo {
    pub voxel_pos: IVec3,
    pub normal: Vec3,
}

pub fn aabb_vs_world(
    aabb: Aabb,
    chunks: &mut Query<&mut ChunkComp>,
    velocity: Vec3,
    current_chunks: &CurrentChunks,
    block_table: &BlockTable,
) -> Option<Vec<CollisionInfo>> {
    let mut collisions: Vec<CollisionInfo> = Vec::new();
    for x in -5..=5 {
        for y in -5..=5 {
            for z in -5..=5 {
                let (check_chunk_pos, check_block_cpos) = world_to_voxel(
                    Vec3::from(aabb.center) + Vec3::new(x as f32, y as f32, z as f32),
                );
                if let Some(chunk_entity) = current_chunks.get_entity(check_chunk_pos) {
                    if let Ok(chunk) = chunks.get(chunk_entity) {
                        let block_data: BlockDescriptor = chunk
                            .chunk_data
                            .get_data(RawChunk::linearize(check_block_cpos), block_table);
                        let voxel_pos = voxel_to_global_voxel(check_block_cpos, check_chunk_pos);
                        if let Some(block) = block_data.visibility {
                            if block != VoxelVisibility::Empty {
                                let block_aabb = Aabb {
                                    center: (Vec3A::from(voxel_pos.as_vec3())
                                        + Vec3A::new(0.5, 0.5, 0.5)),
                                    half_extents: Vec3::new(0.5, 0.5, 0.5).into(),
                                };

                                let mut inv_enter = Vec3::ZERO;
                                let mut inv_exit = Vec3::ZERO;

                                if velocity.x > 0.0 {
                                    inv_enter.x = block_aabb.min().x - aabb.max().x;
                                    inv_exit.x = block_aabb.max().x - aabb.min().x;
                                } else {
                                    inv_enter.x = block_aabb.max().x - aabb.min().x;
                                    inv_exit.x = block_aabb.min().x - aabb.max().x;
                                }

                                if velocity.y > 0.0 {
                                    inv_enter.y = block_aabb.min().y - aabb.max().y;
                                    inv_exit.y = block_aabb.max().y - aabb.min().y;
                                } else {
                                    inv_enter.y = block_aabb.max().y - aabb.min().y;
                                    inv_exit.y = block_aabb.min().y - aabb.max().y;
                                }

                                if velocity.z > 0.0 {
                                    inv_enter.z = block_aabb.min().z - aabb.max().z;
                                    inv_exit.z = block_aabb.max().z - aabb.min().z;
                                } else {
                                    inv_enter.z = block_aabb.max().z - aabb.min().z;
                                    inv_exit.z = block_aabb.min().z - aabb.max().z;
                                }

                                let mut enter = Vec3::ZERO;
                                let mut exit = Vec3::ZERO;

                                if velocity.x == 0.0 {
                                    if inv_enter.x.signum() == inv_exit.x.signum() {
                                        continue; // Impossible to collide because not already within it on this axis
                                    }
                                    enter.x = NEG_INFINITY;
                                    exit.x = INFINITY;
                                } else {
                                    enter.x = inv_enter.x / velocity.x;
                                    exit.x = inv_exit.x / velocity.x;
                                }
                                if velocity.y == 0.0 {
                                    if inv_enter.y.signum() == inv_exit.y.signum() {
                                        continue; // Impossible to collide because not already within it on this axis
                                    }
                                    enter.y = NEG_INFINITY;
                                    exit.y = INFINITY;
                                } else {
                                    enter.y = inv_enter.y / velocity.y;
                                    exit.y = inv_exit.y / velocity.y;
                                }
                                if velocity.z == 0.0 {
                                    if inv_enter.z.signum() == inv_exit.z.signum() {
                                        continue; // Impossible to collide because not alreadz within it on this axis
                                    }
                                    enter.z = NEG_INFINITY;
                                    exit.z = INFINITY;
                                } else {
                                    enter.z = inv_enter.z / velocity.z;
                                    exit.z = inv_exit.z / velocity.z;
                                }
                                let mut normal = Vec3::ZERO;
                                let enter_time = max(
                                    max(FloatOrd(enter.x), FloatOrd(enter.y)),
                                    FloatOrd(enter.z),
                                );

                                let exit_time =
                                    min(min(FloatOrd(exit.x), FloatOrd(exit.y)), FloatOrd(exit.z));
                                if enter_time > exit_time
                                    || enter.x < 0.0 && enter.y < 0.0 && enter.z < 0.0
                                    || enter.x > 1.0
                                    || enter.y > 1.0
                                    || enter.z > 1.0
                                {
                                    // No collision happens this frame
                                    continue;
                                } else {
                                    // There is a collision this frame
                                    if enter_time.0 == enter.x {
                                        normal.x = if inv_enter.x < 0.0 { 1.0 } else { -1.0 }
                                    } else if enter_time.0 == enter.y {
                                        normal.y = if inv_enter.y < 0.0 { 1.0 } else { -1.0 }
                                    } else if enter_time.0 == enter.z {
                                        normal.z = if inv_enter.z < 0.0 { 1.0 } else { -1.0 }
                                    }
                                    collisions.push(CollisionInfo { voxel_pos, normal });
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    // Remove all detected collisions that are on a face blocked by another block
    collisions.retain(|col| {
        let (chunk_pos, voxel_cpos) = world_to_voxel(col.voxel_pos.as_vec3() + col.normal);
        if let Some(chunk_entity) = current_chunks.get_entity(chunk_pos) {
            if let Ok(chunk) = chunks.get(chunk_entity) {
                let block_data: BlockDescriptor = chunk
                    .chunk_data
                    .get_data(RawChunk::linearize(voxel_cpos), block_table);
                let side_empty = block_data.visibility.unwrap_or(VoxelVisibility::Empty)
                    == VoxelVisibility::Empty;
                return side_empty;
            }
        }
        return true;
    });
    if !collisions.is_empty() {
        return Some(collisions);
    }
    None
}
