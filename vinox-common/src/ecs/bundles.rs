use bevy::{math::Vec3A, prelude::*, render::primitives::Aabb};
use serde::{Deserialize, Serialize};

use crate::{
    networking::protocol::Player,
    storage::items::descriptor::{ItemData, ItemDescriptor, MAX_STACK_SIZE},
};

#[derive(Default, Deref, DerefMut, Serialize, Deserialize, Debug, Clone)]
pub struct HotBar(pub [[Option<ItemData>; 3]; 3]);

#[derive(Default, Deref, DerefMut, Serialize, Deserialize, Debug, Clone)]
pub struct CurrentBar(pub usize);

#[derive(Default, Deref, DerefMut, Serialize, Deserialize, Debug, Clone)]
pub struct CurrentItem(pub usize);

#[derive(Component, Default, Serialize, Deserialize, Clone, Debug)]
pub struct Inventory {
    pub username: String,
    pub hotbar: HotBar,
    pub slots: [[Option<ItemData>; 9]; 5],
    pub current_bar: CurrentBar,
    pub current_item: CurrentItem,
}

impl Inventory {
    // String says whether int the hotbar array or slots
    pub fn get_first_slot(&self) -> Option<(&str, usize, usize)> {
        for (hotbar_num, hotbar_sect) in self.hotbar.iter().cloned().enumerate() {
            for (item_num, item) in hotbar_sect.iter().cloned().enumerate() {
                if item.is_none() {
                    return Some(("hotbar", hotbar_num, item_num));
                }
            }
        }

        for (row_num, row) in self.slots.iter().cloned().enumerate() {
            for (item_num, item) in row.iter().cloned().enumerate() {
                if item.is_none() {
                    return Some(("inventory", row_num, item_num));
                }
            }
        }
        None
    }
    pub fn get_first_item(&self, item_comp: &ItemDescriptor) -> Option<(&str, usize, usize, u32)> {
        for (hotbar_num, hotbar_sect) in self.hotbar.iter().cloned().enumerate() {
            for (item_num, item) in hotbar_sect.iter().cloned().enumerate() {
                if let Some(item) = item {
                    if item.name + &item.namespace
                        == item_comp.clone().name + &item_comp.clone().namespace
                        && item.stack_size < MAX_STACK_SIZE
                    {
                        return Some(("hotbar", hotbar_num, item_num, item.stack_size));
                    }
                }
            }
        }

        for (row_num, row) in self.slots.iter().cloned().enumerate() {
            for (item_num, item) in row.iter().cloned().enumerate() {
                if let Some(item) = item {
                    if item.name + &item.namespace
                        == item_comp.clone().name + &item_comp.clone().namespace
                        && item.stack_size < MAX_STACK_SIZE
                    {
                        return Some(("inventory", row_num, item_num, item.stack_size));
                    }
                }
            }
        }
        None
    }
}

#[derive(Component, Default, Deref, DerefMut)]
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
