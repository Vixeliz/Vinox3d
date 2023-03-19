use std::collections::BTreeMap;

use bevy::prelude::*;
use bevy_egui::{
    egui::{Color32, FontId, Sense},
    *,
};
use vinox_common::{
    ecs::bundles::{CurrentInvBar, CurrentInvItem, Inventory},
    storage::items::descriptor::ItemData,
};

use crate::states::{components::GameOptions, game::world::chunks::ControlledPlayer};

pub fn status_bar(
    mut player_query: Query<&mut Inventory, With<ControlledPlayer>>,
    mut contexts: EguiContexts,
    options: Res<GameOptions>,
    mut held_items: ResMut<CurrentItemsHeld>,
    mut holding: ResMut<Holding>,
    // mut texture_ids: Local<[Option<egui::TextureId>; 9]>,
) {
    if !options.dark_theme {
        catppuccin_egui::set_theme(contexts.ctx_mut(), catppuccin_egui::MOCHA);
    }

    egui::TopBottomPanel::bottom("status_bar")
        .default_height(40.0)
        .max_height(75.0)
        .show(contexts.ctx_mut(), |ui| {
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                ui.ctx().set_style(egui::Style {
                    text_styles: {
                        let mut texts = BTreeMap::new();
                        texts.insert(egui::style::TextStyle::Small, FontId::proportional(18.0));
                        texts.insert(egui::style::TextStyle::Body, FontId::proportional(18.0));
                        texts.insert(egui::style::TextStyle::Heading, FontId::proportional(20.0));
                        texts.insert(egui::style::TextStyle::Monospace, FontId::monospace(18.0));
                        texts.insert(egui::style::TextStyle::Button, FontId::proportional(18.0));
                        texts
                    },
                    ..Default::default()
                });
                if let Ok(mut inventory) = player_query.get_single_mut() {
                    for (hotbar_num, hotbar_section) in
                        inventory.clone().hotbar.iter().cloned().enumerate()
                    {
                        ui.separator();
                        for (item_num, item) in hotbar_section.iter().clone().enumerate() {
                            let color = if *inventory.current_item == item_num
                                && *inventory.current_bar == hotbar_num
                            {
                                Color32::WHITE
                            } else {
                                ui.style().visuals.window_fill
                            };

                            egui::Frame::none()
                                .fill(color)
                                .outer_margin(2.0)
                                .show(ui, |ui| {
                                    ui.separator();
                                    if let Some(item) = item {
                                        if ui
                                            .add(
                                                egui::Label::new(format!(
                                                    "{}: {}",
                                                    item.name, item.stack_size
                                                ))
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
                                    } else if ui
                                        .add(egui::Label::new("None").sense(Sense::click()))
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
                                    ui.separator();
                                });
                        }
                    }
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.separator();
                    ui.label(format!("Thirst: {}", 100.0));
                    ui.separator();
                    ui.label(format!("Hunger: {}", 100.0));
                    ui.separator();
                    ui.label(format!("Health: {}", 100.0));
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
    options: Res<GameOptions>,
) {
    if !options.dark_theme {
        catppuccin_egui::set_theme(contexts.ctx_mut(), catppuccin_egui::MOCHA);
    }
    if let Ok(mut inventory) = player_query.get_single_mut() {
        if inventory.open {
            egui::Window::new("inventory").show(contexts.ctx_mut(), |ui| {
                ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                    ui.ctx().set_style(egui::Style {
                        text_styles: {
                            let mut texts = BTreeMap::new();
                            texts.insert(egui::style::TextStyle::Small, FontId::proportional(18.0));
                            texts.insert(egui::style::TextStyle::Body, FontId::proportional(18.0));
                            texts.insert(
                                egui::style::TextStyle::Heading,
                                FontId::proportional(20.0),
                            );
                            texts
                                .insert(egui::style::TextStyle::Monospace, FontId::monospace(18.0));
                            texts
                                .insert(egui::style::TextStyle::Button, FontId::proportional(18.0));
                            texts
                        },
                        ..Default::default()
                    });
                    let cloned_inv = inventory.clone();
                    for (row_num, row_section) in cloned_inv.slots.iter().cloned().enumerate() {
                        ui.separator();
                        ui.horizontal(|ui| {
                            for (item_num, item) in row_section.iter().clone().enumerate() {
                                let color = if *inventory.current_inv_item == item_num
                                    && *inventory.current_inv_bar == row_num
                                    && **holding
                                {
                                    Color32::WHITE
                                } else {
                                    ui.style().visuals.window_fill
                                };

                                egui::Frame::none()
                                    .fill(color)
                                    .outer_margin(2.0)
                                    .show(ui, |ui| {
                                        ui.separator();
                                        if let Some(item) = item {
                                            if ui
                                                .add(
                                                    egui::Label::new(format!(
                                                        "{}: {}",
                                                        item.name, item.stack_size
                                                    ))
                                                    .sense(Sense::click()),
                                                )
                                                .clicked()
                                            {
                                                inventory.current_inv_item =
                                                    CurrentInvItem(item_num);
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
                                            .add(egui::Label::new("None").sense(Sense::click()))
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
                                        ui.separator();
                                    });
                            }
                        });
                        ui.separator();
                    }
                });
            });
        }
    }
}
