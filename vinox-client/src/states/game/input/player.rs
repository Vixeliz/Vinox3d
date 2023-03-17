use leafwing_input_manager::prelude::*;
use std::f32::consts::{FRAC_PI_2, PI};

use bevy::{
    input::mouse::MouseMotion,
    prelude::*,
    render::{
        camera::CameraProjection,
        primitives::{Aabb, Frustum},
    },
    window::{CursorGrabMode, PrimaryWindow},
};
use bevy_quinnet::client::Client;
use vinox_common::{
    collision::raycast::raycast_world,
    networking::protocol::ClientMessage,
    world::chunks::{
        ecs::{ChunkComp, CurrentChunks},
        positions::{relative_voxel_to_world, voxel_to_world, world_to_chunk, world_to_voxel},
        storage::{BlockData, BlockTable, CHUNK_SIZE_ARR},
    },
};

use crate::states::{
    components::{GameActions, GameOptions},
    game::{
        networking::syncing::HighLightCube,
        rendering::meshing::{NeedsMesh, PriorityMesh},
        ui::{dropdown::ConsoleOpen, plugin::InUi},
        world::chunks::ControlledPlayer,
    },
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
                ..default()
            }
        };
        commands.insert_resource(ClearColor(Color::rgba(0.5, 0.8, 0.9, 1.0)));
        commands.entity(player_entity).with_children(|c| {
            c.spawn((
                GlobalTransform::default(),
                Transform::from_xyz(0.0, 1.0, 0.0),
            ));
            c.spawn((FPSCamera::default(), camera));
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
    time: Res<Time>,
    mut stationary_frames: Local<i32>,
    current_chunks: Res<CurrentChunks>,
) {
    if let Ok((translation, action_state)) = player_position.get_single_mut() {
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
                    fps_camera.velocity.y = 12.0;
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

            if current_chunks.get_entity(chunk_pos).is_none() {
                return;
            }

            let looking_at = Vec3::new(
                10.0 * fps_camera.phi.cos() * fps_camera.theta.sin(),
                10.0 * fps_camera.theta.cos(),
                10.0 * fps_camera.phi.sin() * fps_camera.theta.sin(),
            );

            transform.look_at(looking_at, Vec3::new(0.0, 1.0, 0.0));
            fps_camera.velocity.y -= 35.0 * time.delta().as_secs_f32().clamp(0.0, 0.1);
        }
    }
}

#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
pub fn interact(
    mut commands: Commands,
    windows: Query<&mut Window, With<PrimaryWindow>>,
    camera_query: Query<&GlobalTransform, With<Camera>>,
    mut client: ResMut<Client>,
    player_position: Query<(&Transform, &ActionState<GameActions>), With<ControlledPlayer>>,
    mut cube_position: Query<
        (&mut Transform, &mut Visibility),
        (With<HighLightCube>, Without<ControlledPlayer>),
    >,
    mut chunks: Query<&mut ChunkComp>,
    current_chunks: Res<CurrentChunks>,
    block_table: Res<BlockTable>,
) {
    let window = windows.single();
    if window.cursor.grab_mode != CursorGrabMode::Locked {
        return;
    }
    if let Ok((player_transform, action_state)) = player_position.get_single() {
        let item = BlockData::new("vinox".to_string(), "cobblestone".to_string());

        let mouse_left = action_state.just_pressed(GameActions::PrimaryInteract);
        let mouse_right = action_state.just_pressed(GameActions::SecondaryInteract);
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
            if let Some((chunk_pos, voxel_pos, normal, _)) = hit {
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
                                    // TODO: Get neighboring chunk when needed instead of just disregarding
                                    let (chunk_pos, voxel_pos) =
                                        world_to_voxel(relative_voxel_to_world(
                                            voxel_pos.as_ivec3() + normal.as_ivec3(),
                                            chunk_pos,
                                        ));
                                    if let Some(chunk_entity) = current_chunks.get_entity(chunk_pos)
                                    {
                                        if let Ok(mut chunk) = chunks.get_mut(chunk_entity) {
                                            chunk.chunk_data.add_block_state(&item);
                                            chunk.chunk_data.set_block(voxel_pos, &item);
                                            client.connection_mut().try_send_message(
                                                ClientMessage::SentBlock {
                                                    chunk_pos,
                                                    voxel_pos: [
                                                        voxel_pos.x as u8, // TODO: use normal and make sure to get neighbor chunk if needed
                                                        voxel_pos.y as u8,
                                                        voxel_pos.z as u8,
                                                    ],
                                                    block_type: item,
                                                },
                                            );
                                            match voxel_pos.x {
                                                0 => {
                                                    if let Some(neighbor_chunk) = current_chunks
                                                        .get_entity(
                                                            chunk_pos + IVec3::new(-1, 0, 0),
                                                        )
                                                    {
                                                        commands
                                                            .entity(neighbor_chunk)
                                                            .remove::<NeedsMesh>();
                                                        commands
                                                            .entity(neighbor_chunk)
                                                            .insert(PriorityMesh);
                                                    }
                                                }
                                                CHUNK_SIZE_ARR => {
                                                    if let Some(neighbor_chunk) = current_chunks
                                                        .get_entity(chunk_pos + IVec3::new(1, 0, 0))
                                                    {
                                                        commands
                                                            .entity(neighbor_chunk)
                                                            .remove::<NeedsMesh>();
                                                        commands
                                                            .entity(neighbor_chunk)
                                                            .insert(PriorityMesh);
                                                    }
                                                }
                                                _ => {}
                                            }
                                            match voxel_pos.y {
                                                0 => {
                                                    if let Some(neighbor_chunk) = current_chunks
                                                        .get_entity(
                                                            chunk_pos + IVec3::new(0, -1, 0),
                                                        )
                                                    {
                                                        commands
                                                            .entity(neighbor_chunk)
                                                            .remove::<NeedsMesh>();
                                                        commands
                                                            .entity(neighbor_chunk)
                                                            .insert(PriorityMesh);
                                                    }
                                                }
                                                CHUNK_SIZE_ARR => {
                                                    if let Some(neighbor_chunk) = current_chunks
                                                        .get_entity(chunk_pos + IVec3::new(0, 1, 0))
                                                    {
                                                        commands
                                                            .entity(neighbor_chunk)
                                                            .remove::<NeedsMesh>();
                                                        commands
                                                            .entity(neighbor_chunk)
                                                            .insert(PriorityMesh);
                                                    }
                                                }
                                                _ => {}
                                            }
                                            match voxel_pos.z {
                                                0 => {
                                                    if let Some(neighbor_chunk) = current_chunks
                                                        .get_entity(
                                                            chunk_pos + IVec3::new(0, 0, -1),
                                                        )
                                                    {
                                                        commands
                                                            .entity(neighbor_chunk)
                                                            .remove::<NeedsMesh>();
                                                        commands
                                                            .entity(neighbor_chunk)
                                                            .insert(PriorityMesh);
                                                    }
                                                }
                                                CHUNK_SIZE_ARR => {
                                                    if let Some(neighbor_chunk) = current_chunks
                                                        .get_entity(chunk_pos + IVec3::new(0, 0, 1))
                                                    {
                                                        commands
                                                            .entity(neighbor_chunk)
                                                            .remove::<NeedsMesh>();
                                                        commands
                                                            .entity(neighbor_chunk)
                                                            .insert(PriorityMesh);
                                                    }
                                                }
                                                _ => {}
                                            }
                                            commands.entity(chunk_entity).insert(PriorityMesh);
                                        }
                                    }
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

                                match voxel_pos.x {
                                    0 => {
                                        if let Some(neighbor_chunk) = current_chunks
                                            .get_entity(chunk_pos + IVec3::new(-1, 0, 0))
                                        {
                                            commands.entity(neighbor_chunk).remove::<NeedsMesh>();
                                            commands.entity(neighbor_chunk).insert(PriorityMesh);
                                        }
                                    }
                                    CHUNK_SIZE_ARR => {
                                        if let Some(neighbor_chunk) = current_chunks
                                            .get_entity(chunk_pos + IVec3::new(1, 0, 0))
                                        {
                                            commands.entity(neighbor_chunk).remove::<NeedsMesh>();
                                            commands.entity(neighbor_chunk).insert(PriorityMesh);
                                        }
                                    }
                                    _ => {}
                                }
                                match voxel_pos.y {
                                    0 => {
                                        if let Some(neighbor_chunk) = current_chunks
                                            .get_entity(chunk_pos + IVec3::new(0, -1, 0))
                                        {
                                            commands.entity(neighbor_chunk).remove::<NeedsMesh>();
                                            commands.entity(neighbor_chunk).insert(PriorityMesh);
                                        }
                                    }
                                    CHUNK_SIZE_ARR => {
                                        if let Some(neighbor_chunk) = current_chunks
                                            .get_entity(chunk_pos + IVec3::new(0, 1, 0))
                                        {
                                            commands.entity(neighbor_chunk).remove::<NeedsMesh>();
                                            commands.entity(neighbor_chunk).insert(PriorityMesh);
                                        }
                                    }
                                    _ => {}
                                }
                                match voxel_pos.z {
                                    0 => {
                                        if let Some(neighbor_chunk) = current_chunks
                                            .get_entity(chunk_pos + IVec3::new(0, 0, -1))
                                        {
                                            commands.entity(neighbor_chunk).remove::<NeedsMesh>();
                                            commands.entity(neighbor_chunk).insert(PriorityMesh);
                                        }
                                    }
                                    CHUNK_SIZE_ARR => {
                                        if let Some(neighbor_chunk) = current_chunks
                                            .get_entity(chunk_pos + IVec3::new(0, 0, 1))
                                        {
                                            commands.entity(neighbor_chunk).remove::<NeedsMesh>();
                                            commands.entity(neighbor_chunk).insert(PriorityMesh);
                                        }
                                    }
                                    _ => {}
                                }
                                commands.entity(chunk_entity).insert(PriorityMesh);
                            }
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

// TODO: Move this to collision
pub fn collision_movement_system(
    mut camera: Query<(Entity, &mut FPSCamera)>,
    player: Query<(Entity, &Aabb), With<ControlledPlayer>>,
    mut transforms: Query<&mut Transform>,
    time: Res<Time>,
    chunks: Query<&mut ChunkComp>,
    current_chunks: Res<CurrentChunks>,
    block_table: Res<BlockTable>,
) {
    if let Ok((entity_camera, mut fps_camera)) = camera.get_single_mut() {
        if let Ok((entity_player, _player_aabb)) = player.get_single() {
            let looking_at = Vec3::new(
                10.0 * fps_camera.phi.cos() * fps_camera.theta.sin(),
                10.0 * fps_camera.theta.cos(),
                10.0 * fps_camera.phi.sin() * fps_camera.theta.sin(),
            );

            let mut camera_t = transforms.get_mut(entity_camera).unwrap();
            camera_t.look_at(looking_at, Vec3::new(0.0, 1.0, 0.0));

            let mut movement_left = fps_camera.velocity * time.delta().as_secs_f32();
            let leg_height = 0.26;
            let mut max_iter = 0;
            loop {
                max_iter += 1;
                // TODO: Don't do this hacky solution and actually get the player unstuck instead of continulously running the loop
                if movement_left.length() <= 0.0 || max_iter > 1000 {
                    break;
                }
                let mut player_transform = transforms.get_mut(entity_player).unwrap();
                let position = player_transform.translation - Vec3::new(0.0, 0.495, 0.0);

                match raycast_world(
                    position,
                    movement_left,
                    1.0,
                    &chunks,
                    &current_chunks,
                    &block_table,
                ) {
                    None => {
                        player_transform.translation =
                            position + movement_left + Vec3::new(0.0, 0.495, 0.0);
                        break;
                    }
                    Some((_chunk_pos, _voxel_pos, normal, _toi)) => {
                        // TODO: We will use aabb to get unstuck instead of this
                        // if toi < 0.0 {
                        //     let unstuck_vector = transforms
                        //         .get(current_chunks.get_entity(chunk_pos).unwrap())
                        //         .unwrap()
                        //         .translation
                        //         - position;
                        //     transforms.get_mut(entity_player).unwrap().translation -=
                        //         unstuck_vector.normalize() * 0.01;
                        //     fps_camera.velocity = Vec3::new(0.0, 0.0, 0.0);
                        //     break;
                        // }
                        movement_left -= movement_left.dot(normal) * normal;
                        fps_camera.velocity = movement_left / time.delta().as_secs_f32();
                    }
                }
            }

            if fps_camera.velocity.y <= 0.0 {
                let position =
                    transforms.get(entity_player).unwrap().translation - Vec3::new(0.0, 1.19, 0.0);

                if let Some((_chunk_pos, _voxel_pos, _normal, toi)) = raycast_world(
                    position,
                    Vec3::new(0.0, -1.0, 0.0),
                    leg_height,
                    &chunks,
                    &current_chunks,
                    &block_table,
                ) {
                    transforms.get_mut(entity_player).unwrap().translation -=
                        Vec3::new(0.0, toi - leg_height, 0.0);
                    fps_camera.velocity.y = 0.0;
                }
            }
        }
    }
}

pub fn cursor_grab_system(
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mut in_ui: ResMut<InUi>,
    mut is_open: ResMut<ConsoleOpen>,
    btn: Res<Input<MouseButton>>,
    key: Res<Input<KeyCode>>,
) {
    let mut window = windows.single_mut();

    if btn.just_pressed(MouseButton::Left) && !in_ui.0 {
        window.cursor.grab_mode = CursorGrabMode::Locked;
        window.cursor.visible = false;
        let window_center: Option<Vec2> =
            Some(Vec2::new(window.width() / 2.0, window.height() / 2.0));
        window.set_cursor_position(window_center);
    }

    if key.just_pressed(KeyCode::Escape) {
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
        if **in_ui {
            **is_open = false;
        }
        **in_ui = !**in_ui;
    }
}

pub fn update_aabb(mut player: Query<(&mut Aabb, &Transform), With<FPSCamera>>) {
    if let Ok((mut aabb, transform)) = player.get_single_mut() {
        aabb.center = transform.translation.into();
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
