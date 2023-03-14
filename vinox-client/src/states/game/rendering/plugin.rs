use bevy::prelude::*;

use crate::states::components::GameState;

use super::meshing::{
    create_chunk_material, process_priority_queue, process_queue, sort_chunks, sort_faces,
    ChunkMaterial, MeshChannel, MeshQueue, PriorityMeshChannel, SortFaces,
};

pub struct RenderingPlugin;

impl Plugin for RenderingPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MeshQueue::default())
            .insert_resource(ChunkMaterial::default())
            .add_system(create_chunk_material.in_schedule(OnEnter(GameState::Game)))
            .add_systems(
                (
                    process_queue,
                    process_priority_queue,
                    sort_faces,
                    sort_chunks,
                )
                    .in_set(OnUpdate(GameState::Game)),
            )
            .insert_resource(AmbientLight {
                color: Color::WHITE,
                brightness: 1.0,
            })
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
                commands.insert_resource(PriorityMeshChannel(tokio::sync::mpsc::channel(256)));
                commands.insert_resource(MeshChannel(tokio::sync::mpsc::channel(512)));
            })
            .add_event::<SortFaces>();
    }
}
