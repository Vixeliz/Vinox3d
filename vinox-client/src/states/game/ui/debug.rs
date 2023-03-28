use std::collections::BTreeMap;

use bevy::prelude::*;
use bevy_egui::{
    egui::{Align2, FontId},
    *,
};
use vinox_common::world::chunks::{positions::world_to_offsets, storage::name_to_identifier};

use crate::states::{
    components::GameOptions,
    game::world::chunks::{
        ControlledPlayer, PlayerBlock, PlayerChunk, PlayerDirection, PlayerTargetedBlock,
    },
};

pub fn debug(
    mut contexts: EguiContexts,
    // mut windows: Query<&mut Window>,
    options: Res<GameOptions>,
    player_chunk: Res<PlayerChunk>,
    player_block: Res<PlayerBlock>,
    player_direction: Res<PlayerDirection>,
    player_query: Query<&Transform, With<ControlledPlayer>>,
) {
    if !options.dark_theme {
        catppuccin_egui::set_theme(contexts.ctx_mut(), catppuccin_egui::MOCHA);
    }
    if options.debug {
        if let Ok(player_transform) = player_query.get_single() {
            let style = contexts.ctx_mut().style().clone();
            egui::Window::new("Debug")
                .frame(egui::Frame {
                    fill: egui::Color32::from_rgba_unmultiplied(0, 0, 0, 224),
                    rounding: style.visuals.window_rounding,
                    ..Default::default()
                })
                .show(contexts.ctx_mut(), |ui| {
                    ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                        egui::ScrollArea::vertical()
                            .auto_shrink([false; 2])
                            .max_width(2000.0)
                            .show(ui, |ui| {
                                // HERE: This is where you put your own debug
                                ui.separator();
                                ui.label(format!("{:<20}{}", "Chunk", player_chunk.chunk_pos));
                                ui.separator();
                                ui.label(format!("{:<15}{}", "Global Block", player_block.pos));
                                ui.separator();
                                ui.label(format!(
                                    "{:<14}{}",
                                    "Chunk Block",
                                    world_to_offsets(player_block.pos.as_vec3())
                                ));
                                ui.separator();
                                ui.label(format!(
                                    "{:<17}{}",
                                    "Raw Pos", player_transform.translation
                                ));
                                ui.separator();
                                ui.label(format!("{:<20}{:?}", "Facing", **player_direction));
                            });
                    });
                });
        }
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
        egui::Window::new("Targeted Block")
            .anchor(Align2::CENTER_TOP, [0.0, 0.0])
            .default_width(200.0)
            .collapsible(false)
            .constrain(true)
            .vscroll(true)
            .show(contexts.ctx_mut(), |ui| {
                ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                    if let Some(block) = player_looking.block.clone() {
                        if let Some(pos) = player_looking.pos.clone() {
                            let identifier = name_to_identifier(block.namespace, block.name);
                            ui.label(format!("{identifier}"));
                            ui.label(format!("Pos: {}, {}, {}", pos.x, pos.y, pos.z));
                        }
                    } else {
                        ui.label("No Block");
                    }
                });
            });
    }
}
