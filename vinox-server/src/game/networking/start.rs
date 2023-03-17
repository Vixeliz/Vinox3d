use std::net::{IpAddr, Ipv4Addr};

use bevy::prelude::*;
use bevy_quinnet::server::*;
use vinox_common::{storage::blocks::load::load_all_blocks, world::chunks::storage::BlockTable};

pub fn setup_loadables(mut block_table: ResMut<BlockTable>) {
    for block in load_all_blocks() {
        let mut name = block.clone().namespace;
        name.push(':');
        name.push_str(&block.name);
        block_table.insert(name, block);
    }
}

pub fn new_server(mut server: ResMut<Server>) {
    server
        .start_endpoint(
            ServerConfiguration::from_ip(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 25565),
            certificate::CertificateRetrievalMode::GenerateSelfSigned {
                server_hostname: "vinox".to_string(), //TODO: Change to computer hostname
            },
        )
        .unwrap();
    server
        .endpoint_mut()
        .set_default_channel(bevy_quinnet::shared::channel::ChannelId::UnorderedReliable);
}
