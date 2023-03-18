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
    player_query: Query<&Inventory, With<ControlledPlayer>>,
    mut contexts: EguiContexts,
    options: Res<GameOptions>,
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
                if let Ok(inventory) = player_query.get_single() {
                    for (hotbar_num, hotbar_section) in inventory.hotbar.iter().cloned().enumerate()
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
                                        ui.label(format!("{}: {}", item.name, item.stack_size));
                                    } else {
                                        ui.label("None");
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
pub struct CurrentItemsHeld(pub Vec<(ItemData, String, usize, usize)>);

pub fn inventory(
    mut player_query: Query<&mut Inventory, With<ControlledPlayer>>,
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
                                            }
                                        } else {
                                            if ui
                                                .add(egui::Label::new("None").sense(Sense::click()))
                                                .clicked()
                                            {
                                                inventory.current_inv_item =
                                                    CurrentInvItem(item_num);
                                                inventory.current_inv_bar = CurrentInvBar(row_num);
                                            }
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
