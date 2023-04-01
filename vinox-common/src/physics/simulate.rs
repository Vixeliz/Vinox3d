use bevy::{
    math::Vec3A,
    prelude::{Component, Entity, EventWriter, IVec3, Query, Res, Transform, Vec3, With, Without},
    render::primitives::Aabb,
    time::Time,
};
use big_space::GridCell;

use crate::world::chunks::{
    ecs::{ChunkCell, CurrentChunks, NeedsChunkData},
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
    mut moving_entities: Query<(Entity, &mut Aabb, &Velocity), Without<CollidesWithWorld>>,
    time: Res<Time>,
) {
    for (_entity, mut aabb, velocity) in moving_entities.iter_mut() {
        aabb.center += Vec3A::from(velocity.0 * time.delta().as_secs_f32());
    }
}

pub fn move_and_collide(
    mut moving_entities: Query<
        (
            Entity,
            &mut Aabb,
            &mut Velocity,
            &mut Transform,
            &GridCell<i32>,
        ),
        With<CollidesWithWorld>,
    >,
    time: Res<Time>,
    chunks_without_data: Query<With<NeedsChunkData>>,
    chunks: Query<&ChunkData>,
    current_chunks: Res<CurrentChunks>,
    block_table: Res<BlockTable>,
    mut collision_event_writer: EventWriter<VoxelCollisionEvent>,
) {
    for (entity, mut aabb, mut velocity, mut transform, grid_cell) in moving_entities.iter_mut() {
        let chunk_pos: ChunkPos = ChunkPos::from_chunk_cell(grid_cell.clone(), aabb.center.into());
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
                velocity.0.x = 0.0;
                collision_event_writer.send(VoxelCollisionEvent {
                    entity,
                    voxel_pos: c.collision_aabb.center.floor().as_ivec3(),
                    normal: c.normal,
                });
            } else {
                aabb.center.x += movement.x;
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
                velocity.0.y = 0.0;
                collision_event_writer.send(VoxelCollisionEvent {
                    entity,
                    voxel_pos: c.collision_aabb.center.floor().as_ivec3(),
                    normal: c.normal,
                });
            } else {
                aabb.center.y += movement.y;
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
                velocity.0.z = 0.0;
                collision_event_writer.send(VoxelCollisionEvent {
                    entity,
                    voxel_pos: c.collision_aabb.center.floor().as_ivec3(),
                    normal: c.normal,
                });
            } else {
                aabb.center.z += movement.z;
            }
            transform.translation = Vec3::from(aabb.center - Vec3A::Y * aabb.half_extents);
        }
    }
}
