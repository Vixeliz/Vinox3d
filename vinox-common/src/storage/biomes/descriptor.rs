use serde::{Deserialize, Serialize};
use strum::EnumString;

pub const MAX_STACK_SIZE: u32 = 1000;

#[derive(EnumString, Default, Deserialize, Serialize, PartialEq, Eq, Debug, Clone)]
pub enum TerrainCarver {
    #[default]
    Standard,
    Terrace,
    Flat,
    Overhangs,
}

// Anything optional here that is necessary for the game to function but we have a default value for ie texture or geometry
#[derive(Serialize, Deserialize, Debug, PartialEq, Default, Clone)]
pub struct BiomeDescriptor {
    pub namespace: String,
    pub name: String,
    pub depth_bias: i32, // Higher numbers will favor higher depths lower will prefer lower
    pub heat: i32,
    pub humidity: i32,
    pub surface_block: Option<Vec<(String, u16)>>,
    pub surface_depth: Option<u8>,
    pub ceil_block: Option<Vec<(String, u16)>>,
    pub ceil_depth: Option<u8>,
    pub main_block: Vec<(String, u16)>,
    pub feature_rules: Option<Vec<String>>,
}
