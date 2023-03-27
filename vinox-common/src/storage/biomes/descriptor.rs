use serde::{Deserialize, Serialize};
use strum::EnumString;

use crate::storage::structures::descriptor::StructureBlocks;

pub const MAX_STACK_SIZE: u32 = 1000;

#[derive(EnumString, Default, Deserialize, Serialize, PartialEq, Eq, Debug, Clone)]
pub enum TerrainCarver {
    #[default]
    Standard,
    Terrace,
    Flat,
    Overhangs,
}

#[derive(EnumString, Default, Deserialize, Serialize, PartialEq, Eq, Debug, Clone)]
pub enum TerrainType {
    #[default]
    Standard, //Doesn't affect normal noise at all,
    Flat,     // Makes noise more flat
    Mountain, // Self explanatory
    RollingHills,
}

// Anything optional here that is necessary for the game to function but we have a default value for ie texture or geometry
#[derive(Serialize, Deserialize, Debug, PartialEq, Default, Clone)]
pub struct BiomeDescriptor {
    pub namespace: String,
    pub name: String,
    pub terrain_type: TerrainType,
    pub amplitude: Option<f32>, // This is what terrain type sets for simplicity
    pub heat: f32,
    pub humidity: f32,
    pub surface_block: Option<String>,
    pub main_block: String,
    pub structures: Option<Vec<StructureBlocks>>,
}
