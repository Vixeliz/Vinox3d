use bevy::prelude::*;
use bevy_quinnet::shared::ClientId;

#[derive(Resource)]
pub struct NetworkIP(pub String);

use serde::{Deserialize, Serialize};

#[derive(Component)]
pub struct NetworkedEntity;

#[derive(Debug, Component, Default)]
pub struct Player {
    pub id: ClientId,
}

// Networking related
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct NetworkedEntities {
    pub entities: Vec<Entity>,
    pub translations: Vec<Vec3>,
    pub rotations: Vec<Vec4>,
}

#[derive(Default, Resource)]
pub struct EntityBuffer {
    pub entities: [NetworkedEntities; 30],
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ClientMessage {
    Position {
        player_pos: Vec3,
        player_rot: Vec4,
    },
    Interact {
        entity: Entity,
        attack: bool,
    },

    SentBlock {
        chunk_pos: IVec3,
        voxel_pos: [u8; 3],
        block_type: String,
    },
    Join {
        user_name: String, // Username is just for display we use an id for the actual identification of clients
        id: ClientId,
    },
    Leave {
        id: ClientId,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ServerMessage {
    ClientId {
        id: ClientId,
    },
    PlayerCreate {
        entity: Entity,
        id: ClientId,
        translation: Vec3,
        rotation: Vec4,
    },
    PlayerRemove {
        id: ClientId,
    },
    SentBlock {
        chunk_pos: IVec3,
        voxel_pos: [u8; 3],
        block_type: String,
    },
    NetworkedEntities {
        networked_entities: NetworkedEntities,
    },
    LevelData {
        chunk_data: Vec<u8>,
        pos: IVec3,
    },
}
