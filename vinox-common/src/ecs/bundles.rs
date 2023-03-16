use bevy::{math::Vec3A, prelude::*, render::primitives::Aabb};

use crate::networking::protocol::Player;

#[derive(Component, Default)]
pub struct ClientName(pub String);

#[derive(Resource, Default)]
pub struct PlayerBundleBuilder {
    pub default_model: Handle<Scene>,
    pub model_aabb: Aabb,
}

#[derive(Default, Bundle)]
pub struct PlayerBundle {
    pub player_tag: Player,
    #[bundle]
    pub scene_bundle: SceneBundle,
    pub aabb: Aabb,
    pub username: ClientName,
}

impl PlayerBundleBuilder {
    pub fn build(
        &self,
        translation: Vec3,
        id: u64,
        local: bool,
        user_name: String,
    ) -> PlayerBundle {
        let handle = if local {
            Handle::default()
        } else {
            self.default_model.clone()
        };
        PlayerBundle {
            player_tag: Player { id },
            scene_bundle: SceneBundle {
                scene: handle,
                transform: Transform::from_translation(translation),
                ..default()
            },
            aabb: Aabb {
                center: translation.into(),
                half_extents: Vec3A::new(0.4, 0.9, 0.4),
            },
            username: ClientName(user_name),
        }
    }
}
