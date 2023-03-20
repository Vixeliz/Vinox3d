use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use strum::EnumString;

#[derive(EnumString, Default, Deserialize, Serialize, PartialEq, Eq, Debug, Clone)]
pub enum CullDirection {
    #[default]
    Front,
    Back,
    Left,
    Right,
    Up,
    Down,
}
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Default, Clone)]
pub struct FaceDescript {
    pub uv: [((i32, i32), (i32, i32)); 6],
    pub discard: [bool; 6], // Should we completely ignore this face regardless
    pub cull: [bool; 6],    // Should this face be culled if there is a block next to it
    pub origin: (i32, i32, i32),
    pub size: (i32, i32, i32),
    pub rotation: (i32, i32, i32),
    pub pivot: (i32, i32, i32), //CULLING CAN BE DONE BY CHECKING IF ANY GIVEN FACE IS TOUCHING THE SIDES OF THE NEIGHBORS FACE?
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Default, Clone)]
pub struct BlockGeo {
    pub pivot: (i32, i32, i32),
    pub rotation: (i32, i32, i32),
    pub cubes: Vec<FaceDescript>,
}

// Anything optional here that is necessary for the game to function but we have a default value for ie texture or geometry
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Default, Clone)]
pub struct GeometryDescriptor {
    pub namespace: String, // TODO: Make sure that we only allow one namespace:name pair
    pub name: String,      // Name of the recipe
    pub elements: Vec<BlockGeo>,
}
