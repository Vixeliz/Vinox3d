// use bevy::prelude::{info_span, UVec3};
// use bimap::BiMap;
// use itertools::*;
// use serde_big_array::Array;
// use vinox_common::{
//     storage::blocks::descriptor::BlockDescriptor,
//     world::chunks::storage::{
//         BlockTable, RawChunk, RenderedBlockData, VoxelVisibility, CHUNK_SIZE,
//     },
// };

// const CHUNK_BOUND: u32 = CHUNK_SIZE + 1;
// const TOTAL_CHUNK_SIZE_PADDED: usize =
//     (CHUNK_SIZE_PADDED as usize) * (CHUNK_SIZE_PADDED as usize) * (CHUNK_SIZE_PADDED as usize);
// const CHUNK_SIZE_PADDED: u32 = CHUNK_SIZE + 2;

// #[derive(Clone)]
// pub struct ChunkBoundary {
//     pub palette: BiMap<u16, RenderedBlockData>,
//     pub voxels: Box<[u16; TOTAL_CHUNK_SIZE_PADDED]>,
// }

// impl ChunkBoundary {
//     const X: usize = CHUNK_SIZE_PADDED as usize;
//     const Y: usize = CHUNK_SIZE_PADDED as usize;
//     const Z: usize = CHUNK_SIZE_PADDED as usize;
//     fn get(&self, x: u32, y: u32, z: u32, block_table: &BlockTable) -> Self::Output {
//         self.get_voxel(ChunkBoundary::linearize(UVec3::new(x, y, z)), block_table)
//     }
//     fn get_descriptor(&self, x: u32, y: u32, z: u32, block_table: &BlockTable) -> BlockDescriptor {
//         self.get_data(ChunkBoundary::linearize(UVec3::new(x, y, z)), block_table)
//     }
//     fn get_data(&self, x: u32, y: u32, z: u32) -> RenderedBlockData {
//         self.get_block(UVec3::new(x, y, z)).unwrap_or_default()
//     }
// }

// fn max_block_id(palette: &BiMap<u16, RenderedBlockData>) -> u16 {
//     let mut counter = 0;
//     for id in palette.left_values().sorted() {
//         if *id != 0 && counter < id - 1 {
//             return *id;
//         }
//         counter = *id;
//     }
//     counter + 1
// }

// fn add_block_state(palette: &mut BiMap<u16, RenderedBlockData>, block_data: &RenderedBlockData) {
//     palette.insert(max_block_id(palette), block_data.to_owned());
// }

// impl ChunkBoundary {
//     pub fn new(center: RawChunk, neighbors: Box<Array<RawChunk, 26>>) -> Self {
//         const MAX: u32 = CHUNK_SIZE;
//         // Just cause CHUNK_SIZE is long
//         let voxels: Box<[RenderedBlockData; TOTAL_CHUNK_SIZE_PADDED]> = (0
//             ..TOTAL_CHUNK_SIZE_PADDED)
//             .map(|idx| {
//                 let (x, y, z) = ChunkBoundary::delinearize(idx);
//                 match (x, y, z) {
//                     (0, 0, 0) => neighbors[0].get_rend(MAX - 1, MAX - 1, MAX - 1),
//                     (0, 0, 1..=MAX) => neighbors[1].get_rend(MAX - 1, MAX - 1, z - 1),
//                     (0, 0, CHUNK_BOUND) => neighbors[2].get_rend(MAX - 1, MAX - 1, 0),
//                     (0, 1..=MAX, 0) => neighbors[3].get_rend(MAX - 1, y - 1, MAX - 1),
//                     (0, 1..=MAX, 1..=MAX) => neighbors[4].get_rend(MAX - 1, y - 1, z - 1),
//                     (0, 1..=MAX, CHUNK_BOUND) => neighbors[5].get_rend(MAX - 1, y - 1, 0),
//                     (0, CHUNK_BOUND, 0) => neighbors[6].get_rend(MAX - 1, 0, MAX - 1),
//                     (0, CHUNK_BOUND, 1..=MAX) => neighbors[7].get_rend(MAX - 1, 0, z - 1),
//                     (0, CHUNK_BOUND, CHUNK_BOUND) => neighbors[8].get_rend(MAX - 1, 0, 0),
//                     (1..=MAX, 0, 0) => neighbors[9].get_rend(x - 1, MAX - 1, MAX - 1),
//                     (1..=MAX, 0, 1..=MAX) => neighbors[10].get_rend(x - 1, MAX - 1, z - 1),
//                     (1..=MAX, 0, CHUNK_BOUND) => neighbors[11].get_rend(x - 1, MAX - 1, 0),
//                     (1..=MAX, 1..=MAX, 0) => neighbors[12].get_rend(x - 1, y - 1, MAX - 1),
//                     (1..=MAX, 1..=MAX, 1..=MAX) => center.get_rend(x - 1, y - 1, z - 1),
//                     (1..=MAX, 1..=MAX, CHUNK_BOUND) => neighbors[13].get_rend(x - 1, y - 1, 0),
//                     (1..=MAX, CHUNK_BOUND, 0) => neighbors[14].get_rend(x - 1, 0, MAX - 1),
//                     (1..=MAX, CHUNK_BOUND, 1..=MAX) => neighbors[15].get_rend(x - 1, 0, z - 1),
//                     (1..=MAX, CHUNK_BOUND, CHUNK_BOUND) => neighbors[16].get_rend(x - 1, 0, 0),
//                     (CHUNK_BOUND, 0, 0) => neighbors[17].get_rend(0, MAX - 1, MAX - 1),
//                     (CHUNK_BOUND, 0, 1..=MAX) => neighbors[18].get_rend(0, MAX - 1, z - 1),
//                     (CHUNK_BOUND, 0, CHUNK_BOUND) => neighbors[19].get_rend(0, MAX - 1, 0),
//                     (CHUNK_BOUND, 1..=MAX, 0) => neighbors[20].get_rend(0, y - 1, MAX - 1),
//                     (CHUNK_BOUND, 1..=MAX, 1..=MAX) => neighbors[21].get_rend(0, y - 1, z - 1),
//                     (CHUNK_BOUND, 1..=MAX, CHUNK_BOUND) => neighbors[22].get_rend(0, y - 1, 0),
//                     (CHUNK_BOUND, CHUNK_BOUND, 0) => neighbors[23].get_rend(0, 0, MAX - 1),
//                     (CHUNK_BOUND, CHUNK_BOUND, 1..=MAX) => neighbors[24].get_rend(0, 0, z - 1),
//                     (CHUNK_BOUND, CHUNK_BOUND, CHUNK_BOUND) => neighbors[25].get_rend(0, 0, 0),

//                     (_, _, _) => {
//                         RenderedBlockData::new("vinox".to_string(), "air".to_string(), None, None)
//                     }
//                 }
//             })
//             .collect_vec()
//             .try_into()
//             .unwrap();

//         let mut palette = BiMap::new();

//         palette.insert(
//             0,
//             RenderedBlockData::new("vinox".to_string(), "air".to_string(), None, None),
//         );

//         for idx in 0..TOTAL_CHUNK_SIZE_PADDED {
//             if !palette.contains_right(&voxels[idx]) {
//                 add_block_state(&mut palette, &voxels[idx]);
//             }
//         }

//         let fin_voxels: Box<[u16; TOTAL_CHUNK_SIZE_PADDED]> = (0..TOTAL_CHUNK_SIZE_PADDED)
//             .map(|idx| *palette.get_by_right(&voxels[idx]).unwrap())
//             .collect_vec()
//             .try_into()
//             .unwrap();

//         ChunkBoundary {
//             palette,
//             voxels: fin_voxels,
//         }
//     }

//     pub const fn size() -> usize {
//         TOTAL_CHUNK_SIZE_PADDED
//     }

//     pub fn get_voxel(&self, index: usize, block_table: &BlockTable) -> VoxelType {
//         let block_state = self
//             .get_state_for_index(self.voxels[index] as usize)
//             .unwrap();
//         let block_id = self.get_index_for_state(&block_state).unwrap();
//         if let Some(voxel) = block_table.get(&block_state.identifier) {
//             let voxel_visibility = voxel.visibility;
//             if let Some(voxel_visibility) = voxel_visibility {
//                 match voxel_visibility {
//                     VoxelVisibility::Empty => VoxelType::Empty(block_id),
//                     VoxelVisibility::Opaque => VoxelType::Opaque(block_id),
//                     VoxelVisibility::Transparent => VoxelType::Transparent(block_id),
//                 }
//             } else {
//                 VoxelType::Empty(0)
//             }
//         } else {
//             println!("No name: {block_state:?}");
//             VoxelType::Empty(0)
//         }
//     }

//     pub fn get_data(&self, index: usize, block_table: &BlockTable) -> BlockDescriptor {
//         // let my_span = info_span!("full_mesh", name = "full_mesh").entered();
//         let block_state = self
//             .get_state_for_index(self.voxels[index] as usize)
//             .unwrap();
//         block_table.get(&block_state.identifier).unwrap().clone()
//     }

//     pub fn get_index_for_state(&self, block_data: &RenderedBlockData) -> Option<u16> {
//         self.palette.get_by_right(block_data).copied()
//     }

//     pub fn get_state_for_index(&self, index: usize) -> Option<RenderedBlockData> {
//         self.palette.get_by_left(&(index as u16)).cloned()
//     }

//     pub fn get_block(&self, pos: UVec3) -> Option<RenderedBlockData> {
//         let index = ChunkBoundary::linearize(pos);
//         self.get_state_for_index(self.voxels[index] as usize)
//     }

//     pub fn get_identifier(&self, pos: UVec3) -> String {
//         let index = ChunkBoundary::linearize(pos);
//         if let Some(block) = self.get_state_for_index(self.voxels[index] as usize) {
//             block.identifier
//         } else {
//             "vinox:air".to_string()
//         }
//     }
// }

use ndshape::{ConstShape, ConstShape3usize};
use serde_big_array::Array;
use vinox_common::world::chunks::storage::{BlockTable, ChunkData, RawChunk, RenderedBlockData};

use super::meshing::GeometryTable;

const BOUNDARY_EDGE: usize = ChunkData::edge() + 2;
type BoundaryShape = ConstShape3usize<BOUNDARY_EDGE, BOUNDARY_EDGE, BOUNDARY_EDGE>;

pub struct ChunkBoundary {
    voxels: Box<[RenderedBlockData; BoundaryShape::SIZE]>,
}

#[allow(dead_code)]
impl ChunkBoundary {
    pub fn new(
        center: ChunkData,
        neighbors: Box<Array<ChunkData, 26>>,
        block_table: &BlockTable,
        geo_table: &GeometryTable,
    ) -> Self {
        // Must have 26 neighbors
        // assert!(neighbors.len() == 26);

        const MAX: usize = ChunkData::edge();
        const BOUND: usize = MAX + 1;

        let voxels: Box<[RenderedBlockData; BoundaryShape::SIZE]> = (0..BoundaryShape::SIZE)
            .map(|idx| {
                let [x, y, z] = BoundaryShape::delinearize(idx);
                match (x, y, z) {
                    (0, 0, 0) => get_rend(
                        &neighbors[0],
                        MAX - 1,
                        MAX - 1,
                        MAX - 1,
                        geo_table,
                        block_table,
                    ),
                    (0, 0, 1..=MAX) => get_rend(
                        &neighbors[1],
                        MAX - 1,
                        MAX - 1,
                        z - 1,
                        geo_table,
                        block_table,
                    ),
                    (0, 0, BOUND) => {
                        get_rend(&neighbors[2], MAX - 1, MAX - 1, 0, geo_table, block_table)
                    }
                    (0, 1..=MAX, 0) => get_rend(
                        &neighbors[3],
                        MAX - 1,
                        y - 1,
                        MAX - 1,
                        geo_table,
                        block_table,
                    ),
                    (0, 1..=MAX, 1..=MAX) => {
                        get_rend(&neighbors[4], MAX - 1, y - 1, z - 1, geo_table, block_table)
                    }
                    (0, 1..=MAX, BOUND) => {
                        get_rend(&neighbors[5], MAX - 1, y - 1, 0, geo_table, block_table)
                    }
                    (0, BOUND, 0) => {
                        get_rend(&neighbors[6], MAX - 1, 0, MAX - 1, geo_table, block_table)
                    }
                    (0, BOUND, 1..=MAX) => {
                        get_rend(&neighbors[7], MAX - 1, 0, z - 1, geo_table, block_table)
                    }
                    (0, BOUND, BOUND) => {
                        get_rend(&neighbors[8], MAX - 1, 0, 0, geo_table, block_table)
                    }
                    (1..=MAX, 0, 0) => get_rend(
                        &neighbors[9],
                        x - 1,
                        MAX - 1,
                        MAX - 1,
                        geo_table,
                        block_table,
                    ),
                    (1..=MAX, 0, 1..=MAX) => get_rend(
                        &neighbors[10],
                        x - 1,
                        MAX - 1,
                        z - 1,
                        geo_table,
                        block_table,
                    ),
                    (1..=MAX, 0, BOUND) => {
                        get_rend(&neighbors[11], x - 1, MAX - 1, 0, geo_table, block_table)
                    }
                    (1..=MAX, 1..=MAX, 0) => get_rend(
                        &neighbors[12],
                        x - 1,
                        y - 1,
                        MAX - 1,
                        geo_table,
                        block_table,
                    ),
                    (1..=MAX, 1..=MAX, 1..=MAX) => {
                        get_rend(&center, x - 1, y - 1, z - 1, geo_table, block_table)
                    }
                    (1..=MAX, 1..=MAX, BOUND) => {
                        get_rend(&neighbors[13], x - 1, y - 1, 0, geo_table, block_table)
                    }
                    (1..=MAX, BOUND, 0) => {
                        get_rend(&neighbors[14], x - 1, 0, MAX - 1, geo_table, block_table)
                    }
                    (1..=MAX, BOUND, 1..=MAX) => {
                        get_rend(&neighbors[15], x - 1, 0, z - 1, geo_table, block_table)
                    }
                    (1..=MAX, BOUND, BOUND) => {
                        get_rend(&neighbors[16], x - 1, 0, 0, geo_table, block_table)
                    }
                    (BOUND, 0, 0) => {
                        get_rend(&neighbors[17], 0, MAX - 1, MAX - 1, geo_table, block_table)
                    }
                    (BOUND, 0, 1..=MAX) => {
                        get_rend(&neighbors[18], 0, MAX - 1, z - 1, geo_table, block_table)
                    }
                    (BOUND, 0, BOUND) => {
                        get_rend(&neighbors[19], 0, MAX - 1, 0, geo_table, block_table)
                    }
                    (BOUND, 1..=MAX, 0) => {
                        get_rend(&neighbors[20], 0, y - 1, MAX - 1, geo_table, block_table)
                    }
                    (BOUND, 1..=MAX, 1..=MAX) => {
                        get_rend(&neighbors[21], 0, y - 1, z - 1, geo_table, block_table)
                    }
                    (BOUND, 1..=MAX, BOUND) => {
                        get_rend(&neighbors[22], 0, y - 1, 0, geo_table, block_table)
                    }
                    (BOUND, BOUND, 0) => {
                        get_rend(&neighbors[23], 0, 0, MAX - 1, geo_table, block_table)
                    }
                    (BOUND, BOUND, 1..=MAX) => {
                        get_rend(&neighbors[24], 0, 0, z - 1, geo_table, block_table)
                    }
                    (BOUND, BOUND, BOUND) => {
                        get_rend(&neighbors[25], 0, 0, 0, geo_table, block_table)
                    }

                    (_, _, _) => RenderedBlockData::default(),
                }
            })
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();

        Self { voxels }
    }

    pub fn voxels(&self) -> &[RenderedBlockData; BoundaryShape::USIZE] {
        &self.voxels
    }

    pub const fn edge() -> usize {
        BOUNDARY_EDGE
    }

    pub const fn size() -> usize {
        BoundaryShape::SIZE
    }

    pub fn linearize(x: usize, y: usize, z: usize) -> usize {
        BoundaryShape::linearize([x, y, z])
    }

    pub fn delinearize(idx: usize) -> (usize, usize, usize) {
        let res = BoundaryShape::delinearize(idx);
        (res[0], res[1], res[2])
    }

    pub fn x_offset() -> usize {
        ChunkBoundary::linearize(1, 0, 0) - ChunkBoundary::linearize(0, 0, 0)
    }

    pub fn y_offset() -> usize {
        ChunkBoundary::linearize(0, 1, 0) - ChunkBoundary::linearize(0, 0, 0)
    }

    pub fn z_offset() -> usize {
        ChunkBoundary::linearize(0, 0, 1) - ChunkBoundary::linearize(0, 0, 0)
    }
}

pub fn get_rend(
    chunk: &ChunkData,
    x: usize,
    y: usize,
    z: usize,
    geo_table: &GeometryTable,
    block_table: &BlockTable,
) -> RenderedBlockData {
    let voxel = chunk.get(x, y, z);
    let identifier = chunk.get_identifier(x, y, z);
    let block_data = block_table.get(&identifier).unwrap();
    let geo_data = geo_table.get(
        &block_data
            .clone()
            .geometry
            .unwrap_or_default()
            .get_geo_namespace(),
    );
    let tex_variance = block_data.tex_variance.unwrap_or_default();
    let tex_variance = [
        tex_variance[0].unwrap_or(false),
        tex_variance[1].unwrap_or(false),
        tex_variance[2].unwrap_or(false),
        tex_variance[3].unwrap_or(false),
        tex_variance[4].unwrap_or(false),
        tex_variance[5].unwrap_or(false),
    ];
    RenderedBlockData {
        identifier,
        direction: voxel.direction,
        top: voxel.top,
        geo: geo_data.unwrap().element.clone(),
        visibility: voxel.visibility,
        has_direction: block_data.has_direction.unwrap_or_else(|| false),
        exclusive_direction: block_data.exclusive_direction.unwrap_or_else(|| false),
        tex_variance,
        blocks: geo_data.unwrap().blocks,
    }
}
