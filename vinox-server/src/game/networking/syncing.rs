use std::io::Cursor;

use bevy_renet::renet::{RenetServer, ServerEvent};
use rand::seq::SliceRandom;
use rustc_data_structures::stable_set::FxHashSet;

use bevy::prelude::*;
use vinox_common::{
    ecs::bundles::{ClientName, Inventory, PlayerBundleBuilder},
    networking::protocol::{
        ClientChannel, ClientMessage, ClientOrdered, ClientSync, LevelData, NetworkedEntities,
        Player, ServerChannel, ServerMessage, ServerOrdered, ServerSync,
    },
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

// So i dont forget this is actually fine this is just receiving we are just sending out response packets which dont need to be limited since they only happen once per receive
#[allow(clippy::too_many_arguments)]
pub fn get_messages(
    mut server: ResMut<RenetServer>,
    mut server_events: EventReader<ServerEvent>,
    mut commands: Commands,
    mut lobby: ResMut<ServerLobby>,
    mut players: Query<(Entity, &Player, &Transform, &ClientName)>,
    player_builder: Res<PlayerBundleBuilder>,
    mut chunks: Query<&mut ChunkComp>,
    current_chunks: Res<CurrentChunks>,
    mut chunks_to_save: ResMut<ChunksToSave>,
) {
    for message in server_events.iter() {
        match message {
            ServerEvent::ClientConnected(id, _) => {
                let player_entity = commands.spawn_empty().id();
                lobby.players.insert(*id, player_entity);
            }
            ServerEvent::ClientDisconnected(id) => {
                if let Some(player_entity) = lobby.players.remove(&id) {
                    commands.entity(player_entity).despawn();
                }

                server.broadcast_message(
                    ServerChannel::Messages,
                    bincode::serialize(&ServerMessage::PlayerRemove { id: *id }).unwrap(),
                );
            }
        }
    }
    for client_id in server.clients_id().into_iter() {
        while let Some(message) = server.receive_message(client_id, ClientChannel::Messages) {
            let message: ClientMessage = bincode::deserialize(&message).unwrap();
            match message {
                ClientMessage::Join { user_name } => {
                    println!("Player {user_name} connected.");

                    // Initialize other players for this new client
                    for (entity, player, transform, client_name) in players.iter_mut() {
                        server.send_message(
                            client_id,
                            ServerChannel::Messages,
                            bincode::serialize(&ServerMessage::PlayerCreate {
                                id: player.id,
                                entity,
                                translation: transform.translation,
                                yaw: transform.rotation.to_euler(EulerRot::XYZ).1,
                                head_pitch: transform.rotation.to_euler(EulerRot::XYZ).0,
                                user_name: (*client_name).clone(),
                                init: false,
                                inventory: Box::<Inventory>::default(), // TODO: Load from database
                            })
                            .unwrap(),
                        );
                    }

                    // Spawn new player
                    let transform = Transform::from_xyz(0.0, 115.0, 0.0);
                    let player_entity = lobby.players.get(&client_id).unwrap();
                    commands
                        .entity(*player_entity)
                        .insert(player_builder.build(
                            transform.translation,
                            client_id,
                            false,
                            user_name.clone(),
                        ))
                        .insert(SentChunks {
                            chunks: FxHashSet::default(),
                        })
                        .insert(LoadPoint(world_to_chunk(transform.translation)));

                    server.broadcast_message(
                        ServerChannel::Messages,
                        bincode::serialize(&ServerMessage::PlayerCreate {
                            id: client_id,
                            entity: *player_entity,
                            translation: transform.translation,
                            yaw: transform.rotation.to_euler(EulerRot::XYZ).1,
                            head_pitch: transform.rotation.to_euler(EulerRot::XYZ).0,
                            user_name,
                            init: true,
                            inventory: Box::<Inventory>::default(),
                        })
                        .unwrap(),
                    );
                }
                ClientMessage::Leave { id } => {
                    println!("Player {id} disconnected.");
                    if let Some(player_entity) = lobby.players.remove(&id) {
                        commands.entity(player_entity).despawn();
                    }

                    server.broadcast_message(
                        ServerChannel::Messages,
                        bincode::serialize(&ServerMessage::PlayerRemove { id }).unwrap(),
                    );
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
                            server.broadcast_message(
                                ServerChannel::Messages,
                                bincode::serialize(&ServerMessage::SentBlock {
                                    chunk_pos,
                                    voxel_pos,
                                    block_type,
                                })
                                .unwrap(),
                            );
                        }
                    }
                }
                _ => {}
            }
            while let Some(message) = server.receive_message(client_id, ClientChannel::Syncs) {
                let message: ClientSync = bincode::deserialize(&message).unwrap();
                match message {
                    ClientSync::Position {
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
                }
            }
            while let Some(message) = server.receive_message(client_id, ClientChannel::Orders) {
                let message: ClientOrdered = bincode::deserialize(&message).unwrap();
                match message {
                    ClientOrdered::ChatMessage { message } => {
                        if let Some(player_entity) = lobby.players.get(&client_id) {
                            if let Ok((_, _, _, username)) = players.get(*player_entity) {
                                server.broadcast_message(
                                    ServerChannel::Orders,
                                    bincode::serialize(&ServerOrdered::ChatMessage {
                                        user_name: (*username).clone(),
                                        message,
                                        id: client_id,
                                    })
                                    .unwrap(),
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}

#[allow(clippy::type_complexity)]
//This would eventually take in any networkedentity for now just player
pub fn send_entities(mut server: ResMut<RenetServer>, query: Query<(Entity, &Transform)>) {
    let mut networked_entities = NetworkedEntities::default();
    for (entity, transform) in query.iter() {
        networked_entities.entities.push(entity);
        networked_entities.translations.push(transform.translation);
        networked_entities
            .yaws
            .push(transform.rotation.to_euler(EulerRot::XYZ).1);
    }
    server.broadcast_message(
        ServerChannel::Syncs,
        bincode::serialize(&ServerSync::NetworkedEntities { networked_entities }).unwrap(),
    );
}

pub fn send_chunks(
    mut commands: Commands,
    mut server: ResMut<RenetServer>,
    lobby: ResMut<ServerLobby>,
    mut players: Query<(&Transform, &mut SentChunks), With<Player>>,
    mut chunk_manager: ChunkManager,
) {
    let mut rng = rand::thread_rng();
    for client_id in server.clients_id().into_iter() {
        if let Some(player_entity) = lobby.players.get(&client_id) {
            if let Ok((player_transform, mut sent_chunks)) = players.get_mut(*player_entity) {
                let chunk_pos = world_to_chunk(player_transform.translation);
                let load_point = LoadPoint(chunk_pos);
                commands.entity(*player_entity).insert(load_point.clone());
                for chunk in chunk_manager
                    .get_chunks_around_chunk(chunk_pos, &sent_chunks)
                    .choose_multiple(&mut rng, 64)
                {
                    let raw_chunk = chunk.chunk_data.clone();
                    if let Ok(raw_chunk_bin) = bincode::serialize(&raw_chunk) {
                        let mut final_chunk = Cursor::new(raw_chunk_bin);
                        let mut output = Cursor::new(Vec::new());
                        copy_encode(&mut final_chunk, &mut output, 0).unwrap();
                        server.send_message(
                            client_id,
                            ServerChannel::Level,
                            bincode::serialize(&LevelData {
                                chunk_data: output.get_ref().clone(),
                                pos: *chunk.pos,
                            })
                            .unwrap(),
                        );
                        sent_chunks.chunks.insert(*chunk.pos);
                    }
                }
            }
        }
    }
}
