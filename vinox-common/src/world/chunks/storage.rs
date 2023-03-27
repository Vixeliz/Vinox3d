use bitvec::prelude::*;
use rustc_hash::FxHashMap;

use bevy::prelude::*;
use itertools::*;
use ndshape::{ConstShape, ConstShape3usize};
use serde::{Deserialize, Serialize};
use strum::EnumString;

use crate::storage::{
    blocks::descriptor::BlockDescriptor, crafting::descriptor::RecipeDescriptor,
    items::descriptor::ItemDescriptor,
};

use super::light::LightStorage;

pub const HORIZONTAL_DISTANCE: usize = 16;
pub const VERTICAL_DISTANCE: usize = 8;
pub const CHUNK_SIZE: usize = 16;
pub const CHUNK_SIZE_ARR: u32 = CHUNK_SIZE as u32 - 1;
pub const TOTAL_CHUNK_SIZE: usize = (CHUNK_SIZE) * (CHUNK_SIZE) * (CHUNK_SIZE);

type ChunkShape = ConstShape3usize<CHUNK_SIZE, CHUNK_SIZE, CHUNK_SIZE>;

#[derive(Resource, Clone, Default, Deref, DerefMut)]
pub struct RecipeTable(pub FxHashMap<String, RecipeDescriptor>);

#[derive(Resource, Clone, Default, Deref, DerefMut)]
pub struct BlockTable(pub FxHashMap<String, BlockDescriptor>);

#[derive(Resource, Clone, Default, Deref, DerefMut)]
pub struct ItemTable(pub FxHashMap<String, ItemDescriptor>);

#[derive(EnumString, Serialize, Deserialize, Debug, PartialEq, Eq, Default, Clone, Copy, Hash)]
pub enum VoxelVisibility {
    #[default]
    Empty,
    Opaque,
    Transparent,
}

#[derive(EnumString, Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone, Default, Copy)]
pub enum Direction {
    #[default]
    North,
    West,
    East,
    South,
}

impl Direction {
    pub fn get_as_string(&self) -> String {
        match self {
            Direction::North => "north".to_string(),
            Direction::West => "west".to_string(),
            Direction::East => "east".to_string(),
            Direction::South => "south".to_string(),
        }
    }
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

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct RenderedBlockData {
    // pub identifier: String,
    pub direction: Option<Direction>,
    pub top: Option<bool>,
    // pub geo: BlockGeo,
    pub geo_index: usize,
    pub visibility: VoxelVisibility,
    pub has_direction: bool,
    pub exclusive_direction: bool,
    pub textures: [usize; 6],
    pub tex_variance: [bool; 6],
    pub blocks: [bool; 6],
    pub light: u8,
}

pub fn name_to_identifier(namespace: String, name: String) -> String {
    let mut temp_name = namespace;
    temp_name.push(':');
    temp_name.push_str(&name);
    temp_name
}

pub fn identifier_to_name(identifier: String) -> Option<(String, String)> {
    if let Some((namespace, name)) = identifier.splitn(2, ':').tuples().next() {
        return Some((namespace.to_string(), name.to_string()));
    }
    None
}

pub fn identifier_to_just_name(identifier: String) -> Option<String> {
    if let Some((_, name)) = identifier.splitn(2, ':').tuples().next() {
        return Some(name.to_string());
    }
    None
}

pub fn trim_geo_identifier(identifier: String) -> String {
    if let Some((prefix, _)) = identifier.split_once('.') {
        prefix.to_string()
    } else {
        identifier
    }
}

impl Default for RenderedBlockData {
    fn default() -> Self {
        RenderedBlockData {
            // identifier: "vinox:air".to_string(),
            visibility: VoxelVisibility::Empty,
            blocks: [false, false, false, false, false, false],
            tex_variance: [false, false, false, false, false, false],
            textures: [0, 0, 0, 0, 0, 0],
            geo_index: 0,
            has_direction: false,
            exclusive_direction: false,
            direction: None,
            top: None,
            // geo: block_geo().unwrap(),
            light: 0,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone)]
pub struct BlockData {
    pub namespace: String,
    pub name: String,
    pub direction: Option<Direction>,
    pub container: Option<Container>,
    pub growth_state: Option<GrowthState>,
    pub last_tick: Option<u64>,
    pub arbitary_data: Option<String>,
    pub top: Option<bool>,
}

impl BlockData {
    pub fn is_empty(&self, block_table: &BlockTable) -> bool {
        block_table
            .get(&name_to_identifier(
                self.namespace.clone(),
                self.name.clone(),
            ))
            .unwrap()
            .visibility
            .unwrap_or_default()
            == VoxelVisibility::Empty
    }
    pub fn is_opaque(&self, block_table: &BlockTable) -> bool {
        block_table
            .get(&name_to_identifier(
                self.namespace.clone(),
                self.name.clone(),
            ))
            .unwrap()
            .visibility
            .unwrap_or_default()
            == VoxelVisibility::Opaque
    }
    pub fn is_true_empty(&self, block_table: &BlockTable) -> bool {
        let descriptor = block_table
            .get(&name_to_identifier(
                self.namespace.clone(),
                self.name.clone(),
            ))
            .unwrap();
        !(descriptor.visibility.unwrap_or_default() == VoxelVisibility::Opaque
            && descriptor
                .geometry
                .clone()
                .unwrap_or_default()
                .get_geo_namespace()
                == "vinox:block")
    }
}

impl Default for BlockData {
    fn default() -> Self {
        BlockData {
            namespace: "vinox".to_string(),
            name: "air".to_string(),
            direction: None,
            container: None,
            growth_state: None,
            last_tick: None,
            arbitary_data: None,
            top: None,
        }
    }
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Storage {
    Single(SingleStorage),
    Multi(MultiStorage),
}

/// Compressed storage for volumes with a single voxel type
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SingleStorage {
    size: usize,
    voxel: BlockData,
}

/// Palette compressed storage for volumes with multiple voxel types
/// Based on https://voxel.wiki/wiki/palette-compression/
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MultiStorage {
    /// Size of chunk storage, in voxels
    size: usize,
    data: BitBuffer,
    palette: Vec<PaletteEntry>,
    /// Palette capacity given size of indices
    /// Not necessarily equal to palette vector capacity
    palette_capacity: usize,
    /// Bit length of indices into the palette
    indices_length: usize,
}

impl MultiStorage {
    fn new(size: usize, initial_voxel: BlockData) -> Self {
        // Indices_length of 2 since this is only used for multiple voxel types
        let indices_length = 2;
        let initial_capacity = 2_usize.pow(indices_length as u32);
        let mut palette = Vec::with_capacity(initial_capacity);
        palette.push(PaletteEntry {
            voxel_type: initial_voxel,
            ref_count: size,
        });

        Self {
            size,
            data: BitBuffer::new(size * indices_length),
            palette,
            palette_capacity: initial_capacity,
            indices_length,
        }
    }

    fn grow_palette(&mut self) {
        let mut indices: Vec<usize> = Vec::with_capacity(self.size);
        for i in 0..self.size {
            indices.push(self.data.get(i * self.indices_length, self.indices_length));
        }

        self.indices_length <<= 1;
        let new_capacity = 2usize.pow(self.indices_length as u32);
        self.palette.reserve(new_capacity - self.palette_capacity);
        self.palette_capacity = new_capacity;

        self.data = BitBuffer::new(self.size * self.indices_length);

        for (i, idx) in indices.into_iter().enumerate() {
            self.data
                .set(i * self.indices_length, self.indices_length, idx);
        }
    }
}

impl Storage {
    pub fn new(size: usize) -> Self {
        Self::Single(SingleStorage {
            size,
            voxel: BlockData::default(),
        })
    }

    fn toggle_storage_type(&mut self) {
        *self = match self {
            Storage::Single(storage) => {
                Storage::Multi(MultiStorage::new(storage.size, storage.voxel.clone()))
            }
            Storage::Multi(storage) => {
                assert!(storage.palette.len() == 1);
                Storage::Single(SingleStorage {
                    size: storage.size,
                    voxel: storage.palette[0].voxel_type.clone(),
                })
            }
        };
    }

    pub fn set(&mut self, target_idx: usize, voxel: BlockData) {
        match self {
            Storage::Single(storage) => {
                if storage.voxel != voxel {
                    self.toggle_storage_type();
                    self.set(target_idx, voxel);
                }
            }
            Storage::Multi(storage) => {
                let palette_target_idx: usize = storage
                    .data
                    .get(target_idx * storage.indices_length, storage.indices_length);
                if let Some(target) = storage.palette.get_mut(palette_target_idx) {
                    target.ref_count -= 1;
                }

                // Look for voxel palette entry
                let palette_entry_voxel =
                    storage.palette.iter().enumerate().find_map(|(idx, entry)| {
                        if entry.voxel_type == voxel {
                            Some(idx)
                        } else {
                            None
                        }
                    });

                // Voxel type already in palette
                if let Some(idx) = palette_entry_voxel {
                    storage.data.set(
                        target_idx * storage.indices_length,
                        storage.indices_length,
                        idx,
                    );
                    storage
                        .palette
                        .get_mut(idx)
                        .expect("Failed to get palette entry of target voxel")
                        .ref_count += 1;

                    return;
                }

                // Overwrite target palette entry
                if let Some(target) = storage.palette.get_mut(palette_target_idx) {
                    if target.ref_count == 0 {
                        target.voxel_type = voxel;
                        target.ref_count = 1;

                        return;
                    }
                }

                // Create new palette entry
                //bevy::prelude::info!("Creating new voxel entry for {:?}", voxel);
                let new_entry_idx = if let Some((i, entry)) = storage
                    .palette
                    .iter_mut()
                    .enumerate()
                    .find(|(_i, entry)| entry.ref_count == 0)
                {
                    // Recycle a ref_count 0 entry if any exists
                    entry.voxel_type = voxel;
                    entry.ref_count = 1;

                    i
                } else {
                    // Create a new entry from scratch
                    if storage.palette.len() == storage.palette_capacity {
                        storage.grow_palette();
                    }

                    storage.palette.push(PaletteEntry {
                        voxel_type: voxel,
                        ref_count: 1,
                    });

                    storage.palette.len() - 1
                };
                storage.data.set(
                    target_idx * storage.indices_length,
                    storage.indices_length,
                    new_entry_idx,
                );
            }
        }
    }

    pub fn get(&self, idx: usize) -> BlockData {
        match self {
            Storage::Single(storage) => storage.voxel.clone(),
            Storage::Multi(storage) => {
                let palette_idx: usize = storage
                    .data
                    .get(idx * storage.indices_length, storage.indices_length);

                storage
                    .palette
                    .get(palette_idx)
                    .expect("Failed to get palette entry in voxel get")
                    .voxel_type
                    .clone()
            }
        }
    }

    pub fn trim(&mut self) {
        match self {
            Storage::Single(_) => (),
            Storage::Multi(storage) => {
                if storage.palette.len() == 1 {
                    self.toggle_storage_type();
                }
            }
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct PaletteEntry {
    voxel_type: BlockData,
    ref_count: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct BitBuffer {
    bytes: BitVec<u8, Lsb0>,
}

impl BitBuffer {
    /// Create a new BitBuffer
    /// size is specified in bits, not bytes
    fn new(size: usize) -> Self {
        Self {
            bytes: BitVec::repeat(false, size),
        }
    }

    /// Set arbitraty bits in BitBuffer.
    /// idx, bit_length and bits are specified in bits, not bytes
    fn set(&mut self, idx: usize, bit_length: usize, bits: usize) {
        self.bytes[idx..idx + bit_length].store_le::<usize>(bits);
    }

    /// Get arbitraty bits in BitBuffer.
    /// idx, bit_length are specified in bits, not bytes
    fn get(&self, idx: usize, bit_length: usize) -> usize {
        self.bytes[idx..idx + bit_length].load_le::<usize>()
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RawChunk {
    voxels: Storage,
}

#[derive(Component, Clone, Debug)]
pub struct ChunkData {
    voxels: Storage,
    lights: LightStorage,
    change_count: u16,
    dirty: bool,
}

impl Default for ChunkData {
    fn default() -> Self {
        Self {
            voxels: Storage::new(ChunkShape::USIZE),
            change_count: 0,
            dirty: true,
            lights: LightStorage::new(),
        }
    }
}

#[allow(dead_code)]
impl ChunkData {
    pub fn get(&self, x: u32, y: u32, z: u32) -> BlockData {
        self.voxels.get(Self::linearize(x, y, z))
    }

    pub fn get_identifier(&self, x: u32, y: u32, z: u32) -> String {
        let voxel = self.voxels.get(Self::linearize(x, y, z));
        name_to_identifier(voxel.namespace, voxel.name)
    }

    pub fn set(&mut self, x: u32, y: u32, z: u32, voxel: BlockData, _block_table: &BlockTable) {
        self.voxels.set(Self::linearize(x, y, z), voxel);
        self.change_count += 1;
        self.set_dirty(true);

        if self.change_count > 500 {
            self.voxels.trim();
            self.change_count = 0;
        }
        // let descriptor = block_table.get(&self.get_identifier(x, y, z)).unwrap();
        // let self_light = self.get_light(Self::linearize(x, y, z));
        // if let Some(light) = descriptor.light {
        //     let light = LightData {
        //         r: light.0,
        //         g: light.1,
        //         b: light.2,
        //         a: light.3,
        //     };
        //     if self_light != light {
        //         if light != LightData::default() {
        //             self.set_light(Self::linearize(x, y, z), light);
        //             // self.calculate_all_light(block_table);
        //         } else {
        //             self.remove_light(Self::linearize(x, y, z), self_light);
        //             // self.calculate_all_remove_lights();
        //             self.set_light(Self::linearize(x, y, z), light);
        //             // self.calculate_all_light(block_table);
        //         }
        //     }
        // } else {
        //     self.remove_light(Self::linearize(x, y, z), self_light);
        //     // self.calculate_all_remove_lights();
        //     self.set_light(
        //         Self::linearize(x, y, z),
        //         LightData {
        //             r: 0,
        //             b: 0,
        //             g: 0,
        //             a: 0,
        //         },
        //     );
        // self.calculate_all_light(block_table);
        // }
    }

    pub fn is_uniform(&self) -> bool {
        match self.voxels {
            Storage::Single(_) => true,
            Storage::Multi(_) => false,
        }
    }
    pub fn complete_relight(&mut self, _block_table: &BlockTable) -> ChunkData {
        // for x in 0..CHUNK_SIZE {
        //     for y in 0..CHUNK_SIZE {
        //         for z in 0..CHUNK_SIZE {
        //             let index = Self::linearize(x, y, z);
        //             if let Some(block) = block_table.get(&self.get_identifier(x, y, z)) {
        //                 if let Some(light) = block.light {
        //                     self.set_light(
        //                         index,
        //                         LightData {
        //                             r: light.0,
        //                             g: light.1,
        //                             b: light.2,
        //                             a: light.3,
        //                         },
        //                     );
        //                 }
        //             }
        //         }
        //     }
        // }
        // self.calculate_all_light(block_table);
        self.clone()
    }
    pub fn is_empty(&self, block_table: &BlockTable) -> bool {
        self.is_uniform() && self.get(0, 0, 0).is_empty(block_table)
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn set_dirty(&mut self, dirty: bool) {
        self.dirty = dirty;
    }

    pub fn trim(&mut self) {
        self.voxels.trim();
    }

    pub const fn size() -> u32 {
        ChunkShape::USIZE as u32
    }

    pub const fn usize() -> usize {
        ChunkShape::USIZE
    }

    pub const fn edge() -> usize {
        CHUNK_SIZE
    }

    #[inline]
    pub fn linearize(x: u32, y: u32, z: u32) -> usize {
        ChunkShape::linearize([x as usize, y as usize, z as usize])
    }

    #[inline]
    pub fn delinearize(idx: usize) -> (u32, u32, u32) {
        let res = ChunkShape::delinearize(idx);
        (res[0] as u32, res[1] as u32, res[2] as u32)
    }

    pub fn from_raw(raw_chunk: RawChunk) -> Self {
        Self {
            voxels: raw_chunk.voxels,
            change_count: 0,
            dirty: false,
            lights: LightStorage::new(),
        }
    }

    pub fn to_raw(&self) -> RawChunk {
        RawChunk {
            voxels: self.voxels.clone(),
        }
    }

    pub fn get_light(&self, x: u32, y: u32, z: u32) -> u8 {
        self.lights.get_light(Self::linearize(x, y, z))
    }

    /// Output is bounded between 0 and 15
    pub fn get_torchlight(&self, x: u32, y: u32, z: u32) -> u8 {
        self.lights.get_torchlight(Self::linearize(x, y, z))
    }

    /// Input is bounded between 0 and 15
    pub fn set_torchlight(&mut self, x: u32, y: u32, z: u32, value: u8) {
        self.lights.set_torchlight(Self::linearize(x, y, z), value);
    }

    /// Output is bounded between 0 and 15
    pub fn get_sunlight(&self, x: u32, y: u32, z: u32) -> u8 {
        self.lights.get_sunlight(Self::linearize(x, y, z))
    }

    /// Input is bounded between 0 and 15
    pub fn set_sunlight(&mut self, x: u32, y: u32, z: u32, value: u8) {
        self.lights.set_sunlight(Self::linearize(x, y, z), value);
    }
}
