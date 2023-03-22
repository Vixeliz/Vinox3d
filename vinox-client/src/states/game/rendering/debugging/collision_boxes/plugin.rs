use bevy::{pbr::wireframe::Wireframe, prelude::*, render::primitives::Aabb};

use crate::states::game::{input::player::FPSCamera, rendering::meshing::BasicMaterial};

pub struct RenderCollisionBoxPlugin;

#[derive(Component)]
pub struct HasCollisionBoundingBox;

impl Plugin for RenderCollisionBoxPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(add_collision_boxes);
        // app.add_system(despawn_with::<Game>.in_schedule(OnExit(GameState::Game)));
    }
}

pub fn add_collision_boxes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<BasicMaterial>>,
    asset_server: Res<AssetServer>,
    collision_entities: Query<(Entity, &Aabb), (With<FPSCamera>, Without<HasCollisionBoundingBox>)>,
) {
    for (ce, aabb) in collision_entities.iter() {
        let bounding_box = commands
            .spawn(MaterialMeshBundle {
                mesh: meshes.add(Mesh::from(shape::Box::new(
                    aabb.half_extents.x * 2.0,
                    aabb.half_extents.y * 2.0,
                    aabb.half_extents.z * 2.0,
                ))),
                material: materials.add(BasicMaterial {
                    color: Color::rgba(1.1, 1.1, 1.1, 1.0),
                    color_texture: Some(asset_server.load("outline.png")),
                    alpha_mode: AlphaMode::Blend,
                    discard_pix: 0,
                }),
                ..default()
            })
            .insert(Wireframe)
            .id();
        commands.entity(ce).push_children(&[bounding_box]);
        commands.entity(ce).insert(HasCollisionBoundingBox);
    }
}
