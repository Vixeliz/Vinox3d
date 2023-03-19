use bevy::prelude::UVec3;
use bimap::BiMap;
use itertools::*;
use serde_big_array::Array;
use vinox_common::{
    storage::blocks::descriptor::BlockDescriptor,
    world::chunks::storage::{
        BlockTable, Chunk, RawChunk, RenderedBlockData, VoxelType, VoxelVisibility, CHUNK_SIZE,
    },
};

const CHUNK_BOUND: u32 = CHUNK_SIZE + 1;
const TOTAL_CHUNK_SIZE_PADDED: usize =
    (CHUNK_SIZE_PADDED as usize) * (CHUNK_SIZE_PADDED as usize) * (CHUNK_SIZE_PADDED as usize);
const CHUNK_SIZE_PADDED: u32 = CHUNK_SIZE + 2;

#[derive(Clone)]
pub struct ChunkBoundary {
    pub palette: BiMap<u16, RenderedBlockData>,
    pub voxels: Box<[u16; TOTAL_CHUNK_SIZE_PADDED]>,
}

impl Chunk for ChunkBoundary {
    type Output = VoxelType;

    const X: usize = CHUNK_SIZE_PADDED as usize;
    const Y: usize = CHUNK_SIZE_PADDED as usize;
    const Z: usize = CHUNK_SIZE_PADDED as usize;
    fn get(&self, x: u32, y: u32, z: u32, block_table: &BlockTable) -> Self::Output {
        self.get_voxel(ChunkBoundary::linearize(UVec3::new(x, y, z)), block_table)
    }
    fn get_descriptor(&self, x: u32, y: u32, z: u32, block_table: &BlockTable) -> BlockDescriptor {
        self.get_data(ChunkBoundary::linearize(UVec3::new(x, y, z)), block_table)
    }
    fn get_data(&self, x: u32, y: u32, z: u32) -> RenderedBlockData {
        self.get_block(UVec3::new(x, y, z)).unwrap_or_default()
    }
}

fn max_block_id(palette: &BiMap<u16, RenderedBlockData>) -> u16 {
    let mut counter = 0;
    for id in palette.left_values().sorted() {
        if *id != 0 && counter < id - 1 {
            return *id;
        }
        counter = *id;
    }
    counter + 1
}

fn add_block_state(palette: &mut BiMap<u16, RenderedBlockData>, block_data: &RenderedBlockData) {
    palette.insert(max_block_id(palette), block_data.to_owned());
}

impl ChunkBoundary {
    pub fn new(center: RawChunk, neighbors: Box<Array<RawChunk, 26>>) -> Self {
        const MAX: u32 = CHUNK_SIZE;
        // Just cause CHUNK_SIZE is long
        let voxels: Box<[RenderedBlockData; TOTAL_CHUNK_SIZE_PADDED]> =
            Box::new(std::array::from_fn(|idx| {
                let (x, y, z) = ChunkBoundary::delinearize(idx);
                match (x, y, z) {
                    (0, 0, 0) => neighbors[0].get_rend(MAX - 1, MAX - 1, MAX - 1),
                    (0, 0, 1..=MAX) => neighbors[1].get_rend(MAX - 1, MAX - 1, z - 1),
                    (0, 0, CHUNK_BOUND) => neighbors[2].get_rend(MAX - 1, MAX - 1, 0),
                    (0, 1..=MAX, 0) => neighbors[3].get_rend(MAX - 1, y - 1, MAX - 1),
                    (0, 1..=MAX, 1..=MAX) => neighbors[4].get_rend(MAX - 1, y - 1, z - 1),
                    (0, 1..=MAX, CHUNK_BOUND) => neighbors[5].get_rend(MAX - 1, y - 1, 0),
                    (0, CHUNK_BOUND, 0) => neighbors[6].get_rend(MAX - 1, 0, MAX - 1),
                    (0, CHUNK_BOUND, 1..=MAX) => neighbors[7].get_rend(MAX - 1, 0, z - 1),
                    (0, CHUNK_BOUND, CHUNK_BOUND) => neighbors[8].get_rend(MAX - 1, 0, 0),
                    (1..=MAX, 0, 0) => neighbors[9].get_rend(x - 1, MAX - 1, MAX - 1),
                    (1..=MAX, 0, 1..=MAX) => neighbors[10].get_rend(x - 1, MAX - 1, z - 1),
                    (1..=MAX, 0, CHUNK_BOUND) => neighbors[11].get_rend(x - 1, MAX - 1, 0),
                    (1..=MAX, 1..=MAX, 0) => neighbors[12].get_rend(x - 1, y - 1, MAX - 1),
                    (1..=MAX, 1..=MAX, 1..=MAX) => center.get_rend(x - 1, y - 1, z - 1),
                    (1..=MAX, 1..=MAX, CHUNK_BOUND) => neighbors[13].get_rend(x - 1, y - 1, 0),
                    (1..=MAX, CHUNK_BOUND, 0) => neighbors[14].get_rend(x - 1, 0, MAX - 1),
                    (1..=MAX, CHUNK_BOUND, 1..=MAX) => neighbors[15].get_rend(x - 1, 0, z - 1),
                    (1..=MAX, CHUNK_BOUND, CHUNK_BOUND) => neighbors[16].get_rend(x - 1, 0, 0),
                    (CHUNK_BOUND, 0, 0) => neighbors[17].get_rend(0, MAX - 1, MAX - 1),
                    (CHUNK_BOUND, 0, 1..=MAX) => neighbors[18].get_rend(0, MAX - 1, z - 1),
                    (CHUNK_BOUND, 0, CHUNK_BOUND) => neighbors[19].get_rend(0, MAX - 1, 0),
                    (CHUNK_BOUND, 1..=MAX, 0) => neighbors[20].get_rend(0, y - 1, MAX - 1),
                    (CHUNK_BOUND, 1..=MAX, 1..=MAX) => neighbors[21].get_rend(0, y - 1, z - 1),
                    (CHUNK_BOUND, 1..=MAX, CHUNK_BOUND) => neighbors[22].get_rend(0, y - 1, 0),
                    (CHUNK_BOUND, CHUNK_BOUND, 0) => neighbors[23].get_rend(0, 0, MAX - 1),
                    (CHUNK_BOUND, CHUNK_BOUND, 1..=MAX) => neighbors[24].get_rend(0, 0, z - 1),
                    (CHUNK_BOUND, CHUNK_BOUND, CHUNK_BOUND) => neighbors[25].get_rend(0, 0, 0),

                    (_, _, _) => RenderedBlockData::new(
                        "vinox".to_string(),
                        "air".to_string(),
                        None,
                        None,
                        None,
                    ),
                }
            }));

        let mut palette = BiMap::new();

        palette.insert(
            0,
            RenderedBlockData::new("vinox".to_string(), "air".to_string(), None, None, None),
        );

        for idx in 0..TOTAL_CHUNK_SIZE_PADDED {
            if !palette.contains_right(&voxels[idx]) {
                add_block_state(&mut palette, &voxels[idx]);
            }
        }
        let fin_voxels: Box<[u16; TOTAL_CHUNK_SIZE_PADDED]> =
            Box::new(std::array::from_fn(|idx| {
                *palette.get_by_right(&voxels[idx]).unwrap()
            }));

        ChunkBoundary {
            palette,
            voxels: fin_voxels,
        }
    }

    pub const fn size() -> usize {
        TOTAL_CHUNK_SIZE_PADDED
    }

    pub fn get_voxel(&self, index: usize, block_table: &BlockTable) -> VoxelType {
        let block_state = self
            .get_state_for_index(self.voxels[index] as usize)
            .unwrap();
        let block_id = self.get_index_for_state(&block_state).unwrap();
        if let Some(voxel) = block_table.get(&block_state.identifier) {
            let voxel_visibility = voxel.visibility;
            if let Some(voxel_visibility) = voxel_visibility {
                match voxel_visibility {
                    VoxelVisibility::Empty => VoxelType::Empty(block_id),
                    VoxelVisibility::Opaque => VoxelType::Opaque(block_id),
                    VoxelVisibility::Transparent => VoxelType::Transparent(block_id),
                }
            } else {
                VoxelType::Empty(0)
            }
        } else {
            println!("No name: {block_state:?}");
            VoxelType::Empty(0)
        }
    }

    pub fn get_data(&self, index: usize, block_table: &BlockTable) -> BlockDescriptor {
        let block_state = self
            .get_state_for_index(self.voxels[index] as usize)
            .unwrap();
        block_table.get(&block_state.identifier).unwrap().clone()
    }

    pub fn get_index_for_state(&self, block_data: &RenderedBlockData) -> Option<u16> {
        self.palette.get_by_right(block_data).copied()
    }

    pub fn get_state_for_index(&self, index: usize) -> Option<RenderedBlockData> {
        self.palette.get_by_left(&(index as u16)).cloned()
    }

    pub fn get_block(&self, pos: UVec3) -> Option<RenderedBlockData> {
        let index = ChunkBoundary::linearize(pos);
        self.get_state_for_index(self.voxels[index] as usize)
    }

    pub fn get_identifier(&self, pos: UVec3) -> String {
        let index = ChunkBoundary::linearize(pos);
        if let Some(block) = self.get_state_for_index(self.voxels[index] as usize) {
            block.identifier
        } else {
            "vinox:air".to_string()
        }
    }
}
