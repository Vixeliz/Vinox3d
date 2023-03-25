use bevy::{
    math::Vec3A,
    prelude::{Component, Entity, EventWriter, IVec3, Query, Res, Vec3, With, Without},
    render::primitives::Aabb,
    time::Time,
};

use crate::{
    physics::collision::aabb::{aabbs_intersect_or_touch, get_collision_info, CollisionInfo},
    world::chunks::{
        ecs::CurrentChunks,
        positions::{world_to_chunk, ChunkPos},
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
    mut moving_entities: Query<(Entity, &mut Aabb, &mut Velocity), With<CollidesWithWorld>>,
    time: Res<Time>,
    chunks: Query<&ChunkData>,
    current_chunks: Res<CurrentChunks>,
    block_table: Res<BlockTable>,
    mut collision_event_writer: EventWriter<VoxelCollisionEvent>,
) {
    for (entity, mut aabb, mut velocity) in moving_entities.iter_mut() {
        if current_chunks
            .get_entity(ChunkPos(world_to_chunk(Vec3::from(aabb.center))))
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
            println!("All collisions first detected:");
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
                println!(
                    "\tCollision @ {} norm {} dist {}",
                    col.collision_aabb.center, col.normal, col.dist
                );
            }
            // Remove collisions that are blocked by other collisions
            aabb_collisions.retain(|col| {
                let v_filt;
                if col.normal.y != 0.0 {
                    v_filt = Vec3::new(v_after.x, movement.y, v_after.z);
                } else if col.normal.x != 0.0 {
                    v_filt = Vec3::new(movement.x, v_after.y, v_after.z);
                } else {
                    v_filt = Vec3::new(v_after.x, v_after.y, movement.z);
                }
                let hypth_aabb = Aabb {
                    center: aabb.center
                        + Vec3A::from(
                            Vec3::new(
                                max_move.x * movement.x.signum(),
                                max_move.y * movement.y.signum(),
                                max_move.z * movement.z.signum(),
                            ) + v_filt,
                        ),
                    half_extents: aabb.half_extents,
                };
                let intersects = aabbs_intersect(&hypth_aabb, &col.collision_aabb);
                return intersects;
            });
            // Re-calculate normals
            let fm = Vec3::new(
                max_move.x * movement.x.signum(),
                max_move.y * movement.y.signum(),
                max_move.z * movement.z.signum(),
            );
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
                    let new_col = get_collision_info(&hypth_aabb, &col.collision_aabb, &v_filt);
                    if let Some(n) = new_col.clone() {
                        println!("New collision: {} v_filt {v_filt} fm {fm}", n);
                    } else {
                        println!("Collision {} was removed!", col);
                    }
                    return new_col;
                })
                .collect();
            println!("Final list of collisions:");
            aabb_collisions.iter().for_each(|c| println!("\t{c}"));
            // Re-evaluate the new set of collisions
            let mut v_after = movement;
            let mut max_move = movement.abs();
            for col in aabb_collisions {
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
                collision_event_writer.send(VoxelCollisionEvent {
                    entity,
                    voxel_pos: col.collision_aabb.center.floor().as_ivec3(),
                    normal: col.normal,
                });
            }
        }
        // Apply updated velocity
        println!("Final mxmv: {max_move}");
        velocity.0 = v_after / time.delta().as_secs_f32();
        let final_move = Vec3::new(
            max_move.x * movement.x.signum(),
            max_move.y * movement.y.signum(),
            max_move.z * movement.z.signum(),
        );
        aabb.center += Vec3A::from(final_move);
    }
}
