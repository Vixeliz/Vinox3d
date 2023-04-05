use std::io::Cursor;

use rand::{
    Rng,
};
use rustc_data_structures::stable_set::FxHashSet;

use bevy::{app::AppExit, prelude::*};
use bevy_quinnet::server::*;
use vinox_common::{
    ecs::bundles::{ClientName, Inventory, PlayerBundleBuilder},
    networking::protocol::{ClientMessage, NetworkedEntities, Player, ServerMessage},
    world::chunks::{
        ecs::{ChunkManager, CurrentChunks, LoadPoint, NeedsChunkData, PrepassChunk, SentChunks},
        positions::{ChunkPos, RelativeVoxelPos, VoxelPos},
        storage::{BlockTable, ChunkData},
    },
};
use zstd::stream::copy_encode;

use crate::game::world::{chunk::GeneratingChunk, storage::ChunksToSave};

use super::components::{ChunkLimit, LocalGame, ServerLobby};

pub fn connections(
    mut commands: Commands,
    mut server: ResMut<Server>,
    mut lobby: ResMut<ServerLobby>,
    mut connection_events: EventReader<ConnectionEvent>,
    mut connection_lost_events: EventReader<ConnectionLostEvent>,
    local_game: Res<LocalGame>,
    mut exit: EventWriter<AppExit>,
) {
    for client in connection_lost_events.iter() {
        let id = client.id;
        if **local_game {
            exit.send(AppExit);
        } else {
            println!("Player {id} disconnected.");
            if let Some(player_entity) = lobby.players.remove(&id) {
                commands.entity(player_entity).despawn();
            }

            server
                .endpoint_mut()
                .try_broadcast_message(&ServerMessage::PlayerRemove { id });
        }
    }
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
    mut chunks: Query<&mut ChunkData>,
    current_chunks: Res<CurrentChunks>,
    mut chunks_to_save: ResMut<ChunksToSave>,
    _block_table: Res<BlockTable>,
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
                    let transform = Transform::from_xyz(0.0, 75.0, 0.0);
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
                        .insert(VoxelPos::default())
                        .insert(LoadPoint::default())
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
                    if let Some(chunk_entity) = current_chunks.get_entity(ChunkPos(chunk_pos)) {
                        if let Ok(mut chunk) = chunks.get_mut(chunk_entity) {
                            chunk.set(
                                RelativeVoxelPos::new(
                                    voxel_pos[0] as u32,
                                    voxel_pos[1] as u32,
                                    voxel_pos[2] as u32,
                                ),
                                block_type.clone(),
                            );
                            chunks_to_save.push((ChunkPos(chunk_pos), chunk.to_raw()));
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

pub fn sync_voxel_pos(mut players: Query<(&mut VoxelPos, &Transform)>) {
    for (mut voxel_pos, transform) in players.iter_mut() {
        *voxel_pos = VoxelPos::from(transform.translation);
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
    _commands: Commands,
    mut server: ResMut<Server>,
    lobby: ResMut<ServerLobby>,
    mut players: Query<(&LoadPoint, &mut SentChunks), With<Player>>,
    chunk_manager: ChunkManager,
    is_ready: Query<(
        Without<NeedsChunkData>,
        Without<GeneratingChunk>,
        Without<PrepassChunk>,
    )>,
    chunk_limit: Res<ChunkLimit>,
) {
    let mut rng = rand::thread_rng();
    let endpoint = server.endpoint_mut();
    for client_id in endpoint.clients() {
        if let Some(player_entity) = lobby.players.get(&client_id) {
            if let Ok((load_point, mut sent_chunks)) = players.get_mut(*player_entity) {
                let mut count = **chunk_limit;
                while count != 0 {
                    let x = rng.gen_range(-load_point.horizontal..=load_point.horizontal);
                    let y = rng.gen_range(-load_point.vertical..=load_point.vertical);
                    let z = rng.gen_range(-load_point.horizontal..=load_point.horizontal);
                    let (x, y, z) = (
                        x + load_point.chunk_pos.x,
                        y + load_point.chunk_pos.y,
                        z + load_point.chunk_pos.z,
                    );
                    let total_chunks = ((load_point.horizontal * 2) + 1) // Account for middle(0)
                            * ((load_point.vertical * 2) + 1)
                            * ((load_point.horizontal
                            * 2) + 1);

                    if sent_chunks.chunks.len() as i32 == total_chunks {
                        // println!(
                        //     "Total chunks: {total_chunks}, Sent chunks: {}",
                        //     sent_chunks.chunks.len()
                        // );
                        break;
                    }

                    if sent_chunks.chunks.contains(&ChunkPos::new(x, y, z)) {
                        continue;
                    }
                    count -= 1;
                    if let Some(chunk_entity) = chunk_manager
                        .current_chunks
                        .get_entity(ChunkPos::new(x, y, z))
                    {
                        if is_ready.get(chunk_entity).is_ok() {
                            if let Some(chunk) = chunk_manager.get_chunk(chunk_entity) {
                                let raw_chunk = chunk.to_raw();
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
                                                pos: IVec3::new(x, y, z),
                                            },
                                        )
                                        .is_ok()
                                    {
                                        sent_chunks.chunks.insert(ChunkPos::new(x, y, z));
                                    }
                                }
                            }
                        }
                    }
                }
                // for (chunk_entity, chunk_pos) in chunk_manager
                //     .current_chunks
                //     .get_entities(&[*load_point])
                //     .iter()
                //     .choose_multiple(&mut rng, **chunk_limit)
                // {
                //     if has_data.get(*chunk_entity).is_ok() {
                //         if let Some(chunk) = chunk_manager.get_chunk(*chunk_entity) {
                //             let raw_chunk = chunk.to_raw();
                //             if let Ok(raw_chunk_bin) = bincode::serialize(&raw_chunk) {
                //                 let mut final_chunk = Cursor::new(raw_chunk_bin);
                //                 let mut output = Cursor::new(Vec::new());
                //                 copy_encode(&mut final_chunk, &mut output, 0).unwrap();
                //                 if server
                //                     .endpoint_mut()
                //                     .send_message(
                //                         client_id,
                //                         ServerMessage::LevelData {
                //                             chunk_data: output.get_ref().clone(),
                //                             pos: **chunk_pos,
                //                         },
                //                     )
                //                     .is_ok()
                //                 {
                //                     sent_chunks.chunks.insert(*chunk_pos);
                //                 }
                //             }
                //         }
                //     }
                // }
            }
        }
    }
}
