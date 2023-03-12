use std::f32::consts::{FRAC_PI_2, PI};

use bevy::{
    input::mouse::MouseMotion,
    pbr::NotShadowCaster,
    prelude::*,
    render::{camera::CameraProjection, primitives::Frustum},
    window::{CursorGrabMode, PrimaryWindow},
};
use vinox_common::world::chunks::{ecs::CurrentChunks, positions::world_to_chunk};

use crate::states::game::world::chunks::ControlledPlayer;

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
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
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
                        start: 200.0,
                        end: 400.0,
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
    player_position: Query<&Transform, With<ControlledPlayer>>,
    mut camera_transform: Query<&mut Transform, (With<Camera>, Without<ControlledPlayer>)>,
    mut mouse_events: EventReader<MouseMotion>,
    mouse_sensitivity: Res<MouseSensitivity>,
    key_events: Res<Input<KeyCode>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    time: Res<Time>,
    mut stationary_frames: Local<i32>,
    current_chunks: Res<CurrentChunks>,
) {
    if let Ok(translation) = player_position.get_single() {
        let translation = translation.translation;
        if current_chunks
            .get_entity(world_to_chunk(translation))
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
                    let mut fwd = transform.forward().clone();
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
                    let mut back = transform.back().clone();
                    back.y = 0.0;
                    let back = back.normalize();
                    movement += back;
                }

                if key_events.pressed(KeyCode::Space) && *stationary_frames > 2 {
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
            if key_events.pressed(KeyCode::LShift) {
                fps_camera.velocity *= 10.0;
            } else {
                fps_camera.velocity *= 5.0;
            }
            fps_camera.velocity.y = y;
            let chunk_pos = world_to_chunk(translation);

            if current_chunks.get_entity(chunk_pos).is_none() {
                return;
            }

            let looking_at = Vec3::new(
                10.0 * fps_camera.phi.cos() * fps_camera.theta.sin(),
                10.0 * fps_camera.theta.cos(),
                10.0 * fps_camera.phi.sin() * fps_camera.theta.sin(),
            );

            transform.look_at(looking_at, Vec3::new(0.0, 1.0, 0.0));

            // fps_camera.velocity.y -= 35.0 * time.delta().as_secs_f32().clamp(0.0, 0.1);
        }
    }
}
