use std::collections::BTreeMap;

use bevy::prelude::*;
use bevy_egui::{egui::FontId, *};
use vinox_common::ecs::bundles::Inventory;

use crate::states::{components::GameOptions, game::world::chunks::ControlledPlayer};

pub fn status_bar(
    player_query: Query<&Inventory, With<ControlledPlayer>>,
    mut contexts: EguiContexts,
    options: Res<GameOptions>,
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
                    for hotbar_section in inventory.hotbar.iter().cloned() {
                        ui.separator();
                        for item in hotbar_section.iter().clone() {
                            ui.label(format!("{}: {}", item.name, item.stack_size));
                            ui.separator();
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

pub fn inventory() {}
