use std::collections::HashMap;

use crate::{storage::items::descriptor::ToolType, world::chunks::storage::VoxelVisibility};
use serde::{Deserialize, Serialize};
use strum::EnumString;

/*  Technically we could do something similiar to mc for completely custom models. However
due to personal preference i would rather only allow certain types listed below.    */
#[derive(EnumString, Default, Deserialize, Serialize, PartialEq, Eq, Debug, Clone)]
pub enum BlockGeometry {
    #[default]
    Block, // Standard Voxel --DONE
    Stairs,
    Slab,          // Both vertical and horizontal --DONE
    BorderedBlock, //Basically the bottom still touchs the normal bottom of a block but has a border around all the others --DONE
    Fence,
    Flat,           // Flat texture that can go on top of a block --DONE
    Cross,          // Crossed textures think like flowers from a popular block game --DONE
    Custom(String), // Custom models defined by the geometry file type
}

// Anything optional here that is necessary for the game to function but we have a default value for ie texture or geometry
// NOTE: We will also take in any children blocks this block may have. ie any slab, fence, stair variant etc
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Default, Clone)]
pub struct BlockDescriptor {
    pub namespace: String, // TODO: Make sure that we only allow one namespace:name pair
    pub name: String,
    pub textures: Option<HashMap<Option<String>, Option<String>>>,
    pub geometry: Option<BlockGeometry>,
    pub durability: Option<u32>,
    pub tool_type: Option<ToolType>,
    pub friction: Option<u32>,
    pub walk_sound: Option<String>,
    pub break_sound: Option<String>,
    pub script: Option<String>,
    pub container_size: Option<u8>,
    pub visibility: Option<VoxelVisibility>,
    pub light: Option<u8>,
    pub interactable: Option<bool>,
    pub gui: Option<String>,
    pub has_item: Option<bool>, // Basically whether or not we should auto generate an item for this block
}
