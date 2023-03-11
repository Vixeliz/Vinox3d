use serde::{Deserialize, Serialize};
use strum::EnumString;

#[derive(EnumString, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum VoxelVisibility {
    Empty,
    Opaque,
    Transparent,
}

#[derive(EnumString, Serialize, Deserialize, PartialEq, Eq)]
pub enum Direction {
    North,
    West,
    East,
    South,
    Down,
    Up,
}

#[derive(Serialize, Deserialize)]
pub struct Container {
    pub items: Vec<String>, // Possibly could do a palette approach but may not be worth it
    pub max_size: u8,
}
