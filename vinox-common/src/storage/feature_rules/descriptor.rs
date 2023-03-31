use serde::{Deserialize, Serialize};

use crate::world::chunks::storage::VoxelAxis;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone)]
pub enum FeatureRuleType {
    Scatter {
        iterations: u16,
        chance: u16,
    },
    Search {
        start: (i8, i8, i8),
        end: (i8, i8, i8),
        axis: VoxelAxis,
        min_placements: u8,
    },
    Snap {
        range: u8,
        ceiling: bool,
    },
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone)]
pub enum ListType {
    Ordered,
    Weighted,
    Unordered,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone)]
pub struct FeatureRuleDescriptor {
    pub name: String,
    pub namespace: String,
    pub feature: Vec<(String, u16)>,
    pub list_type: Option<ListType>,
    pub feature_type: FeatureRuleType,
}
