use std::io::Cursor;

use rustc_data_structures::stable_set::FxHashSet;

use bevy::prelude::*;
use bevy_quinnet::server::*;
use vinox_common::{
    ecs::bundles::{ClientName, Inventory, PlayerBundleBuilder},
    networking::protocol::{ClientMessage, NetworkedEntities, Player, ServerMessage},
    world::chunks::{
        ecs::{ChunkComp, CurrentChunks},
        positions::world_to_chunk,
    },
};
use zstd::stream::copy_encode;

use crate::game::world::{
    chunk::{ChunkManager, LoadPoint},
    storage::ChunksToSave,
};

use super::components::{SentChunks, ServerLobby};

pub fn connections(
    mut server: ResMut<Server>,
    lobby: Res<ServerLobby>,
    mut connection_events: EventReader<ConnectionEvent>,
) {
    for client in connection_events.iter() {
        // Refuse connection once we already have two players
        if lobby.players.len() >= 8 {
            server.endpoint_mut().disconnect_client(client.id).unwrap();
        } else {
            server
                .endpoint_mut()
                .try_send_message(client.id, ServerMessage::ClientId { id: client.id });
        }
    }
}

// So i dont forget this is actually fine this is just receiving we are just sending out response packets which dont need to be limited since they only happen once per receive
#[allow(clippy::too_many_arguments)]
pub fn get_messages(
    mut server: ResMut<Server>,
    mut commands: Commands,
    mut lobby: ResMut<ServerLobby>,
    mut players: Query<(Entity, &Player, &Transform, &ClientName)>,
    player_builder: Res<PlayerBundleBuilder>,
    mut chunks: Query<&mut ChunkComp>,
    current_chunks: Res<CurrentChunks>,
    mut chunks_to_save: ResMut<ChunksToSave>,
) {
    let endpoint = server.endpoint_mut();
    for client_id in endpoint.clients() {
        while let Some(message) = endpoint.try_receive_message_from::<ClientMessage>(client_id) {
            match message {
                ClientMessage::Join { id, user_name } => {
                    println!("Player {user_name} connected.");

                    // Initialize other players for this new client
                    for (entity, player, transform, client_name) in players.iter_mut() {
                        endpoint.try_send_message(
                            id,
                            ServerMessage::PlayerCreate {
                                id: player.id,
                                entity,
                                translation: transform.translation,
                                yaw: transform.rotation.to_euler(EulerRot::XYZ).1,
                                head_pitch: transform.rotation.to_euler(EulerRot::XYZ).0,
                                user_name: (*client_name).clone(),
                                init: false,
                                inventory: Box::<Inventory>::default(), // TODO: Load from database
                            },
                        );
                    }

                    // Spawn new player
                    let transform = Transform::from_xyz(0.0, 115.0, 0.0);
                    let player_entity = commands
                        .spawn(player_builder.build(
                            transform.translation,
                            id,
                            false,
                            user_name.clone(),
                        ))
                        .insert(SentChunks {
                            chunks: FxHashSet::default(),
                        })
                        .insert(LoadPoint(world_to_chunk(transform.translation)))
                        .id();
                    lobby.players.insert(id, player_entity);

                    endpoint.try_broadcast_message(&ServerMessage::PlayerCreate {
                        id,
                        entity: player_entity,
                        translation: transform.translation,
                        yaw: transform.rotation.to_euler(EulerRot::XYZ).1,
                        head_pitch: transform.rotation.to_euler(EulerRot::XYZ).0,
                        user_name,
                        init: true,
                        inventory: Box::<Inventory>::default(),
                    });
                }
                ClientMessage::Leave { id } => {
                    println!("Player {id} disconnected.");
                    if let Some(player_entity) = lobby.players.remove(&id) {
                        commands.entity(player_entity).despawn();
                    }

                    endpoint.try_broadcast_message(&ServerMessage::PlayerRemove { id });
                }
                ClientMessage::Position {
                    player_pos,
                    yaw,
                    head_pitch: _,
                } => {
                    if let Some(player_entity) = lobby.players.get(&client_id) {
                        commands.entity(*player_entity).insert(
                            Transform::from_translation(player_pos)
                                .with_rotation(Quat::from_euler(EulerRot::XYZ, 0.0, yaw, 0.0)),
                        );
                    }
                }

                ClientMessage::SentBlock {
                    chunk_pos,
                    voxel_pos,
                    block_type,
                } => {
                    if let Some(chunk_entity) = current_chunks.get_entity(chunk_pos) {
                        if let Ok(mut chunk) = chunks.get_mut(chunk_entity) {
                            chunk.chunk_data.add_block_state(&block_type);
                            chunk.chunk_data.set_block(
                                UVec3::new(
                                    voxel_pos[0] as u32,
                                    voxel_pos[1] as u32,
                                    voxel_pos[2] as u32,
                                ),
                                &block_type,
                            );
                            chunks_to_save.push((*chunk.pos, chunk.chunk_data.clone()));
                            endpoint.try_broadcast_message(ServerMessage::SentBlock {
                                chunk_pos,
                                voxel_pos,
                                block_type,
                            });
                        }
                    }
                }
                ClientMessage::ChatMessage { message } => {
                    if let Some(player_entity) = lobby.players.get(&client_id) {
                        if let Ok((_, _, _, username)) = players.get(*player_entity) {
                            endpoint.try_broadcast_message_on(
                                bevy_quinnet::shared::channel::ChannelId::OrderedReliable(1),
                                ServerMessage::ChatMessage {
                                    user_name: (*username).clone(),
                                    message,
                                    id: client_id,
                                },
                            );
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

#[allow(clippy::type_complexity)]
//This would eventually take in any networkedentity for now just player
pub fn send_entities(mut server: ResMut<Server>, query: Query<(Entity, &Transform)>) {
    let mut networked_entities = NetworkedEntities::default();
    for (entity, transform) in query.iter() {
        networked_entities.entities.push(entity);
        networked_entities.translations.push(transform.translation);
        networked_entities
            .yaws
            .push(transform.rotation.to_euler(EulerRot::XYZ).1);
    }
    server.endpoint_mut().try_broadcast_message_on(
        bevy_quinnet::shared::channel::ChannelId::Unreliable,
        ServerMessage::NetworkedEntities { networked_entities },
    );
}

pub fn send_chunks(
    mut commands: Commands,
    mut server: ResMut<Server>,
    lobby: ResMut<ServerLobby>,
    mut players: Query<(&Transform, &mut SentChunks), With<Player>>,
    mut chunk_manager: ChunkManager,
) {
    let endpoint = server.endpoint_mut();
    for client_id in endpoint.clients() {
        if let Some(player_entity) = lobby.players.get(&client_id) {
            if let Ok((player_transform, mut sent_chunks)) = players.get_mut(*player_entity) {
                let chunk_pos = world_to_chunk(player_transform.translation);
                let load_point = LoadPoint(chunk_pos);
                commands.entity(*player_entity).insert(load_point.clone());
                for chunk in chunk_manager.get_chunks_around_chunk(chunk_pos, &sent_chunks) {
                    let raw_chunk = chunk.chunk_data.clone();
                    if let Ok(raw_chunk_bin) = bincode::serialize(&raw_chunk) {
                        let mut final_chunk = Cursor::new(raw_chunk_bin);
                        let mut output = Cursor::new(Vec::new());
                        copy_encode(&mut final_chunk, &mut output, 0).unwrap();
                        if server
                            .endpoint_mut()
                            .send_message(
                                client_id,
                                ServerMessage::LevelData {
                                    chunk_data: output.get_ref().clone(),
                                    pos: *chunk.pos,
                                },
                            )
                            .is_ok()
                        {
                            sent_chunks.chunks.insert(*chunk.pos);
                        }
                    }
                }
            }
        }
    }
}
