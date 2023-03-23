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
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Default, Clone, Copy, Hash)]
pub struct FaceDescript {
    pub uv: [((i8, i8), (i8, i8)); 6],
    pub discard: [bool; 6], // Should we completely ignore this face regardless
    pub cull: [bool; 6],    // Should this face be culled if there is a block next to it
    pub origin: (i8, i8, i8),
    pub end: (i8, i8, i8),
    pub rotation: (i8, i8, i8),
    pub pivot: (i8, i8, i8), //CULLING CAN BE DONE BY CHECKING IF ANY GIVEN FACE IS TOUCHING THE SIDES OF THE NEIGHBORS FACE?
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Default, Clone, Hash)]
pub struct BlockGeo {
    pub pivot: (i8, i8, i8),
    pub rotation: (i8, i8, i8),
    pub cubes: Vec<FaceDescript>,
}

// Anything optional here that is necessary for the game to function but we have a default value for ie texture or geometry
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Default, Clone)]
pub struct GeometryDescriptor {
    pub namespace: String, // TODO: Make sure that we only allow one namespace:name pair
    pub name: String,      // Name of the recipe
    pub blocks: [bool; 6], // Does this block face block the face next to it so its culled
    pub element: BlockGeo,
}
