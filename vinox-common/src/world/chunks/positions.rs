use bevy::prelude::*;

use super::storage::CHUNK_SIZE;

pub fn world_to_chunk(pos: Vec3) -> IVec3 {
    IVec3::new(
        (pos.x / (CHUNK_SIZE as f32)).floor() as i32,
        (pos.y / (CHUNK_SIZE as f32)).floor() as i32,
        (pos.z / (CHUNK_SIZE as f32)).floor() as i32,
    )
}

pub fn world_to_voxel(voxel_pos: Vec3) -> (IVec3, UVec3) {
    (
        world_to_chunk(voxel_pos),
        UVec3::new(
            voxel_pos.floor().x.rem_euclid(CHUNK_SIZE as f32) as u32,
            voxel_pos.floor().y.rem_euclid(CHUNK_SIZE as f32) as u32,
            voxel_pos.floor().z.rem_euclid(CHUNK_SIZE as f32) as u32,
        ),
    )
}

pub fn voxel_to_world(voxel_pos: UVec3, chunk_pos: IVec3) -> Vec3 {
    let world_chunk = chunk_pos * IVec3::splat(CHUNK_SIZE as i32);
    Vec3::new(
        (world_chunk.x as f32) + voxel_pos.x as f32,
        (world_chunk.y as f32) + voxel_pos.y as f32,
        (world_chunk.z as f32) + voxel_pos.z as f32,
    )
}
