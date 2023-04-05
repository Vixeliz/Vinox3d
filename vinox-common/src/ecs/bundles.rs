use bevy::{math::Vec3A, prelude::*, render::primitives::Aabb};
use big_space::{FloatingOrigin, FloatingSpatialBundle};
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
#[derive(Default, Deref, DerefMut, Serialize, Deserialize, Debug, Clone)]
pub struct CurrentInvBar(pub usize);

#[derive(Default, Deref, DerefMut, Serialize, Deserialize, Debug, Clone)]
pub struct CurrentInvItem(pub usize);

#[derive(Component, Default, Serialize, Deserialize, Clone, Debug)]
pub struct Inventory {
    pub username: String,
    pub hotbar: HotBar,
    pub slots: [[Option<ItemData>; 9]; 5],
    pub current_bar: CurrentBar,
    pub current_item: CurrentItem,
    pub current_inv_bar: CurrentInvBar,
    pub current_inv_item: CurrentInvItem,
    pub open: bool,
}

#[derive(Bundle)]
pub struct BoilerOrigin {
    // pub cell: ChunkCell,
    pub origin: FloatingOrigin,
    #[bundle]
    pub spatial: FloatingSpatialBundle<i32>,
}

impl Default for BoilerOrigin {
    fn default() -> Self {
        Self {
            // cell: ChunkCell::default(),
            origin: FloatingOrigin,
            spatial: FloatingSpatialBundle::default(),
        }
    }
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

    pub fn add_item(&mut self, item_comp: &ItemDescriptor) -> Result<u8, u8> {
        if let Some((section, row, idx, amount)) = self.get_first_item(item_comp) {
            if amount < item_comp.max_stack_size.unwrap_or(MAX_STACK_SIZE) {
                match section {
                    "inventory" => {
                        self.slots[row][row] = Some(ItemData {
                            name: item_comp.name.clone(),
                            namespace: item_comp.namespace.clone(),
                            stack_size: amount + 1,
                            ..Default::default()
                        });
                    }
                    "hotbar" => {
                        self.hotbar[row][idx] = Some(ItemData {
                            name: item_comp.name.clone(),
                            namespace: item_comp.namespace.clone(),
                            stack_size: amount + 1,
                            ..Default::default()
                        });
                    }
                    _ => {}
                }
                return Ok(1);
            }
        }
        if let Some((section, row, idx)) = self.get_first_slot() {
            match section {
                "inventory" => {
                    self.slots[row][row] = Some(ItemData {
                        name: item_comp.name.clone(),
                        namespace: item_comp.namespace.clone(),
                        stack_size: 1,
                        ..Default::default()
                    });
                }
                "hotbar" => {
                    self.hotbar[row][idx] = Some(ItemData {
                        name: item_comp.name.clone(),
                        namespace: item_comp.namespace.clone(),
                        stack_size: 1,
                        ..Default::default()
                    });
                }
                _ => {}
            }
            return Ok(1);
        }
        Err(0)
    }

    pub fn item_decrement(
        &mut self,
        section: &str,
        row: usize,
        num: usize,
        // item_comp: &ItemDescriptor,
    ) {
        match section {
            "inventory" => {
                if self.slots[row][num].clone().unwrap_or_default().stack_size == 1 {
                    self.slots[row][num] = None;
                } else if let Some(item) = self.slots[row][num].as_mut() {
                    item.stack_size -= 1;
                }
            }
            "hotbar" => {
                if self.hotbar[row][num].clone().unwrap_or_default().stack_size == 1 {
                    self.hotbar[row][num] = None;
                } else if let Some(item) = self.hotbar[row][num].as_mut() {
                    item.stack_size -= 1;
                }
            }
            _ => {}
        }
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
                center: Vec3A::from(translation) + Vec3A::new(0.4, 0.9, 0.4),
                half_extents: Vec3A::new(0.3, 0.9, 0.3),
            },
            username: ClientName(user_name),
        }
    }
}
