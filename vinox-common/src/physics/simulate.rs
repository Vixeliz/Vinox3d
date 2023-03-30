use bevy::{
    math::Vec3A,
    prelude::{Component, Entity, EventWriter, IVec3, Query, Res, Transform, Vec3, With, Without},
    render::primitives::Aabb,
    time::Time,
};

use crate::{
    physics::collision::aabb::{get_collision_info, CollisionInfo},
    world::chunks::{
        ecs::CurrentChunks,
        positions::{ChunkPos, VoxelPos},
        storage::{BlockTable, ChunkData},
    },
};

use super::collision::aabb::{aabb_vs_world, aabbs_intersect};

#[derive(Component)]
pub struct CollidesWithWorld;

#[derive(Component)]
pub struct Velocity(pub Vec3);

#[derive(Debug)]
pub struct VoxelCollisionEvent {
    pub entity: Entity,
    pub voxel_pos: IVec3,
    pub normal: Vec3,
}

pub fn move_no_collide(
    mut moving_entities: Query<(Entity, &mut Aabb, &Velocity), Without<CollidesWithWorld>>,
    time: Res<Time>,
) {
    for (_entity, mut aabb, velocity) in moving_entities.iter_mut() {
        aabb.center += Vec3A::from(velocity.0 * time.delta().as_secs_f32());
    }
}

pub fn move_and_collide(
    mut moving_entities: Query<
        (Entity, &mut Aabb, &mut Velocity, &mut Transform),
        With<CollidesWithWorld>,
    >,
    time: Res<Time>,
    chunks: Query<&ChunkData>,
    current_chunks: Res<CurrentChunks>,
    block_table: Res<BlockTable>,
    mut collision_event_writer: EventWriter<VoxelCollisionEvent>,
) {
    for (entity, mut aabb, mut velocity, mut transform) in moving_entities.iter_mut() {
        if current_chunks
            .get_entity(ChunkPos::from_world(VoxelPos::from_world(Vec3::from(
                aabb.center,
            ))))
            .is_none()
        {
            return;
        }
        let movement = velocity.0 * time.delta().as_secs_f32();
        let mut v_after = movement;
        let mut max_move = v_after.abs();
        if let Some(mut aabb_collisions) =
            aabb_vs_world(&aabb, &chunks, movement, &current_chunks, &block_table)
        {
            // First pass to evaluate all collisions
            for col in aabb_collisions.iter() {
                if col.normal.x != 0.0 {
                    max_move.x = f32::min(max_move.x, col.dist);
                    v_after.x = 0.0;
                } else if col.normal.y != 0.0 {
                    max_move.y = f32::min(max_move.y, col.dist);
                    v_after.y = 0.0;
                } else if col.normal.z != 0.0 {
                    max_move.z = f32::min(max_move.z, col.dist);
                    v_after.z = 0.0;
                }
            }
            // Remove collisions that are blocked by other collisions
            aabb_collisions.retain(|col| {
                let v_filt;
                if col.normal.y != 0.0 {
                    v_filt = Vec3::new(
                        v_after.x,
                        movement.y - max_move.y.copysign(movement.y),
                        v_after.z,
                    );
                } else if col.normal.x != 0.0 {
                    v_filt = Vec3::new(
                        movement.x - max_move.x.copysign(movement.x),
                        v_after.y,
                        v_after.z,
                    );
                } else {
                    v_filt = Vec3::new(
                        v_after.x,
                        v_after.y,
                        movement.z - max_move.z.copysign(movement.z),
                    );
                }
                let hypth_aabb = Aabb {
                    center: aabb.center + Vec3A::from(max_move.copysign(movement) + v_filt),
                    half_extents: aabb.half_extents,
                };
                aabbs_intersect(&hypth_aabb, &col.collision_aabb)
            });
            // Re-calculate normals
            let fm = max_move.copysign(movement);
            let aabb_collisions: Vec<CollisionInfo> = aabb_collisions
                .iter()
                .filter_map(|col| {
                    let v_filt;
                    if col.normal.y != 0.0 {
                        v_filt = Vec3::new(v_after.x, movement.y, v_after.z);
                    } else if col.normal.x != 0.0 {
                        v_filt = Vec3::new(movement.x, v_after.y, v_after.z);
                    } else {
                        v_filt = Vec3::new(v_after.x, v_after.y, movement.z);
                    }
                    let hypth_aabb = Aabb {
                        center: aabb.center + Vec3A::from(fm),
                        half_extents: aabb.half_extents,
                    };
                    let colinfo = get_collision_info(&hypth_aabb, &col.collision_aabb, &v_filt);
                    colinfo
                })
                .collect();
            // Re-evaluate the new set of collisions
            let mut v_after = movement;
            let mut max_move = movement.abs();
            for col in aabb_collisions.iter() {
                if col.normal.x != 0.0 {
                    max_move.x = f32::min(max_move.x, col.dist);
                    v_after.x = 0.0;
                } else if col.normal.y != 0.0 {
                    max_move.y = f32::min(max_move.y, col.dist);
                    v_after.y = 0.0;
                } else {
                    max_move.z = f32::min(max_move.z, col.dist);
                    v_after.z = 0.0;
                }
                collision_event_writer.send(VoxelCollisionEvent {
                    entity,
                    voxel_pos: col.collision_aabb.center.floor().as_ivec3(),
                    normal: col.normal,
                });
            }
            // Apply updated velocity
            velocity.0 = v_after / time.delta().as_secs_f32();
            let final_move = max_move.copysign(movement);
            aabb.center += Vec3A::from(final_move);
            transform.translation = Vec3::from(aabb.center - Vec3A::Y * aabb.half_extents);
            // // Auto-step
            // for col in aabb_collisions {
            //     if col.normal.y == 0.0 {
            //         let step_height: f32 = 1.1;
            //         let block_step_height = col.collision_aabb.max().y - aabb.min().y;
            //         let hypth_aabb = Aabb {
            //             center: aabb.center
            //                 + Vec3A::from(-col.normal * 0.05)
            //                 + Vec3A::Y * (block_step_height + 0.005),
            //             half_extents: aabb.half_extents,
            //         };
            //         if block_step_height > 0.0 && block_step_height < step_height {
            //             if !aabb_intersects_world(
            //                 &hypth_aabb,
            //                 &chunks,
            //                 &current_chunks,
            //                 &block_table,
            //             ) {
            //                 aabb.center = hypth_aabb.center;
            //                 transform.translation =
            //                     Vec3::from(aabb.center - Vec3A::Y * aabb.half_extents);
            //                 return;
            //             } else {
            //                 let intersectors = get_aabb_world_intersectors(
            //                     &hypth_aabb,
            //                     &chunks,
            //                     &current_chunks,
            //                     &block_table,
            //                 );
            //                 println!(
            //                     "Not stepping for collision {col} because it intersects with {intersectors:?}"
            //                 );
            //             }
            //         }
            //     }
            // }
        } else {
            // Apply updated velocity
            velocity.0 = v_after / time.delta().as_secs_f32();
            let final_move = max_move.copysign(movement);
            aabb.center += Vec3A::from(final_move);
            transform.translation = Vec3::from(aabb.center - Vec3A::Y * aabb.half_extents);
        }
    }
}
