use std::time::Duration;

use bevy::prelude::*;
use bevy_renet::renet::{
    ChannelConfig, ReliableChannelConfig, RenetConnectionConfig, UnreliableChannelConfig,
};

#[derive(Resource, Deref, DerefMut)]
pub struct NetworkIP(pub String);

use serde::{Deserialize, Serialize};

use crate::{ecs::bundles::Inventory, world::chunks::storage::BlockData};

pub const PROTOCOL_ID: u64 = 1;
pub const RELIABLE_CHANNEL_MAX_LENGTH: u64 = 10240;

#[derive(Component)]
pub struct NetworkedEntity;

#[derive(Debug, Component, Default)]
pub struct Player {
    pub id: u64,
}

pub enum ClientChannel {
    Syncs,
    Messages,
    Orders,
}

pub enum ServerChannel {
    Messages,
    Syncs,
    Orders,
    Level,
}

// Networking related
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct NetworkedEntities {
    pub entities: Vec<Entity>,
    pub translations: Vec<Vec3>,
    pub yaws: Vec<f32>,
    pub head_pitchs: Vec<f32>,
}

#[derive(Default, Resource)]
pub struct EntityBuffer {
    pub entities: [NetworkedEntities; 30],
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ClientSync {
    Position {
        player_pos: Vec3,
        yaw: f32,
        head_pitch: f32,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ClientOrdered {
    ChatMessage { message: String },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LevelData {
    pub chunk_data: Vec<u8>,
    pub pos: IVec3,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ClientMessage {
    Interact {
        entity: Entity,
        attack: bool,
    },

    SentBlock {
        chunk_pos: IVec3,
        voxel_pos: [u8; 3],
        block_type: BlockData,
    },
    Join {
        user_name: String, //TODO: Make sure client doesn't send same user_name as another. Also add some sort of password system
    },
    Leave {
        id: u64,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ServerOrdered {
    ChatMessage {
        user_name: String,
        message: String,
        id: u64,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ServerSync {
    NetworkedEntities {
        networked_entities: NetworkedEntities,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ServerMessage {
    PlayerCreate {
        entity: Entity,
        id: u64,
        translation: Vec3,
        yaw: f32,
        head_pitch: f32,
        user_name: String,
        init: bool,
        inventory: Box<Inventory>,
    },
    PlayerRemove {
        id: u64,
    },
    SentBlock {
        chunk_pos: IVec3,
        voxel_pos: [u8; 3],
        block_type: BlockData,
    },
}

impl From<ClientChannel> for u8 {
    fn from(channel_id: ClientChannel) -> Self {
        match channel_id {
            ClientChannel::Messages => 0,
            ClientChannel::Syncs => 1,
            ClientChannel::Orders => 2,
        }
    }
}

impl ClientChannel {
    pub fn channels_config() -> Vec<ChannelConfig> {
        vec![
            ReliableChannelConfig {
                channel_id: Self::Messages.into(),
                message_resend_time: Duration::ZERO,
                ..Default::default()
            }
            .into(),
            UnreliableChannelConfig {
                channel_id: Self::Syncs.into(),
                ..Default::default()
            }
            .into(),
            ReliableChannelConfig {
                channel_id: Self::Orders.into(),
                message_resend_time: Duration::ZERO,
                ordered: true,
                ..Default::default()
            }
            .into(),
        ]
    }
}

impl From<ServerChannel> for u8 {
    fn from(channel_id: ServerChannel) -> Self {
        match channel_id {
            ServerChannel::Messages => 0,
            ServerChannel::Syncs => 1,
            ServerChannel::Orders => 2,
            ServerChannel::Level => 3,
        }
    }
}

impl ServerChannel {
    pub fn channels_config() -> Vec<ChannelConfig> {
        vec![
            ReliableChannelConfig {
                channel_id: Self::Messages.into(),
                message_resend_time: Duration::from_millis(200),
                ..Default::default()
            }
            .into(),
            UnreliableChannelConfig {
                channel_id: Self::Syncs.into(),
                sequenced: true, // We don't care about old positions
                ..Default::default()
            }
            .into(),
            ReliableChannelConfig {
                channel_id: Self::Orders.into(),
                message_resend_time: Duration::from_millis(200),
                ordered: true,
                ..Default::default()
            }
            .into(),
            ReliableChannelConfig {
                channel_id: Self::Level.into(),
                message_resend_time: Duration::ZERO,
                max_message_size: RELIABLE_CHANNEL_MAX_LENGTH,
                packet_budget: RELIABLE_CHANNEL_MAX_LENGTH * 2,
                ..Default::default()
            }
            .into(),
        ]
    }
}

pub fn client_connection_config() -> RenetConnectionConfig {
    RenetConnectionConfig {
        send_channels_config: ClientChannel::channels_config(),
        receive_channels_config: ServerChannel::channels_config(),
        ..Default::default()
    }
}

pub fn server_connection_config() -> RenetConnectionConfig {
    RenetConnectionConfig {
        send_channels_config: ServerChannel::channels_config(),
        receive_channels_config: ClientChannel::channels_config(),
        ..Default::default()
    }
}
