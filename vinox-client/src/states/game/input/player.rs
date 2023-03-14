use std::f32::consts::{FRAC_PI_2, PI};

use bevy::{
    input::mouse::MouseMotion,
    prelude::*,
    render::{camera::CameraProjection, primitives::Frustum},
    window::{CursorGrabMode, PrimaryWindow},
};
use bevy_quinnet::client::Client;
use vinox_common::{
    collision::raycast::raycast_world,
    networking::protocol::ClientMessage,
    world::chunks::{
        ecs::{ChunkComp, CurrentChunks},
        positions::{voxel_to_world, world_to_chunk, world_to_global_voxel, world_to_voxel},
        storage::{BlockData, BlockTable, CHUNK_SIZE, CHUNK_SIZE_ARR, HORIZONTAL_DISTANCE},
    },
};

use crate::states::game::{
    networking::syncing::HighLightCube, rendering::meshing::PriorityMesh,
    world::chunks::ControlledPlayer,
};

#[derive(Component)]
pub struct FPSCamera {
    pub phi: f32,
    pub theta: f32,
    pub velocity: Vec3,
}

impl Default for FPSCamera {
    fn default() -> Self {
        FPSCamera {
            phi: 0.0,
            theta: FRAC_PI_2,
            velocity: Vec3::ZERO,
        }
    }
}

pub fn spawn_camera(
    mut commands: Commands,
    player_entity: Query<Entity, With<ControlledPlayer>>,
    mut local: Local<bool>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    if *local {
        return;
    }
    if let Ok(player_entity) = player_entity.get_single() {
        let Ok(mut window) = windows.get_single_mut() else {
            return;
        };
        window.cursor.grab_mode = CursorGrabMode::Locked;
        window.cursor.visible = false;

        *local = true;
        let camera = {
            let perspective_projection = PerspectiveProjection {
                fov: std::f32::consts::PI / 1.8,
                near: 0.001,
                far: 1000.0,
                aspect_ratio: 1.0,
            };
            let view_projection = perspective_projection.get_projection_matrix();
            let frustum = Frustum::from_view_projection(
                &view_projection,
                // &Vec3::ZERO,
                // &Vec3::Z,
                // perspective_projection.far(),
            );
            Camera3dBundle {
                projection: Projection::Perspective(perspective_projection),
                frustum,
                ..default()
            }
        };
        commands.insert_resource(ClearColor(Color::rgba(0.5, 0.8, 0.9, 1.0)));
        commands.entity(player_entity).with_children(|c| {
            c.spawn((
                GlobalTransform::default(),
                Transform::from_xyz(0.0, 1.0, 0.0),
            ));
            c.spawn((
                FPSCamera::default(),
                camera,
                FogSettings {
                    color: Color::rgba(0.5, 0.8, 0.9, 1.0),
                    directional_light_color: Color::WHITE,
                    directional_light_exponent: 30.0,
                    falloff: FogFalloff::Linear {
                        start: ((HORIZONTAL_DISTANCE - 2) * CHUNK_SIZE as i32) as f32,
                        end: ((HORIZONTAL_DISTANCE + 2) * CHUNK_SIZE as i32) as f32,
                    },
                },
            ));
        });
    }
}

#[derive(Resource)]
pub struct MouseSensitivity(pub f32);

#[allow(clippy::too_many_arguments)]
pub fn movement_input(
    mut player: Query<&mut FPSCamera>,
    mut player_position: Query<&mut Transform, With<ControlledPlayer>>,
    mut camera_transform: Query<&mut Transform, (With<Camera>, Without<ControlledPlayer>)>,
    mut mouse_events: EventReader<MouseMotion>,
    mouse_sensitivity: Res<MouseSensitivity>,
    key_events: Res<Input<KeyCode>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    time: Res<Time>,
    mut stationary_frames: Local<i32>,
    current_chunks: Res<CurrentChunks>,
) {
    if let Ok(mut translation) = player_position.get_single_mut() {
        if current_chunks
            .get_entity(world_to_chunk(translation.translation))
            .is_none()
        {
            return;
        }

        let Ok(window) = windows.get_single() else {
            return
        };
        let mut movement = Vec3::default();
        if let Ok(mut fps_camera) = player.get_single_mut() {
            let mut transform = camera_transform.single_mut();

            if window.cursor.grab_mode == CursorGrabMode::Locked {
                for MouseMotion { delta } in mouse_events.iter() {
                    fps_camera.phi += delta.x * mouse_sensitivity.0 * 0.003;
                    fps_camera.theta = (fps_camera.theta + delta.y * mouse_sensitivity.0 * 0.003)
                        .clamp(0.00005, PI - 0.00005);
                }

                if key_events.pressed(KeyCode::W) {
                    let mut fwd = transform.forward();
                    fwd.y = 0.0;
                    let fwd = fwd.normalize();
                    movement += fwd;
                }
                if key_events.pressed(KeyCode::A) {
                    movement += transform.left()
                }
                if key_events.pressed(KeyCode::D) {
                    movement += transform.right()
                }
                if key_events.pressed(KeyCode::S) {
                    let mut back = transform.back();
                    back.y = 0.0;
                    let back = back.normalize();
                    movement += back;
                }

                if key_events.pressed(KeyCode::Space) && *stationary_frames > 2 {
                    *stationary_frames = 0;
                    fps_camera.velocity.y = 12.0;
                }
                if key_events.pressed(KeyCode::C) {
                    fps_camera.velocity.y = -5.0;
                } else {
                    fps_camera.velocity.y = 0.0;
                }
            }

            movement = movement.normalize_or_zero();

            if fps_camera.velocity.y.abs() < 0.001 && *stationary_frames < 10 {
                *stationary_frames += 4;
            } else if *stationary_frames >= 0 {
                *stationary_frames -= 1;
            }

            let y = fps_camera.velocity.y;
            fps_camera.velocity.y = 0.0;
            fps_camera.velocity = movement;
            if key_events.pressed(KeyCode::LShift) {
                fps_camera.velocity *= 10.0;
            } else {
                fps_camera.velocity *= 5.0;
            }
            fps_camera.velocity.y = y;
            let chunk_pos = world_to_chunk(translation.translation);

            if current_chunks.get_entity(chunk_pos).is_none() {
                return;
            }

            let looking_at = Vec3::new(
                10.0 * fps_camera.phi.cos() * fps_camera.theta.sin(),
                10.0 * fps_camera.theta.cos(),
                10.0 * fps_camera.phi.sin() * fps_camera.theta.sin(),
            );

            transform.look_at(looking_at, Vec3::new(0.0, 1.0, 0.0));
            translation.translation += fps_camera.velocity * 1.5 * time.delta().as_secs_f32();
            // fps_camera.velocity.y -= 35.0 * time.delta().as_secs_f32().clamp(0.0, 0.1);
        }
    }
}

// HEAVILY TEMPORARY BOYFRIEND WANTED ITEMS TO BUILD WITH
#[derive(Default, Clone)]
pub enum CurrentItem {
    #[default]
    Grass,
    Dirt,
    Cobblestone,
    Glass,
}

#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
pub fn interact(
    mut commands: Commands,
    mut chunks: Query<&mut ChunkComp>,
    mouse_button_input: Res<Input<MouseButton>>,
    keys: Res<Input<KeyCode>>,
    current_chunks: Res<CurrentChunks>,
    camera_query: Query<&GlobalTransform, With<Camera>>,
    mut client: ResMut<Client>,
    player_position: Query<&Transform, With<ControlledPlayer>>,
    mut cube_position: Query<
        (&mut Transform, &mut Visibility),
        (With<HighLightCube>, Without<ControlledPlayer>),
    >,
    mut current_item: Local<CurrentItem>,
    block_table: Res<BlockTable>,
    // mut materials: ResMut<Assets<StandardMaterial>>,
    // mut meshes: ResMut<Assets<Mesh>>,
) {
    let item_string = match current_item.clone() {
        CurrentItem::Grass => BlockData::new("vinox".to_string(), "grass".to_string()),
        CurrentItem::Dirt => BlockData::new("vinox".to_string(), "dirt".to_string()),
        CurrentItem::Cobblestone => BlockData::new("vinox".to_string(), "cobblestone".to_string()),
        CurrentItem::Glass => BlockData::new("vinox".to_string(), "glass".to_string()),
    };

    for key in keys.get_just_pressed() {
        match key {
            KeyCode::Key1 => *current_item = CurrentItem::Dirt,
            KeyCode::Key2 => *current_item = CurrentItem::Grass,
            KeyCode::Key3 => *current_item = CurrentItem::Glass,
            KeyCode::Key4 => *current_item = CurrentItem::Cobblestone,
            _ => {}
        }
    }

    let mouse_left = mouse_button_input.just_pressed(MouseButton::Left);
    let mouse_right = mouse_button_input.just_pressed(MouseButton::Right);
    if let Ok(player_transform) = player_position.get_single() {
        if let Ok(camera_transform) = camera_query.get_single() {
            // Then cast the ray.
            let hit = raycast_world(
                camera_transform.translation(),
                camera_transform.forward(),
                50.0,
                &chunks,
                &current_chunks,
                &block_table,
            );
            if let Some((chunk_pos, voxel_pos, _normal)) = hit {
                let point = voxel_to_world(voxel_pos, chunk_pos);

                if let Some(chunk_entity) = current_chunks.get_entity(chunk_pos) {
                    if let Ok((mut block_transform, mut block_visibility)) =
                        cube_position.get_single_mut()
                    {
                        if *block_visibility == Visibility::Hidden {
                            *block_visibility = Visibility::Visible;
                        }
                        block_transform.translation = point + Vec3::splat(0.5);
                    }
                    if mouse_left || mouse_right {
                        if let Ok(mut chunk) = chunks.get_mut(chunk_entity) {
                            if mouse_right {
                                if (point.x <= player_transform.translation.x - 0.5
                                    || point.x >= player_transform.translation.x + 0.5)
                                    || (point.z <= player_transform.translation.z - 0.5
                                        || point.z >= player_transform.translation.z + 0.5)
                                    || (point.y <= player_transform.translation.y - 1.0
                                        || point.y >= player_transform.translation.y + 1.0)
                                {
                                    chunk.chunk_data.add_block_state(&item_string);
                                    chunk.chunk_data.set_block(voxel_pos, &item_string);
                                    client.connection_mut().try_send_message(
                                        ClientMessage::SentBlock {
                                            chunk_pos,
                                            voxel_pos: [
                                                voxel_pos.x as u8, // TODO: use normal and make sure to get neighbor chunk if needed
                                                voxel_pos.y as u8,
                                                voxel_pos.z as u8,
                                            ],
                                            block_type: item_string,
                                        },
                                    );
                                }
                            } else if mouse_left {
                                chunk.chunk_data.set_block(
                                    voxel_pos,
                                    &BlockData::new("vinox".to_string(), "air".to_string()),
                                );
                                client.connection_mut().try_send_message(
                                    ClientMessage::SentBlock {
                                        chunk_pos,
                                        voxel_pos: [
                                            voxel_pos.x as u8,
                                            voxel_pos.y as u8,
                                            voxel_pos.z as u8,
                                        ],
                                        block_type: BlockData::new(
                                            "vinox".to_string(),
                                            "air".to_string(),
                                        ),
                                    },
                                );
                            }
                            match voxel_pos.x {
                                0 => {
                                    if let Some(neighbor_chunk) =
                                        current_chunks.get_entity(chunk_pos + IVec3::new(-1, 0, 0))
                                    {
                                        commands.entity(neighbor_chunk).insert(PriorityMesh);
                                    }
                                }
                                CHUNK_SIZE_ARR => {
                                    if let Some(neighbor_chunk) =
                                        current_chunks.get_entity(chunk_pos + IVec3::new(1, 0, 0))
                                    {
                                        commands.entity(neighbor_chunk).insert(PriorityMesh);
                                    }
                                }
                                _ => {}
                            }
                            match voxel_pos.y {
                                0 => {
                                    if let Some(neighbor_chunk) =
                                        current_chunks.get_entity(chunk_pos + IVec3::new(0, -1, 0))
                                    {
                                        commands.entity(neighbor_chunk).insert(PriorityMesh);
                                    }
                                }
                                CHUNK_SIZE_ARR => {
                                    if let Some(neighbor_chunk) =
                                        current_chunks.get_entity(chunk_pos + IVec3::new(0, 1, 0))
                                    {
                                        commands.entity(neighbor_chunk).insert(PriorityMesh);
                                    }
                                }
                                _ => {}
                            }
                            match voxel_pos.z {
                                0 => {
                                    if let Some(neighbor_chunk) =
                                        current_chunks.get_entity(chunk_pos + IVec3::new(0, 0, -1))
                                    {
                                        commands.entity(neighbor_chunk).insert(PriorityMesh);
                                    }
                                }
                                CHUNK_SIZE_ARR => {
                                    if let Some(neighbor_chunk) =
                                        current_chunks.get_entity(chunk_pos + IVec3::new(0, 0, 1))
                                    {
                                        commands.entity(neighbor_chunk).insert(PriorityMesh);
                                    }
                                }
                                _ => {}
                            }
                            commands.entity(chunk_entity).insert(PriorityMesh);
                        }
                    }
                } else if let Ok((_, mut block_visibility)) = cube_position.get_single_mut() {
                    if *block_visibility == Visibility::Visible {
                        *block_visibility = Visibility::Hidden;
                    }
                }
            }
        }
    }
}
