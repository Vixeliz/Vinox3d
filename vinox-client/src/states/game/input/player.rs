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
    collision::aabb::aabb_vs_world,
    collision::raycast::raycast_world,
    ecs::bundles::Inventory,
    networking::protocol::ClientMessage,
    storage::{blocks::descriptor::BlockGeometry, items::descriptor::ItemData},
    world::chunks::{
        ecs::{ChunkManager, CurrentChunks},
        positions::{relative_voxel_to_world, voxel_to_world, world_to_chunk, world_to_voxel},
        positions::{voxel_to_global_voxel, ChunkPos},
        storage::{
            self, name_to_identifier, trim_geo_identifier, BlockData, BlockTable, ChunkData,
            ItemTable, CHUNK_SIZE, CHUNK_SIZE_ARR, HORIZONTAL_DISTANCE,
        },
    },
};

use crate::states::{
    components::{GameActions, GameOptions},
    game::{
        networking::syncing::HighLightCube,
        ui::{dropdown::ConsoleOpen, plugin::InUi},
        world::chunks::ControlledPlayer,
    },
    menu::ui::InOptions,
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
                // camera: Camera {
                //     hdr: true,
                //     ..Default::default()
                // },
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
                        start: (HORIZONTAL_DISTANCE * CHUNK_SIZE) as f32
                            - (CHUNK_SIZE * (HORIZONTAL_DISTANCE / 2)) as f32,
                        end: (HORIZONTAL_DISTANCE * CHUNK_SIZE) as f32 + (CHUNK_SIZE * 2) as f32,
                    },
                    ..Default::default()
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
    mut player_position: Query<(&mut Transform, &ActionState<GameActions>), With<ControlledPlayer>>,
    mut camera_transform: Query<&mut Transform, (With<Camera>, Without<ControlledPlayer>)>,
    mut mouse_events: EventReader<MouseMotion>,
    mouse_sensitivity: Res<MouseSensitivity>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut stationary_frames: Local<i32>,
    current_chunks: Res<CurrentChunks>,
) {
    if let Ok((translation, action_state)) = player_position.get_single_mut() {
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

                if action_state.pressed(GameActions::Jump) && *stationary_frames > 2 {
                    *stationary_frames = 0;
                    fps_camera.velocity.y = 14.0;
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
            if action_state.pressed(GameActions::Run) {
                fps_camera.velocity *= 10.0;
            } else {
                fps_camera.velocity *= 5.0;
            }
            fps_camera.velocity.y = y;
            let chunk_pos = world_to_chunk(translation.translation);

            if current_chunks.get_entity(ChunkPos(chunk_pos)).is_none() {
                return;
            }

            let looking_at = Vec3::new(
                10.0 * fps_camera.phi.cos() * fps_camera.theta.sin(),
                10.0 * fps_camera.theta.cos(),
                10.0 * fps_camera.phi.sin() * fps_camera.theta.sin(),
            );

            transform.look_at(looking_at, Vec3::new(0.0, 1.0, 0.0));
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
    mut commands: Commands,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    camera_query: Query<&GlobalTransform, With<Camera>>,
    mut client: ResMut<Client>,
    mut player: Query<
        (&Transform, &ActionState<GameActions>, &mut Inventory),
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
) {
    let window = windows.single_mut();
    if window.cursor.grab_mode != CursorGrabMode::Locked {
        return;
    }
    if let Ok((player_transform, action_state, mut inventory)) = player.get_single_mut() {
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
            Some(BlockData::new(item.namespace, item.name))
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
                50.0,
                &chunk_manager,
            );
            if let Some((chunk_pos, voxel_pos, normal, _)) = hit {
                let point = voxel_to_world(voxel_pos.as_vec3().as_uvec3(), *chunk_pos);

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
                        if item_data.unwrap_or_default().stack_size == 1 {
                            inventory.hotbar[*cur_bar][*cur_item] = None;
                        } else if let Some(item) = inventory.hotbar[*cur_bar][*cur_item].as_mut() {
                            item.stack_size -= 1;
                        }

                        if (point.x <= player_transform.translation.x - 0.5
                            || point.x >= player_transform.translation.x + 0.5)
                            || (point.z <= player_transform.translation.z - 0.5
                                || point.z >= player_transform.translation.z + 0.5)
                            || (point.y <= player_transform.translation.y - 1.0
                                || point.y >= player_transform.translation.y + 1.0)
                        {
                            let (chunk_pos, voxel_pos) = world_to_voxel(relative_voxel_to_world(
                                voxel_pos.as_vec3().as_ivec3() + normal.as_ivec3(),
                                *chunk_pos,
                            ));
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
                                        if modified_item.direction.is_none() {
                                            let difference = player_transform.translation - point;
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
                                            let difference = player_transform.translation - point;
                                            if difference.y > 0.0 {
                                                modified_item.top = Some(true);
                                            } else {
                                                modified_item.top = Some(false);
                                            }
                                        }
                                    }
                                }

                                chunk_manager.set_block(
                                    voxel_to_global_voxel(voxel_pos, chunk_pos),
                                    place_item.unwrap(),
                                );
                                client.connection_mut().try_send_message(
                                    ClientMessage::SentBlock {
                                        chunk_pos,
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
                    } else if mouse_left {
                        if let Some(identifier) = chunk_manager
                            .get_identifier(voxel_to_global_voxel(voxel_pos, *chunk_pos))
                        {
                            let identifier = trim_geo_identifier(identifier);
                            if let Some(item_def) = item_table.get(&identifier) {
                                if let Some((section, row_index, item_index, stack_size)) =
                                    inventory.get_first_item(item_def)
                                {
                                    match section {
                                        "inventory" => {
                                            inventory.slots[row_index][item_index] =
                                                Some(ItemData {
                                                    name: item_def.name.clone(),
                                                    namespace: item_def.namespace.clone(),
                                                    stack_size: stack_size + 1,
                                                    ..Default::default()
                                                });
                                        }
                                        "hotbar" => {
                                            inventory.hotbar[row_index][item_index] =
                                                Some(ItemData {
                                                    name: item_def.name.clone(),
                                                    namespace: item_def.namespace.clone(),
                                                    stack_size: stack_size + 1,
                                                    ..Default::default()
                                                });
                                        }
                                        _ => {}
                                    }
                                } else if let Some((section, row_index, item_index)) =
                                    inventory.get_first_slot()
                                {
                                    match section {
                                        "inventory" => {
                                            inventory.slots[row_index][item_index] =
                                                Some(ItemData {
                                                    name: item_def.name.clone(),
                                                    namespace: item_def.namespace.clone(),
                                                    stack_size: 1,
                                                    ..Default::default()
                                                });
                                        }
                                        "hotbar" => {
                                            inventory.hotbar[row_index][item_index] =
                                                Some(ItemData {
                                                    name: item_def.name.clone(),
                                                    namespace: item_def.namespace.clone(),
                                                    stack_size: 1,
                                                    ..Default::default()
                                                });
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            chunk_manager.set_block(
                                voxel_to_global_voxel(voxel_pos, *chunk_pos),
                                BlockData::new("vinox".to_string(), "air".to_string()),
                            );
                            client
                                .connection_mut()
                                .try_send_message(ClientMessage::SentBlock {
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
                                });
                        }
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

pub fn collision_movement_system(
    mut camera: Query<(Entity, &mut FPSCamera)>,
    mut player: Query<(Entity, &mut Aabb), With<ControlledPlayer>>,
    mut transforms: Query<&mut Transform>,
    time: Res<Time>,
    chunks: Query<&ChunkData>,
    current_chunks: Res<CurrentChunks>,
    block_table: Res<BlockTable>,
) {
    if let Ok((entity_camera, mut fps_camera)) = camera.get_single_mut() {
        if let Ok((entity_player, mut player_aabb)) = player.get_single_mut() {
            let mut camera_t = transforms.get_mut(entity_camera).unwrap();
            let looking_at = Vec3::new(
                10.0 * fps_camera.phi.cos() * fps_camera.theta.sin(),
                10.0 * fps_camera.theta.cos(),
                10.0 * fps_camera.phi.sin() * fps_camera.theta.sin(),
            );
            camera_t.look_at(looking_at, Vec3::new(0.0, 1.0, 0.0));
            camera_t.translation = Vec3::new(0.0, 1.8, 0.0);

            if current_chunks
                .get_entity(ChunkPos(world_to_chunk(Vec3::from(player_aabb.center))))
                .is_none()
            {
                return;
            }

            let mut player_transform = transforms.get_mut(entity_player).unwrap();
            fps_camera.velocity.y -= 35.0 * time.delta().as_secs_f32().clamp(0.0, 0.1);
            let movement = fps_camera.velocity * time.delta().as_secs_f32();
            let mut v_after = movement;
            let mut max_move = v_after.abs();
            let margin: f32 = 0.01;
            let aabb_collisions = aabb_vs_world(
                player_aabb.clone(),
                &chunks,
                v_after,
                &current_chunks,
                &block_table,
                margin,
            );
            if let Some(collisions_list) = aabb_collisions {
                for col in collisions_list {
                    let margin_dist = col.dist;
                    if col.normal.x != 0.0 {
                        max_move.x = f32::min(max_move.x, margin_dist);
                        v_after.x = 0.0;
                    } else if col.normal.y != 0.0 {
                        max_move.y = f32::min(max_move.y, margin_dist);
                        v_after.y = 0.0;
                    } else if col.normal.z != 0.0 {
                        max_move.z = f32::min(max_move.z, margin_dist);
                        v_after.z = 0.0;
                    }
                }
            }
            fps_camera.velocity = v_after / time.delta().as_secs_f32();
            set_pos(
                player_transform.translation
                    + Vec3::new(
                        max_move.x * movement.x.signum(),
                        max_move.y * movement.y.signum(),
                        max_move.z * movement.z.signum(),
                    ),
                &mut player_aabb,
                &mut player_transform,
            );
        }
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

pub fn set_pos(position: Vec3, aabb: &mut Aabb, transform: &mut Transform) {
    aabb.center = Vec3A::from(position) + aabb.half_extents;
    transform.translation = position;
}

pub fn update_aabb(mut player: Query<(&mut Aabb, &Transform), With<ControlledPlayer>>) {
    if let Ok((mut aabb, transform)) = player.get_single_mut() {
        aabb.center =
            Vec3A::from(transform.translation) + Vec3A::new(0.0, aabb.half_extents.y, 0.0);
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
