use egui_extras::{Size, StripBuilder};


use bevy::prelude::*;
use bevy_egui::{
    egui::{Color32, Sense},
    *,
};
use vinox_common::{
    ecs::bundles::{CurrentInvBar, CurrentInvItem, Inventory},
    storage::items::descriptor::ItemData,
    world::chunks::storage::name_to_identifier,
};

use crate::states::{
    assets::load::LoadableAssets, components::GameOptions, game::world::chunks::ControlledPlayer,
};

pub fn status_bar(
    mut player_query: Query<&mut Inventory, With<ControlledPlayer>>,
    mut contexts: EguiContexts,
    _options: Res<GameOptions>,
    mut held_items: ResMut<CurrentItemsHeld>,
    mut holding: ResMut<Holding>,
    loadable_assets: Res<LoadableAssets>,
) {
    let ctx = contexts.ctx_mut().clone();
    let style = ctx.style();
    egui::TopBottomPanel::bottom("status_bar")
        .default_height(40.0)
        .max_height(75.0)
        .show(&ctx, |ui| {
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                if let Ok(mut inventory) = player_query.get_single_mut() {
                    StripBuilder::new(ui)
                        .size(Size::exact(50.0))
                        .size(Size::exact(50.0))
                        .size(Size::exact(50.0))
                        .size(Size::exact(50.0))
                        .size(Size::exact(50.0))
                        .size(Size::exact(50.0))
                        .size(Size::exact(50.0))
                        .size(Size::exact(50.0))
                        .size(Size::exact(50.0))
                        .horizontal(|mut strip| {
                            for (hotbar_num, hotbar_section) in
                                inventory.clone().hotbar.iter().cloned().enumerate()
                            {
                                for (item_num, item) in hotbar_section.iter().clone().enumerate() {
                                    strip.cell(|ui| {
                                        let color = if *inventory.current_item == item_num
                                            && *inventory.current_bar == hotbar_num
                                        {
                                            Color32::from_white_alpha(128)
                                        } else if item.is_none() {
                                            style.visuals.faint_bg_color
                                        } else {
                                            Color32::WHITE
                                        };
                                        egui::Frame::none().outer_margin(2.0).fill(color).show(
                                            ui,
                                            |ui| {
                                                if let Some(item) = item {
                                                    let image = ui
                                                        .add(
                                                            egui::widgets::Image::new(
                                                                contexts
                                                                    .image_id(
                                                                        loadable_assets
                                                                            .item_textures
                                                                            .get(
                                                                                &name_to_identifier(
                                                                                    item.namespace
                                                                                        .clone(),
                                                                                    item.name
                                                                                        .clone(),
                                                                                ),
                                                                            )
                                                                            .unwrap(),
                                                                    )
                                                                    .unwrap(),
                                                                [48.0, 48.0],
                                                            )
                                                            .tint(color)
                                                            .sense(Sense::click()),
                                                        )
                                                        .on_hover_ui(|ui| {
                                                            ui.label(format!(
                                                                "{}: x{}",
                                                                item.name.clone(),
                                                                item.stack_size
                                                            ));
                                                        });
                                                    let mut modified_rect = image.rect;
                                                    modified_rect.min.y +=
                                                        modified_rect.size().y / 2.0;
                                                    ui.allocate_ui_at_rect(modified_rect, |ui| {
                                                        egui::Frame::none()
                                                            .fill(Color32::from_rgba_unmultiplied(
                                                                0, 0, 0, 164,
                                                            ))
                                                            .show(ui, |ui| {
                                                                ui.add(egui::Label::new(format!(
                                                                    "{}",
                                                                    item.stack_size
                                                                )));
                                                            });
                                                    });
                                                    if image.clicked() {
                                                        grab_stack(
                                                            &mut held_items,
                                                            &mut inventory,
                                                            &mut holding,
                                                            hotbar_num,
                                                            item_num,
                                                            true,
                                                        );
                                                    }
                                                } else if ui
                                                    .add(
                                                        egui::widgets::Image::new(
                                                            contexts
                                                                .image_id(
                                                                    loadable_assets
                                                                        .item_textures
                                                                        .get(&"empty".to_string())
                                                                        .unwrap(),
                                                                )
                                                                .unwrap(),
                                                            [48.0, 48.0],
                                                        )
                                                        .tint(color)
                                                        .sense(Sense::click()),
                                                    )
                                                    .clicked()
                                                {
                                                    grab_stack(
                                                        &mut held_items,
                                                        &mut inventory,
                                                        &mut holding,
                                                        hotbar_num,
                                                        item_num,
                                                        true,
                                                    );
                                                }
                                            },
                                        );
                                    });
                                }
                            }
                        });
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.separator();
                    ui.label(format!("Thirst: {}", 100.0));
                    ui.separator();
                    ui.label(format!("Hunger: {}", 100.0));
                    ui.separator();
                    ui.label(format!("Health: {}", 100.0));
                    ui.separator();
                });
            });
        });
}

#[derive(Resource, Default, Clone, Debug, Deref, DerefMut)]
pub struct CurrentItemsHeld(pub Vec<(ItemData, &'static str, usize, usize)>);
#[derive(Resource, Default, Clone, Debug, Deref, DerefMut)]
pub struct Holding(pub bool);

// Item data is the items we are moving importantly also how much we are moving. So to move half a stack we set the item data to half of the source item.
// Right now we only ever have one held but this is a vec for the future for bulk moving

// TODO: Change bar and inventory slots possible to be one big array instead of two seperate. Would make it cleaner to access items
pub fn grab_stack(
    held_items: &mut CurrentItemsHeld,
    inventory: &mut Inventory,
    holding: &mut Holding,
    row_index: usize,
    row_item: usize,
    bar: bool,
) {
    if !**holding {
        if bar {
            if inventory.hotbar[row_index][row_item].is_some() {
                held_items.insert(
                    0,
                    (
                        inventory.hotbar[row_index][row_item].clone().unwrap(),
                        "bar",
                        row_index,
                        row_item,
                    ),
                );
                inventory.hotbar[row_index][row_item] = None;
                **holding = true;
            }
        } else if inventory.slots[row_index][row_item].is_some() {
            held_items.insert(
                0,
                (
                    inventory.slots[row_index][row_item].clone().unwrap(),
                    "inventory",
                    row_index,
                    row_item,
                ),
            );
            inventory.slots[row_index][row_item] = None;
            **holding = true;
        }
    } else {
        if bar {
            if inventory.hotbar[row_index][row_item].is_none() {
                inventory.hotbar[row_index][row_item] =
                    Some(held_items.0.get(0).unwrap().0.clone());
                held_items.remove(0);
            } else {
                let held_item = held_items.get(0).unwrap().clone();
                held_items.remove(0);
                let temp_item = inventory.hotbar[row_index][row_item].clone();
                inventory.hotbar[row_index][row_item] = Some(held_item.0.clone());
                match held_item.1 {
                    "inventory" => {
                        inventory.slots[held_item.2][held_item.3] = temp_item;
                    }
                    "bar" => {
                        inventory.hotbar[held_item.2][held_item.3] = temp_item;
                    }
                    _ => {}
                }
            }
        } else if inventory.slots[row_index][row_item].is_none() {
            inventory.slots[row_index][row_item] = Some(held_items.0.get(0).unwrap().0.clone());
            held_items.remove(0);
        } else {
            let held_item = held_items.get(0).unwrap().clone();
            held_items.remove(0);
            let temp_item = inventory.slots[row_index][row_item].clone();
            inventory.slots[row_index][row_item] = Some(held_item.0.clone());
            match held_item.1 {
                "inventory" => {
                    inventory.slots[held_item.2][held_item.3] = temp_item;
                }
                "bar" => {
                    inventory.hotbar[held_item.2][held_item.3] = temp_item;
                }
                _ => {}
            }
        }
        **holding = false;
    }
}

// TODO: Actually make this work
// pub fn grab_half_stack(
//     held_items: &mut CurrentItemsHeld,
//     inventory: &mut Inventory,
//     holding: &mut Holding,
//     row_index: usize,
//     row_item: usize,
//     bar: bool,
// ) {
// }

pub fn inventory(
    mut player_query: Query<&mut Inventory, With<ControlledPlayer>>,
    mut held_items: ResMut<CurrentItemsHeld>,
    mut holding: ResMut<Holding>,
    mut contexts: EguiContexts,
    _options: Res<GameOptions>,
    loadable_assets: Res<LoadableAssets>,
) {
    let ctx = contexts.ctx_mut().clone();
    let style = ctx.style();
    if let Ok(mut inventory) = player_query.get_single_mut() {
        if inventory.open {
            egui::Window::new("inventory")
                .resizable(false)
                .constrain(true)
                .default_pos([400.0, 200.0])
                .show(&ctx, |ui| {
                    let cloned_inv = inventory.clone();
                    StripBuilder::new(ui)
                        .size(Size::exact(50.0))
                        .size(Size::exact(50.0))
                        .size(Size::exact(50.0))
                        .size(Size::exact(50.0))
                        .size(Size::exact(50.0))
                        .vertical(|mut strip| {
                            for (row_num, row_section) in
                                cloned_inv.slots.iter().cloned().enumerate()
                            {
                                strip.cell(|ui| {
                                    StripBuilder::new(ui)
                                        .size(Size::exact(50.0))
                                        .size(Size::exact(50.0))
                                        .size(Size::exact(50.0))
                                        .size(Size::exact(50.0))
                                        .size(Size::exact(50.0))
                                        .size(Size::exact(50.0))
                                        .size(Size::exact(50.0))
                                        .size(Size::exact(50.0))
                                        .size(Size::exact(50.0))
                                        .horizontal(|mut strip| {
                                            for (item_num, item) in
                                                row_section.iter().clone().enumerate()
                                            {
                                                strip.cell(|ui| {
                                                    let color = if *inventory.current_inv_item
                                                        == item_num
                                                        && *inventory.current_inv_bar == row_num
                                                        && **holding
                                                    {
                                                        Color32::from_white_alpha(128)
                                                    } else if item.is_none() {
                                                        style.visuals.faint_bg_color
                                                    } else {
                                                        Color32::WHITE
                                                    };
                                                    egui::Frame::none()
                                                        .outer_margin(2.0)
                                                        .fill(color)
                                                        .show(ui, |ui| {
                                                            if let Some(item) = item {
                                                            let image = ui
                                                    .add(
                                                        egui::widgets::Image::new(
                                                            contexts
                                                                .image_id(
                                                                    loadable_assets
                                                                        .item_textures
                                                                        .get(&name_to_identifier(
                                                                            item.namespace.clone(),
                                                                            item.name.clone(),
                                                                        ))
                                                                        .unwrap(),
                                                                )
                                                                .unwrap(),
                                                            [48.0, 48.0],
                                                        )
                                                        .tint(color)
                                                        .sense(Sense::click()),
                                                    )
                                                    .on_hover_ui(|ui| {
                                                        ui.label(format!(
                                                            "{}: x{}",
                                                            item.name.clone(),
                                                            item.stack_size
                                                        ));
                                                    });
                                                            let mut modified_rect =
                                                                image.rect;
                                                            modified_rect.min.y +=
                                                                modified_rect.size().y / 2.0;
                                                            ui.allocate_ui_at_rect(
                                                                modified_rect,
                                                                |ui| {
                                                                    egui::Frame::none()
                                                        .fill(Color32::from_rgba_unmultiplied(
                                                            0, 0, 0, 164,
                                                        ))
                                                        .show(ui, |ui| {
                                                            ui.add(egui::Label::new(format!(
                                                                "{}",
                                                                item.stack_size
                                                            )));
                                                        });
                                                                },
                                                            );
                                                            if image.clicked() {
                                                                inventory.current_inv_item = CurrentInvItem(item_num);
                                                                inventory.current_inv_bar = CurrentInvBar(row_num);
                                                                grab_stack(
                                                                    &mut held_items,
                                                                    &mut inventory,
                                                                    &mut holding,
                                                                    row_num,
                                                                    item_num,
                                                                    false,
                                                                );
                                                            }
                                                        } else if ui
                                                            .add(
                                                                egui::widgets::Image::new(
                                                                    contexts
                                                                        .image_id(
                                                                            loadable_assets
                                                                                .item_textures
                                                                                .get(
                                                                                    &"empty"
                                                                                        .to_string(
                                                                                        ),
                                                                                )
                                                                                .unwrap(),
                                                                        )
                                                                        .unwrap(),
                                                                    [48.0, 48.0],
                                                                )
                                                                .tint(color)
                                                                .sense(Sense::click()),
                                                            )
                                                            .clicked()
                                                        {
                                                            inventory.current_inv_item = CurrentInvItem(item_num);
                                                            inventory.current_inv_bar = CurrentInvBar(row_num);
                                                            grab_stack(
                                                                &mut held_items,
                                                                &mut inventory,
                                                                &mut holding,
                                                                row_num,
                                                                item_num,
                                                                false,
                                                            );
                                                        }
                                                        });
                                                });
                                            }
                                        });
                                });
                            }
                        });
                });
        }
    }
}
