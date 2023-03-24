use bevy::{
    math::Vec3A,
    prelude::{Component, Entity, EventWriter, IVec3, Query, Res, Vec3, With, Without},
    render::primitives::Aabb,
    time::Time,
};

use crate::world::chunks::{
    ecs::CurrentChunks,
    positions::{world_to_chunk, ChunkPos},
    storage::{BlockTable, ChunkData},
};

use super::collision::aabb::{aabb_vs_world, aabbs_intersecting};

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
        // Do not simulate outside of loaded chunks
        if current_chunks
            .get_entity(ChunkPos(world_to_chunk(Vec3::from(aabb.center))))
            .is_none()
        {
            return;
        }
        let movement = velocity.0 * time.delta().as_secs_f32();
        let mut v_after = movement;
        let mut max_move = v_after.abs();
        let margin: f32 = 0.01;
        if let Some(mut aabb_collisions) = aabb_vs_world(
            &aabb,
            &chunks,
            movement,
            &current_chunks,
            &block_table,
            margin,
        ) {
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
                let intersects = aabbs_intersecting(&hypth_aabb, &col.collider_aabb);
                return intersects;
            });
            v_after = movement;
            max_move = movement.abs();
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
                    voxel_pos: col.voxel_pos,
                    normal: col.normal,
                });
            }
        }
        velocity.0 = v_after / time.delta().as_secs_f32();
        let final_move = Vec3::new(
            max_move.x * movement.x.signum(),
            max_move.y * movement.y.signum(),
            max_move.z * movement.z.signum(),
        );
        aabb.center += Vec3A::from(final_move);
    }
}
