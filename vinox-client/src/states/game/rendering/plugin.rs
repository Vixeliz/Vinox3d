use bevy::prelude::*;

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

impl Plugin for RenderingPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AmbientLight {
            brightness: 1.0,
            color: Color::WHITE,
        })
        .add_plugin(EdgeDetectionPlugin)
        // .init_resource::<EdgeDetectionConfig>()
        .insert_resource(EdgeDetectionConfig {
            depth_threshold: 0.2,
            normal_threshold: 0.05,
            color_threshold: 10000.0,
            edge_color: Color::BLACK,
            debug: 0,
            enabled: 1,
        })
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
