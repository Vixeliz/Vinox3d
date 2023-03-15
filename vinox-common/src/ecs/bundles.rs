use bevy::{math::Vec3A, prelude::*, render::primitives::Aabb};

use crate::networking::protocol::Player;

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
}

impl PlayerBundleBuilder {
    pub fn build(&self, translation: Vec3, id: u64, local: bool) -> PlayerBundle {
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
        }
    }
}
