use std::{
    net::{IpAddr, Ipv4Addr, UdpSocket},
    time::SystemTime,
};

use bevy::prelude::*;
use bevy_renet::renet::{RenetServer, ServerAuthentication, ServerConfig};
use vinox_common::{
    networking::protocol::{server_connection_config, PROTOCOL_ID},
    storage::{blocks::load::load_all_blocks, items::load::item_from_block},
    world::chunks::storage::{BlockTable, ItemTable},
};

pub fn setup_loadables(mut block_table: ResMut<BlockTable>, mut item_table: ResMut<ItemTable>) {
    for block in load_all_blocks() {
        let mut name = block.clone().namespace;
        name.push(':');
        name.push_str(&block.name);
        if let Some(has_item) = block.has_item {
            if has_item {
                item_table.insert(name.clone(), item_from_block(block.clone()));
            }
        }
        block_table.insert(name, block);
    }
}

pub fn new_server(mut commands: Commands) {
    let server_addr = ("127.0.0.1".to_string() + ":25565").parse().unwrap();
    let socket = UdpSocket::bind("0.0.0.0:25565").unwrap();
    let connection_config = server_connection_config();
    let server_config =
        ServerConfig::new(8, PROTOCOL_ID, server_addr, ServerAuthentication::Unsecure);
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    commands.insert_resource(
        RenetServer::new(current_time, server_config, connection_config, socket).unwrap(),
    );
}
