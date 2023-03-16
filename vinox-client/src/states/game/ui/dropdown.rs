use std::collections::BTreeMap;

use bevy::prelude::*;
use bevy_egui::{egui::FontId, *};

#[derive(Resource, Default)]
pub struct ConsoleOpen(pub bool);

pub fn create_ui(
    mut commands: Commands,
    mut contexts: EguiContexts,
    is_open: Res<ConsoleOpen>, // mut username_res: ResMut<UserName>,
    mut current_message: Local<String>,
) {
    if is_open.0 {
        catppuccin_egui::set_theme(contexts.ctx_mut(), catppuccin_egui::MOCHA);
        egui::TopBottomPanel::top("menu_side_panel").show(contexts.ctx_mut(), |ui| {
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                ui.ctx().set_style(egui::Style {
                    text_styles: {
                        let mut texts = BTreeMap::new();
                        texts.insert(egui::style::TextStyle::Small, FontId::proportional(16.0));
                        texts.insert(egui::style::TextStyle::Body, FontId::proportional(16.0));
                        texts.insert(egui::style::TextStyle::Heading, FontId::proportional(36.0));
                        texts.insert(egui::style::TextStyle::Monospace, FontId::monospace(16.0));
                        texts.insert(egui::style::TextStyle::Button, FontId::proportional(26.0));
                        texts
                    },
                    ..Default::default()
                });
                ui.heading("Vinox");

                ui.horizontal(|ui| {
                    ui.label("Type: ");
                    ui.text_edit_singleline(&mut *current_message);
                });
            });
        });
    }
}
