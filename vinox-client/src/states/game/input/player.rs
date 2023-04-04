use big_space::{FloatingOrigin, FloatingOriginSettings, GridCell};
use leafwing_input_manager::prelude::*;
use std::f32::consts::{FRAC_PI_2, PI};

use bevy::{
    input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel},
    math::Vec3A,
    prelude::*,
    render::{
        camera::CameraProjection,
        primitives::{Aabb, Frustum},
    },
    window::{CursorGrabMode, PresentMode, PrimaryWindow},
};
use bevy_quinnet::client::Client;
use vinox_common::{
    ecs::bundles::Inventory,
    networking::protocol::ClientMessage,
    physics::{
        collision::raycast::raycast_world,
        simulate::{CollidesWithWorld, Velocity},
    },
    storage::blocks::descriptor::BlockGeometry,
    world::chunks::{
        ecs::{ChunkManager, CurrentChunks},
        positions::{ChunkPos, RelativeVoxelPos, VoxelPos},
        storage::{
            self, name_to_identifier, trim_geo_identifier, BlockData, ItemTable, CHUNK_SIZE,
            HORIZONTAL_DISTANCE,
        },
    },
};

use crate::states::{
    components::{GameActions, GameOptions},
    game::{
        networking::syncing::HighLightCube,
        ui::{dropdown::ConsoleOpen, plugin::InUi},
        world::chunks::{ControlledPlayer, PlayerChunk, PlayerTargetedBlock},
    },
    menu::ui::InOptions,
};

#[derive(Component)]
pub struct FPSCamera {
    pub phi: f32,
    pub theta: f32,
}

impl Default for FPSCamera {
    fn default() -> Self {
        FPSCamera {
            phi: 0.0,
            theta: FRAC_PI_2,
        }
    }
}

pub fn update_input(
    mut commands: Commands,
    mut player_query: Query<Entity, With<ControlledPlayer>>,
    options: Res<GameOptions>,
) {
    if let Ok(entity) = player_query.get_single_mut() {
        if options.is_changed() {
            commands
                .entity(entity)
                .insert(InputManagerBundle::<GameActions> {
                    action_state: ActionState::default(),
                    input_map: options.input.clone(),
                });
        }
    }
}

pub fn update_vsync(options: Res<GameOptions>, mut windows: Query<&mut Window>) {
    if options.is_changed() {
        let mut window = windows.single_mut();
        window.present_mode = if options.vsync {
            PresentMode::AutoVsync
        } else {
            PresentMode::AutoNoVsync
        };
    }
}

pub fn update_fov(mut camera: Query<(&mut Projection, &mut Frustum)>, options: Res<GameOptions>) {
    if let Ok((mut projection, mut frustum)) = camera.get_single_mut() {
        if options.is_changed() {
            let perspective_projection = PerspectiveProjection {
                fov: options.fov.to_radians(),
                near: 0.001,
                far: 1000.0,
                aspect_ratio: 1.0,
            };
            let view_projection = perspective_projection.get_projection_matrix();
            *frustum = Frustum::from_view_projection(
                &view_projection,
                // &Vec3::ZERO,
                // &Vec3::Z,
                // perspective_projection.far(),
            );
            *projection = Projection::Perspective(perspective_projection);
        }
    }
}

pub fn spawn_camera(
    mut commands: Commands,
    player_entity: Query<Entity, With<ControlledPlayer>>,
    mut local: Local<bool>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    options: Res<GameOptions>,
    _boiler_player: Query<Entity, With<FloatingOrigin>>,
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
                fov: options.fov.to_radians(),
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
                transform: Transform::from_translation(Vec3::new(0.0, 1.65, 0.0)),
                // camera: Camera {
                //     hdr: true,
                //     ..Default::default()
                // },
                ..default()
            }
        };
        commands.insert_resource(ClearColor(Color::rgba(0.1, 0.1, 0.1, 1.0)));
        commands.entity(player_entity).with_children(|c| {
            c.spawn((
                GlobalTransform::default(),
                // ChunkCell::default(),
                Transform::from_xyz(0.0, 1.0, 0.0),
            ));
            c.spawn((
                FPSCamera::default(),
                // ChunkCell::default(),
                camera,
                // FloatingOrigin,
                FogSettings {
                    color: Color::rgba(0.1, 0.1, 0.1, 1.0),
                    directional_light_color: Color::WHITE,
                    directional_light_exponent: 10.0,
                    falloff: FogFalloff::Linear {
                        start: (HORIZONTAL_DISTANCE * CHUNK_SIZE) as f32
                            - (CHUNK_SIZE * (HORIZONTAL_DISTANCE / 3)) as f32,
                        end: (HORIZONTAL_DISTANCE * CHUNK_SIZE) as f32 + (CHUNK_SIZE) as f32,
                    },
                },
            ));
        });
        // if let Ok(boiler) = boiler_player.get_single() {
        //     commands.entity(boiler).despawn_recursive();
        // }
    }
}

#[derive(Resource)]
pub struct MouseSensitivity(pub f32);

#[allow(clippy::too_many_arguments)]
pub fn handle_movement(
    mut player: Query<&mut FPSCamera>,
    mut player_position: Query<
        (
            &mut Transform,
            &mut Velocity,
            &ActionState<GameActions>,
            Option<&CollidesWithWorld>,
        ),
        With<ControlledPlayer>,
    >,
    mut camera_transform: Query<&mut Transform, (With<Camera>, Without<ControlledPlayer>)>,
    mut mouse_events: EventReader<MouseMotion>,
    mouse_sensitivity: Res<MouseSensitivity>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut stationary_frames: Local<i32>,
    current_chunks: Res<CurrentChunks>,
    time: Res<Time>,
    player_chunk: Res<PlayerChunk>,
) {
    let Ok(window) = windows.get_single() else {
        return;
    };
    let Ok(mut transform) = camera_transform.get_single_mut() else {
        return;
    };
    // Update camera look
    if window.cursor.grab_mode == CursorGrabMode::Locked {
        if let Ok(mut fps_camera) = player.get_single_mut() {
            for MouseMotion { delta } in mouse_events.iter() {
                fps_camera.phi += delta.x * mouse_sensitivity.0 * 0.003;
                fps_camera.theta = (fps_camera.theta + delta.y * mouse_sensitivity.0 * 0.003)
                    .clamp(0.00005, PI - 0.00005);
            }
            let looking_at = Vec3::new(
                10.0 * fps_camera.phi.cos() * fps_camera.theta.sin(),
                10.0 * fps_camera.theta.cos(),
                10.0 * fps_camera.phi.sin() * fps_camera.theta.sin(),
            );
            transform.look_at(looking_at, Vec3::new(0.0, 1.0, 0.0));
        }
    }
    // Update velocity with movement input
    if let Ok((translation, mut velocity, action_state, world_collide)) =
        player_position.get_single_mut()
    {
        if world_collide.is_some() {
            let mut movement = Vec3::ZERO;

            if velocity.0.y.abs() < 0.001 && *stationary_frames < 10 {
                *stationary_frames += 4;
            } else if *stationary_frames >= 0 {
                *stationary_frames -= 1;
            }

            let gravity = 35.0 * Vec3::NEG_Y;
            velocity.0 += gravity * time.delta().as_secs_f32().clamp(0.0, 0.1);

            if window.cursor.grab_mode == CursorGrabMode::Locked {
                if current_chunks.get_entity(player_chunk.chunk_pos).is_none() {
                    return;
                }

                if action_state.pressed(GameActions::Forward) {
                    let mut fwd = transform.forward();
                    fwd.y = 0.0;
                    let fwd = fwd.normalize();
                    movement += fwd;
                }
                if action_state.pressed(GameActions::Left) {
                    movement += transform.left()
                }
                if action_state.pressed(GameActions::Right) {
                    movement += transform.right()
                }
                if action_state.pressed(GameActions::Backward) {
                    let mut back = transform.back();
                    back.y = 0.0;
                    let back = back.normalize();
                    movement += back;
                }
                movement = movement.normalize_or_zero();
                if action_state.pressed(GameActions::Run) {
                    movement *= 10.0;
                } else {
                    movement *= 5.0;
                }
                if action_state.pressed(GameActions::Jump) && *stationary_frames > 8 {
                    *stationary_frames = 0;
                    velocity.0.y = 10.0;
                }
            }
            velocity.0 = Vec3::new(movement.x, velocity.0.y, movement.z);
        } else {
            velocity.0 = Vec3::ZERO;
            if window.cursor.grab_mode == CursorGrabMode::Locked {
                if action_state.pressed(GameActions::Forward) {
                    velocity.0 += transform.forward().normalize();
                }
                if action_state.pressed(GameActions::Left) {
                    velocity.0 += transform.left()
                }
                if action_state.pressed(GameActions::Right) {
                    velocity.0 += transform.right()
                }
                if action_state.pressed(GameActions::Backward) {
                    velocity.0 += transform.back().normalize();
                }
                velocity.0 = velocity.0.normalize_or_zero();
                if action_state.pressed(GameActions::Run) {
                    velocity.0 *= 10.0;
                } else {
                    velocity.0 *= 5.0;
                }
                if action_state.pressed(GameActions::Jump) {
                    velocity.0.y = 10.0;
                }
            }
        }
    }
}

fn norm_to_bar(item: usize) -> Option<(usize, usize)> {
    if item > 8 {
        return None;
    }
    let f_item = item as f32;
    Some(((f_item / 3.0).floor() as usize, item.rem_euclid(3)))
}

//TODO: Overhaul of inventory and crafting to be reliant on server.
#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
pub fn interact(
    _commands: Commands,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    camera_query: Query<&GlobalTransform, With<Camera>>,
    mut client: ResMut<Client>,
    mut player: Query<
        (
            &VoxelPos,
            &ActionState<GameActions>,
            &mut Inventory,
            &Transform,
            &GridCell<i32>,
        ),
        With<ControlledPlayer>,
    >,
    mut cube_position: Query<
        (&mut Transform, &mut Visibility),
        (With<HighLightCube>, Without<ControlledPlayer>),
    >,
    // mut chunks: Query<&mut ChunkData>,
    // current_chunks: Res<CurrentChunks>,
    // block_table: Res<BlockTable>,
    mut chunk_manager: ChunkManager,
    item_table: Res<ItemTable>,
    mut temp_bar: Local<Option<usize>>,
    mut item_type: Local<BlockGeometry>,
    mut norm_item: Local<usize>,
    mut scroll_evr: EventReader<MouseWheel>,
    keys: Res<Input<KeyCode>>,
    options: Res<GameOptions>,
    mut player_targeted: ResMut<PlayerTargetedBlock>,
) {
    let window = windows.single_mut();
    if window.cursor.grab_mode != CursorGrabMode::Locked {
        return;
    }
    if let Ok((player_transform, action_state, mut inventory, transform, grid_cell)) =
        player.get_single_mut()
    {
        for ev in scroll_evr.iter() {
            match ev.unit {
                MouseScrollUnit::Line => {
                    if (ev.y * 10.0) < -1.0 {
                        if *norm_item < 9 {
                            *norm_item += 1;
                        } else {
                            *norm_item = 0;
                        }
                    } else if (ev.y * 10.0) > 1.0 {
                        if *norm_item > 0 {
                            *norm_item -= 1;
                        } else {
                            *norm_item = 8;
                        }
                    }
                    if let Some((cur_bar, cur_item)) = norm_to_bar(*norm_item) {
                        *inventory.current_bar = cur_bar;
                        *inventory.current_item = cur_item;
                    }
                }
                MouseScrollUnit::Pixel => {
                    if (ev.y * 0.05) < -1.0 {
                        if *norm_item < 9 {
                            *norm_item += 1;
                        } else {
                            *norm_item = 0;
                        }
                    } else if (ev.y * 0.05) > 1.0 {
                        if *norm_item > 0 {
                            *norm_item -= 1;
                        } else {
                            *norm_item = 8;
                        }
                    }
                    if let Some((cur_bar, cur_item)) = norm_to_bar(*norm_item) {
                        *inventory.current_bar = cur_bar;
                        *inventory.current_item = cur_item;
                    }
                }
            }
        }
        //Temporary
        if keys.just_pressed(KeyCode::J) {
            *item_type = BlockGeometry::Block;
        }
        if keys.just_pressed(KeyCode::K) {
            *item_type = BlockGeometry::Stairs;
        }
        if keys.just_pressed(KeyCode::F) {
            *item_type = BlockGeometry::Slab;
        }
        if keys.just_pressed(KeyCode::L) {
            *item_type = BlockGeometry::BorderedBlock;
        }
        if keys.just_pressed(KeyCode::U) {
            *item_type = BlockGeometry::Cross;
        }
        if keys.just_pressed(KeyCode::I) {
            *item_type = BlockGeometry::Flat;
        }
        if keys.just_pressed(KeyCode::N) {
            *item_type = BlockGeometry::Fence;
        }
        if keys.just_pressed(KeyCode::P) {
            *item_type = BlockGeometry::Custom("vinox:pole".to_string());
        }

        if !options.standard_bar {
            if keys.just_pressed(KeyCode::Key1) {
                if temp_bar.is_some() {
                    *inventory.current_bar = temp_bar.unwrap();
                    *inventory.current_item = 0;
                    *temp_bar = None;
                } else {
                    *temp_bar = Some(0);
                }
            } else if keys.just_pressed(KeyCode::Key2) {
                if temp_bar.is_some() {
                    *inventory.current_bar = temp_bar.unwrap();
                    *inventory.current_item = 1;
                    *temp_bar = None;
                } else {
                    *temp_bar = Some(1);
                }
            } else if keys.just_pressed(KeyCode::Key3) {
                if temp_bar.is_some() {
                    *inventory.current_bar = temp_bar.unwrap();
                    *inventory.current_item = 2;
                    *temp_bar = None;
                } else {
                    *temp_bar = Some(2);
                }
            }
        } else {
            for key in keys.get_just_pressed() {
                match key {
                    KeyCode::Key1 => {
                        *norm_item = 0;

                        if let Some((cur_bar, cur_item)) = norm_to_bar(*norm_item) {
                            *inventory.current_bar = cur_bar;
                            *inventory.current_item = cur_item;
                        }
                    }
                    KeyCode::Key2 => {
                        *norm_item = 1;

                        if let Some((cur_bar, cur_item)) = norm_to_bar(*norm_item) {
                            *inventory.current_bar = cur_bar;
                            *inventory.current_item = cur_item;
                        }
                    }
                    KeyCode::Key3 => {
                        *norm_item = 2;

                        if let Some((cur_bar, cur_item)) = norm_to_bar(*norm_item) {
                            *inventory.current_bar = cur_bar;
                            *inventory.current_item = cur_item;
                        }
                    }
                    KeyCode::Key4 => {
                        *norm_item = 3;

                        if let Some((cur_bar, cur_item)) = norm_to_bar(*norm_item) {
                            *inventory.current_bar = cur_bar;
                            *inventory.current_item = cur_item;
                        }
                    }
                    KeyCode::Key5 => {
                        *norm_item = 4;

                        if let Some((cur_bar, cur_item)) = norm_to_bar(*norm_item) {
                            *inventory.current_bar = cur_bar;
                            *inventory.current_item = cur_item;
                        }
                    }
                    KeyCode::Key6 => {
                        *norm_item = 5;

                        if let Some((cur_bar, cur_item)) = norm_to_bar(*norm_item) {
                            *inventory.current_bar = cur_bar;
                            *inventory.current_item = cur_item;
                        }
                    }
                    KeyCode::Key7 => {
                        *norm_item = 6;

                        if let Some((cur_bar, cur_item)) = norm_to_bar(*norm_item) {
                            *inventory.current_bar = cur_bar;
                            *inventory.current_item = cur_item;
                        }
                    }
                    KeyCode::Key8 => {
                        *norm_item = 7;

                        if let Some((cur_bar, cur_item)) = norm_to_bar(*norm_item) {
                            *inventory.current_bar = cur_bar;
                            *inventory.current_item = cur_item;
                        }
                    }
                    KeyCode::Key9 => {
                        *norm_item = 8;

                        if let Some((cur_bar, cur_item)) = norm_to_bar(*norm_item) {
                            *inventory.current_bar = cur_bar;
                            *inventory.current_item = cur_item;
                        }
                    }
                    _ => {}
                }
            }
        }

        let cur_item = inventory.clone().current_item;
        let cur_bar = inventory.clone().current_bar;
        let item_data = inventory.clone().hotbar[*cur_bar][*cur_item].clone();
        let place_item = if let Some(item) = item_data.clone() {
            if let Some(item_descriptor) = item_table.get(&name_to_identifier(
                item.namespace.clone(),
                item.name.clone(),
            )) {
                if item_descriptor.associated_block.is_some() {
                    Some(BlockData::new(item.namespace, item.name))
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        let mouse_left = action_state.just_pressed(GameActions::PrimaryInteract);
        let mouse_right = action_state.just_pressed(GameActions::SecondaryInteract);
        if let Ok(camera_transform) = camera_query.get_single() {
            // Then cast the ray.
            let hit = raycast_world(
                camera_transform.translation(),
                camera_transform.forward(),
                6.0,
                &chunk_manager,
                grid_cell,
            );
            if let Some((chunk_pos, voxel_pos, normal, _)) = hit {
                let point = VoxelPos::from((voxel_pos, chunk_pos)).relative_to_cell(*grid_cell);
                let global_voxel = VoxelPos::from(point.clone());
                //     world_to_global_voxel(relative_voxel_to_world(
                //     voxel_pos.as_vec3().as_ivec3(),
                //     *chunk_pos,
                // ));

                player_targeted.block = chunk_manager.get_block(global_voxel).clone();
                player_targeted.pos = Some(global_voxel);

                if let Ok((mut block_transform, mut block_visibility)) =
                    cube_position.get_single_mut()
                {
                    if *block_visibility == Visibility::Hidden {
                        *block_visibility = Visibility::Visible;
                    }
                    block_transform.translation = point + Vec3::splat(0.5);
                }
                if mouse_left || (mouse_right && place_item.is_some()) {
                    if mouse_right {
                        if (point.x as f32 <= player_transform.x as f32 - 0.5
                            || point.x as f32 >= player_transform.x as f32 + 0.5)
                            || (point.z as f32 <= player_transform.z as f32 - 0.5
                                || point.z as f32 >= player_transform.z as f32 + 0.5)
                            || (point.y as f32 <= player_transform.y as f32 - 1.0
                                || point.y as f32 >= player_transform.y as f32 + 1.0)
                        {
                            let (voxel_pos, chunk_pos) = VoxelPos::from((
                                RelativeVoxelPos(
                                    (voxel_pos.as_vec3().as_ivec3() + normal.as_ivec3()).as_uvec3(),
                                ),
                                chunk_pos,
                            ))
                            .to_offsets();
                            if let Some(mut modified_item) = place_item.clone() {
                                modified_item.name = if chunk_manager
                                    .block_table
                                    .get(&name_to_identifier(
                                        modified_item.namespace.clone(),
                                        item_type.geo_new_block(modified_item.name.clone()),
                                    ))
                                    .is_some()
                                {
                                    item_type.geo_new_block(modified_item.name.clone())
                                } else {
                                    place_item.clone().unwrap().name
                                };
                                let normal = normal.as_ivec3();
                                if chunk_manager
                                    .block_table
                                    .get(&name_to_identifier(
                                        modified_item.namespace.clone(),
                                        modified_item.name.clone(),
                                    ))
                                    .unwrap()
                                    .has_direction
                                    .unwrap_or(false)
                                {
                                    match normal.x {
                                        -1 => {
                                            modified_item.direction = Some(storage::Direction::West)
                                        }
                                        1 => {
                                            modified_item.direction = Some(storage::Direction::East)
                                        }
                                        _ => {}
                                    }
                                    match normal.y {
                                        -1 => {
                                            modified_item.top = Some(true);
                                        }
                                        1 => {
                                            modified_item.top = Some(false);
                                        }
                                        _ => {
                                            // modified_item.top = Some(false);
                                            // Stairs need tops and bottoms
                                        }
                                    }
                                    match normal.z {
                                        -1 => {
                                            modified_item.direction =
                                                Some(storage::Direction::South)
                                        }
                                        1 => {
                                            modified_item.direction =
                                                Some(storage::Direction::North)
                                        }
                                        _ => {}
                                    }

                                    if !chunk_manager
                                        .block_table
                                        .get(&name_to_identifier(
                                            modified_item.namespace.clone(),
                                            modified_item.name.clone(),
                                        ))
                                        .unwrap()
                                        .exclusive_direction
                                        .unwrap_or(false)
                                    {
                                        let translation: Vec3 = Vec3::from(*player_transform);
                                        if modified_item.direction.is_none() {
                                            let difference = translation - point;
                                            if difference.x > difference.z {
                                                if difference.x < 0.0 {
                                                    modified_item.direction =
                                                        Some(storage::Direction::West)
                                                } else {
                                                    modified_item.direction =
                                                        Some(storage::Direction::East)
                                                }
                                            } else if difference.z < 0.0 {
                                                modified_item.direction =
                                                    Some(storage::Direction::South)
                                            } else {
                                                modified_item.direction =
                                                    Some(storage::Direction::North)
                                            }
                                        }
                                        if modified_item.top.is_none() {
                                            let difference: Vec3 = translation - point;
                                            if difference.y > 0.0 {
                                                modified_item.top = Some(true);
                                            } else {
                                                modified_item.top = Some(false);
                                            }
                                        }
                                    }
                                }

                                if let Some(block) =
                                    chunk_manager.get_block(VoxelPos::from((voxel_pos, chunk_pos)))
                                {
                                    if block.is_empty(&chunk_manager.block_table) {
                                        inventory.item_decrement("hotbar", *cur_bar, *cur_item);

                                        chunk_manager.set_block(
                                            VoxelPos::from((voxel_pos, chunk_pos)),
                                            place_item.unwrap(),
                                        );
                                        client.connection_mut().try_send_message(
                                            ClientMessage::SentBlock {
                                                chunk_pos: *chunk_pos,
                                                voxel_pos: [
                                                    voxel_pos.x as u8,
                                                    voxel_pos.y as u8,
                                                    voxel_pos.z as u8,
                                                ],
                                                block_type: modified_item,
                                            },
                                        );
                                    }
                                }
                            }
                        }
                    } else if mouse_left {
                        if let Some(identifier) =
                            chunk_manager.get_identifier(VoxelPos::from((voxel_pos, chunk_pos)))
                        {
                            let identifier = trim_geo_identifier(identifier);
                            if let Some(item_def) = item_table.get(&identifier) {
                                if inventory.add_item(item_def).is_ok() {
                                    chunk_manager.set_block(
                                        VoxelPos::from((voxel_pos, chunk_pos)),
                                        BlockData::new("vinox".to_string(), "air".to_string()),
                                    );
                                    client.connection_mut().try_send_message(
                                        ClientMessage::SentBlock {
                                            chunk_pos: *chunk_pos,
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
                            } else {
                                chunk_manager.set_block(
                                    VoxelPos::from((voxel_pos, chunk_pos)),
                                    BlockData::new("vinox".to_string(), "air".to_string()),
                                );
                                client.connection_mut().try_send_message(
                                    ClientMessage::SentBlock {
                                        chunk_pos: *chunk_pos,
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
                        }
                    }
                }
            } else if let Ok((_, mut block_visibility)) = cube_position.get_single_mut() {
                *block_visibility = Visibility::Hidden;
                player_targeted.block = None;
                player_targeted.pos = None;
            }
        }
    }
}

// Update main position based on the AABB
pub fn update_visual_position(
    mut player: Query<(&mut Transform, &mut VoxelPos, &mut GridCell<i32>), With<ControlledPlayer>>,
    floating_settings: Res<FloatingOriginSettings>,
) {
    if let Ok((mut transform, mut voxel_pos, mut grid_cell)) = player.get_single_mut() {
        // (*grid_cell, transform.translation) = floating_settings
        //     .imprecise_translation_to_grid::<i32>(Vec3::from(
        //         aabb.center - Vec3A::Y * aabb.half_extents,
        //     ));
        *voxel_pos = VoxelPos::from_chunk_cell(*grid_cell, transform.translation);
        // *voxel_pos = VoxelPos::from(aabb.center - Vec3A::Y * aabb.half_extents);
        // transform.translation = Vec3::from(aabb.center - Vec3A::Y * aabb.half_extents)
    }
}

pub fn cursor_grab_system(
    mut inventory: Query<(&mut Inventory, &ActionState<GameActions>), With<ControlledPlayer>>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mut in_ui: ResMut<InUi>,
    mut is_open: ResMut<ConsoleOpen>,
    btn: Res<Input<MouseButton>>,
    key: Res<Input<KeyCode>>,
    mut in_options: ResMut<InOptions>,
) {
    let mut window = windows.single_mut();
    if let Ok((mut inventory, action_state)) = inventory.get_single_mut() {
        if action_state.just_pressed(GameActions::Inventory) {
            if window.cursor.grab_mode == CursorGrabMode::None && inventory.open {
                window.cursor.grab_mode = CursorGrabMode::Locked;
                window.cursor.visible = false;
                inventory.open = !inventory.open;
                **in_ui = !**in_ui;
            } else if !**in_ui {
                let window_center: Option<Vec2> =
                    Some(Vec2::new(window.width() / 2.0, window.height() / 2.0));
                window.set_cursor_position(window_center);
                window.cursor.grab_mode = CursorGrabMode::None;
                window.cursor.visible = true;
                inventory.open = !inventory.open;
                **in_ui = !**in_ui;
            }
        }

        if btn.just_pressed(MouseButton::Left) && !in_ui.0 {
            window.cursor.grab_mode = CursorGrabMode::Locked;
            window.cursor.visible = false;
            **is_open = false;
            inventory.open = false;
        }

        if key.just_pressed(KeyCode::Escape) {
            if window.cursor.grab_mode == CursorGrabMode::None {
                window.cursor.grab_mode = CursorGrabMode::Locked;
                window.cursor.visible = false;
                if **in_options {
                    **in_options = !**in_options;
                }
            } else {
                let window_center: Option<Vec2> =
                    Some(Vec2::new(window.width() / 2.0, window.height() / 2.0));
                window.set_cursor_position(window_center);
                window.cursor.grab_mode = CursorGrabMode::None;
                window.cursor.visible = true;
                **in_options = !**in_options;
            }
            if **in_ui {
                **is_open = false;
                inventory.open = false;
            }
            **in_ui = !**in_ui;
        }
    }
}

pub fn ui_input(
    mut is_open: ResMut<ConsoleOpen>,
    mut in_ui: ResMut<InUi>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    player_actions: Query<&ActionState<GameActions>, With<ControlledPlayer>>,
) {
    let mut window = windows.single_mut();
    if let Ok(action_state) = player_actions.get_single() {
        if action_state.just_pressed(GameActions::Chat) && !**in_ui {
            let window_center: Option<Vec2> =
                Some(Vec2::new(window.width() / 2.0, window.height() / 2.0));
            window.set_cursor_position(window_center);
            if window.cursor.grab_mode == CursorGrabMode::None {
                window.cursor.grab_mode = CursorGrabMode::Locked;
                window.cursor.visible = false;
            } else {
                window.cursor.grab_mode = CursorGrabMode::None;
                window.cursor.visible = true;
            }

            **is_open = !**is_open;
            **in_ui = !**in_ui;
        }
    }
}

pub fn debug_input(
    mut is_open: ResMut<GameOptions>,
    player_actions: Query<&ActionState<GameActions>, With<ControlledPlayer>>,
    // mut in_ui: ResMut<InUi>,
) {
    if let Ok(action_state) = player_actions.get_single() {
        if action_state.just_released(GameActions::Debug) {
            is_open.debug = !is_open.debug;
        }
    }
}
