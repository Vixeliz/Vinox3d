use bevy::prelude::*;

use super::{
    ecs::ViewRadius,
    storage::{ChunkData, CHUNK_SIZE},
};

pub fn circle_points(view_radius: &ViewRadius) -> Vec<IVec2> {
    let center = IVec2::new(0, 0);
    let radius_squared = view_radius.horizontal * view_radius.horizontal;
    let mut points = Vec::new();

    for x in -view_radius.horizontal..=view_radius.horizontal {
        for z in -view_radius.horizontal..=view_radius.horizontal {
            let point = IVec2::new(x, z);
            let distance_squared = point.as_vec2().distance_squared(center.as_vec2()) as i32;
            if distance_squared <= radius_squared {
                points.push(point);
            }
        }
    }

    points
}

// pub fn world_to_chunk(pos: Vec3) -> IVec3 {
//     IVec3::new(
//         (pos.x / (CHUNK_SIZE as f32)).floor() as i32,
//         (pos.y / (CHUNK_SIZE as f32)).floor() as i32,
//         (pos.z / (CHUNK_SIZE as f32)).floor() as i32,
//     )
// }

// pub fn world_to_global_voxel(voxel_pos: Vec3) -> IVec3 {
//     voxel_pos.floor().as_ivec3()
// }

pub fn world_to_offsets(voxel_pos: Vec3) -> UVec3 {
    UVec3::new(
        voxel_pos.floor().x.rem_euclid(CHUNK_SIZE as f32) as u32,
        voxel_pos.floor().y.rem_euclid(CHUNK_SIZE as f32) as u32,
        voxel_pos.floor().z.rem_euclid(CHUNK_SIZE as f32) as u32,
    )
}

pub fn world_to_voxel(voxel_pos: Vec3) -> (IVec3, UVec3) {
    (
        ChunkPos::from_global_coords(voxel_pos.x, voxel_pos.y, voxel_pos.z).as_ivec3(),
        world_to_offsets(voxel_pos),
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

pub fn relative_voxel_to_world(voxel_pos: IVec3, chunk_pos: IVec3) -> Vec3 {
    let world_chunk = chunk_pos * IVec3::splat(CHUNK_SIZE as i32);
    Vec3::new(
        (world_chunk.x as f32) + voxel_pos.x as f32,
        (world_chunk.y as f32) + voxel_pos.y as f32,
        (world_chunk.z as f32) + voxel_pos.z as f32,
    )
}

// #[cfg(test)]
// mod test {
//     use super::*;

//     #[test]
//     fn test_world_to_chunk() {
//         assert_eq!(world_to_chunk(Vec3::splat(0.0)), IVec3::splat(0));
//         assert_eq!(world_to_chunk(Vec3::splat(1.0)), IVec3::splat(0));
//         assert_eq!(world_to_chunk(Vec3::splat(15.0)), IVec3::splat(0));

//         assert_eq!(world_to_chunk(Vec3::splat(16.0)), IVec3::splat(1));
//         assert_eq!(world_to_chunk(Vec3::splat(17.0)), IVec3::splat(1));
//         assert_eq!(world_to_chunk(Vec3::splat(31.0)), IVec3::splat(1));

//         assert_eq!(world_to_chunk(Vec3::splat(-1.0)), IVec3::splat(-1));
//         assert_eq!(world_to_chunk(Vec3::splat(-15.0)), IVec3::splat(-1));

//         assert_eq!(
//             world_to_chunk(Vec3::new(2.0, -1.0, 5.0)),
//             IVec3::new(0, -1, 0)
//         );
//         assert_eq!(
//             world_to_chunk(Vec3::new(2.0, -15.0, 5.0)),
//             IVec3::new(0, -1, 0)
//         );

//         // With fractions
//         assert_eq!(
//             world_to_chunk(Vec3::new(2.0, -15.1, 5.0)),
//             IVec3::new(0, -1, 0)
//         );
//         assert_eq!(
//             world_to_chunk(Vec3::new(2.0, -15.9, 5.0)),
//             IVec3::new(0, -1, 0)
//         );
//         assert_eq!(
//             world_to_chunk(Vec3::new(2.0, 15.1, 5.0)),
//             IVec3::new(0, 0, 0)
//         );
//         assert_eq!(
//             world_to_chunk(Vec3::new(2.0, 15.9, 5.0)),
//             IVec3::new(0, 0, 0)
//         );
//     }

//     #[test]
//     fn test_voxel_offsets() {
//         assert_eq!(world_to_offsets(Vec3::splat(0.0)), UVec3::splat(0));
//         assert_eq!(world_to_offsets(Vec3::splat(1.0)), UVec3::splat(1));
//         assert_eq!(world_to_offsets(Vec3::splat(15.0)), UVec3::splat(15));

//         assert_eq!(world_to_offsets(Vec3::splat(16.0)), UVec3::splat(0));
//         assert_eq!(world_to_offsets(Vec3::splat(17.0)), UVec3::splat(1));
//         assert_eq!(world_to_offsets(Vec3::splat(31.0)), UVec3::splat(15));

//         assert_eq!(world_to_offsets(Vec3::splat(-16.0)), UVec3::splat(0));
//         assert_eq!(world_to_offsets(Vec3::splat(-15.0)), UVec3::splat(1));
//         assert_eq!(world_to_offsets(Vec3::splat(-1.0)), UVec3::splat(15));

//         assert_eq!(world_to_offsets(Vec3::splat(-32.0)), UVec3::splat(0));
//         assert_eq!(world_to_offsets(Vec3::splat(-31.0)), UVec3::splat(1));
//         assert_eq!(world_to_offsets(Vec3::splat(-17.0)), UVec3::splat(15));

//         assert_eq!(
//             world_to_offsets(Vec3::new(-2.0, 1.0, 5.0)),
//             UVec3::new(14, 1, 5)
//         );
//         assert_eq!(
//             world_to_offsets(Vec3::new(2.0, -1.0, 5.0)),
//             UVec3::new(2, 15, 5)
//         );
//         assert_eq!(
//             world_to_offsets(Vec3::new(2.0, 1.0, -5.0)),
//             UVec3::new(2, 1, 11)
//         );

//         assert_eq!(
//             world_to_offsets(Vec3::new(-15.0, 1.0, 5.0)),
//             UVec3::new(1, 1, 5)
//         );
//         assert_eq!(
//             world_to_offsets(Vec3::new(15.0, -1.0, 5.0)),
//             UVec3::new(15, 15, 5)
//         );
//         assert_eq!(
//             world_to_offsets(Vec3::new(15.0, 1.0, -5.0)),
//             UVec3::new(15, 1, 11)
//         );

//         // With fractions
//         assert_eq!(world_to_offsets(Vec3::splat(0.1)), UVec3::splat(0));
//         assert_eq!(world_to_offsets(Vec3::splat(16.1)), UVec3::splat(0));
//         assert_eq!(world_to_offsets(Vec3::splat(-15.9)), UVec3::splat(0));
//         assert_eq!(world_to_offsets(Vec3::splat(-15.1)), UVec3::splat(0));

//         assert_eq!(world_to_offsets(Vec3::splat(15.9)), UVec3::splat(15));
//         assert_eq!(world_to_offsets(Vec3::splat(31.9)), UVec3::splat(15));
//         assert_eq!(world_to_offsets(Vec3::splat(-0.9)), UVec3::splat(15));
//         assert_eq!(world_to_offsets(Vec3::splat(-0.1)), UVec3::splat(15));
//     }

//     #[test]
//     fn test_voxel_to_world() {
//         assert_eq!(
//             voxel_to_world(UVec3::splat(0), IVec3::splat(0)),
//             Vec3::splat(0.0)
//         );
//         assert_eq!(
//             voxel_to_world(UVec3::splat(1), IVec3::splat(0)),
//             Vec3::splat(1.0)
//         );
//         assert_eq!(
//             voxel_to_world(UVec3::splat(15), IVec3::splat(0)),
//             Vec3::splat(15.0)
//         );

//         assert_eq!(
//             voxel_to_world(UVec3::splat(0), IVec3::splat(1)),
//             Vec3::splat(16.0)
//         );
//         assert_eq!(
//             voxel_to_world(UVec3::splat(1), IVec3::splat(1)),
//             Vec3::splat(17.0)
//         );
//         assert_eq!(
//             voxel_to_world(UVec3::splat(15), IVec3::splat(1)),
//             Vec3::splat(31.0)
//         );

//         assert_eq!(
//             voxel_to_world(UVec3::splat(0), IVec3::splat(-1)),
//             Vec3::splat(-16.0)
//         );
//         assert_eq!(
//             voxel_to_world(UVec3::splat(1), IVec3::splat(-1)),
//             Vec3::splat(-15.0)
//         );
//         assert_eq!(
//             voxel_to_world(UVec3::splat(15), IVec3::splat(-1)),
//             Vec3::splat(-1.0)
//         );

//         assert_eq!(
//             voxel_to_world(UVec3::new(0, 1, 15), IVec3::splat(-1)),
//             Vec3::new(-16.0, -15.0, -1.0)
//         );
//         assert_eq!(
//             voxel_to_world(UVec3::new(0, 1, 15), IVec3::new(0, 1, 0)),
//             Vec3::new(0.0, 17.0, 15.0)
//         );
//     }

//     #[test]
//     fn test_conversions() {
//         let p = Vec3::splat(0.0);
//         assert_eq!(p, voxel_to_world(world_to_offsets(p), world_to_chunk(p)));

//         let p = Vec3::splat(1.0);
//         assert_eq!(p, voxel_to_world(world_to_offsets(p), world_to_chunk(p)));

//         let p = Vec3::splat(15.0);
//         assert_eq!(p, voxel_to_world(world_to_offsets(p), world_to_chunk(p)));

//         let p = Vec3::splat(-1.0);
//         assert_eq!(p, voxel_to_world(world_to_offsets(p), world_to_chunk(p)));

//         let p = Vec3::splat(-15.0);
//         assert_eq!(p, voxel_to_world(world_to_offsets(p), world_to_chunk(p)));

//         let p = Vec3::new(-1.0, 12.0, 5.0);
//         assert_eq!(p, voxel_to_world(world_to_offsets(p), world_to_chunk(p)));

//         let p = Vec3::new(1.0, -12.0, 5.0);
//         assert_eq!(p, voxel_to_world(world_to_offsets(p), world_to_chunk(p)));

//         let p = Vec3::new(1.0, 12.0, -5.0);
//         assert_eq!(p, voxel_to_world(world_to_offsets(p), world_to_chunk(p)));
//     }
// }

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ChunkPos {
    pub x: usize,
    pub y: usize,
    pub z: usize,
}

impl ChunkPos {
    pub fn new(x: usize, y: usize, z: usize) -> Self {
        Self { x, y, z }
    }

    pub fn from_ivec3(pos: IVec3) -> Self {
        ChunkPos {
            x: pos.x as usize,
            y: pos.y as usize,
            z: pos.z as usize,
        }
    }

    pub fn as_ivec3(&self) -> IVec3 {
        IVec3::new(self.x as i32, self.y as i32, self.z as i32)
    }

    pub fn from_global_coords(x: f32, y: f32, z: f32) -> Self {
        Self {
            x: (x / (ChunkData::edge() as f32)).floor() as usize,
            y: (y / (ChunkData::edge() as f32)).floor() as usize,
            z: (z / (ChunkData::edge() as f32)).floor() as usize,
        }
    }

    pub fn to_global_coords(self) -> (f32, f32, f32) {
        (
            (self.x * ChunkData::edge()) as f32,
            (self.y * ChunkData::edge()) as f32,
            (self.z * ChunkData::edge()) as f32,
        )
    }

    pub fn to_voxel_coords(self) -> VoxelPos {
        VoxelPos::new(
            self.x * ChunkData::edge(),
            self.y * ChunkData::edge(),
            self.z * ChunkData::edge(),
        )
    }

    pub fn neighbors(&self) -> Vec<ChunkPos> {
        vec![
            ChunkPos::new(
                self.x.wrapping_sub(1),
                self.y.wrapping_sub(1),
                self.z.wrapping_sub(1),
            ),
            ChunkPos::new(self.x.wrapping_sub(1), self.y.wrapping_sub(1), self.z),
            ChunkPos::new(self.x.wrapping_sub(1), self.y.wrapping_sub(1), self.z + 1),
            ChunkPos::new(self.x.wrapping_sub(1), self.y, self.z.wrapping_sub(1)),
            ChunkPos::new(self.x.wrapping_sub(1), self.y, self.z),
            ChunkPos::new(self.x.wrapping_sub(1), self.y, self.z + 1),
            ChunkPos::new(self.x.wrapping_sub(1), self.y + 1, self.z.wrapping_sub(1)),
            ChunkPos::new(self.x.wrapping_sub(1), self.y + 1, self.z),
            ChunkPos::new(self.x.wrapping_sub(1), self.y + 1, self.z + 1),
            ChunkPos::new(self.x, self.y.wrapping_sub(1), self.z.wrapping_sub(1)),
            ChunkPos::new(self.x, self.y.wrapping_sub(1), self.z),
            ChunkPos::new(self.x, self.y.wrapping_sub(1), self.z + 1),
            ChunkPos::new(self.x, self.y, self.z.wrapping_sub(1)),
            ChunkPos::new(self.x, self.y, self.z + 1),
            ChunkPos::new(self.x, self.y + 1, self.z.wrapping_sub(1)),
            ChunkPos::new(self.x, self.y + 1, self.z),
            ChunkPos::new(self.x, self.y + 1, self.z + 1),
            ChunkPos::new(self.x + 1, self.y.wrapping_sub(1), self.z.wrapping_sub(1)),
            ChunkPos::new(self.x + 1, self.y.wrapping_sub(1), self.z),
            ChunkPos::new(self.x + 1, self.y.wrapping_sub(1), self.z + 1),
            ChunkPos::new(self.x + 1, self.y, self.z.wrapping_sub(1)),
            ChunkPos::new(self.x + 1, self.y, self.z),
            ChunkPos::new(self.x + 1, self.y, self.z + 1),
            ChunkPos::new(self.x + 1, self.y + 1, self.z.wrapping_sub(1)),
            ChunkPos::new(self.x + 1, self.y + 1, self.z),
            ChunkPos::new(self.x + 1, self.y + 1, self.z + 1),
        ]
    }

    pub fn distance(&self, other: &ChunkPos) -> f32 {
        Vec3::new(self.x as f32, self.y as f32, self.z as f32).distance(Vec3::new(
            other.x as f32,
            other.y as f32,
            other.z as f32,
        ))
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VoxelPos {
    pub x: usize,
    pub y: usize,
    pub z: usize,
}

#[allow(dead_code)]
impl VoxelPos {
    pub fn new(x: usize, y: usize, z: usize) -> Self {
        Self { x, y, z }
    }

    pub fn as_vec3(&self) -> Vec3 {
        Vec3::new(self.x as f32, self.y as f32, self.z as f32)
    }

    pub fn from_chunk_coords(
        chunk_pos: ChunkPos,
        chunk_local_x: usize,
        chunk_local_y: usize,
        chunk_local_z: usize,
    ) -> Self {
        let chunk_voxel_pos = chunk_pos.to_voxel_coords();
        Self::new(
            chunk_voxel_pos.x + chunk_local_x,
            chunk_voxel_pos.y + chunk_local_y,
            chunk_voxel_pos.z + chunk_local_z,
        )
    }

    pub fn to_chunk_coords(self) -> (ChunkPos, usize, usize, usize) {
        (
            ChunkPos::new(
                self.x / ChunkData::edge(),
                self.y / ChunkData::edge(),
                self.z / ChunkData::edge(),
            ),
            self.x.rem_euclid(ChunkData::edge()),
            self.y.rem_euclid(ChunkData::edge()),
            self.z.rem_euclid(ChunkData::edge()),
        )
    }

    pub fn from_global_coords(x: f32, y: f32, z: f32) -> Self {
        Self {
            x: x.floor() as usize,
            y: y.floor() as usize,
            z: z.floor() as usize,
        }
    }

    pub fn neighbors(&self) -> [VoxelPos; 6] {
        [
            VoxelPos::new(self.x + 1, self.y, self.z),
            VoxelPos::new(self.x.wrapping_sub(1), self.y, self.z),
            VoxelPos::new(self.x, self.y + 1, self.z),
            VoxelPos::new(self.x, self.y.wrapping_sub(1), self.z),
            VoxelPos::new(self.x, self.y, self.z + 1),
            VoxelPos::new(self.x, self.y, self.z.wrapping_sub(1)),
        ]
    }

    pub fn distance(&self, other: &VoxelPos) -> f32 {
        Vec3::new(self.x as f32, self.y as f32, self.y as f32).distance(Vec3::new(
            other.x as f32,
            other.y as f32,
            other.z as f32,
        ))
    }
}
