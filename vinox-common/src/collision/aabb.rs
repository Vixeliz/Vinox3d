use std::{
    cmp::{max, min},
    f32::{INFINITY, NEG_INFINITY},
};

use bevy::{math::Vec3A, prelude::*, render::primitives::Aabb, utils::FloatOrd};

use crate::world::chunks::{
    ecs::{ChunkComp, CurrentChunks},
    positions::{world_to_chunk, world_to_offsets},
    storage::{BlockTable, Chunk, RawChunk, VoxelVisibility},
};

pub fn aabb_vs_world(
    position: Vec3,
    aabb: Aabb,
    chunks: &Query<&mut ChunkComp>,
    velocity: Vec3,
    current_chunks: &CurrentChunks,
    block_table: &BlockTable,
) -> Option<Vec<(f32, Vec3)>> {
    // TODO: Get neighboring chunks
    let mut collisions = Vec::new();
    if let Some(chunk_entity) = current_chunks.get_entity(world_to_chunk(position)) {
        if let Ok(chunk) = chunks.get(chunk_entity) {
            for x in -1..=1 {
                for y in -1..=1 {
                    for z in -1..=1 {
                        if let Some(block) = chunk
                            .chunk_data
                            .get_data(
                                RawChunk::linearize(world_to_offsets(
                                    position + Vec3::new(x as f32, y as f32, z as f32),
                                )),
                                block_table,
                            )
                            .visibility
                        {
                            if block != VoxelVisibility::Empty {
                                let block_aabb = Aabb {
                                    center: (position + Vec3::new(x as f32, y as f32, z as f32))
                                        .into(),
                                    half_extents: Vec3A {
                                        x: 0.5,
                                        y: 0.5,
                                        z: 0.5,
                                    },
                                };
                                let mut inv_enter = Vec3::ZERO;
                                let mut inv_exit = Vec3::ZERO;
                                if velocity.x > 0.0 {
                                    inv_enter.x = block_aabb.max().x
                                        - (aabb.max().x + (aabb.half_extents.x * 2.0));
                                    inv_exit.x = (block_aabb.max().x
                                        + block_aabb.half_extents.x * 2.0)
                                        - aabb.max().x;
                                } else {
                                    inv_enter.x = (block_aabb.max().x
                                        + block_aabb.half_extents.x * 2.0)
                                        - aabb.max().x;
                                    inv_exit.x = block_aabb.max().x
                                        - (aabb.max().x + aabb.half_extents.x * 2.0);
                                }

                                if velocity.y > 0.0 {
                                    inv_enter.y = block_aabb.max().y
                                        - (aabb.max().y + aabb.half_extents.y * 2.0);
                                    inv_exit.y = (block_aabb.max().y
                                        + block_aabb.half_extents.y * 2.0)
                                        - aabb.max().y;
                                } else {
                                    inv_enter.y = (block_aabb.max().y
                                        + block_aabb.half_extents.y * 2.0)
                                        - aabb.max().y;
                                    inv_exit.y = block_aabb.max().y
                                        - (aabb.max().y + aabb.half_extents.y * 2.0);
                                }
                                if velocity.z > 0.0 {
                                    inv_enter.z = block_aabb.max().z
                                        - (aabb.max().z + (aabb.half_extents.z * 2.0));
                                    inv_exit.z = (block_aabb.max().z
                                        + block_aabb.half_extents.z * 2.0)
                                        - aabb.max().z;
                                } else {
                                    inv_enter.z = (block_aabb.max().z
                                        + block_aabb.half_extents.z * 2.0)
                                        - aabb.max().z;
                                    inv_exit.z = block_aabb.max().z
                                        - (aabb.max().z + aabb.half_extents.z * 2.0);
                                }

                                let mut enter = Vec3::ZERO;
                                let mut exit = Vec3::ZERO;

                                if velocity.x == 0.0 {
                                    enter.x = NEG_INFINITY;
                                    exit.x = INFINITY;
                                } else {
                                    enter.x = inv_enter.x / velocity.x;
                                    exit.x = inv_exit.x / velocity.x;
                                }

                                if velocity.y == 0.0 {
                                    enter.y = NEG_INFINITY;
                                    exit.y = INFINITY;
                                } else {
                                    enter.y = inv_enter.y / velocity.y;
                                    exit.y = inv_exit.y / velocity.y;
                                }
                                if velocity.z == 0.0 {
                                    enter.z = NEG_INFINITY;
                                    exit.z = INFINITY;
                                } else {
                                    enter.z = inv_enter.z / velocity.z;
                                    exit.z = inv_exit.z / velocity.z;
                                }
                                let mut normal = Vec3::ZERO;
                                let enter_time =
                                    max(max(FloatOrd(enter.x), FloatOrd(exit.y)), FloatOrd(exit.z));
                                let exit_time =
                                    min(min(FloatOrd(exit.x), FloatOrd(exit.y)), FloatOrd(exit.z));
                                if enter_time > exit_time
                                    || enter.x < 0.0 && enter.y < 0.0 && enter.z < 0.0
                                    || enter.x > 1.0
                                    || enter.y > 1.0
                                    || enter.z > 1.0
                                {
                                    continue;
                                    // return None;
                                } else {
                                    if enter.x > enter.y && enter.x > enter.z {
                                        if inv_enter.x < 0.0 {
                                            normal.x = 1.0;
                                            normal.y = 0.0;
                                            normal.z = 0.0;
                                        } else {
                                            normal.x = -1.0;
                                            normal.y = 0.0;
                                            normal.z = 0.0;
                                        }
                                    } else if enter.z > enter.x && enter.z > enter.y {
                                        if inv_enter.z < 0.0 {
                                            normal.x = 0.0;
                                            normal.y = 0.0;
                                            normal.z = 1.0;
                                        } else {
                                            normal.x = 0.0;
                                            normal.y = 0.0;
                                            normal.z = -1.0;
                                        }
                                    } else if enter.y > enter.x && enter.y > enter.z {
                                        if inv_enter.y < 0.0 {
                                            normal.x = 0.0;
                                            normal.y = 1.0;
                                            normal.z = 0.0;
                                        } else {
                                            normal.x = 0.0;
                                            normal.y = -1.0;
                                            normal.z = 0.0;
                                        }
                                    }
                                    collisions.push((enter_time.0, normal));
                                }
                            }
                        }
                    }
                }
            }
            if !collisions.is_empty() {
                return Some(collisions);
            }
        }
    }
    None
}
