use std::collections::BTreeMap;

use bevy::prelude::*;
use bevy_egui::{egui::FontId, *};
use vinox_common::world::chunks::positions::world_to_offsets;

use crate::states::{
    components::GameOptions,
    game::world::chunks::{PlayerBlock, PlayerChunk},
};

pub fn debug(
    mut contexts: EguiContexts,
    // mut windows: Query<&mut Window>,
    options: Res<GameOptions>,
    player_chunk: Res<PlayerChunk>,
    player_block: Res<PlayerBlock>,
) {
    if !options.dark_theme {
        catppuccin_egui::set_theme(contexts.ctx_mut(), catppuccin_egui::MOCHA);
    }
    if options.debug {
        egui::Window::new("Debug").show(contexts.ctx_mut(), |ui| {
            ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
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
                egui::ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .max_width(2000.0)
                    .show(ui, |ui| {
                        // HERE: This is where you put your own debug
                        ui.separator();
                        ui.label(format!("Player Chunk: {}", player_chunk.chunk_pos));
                        ui.label(format!("Player Global Block: {}", player_block.pos));
                        ui.label(format!(
                            "Player Local Block: {}",
                            world_to_offsets(player_block.pos.as_vec3())
                        ));
                    });
            });
        });
    }
}
