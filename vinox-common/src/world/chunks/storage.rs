use std::collections::HashMap;

use bevy::prelude::*;
use bimap::BiMap;
use serde::{Deserialize, Serialize};
use serde_big_array::Array;
use strum::EnumString;

const CHUNK_SIZE: u32 = 32;
const TOTAL_CHUNK_SIZE: usize =
    (CHUNK_SIZE as usize) * (CHUNK_SIZE as usize) * (CHUNK_SIZE as usize);
// TODO: Move these three to client as its only for meshing
const CHUNK_BOUND: u32 = CHUNK_SIZE + 1;
const TOTAL_CHUNK_SIZE_PADDED: usize =
    (CHUNK_SIZE_PADDED as usize) * (CHUNK_SIZE_PADDED as usize) * (CHUNK_SIZE_PADDED as usize);
const CHUNK_SIZE_PADDED: u32 = CHUNK_SIZE + 2;

#[derive(Resource, Default)]
pub struct CurrentChunks {
    pub chunks: HashMap<IVec3, Entity>,
}

#[derive(EnumString, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum VoxelVisibility {
    Empty,
    Opaque,
    Transparent,
}

#[derive(EnumString, Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
pub enum Direction {
    North,
    West,
    East,
    South,
    Down,
    Up,
}

#[derive(EnumString, Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
pub enum GrowthState {
    Planted,
    Sapling,
    Young,
    Ripe,
    Spoiled,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
pub struct Container {
    pub items: Vec<String>, // Hashmap would be better and may do more into implementing hashmyself at some point but this approach works for now
    pub max_size: u8,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
pub struct BlockData {
    pub namespace: String,
    pub name: String,
    pub direction: Option<Direction>,
    pub container: Option<Container>,
    pub growth_state: Option<GrowthState>,
    pub last_tick: Option<u64>,
    pub arbitary_data: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct RawChunk {
    palette: BiMap<u16, BlockData>,
    voxels: Box<Array<u16, TOTAL_CHUNK_SIZE>>,
}
