use std::fmt;

use bevy::prelude::*;

use super::storage::CHUNK_SIZE;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Deref, DerefMut, Default)]
pub struct ChunkPos(pub IVec3);

impl fmt::Display for ChunkPos {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl ChunkPos {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        ChunkPos(IVec3::new(x, y, z))
    }
    pub fn neighbors(&self) -> Vec<ChunkPos> {
        vec![
            ChunkPos::new(
                self.x.wrapping_sub(1),
                self.y.wrapping_sub(1),
                self.z.wrapping_sub(1),
            ), //0
            ChunkPos::new(self.x.wrapping_sub(1), self.y.wrapping_sub(1), self.z), // 1
            ChunkPos::new(self.x.wrapping_sub(1), self.y.wrapping_sub(1), self.z + 1), //2
            ChunkPos::new(self.x.wrapping_sub(1), self.y, self.z.wrapping_sub(1)), // 3
            ChunkPos::new(self.x.wrapping_sub(1), self.y, self.z),                 // 4
            ChunkPos::new(self.x.wrapping_sub(1), self.y, self.z + 1),             // 5
            ChunkPos::new(self.x.wrapping_sub(1), self.y + 1, self.z.wrapping_sub(1)), // 6
            ChunkPos::new(self.x.wrapping_sub(1), self.y + 1, self.z),             // 7
            ChunkPos::new(self.x.wrapping_sub(1), self.y + 1, self.z + 1),         // 8
            ChunkPos::new(self.x, self.y.wrapping_sub(1), self.z.wrapping_sub(1)), // 9
            ChunkPos::new(self.x, self.y.wrapping_sub(1), self.z),                 // 10
            ChunkPos::new(self.x, self.y.wrapping_sub(1), self.z + 1),             // 11
            ChunkPos::new(self.x, self.y, self.z.wrapping_sub(1)),                 // 12
            ChunkPos::new(self.x, self.y, self.z + 1),                             // 13
            ChunkPos::new(self.x, self.y + 1, self.z.wrapping_sub(1)),             // 14
            ChunkPos::new(self.x, self.y + 1, self.z),                             // 15
            ChunkPos::new(self.x, self.y + 1, self.z + 1),                         // 16
            ChunkPos::new(self.x + 1, self.y.wrapping_sub(1), self.z.wrapping_sub(1)), // 17
            ChunkPos::new(self.x + 1, self.y.wrapping_sub(1), self.z),             // 18
            ChunkPos::new(self.x + 1, self.y.wrapping_sub(1), self.z + 1),         // 19
            ChunkPos::new(self.x + 1, self.y, self.z.wrapping_sub(1)),             // 20
            ChunkPos::new(self.x + 1, self.y, self.z),                             // 21
            ChunkPos::new(self.x + 1, self.y, self.z + 1),                         // 22
            ChunkPos::new(self.x + 1, self.y + 1, self.z.wrapping_sub(1)),         // 23
            ChunkPos::new(self.x + 1, self.y + 1, self.z),                         // 24
            ChunkPos::new(self.x + 1, self.y + 1, self.z + 1),                     // 25
        ]
    }

    pub fn distance(&self, other: &ChunkPos) -> f32 {
        Vec3::new(self.x as f32, self.y as f32, self.z as f32).distance(Vec3::new(
            other.x as f32,
            other.y as f32,
            other.z as f32,
        ))
    }

    pub fn from_world(pos: VoxelPos) -> Self {
        ChunkPos(IVec3::new(
            (pos.x as f32 / (CHUNK_SIZE as f32)).floor() as i32,
            (pos.y as f32 / (CHUNK_SIZE as f32)).floor() as i32,
            (pos.z as f32 / (CHUNK_SIZE as f32)).floor() as i32,
        ))
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Deref, DerefMut, Default)]
pub struct VoxelPos(pub IVec3);

impl fmt::Display for VoxelPos {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl VoxelPos {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        VoxelPos(IVec3::new(x, y, z))
    }
    pub fn distance(&self, other: &VoxelPos) -> f32 {
        Vec3::new(self.x as f32, self.y as f32, self.z as f32).distance(Vec3::new(
            other.x as f32,
            other.y as f32,
            other.z as f32,
        ))
    }
    pub fn from_offsets(voxel_pos: RelativeVoxelPos, chunk_pos: ChunkPos) -> Self {
        let world_chunk = *chunk_pos * IVec3::splat(CHUNK_SIZE as i32);
        VoxelPos(
            Vec3::new(
                (world_chunk.x as f32) + voxel_pos.x as f32,
                (world_chunk.y as f32) + voxel_pos.y as f32,
                (world_chunk.z as f32) + voxel_pos.z as f32,
            )
            .as_ivec3(),
        )
    }
    pub fn from_world(voxel_pos: Vec3) -> Self {
        VoxelPos(voxel_pos.floor().as_ivec3())
    }

    pub fn as_vec3(&self) -> Vec3 {
        Vec3::new(self.x as f32, self.y as f32, self.z as f32)
    }

    pub fn to_offsets(&self) -> (RelativeVoxelPos, ChunkPos) {
        (
            RelativeVoxelPos::from_voxel(*self),
            ChunkPos::from_world(*self),
        )
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Deref, DerefMut, Default)]
pub struct RelativeVoxelPos(pub UVec3);

impl fmt::Display for RelativeVoxelPos {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl RelativeVoxelPos {
    pub fn new(x: u32, y: u32, z: u32) -> Self {
        RelativeVoxelPos(UVec3::new(x, y, z))
    }
    pub fn distance(&self, other: &RelativeVoxelPos) -> f32 {
        Vec3::new(self.x as f32, self.y as f32, self.z as f32).distance(Vec3::new(
            other.x as f32,
            other.y as f32,
            other.z as f32,
        ))
    }
    pub fn from_voxel(voxel_pos: VoxelPos) -> Self {
        RelativeVoxelPos(UVec3::new(
            voxel_pos.x.rem_euclid(CHUNK_SIZE as i32) as u32,
            voxel_pos.y.rem_euclid(CHUNK_SIZE as i32) as u32,
            voxel_pos.z.rem_euclid(CHUNK_SIZE as i32) as u32,
        ))
    }
}
