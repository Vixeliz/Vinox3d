use bevy_quinnet::client::Client;
use brigadier_rs::*;
use egui_notify::Toasts;
use std::{collections::BTreeMap, convert::Infallible};
use vinox_common::networking::protocol::ClientMessage;

use bevy::prelude::*;
use bevy_egui::{
    egui::{Align2, FontId},
    *,
};

use crate::states::{components::GameOptions, game::networking::components::ChatMessages};

#[derive(Resource, Default, Deref, DerefMut)]
pub struct ConsoleOpen(pub bool);

#[derive(Resource, Default, Deref, DerefMut)]
pub struct Toast(pub Toasts);

pub fn create_ui(
    // mut commands: Commands,
    mut client: ResMut<Client>,
    is_open: Res<ConsoleOpen>, // mut username_res: ResMut<UserName>,
    mut current_message: Local<String>,
    mut messages: ResMut<ChatMessages>,
    mut contexts: EguiContexts,
    mut toast: ResMut<Toast>,
    options: Res<GameOptions>,
) {
    if !options.dark_theme {
        catppuccin_egui::set_theme(contexts.ctx_mut(), catppuccin_egui::MOCHA);
    }
    toast.show(contexts.ctx_mut());
    if **is_open {
        let parser = literal("/add")
            .then(integer_i32("integer").build_exec(|_ctx: (), bar| {
                println!("Integer is {bar}");
                Ok::<(), Infallible>(())
            }))
            .build_exec(|_ctx: ()| {
                println!("Called foo with no arguments");
                Ok::<(), Infallible>(())
            });
        if !options.dark_theme {
            catppuccin_egui::set_theme(contexts.ctx_mut(), catppuccin_egui::MOCHA);
        }
        egui::Window::new("Console")
            .anchor(Align2::CENTER_TOP, [0.0, 0.0])
            .default_width(1000.0)
            .collapsible(false)
            .constrain(true)
            .vscroll(true)
            .show(contexts.ctx_mut(), |ui| {
                ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                    ui.ctx().set_style(egui::Style {
                        text_styles: {
                            let mut texts = BTreeMap::new();
                            texts.insert(egui::style::TextStyle::Small, FontId::proportional(16.0));
                            texts.insert(egui::style::TextStyle::Body, FontId::proportional(16.0));
                            texts.insert(
                                egui::style::TextStyle::Heading,
                                FontId::proportional(20.0),
                            );
                            texts
                                .insert(egui::style::TextStyle::Monospace, FontId::monospace(16.0));
                            texts
                                .insert(egui::style::TextStyle::Button, FontId::proportional(16.0));
                            texts
                        },
                        ..Default::default()
                    });

                    egui::TopBottomPanel::bottom("text_box")
                        .resizable(false)
                        .show_inside(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label("Type: ");
                                let response = ui.text_edit_singleline(&mut *current_message);

                                // Pressing enter makes we lose focus
                                let input_send = response.lost_focus()
                                    && ui.input(|input| input.key_pressed(egui::Key::Enter));
                                if input_send {
                                    if let Ok((result, _)) = parser.parse((), &current_message) {
                                        messages
                                            .push(("Console".to_string(), result.to_string()));
                                    } else {
                                        client.connection_mut().try_send_message(
                                            ClientMessage::ChatMessage {
                                                message: current_message.to_string(),
                                            },
                                        );
                                        current_message.clear();
                                    }
                                }
                            });
                        });

                    egui::ScrollArea::vertical()
                        .auto_shrink([false; 2])
                        .max_width(2000.0)
                        .show(ui, |ui| {
                            //TODO: replace with real chat messages
                            for (username, message) in messages.iter() {
                                ui.label(format!("{username}: {message}"));
                            }
                        });
                });
            });
    }
}
