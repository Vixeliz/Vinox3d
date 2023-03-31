use serde::{Deserialize, Serialize};

use crate::world::chunks::storage::{BlockData, RelativeVoxelAxis, VoxelAxis};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone)]
pub enum FeatureType {
    SingleBlock {
        places: BlockData,
        valid_placements: Option<Vec<BlockData>>,
        can_replace: Option<Vec<BlockData>>,
    },
    Vein {
        amount: Option<u8>,
        can_replace: Option<Vec<BlockData>>,
        places: BlockData,
    },
    ColumnHead {
        direction: u8,                 // 0 for Down 1 for Up
        heights: Vec<((u8, u8), u16)>, // u8s for min and max height range then u16 for weight
        head_block: Vec<(BlockData, u16)>,
        body_blocks: Vec<(BlockData, u16)>,
    },
    Column {
        direction: u8,                 // 0 for Down 1 for Up
        heights: Vec<((u8, u8), u16)>, // u8s for min and max height range then u16 for weight
        body_blocks: Vec<(BlockData, u16)>,
    },
    FaceFeature {
        // Can be placed on any face
        places: BlockData,
        range: u8,
        ceiling: bool,
        floor: bool,
        wall: bool,
        valid_placements: Option<Vec<BlockData>>,
    },
    ExposedBlob {
        places: BlockData,
        exposed: RelativeVoxelAxis,
    },
    CappedColumn {
        direction: u8,                 // 0 for Down 1 for Up
        heights: Vec<((u8, u8), u16)>, // u8s for min and max height range then u16 for weight
        cap_block: Vec<(BlockData, u16)>,
        body_blocks: Vec<(BlockData, u16)>,
        cap_radius: u8,
    },
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone)]
pub struct FeatureDescriptor {
    pub name: String,
    pub namespace: String,
    pub feature_type: FeatureType,
}
