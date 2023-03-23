use std::{
    cmp::{max, min},
    f32::{INFINITY, NEG_INFINITY},
};

use bevy::{math::Vec3A, prelude::*, render::primitives::Aabb, utils::FloatOrd};

use crate::world::chunks::{
    ecs::CurrentChunks,
    positions::{voxel_to_global_voxel, world_to_voxel, ChunkPos},
    storage::{BlockData, BlockTable, ChunkData},
};

#[derive(Debug)]
pub struct CollisionInfo {
    pub voxel_pos: IVec3,
    pub normal: Vec3,
    pub dist: f32,
}

pub fn aabb_vs_world(
    aabb: Aabb,
    chunks: &Query<&ChunkData>,
    velocity: Vec3,
    current_chunks: &CurrentChunks,
    block_table: &BlockTable,
) -> Option<Vec<CollisionInfo>> {
    let mut collisions: Vec<CollisionInfo> = Vec::new();
    // Extend the area to check for collisions to what can be conceivably reached beased on velocity
    let area_to_check = (
        (Vec3::from(-aabb.half_extents) + velocity)
            .floor()
            .as_ivec3(),
        (Vec3::from(aabb.half_extents) + velocity).ceil().as_ivec3(),
    );
    for x in area_to_check.0.x..=area_to_check.1.x {
        for y in area_to_check.0.y..=area_to_check.1.y {
            for z in area_to_check.0.z..=area_to_check.1.z {
                let (check_chunk_pos, check_block_cpos) = world_to_voxel(
                    Vec3::from(aabb.center) + Vec3::new(x as f32, y as f32, z as f32),
                );
                if let Some(chunk_entity) = current_chunks.get_entity(ChunkPos(check_chunk_pos)) {
                    if let Ok(chunk) = chunks.get(chunk_entity) {
                        let block_data: BlockData = chunk.get(
                            check_block_cpos.x as usize,
                            check_block_cpos.y as usize,
                            check_block_cpos.z as usize,
                        );
                        let voxel_pos = voxel_to_global_voxel(check_block_cpos, check_chunk_pos);
                        if !block_data.is_empty(block_table) {
                            let block_aabb = Aabb {
                                center: (Vec3A::from(voxel_pos.as_vec3())
                                    + Vec3A::new(0.5, 0.5, 0.5)),
                                half_extents: Vec3A::new(0.5, 0.5, 0.5),
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
                            let enter_time =
                                max(max(FloatOrd(enter.x), FloatOrd(enter.y)), FloatOrd(enter.z));

                            let exit_time =
                                min(min(FloatOrd(exit.x), FloatOrd(exit.y)), FloatOrd(exit.z));
                            if enter_time > exit_time
                                || enter.x < 0.0 && enter.y < 0.0 && enter.z < 0.0
                                || enter.x > 1.0
                                || enter.y > 1.0
                                || enter.z > 1.0
                            {
                                // No collision happens here this frame
                                continue;
                            } else {
                                // This might be a collision this frame
                                let dist: f32;
                                if enter_time.0 == enter.x {
                                    normal.x = if inv_enter.x < 0.0 { 1.0 } else { -1.0 };
                                    dist = inv_enter.x.abs();
                                } else if enter_time.0 == enter.y {
                                    normal.y = if inv_enter.y < 0.0 { 1.0 } else { -1.0 };
                                    dist = inv_enter.y.abs();
                                } else {
                                    normal.z = if inv_enter.z < 0.0 { 1.0 } else { -1.0 };
                                    dist = inv_enter.z.abs();
                                }
                                collisions.push(CollisionInfo {
                                    voxel_pos,
                                    normal,
                                    dist,
                                });
                            }
                        }
                    }
                }
            }
        }
    }
    // Remove all detected collisions that are on a face blocked by another block
    collisions.retain(|col| {
        let blocked_pos = col.voxel_pos.as_vec3() + Vec3::new(0.5, 0.5, 0.5) + col.normal;
        let block_aabb = Aabb {
            center: Vec3A::from(blocked_pos),
            half_extents: Vec3A::new(0.5, 0.5, 0.5),
        };
        // If the "blocked pos" is inside the player, don't count it as blocked
        if aabbs_intersecting(&aabb, &block_aabb) {
            return true;
        }
        let (chunk_pos, voxel_cpos) = world_to_voxel(blocked_pos);
        if let Some(chunk_entity) = current_chunks.get_entity(ChunkPos(chunk_pos)) {
            if let Ok(chunk) = chunks.get(chunk_entity) {
                let block_data: BlockData = chunk.get(
                    voxel_cpos.x as usize,
                    voxel_cpos.y as usize,
                    voxel_cpos.z as usize,
                );
                return block_data.is_empty(block_table);
            }
        }
        return true;
    });
    if !collisions.is_empty() {
        return Some(collisions);
    }
    None
}

#[inline]
fn is_inside_aabb(pos: Vec3A, aabb: &Aabb) -> bool {
    let min = aabb.min();
    let max = aabb.max();
    return pos.x >= min.x
        && pos.x <= max.x
        && pos.y >= min.x
        && pos.y <= max.y
        && pos.z >= min.z
        && pos.z <= max.z;
}

#[inline]
fn aabbs_intersecting(a: &Aabb, b: &Aabb) -> bool {
    return is_inside_aabb(a.min(), b)
        || is_inside_aabb(a.max(), b)
        || is_inside_aabb(b.min(), a)
        || is_inside_aabb(b.max(), a);
}
