use bevy::{
    math::Vec3A,
    prelude::{Component, Entity, EventWriter, IVec3, Query, Res, Transform, Vec3, With, Without},
    render::primitives::Aabb,
    time::Time,
};
use big_space::GridCell;

use crate::world::chunks::{
    ecs::{CurrentChunks, NeedsChunkData},
    positions::ChunkPos,
    storage::{BlockTable, ChunkData},
};

use super::collision::aabb::test_move_axis;

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
    mut moving_entities: Query<(Entity, &mut Transform, &Velocity), Without<CollidesWithWorld>>,
    time: Res<Time>,
) {
    for (_entity, mut transform, velocity) in moving_entities.iter_mut() {
        transform.translation += velocity.0 * time.delta().as_secs_f32();
    }
}

pub fn move_and_collide(
    mut moving_entities: Query<
        (
            Entity,
            &mut Aabb,
            &mut Velocity,
            &mut Transform,
            &mut GridCell<i32>,
        ),
        With<CollidesWithWorld>,
    >,
    time: Res<Time>,
    chunks_without_data: Query<With<NeedsChunkData>>,
    chunks: Query<&ChunkData>,
    current_chunks: Res<CurrentChunks>,
    block_table: Res<BlockTable>,
    mut _collision_event_writer: EventWriter<VoxelCollisionEvent>,
) {
    for (_entity, aabb, mut velocity, mut transform, grid_cell) in moving_entities.iter_mut() {
        let mut aabb = *aabb;
        aabb.center = Vec3A::new(
            transform.translation.x,
            transform.translation.y + aabb.half_extents.y,
            transform.translation.z,
        );
        let chunk_pos: ChunkPos =
            ChunkPos::from_chunk_cell(*grid_cell, transform.translation);
        if let Some(chunk_entity) = current_chunks.get_entity(chunk_pos) {
            if chunks_without_data.get(chunk_entity).is_ok() {
                continue;
            }
            let movement = velocity.0 * time.delta().as_secs_f32();
            let x_col = test_move_axis(
                &aabb,
                *grid_cell,
                &Vec3 {
                    x: movement.x,
                    y: 0.0,
                    z: 0.0,
                },
                &chunks,
                &current_chunks,
                &block_table,
            );
            if let Some(c) = x_col {
                aabb.center.x += c.dist.copysign(movement.x);
                transform.translation.x += c.dist.copysign(movement.x);
                velocity.0.x = 0.0;
                // collision_event_writer.send(VoxelCollisionEvent {
                //     entity,
                //     voxel_pos: c.collision_aabb.center.floor().as_ivec3(),
                //     normal: c.normal,
                // });
            } else {
                aabb.center.x += movement.x;
                transform.translation.x += movement.x;
            }
            let y_col = test_move_axis(
                &aabb,
                *grid_cell,
                &Vec3 {
                    x: 0.0,
                    y: movement.y,
                    z: 0.0,
                },
                &chunks,
                &current_chunks,
                &block_table,
            );
            if let Some(c) = y_col {
                aabb.center.y += c.dist.copysign(movement.y);
                transform.translation.y += c.dist.copysign(movement.y);
                velocity.0.y = 0.0;
                // collision_event_writer.send(VoxelCollisionEvent {
                //     entity,
                //     voxel_pos: c.collision_aabb.center.floor().as_ivec3(),
                //     normal: c.normal,
                // });
            } else {
                aabb.center.y += movement.y;
                transform.translation.y += movement.y;
            }
            let z_col = test_move_axis(
                &aabb,
                *grid_cell,
                &Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: movement.z,
                },
                &chunks,
                &current_chunks,
                &block_table,
            );
            if let Some(c) = z_col {
                aabb.center.z += c.dist.copysign(movement.z);
                transform.translation.z += c.dist.copysign(movement.z);
                velocity.0.z = 0.0;
                // collision_event_writer.send(VoxelCollisionEvent {
                //     entity,
                //     voxel_pos: c.collision_aabb.center.floor().as_ivec3(),
                //     normal: c.normal,
                // });
            } else {
                aabb.center.z += movement.z;
                transform.translation.z += movement.z;
            }
        }
    }
}
