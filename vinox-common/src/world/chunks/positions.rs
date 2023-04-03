use std::fmt;

use bevy::{math::Vec3A, prelude::*};
use big_space::GridCell;

use super::storage::CHUNK_SIZE;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Deref, DerefMut, Default)]
pub struct ChunkPos(pub IVec3);

impl fmt::Display for ChunkPos {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<IVec3> for ChunkPos {
    fn from(item: IVec3) -> Self {
        Self(item)
    }
}

impl From<Vec3> for ChunkPos {
    fn from(item: Vec3) -> Self {
        let pos = item.floor().as_ivec3();
        Self(IVec3::new(
            (pos.x as f32 / (CHUNK_SIZE as f32)).floor() as i32,
            (pos.y as f32 / (CHUNK_SIZE as f32)).floor() as i32,
            (pos.z as f32 / (CHUNK_SIZE as f32)).floor() as i32,
        ))
    }
}

impl From<Vec3A> for ChunkPos {
    fn from(item: Vec3A) -> Self {
        let pos = item.floor().as_ivec3();
        Self(IVec3::new(
            (pos.x as f32 / (CHUNK_SIZE as f32)).floor() as i32,
            (pos.y as f32 / (CHUNK_SIZE as f32)).floor() as i32,
            (pos.z as f32 / (CHUNK_SIZE as f32)).floor() as i32,
        ))
    }
}

impl From<VoxelPos> for ChunkPos {
    fn from(item: VoxelPos) -> Self {
        Self(IVec3::new(
            (item.x as f32 / (CHUNK_SIZE as f32)).floor() as i32,
            (item.y as f32 / (CHUNK_SIZE as f32)).floor() as i32,
            (item.z as f32 / (CHUNK_SIZE as f32)).floor() as i32,
        ))
    }
}

impl From<ChunkPos> for Vec3 {
    fn from(item: ChunkPos) -> Self {
        Vec3::new(
            (item.x * (CHUNK_SIZE) as i32) as f32,
            (item.y * (CHUNK_SIZE) as i32) as f32,
            (item.z * (CHUNK_SIZE) as i32) as f32,
        )
    }
}
impl From<ChunkPos> for Vec3A {
    fn from(item: ChunkPos) -> Self {
        Vec3A::new(
            (item.x * (CHUNK_SIZE) as i32) as f32,
            (item.y * (CHUNK_SIZE) as i32) as f32,
            (item.z * (CHUNK_SIZE) as i32) as f32,
        )
    }
}
impl From<ChunkPos> for IVec3 {
    fn from(item: ChunkPos) -> Self {
        *item
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

    pub fn from_chunk_cell(chunk_cell: GridCell<i32>, pos: Vec3) -> ChunkPos {
        ChunkPos::from(VoxelPos::from_chunk_cell(chunk_cell, pos))
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Deref, DerefMut, Default)]
pub struct VoxelPos(pub IVec3);

impl From<Vec3> for VoxelPos {
    fn from(item: Vec3) -> Self {
        Self(item.floor().as_ivec3())
    }
}

impl From<Vec3A> for VoxelPos {
    fn from(item: Vec3A) -> Self {
        Self(item.floor().as_ivec3())
    }
}

impl From<IVec3> for VoxelPos {
    fn from(item: IVec3) -> Self {
        Self(item)
    }
}

impl From<VoxelPos> for Vec3 {
    fn from(item: VoxelPos) -> Self {
        Vec3::new(item.x as f32, item.y as f32, item.z as f32)
    }
}

impl From<VoxelPos> for Vec3A {
    fn from(item: VoxelPos) -> Self {
        Vec3A::new(item.x as f32, item.y as f32, item.z as f32)
    }
}

impl From<VoxelPos> for IVec3 {
    fn from(item: VoxelPos) -> Self {
        *item
    }
}

impl From<(RelativeVoxelPos, ChunkPos)> for VoxelPos {
    fn from(item: (RelativeVoxelPos, ChunkPos)) -> Self {
        let world_chunk = *item.1 * IVec3::splat(CHUNK_SIZE as i32);
        VoxelPos::from(Vec3::new(
            (world_chunk.x as f32) + item.0.x as f32,
            (world_chunk.y as f32) + item.0.y as f32,
            (world_chunk.z as f32) + item.0.z as f32,
        ))
    }
}

// impl From<VoxelPos> for (RelativeVoxelPos, ChunkPos) {
//     fn from(item: VoxelPos) -> Self {
//         (RelativeVoxelPos::from(*item), Into::<ChunkPos>::into(*item))
//     }
// }

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
    pub fn to_offsets(&self) -> (RelativeVoxelPos, ChunkPos) {
        (RelativeVoxelPos::from(*self), Into::<ChunkPos>::into(*self))
    }

    pub fn from_chunk_cell(grid_cell: GridCell<i32>, pos: Vec3) -> VoxelPos {
        VoxelPos(IVec3::new(
            (grid_cell.x * 10000) + pos.x.floor() as i32,
            (grid_cell.y * 10000) + pos.y.floor() as i32,
            (grid_cell.z * 10000) + pos.z.floor() as i32,
        ))
    }

    pub fn relative_to_cell(&self, grid_cell: GridCell<i32>) -> Vec3 {
        let grid_cell_center: IVec3 = IVec3 {
            x: grid_cell.x * 10000,
            y: grid_cell.y * 10000,
            z: grid_cell.z * 10000,
        };
        (self.0 - grid_cell_center).as_vec3()
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Deref, DerefMut, Default)]
pub struct RelativeVoxelPos(pub UVec3);

impl fmt::Display for RelativeVoxelPos {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<VoxelPos> for RelativeVoxelPos {
    fn from(item: VoxelPos) -> Self {
        Self(UVec3::new(
            item.x.rem_euclid(CHUNK_SIZE as i32) as u32,
            item.y.rem_euclid(CHUNK_SIZE as i32) as u32,
            item.z.rem_euclid(CHUNK_SIZE as i32) as u32,
        ))
    }
}

impl From<Vec3> for RelativeVoxelPos {
    fn from(item: Vec3) -> Self {
        From::<VoxelPos>::from(item.into())
    }
}
impl From<IVec3> for RelativeVoxelPos {
    fn from(item: IVec3) -> Self {
        From::<VoxelPos>::from(item.into())
    }
}
impl From<Vec3A> for RelativeVoxelPos {
    fn from(item: Vec3A) -> Self {
        From::<VoxelPos>::from(item.into())
    }
}
impl From<UVec3> for RelativeVoxelPos {
    fn from(item: UVec3) -> Self {
        Self(item)
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

    pub fn from_chunk_cell(chunk_cell: GridCell<i32>, pos: Vec3) -> RelativeVoxelPos {
        RelativeVoxelPos::from(VoxelPos::from_chunk_cell(chunk_cell, pos))
    }
}
