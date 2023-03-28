use std::collections::BTreeMap;

use bevy::prelude::*;
use bevy_egui::{
    egui::{Align2, FontId},
    *,
};
use vinox_common::world::chunks::{positions::world_to_offsets, storage::name_to_identifier};

use crate::states::{
    components::GameOptions,
    game::world::chunks::{PlayerBlock, PlayerChunk, PlayerDirection, PlayerTargetedBlock},
};

pub fn debug(
    mut contexts: EguiContexts,
    // mut windows: Query<&mut Window>,
    options: Res<GameOptions>,
    player_chunk: Res<PlayerChunk>,
    player_block: Res<PlayerBlock>,
    player_direction: Res<PlayerDirection>,
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
                        ui.label(format!("Player Facing Direction: {:?}", **player_direction));
                    });
            });
        });
    }
}

pub fn targeted_block(
    mut contexts: EguiContexts,
    options: Res<GameOptions>,
    player_looking: Res<PlayerTargetedBlock>,
) {
    if !options.dark_theme {
        catppuccin_egui::set_theme(contexts.ctx_mut(), catppuccin_egui::MOCHA);
    }
    if options.looking_at {
        contexts.ctx_mut().set_style(egui::Style {
            text_styles: {
                let mut texts = BTreeMap::new();
                texts.insert(egui::style::TextStyle::Small, FontId::proportional(14.0));
                texts.insert(egui::style::TextStyle::Body, FontId::proportional(14.0));
                texts.insert(egui::style::TextStyle::Heading, FontId::proportional(16.0));
                texts.insert(egui::style::TextStyle::Monospace, FontId::monospace(14.0));
                texts.insert(egui::style::TextStyle::Button, FontId::proportional(14.0));
                texts
            },
            ..Default::default()
        });
        egui::Window::new("Targeted Block")
            .anchor(Align2::CENTER_TOP, [0.0, 0.0])
            .default_width(200.0)
            .collapsible(false)
            .constrain(true)
            .vscroll(true)
            .show(contexts.ctx_mut(), |ui| {
                ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                    if let Some(block) = player_looking.clone() {
                        let identifier = name_to_identifier(block.namespace, block.name);
                        ui.label(format!("{identifier}"));
                    } else {
                        ui.label("No Block");
                    }
                });
            });
    }
}
