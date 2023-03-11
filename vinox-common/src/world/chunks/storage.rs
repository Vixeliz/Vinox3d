use std::collections::HashMap;

use bevy::prelude::*;
use bimap::BiMap;
use itertools::*;
use serde::{Deserialize, Serialize};
use serde_big_array::Array;
use strum::EnumString;

use crate::storage::blocks::descriptor::BlockDescriptor;

pub const CHUNK_SIZE: u32 = 32;
pub const TOTAL_CHUNK_SIZE: usize =
    (CHUNK_SIZE as usize) * (CHUNK_SIZE as usize) * (CHUNK_SIZE as usize);

#[derive(Resource, Clone)]
pub struct BlockTable(pub HashMap<String, BlockDescriptor>);

#[derive(Resource, Default)]
pub struct CurrentChunks {
    pub chunks: HashMap<IVec3, Entity>,
}

#[derive(EnumString, Serialize, Deserialize, Debug, PartialEq, Eq, Default, Clone, Copy)]
pub enum VoxelVisibility {
    #[default]
    Empty,
    Opaque,
    Transparent,
}

#[derive(EnumString, Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone)]
pub enum Direction {
    North,
    West,
    East,
    South,
    Down,
    Up,
}

#[derive(EnumString, Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Default, Clone)]
pub enum GrowthState {
    #[default]
    Planted,
    Sapling,
    Young,
    Ripe,
    Spoiled,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone)]
pub struct Container {
    pub items: Vec<String>, // Hashmap would be better and may do more into implementing hashmyself at some point but this approach works for now
    pub max_size: u8,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Default, Clone)]
pub struct BlockData {
    pub namespace: String,
    pub name: String,
    pub direction: Option<Direction>,
    pub container: Option<Container>,
    pub growth_state: Option<GrowthState>,
    pub last_tick: Option<u64>,
    pub arbitary_data: Option<String>,
}

impl BlockData {
    pub fn new(namespace: String, name: String) -> Self {
        BlockData {
            namespace,
            name,
            ..Default::default()
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct RawChunk {
    pub palette: BiMap<u16, BlockData>,
    pub voxels: Box<Array<u16, TOTAL_CHUNK_SIZE>>,
}

#[derive(Copy, Clone, Hash, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum VoxelType {
    Empty(u16),
    Opaque(u16),
    Transparent(u16),
}

impl Default for VoxelType {
    fn default() -> VoxelType {
        Self::Empty(0)
    }
}

impl Voxel for VoxelType {
    fn visibility(&self) -> VoxelVisibility {
        match self {
            Self::Empty(_) => VoxelVisibility::Empty,
            Self::Opaque(_) => VoxelVisibility::Opaque,
            Self::Transparent(_) => VoxelVisibility::Transparent,
        }
    }
}

pub trait Voxel: Eq {
    fn visibility(&self) -> VoxelVisibility;
}

macro_rules! as_variant {
    ($value:expr, $variant:path) => {
        match $value {
            $variant(x) => Some(x),
            _ => None,
        }
    };
}

impl VoxelType {
    pub fn value(self) -> u16 {
        match self {
            Self::Empty(_) => as_variant!(self, VoxelType::Empty).unwrap_or(0),
            Self::Opaque(_) => as_variant!(self, VoxelType::Opaque).unwrap_or(0),
            Self::Transparent(_) => as_variant!(self, VoxelType::Transparent).unwrap_or(0),
        }
    }
}

pub trait Chunk {
    type Output;

    const X: usize;
    const Y: usize;
    const Z: usize;

    fn size() -> usize {
        Self::X * Self::Y * Self::Z
    }

    fn linearize(pos: UVec3) -> usize {
        let x = pos.x as usize;
        let y = pos.y as usize;
        let z = pos.z as usize;
        x + (y * Self::X) + (z * Self::X * Self::Y)
    }

    fn delinearize(mut index: usize) -> (u32, u32, u32) {
        let z = index / (Self::X * Self::Y);
        index -= z * (Self::X * Self::Y);

        let y = index / Self::X;
        index -= y * Self::X;

        let x = index;

        (x as u32, y as u32, z as u32)
    }

    fn get(&self, x: u32, y: u32, z: u32, block_table: &BlockTable) -> Self::Output;
}

impl Default for RawChunk {
    fn default() -> RawChunk {
        let mut raw_chunk = RawChunk {
            palette: BiMap::new(),
            voxels: Box::default(),
        };
        raw_chunk.palette.insert(
            0,
            BlockData {
                namespace: "vinox".to_string(),
                name: "air".to_string(),
                ..Default::default()
            },
        );
        raw_chunk
    }
}

impl Chunk for RawChunk {
    type Output = VoxelType;

    const X: usize = CHUNK_SIZE as usize;
    const Y: usize = CHUNK_SIZE as usize;
    const Z: usize = CHUNK_SIZE as usize;

    fn get(&self, x: u32, y: u32, z: u32, block_table: &BlockTable) -> Self::Output {
        self.get_voxel(RawChunk::linearize(UVec3::new(x, y, z)), block_table)
    }
}

impl RawChunk {
    // Very important to use this for creation because of air
    pub fn new() -> RawChunk {
        let mut raw_chunk = RawChunk {
            palette: BiMap::new(),
            voxels: Box::default(),
        };
        raw_chunk.palette.insert(
            0,
            BlockData {
                namespace: "vinox".to_string(),
                name: "air".to_string(),
                ..Default::default()
            },
        );
        raw_chunk
    }

    pub fn get_voxel(&self, index: usize, block_table: &BlockTable) -> VoxelType {
        let block_state = self
            .get_state_for_index(self.voxels[index] as usize)
            .unwrap();
        let block_id = self.get_index_for_state(&block_state).unwrap();
        let mut block_name = block_state.namespace.clone();
        block_name.push(':');
        block_name.push_str(block_state.name.as_str());
        let voxel_visibility = block_table.0.get(&block_name).unwrap().visibility;
        if let Some(voxel_visibility) = voxel_visibility {
            match voxel_visibility {
                VoxelVisibility::Empty => VoxelType::Empty(block_id),
                VoxelVisibility::Opaque => VoxelType::Opaque(block_id),
                VoxelVisibility::Transparent => VoxelType::Transparent(block_id),
            }
        } else {
            VoxelType::Empty(0)
        }
    }

    pub fn get_data(&self, index: usize, block_table: &BlockTable) -> BlockDescriptor {
        let block_state = self
            .get_state_for_index(self.voxels[index] as usize)
            .unwrap();
        let mut block_name = block_state.namespace.clone();
        block_name.push(':');
        block_name.push_str(block_state.name.as_str());
        block_table.0.get(&block_name).unwrap().clone()
    }

    pub fn get_index_for_state(&self, block_data: &BlockData) -> Option<u16> {
        self.palette.get_by_right(block_data).copied()
    }

    pub fn get_state_for_index(&self, index: usize) -> Option<BlockData> {
        self.palette.get_by_left(&(index as u16)).cloned()
    }

    // This is most likely a VERY awful way to handle this however for now I just want a working solution ill
    // rewrite this if it causes major performance issues
    pub fn update_chunk_pal(&mut self, old_pal: &BiMap<u16, BlockData>) {
        for i in 0..self.voxels.len() {
            if let Some(block_data) = old_pal.get_by_left(&self.voxels[i]) {
                if let Some(new_index) = self.get_index_for_state(block_data) {
                    self.voxels[i] = new_index;
                } else {
                    self.voxels[i] = 0;
                }
            }
        }
    }
    fn max_block_id(&self) -> u16 {
        let mut counter = 0;
        for id in self.palette.left_values().sorted() {
            if *id != 0 && counter < id - 1 {
                return *id;
            }
            counter = *id;
        }
        counter + 1
    }
    pub fn add_block_state(&mut self, block_data: &BlockData) {
        let old_pal = self.palette.clone();
        if let Some(_id) = self.get_index_for_state(block_data) {
        } else {
            self.palette
                .insert(self.max_block_id(), block_data.to_owned());
            self.update_chunk_pal(&old_pal);
        }
    }
    pub fn remove_block_state(&mut self, block_data: &BlockData) {
        if block_data.eq(&BlockData {
            namespace: "vinox".to_string(),
            name: "air".to_string(),
            ..Default::default()
        }) {
            return;
        }
        let old_pal = self.palette.clone();
        if let Some(id) = self.get_index_for_state(block_data) {
            self.palette.remove_by_left(&id);
            self.update_chunk_pal(&old_pal);
        } else {
            warn!("Block data: {}, doesn't exist!", block_data.name);
        }
    }
    // This actual chunks data starts at 1,1,1 and ends at chunk_size
    pub fn set_block(&mut self, pos: UVec3, block_data: &BlockData) {
        let index = RawChunk::linearize(pos);
        if let Some(block_type) = self.get_index_for_state(block_data) {
            if block_type == 0 {
                self.voxels[index] = 0;
            } else {
                self.voxels[index] = block_type; // Set based off of transluency
            }
        } else {
            warn!("Voxel doesn't exist");
        }
    }
    pub fn get_block(&self, pos: UVec3) -> Option<BlockData> {
        let index = RawChunk::linearize(pos);
        self.get_state_for_index(self.voxels[index] as usize)
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::UVec3;

    use super::{BlockData, RawChunk};

    #[test]
    fn palette_works() {
        let mut raw_chunk = RawChunk::default();
        raw_chunk.add_block_state(&BlockData::new("vinox".to_string(), "dirt".to_string()));
        let grass = BlockData::new("vinox".to_string(), "grass".to_string());
        raw_chunk.add_block_state(&grass);
        raw_chunk.set_block(UVec3::new(1, 1, 1), &grass);
        assert_eq!(
            raw_chunk.get_block(UVec3::new(1, 1, 1)),
            Some(grass.clone())
        );
        raw_chunk.remove_block_state(&BlockData::new("vinox".to_string(), "dirt".to_string()));
        assert_eq!(raw_chunk.get_block(UVec3::new(1, 1, 1)), Some(grass));
    }
}
