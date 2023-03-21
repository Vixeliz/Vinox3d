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
    tasks::{AsyncComputeTaskPool, ComputeTaskPool},
    utils::FloatOrd,
};
use bevy_tweening::{lens::TransformPositionLens, *};
use itertools::Itertools;
// use rand::seq::IteratorRandom;
use serde_big_array::Array;
use std::{collections::HashMap, ops::Deref, time::Duration};
use tokio::sync::mpsc::{Receiver, Sender};
use vinox_common::{
    storage::{blocks::descriptor::BlockGeometry, geometry::descriptor::GeometryDescriptor},
    world::chunks::{
        ecs::{ChunkComp, CurrentChunks},
        positions::voxel_to_world,
        storage::{
            self, name_to_identifier, BlockTable, Chunk, RawChunk, RenderedBlockData, Voxel,
            VoxelVisibility, CHUNK_SIZE,
        },
    },
};

use crate::states::{
    assets::load::LoadableAssets,
    game::world::chunks::{ChunkManager, PlayerBlock, PlayerChunk},
};

use super::chunk::ChunkBoundary;

#[derive(Resource, Clone, Default, Deref, DerefMut)]
pub struct GeometryTable(pub HashMap<String, GeometryDescriptor>);

pub const EMPTY: VoxelVisibility = VoxelVisibility::Empty;
pub const OPAQUE: VoxelVisibility = VoxelVisibility::Opaque;
pub const TRANSPARENT: VoxelVisibility = VoxelVisibility::Transparent;

#[derive(Copy, Clone, Debug)]
pub struct Quad {
    pub voxel: [usize; 3],
    pub start: (i32, i32),
    pub end: (i32, i32),
    pub self_start: i32,
    pub self_end: i32,
    pub cube: usize,
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

    pub fn iter_with_ao<'a, C, V>(
        &'a self,
        chunk: &'a C,
        block_table: &'a BlockTable,
    ) -> impl Iterator<Item = FaceWithAO<'a>>
    where
        C: Chunk<Output = V>,
        V: Voxel,
    {
        self.iter()
            .map(|face| FaceWithAO::new(face, chunk, block_table))
    }
}

pub fn face_aos<C, V>(face: &Face, chunk: &C, block_table: &BlockTable) -> [u32; 4]
where
    C: Chunk<Output = V>,
    V: Voxel,
{
    let [x, y, z] = face.voxel();
    let (x, y, z) = (x as u32, y as u32, z as u32);

    match (face.side.axis, face.side.positive) {
        (Axis::X, false) => side_aos([
            chunk.get(x - 1, y, z + 1, block_table),
            chunk.get(x - 1, y - 1, z + 1, block_table),
            chunk.get(x - 1, y - 1, z, block_table),
            chunk.get(x - 1, y - 1, z - 1, block_table),
            chunk.get(x - 1, y, z - 1, block_table),
            chunk.get(x - 1, y + 1, z - 1, block_table),
            chunk.get(x - 1, y + 1, z, block_table),
            chunk.get(x - 1, y + 1, z + 1, block_table),
        ]),
        (Axis::X, true) => side_aos([
            chunk.get(x + 1, y, z - 1, block_table),
            chunk.get(x + 1, y - 1, z - 1, block_table),
            chunk.get(x + 1, y - 1, z, block_table),
            chunk.get(x + 1, y - 1, z + 1, block_table),
            chunk.get(x + 1, y, z + 1, block_table),
            chunk.get(x + 1, y + 1, z + 1, block_table),
            chunk.get(x + 1, y + 1, z, block_table),
            chunk.get(x + 1, y + 1, z - 1, block_table),
        ]),
        (Axis::Y, false) => side_aos([
            chunk.get(x - 1, y - 1, z, block_table),
            chunk.get(x - 1, y - 1, z + 1, block_table),
            chunk.get(x, y - 1, z + 1, block_table),
            chunk.get(x + 1, y - 1, z + 1, block_table),
            chunk.get(x + 1, y - 1, z, block_table),
            chunk.get(x + 1, y - 1, z - 1, block_table),
            chunk.get(x, y - 1, z - 1, block_table),
            chunk.get(x - 1, y - 1, z - 1, block_table),
        ]),
        (Axis::Y, true) => side_aos([
            chunk.get(x, y + 1, z + 1, block_table),
            chunk.get(x - 1, y + 1, z + 1, block_table),
            chunk.get(x - 1, y + 1, z, block_table),
            chunk.get(x - 1, y + 1, z - 1, block_table),
            chunk.get(x, y + 1, z - 1, block_table),
            chunk.get(x + 1, y + 1, z - 1, block_table),
            chunk.get(x + 1, y + 1, z, block_table),
            chunk.get(x + 1, y + 1, z + 1, block_table),
        ]),
        (Axis::Z, false) => side_aos([
            chunk.get(x - 1, y, z - 1, block_table),
            chunk.get(x - 1, y - 1, z - 1, block_table),
            chunk.get(x, y - 1, z - 1, block_table),
            chunk.get(x + 1, y - 1, z - 1, block_table),
            chunk.get(x + 1, y, z - 1, block_table),
            chunk.get(x + 1, y + 1, z - 1, block_table),
            chunk.get(x, y + 1, z - 1, block_table),
            chunk.get(x - 1, y + 1, z - 1, block_table),
        ]),
        (Axis::Z, true) => side_aos([
            chunk.get(x + 1, y, z + 1, block_table),
            chunk.get(x + 1, y - 1, z + 1, block_table),
            chunk.get(x, y - 1, z + 1, block_table),
            chunk.get(x - 1, y - 1, z + 1, block_table),
            chunk.get(x - 1, y, z + 1, block_table),
            chunk.get(x - 1, y + 1, z + 1, block_table),
            chunk.get(x, y + 1, z + 1, block_table),
            chunk.get(x + 1, y + 1, z + 1, block_table),
        ]),
    }
}

pub struct FaceWithAO<'a> {
    face: Face<'a>,
    aos: [u32; 4],
}

impl<'a> FaceWithAO<'a> {
    pub fn new<C, V>(face: Face<'a>, chunk: &C, block_table: &BlockTable) -> Self
    where
        C: Chunk<Output = V>,
        V: Voxel,
    {
        let aos = face_aos(&face, chunk, block_table);
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

pub(crate) fn side_aos<V: Voxel>(neighbors: [V; 8]) -> [u32; 4] {
    let ns = [
        neighbors[0].visibility() == OPAQUE,
        neighbors[1].visibility() == OPAQUE,
        neighbors[2].visibility() == OPAQUE,
        neighbors[3].visibility() == OPAQUE,
        neighbors[4].visibility() == OPAQUE,
        neighbors[5].visibility() == OPAQUE,
        neighbors[6].visibility() == OPAQUE,
        neighbors[7].visibility() == OPAQUE,
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
        geo: &GeometryDescriptor,
        direction: Option<storage::Direction>,
        top: Option<bool>,
    ) -> [[f32; 3]; 4] {
        let (min_one, min_two, max_one, max_two, min_self, max_self) = (
            (self.quad.start.0 as f32 / 16.0),
            (self.quad.start.1 as f32 / 16.0),
            (self.quad.end.0 as f32 / 16.0),
            (self.quad.end.1 as f32 / 16.0),
            (self.quad.self_start as f32 / 16.0),
            (self.quad.self_end as f32 / 16.0),
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
        let cube_pivot = geo.element.cubes.get(self.quad.cube).unwrap().pivot;
        let cube_rotation = geo.element.cubes.get(self.quad.cube).unwrap().rotation;
        let block_pivot = geo.element.pivot;
        let block_rotation = geo.element.rotation;
        if (cube_rotation != (0, 0, 0) || block_rotation != (0, 0, 0))
            && direction.is_none()
            && top.is_none()
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
                if let Some(direction) = direction.clone() {
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
                if let Some(top) = top.clone() {
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
        geo: &GeometryDescriptor,
        texture_atlas: &TextureAtlas,
        matched_ind: usize,
        loadable_assets: &LoadableAssets,
        block: &RenderedBlockData,
    ) -> [[f32; 2]; 4] {
        if let Some(texture_index) = texture_atlas.get_texture_index(
            &loadable_assets
                .block_textures
                .get(&block.identifier)
                .unwrap()[matched_ind],
        ) {
            let uv = geo.element.cubes.get(self.quad.cube).unwrap().uv;
            let mut face_tex = [[0.0; 2]; 4];
            let min_x = texture_atlas.textures.get(texture_index).unwrap().min.x;
            let min_y = texture_atlas.textures.get(texture_index).unwrap().min.y;
            let (min_x, min_y) = match (&self.side.axis, &self.side.positive) {
                (Axis::X, false) => (
                    min_x + uv.get(0).unwrap().0 .0 as f32,
                    min_y + uv.get(0).unwrap().0 .1 as f32,
                ),
                (Axis::X, true) => (
                    min_x + uv.get(1).unwrap().0 .0 as f32,
                    min_y + uv.get(1).unwrap().0 .1 as f32,
                ),
                (Axis::Y, false) => (
                    min_x + uv.get(2).unwrap().0 .0 as f32,
                    min_y + uv.get(2).unwrap().0 .1 as f32,
                ),
                (Axis::Y, true) => (
                    min_x + uv.get(3).unwrap().0 .0 as f32,
                    min_y + uv.get(3).unwrap().0 .1 as f32,
                ),
                (Axis::Z, false) => (
                    min_x + uv.get(4).unwrap().0 .0 as f32,
                    min_y + uv.get(4).unwrap().0 .1 as f32,
                ),
                (Axis::Z, true) => (
                    min_x + uv.get(5).unwrap().0 .0 as f32,
                    min_y + uv.get(5).unwrap().0 .1 as f32,
                ),
            };
            let (max_x, max_y) = match (&self.side.axis, &self.side.positive) {
                (Axis::X, false) => (
                    min_x + uv.get(0).unwrap().1 .0 as f32,
                    min_y + uv.get(0).unwrap().1 .1 as f32,
                ),
                (Axis::X, true) => (
                    min_x + uv.get(1).unwrap().1 .0 as f32,
                    min_y + uv.get(1).unwrap().1 .1 as f32,
                ),
                (Axis::Y, false) => (
                    min_x + uv.get(2).unwrap().1 .0 as f32,
                    min_y + uv.get(2).unwrap().1 .1 as f32,
                ),
                (Axis::Y, true) => (
                    min_x + uv.get(3).unwrap().1 .0 as f32,
                    min_y + uv.get(3).unwrap().1 .1 as f32,
                ),
                (Axis::Z, false) => (
                    min_x + uv.get(4).unwrap().1 .0 as f32,
                    min_y + uv.get(4).unwrap().1 .1 as f32,
                ),
                (Axis::Z, true) => (
                    min_x + uv.get(5).unwrap().1 .0 as f32,
                    min_y + uv.get(5).unwrap().1 .1 as f32,
                ),
            };
            let (min_x, min_y, max_x, max_y) = (
                min_x / texture_atlas.size.x,
                min_y / texture_atlas.size.y,
                max_x / texture_atlas.size.x,
                max_y / texture_atlas.size.y,
            );
            face_tex[2][0] = min_x;
            face_tex[2][1] = min_y;
            face_tex[3][0] = max_x;
            face_tex[3][1] = min_y;
            face_tex[0][0] = min_x;
            face_tex[0][1] = max_y;
            face_tex[1][0] = max_x;
            face_tex[1][1] = max_y;
            return face_tex;
        }
        match (flip_u, flip_v) {
            (true, true) => [[1.0, 1.0], [0.0, 1.0], [1.0, 0.0], [0.0, 0.0]],
            (true, false) => [[1.0, 0.0], [0.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
            (false, true) => [[0.0, 1.0], [1.0, 1.0], [0.0, 0.0], [1.0, 0.0]],
            (false, false) => [[0.0, 0.0], [1.0, 0.0], [0.0, 1.0], [1.0, 1.0]],
        }
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
    pub mesh: MaterialMeshBundle<BasicMaterial>,
    pub aabb: Aabb,
}

#[derive(Default, Resource)]
pub struct MeshQueue {
    pub mesh: Vec<(IVec3, RawChunk, Box<Array<RawChunk, 26>>)>,
    pub priority: Vec<(IVec3, RawChunk, Box<Array<RawChunk, 26>>)>,
}

#[derive(Component, Default)]
pub struct NeedsMesh;

#[derive(Component, Default)]
pub struct PriorityMesh;

#[derive(Resource)]
pub struct PriorityMeshChannel {
    pub tx: Sender<MeshedChunk>,
    pub rx: Receiver<MeshedChunk>,
}

impl Default for PriorityMeshChannel {
    fn default() -> Self {
        let (tx, rx) = tokio::sync::mpsc::channel(256);
        Self { tx, rx }
    }
}

#[derive(Resource)]
pub struct MeshChannel {
    pub tx: Sender<MeshedChunk>,
    pub rx: Receiver<MeshedChunk>,
}

impl Default for MeshChannel {
    fn default() -> Self {
        let (tx, rx) = tokio::sync::mpsc::channel(512);
        Self { tx, rx }
    }
}

// Possibly have this just fully generate the mesh
pub fn generate_mesh<C, T>(
    chunk: &C,
    block_table: &BlockTable,
    solid_pass: bool,
    geometry_table: &GeometryTable,
) -> QuadGroups
where
    C: Chunk<Output = T>,
    T: Voxel,
{
    assert!(C::X >= 2);
    assert!(C::Y >= 2);
    assert!(C::Z >= 2);

    let mut buffer = QuadGroups::default();

    for z in 1..C::Z - 1 {
        for y in 1..C::Y - 1 {
            for x in 1..C::X - 1 {
                let (x, y, z) = (x as u32, y as u32, z as u32);
                let voxel = chunk.get(x, y, z, block_table);
                match voxel.visibility() {
                    EMPTY => continue,
                    visibility => {
                        let neighbors = [
                            chunk.get(x - 1, y, z, block_table),
                            chunk.get(x + 1, y, z, block_table),
                            chunk.get(x, y - 1, z, block_table),
                            chunk.get(x, y + 1, z, block_table),
                            chunk.get(x, y, z - 1, block_table),
                            chunk.get(x, y, z + 1, block_table),
                        ];
                        let neighbor_block = [
                            chunk.get_descriptor(x - 1, y, z, block_table),
                            chunk.get_descriptor(x + 1, y, z, block_table),
                            chunk.get_descriptor(x, y - 1, z, block_table),
                            chunk.get_descriptor(x, y + 1, z, block_table),
                            chunk.get_descriptor(x, y, z - 1, block_table),
                            chunk.get_descriptor(x, y, z + 1, block_table),
                        ];
                        // let mut element_vertices = Vec::new();
                        // let mut element_indices = Vec::new();
                        let voxel_descriptor = chunk.get_descriptor(x, y, z, block_table);
                        // let voxel_data = chunk.get_data(x, y, z);
                        if let Some(geometry) = geometry_table.get(
                            &voxel_descriptor
                                .clone()
                                .geometry
                                .unwrap_or_default()
                                .get_geo_namespace(),
                        ) {
                            let element = geometry.element.clone();
                            for (cube_num, cube) in element.cubes.into_iter().enumerate() {
                                for (i, neighbor) in neighbors.iter().enumerate() {
                                    let neighbor_geometry = geometry_table
                                        .get(
                                            &neighbor_block[i]
                                                .clone()
                                                .geometry
                                                .unwrap_or_default()
                                                .get_geo_namespace(),
                                        )
                                        .unwrap_or(geometry_table.get("vinox:block").unwrap());
                                    let culled = cube.cull[i];
                                    if cube.discard[i] {
                                        continue;
                                    }
                                    let blocked = match i {
                                        0 => neighbor_geometry.blocks[1],
                                        1 => neighbor_geometry.blocks[0],
                                        2 => neighbor_geometry.blocks[3],
                                        3 => neighbor_geometry.blocks[2],
                                        4 => neighbor_geometry.blocks[5],
                                        5 => neighbor_geometry.blocks[4],
                                        _ => true,
                                    };
                                    let other = neighbor.visibility();
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
                                    } else if (visibility == OPAQUE && solid_pass)
                                        || (visibility == TRANSPARENT && !solid_pass) && !blocked
                                    {
                                        true
                                    } else {
                                        false
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
                                            voxel: [x as usize, y as usize, z as usize],
                                            start: (origin_one, origin_two),
                                            end: (end_one, end_two),
                                            self_start,
                                            self_end,
                                            cube: cube_num,
                                        });
                                    }
                                }
                            }
                            //
                        }
                    }
                }
            }
        }
    }
    buffer
}

fn full_mesh(
    raw_chunk: &ChunkBoundary,
    block_table: &BlockTable,
    geo_table: &GeometryTable,
    loadable_assets: &LoadableAssets,
    texture_atlas: &TextureAtlas,
    chunk_pos: IVec3,
) -> MeshedChunk {
    let mesh_result = generate_mesh(raw_chunk, block_table, true, geo_table);
    let mut positions = Vec::new();
    let mut indices = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut ao = Vec::new();
    for face in mesh_result.iter_with_ao(raw_chunk, block_table) {
        indices.extend_from_slice(&face.indices(positions.len() as u32));
        let geo = geo_table
            .get(
                &raw_chunk
                    .get_descriptor(
                        face.voxel()[0] as u32,
                        face.voxel()[1] as u32,
                        face.voxel()[2] as u32,
                        block_table,
                    )
                    .clone()
                    .geometry
                    .unwrap_or_default()
                    .get_geo_namespace(),
            )
            .unwrap_or(geo_table.get("vinox:block").unwrap());

        let vox_data = raw_chunk
            .get_block(UVec3::new(
                face.voxel()[0] as u32,
                face.voxel()[1] as u32,
                face.voxel()[2] as u32,
            ))
            .unwrap();

        positions.extend_from_slice(&face.positions(1.0, geo, vox_data.direction, vox_data.top)); // Voxel size is 1m
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
        let block = &raw_chunk
            .get_block(UVec3::new(
                face.voxel()[0] as u32,
                face.voxel()[1] as u32,
                face.voxel()[2] as u32,
            ))
            .unwrap();

        uvs.extend_from_slice(&face.uvs(
            false,
            false,
            geo,
            texture_atlas,
            matched_index,
            loadable_assets,
            block,
        ));
    }
    let final_ao = ao_convert(ao);
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.set_indices(Some(Indices::U32(indices)));
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions.clone());
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, final_ao);

    //Transparent Mesh
    let mesh_result = generate_mesh(raw_chunk, block_table, false, geo_table);
    let mut ao = Vec::new();
    let mut positions = Vec::new();
    let mut indices = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    for face in mesh_result.iter_with_ao(raw_chunk, block_table) {
        indices.extend_from_slice(&face.indices(positions.len() as u32));
        let geo = geo_table
            .get(
                &raw_chunk
                    .get_descriptor(
                        face.voxel()[0] as u32,
                        face.voxel()[1] as u32,
                        face.voxel()[2] as u32,
                        block_table,
                    )
                    .clone()
                    .geometry
                    .unwrap_or_default()
                    .get_geo_namespace(),
            )
            .unwrap_or(geo_table.get("vinox:block").unwrap());
        let vox_data = raw_chunk
            .get_block(UVec3::new(
                face.voxel()[0] as u32,
                face.voxel()[1] as u32,
                face.voxel()[2] as u32,
            ))
            .unwrap();

        positions.extend_from_slice(&face.positions(1.0, geo, vox_data.direction, vox_data.top)); // Voxel size is 1m
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

        let block = &raw_chunk
            .get_block(UVec3::new(
                face.voxel()[0] as u32,
                face.voxel()[1] as u32,
                face.voxel()[2] as u32,
            ))
            .unwrap();

        uvs.extend_from_slice(&face.uvs(
            false,
            false,
            geo,
            texture_atlas,
            matched_index,
            loadable_assets,
            block,
        ));
    }

    let mut transparent_mesh = Mesh::new(PrimitiveTopology::TriangleList);
    transparent_mesh.set_indices(Some(Indices::U32(indices)));
    transparent_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions.clone());
    transparent_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    transparent_mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    MeshedChunk {
        chunk_mesh: mesh,
        transparent_mesh,
        pos: chunk_pos,
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
    mut priority_channel: ResMut<PriorityMeshChannel>,
    mut meshes: ResMut<Assets<Mesh>>,
    chunk_material: Res<ChunkMaterial>,
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
        let cloned_sender = priority_channel.tx.clone();

        task_pool
            .spawn(async move {
                let raw_chunk = ChunkBoundary::new(center_chunk, neighbors);
                cloned_sender
                    .send(full_mesh(
                        &raw_chunk,
                        &cloned_table,
                        &cloned_geo_table,
                        &cloned_assets,
                        &clone_atlas,
                        chunk_pos,
                    ))
                    .await
                    .ok();
            })
            .detach() // TODO: Switch to polling so we can cancel task outside of view distance or if we break or place a block
    }

    while let Ok(chunk) = priority_channel.rx.try_recv() {
        if let Some(chunk_entity) = current_chunks.get_entity(chunk.pos) {
            commands.entity(chunk_entity).despawn_descendants();

            let chunk_pos = Vec3::new(
                (chunk.pos[0] * (CHUNK_SIZE) as i32) as f32,
                (chunk.pos[1] * (CHUNK_SIZE) as i32) as f32,
                (chunk.pos[2] * (CHUNK_SIZE) as i32) as f32,
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
        } else {
        }
    }
}

pub fn priority_mesh(
    mut commands: Commands,
    chunks: Query<&ChunkComp, With<PriorityMesh>>,
    chunk_manager: ChunkManager,
    mut chunk_queue: ResMut<MeshQueue>,
) {
    for chunk in chunks.iter() {
        if let Some(neighbors) = chunk_manager.get_neighbors(chunk.pos.clone()) {
            if let Ok(neighbors) = neighbors.try_into() {
                chunk_queue.priority.push((
                    *chunk.pos,
                    chunk.chunk_data.clone(),
                    Box::new(Array(neighbors)),
                ));
                commands
                    .entity(chunk_manager.current_chunks.get_entity(*chunk.pos).unwrap())
                    .remove::<PriorityMesh>();
            }
        }
    }
}

pub fn build_mesh(
    mut commands: Commands,
    mut chunk_queue: ResMut<MeshQueue>,
    chunks: Query<&ChunkComp, With<NeedsMesh>>,
    chunk_manager: ChunkManager,
    player_chunk: Res<PlayerChunk>,
) {
    // let mut rng = rand::thread_rng();
    for (count, chunk) in chunks
        .iter()
        .sorted_unstable_by_key(|key| {
            FloatOrd(key.pos.as_vec3().distance(player_chunk.chunk_pos.as_vec3()))
        })
        .enumerate()
    {
        if count > 256 {
            return;
        }
        // for chunk in chunks.iter().choose_multiple(&mut rng, 256) {
        if chunk_manager
            .current_chunks
            .all_neighbors_exist(chunk.pos.clone())
        {
            if let Some(neighbors) = chunk_manager.get_neighbors(chunk.pos.clone()) {
                if let Ok(neighbors) = neighbors.try_into() {
                    chunk_queue.mesh.push((
                        *chunk.pos,
                        chunk.chunk_data.clone(),
                        Box::new(Array(neighbors)),
                    ));

                    commands
                        .entity(chunk_manager.current_chunks.get_entity(*chunk.pos).unwrap())
                        .remove::<NeedsMesh>();
                }
            }
        }
    }
}

#[derive(Component)]
pub struct MeshedChunk {
    chunk_mesh: Mesh,
    transparent_mesh: Mesh,
    pos: IVec3,
}

#[derive(Resource, Default)]
pub struct ChunkMaterial {
    opaque: Handle<BasicMaterial>,
    transparent: Handle<BasicMaterial>,
}

pub fn create_chunk_material(
    mut materials: ResMut<Assets<BasicMaterial>>,
    mut chunk_material: ResMut<ChunkMaterial>,
    texture_atlas: Res<Assets<TextureAtlas>>,
    loadable_assets: ResMut<LoadableAssets>,
) {
    chunk_material.transparent = materials.add(BasicMaterial {
        color: Color::WHITE,
        color_texture: Some(
            texture_atlas
                .get(&loadable_assets.block_atlas)
                .unwrap()
                .texture
                .clone(),
        ),
        alpha_mode: AlphaMode::Blend,
        discard_pix: 0,
    });
    chunk_material.opaque = materials.add(BasicMaterial {
        color: Color::WHITE,
        color_texture: Some(
            texture_atlas
                .get(&loadable_assets.block_atlas)
                .unwrap()
                .texture
                .clone(),
        ),
        alpha_mode: AlphaMode::Opaque,
        discard_pix: 1,
    });
}
#[allow(clippy::too_many_arguments)]
pub fn process_queue(
    mut chunk_queue: ResMut<MeshQueue>,
    mut commands: Commands,
    loadable_assets: ResMut<LoadableAssets>,
    block_table: Res<BlockTable>,
    geo_table: Res<GeometryTable>,
    texture_atlas: Res<Assets<TextureAtlas>>,
    mut mesh_channel: ResMut<MeshChannel>,
    mut meshes: ResMut<Assets<Mesh>>,
    chunk_material: Res<ChunkMaterial>,
    current_chunks: ResMut<CurrentChunks>,
    chunks: Query<&Handle<Mesh>>,
    player_chunk: Res<PlayerChunk>,
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
        let cloned_sender = mesh_channel.tx.clone();

        task_pool
            .spawn(async move {
                let raw_chunk = ChunkBoundary::new(center_chunk, neighbors);
                cloned_sender
                    .send(full_mesh(
                        &raw_chunk,
                        &cloned_table,
                        &cloned_geo_table,
                        &cloned_assets,
                        &clone_atlas,
                        chunk_pos,
                    ))
                    .await
                    .ok();
            })
            .detach()
    }

    while let Ok(chunk) = mesh_channel.rx.try_recv() {
        if let Some(chunk_entity) = current_chunks.get_entity(chunk.pos) {
            commands.entity(chunk_entity).despawn_descendants();
            let tween = Tween::new(
                EaseFunction::QuadraticInOut,
                Duration::from_secs(1),
                TransformPositionLens {
                    start: Vec3::new(
                        (chunk.pos[0] * (CHUNK_SIZE) as i32) as f32,
                        ((chunk.pos[1] * (CHUNK_SIZE) as i32) as f32) - CHUNK_SIZE as f32,
                        (chunk.pos[2] * (CHUNK_SIZE) as i32) as f32,
                    ),

                    end: Vec3::new(
                        (chunk.pos[0] * (CHUNK_SIZE) as i32) as f32,
                        (chunk.pos[1] * (CHUNK_SIZE) as i32) as f32,
                        (chunk.pos[2] * (CHUNK_SIZE) as i32) as f32,
                    ),
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
                Vec3::new(
                    (chunk.pos[0] * (CHUNK_SIZE) as i32) as f32,
                    ((chunk.pos[1] * (CHUNK_SIZE) as i32) as f32) - CHUNK_SIZE as f32,
                    (chunk.pos[2] * (CHUNK_SIZE) as i32) as f32,
                )
            } else {
                Vec3::new(
                    (chunk.pos[0] * (CHUNK_SIZE) as i32) as f32,
                    (chunk.pos[1] * (CHUNK_SIZE) as i32) as f32,
                    (chunk.pos[2] * (CHUNK_SIZE) as i32) as f32,
                )
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
        } else {
        }
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
    chunks: Query<&Children, With<ChunkComp>>,
    mut meshes: ResMut<Assets<Mesh>>,
    camera_transform: Query<&GlobalTransform, With<Camera>>,
    mut events: EventReader<SortFaces>,
) {
    for evt in events.iter() {
        if let Ok(camera_transform) = camera_transform.get_single() {
            if let Some(chunk_entity) = current_chunks.get_entity(evt.chunk_pos) {
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
