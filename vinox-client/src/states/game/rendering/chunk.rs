use bevy::prelude::{warn, UVec3};
use bimap::BiMap;
use itertools::*;
use serde_big_array::Array;
use vinox_common::{
    storage::blocks::descriptor::BlockDescriptor,
    world::chunks::storage::{
        BlockData, BlockTable, Chunk, RawChunk, VoxelType, VoxelVisibility, CHUNK_SIZE,
    },
};

const CHUNK_BOUND: u32 = CHUNK_SIZE + 1;
const TOTAL_CHUNK_SIZE_PADDED: usize =
    (CHUNK_SIZE_PADDED as usize) * (CHUNK_SIZE_PADDED as usize) * (CHUNK_SIZE_PADDED as usize);
const CHUNK_SIZE_PADDED: u32 = CHUNK_SIZE + 2;

#[derive(Clone)]
pub struct ChunkBoundary {
    pub voxels: Box<[BlockData; TOTAL_CHUNK_SIZE_PADDED]>,
}

impl Chunk for ChunkBoundary {
    type Output = VoxelType;

    const X: usize = CHUNK_SIZE_PADDED as usize;
    const Y: usize = CHUNK_SIZE_PADDED as usize;
    const Z: usize = CHUNK_SIZE_PADDED as usize;
    fn get(&self, x: u32, y: u32, z: u32, block_table: &BlockTable) -> Self::Output {
        self.get_voxel(ChunkBoundary::linearize(UVec3::new(x, y, z)), block_table)
    }
}

fn vec_to_boxed_array<T: Clone, const N: usize>(val: T) -> Box<[T; N]> {
    let boxed_slice = vec![val; N].into_boxed_slice();

    let ptr = Box::into_raw(boxed_slice) as *mut [T; N];

    unsafe { Box::from_raw(ptr) }
}

impl ChunkBoundary {
    pub fn new(center: RawChunk, neighbors: Box<Array<RawChunk, 26>>) -> Self {
        const MAX: u32 = CHUNK_SIZE;
        // Just cause CHUNK_SIZE is long
        let mut voxels: Box<[BlockData; TOTAL_CHUNK_SIZE_PADDED]> =
            vec_to_boxed_array(BlockData::new("vinox".to_string(), "air".to_string()));
        for idx in 0..TOTAL_CHUNK_SIZE_PADDED {
            let (x, y, z) = ChunkBoundary::delinearize(idx);
            voxels[idx] = match (x, y, z) {
                (0, 0, 0) => neighbors[0].get_data_pos(MAX - 1, MAX - 1, MAX - 1),
                (0, 0, 1..=MAX) => neighbors[1].get_data_pos(MAX - 1, MAX - 1, z - 1),
                (0, 0, CHUNK_BOUND) => neighbors[2].get_data_pos(MAX - 1, MAX - 1, 0),
                (0, 1..=MAX, 0) => neighbors[3].get_data_pos(MAX - 1, y - 1, MAX - 1),
                (0, 1..=MAX, 1..=MAX) => neighbors[4].get_data_pos(MAX - 1, y - 1, z - 1),
                (0, 1..=MAX, CHUNK_BOUND) => neighbors[5].get_data_pos(MAX - 1, y - 1, 0),
                (0, CHUNK_BOUND, 0) => neighbors[6].get_data_pos(MAX - 1, 0, MAX - 1),
                (0, CHUNK_BOUND, 1..=MAX) => neighbors[7].get_data_pos(MAX - 1, 0, z - 1),
                (0, CHUNK_BOUND, CHUNK_BOUND) => neighbors[8].get_data_pos(MAX - 1, 0, 0),
                (1..=MAX, 0, 0) => neighbors[9].get_data_pos(x - 1, MAX - 1, MAX - 1),
                (1..=MAX, 0, 1..=MAX) => neighbors[10].get_data_pos(x - 1, MAX - 1, z - 1),
                (1..=MAX, 0, CHUNK_BOUND) => neighbors[11].get_data_pos(x - 1, MAX - 1, 0),
                (1..=MAX, 1..=MAX, 0) => neighbors[12].get_data_pos(x - 1, y - 1, MAX - 1),
                (1..=MAX, 1..=MAX, 1..=MAX) => center.get_data_pos(x - 1, y - 1, z - 1),
                (1..=MAX, 1..=MAX, CHUNK_BOUND) => neighbors[13].get_data_pos(x - 1, y - 1, 0),
                (1..=MAX, CHUNK_BOUND, 0) => neighbors[14].get_data_pos(x - 1, 0, MAX - 1),
                (1..=MAX, CHUNK_BOUND, 1..=MAX) => neighbors[15].get_data_pos(x - 1, 0, z - 1),
                (1..=MAX, CHUNK_BOUND, CHUNK_BOUND) => neighbors[16].get_data_pos(x - 1, 0, 0),
                (CHUNK_BOUND, 0, 0) => neighbors[17].get_data_pos(0, MAX - 1, MAX - 1),
                (CHUNK_BOUND, 0, 1..=MAX) => neighbors[18].get_data_pos(0, MAX - 1, z - 1),
                (CHUNK_BOUND, 0, CHUNK_BOUND) => neighbors[19].get_data_pos(0, MAX - 1, 0),
                (CHUNK_BOUND, 1..=MAX, 0) => neighbors[20].get_data_pos(0, y - 1, MAX - 1),
                (CHUNK_BOUND, 1..=MAX, 1..=MAX) => neighbors[21].get_data_pos(0, y - 1, z - 1),
                (CHUNK_BOUND, 1..=MAX, CHUNK_BOUND) => neighbors[22].get_data_pos(0, y - 1, 0),
                (CHUNK_BOUND, CHUNK_BOUND, 0) => neighbors[23].get_data_pos(0, 0, MAX - 1),
                (CHUNK_BOUND, CHUNK_BOUND, 1..=MAX) => neighbors[24].get_data_pos(0, 0, z - 1),
                (CHUNK_BOUND, CHUNK_BOUND, CHUNK_BOUND) => neighbors[25].get_data_pos(0, 0, 0),

                (_, _, _) => BlockData::new("vinox".to_string(), "air".to_string()),
            };
        }

        // let voxels: Box<[BlockData; TOTAL_CHUNK_SIZE_PADDED]> =
        //     Box::new(std::array::from_fn(|idx| {
        //         let (x, y, z) = ChunkBoundary::delinearize(idx);
        //         match (x, y, z) {
        //             (0, 0, 0) => neighbors[0].get_data_pos(MAX - 1, MAX - 1, MAX - 1),
        //             (0, 0, 1..=MAX) => neighbors[1].get_data_pos(MAX - 1, MAX - 1, z - 1),
        //             (0, 0, CHUNK_BOUND) => neighbors[2].get_data_pos(MAX - 1, MAX - 1, 0),
        //             (0, 1..=MAX, 0) => neighbors[3].get_data_pos(MAX - 1, y - 1, MAX - 1),
        //             (0, 1..=MAX, 1..=MAX) => neighbors[4].get_data_pos(MAX - 1, y - 1, z - 1),
        //             (0, 1..=MAX, CHUNK_BOUND) => neighbors[5].get_data_pos(MAX - 1, y - 1, 0),
        //             (0, CHUNK_BOUND, 0) => neighbors[6].get_data_pos(MAX - 1, 0, MAX - 1),
        //             (0, CHUNK_BOUND, 1..=MAX) => neighbors[7].get_data_pos(MAX - 1, 0, z - 1),
        //             (0, CHUNK_BOUND, CHUNK_BOUND) => neighbors[8].get_data_pos(MAX - 1, 0, 0),
        //             (1..=MAX, 0, 0) => neighbors[9].get_data_pos(x - 1, MAX - 1, MAX - 1),
        //             (1..=MAX, 0, 1..=MAX) => neighbors[10].get_data_pos(x - 1, MAX - 1, z - 1),
        //             (1..=MAX, 0, CHUNK_BOUND) => neighbors[11].get_data_pos(x - 1, MAX - 1, 0),
        //             (1..=MAX, 1..=MAX, 0) => neighbors[12].get_data_pos(x - 1, y - 1, MAX - 1),
        //             (1..=MAX, 1..=MAX, 1..=MAX) => center.get_data_pos(x - 1, y - 1, z - 1),
        //             (1..=MAX, 1..=MAX, CHUNK_BOUND) => neighbors[13].get_data_pos(x - 1, y - 1, 0),
        //             (1..=MAX, CHUNK_BOUND, 0) => neighbors[14].get_data_pos(x - 1, 0, MAX - 1),
        //             (1..=MAX, CHUNK_BOUND, 1..=MAX) => neighbors[15].get_data_pos(x - 1, 0, z - 1),
        //             (1..=MAX, CHUNK_BOUND, CHUNK_BOUND) => neighbors[16].get_data_pos(x - 1, 0, 0),
        //             (CHUNK_BOUND, 0, 0) => neighbors[17].get_data_pos(0, MAX - 1, MAX - 1),
        //             (CHUNK_BOUND, 0, 1..=MAX) => neighbors[18].get_data_pos(0, MAX - 1, z - 1),
        //             (CHUNK_BOUND, 0, CHUNK_BOUND) => neighbors[19].get_data_pos(0, MAX - 1, 0),
        //             (CHUNK_BOUND, 1..=MAX, 0) => neighbors[20].get_data_pos(0, y - 1, MAX - 1),
        //             (CHUNK_BOUND, 1..=MAX, 1..=MAX) => neighbors[21].get_data_pos(0, y - 1, z - 1),
        //             (CHUNK_BOUND, 1..=MAX, CHUNK_BOUND) => neighbors[22].get_data_pos(0, y - 1, 0),
        //             (CHUNK_BOUND, CHUNK_BOUND, 0) => neighbors[23].get_data_pos(0, 0, MAX - 1),
        //             (CHUNK_BOUND, CHUNK_BOUND, 1..=MAX) => neighbors[24].get_data_pos(0, 0, z - 1),
        //             (CHUNK_BOUND, CHUNK_BOUND, CHUNK_BOUND) => neighbors[25].get_data_pos(0, 0, 0),

        //             (_, _, _) => BlockData::new("vinox".to_string(), "air".to_string()),
        //         }
        //     }));

        ChunkBoundary { voxels }
    }

    pub const fn size() -> usize {
        TOTAL_CHUNK_SIZE_PADDED
    }

    pub fn get_voxel(&self, index: usize, block_table: &BlockTable) -> VoxelType {
        let block_state = self.voxels[index].clone();
        let mut block_name = block_state.namespace.clone();
        block_name.push(':');
        block_name.push_str(block_state.name.as_str());
        if let Some(voxel) = block_table.get(&block_name) {
            let voxel_visibility = voxel.visibility;
            if let Some(voxel_visibility) = voxel_visibility {
                match voxel_visibility {
                    VoxelVisibility::Empty => VoxelType::Empty(0),
                    VoxelVisibility::Opaque => VoxelType::Opaque(0),
                    VoxelVisibility::Transparent => VoxelType::Transparent(0),
                }
            } else {
                VoxelType::Empty(0)
            }
        } else {
            println!("No name: {block_name:?}");
            VoxelType::Empty(0)
        }
    }

    pub fn get_data(&self, index: usize, block_table: &BlockTable) -> BlockDescriptor {
        let block_state = self.voxels[index].clone();
        let mut block_name = block_state.namespace.clone();
        block_name.push(':');
        block_name.push_str(block_state.name.as_str());
        block_table.get(&block_name).unwrap().clone()
    }

    pub fn get_block(&self, pos: UVec3) -> Option<BlockData> {
        let index = ChunkBoundary::linearize(pos);
        Some(self.voxels[index].clone())
    }

    pub fn get_identifier(&self, pos: UVec3) -> String {
        let index = ChunkBoundary::linearize(pos);
        let block = self.voxels[index].clone();
        let mut identifier = block.namespace.clone();
        identifier.push(':');
        identifier.push_str(&block.name);
        identifier
    }
}
