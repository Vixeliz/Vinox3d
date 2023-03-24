use bevy::prelude::*;
use serde_big_array::Array;

use super::storage::{BlockTable, ChunkData, TOTAL_CHUNK_SIZE};

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct LightData {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8, // More like intensity
}

#[derive(Clone, Debug)]
pub struct LightNode {
    pub index: usize,
}

#[derive(Component, Default, Debug, Clone)]
pub struct LightChunk {
    pub light: Box<Array<(LightData, LightData), TOTAL_CHUNK_SIZE>>,
    pub queue: Vec<LightNode>,
    pub remove_queue: Vec<LightNode>,
} // First light data is light placed, second is sky
