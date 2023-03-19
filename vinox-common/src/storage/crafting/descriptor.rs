use std::collections::HashMap;

use serde::{Deserialize, Serialize};

// Anything optional here that is necessary for the game to function but we have a default value for ie texture or geometry
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Default, Clone)]
pub struct RecipeDescriptor {
    pub namespace: String, // TODO: Make sure that we only allow one namespace:name pair
    pub name: String,      // Name of the recipe
    pub required_items: Option<HashMap<String, u32>>,
    pub output_item: (String, u32),
    pub script: Option<String>,
}
