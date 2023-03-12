use super::components::{ClientData, ClientLobby, NetworkMapping, PlayerInfo};
use crate::states::game::world::chunks::{ControlledPlayer, CreateChunkEvent, SetBlockEvent};
use bevy::prelude::*;
use bevy_quinnet::client::*;
use bevy_spectator::Spectator;
use bevy_tweening::{
    lens::{TransformPositionLens, TransformRotationLens},
    *,
};
use std::{io::Cursor, time::Duration};
use vinox_common::{
    ecs::bundles::PlayerBundleBuilder,
    networking::protocol::{ClientMessage, EntityBuffer, ServerMessage},
    world::chunks::storage::RawChunk,
};
use zstd::stream::copy_decode;

#[derive(Component)]
pub struct JustSpawned {
    timer: Timer,
}

#[derive(Component)]
pub struct HighLightCube;

pub fn get_id(
    mut client: ResMut<Client>,
    mut client_data: ResMut<ClientData>,
    mut has_connected: Local<bool>,
) {
    if *has_connected {
    } else {
        while let Some(message) = client
            .connection_mut()
            .try_receive_message::<ServerMessage>()
        {
            if let ServerMessage::ClientId { id } = message {
                client_data.0 = id;
                client
                    .connection_mut()
                    .try_send_message(ClientMessage::Join {
                        user_name: "test".to_string(), //Global resource will be used instead
                        id,
                    });
                *has_connected = true;
            }
        }
    }
}

//TODO: Refactor this is a lot in one function
#[allow(clippy::clone_on_copy)]
#[allow(clippy::too_many_arguments)]
pub fn get_messages(
    mut cmd1: Commands,
    mut cmd2: Commands,
    mut client: ResMut<Client>,
    client_data: Res<ClientData>,
    mut lobby: ResMut<ClientLobby>,
    mut network_mapping: ResMut<NetworkMapping>,
    mut entity_buffer: ResMut<EntityBuffer>,
    player_builder: Res<PlayerBundleBuilder>,
    mut chunk_event: EventWriter<CreateChunkEvent>,
    mut block_event: EventWriter<SetBlockEvent>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    if client_data.0 != 0 {
        while let Some(message) = client
            .connection_mut()
            .try_receive_message::<ServerMessage>()
        {
            match message {
                ServerMessage::PlayerCreate {
                    id,
                    translation,
                    entity,
                    rotation,
                } => {
                    let mut client_entity = cmd1.spawn_empty();
                    if client_data.0 == id {
                        println!("You connected.");
                        cmd2.spawn(PbrBundle {
                            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.001 })),
                            material: materials.add(StandardMaterial {
                                base_color: Color::rgba(1.1, 1.1, 1.1, 1.0),
                                base_color_texture: Some(asset_server.load("outline.png")),
                                alpha_mode: AlphaMode::Blend,
                                unlit: true,
                                ..Default::default()
                            }),
                            transform: Transform::from_translation(
                                translation + Vec3::new(0.0, 0.0, -5.0),
                            ),
                            ..default()
                        })
                        .insert(HighLightCube);

                        client_entity
                            .insert(player_builder.build(translation, id, true))
                            .insert(ControlledPlayer)
                            .insert(JustSpawned {
                                timer: Timer::new(Duration::from_secs(10), TimerMode::Once),
                            });
                    } else {
                        println!("Player {id} connected.");
                        client_entity.insert(player_builder.build(translation, id, false));
                        client_entity.insert(
                            Transform::from_translation(translation)
                                .with_rotation(Quat::from_vec4(rotation)),
                        );
                    }

                    let player_info = PlayerInfo {
                        server_entity: entity,
                        client_entity: client_entity.id(),
                    };
                    lobby.players.insert(id, player_info);
                    network_mapping.0.insert(entity, client_entity.id());
                }
                ServerMessage::PlayerRemove { id } => {
                    println!("Player {id} disconnected.");
                    if let Some(PlayerInfo {
                        server_entity,
                        client_entity,
                    }) = lobby.players.remove(&id)
                    {
                        cmd1.entity(client_entity).despawn();
                        network_mapping.0.remove(&server_entity);
                    }
                }
                ServerMessage::SentBlock {
                    chunk_pos,
                    voxel_pos,
                    block_type,
                } => block_event.send(SetBlockEvent {
                    chunk_pos,
                    voxel_pos: UVec3::new(
                        voxel_pos[0] as u32,
                        voxel_pos[1] as u32,
                        voxel_pos[2] as u32,
                    ),
                    block_type,
                }),
                ServerMessage::NetworkedEntities { networked_entities } => {
                    let arr_len = entity_buffer.entities.len() - 1;
                    entity_buffer.entities.rotate_left(1);
                    entity_buffer.entities[arr_len] = networked_entities;
                }
                ServerMessage::LevelData { chunk_data, pos } => {
                    let mut temp_output = Cursor::new(Vec::new());
                    copy_decode(&chunk_data[..], &mut temp_output).unwrap();
                    let level_data: RawChunk = bincode::deserialize(temp_output.get_ref()).unwrap();
                    chunk_event.send(CreateChunkEvent {
                        raw_chunk: level_data,
                        pos,
                    });
                }
                _ => {}
            }
        }
    }
}

pub fn lerp_new_location(
    mut commands: Commands,
    entity_buffer: ResMut<EntityBuffer>,
    lobby: ResMut<ClientLobby>,
    network_mapping: ResMut<NetworkMapping>,
    client_data: ResMut<ClientData>,
    transform_query: Query<&Transform>,
) {
    for i in 0..entity_buffer.entities[0].entities.len() {
        if let Some(entity) = network_mapping
            .0
            .get(&entity_buffer.entities[0].entities[i])
        {
            let translation = entity_buffer.entities[0].translations[i];
            let rotation = Quat::from_vec4(entity_buffer.entities[0].rotations[i]);
            let transform = Transform {
                translation,
                ..Default::default()
            }
            .with_rotation(rotation);
            if let Some(player_entity) = lobby.players.get(&client_data.0) {
                if player_entity.client_entity != *entity {
                    if let Ok(old_transform) = transform_query.get(*entity) {
                        let tween = Tween::new(
                            EaseFunction::QuadraticIn,
                            Duration::from_millis(25),
                            TransformPositionLens {
                                start: old_transform.translation,
                                end: transform.translation,
                            },
                        )
                        .with_repeat_count(RepeatCount::Finite(1));
                        let tween_rot = Tween::new(
                            EaseFunction::QuadraticIn,
                            Duration::from_millis(25),
                            TransformRotationLens {
                                start: old_transform.rotation,
                                end: transform.rotation,
                            },
                        )
                        .with_repeat_count(RepeatCount::Finite(1));
                        let track = Tracks::new([tween, tween_rot]);
                        commands
                            .get_entity(*entity)
                            .unwrap()
                            .insert(Animator::new(track));
                    }
                } else {
                }
            } else {
                //Different entity rather then player.
            }
        }
    }
}

pub fn client_send_naive_position(
    mut transform_query: Query<&mut Transform, With<ControlledPlayer>>,
    mut camera_query: Query<&mut Transform, (With<Camera>, Without<ControlledPlayer>)>,
    mut client: ResMut<Client>,
) {
    if let Ok(transform) = transform_query.get_single_mut() {
        if let Ok(camera_transform) = camera_query.get_single_mut() {
            client
                .connection_mut()
                .send_message_on(
                    bevy_quinnet::shared::channel::ChannelId::Unreliable,
                    ClientMessage::Position {
                        player_pos: transform.translation,
                        player_rot: Vec4::from(camera_transform.rotation),
                    },
                )
                .unwrap();
        }
    }
}

// TODO: Have a more elegant way to wait on loading section or by actually waiting till all the intial chunks are loaded
pub fn wait_for_chunks(
    mut just_spawned_query: Query<(&mut JustSpawned, Entity, &mut Transform)>,
    time: Res<Time>,
    mut commands: Commands,
) {
    if let Ok((mut just_spawned, entity, mut player_transform)) =
        just_spawned_query.get_single_mut()
    {
        just_spawned.timer.tick(time.delta());
        if just_spawned.timer.finished() {
            commands.entity(entity).remove::<JustSpawned>();
        } else {
            player_transform.translation = Vec3::new(0.0, 110.0, 0.0);
        }
    }
}
