use bevy::prelude::UVec3;
use serde_big_array::Array;
use vinox_common::world::chunks::storage::{
    BlockData, BlockTable, Chunk, RawChunk, VoxelType, CHUNK_SIZE,
};

const CHUNK_BOUND: u32 = CHUNK_SIZE + 1;
const TOTAL_CHUNK_SIZE_PADDED: usize =
    (CHUNK_SIZE_PADDED as usize) * (CHUNK_SIZE_PADDED as usize) * (CHUNK_SIZE_PADDED as usize);
const CHUNK_SIZE_PADDED: u32 = CHUNK_SIZE + 2;

pub struct ChunkBoundary {
    center: RawChunk,
    neighbors: Box<Array<RawChunk, 26>>,
}

impl Chunk for ChunkBoundary {
    type Output = VoxelType;

    const X: usize = CHUNK_SIZE_PADDED as usize;
    const Y: usize = CHUNK_SIZE_PADDED as usize;
    const Z: usize = CHUNK_SIZE_PADDED as usize;

    fn get(&self, x: u32, y: u32, z: u32, block_table: &BlockTable) -> Self::Output {
        // let [x, y, z] = ChunkBoundary::delinearize(idx);
        // const MAX: usize = CHUNK_SIZE as usize;
        const MAX: u32 = CHUNK_SIZE; // Just cause CHUNK_SIZE is long
        match (x, y, z) {
            (0, 0, 0) => self.neighbors[0].get(MAX - 1, MAX - 1, MAX - 1, block_table),
            (0, 0, 1..=MAX) => self.neighbors[1].get(MAX - 1, MAX - 1, z - 1, block_table),
            (0, 0, CHUNK_BOUND) => self.neighbors[2].get(MAX - 1, MAX - 1, 0, block_table),
            (0, 1..=MAX, 0) => self.neighbors[3].get(MAX - 1, y - 1, MAX - 1, block_table),
            (0, 1..=MAX, 1..=MAX) => self.neighbors[4].get(MAX - 1, y - 1, z - 1, block_table),
            (0, 1..=MAX, CHUNK_BOUND) => self.neighbors[5].get(MAX - 1, y - 1, 0, block_table),
            (0, CHUNK_BOUND, 0) => self.neighbors[6].get(MAX - 1, 0, MAX - 1, block_table),
            (0, CHUNK_BOUND, 1..=MAX) => self.neighbors[7].get(MAX - 1, 0, z - 1, block_table),
            (0, CHUNK_BOUND, CHUNK_BOUND) => self.neighbors[8].get(MAX - 1, 0, 0, block_table),
            (1..=MAX, 0, 0) => self.neighbors[9].get(x - 1, MAX - 1, MAX - 1, block_table),
            (1..=MAX, 0, 1..=MAX) => self.neighbors[10].get(x - 1, MAX - 1, z - 1, block_table),
            (1..=MAX, 0, CHUNK_BOUND) => self.neighbors[11].get(x - 1, MAX - 1, 0, block_table),
            (1..=MAX, 1..=MAX, 0) => self.neighbors[12].get(x - 1, y - 1, MAX - 1, block_table),
            (1..=MAX, 1..=MAX, 1..=MAX) => self.center.get(x - 1, y - 1, z - 1, block_table),
            (1..=MAX, 1..=MAX, CHUNK_BOUND) => self.neighbors[13].get(x - 1, y - 1, 0, block_table),
            (1..=MAX, CHUNK_BOUND, 0) => self.neighbors[14].get(x - 1, 0, MAX - 1, block_table),
            (1..=MAX, CHUNK_BOUND, 1..=MAX) => self.neighbors[15].get(x - 1, 0, z - 1, block_table),
            (1..=MAX, CHUNK_BOUND, CHUNK_BOUND) => self.neighbors[16].get(x - 1, 0, 0, block_table),
            (CHUNK_BOUND, 0, 0) => self.neighbors[17].get(0, MAX - 1, MAX - 1, block_table),
            (CHUNK_BOUND, 0, 1..=MAX) => self.neighbors[18].get(0, MAX - 1, z - 1, block_table),
            (CHUNK_BOUND, 0, CHUNK_BOUND) => self.neighbors[19].get(0, MAX - 1, 0, block_table),
            (CHUNK_BOUND, 1..=MAX, 0) => self.neighbors[20].get(0, y - 1, MAX - 1, block_table),
            (CHUNK_BOUND, 1..=MAX, 1..=MAX) => self.neighbors[21].get(0, y - 1, z - 1, block_table),
            (CHUNK_BOUND, 1..=MAX, CHUNK_BOUND) => self.neighbors[22].get(0, y - 1, 0, block_table),
            (CHUNK_BOUND, CHUNK_BOUND, 0) => self.neighbors[23].get(0, 0, MAX - 1, block_table),
            (CHUNK_BOUND, CHUNK_BOUND, 1..=MAX) => self.neighbors[24].get(0, 0, z - 1, block_table),
            (CHUNK_BOUND, CHUNK_BOUND, CHUNK_BOUND) => self.neighbors[25].get(0, 0, 0, block_table),

            (_, _, _) => VoxelType::Empty(0),
        }
    }
}

impl ChunkBoundary {
    pub fn new(center: RawChunk, neighbors: Box<Array<RawChunk, 26>>) -> Self {
        Self { center, neighbors }
    }

    pub const fn size() -> usize {
        TOTAL_CHUNK_SIZE_PADDED
    }

    pub fn get_block(&self, x: u32, y: u32, z: u32) -> Option<BlockData> {
        // let [x, y, z] = ChunkBoundary::delinearize(idx);
        // const MAX: usize = CHUNK_SIZE as usize;
        const MAX: u32 = CHUNK_SIZE; // Just cause CHUNK_SIZE is long
        match (x, y, z) {
            (0, 0, 0) => self.neighbors[0].get_block(UVec3::new(MAX - 1, MAX - 1, MAX - 1)),
            (0, 0, 1..=MAX) => self.neighbors[1].get_block(UVec3::new(MAX - 1, MAX - 1, z - 1)),
            (0, 0, CHUNK_BOUND) => self.neighbors[2].get_block(UVec3::new(MAX - 1, MAX - 1, 0)),
            (0, 1..=MAX, 0) => self.neighbors[3].get_block(UVec3::new(MAX - 1, y - 1, MAX - 1)),
            (0, 1..=MAX, 1..=MAX) => self.neighbors[4].get_block(UVec3::new(MAX - 1, y - 1, z - 1)),
            (0, 1..=MAX, CHUNK_BOUND) => self.neighbors[5].get_block(UVec3::new(MAX - 1, y - 1, 0)),
            (0, CHUNK_BOUND, 0) => self.neighbors[6].get_block(UVec3::new(MAX - 1, 0, MAX - 1)),
            (0, CHUNK_BOUND, 1..=MAX) => self.neighbors[7].get_block(UVec3::new(MAX - 1, 0, z - 1)),
            (0, CHUNK_BOUND, CHUNK_BOUND) => self.neighbors[8].get_block(UVec3::new(MAX - 1, 0, 0)),
            (1..=MAX, 0, 0) => self.neighbors[9].get_block(UVec3::new(x - 1, MAX - 1, MAX - 1)),
            (1..=MAX, 0, 1..=MAX) => {
                self.neighbors[10].get_block(UVec3::new(x - 1, MAX - 1, z - 1))
            }
            (1..=MAX, 0, CHUNK_BOUND) => {
                self.neighbors[11].get_block(UVec3::new(x - 1, MAX - 1, 0))
            }
            (1..=MAX, 1..=MAX, 0) => {
                self.neighbors[12].get_block(UVec3::new(x - 1, y - 1, MAX - 1))
            }
            (1..=MAX, 1..=MAX, 1..=MAX) => self.center.get_block(UVec3::new(x - 1, y - 1, z - 1)),
            (1..=MAX, 1..=MAX, CHUNK_BOUND) => {
                self.neighbors[13].get_block(UVec3::new(x - 1, y - 1, 0))
            }
            (1..=MAX, CHUNK_BOUND, 0) => {
                self.neighbors[14].get_block(UVec3::new(x - 1, 0, MAX - 1))
            }
            (1..=MAX, CHUNK_BOUND, 1..=MAX) => {
                self.neighbors[15].get_block(UVec3::new(x - 1, 0, z - 1))
            }
            (1..=MAX, CHUNK_BOUND, CHUNK_BOUND) => {
                self.neighbors[16].get_block(UVec3::new(x - 1, 0, 0))
            }
            (CHUNK_BOUND, 0, 0) => self.neighbors[17].get_block(UVec3::new(0, MAX - 1, MAX - 1)),
            (CHUNK_BOUND, 0, 1..=MAX) => {
                self.neighbors[18].get_block(UVec3::new(0, MAX - 1, z - 1))
            }
            (CHUNK_BOUND, 0, CHUNK_BOUND) => {
                self.neighbors[19].get_block(UVec3::new(0, MAX - 1, 0))
            }
            (CHUNK_BOUND, 1..=MAX, 0) => {
                self.neighbors[20].get_block(UVec3::new(0, y - 1, MAX - 1))
            }
            (CHUNK_BOUND, 1..=MAX, 1..=MAX) => {
                self.neighbors[21].get_block(UVec3::new(0, y - 1, z - 1))
            }
            (CHUNK_BOUND, 1..=MAX, CHUNK_BOUND) => {
                self.neighbors[22].get_block(UVec3::new(0, y - 1, 0))
            }
            (CHUNK_BOUND, CHUNK_BOUND, 0) => {
                self.neighbors[23].get_block(UVec3::new(0, 0, MAX - 1))
            }
            (CHUNK_BOUND, CHUNK_BOUND, 1..=MAX) => {
                self.neighbors[24].get_block(UVec3::new(0, 0, z - 1))
            }
            (CHUNK_BOUND, CHUNK_BOUND, CHUNK_BOUND) => {
                self.neighbors[25].get_block(UVec3::new(0, 0, 0))
            }

            (_, _, _) => Some(BlockData::new("vinox".to_string(), "air".to_string())),
        }
    }
}

pub fn chunk_boundary() {}
