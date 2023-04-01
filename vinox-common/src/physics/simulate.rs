use bevy::{
    math::Vec3A,
    prelude::{Component, Entity, EventWriter, IVec3, Query, Res, Transform, Vec3, With, Without},
    render::primitives::Aabb,
    time::Time,
};

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
    chunks_without_data: Query<With<NeedsChunkData>>,
    chunks: Query<&ChunkData>,
    current_chunks: Res<CurrentChunks>,
    block_table: Res<BlockTable>,
    mut collision_event_writer: EventWriter<VoxelCollisionEvent>,
) {
    for (entity, mut aabb, mut velocity, mut transform) in moving_entities.iter_mut() {
        if let Some(chunk_entity) = current_chunks.get_entity(ChunkPos::from(aabb.center)) {
            if chunks_without_data.get(chunk_entity).is_ok() {
                return;
            }
        } else {
            return;
        }
        let movement = velocity.0 * time.delta().as_secs_f32();
        let (max_move_x, x_collided) = test_move_axis(
            &aabb,
            &Vec3 {
                x: movement.x,
                y: 0.0,
                z: 0.0,
            },
            &chunks,
            &current_chunks,
            &block_table,
        );
        aabb.center.x += max_move_x.copysign(movement.x);
        let (max_move_y, y_collided) = test_move_axis(
            &aabb,
            &Vec3 {
                x: 0.0,
                y: movement.y,
                z: 0.0,
            },
            &chunks,
            &current_chunks,
            &block_table,
        );
        aabb.center.y += max_move_y.copysign(movement.y);
        let (max_move_z, z_collided) = test_move_axis(
            &aabb,
            &Vec3 {
                x: 0.0,
                y: 0.0,
                z: movement.z,
            },
            &chunks,
            &current_chunks,
            &block_table,
        );
        aabb.center.z += max_move_z.copysign(movement.z);
        transform.translation = Vec3::from(aabb.center - Vec3A::Y * aabb.half_extents);
        if x_collided {
            velocity.0.x = 0.0
        }
        if y_collided {
            velocity.0.y = 0.0
        }
        if z_collided {
            velocity.0.z = 0.0
        }
    }
}
