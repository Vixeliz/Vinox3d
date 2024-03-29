use serde::{Deserialize, Serialize};
use strum::EnumString;

pub const MAX_STACK_SIZE: u32 = 1000;

#[derive(EnumString, Default, Deserialize, Serialize, PartialEq, Eq, Debug, Clone)]
pub enum ToolType {
    Axe,
    #[default]
    Hand,
    Hoe,
    Knife,
    Pickaxe,
    Shovel,
}

// Anything optional here that is necessary for the game to function but we have a default value for ie texture or geometry
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Default, Clone)]
pub struct ItemDescriptor {
    pub namespace: String,
    pub name: String,
    pub texture: Option<String>,
    pub max_durability: Option<u32>,
    pub max_stack_size: Option<u32>, // Default will be the max
    pub tool_type: Option<ToolType>, // Basically for blocks we just do associated_block with no tool and vice versa for tools. But this allows people to make a tool that places a block for example. Scripts will also allow for people to add different functionality to items
    pub script: Option<String>,
    pub associated_block: Option<String>, // String should be an identifier in form of namespace:name, Potentially may change this to be block data instead so people could choose a certain state of a block to put down but we will see
}

// Instance of a item with some data
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Default, Clone)]
pub struct ItemData {
    pub namespace: String,
    pub name: String,
    pub stack_size: u32,
    pub durability: u32,
    pub arbitary_data: Option<String>,
}
