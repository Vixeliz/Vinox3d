use bevy::{
    math::Vec3A,
    pbr::{NotShadowCaster, NotShadowReceiver},
    prelude::*,
    reflect::TypeUuid,
    render::{
        mesh::Indices,
        primitives::Aabb,
        render_resource::{AsBindGroup, PrimitiveTopology, ShaderRef},
    },
    tasks::{AsyncComputeTaskPool, ComputeTaskPool, Task},
    utils::FloatOrd,
};
use bevy_tweening::{lens::TransformPositionLens, *};
use futures_lite::future;
use itertools::Itertools;
use rand::{rngs::StdRng, Rng, SeedableRng};
use rustc_hash::FxHashMap;
// use rand::seq::IteratorRandom;
use serde_big_array::Array;
use std::{ops::Deref, time::Duration};
use tokio::sync::mpsc::{Receiver, Sender};
use vinox_common::{
    storage::geometry::descriptor::{BlockGeo, GeometryDescriptor},
    world::chunks::{
        ecs::{ChunkManager, ChunkUpdate, CurrentChunks, NeedsMesh, PriorityMesh},
        positions::{voxel_to_world, world_to_global_voxel, ChunkPos},
        storage::{self, BlockTable, ChunkData, RenderedBlockData, VoxelVisibility, CHUNK_SIZE},
    },
};

use crate::states::{
    assets::load::LoadableAssets,
    components::GameOptions,
    game::world::chunks::{PlayerBlock, PlayerChunk},
};

use super::chunk::ChunkBoundary;

#[derive(Resource, Clone, Default, Deref, DerefMut)]
pub struct GeometryTable(pub FxHashMap<String, GeometryDescriptor>);

pub const EMPTY: VoxelVisibility = VoxelVisibility::Empty;
pub const OPAQUE: VoxelVisibility = VoxelVisibility::Opaque;
pub const TRANSPARENT: VoxelVisibility = VoxelVisibility::Transparent;

#[derive(Clone, Debug)]
pub struct Quad {
    pub voxel: [usize; 3],
    pub start: (i8, i8, i8),
    pub end: (i8, i8, i8),
    pub cube: usize,
    pub data: RenderedBlockData,
}

#[derive(Default)]
pub struct QuadGroups {
    pub groups: [Vec<Quad>; 6],
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Axis {
    X,
    Y,
    Z,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Side {
    pub axis: Axis,
    pub positive: bool,
}

impl Side {
    pub fn new(axis: Axis, positive: bool) -> Self {
        Self { axis, positive }
    }

    pub fn normal(&self) -> [f32; 3] {
        match (&self.axis, &self.positive) {
            (Axis::X, true) => [1.0, 0.0, 0.0],   // X+
            (Axis::X, false) => [-1.0, 0.0, 0.0], // X-
            (Axis::Y, true) => [0.0, 1.0, 0.0],   // Y+
            (Axis::Y, false) => [0.0, -1.0, 0.0], // Y-
            (Axis::Z, true) => [0.0, 0.0, 1.0],   // Z+
            (Axis::Z, false) => [0.0, 0.0, -1.0], // Z-
        }
    }

    pub fn normals(&self) -> [[f32; 3]; 4] {
        [self.normal(), self.normal(), self.normal(), self.normal()]
    }
}

pub struct Face<'a> {
    side: Side,
    quad: &'a Quad,
}

impl From<usize> for Side {
    fn from(value: usize) -> Self {
        match value {
            0 => Self::new(Axis::X, false), // X-
            1 => Self::new(Axis::X, true),  // X+
            2 => Self::new(Axis::Y, false), // Y-
            3 => Self::new(Axis::Y, true),  // Y+
            4 => Self::new(Axis::Z, false), // Z-
            5 => Self::new(Axis::Z, true),  // Z+
            _ => unreachable!(),
        }
    }
}
impl QuadGroups {
    pub fn iter(&self) -> impl Iterator<Item = Face> {
        self.groups
            .iter()
            .enumerate()
            .flat_map(|(index, quads)| quads.iter().map(move |quad| (index, quad)))
            .map(|(index, quad)| Face {
                side: index.into(),
                quad,
            })
    }

    pub fn iter_with_ao<'a>(
        &'a self,
        chunk: &'a ChunkBoundary,
    ) -> impl Iterator<Item = FaceWithAO<'a>> {
        self.iter().map(|face| FaceWithAO::new(face, chunk))
    }

    pub fn clear(&mut self) {
        self.groups.iter_mut().for_each(|g| g.clear());
    }
}

pub fn face_aos(face: &Face, chunk: &ChunkBoundary) -> [u32; 4] {
    let [x, y, z] = face.voxel();
    // let (x, y, z) = (x as u32, y as u32, z as u32);

    match (face.side.axis, face.side.positive) {
        (Axis::X, false) => side_aos([
            (
                chunk.voxels()[ChunkBoundary::linearize(x - 1, y, z + 1)],
                BlockGeo::default(),
            ),
            (
                chunk.voxels()[ChunkBoundary::linearize(x - 1, y - 1, z + 1)],
                BlockGeo::default(),
            ),
            (
                chunk.voxels()[ChunkBoundary::linearize(x - 1, y - 1, z)],
                BlockGeo::default(),
            ),
            (
                chunk.voxels()[ChunkBoundary::linearize(x - 1, y - 1, z - 1)],
                BlockGeo::default(),
            ),
            (
                chunk.voxels()[ChunkBoundary::linearize(x - 1, y, z - 1)],
                BlockGeo::default(),
            ),
            (
                chunk.voxels()[ChunkBoundary::linearize(x - 1, y + 1, z - 1)],
                BlockGeo::default(),
            ),
            (
                chunk.voxels()[ChunkBoundary::linearize(x - 1, y + 1, z)],
                BlockGeo::default(),
            ),
            (
                chunk.voxels()[ChunkBoundary::linearize(x - 1, y + 1, z + 1)],
                BlockGeo::default(),
            ),
        ]),
        (Axis::X, true) => side_aos([
            (
                chunk.voxels()[ChunkBoundary::linearize(x + 1, y, z - 1)],
                BlockGeo::default(),
            ),
            (
                chunk.voxels()[ChunkBoundary::linearize(x + 1, y - 1, z - 1)],
                BlockGeo::default(),
            ),
            (
                chunk.voxels()[ChunkBoundary::linearize(x + 1, y - 1, z)],
                BlockGeo::default(),
            ),
            (
                chunk.voxels()[ChunkBoundary::linearize(x + 1, y - 1, z + 1)],
                BlockGeo::default(),
            ),
            (
                chunk.voxels()[ChunkBoundary::linearize(x + 1, y, z + 1)],
                BlockGeo::default(),
            ),
            (
                chunk.voxels()[ChunkBoundary::linearize(x + 1, y + 1, z + 1)],
                BlockGeo::default(),
            ),
            (
                chunk.voxels()[ChunkBoundary::linearize(x + 1, y + 1, z)],
                BlockGeo::default(),
            ),
            (
                chunk.voxels()[ChunkBoundary::linearize(x + 1, y + 1, z - 1)],
                BlockGeo::default(),
            ),
        ]),
        (Axis::Y, false) => side_aos([
            (
                chunk.voxels()[ChunkBoundary::linearize(x - 1, y - 1, z)],
                BlockGeo::default(),
            ),
            (
                chunk.voxels()[ChunkBoundary::linearize(x - 1, y - 1, z + 1)],
                BlockGeo::default(),
            ),
            (
                chunk.voxels()[ChunkBoundary::linearize(x, y - 1, z + 1)],
                BlockGeo::default(),
            ),
            (
                chunk.voxels()[ChunkBoundary::linearize(x + 1, y - 1, z + 1)],
                BlockGeo::default(),
            ),
            (
                chunk.voxels()[ChunkBoundary::linearize(x + 1, y - 1, z)],
                BlockGeo::default(),
            ),
            (
                chunk.voxels()[ChunkBoundary::linearize(x + 1, y - 1, z - 1)],
                BlockGeo::default(),
            ),
            (
                chunk.voxels()[ChunkBoundary::linearize(x, y - 1, z - 1)],
                BlockGeo::default(),
            ),
            (
                chunk.voxels()[ChunkBoundary::linearize(x - 1, y - 1, z - 1)],
                BlockGeo::default(),
            ),
        ]),
        (Axis::Y, true) => side_aos([
            (
                chunk.voxels()[ChunkBoundary::linearize(x, y + 1, z + 1)],
                BlockGeo::default(),
            ),
            (
                chunk.voxels()[ChunkBoundary::linearize(x - 1, y + 1, z + 1)],
                BlockGeo::default(),
            ),
            (
                chunk.voxels()[ChunkBoundary::linearize(x - 1, y + 1, z)],
                BlockGeo::default(),
            ),
            (
                chunk.voxels()[ChunkBoundary::linearize(x - 1, y + 1, z - 1)],
                BlockGeo::default(),
            ),
            (
                chunk.voxels()[ChunkBoundary::linearize(x, y + 1, z - 1)],
                BlockGeo::default(),
            ),
            (
                chunk.voxels()[ChunkBoundary::linearize(x + 1, y + 1, z - 1)],
                BlockGeo::default(),
            ),
            (
                chunk.voxels()[ChunkBoundary::linearize(x + 1, y + 1, z)],
                BlockGeo::default(),
            ),
            (
                chunk.voxels()[ChunkBoundary::linearize(x + 1, y + 1, z + 1)],
                BlockGeo::default(),
            ),
        ]),
        (Axis::Z, false) => side_aos([
            (
                chunk.voxels()[ChunkBoundary::linearize(x - 1, y, z - 1)],
                BlockGeo::default(),
            ),
            (
                chunk.voxels()[ChunkBoundary::linearize(x - 1, y - 1, z - 1)],
                BlockGeo::default(),
            ),
            (
                chunk.voxels()[ChunkBoundary::linearize(x, y - 1, z - 1)],
                BlockGeo::default(),
            ),
            (
                chunk.voxels()[ChunkBoundary::linearize(x + 1, y - 1, z - 1)],
                BlockGeo::default(),
            ),
            (
                chunk.voxels()[ChunkBoundary::linearize(x + 1, y, z - 1)],
                BlockGeo::default(),
            ),
            (
                chunk.voxels()[ChunkBoundary::linearize(x + 1, y + 1, z - 1)],
                BlockGeo::default(),
            ),
            (
                chunk.voxels()[ChunkBoundary::linearize(x, y + 1, z - 1)],
                BlockGeo::default(),
            ),
            (
                chunk.voxels()[ChunkBoundary::linearize(x - 1, y + 1, z - 1)],
                BlockGeo::default(),
            ),
        ]),
        (Axis::Z, true) => side_aos([
            (
                chunk.voxels()[ChunkBoundary::linearize(x + 1, y, z + 1)],
                BlockGeo::default(),
            ),
            (
                chunk.voxels()[ChunkBoundary::linearize(x + 1, y - 1, z + 1)],
                BlockGeo::default(),
            ),
            (
                chunk.voxels()[ChunkBoundary::linearize(x, y - 1, z + 1)],
                BlockGeo::default(),
            ),
            (
                chunk.voxels()[ChunkBoundary::linearize(x - 1, y - 1, z + 1)],
                BlockGeo::default(),
            ),
            (
                chunk.voxels()[ChunkBoundary::linearize(x - 1, y, z + 1)],
                BlockGeo::default(),
            ),
            (
                chunk.voxels()[ChunkBoundary::linearize(x - 1, y + 1, z + 1)],
                BlockGeo::default(),
            ),
            (
                chunk.voxels()[ChunkBoundary::linearize(x, y + 1, z + 1)],
                BlockGeo::default(),
            ),
            (
                chunk.voxels()[ChunkBoundary::linearize(x + 1, y + 1, z + 1)],
                BlockGeo::default(),
            ),
        ]),
    }
}

pub struct FaceWithAO<'a> {
    face: Face<'a>,
    aos: [u32; 4],
}

impl<'a> FaceWithAO<'a> {
    pub fn new(face: Face<'a>, chunk: &ChunkBoundary) -> Self {
        let aos = face_aos(&face, chunk);
        Self { face, aos }
    }

    pub fn aos(&self) -> [u32; 4] {
        self.aos
    }

    pub fn indices(&self, start: u32) -> [u32; 6] {
        let aos = self.aos();

        if (aos[1] + aos[2]) > (aos[0] + aos[3]) {
            [start, start + 2, start + 1, start + 1, start + 2, start + 3]
        } else {
            [start, start + 3, start + 1, start, start + 2, start + 3]
        }
    }
}

pub(crate) fn ao_value(side1: bool, corner: bool, side2: bool) -> u32 {
    match (side1, corner, side2) {
        (true, _, true) => 0,
        (true, true, false) | (false, true, true) => 1,
        (false, false, false) => 3,
        _ => 2,
    }
}

pub(crate) fn side_aos(neighbors: [(RenderedBlockData, BlockGeo); 8]) -> [u32; 4] {
    let ns = [
        neighbors[0].0.visibility == OPAQUE && neighbors[0].1 == BlockGeo::default(),
        neighbors[1].0.visibility == OPAQUE && neighbors[1].1 == BlockGeo::default(),
        neighbors[2].0.visibility == OPAQUE && neighbors[2].1 == BlockGeo::default(),
        neighbors[3].0.visibility == OPAQUE && neighbors[3].1 == BlockGeo::default(),
        neighbors[4].0.visibility == OPAQUE && neighbors[4].1 == BlockGeo::default(),
        neighbors[5].0.visibility == OPAQUE && neighbors[5].1 == BlockGeo::default(),
        neighbors[6].0.visibility == OPAQUE && neighbors[6].1 == BlockGeo::default(),
        neighbors[7].0.visibility == OPAQUE && neighbors[7].1 == BlockGeo::default(),
    ];

    [
        ao_value(ns[0], ns[1], ns[2]),
        ao_value(ns[2], ns[3], ns[4]),
        ao_value(ns[6], ns[7], ns[0]),
        ao_value(ns[4], ns[5], ns[6]),
    ]
}

impl<'a> Deref for FaceWithAO<'a> {
    type Target = Face<'a>;

    fn deref(&self) -> &Self::Target {
        &self.face
    }
}

impl<'a> Face<'a> {
    pub fn indices(&self, start: u32) -> [u32; 6] {
        [start, start + 2, start + 1, start + 1, start + 2, start + 3]
    }

    pub fn positions(
        &self,
        voxel_size: f32,
        chunk: &ChunkBoundary,
        // geo: &GeometryDescriptor,
        // direction: Option<storage::Direction>,
        // top: Option<bool>,
    ) -> [[f32; 3]; 4] {
        let (min_one, min_two, max_one, max_two, min_self, max_self) = (
            (self.quad.start.0 as f32 / 16.0),
            (self.quad.start.1 as f32 / 16.0),
            (self.quad.end.0 as f32 / 16.0),
            (self.quad.end.1 as f32 / 16.0),
            (self.quad.start.2 as f32 / 16.0),
            (self.quad.end.2 as f32 / 16.0),
        );
        let positions = match (&self.side.axis, &self.side.positive) {
            (Axis::X, false) => [
                [min_self, min_one, max_two],
                [min_self, min_one, min_two],
                [min_self, max_one, max_two],
                [min_self, max_one, min_two],
            ],
            (Axis::X, true) => [
                [max_self, min_one, min_two],
                [max_self, min_one, max_two],
                [max_self, max_one, min_two],
                [max_self, max_one, max_two],
            ],
            (Axis::Y, false) => [
                [min_one, min_self, max_two],
                [max_one, min_self, max_two],
                [min_one, min_self, min_two],
                [max_one, min_self, min_two],
            ],
            (Axis::Y, true) => [
                [min_one, max_self, max_two],
                [min_one, max_self, min_two],
                [max_one, max_self, max_two],
                [max_one, max_self, min_two],
            ],
            (Axis::Z, false) => [
                [min_one, min_two, min_self],
                [max_one, min_two, min_self],
                [min_one, max_two, min_self],
                [max_one, max_two, min_self],
            ],
            (Axis::Z, true) => [
                [max_one, min_two, max_self],
                [min_one, min_two, max_self],
                [max_one, max_two, max_self],
                [min_one, max_two, max_self],
            ],
        };

        let (x, y, z) = (
            (self.quad.voxel[0] - 1) as f32,
            (self.quad.voxel[1] - 1) as f32,
            (self.quad.voxel[2] - 1) as f32,
        );
        let mut temp_arr = [
            Vec3::new(
                x * voxel_size + positions[0][0] * voxel_size,
                y * voxel_size + positions[0][1] * voxel_size,
                z * voxel_size + positions[0][2] * voxel_size,
            ),
            Vec3::new(
                x * voxel_size + positions[1][0] * voxel_size,
                y * voxel_size + positions[1][1] * voxel_size,
                z * voxel_size + positions[1][2] * voxel_size,
            ),
            Vec3::new(
                x * voxel_size + positions[2][0] * voxel_size,
                y * voxel_size + positions[2][1] * voxel_size,
                z * voxel_size + positions[2][2] * voxel_size,
            ),
            Vec3::new(
                x * voxel_size + positions[3][0] * voxel_size,
                y * voxel_size + positions[3][1] * voxel_size,
                z * voxel_size + positions[3][2] * voxel_size,
            ),
        ];
        let geo = chunk.geometry_pal.get(self.quad.data.geo_index).unwrap();
        let cube_pivot = geo.cubes.get(self.quad.cube).unwrap().pivot;
        let cube_rotation = geo.cubes.get(self.quad.cube).unwrap().rotation;
        let block_pivot = geo.pivot;
        let block_rotation = geo.rotation;
        if (cube_rotation != (0, 0, 0) || block_rotation != (0, 0, 0))
            && self.quad.data.direction.is_none()
            && self.quad.data.top.is_none()
        {
            let pivot = Vec3::new(
                block_pivot.0 as f32 / 16.0 + x,
                block_pivot.1 as f32 / 16.0 + y,
                block_pivot.2 as f32 / 16.0 + z,
            ); // TO emulate how itll be getting from geometry
            let rotation = Quat::from_euler(
                EulerRot::XYZ,
                (block_rotation.0 as f32).to_radians(),
                (block_rotation.1 as f32).to_radians(),
                (block_rotation.2 as f32).to_radians(),
            );
            let pivot_cube = Vec3::new(
                cube_pivot.0 as f32 / 16.0 + x,
                cube_pivot.1 as f32 / 16.0 + y,
                cube_pivot.2 as f32 / 16.0 + z,
            ); // TO emulate how itll be getting from geometry
            let rotation_cube = Quat::from_euler(
                EulerRot::XYZ,
                (cube_rotation.0 as f32).to_radians(),
                (cube_rotation.1 as f32).to_radians(),
                (cube_rotation.2 as f32).to_radians(),
            );
            for point in temp_arr.iter_mut() {
                // let mut temp_transform = Transform::from_translation(*point);
                // temp_transform.rotate_around(pivot, rotation);
                // *point = temp_transform.translation;
                *point = pivot + rotation * (*point - pivot);
                *point = pivot_cube + rotation_cube * (*point - pivot_cube);
                if let Some(direction) = self.quad.data.direction.clone() {
                    let pivot = Vec3::new(0.5 + x, 0.5 + y, 0.5 + z); // TO emulate how itll be getting from geometry
                    let rotation = match direction {
                        storage::Direction::North => {
                            Quat::from_euler(EulerRot::XYZ, -90.0_f32.to_radians(), 0.0, 0.0)
                        }
                        storage::Direction::South => {
                            Quat::from_euler(EulerRot::XYZ, 90.0_f32.to_radians(), 0.0, 0.0)
                        }
                        storage::Direction::West => {
                            Quat::from_euler(EulerRot::XYZ, 0.0, 0.0, 90.0_f32.to_radians())
                        }
                        storage::Direction::East => {
                            Quat::from_euler(EulerRot::XYZ, 0.0, 0.0, -90.0_f32.to_radians())
                        }
                    };
                    *point = pivot + rotation * (*point - pivot);
                }
                if let Some(top) = self.quad.data.top {
                    let pivot = Vec3::new(0.5 + x, 0.5 + y, 0.5 + z); // TO emulate how itll be getting from geometry
                    let rotation = if top {
                        Quat::from_euler(EulerRot::XYZ, 180.0_f32.to_radians(), 0.0, 0.0)
                    } else {
                        Quat::from_euler(EulerRot::XYZ, 0.0, 0.0, 0.0)
                    };
                    *point = pivot + rotation * (*point - pivot);
                }
            }
        }
        let mut final_arr = [
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
        ];

        for (point_num, point) in temp_arr.iter_mut().enumerate() {
            final_arr[point_num] = Into::<[f32; 3]>::into(*point);
        }

        final_arr
    }

    pub fn normals(&self) -> [[f32; 3]; 4] {
        self.side.normals()
    }

    pub fn uvs(
        &self,
        flip_u: bool,
        flip_v: bool,
        // geo: &GeometryDescriptor,
        texture_atlas: &TextureAtlas,
        matched_ind: usize,
        loadable_assets: &LoadableAssets,
        // block: &RenderedBlockData,
        // descriptor: &BlockDescriptor,
        world_pos: IVec3,
        chunk: &ChunkBoundary,
    ) -> [[f32; 2]; 4] {
        let texture_index = self.quad.data.textures[matched_ind];
        let geo = chunk.geometry_pal.get(self.quad.data.geo_index).unwrap();
        let uv = geo.cubes.get(self.quad.cube).unwrap().uv;
        let mut face_tex = [[0.0; 2]; 4];
        let min_x = texture_atlas.textures.get(texture_index).unwrap().min.x;
        let min_y = texture_atlas.textures.get(texture_index).unwrap().min.y;
        let face_index = match (&self.side.axis, &self.side.positive) {
            (Axis::X, false) => 0,
            (Axis::X, true) => 1,
            (Axis::Y, false) => 2,
            (Axis::Y, true) => 3,
            (Axis::Z, false) => 4,
            (Axis::Z, true) => 5,
        };
        let (min_x, min_y) = (
            min_x + uv.get(face_index).unwrap().0 .0 as f32,
            min_y + uv.get(face_index).unwrap().0 .1 as f32,
        );
        let (max_x, max_y) = (
            min_x + uv.get(face_index).unwrap().1 .0 as f32,
            min_y + uv.get(face_index).unwrap().1 .1 as f32,
        );
        let (min_x, min_y, max_x, max_y) = (
            min_x / texture_atlas.size.x,
            min_y / texture_atlas.size.y,
            max_x / texture_atlas.size.x,
            max_y / texture_atlas.size.y,
        );
        let flip_num = if self.quad.data.tex_variance[face_index] {
            let mut rng: StdRng = SeedableRng::seed_from_u64(world_pos.reflect_hash().unwrap());
            rng.gen_range(0..6)
        } else {
            0
        };
        match flip_num {
            0 => {
                face_tex[2][0] = min_x;
                face_tex[2][1] = min_y;
                face_tex[3][0] = max_x;
                face_tex[3][1] = min_y;
                face_tex[0][0] = min_x;
                face_tex[0][1] = max_y;
                face_tex[1][0] = max_x;
                face_tex[1][1] = max_y;
            }
            1 => {
                face_tex[2][0] = max_x;
                face_tex[2][1] = max_y;
                face_tex[3][0] = min_x;
                face_tex[3][1] = max_y;
                face_tex[0][0] = max_x;
                face_tex[0][1] = min_y;
                face_tex[1][0] = min_x;
                face_tex[1][1] = min_y;
            }
            2 => {
                face_tex[2][0] = max_x;
                face_tex[2][1] = min_y;
                face_tex[3][0] = min_x;
                face_tex[3][1] = min_y;
                face_tex[0][0] = max_x;
                face_tex[0][1] = max_y;
                face_tex[1][0] = min_x;
                face_tex[1][1] = max_y;
            }
            3 => {
                face_tex[2][0] = min_x;
                face_tex[2][1] = max_y;
                face_tex[3][0] = max_x;
                face_tex[3][1] = max_y;
                face_tex[0][0] = min_x;
                face_tex[0][1] = min_y;
                face_tex[1][0] = max_x;
                face_tex[1][1] = min_y;
            }
            4 => {
                face_tex[2][0] = max_x;
                face_tex[2][1] = max_y;
                face_tex[3][0] = max_x;
                face_tex[3][1] = min_y;
                face_tex[0][0] = min_x;
                face_tex[0][1] = max_y;
                face_tex[1][0] = min_x;
                face_tex[1][1] = min_y;
            }
            5 => {
                face_tex[2][0] = min_x;
                face_tex[2][1] = min_y;
                face_tex[3][0] = min_x;
                face_tex[3][1] = max_y;
                face_tex[0][0] = max_x;
                face_tex[0][1] = min_y;
                face_tex[1][0] = max_x;
                face_tex[1][1] = max_y;
            }
            _ => {}
        }
        return face_tex;

        // match (flip_u, flip_v) {
        //     (true, true) => [[1.0, 1.0], [0.0, 1.0], [1.0, 0.0], [0.0, 0.0]],
        //     (true, false) => [[1.0, 0.0], [0.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
        //     (false, true) => [[0.0, 1.0], [1.0, 1.0], [0.0, 0.0], [1.0, 0.0]],
        //     (false, false) => [[0.0, 0.0], [1.0, 0.0], [0.0, 1.0], [1.0, 1.0]],
        // }
    }

    pub fn voxel(&self) -> [usize; 3] {
        self.quad.voxel
    }
}

#[derive(AsBindGroup, TypeUuid, Debug, Clone)]
#[uuid = "f690fdae-d598-45ab-8225-97e2a3f056e0"]
pub struct BasicMaterial {
    #[uniform(0)]
    pub color: Color,
    #[uniform(0)]
    pub discard_pix: u32,
    #[texture(1)]
    #[sampler(2)]
    pub color_texture: Option<Handle<Image>>,
    pub alpha_mode: AlphaMode,
}

impl Material for BasicMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/basic_material.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        self.alpha_mode
    }
}

#[derive(Bundle)]
pub struct RenderedChunk {
    #[bundle]
    pub mesh: PbrBundle,
    pub aabb: Aabb,
}

#[derive(Default, Resource)]
pub struct MeshQueue {
    pub mesh: Vec<(IVec3, ChunkData, Box<Array<ChunkData, 26>>)>,
    pub priority: Vec<(IVec3, ChunkData, Box<Array<ChunkData, 26>>)>,
}

#[derive(Component)]
pub struct ComputeMesh(Task<MeshedChunk>);

#[derive(Component)]
pub struct PriorityComputeMesh(Task<MeshedChunk>);

pub fn process_priority_task(
    mut commands: Commands,
    mut mesh_tasks: Query<(Entity, &mut PriorityComputeMesh)>,
    mut meshes: ResMut<Assets<Mesh>>,
    chunk_material: Res<ChunkMaterial>,
    current_chunks: Res<CurrentChunks>,
) {
    mesh_tasks.for_each_mut(|(entity, mut task)| {
        if let Some(chunk) = future::block_on(future::poll_once(&mut task.0)) {
            if let Some(chunk_entity) = current_chunks.get_entity(chunk.pos) {
                commands.entity(chunk_entity).despawn_descendants();

                let chunk_pos = Vec3::new(
                    (chunk.pos.x * (CHUNK_SIZE) as i32) as f32,
                    (chunk.pos.y * (CHUNK_SIZE) as i32) as f32,
                    (chunk.pos.z * (CHUNK_SIZE) as i32) as f32,
                );

                let trans_entity = commands
                    .spawn((
                        RenderedChunk {
                            aabb: Aabb {
                                center: Vec3A::new(
                                    (CHUNK_SIZE / 2) as f32,
                                    (CHUNK_SIZE / 2) as f32,
                                    (CHUNK_SIZE / 2) as f32,
                                ),
                                half_extents: Vec3A::new(
                                    (CHUNK_SIZE / 2) as f32,
                                    (CHUNK_SIZE / 2) as f32,
                                    (CHUNK_SIZE / 2) as f32,
                                ),
                            },
                            mesh: MaterialMeshBundle {
                                mesh: meshes.add(chunk.transparent_mesh.clone()),
                                material: chunk_material.transparent.clone(),
                                ..Default::default()
                            },
                        },
                        NotShadowCaster,
                        NotShadowReceiver,
                    ))
                    .id();

                commands.entity(chunk_entity).insert((
                    RenderedChunk {
                        aabb: Aabb {
                            center: Vec3A::new(
                                (CHUNK_SIZE / 2) as f32,
                                (CHUNK_SIZE / 2) as f32,
                                (CHUNK_SIZE / 2) as f32,
                            ),
                            half_extents: Vec3A::new(
                                (CHUNK_SIZE / 2) as f32,
                                (CHUNK_SIZE / 2) as f32,
                                (CHUNK_SIZE / 2) as f32,
                            ),
                        },
                        mesh: MaterialMeshBundle {
                            mesh: meshes.add(chunk.chunk_mesh.clone()),
                            material: chunk_material.opaque.clone(),
                            transform: Transform::from_translation(chunk_pos),
                            ..Default::default()
                        },
                    },
                    NotShadowCaster,
                    NotShadowReceiver,
                ));

                commands.entity(chunk_entity).push_children(&[trans_entity]);
                commands.entity(entity).despawn_recursive();
            } else {
                commands.entity(entity).despawn_recursive();
            }
        }
    });
}

pub fn process_task(
    mut commands: Commands,
    mut mesh_tasks: Query<(Entity, &mut ComputeMesh)>,
    mut meshes: ResMut<Assets<Mesh>>,
    chunk_material: Res<ChunkMaterial>,
    chunks: Query<&ChunkPos, With<NeedsMesh>>,
    player_chunk: Res<PlayerChunk>,
    current_chunks: Res<CurrentChunks>,
) {
    mesh_tasks.for_each_mut(|(entity, mut task)| {
        if let Some(chunk) = future::block_on(future::poll_once(&mut task.0)) {
            if let Some(chunk_entity) = current_chunks.get_entity(chunk.pos) {
                commands.entity(chunk_entity).despawn_descendants();

                let chunk_pos = Vec3::new(
                    (chunk.pos.x * (CHUNK_SIZE) as i32) as f32,
                    (chunk.pos.y * (CHUNK_SIZE) as i32) as f32,
                    (chunk.pos.z * (CHUNK_SIZE) as i32) as f32,
                );

                let tween = Tween::new(
                    EaseFunction::QuadraticInOut,
                    Duration::from_secs(1),
                    TransformPositionLens {
                        start: Vec3::new(chunk_pos.x, chunk_pos.y - CHUNK_SIZE as f32, chunk_pos.z),
                        end: chunk_pos,
                    },
                )
                .with_repeat_count(RepeatCount::Finite(1));

                let chunk_pos = if chunks.get(chunk_entity).is_err()
                    && chunk
                        .pos
                        .as_vec3()
                        .distance(player_chunk.chunk_pos.as_vec3())
                        > 4.0
                {
                    commands.entity(chunk_entity).insert(Animator::new(tween));
                    Vec3::new(chunk_pos.x, chunk_pos.y - CHUNK_SIZE as f32, chunk_pos.z)
                } else {
                    chunk_pos
                };

                let trans_entity = commands
                    .spawn((
                        RenderedChunk {
                            aabb: Aabb {
                                center: Vec3A::new(
                                    (CHUNK_SIZE / 2) as f32,
                                    (CHUNK_SIZE / 2) as f32,
                                    (CHUNK_SIZE / 2) as f32,
                                ),
                                half_extents: Vec3A::new(
                                    (CHUNK_SIZE / 2) as f32,
                                    (CHUNK_SIZE / 2) as f32,
                                    (CHUNK_SIZE / 2) as f32,
                                ),
                            },
                            mesh: MaterialMeshBundle {
                                mesh: meshes.add(chunk.transparent_mesh.clone()),
                                material: chunk_material.transparent.clone(),
                                ..Default::default()
                            },
                        },
                        NotShadowCaster,
                        NotShadowReceiver,
                    ))
                    .id();

                commands.entity(chunk_entity).insert((
                    RenderedChunk {
                        aabb: Aabb {
                            center: Vec3A::new(
                                (CHUNK_SIZE / 2) as f32,
                                (CHUNK_SIZE / 2) as f32,
                                (CHUNK_SIZE / 2) as f32,
                            ),
                            half_extents: Vec3A::new(
                                (CHUNK_SIZE / 2) as f32,
                                (CHUNK_SIZE / 2) as f32,
                                (CHUNK_SIZE / 2) as f32,
                            ),
                        },
                        mesh: MaterialMeshBundle {
                            mesh: meshes.add(chunk.chunk_mesh.clone()),
                            material: chunk_material.opaque.clone(),
                            transform: Transform::from_translation(chunk_pos),
                            ..Default::default()
                        },
                    },
                    NotShadowCaster,
                    NotShadowReceiver,
                ));

                commands.entity(chunk_entity).push_children(&[trans_entity]);
                commands.entity(entity).despawn_recursive();
            } else {
                commands.entity(entity).despawn_recursive();
            }
        }
    });
}

// #[derive(Resource)]
// pub struct PriorityMeshChannel {
//     pub tx: Sender<MeshedChunk>,
//     pub rx: Receiver<MeshedChunk>,
// }

// impl Default for PriorityMeshChannel {
//     fn default() -> Self {
//         let (tx, rx) = tokio::sync::mpsc::channel(256);
//         Self { tx, rx }
//     }
// }

// #[derive(Resource)]
// pub struct MeshChannel {
//     pub tx: Sender<MeshedChunk>,
//     pub rx: Receiver<MeshedChunk>,
// }

// impl Default for MeshChannel {
//     fn default() -> Self {
//         let (tx, rx) = tokio::sync::mpsc::channel(1024);
//         Self { tx, rx }
//     }
// }

// Possibly have this just fully generate the mesh
pub fn generate_mesh(chunk: &ChunkBoundary, solid_pass: bool, buffer: &mut QuadGroups) {
    buffer.clear();
    for z in 1..ChunkBoundary::edge() - 1 {
        for y in 1..ChunkBoundary::edge() - 1 {
            for x in 1..ChunkBoundary::edge() - 1 {
                let voxel = chunk.voxels()[ChunkBoundary::linearize(x, y, z)];
                match voxel.visibility {
                    EMPTY => continue,
                    visibility => {
                        let neighbor_block = [
                            chunk.voxels()[ChunkBoundary::linearize(x - 1, y, z)],
                            chunk.voxels()[ChunkBoundary::linearize(x + 1, y, z)],
                            chunk.voxels()[ChunkBoundary::linearize(x, y - 1, z)],
                            chunk.voxels()[ChunkBoundary::linearize(x, y + 1, z)],
                            chunk.voxels()[ChunkBoundary::linearize(x, y, z - 1)],
                            chunk.voxels()[ChunkBoundary::linearize(x, y, z + 1)],
                        ];
                        let geo = chunk.geometry_pal.get(voxel.geo_index).unwrap();
                        for (cube_num, cube) in geo.cubes.iter().enumerate() {
                            for (i, neighbor) in neighbor_block.iter().enumerate() {
                                let culled = cube.cull[i];
                                if cube.discard[i] {
                                    continue;
                                }
                                let blocked = match i {
                                    0 => neighbor_block[i].blocks[1],
                                    1 => neighbor_block[i].blocks[0],
                                    2 => neighbor_block[i].blocks[3],
                                    3 => neighbor_block[i].blocks[2],
                                    4 => neighbor_block[i].blocks[5],
                                    5 => neighbor_block[i].blocks[4],
                                    _ => true,
                                };
                                let other = neighbor.visibility;
                                let generate = if culled && blocked {
                                    if solid_pass {
                                        match (visibility, other) {
                                            (OPAQUE, EMPTY) | (OPAQUE, TRANSPARENT) => true,

                                            (TRANSPARENT, TRANSPARENT) => voxel != *neighbor,

                                            (_, _) => false,
                                        }
                                    } else {
                                        match (visibility, other) {
                                            (TRANSPARENT, EMPTY) => true,

                                            (TRANSPARENT, TRANSPARENT) => voxel != *neighbor,

                                            (_, _) => false,
                                        }
                                    }
                                } else {
                                    (visibility == OPAQUE && solid_pass)
                                        || (visibility == TRANSPARENT && !solid_pass) && !blocked
                                };
                                let origin_one = match i {
                                    0 => cube.origin.1,
                                    1 => cube.origin.1,
                                    2 => cube.origin.0,
                                    3 => cube.origin.0,
                                    4 => cube.origin.0,
                                    5 => cube.origin.0,
                                    _ => 0,
                                };
                                let end_one = match i {
                                    0 => cube.end.1,
                                    1 => cube.end.1,
                                    2 => cube.end.0,
                                    3 => cube.end.0,
                                    4 => cube.end.0,
                                    5 => cube.end.0,
                                    _ => 0,
                                };
                                let origin_two = match i {
                                    0 => cube.origin.2,
                                    1 => cube.origin.2,
                                    2 => cube.origin.2,
                                    3 => cube.origin.2,
                                    4 => cube.origin.1,
                                    5 => cube.origin.1,
                                    _ => 0,
                                };
                                let end_two = match i {
                                    0 => cube.end.2,
                                    1 => cube.end.2,
                                    2 => cube.end.2,
                                    3 => cube.end.2,
                                    4 => cube.end.1,
                                    5 => cube.end.1,
                                    _ => 0,
                                };
                                let self_start = match i {
                                    0 => cube.origin.0,
                                    1 => cube.origin.0,
                                    2 => cube.origin.1,
                                    3 => cube.origin.1,
                                    4 => cube.origin.2,
                                    5 => cube.origin.2,
                                    _ => 0,
                                };
                                let self_end = match i {
                                    0 => cube.end.0,
                                    1 => cube.end.0,
                                    2 => cube.end.1,
                                    3 => cube.end.1,
                                    4 => cube.end.2,
                                    5 => cube.end.2,
                                    _ => 0,
                                };

                                if generate {
                                    buffer.groups[i].push(Quad {
                                        voxel: [x, y, z],
                                        start: (origin_one, origin_two, self_start),
                                        end: (end_one, end_two, self_end),
                                        cube: cube_num,
                                        data: voxel,
                                    });
                                }
                            }
                            //
                        }
                    }
                }
            }
        }
    }
    // buffer
}

fn full_mesh(
    raw_chunk: &ChunkBoundary,
    loadable_assets: &LoadableAssets,
    texture_atlas: &TextureAtlas,
    chunk_pos: IVec3,
) -> MeshedChunk {
    let mut buffer = QuadGroups::default();
    generate_mesh(raw_chunk, true, &mut buffer);
    let mut positions = Vec::new();
    let mut indices = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut ao = Vec::new();
    let mut light = Vec::new();
    for face in buffer.iter_with_ao(raw_chunk) {
        indices.extend_from_slice(&face.indices(positions.len() as u32));
        positions.extend_from_slice(&face.positions(1.0, &raw_chunk)); // Voxel size is 1m
        normals.extend_from_slice(&face.normals());
        ao.extend_from_slice(&face.aos());
        let matched_index = match (face.side.axis, face.side.positive) {
            (Axis::X, false) => 2,
            (Axis::X, true) => 3,
            (Axis::Y, false) => 1,
            (Axis::Y, true) => 0,
            (Axis::Z, false) => 5,
            (Axis::Z, true) => 4,
        };
        let matched_neighbor = match (face.side.axis, face.side.positive) {
            (Axis::X, false) => (face.voxel()[0] - 1, face.voxel()[1], face.voxel()[2]),
            (Axis::X, true) => (face.voxel()[0] + 1, face.voxel()[1], face.voxel()[2]),
            (Axis::Y, false) => (face.voxel()[0], face.voxel()[1] - 1, face.voxel()[2]),
            (Axis::Y, true) => (face.voxel()[0], face.voxel()[1] + 1, face.voxel()[2]),
            (Axis::Z, false) => (face.voxel()[0], face.voxel()[1], face.voxel()[2] - 1),
            (Axis::Z, true) => (face.voxel()[0], face.voxel()[1], face.voxel()[2] + 1),
        };
        let light_val = raw_chunk.voxels()
            [ChunkBoundary::linearize(matched_neighbor.0, matched_neighbor.1, matched_neighbor.2)]
        .light
        .clone();

        light.extend_from_slice(&[light_val, light_val, light_val, light_val]);

        uvs.extend_from_slice(
            &face.uvs(
                false,
                false,
                texture_atlas,
                matched_index,
                loadable_assets,
                world_to_global_voxel(Vec3::new(
                    face.voxel()[0] as f32,
                    face.voxel()[1] as f32,
                    face.voxel()[2] as f32,
                ))
                .as_vec3()
                .as_ivec3(),
                &raw_chunk,
            ),
        );
    }
    let final_ao = ao_convert(ao);
    let mut final_color = Vec::new();
    for (idx, color) in final_ao.iter().enumerate() {
        let light_level = light_to_inten(light[idx]);
        // let light_level_red = light_to_color(light[idx].r);
        // let light_level_green = light_to_color(light[idx].g);
        // let light_level_blue = light_to_color(light[idx].b);
        final_color.extend_from_slice(&[[
            color[0] * light_level,
            color[1] * light_level,
            color[2] * light_level,
            color[3],
        ]]);
    }
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.set_indices(Some(Indices::U32(indices)));
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions.clone());
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, final_color);
    buffer.clear();
    //Transparent Mesh
    generate_mesh(raw_chunk, false, &mut buffer);
    let mut ao = Vec::new();
    let mut positions = Vec::new();
    let mut indices = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    for face in buffer.iter_with_ao(raw_chunk) {
        indices.extend_from_slice(&face.indices(positions.len() as u32));

        positions.extend_from_slice(&face.positions(1.0, &raw_chunk)); // Voxel size is 1m
        normals.extend_from_slice(&face.normals());
        ao.extend_from_slice(&face.aos());
        let matched_index = match (face.side.axis, face.side.positive) {
            (Axis::X, false) => 2,
            (Axis::X, true) => 3,
            (Axis::Y, false) => 1,
            (Axis::Y, true) => 0,
            (Axis::Z, false) => 5,
            (Axis::Z, true) => 4,
        };

        uvs.extend_from_slice(
            &face.uvs(
                false,
                false,
                texture_atlas,
                matched_index,
                loadable_assets,
                world_to_global_voxel(Vec3::new(
                    face.voxel()[0] as f32,
                    face.voxel()[1] as f32,
                    face.voxel()[2] as f32,
                ))
                .as_vec3()
                .as_ivec3(),
                &raw_chunk,
            ),
        );
    }

    let mut transparent_mesh = Mesh::new(PrimitiveTopology::TriangleList);
    transparent_mesh.set_indices(Some(Indices::U32(indices)));
    transparent_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions.clone());
    transparent_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    transparent_mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    MeshedChunk {
        chunk_mesh: mesh,
        transparent_mesh,
        pos: ChunkPos(chunk_pos),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn process_priority_queue(
    mut chunk_queue: ResMut<MeshQueue>,
    mut commands: Commands,
    loadable_assets: ResMut<LoadableAssets>,
    block_table: Res<BlockTable>,
    geo_table: Res<GeometryTable>,
    texture_atlas: Res<Assets<TextureAtlas>>,
    current_chunks: ResMut<CurrentChunks>,
) {
    let task_pool = ComputeTaskPool::get();
    let block_atlas: TextureAtlas = texture_atlas
        .get(&loadable_assets.block_atlas)
        .unwrap()
        .clone();
    for (chunk_pos, center_chunk, neighbors) in chunk_queue.priority.drain(..) {
        let cloned_table: BlockTable = block_table.clone();
        let cloned_geo_table: GeometryTable = geo_table.clone();
        let cloned_assets: LoadableAssets = loadable_assets.clone();
        let clone_atlas: TextureAtlas = block_atlas.clone();

        let task = task_pool.spawn(async move {
            let raw_chunk = ChunkBoundary::new(
                center_chunk,
                neighbors,
                &cloned_table,
                &cloned_geo_table,
                &cloned_assets,
                &clone_atlas,
            );
            full_mesh(&raw_chunk, &cloned_assets, &clone_atlas, chunk_pos)
        });
        // commands
        //     .entity(current_chunks.get_entity(ChunkPos(chunk_pos)).unwrap())
        //     .insert(PriorityComputeMesh(task));
        commands.spawn(PriorityComputeMesh(task));
    }
}

pub fn priority_mesh(
    mut commands: Commands,
    chunks: Query<&ChunkPos, With<PriorityMesh>>,
    chunk_manager: ChunkManager,
    mut chunk_queue: ResMut<MeshQueue>,
) {
    for chunk in chunks.iter() {
        if let Some(neighbors) = chunk_manager.get_neighbors(*chunk) {
            if let Ok(neighbors) = neighbors.try_into() {
                if let Some(chunk_entity) = chunk_manager.current_chunks.get_entity(*chunk) {
                    if let Some(chunk_data) = chunk_manager.get_chunk(chunk_entity) {
                        chunk_queue.priority.push((
                            **chunk,
                            chunk_data,
                            Box::new(Array(neighbors)),
                        ));
                        commands.entity(chunk_entity).remove::<PriorityMesh>();
                        commands.entity(chunk_entity).remove::<NeedsMesh>();
                    }
                }
            }
        }
    }
}

pub fn build_mesh(
    mut commands: Commands,
    mut chunk_queue: ResMut<MeshQueue>,
    chunk_manager: ChunkManager,
    chunks: Query<&ChunkPos, With<NeedsMesh>>,
    player_chunk: Res<PlayerChunk>,
    options: Res<GameOptions>,
) {
    for (count, chunk) in chunks
        .iter()
        .sorted_unstable_by_key(|key| {
            FloatOrd(key.as_vec3().distance(player_chunk.chunk_pos.as_vec3()))
        })
        .enumerate()
    {
        if count > options.meshes_frame {
            return;
        }
        if chunk_manager.current_chunks.all_neighbors_exist(*chunk) {
            if let Some(neighbors) = chunk_manager.get_neighbors(*chunk) {
                if let Ok(neighbors) = neighbors.try_into() {
                    if let Some(chunk_entity) = chunk_manager.current_chunks.get_entity(*chunk) {
                        if let Some(chunk_data) = chunk_manager.get_chunk(chunk_entity) {
                            chunk_queue.mesh.push((
                                **chunk,
                                chunk_data,
                                Box::new(Array(neighbors)),
                            ));
                            commands.entity(chunk_entity).remove::<NeedsMesh>();
                        }
                    }
                }
            }
        }
    }
}

#[derive(Component)]
pub struct MeshedChunk {
    chunk_mesh: Mesh,
    transparent_mesh: Mesh,
    pos: ChunkPos,
}

#[derive(Resource, Default)]
pub struct ChunkMaterial {
    opaque: Handle<StandardMaterial>,
    transparent: Handle<StandardMaterial>,
}

pub fn create_chunk_material(
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut chunk_material: ResMut<ChunkMaterial>,
    texture_atlas: Res<Assets<TextureAtlas>>,
    loadable_assets: ResMut<LoadableAssets>,
) {
    chunk_material.transparent = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: Some(
            texture_atlas
                .get(&loadable_assets.block_atlas)
                .unwrap()
                .texture
                .clone(),
        ),
        alpha_mode: AlphaMode::Blend,
        perceptual_roughness: 0.5,
        ..Default::default() // discard_pix: 0,
    });
    chunk_material.opaque = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: Some(
            texture_atlas
                .get(&loadable_assets.block_atlas)
                .unwrap()
                .texture
                .clone(),
        ),
        alpha_mode: AlphaMode::Mask(0.5),
        perceptual_roughness: 1.0,
        ..Default::default() // discard_pix: 1,
    });
}

pub fn priority_player(
    player_chunk: Res<PlayerChunk>,
    current_chunks: Res<CurrentChunks>,
    chunks: Query<&Handle<Mesh>>,
    mut commands: Commands,
) {
    if let Some(chunk_entity) = current_chunks.get_entity(ChunkPos(player_chunk.chunk_pos)) {
        if chunks.get(chunk_entity).is_err() {
            commands.entity(chunk_entity).insert(PriorityMesh);
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn process_queue(
    mut chunk_queue: ResMut<MeshQueue>,
    mut commands: Commands,
    loadable_assets: ResMut<LoadableAssets>,
    block_table: Res<BlockTable>,
    geo_table: Res<GeometryTable>,
    texture_atlas: Res<Assets<TextureAtlas>>,
) {
    let task_pool = AsyncComputeTaskPool::get();
    let block_atlas: TextureAtlas = texture_atlas
        .get(&loadable_assets.block_atlas)
        .unwrap()
        .clone();
    for (chunk_pos, center_chunk, neighbors) in chunk_queue.mesh.drain(..).rev() {
        let cloned_table: BlockTable = block_table.clone();
        let cloned_geo_table: GeometryTable = geo_table.clone();
        let cloned_assets: LoadableAssets = loadable_assets.clone();
        let clone_atlas: TextureAtlas = block_atlas.clone();

        let task = task_pool.spawn(async move {
            let raw_chunk = ChunkBoundary::new(
                center_chunk,
                neighbors,
                &cloned_table,
                &cloned_geo_table,
                &cloned_assets,
                &clone_atlas,
            );
            full_mesh(&raw_chunk, &cloned_assets, &clone_atlas, chunk_pos)
        });
        commands.spawn(ComputeMesh(task));
    }
}

fn light_to_color(color: u8) -> f32 {
    match color {
        0 => 0.0,
        1 => 0.0,
        2 => 0.1,
        3 => 0.2,
        4 => 0.25,
        5 => 1.0,
        6 => 1.1,
        7 => 1.2,
        8 => 1.3,
        9 => 1.5,
        10 => 2.0,
        11 => 3.0,
        12 => 4.0,
        13 => 4.5,
        14 => 5.0,
        15 => 7.5,
        _ => 10.0,
    }
}
fn light_to_inten(color: u8) -> f32 {
    match color {
        0 => 0.25,
        1 => 0.4,
        2 => 0.5,
        3 => 0.75,
        4 => 0.9,
        5 => 1.0,
        6 => 1.1,
        7 => 1.2,
        8 => 1.3,
        9 => 1.5,
        10 => 2.0,
        11 => 3.0,
        12 => 4.0,
        13 => 4.5,
        14 => 5.0,
        15 => 7.5,
        _ => 10.0,
    }
}

fn ao_convert(ao: Vec<u32>) -> Vec<[f32; 4]> {
    let mut res = Vec::new();
    for value in ao {
        match value {
            0 => res.extend_from_slice(&[[0.1, 0.1, 0.1, 1.0]]),
            1 => res.extend_from_slice(&[[0.25, 0.25, 0.25, 1.0]]),
            2 => res.extend_from_slice(&[[0.5, 0.5, 0.5, 1.0]]),
            _ => res.extend_from_slice(&[[1., 1., 1., 1.0]]),
        }
    }
    res
}

pub struct SortFaces {
    chunk_pos: IVec3,
}

pub fn sort_faces(
    current_chunks: Res<CurrentChunks>,
    handles: Query<&Handle<Mesh>>,
    chunks: Query<&Children, With<ChunkData>>,
    mut meshes: ResMut<Assets<Mesh>>,
    camera_transform: Query<&GlobalTransform, With<Camera>>,
    mut events: EventReader<SortFaces>,
) {
    for evt in events.iter() {
        if let Ok(camera_transform) = camera_transform.get_single() {
            if let Some(chunk_entity) = current_chunks.get_entity(ChunkPos(evt.chunk_pos)) {
                if let Ok(children) = chunks.get(chunk_entity) {
                    if let Some(child_entity) = children.get(0) {
                        if let Ok(chunk_mesh_handle) = handles.get(*child_entity) {
                            if let Some(chunk_mesh) = meshes.get_mut(chunk_mesh_handle) {
                                let mut collected_indices = Vec::new();
                                let mut sorted_indices: Vec<([usize; 6], f32)> = Vec::new();
                                if let Some(vertex_array) =
                                    chunk_mesh.attribute(Mesh::ATTRIBUTE_POSITION)
                                {
                                    if let Some(raw_array) = vertex_array.as_float3() {
                                        if let Some(indices) = chunk_mesh.indices() {
                                            for indice in indices.iter().chunks(6).into_iter() {
                                                let vec_ind: Vec<usize> = indice.collect();
                                                let x = (raw_array[vec_ind[1]][0]
                                                    + raw_array[vec_ind[3]][0]
                                                    + raw_array[vec_ind[4]][0]
                                                    + raw_array[vec_ind[5]][0])
                                                    / 4.0;
                                                let y = (raw_array[vec_ind[1]][1]
                                                    + raw_array[vec_ind[3]][1]
                                                    + raw_array[vec_ind[4]][1]
                                                    + raw_array[vec_ind[5]][1])
                                                    / 4.0;
                                                let z = (raw_array[vec_ind[1]][2]
                                                    + raw_array[vec_ind[3]][2]
                                                    + raw_array[vec_ind[4]][2]
                                                    + raw_array[vec_ind[5]][2])
                                                    / 4.0;
                                                let real_pos = voxel_to_world(
                                                    UVec3::new(x as u32, y as u32, z as u32),
                                                    evt.chunk_pos,
                                                );
                                                let dist = camera_transform
                                                    .translation()
                                                    .distance(real_pos);
                                                sorted_indices.push((
                                                    [
                                                        vec_ind[0], vec_ind[1], vec_ind[2],
                                                        vec_ind[3], vec_ind[4], vec_ind[5],
                                                    ],
                                                    dist,
                                                ));
                                            }
                                            sorted_indices
                                                .sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
                                            sorted_indices.reverse();

                                            // This is horrible most definitely a better way to do this
                                            for indice in sorted_indices.iter() {
                                                collected_indices.push(indice.0[0] as u32);
                                                collected_indices.push(indice.0[1] as u32);
                                                collected_indices.push(indice.0[2] as u32);
                                                collected_indices.push(indice.0[3] as u32);
                                                collected_indices.push(indice.0[4] as u32);
                                                collected_indices.push(indice.0[5] as u32);
                                            }
                                        }
                                    }
                                }

                                chunk_mesh.set_indices(Some(Indices::U32(collected_indices)));
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn sort_chunks(
    player_chunk: Res<PlayerChunk>,
    player_block: Res<PlayerBlock>,
    mut sort_face: EventWriter<SortFaces>,
) {
    if player_chunk.is_changed() {
        sort_face.send(SortFaces {
            chunk_pos: player_chunk.chunk_pos,
        });
        sort_face.send(SortFaces {
            chunk_pos: player_chunk.chunk_pos + IVec3::new(1, 0, 0),
        });
        sort_face.send(SortFaces {
            chunk_pos: player_chunk.chunk_pos + IVec3::new(-1, 0, 0),
        });
        sort_face.send(SortFaces {
            chunk_pos: player_chunk.chunk_pos + IVec3::new(0, 1, 0),
        });
        sort_face.send(SortFaces {
            chunk_pos: player_chunk.chunk_pos + IVec3::new(0, -1, 0),
        });
        sort_face.send(SortFaces {
            chunk_pos: player_chunk.chunk_pos + IVec3::new(0, 0, 1),
        });
        sort_face.send(SortFaces {
            chunk_pos: player_chunk.chunk_pos + IVec3::new(0, 0, -1),
        });
    }

    if player_block.is_changed() {
        sort_face.send(SortFaces {
            chunk_pos: player_chunk.chunk_pos,
        });
    }
}
