use bevy_atmosphere::prelude::*;
use std::f32::consts::PI;

use bevy::{pbr::DirectionalLightShadowMap, prelude::*};

use crate::states::components::{GameOptions, GameState};

use super::meshing::{
    create_chunk_material, process_priority_task, process_task, sort_chunks, sort_faces,
    ChunkMaterial, SortFaces,
};

use bevy_mod_edge_detection::{EdgeDetectionConfig, EdgeDetectionPlugin};

pub struct RenderingPlugin;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum RenderSet {
    StartMeshing,
    ProcessTask,
    Sorting,
}

#[derive(Component)]
pub struct Sun;

// Timer for updating the daylight cycle (updating the atmosphere every frame is slow, so it's better to do incremental changes)
#[derive(Resource)]
pub struct CycleTimer(Timer);

impl Plugin for RenderingPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AmbientLight {
            brightness: 1.0,
            color: Color::WHITE,
        })
        .add_plugin(EdgeDetectionPlugin)
        .add_plugin(AtmospherePlugin)
        .insert_resource(AtmosphereModel::default()) // Default Atmosphere material, we can edit it to simulate another planet
        .insert_resource(CycleTimer(Timer::new(
            bevy::utils::Duration::from_millis(60), // Update our atmosphere every 50ms (in a real game, this would be much slower, but for the sake of an example we use a faster update)
            TimerMode::Repeating,
        )))
        // .init_resource::<EdgeDetectionConfig>()
        .insert_resource(EdgeDetectionConfig {
            depth_threshold: 0.2,
            normal_threshold: 0.05,
            color_threshold: 10000.0,
            edge_color: Color::BLACK,
            debug: 0,
            enabled: 0,
        })
        .insert_resource(DirectionalLightShadowMap { size: 512 }) // TODO: Make this a setting
        // .insert_resource(MeshQueue::default())
        .insert_resource(ChunkMaterial::default())
        .add_system(create_chunk_material.in_schedule(OnEnter(GameState::Game)))
        .add_systems(
            (
                // process_queue,
                // process_priority_queue,
                update_outline,
                process_task,
                process_priority_task,
                // priority_player,
                sort_faces,
                sort_chunks,
                daylight_cycle,
            )
                .in_set(OnUpdate(GameState::Game)),
        )
        .add_startup_system(|mut commands: Commands, assets: Res<AssetServer>| {
            commands
                .spawn(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Percent(100.), Val::Auto),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(ImageBundle {
                        image: assets.load("crosshair.png").into(),
                        ..default()
                    });
                });

            commands.spawn((
                DirectionalLightBundle {
                    directional_light: DirectionalLight {
                        shadows_enabled: true,
                        illuminance: 10000.0,
                        ..default()
                    },
                    transform: Transform {
                        translation: Vec3::new(0.0, 2.0, 0.0),
                        rotation: Quat::from_rotation_x(-PI / 4.),
                        ..default()
                    },
                    ..default()
                },
                Sun,
            ));

            // commands.insert_resource(PriorityMeshChannel::default());
            // commands.insert_resource(MeshChannel::default());
        })
        .add_event::<SortFaces>();
    }
}

pub fn update_outline(options: Res<GameOptions>, mut edge_config: ResMut<EdgeDetectionConfig>) {
    if options.is_changed() {
        edge_config.enabled = options.outline as u32;
    }
}

pub fn daylight_cycle(
    mut atmosphere: AtmosphereMut<Nishita>,
    mut query: Query<(&mut Transform, &mut DirectionalLight), With<Sun>>,
    mut timer: ResMut<CycleTimer>,
    time: Res<Time>,
) {
    timer.0.tick(time.delta());

    if timer.0.finished() {
        let t = time.elapsed_seconds_wrapped() as f32 / 8.0;
        atmosphere.sun_position = Vec3::new(0., t.sin(), t.cos());

        if let Some((mut light_trans, mut directional)) = query.single_mut().into() {
            light_trans.rotation = Quat::from_rotation_x(-t);
            directional.illuminance = t.sin().max(0.0).powf(2.0) * 75000.0;
        }
    }
}
