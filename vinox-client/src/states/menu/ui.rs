use std::collections::BTreeMap;

use bevy::{
    input::{keyboard::KeyboardInput, mouse::MouseButtonInput, ButtonState},
    prelude::*,
    window::PrimaryWindow,
};
use bevy_egui::{
    egui::{self, FontId},
    EguiContexts, EguiSettings,
};
use vinox_common::networking::protocol::NetworkIP;

use crate::states::{
    components::{GameActions, GameOptions, GameState, Menu},
    game::networking::components::UserName,
};

#[derive(Resource, Default)]
pub struct InOptions(pub bool);

pub fn configure_visuals(mut contexts: EguiContexts) {
    contexts.ctx_mut().set_visuals(egui::Visuals {
        window_rounding: 0.0.into(),
        ..Default::default()
    });
}

pub fn update_ui_scale_factor(
    keyboard_input: Res<Input<KeyCode>>,
    mut toggle_scale_factor: Local<Option<bool>>,
    mut egui_settings: ResMut<EguiSettings>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    if keyboard_input.just_pressed(KeyCode::Slash) || toggle_scale_factor.is_none() {
        *toggle_scale_factor = Some(!toggle_scale_factor.unwrap_or(false));

        if let Ok(window) = windows.get_single() {
            let scale_factor = if toggle_scale_factor.unwrap() {
                1.0
            } else {
                1.0 / window.scale_factor()
            };
            egui_settings.scale_factor = scale_factor;
        }
    }
}

pub fn options(
    mut contexts: EguiContexts,
    mut in_options: ResMut<InOptions>,
    mut options: ResMut<GameOptions>,
    mut current_change: Local<Option<GameActions>>,
    mut keys: EventReader<KeyboardInput>,
    mut mouse_buttons: EventReader<MouseButtonInput>,
) {
    if in_options.0 {
        if let Some(current_action) = *current_change {
            if let Some(keyboard_input) = keys.iter().next() {
                if keyboard_input.state == ButtonState::Released {
                    if let Some(key_code) = keyboard_input.key_code {
                        options.input.clear_action(current_action);
                        options.input.insert(key_code, current_action);
                        *current_change = None;
                    }
                }
            }
            if let Some(mouse_input) = mouse_buttons.iter().next() {
                if mouse_input.state == ButtonState::Released {
                    options.input.clear_action(current_action);
                    options.input.insert(mouse_input.button, current_action);
                    *current_change = None;
                }
            }
        }
        if !options.dark_theme {
            catppuccin_egui::set_theme(contexts.ctx_mut(), catppuccin_egui::MOCHA);
        }
        egui::Window::new("Options")
            .open(&mut in_options.0)
            .show(contexts.ctx_mut(), |ui| {
                ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                    ui.ctx().set_style(egui::Style {
                        text_styles: {
                            let mut texts = BTreeMap::new();
                            texts.insert(egui::style::TextStyle::Small, FontId::proportional(16.0));
                            texts.insert(egui::style::TextStyle::Body, FontId::proportional(16.0));
                            texts.insert(
                                egui::style::TextStyle::Heading,
                                FontId::proportional(36.0),
                            );
                            texts
                                .insert(egui::style::TextStyle::Monospace, FontId::monospace(16.0));
                            texts
                                .insert(egui::style::TextStyle::Button, FontId::proportional(26.0));
                            texts
                        },
                        ..Default::default()
                    });
                    egui::ScrollArea::vertical()
                        .auto_shrink([false; 2])
                        .max_width(2000.0)
                        .show(ui, |ui| {
                            for (input, action) in options.input.iter() {
                                ui.horizontal(|ui| {
                                    ui.label(format!("{action:?}"));
                                    if let Some(key) =
                                        input.get_at(0).unwrap().raw_inputs().keycodes.get(0)
                                    {
                                        if ui.small_button(format!("{key:?}")).clicked() {
                                            *current_change = Some(action);
                                        }
                                    } else if let Some(mouse) =
                                        input.get_at(0).unwrap().raw_inputs().mouse_buttons.get(0)
                                    {
                                        if ui.small_button(format!("{mouse:?}")).clicked() {
                                            *current_change = Some(action);
                                        }
                                    };
                                });
                                ui.separator();
                            }
                            ui.horizontal(|ui| {
                                ui.label("Dark mode: ");
                                if ui.small_button(format!("{}", options.dark_theme)).clicked() {
                                    options.dark_theme = !options.dark_theme;
                                }
                            });
                            ui.separator();
                            ui.horizontal(|ui| {
                                ui.label("FOV: ");
                                ui.add(egui::Slider::new(&mut options.fov, 30.0..=120.0));
                            });
                        });
                });
            });
    }
}

pub fn create_ui(
    mut commands: Commands,
    mut contexts: EguiContexts,
    mut ip_res: ResMut<NetworkIP>,
    mut username_res: ResMut<UserName>,
    mut in_options: ResMut<InOptions>,
    options: Res<GameOptions>,
) {
    if !options.dark_theme {
        catppuccin_egui::set_theme(contexts.ctx_mut(), catppuccin_egui::MOCHA);
    }
    egui::SidePanel::left("menu_side_panel")
        .default_width(250.0)
        .show(contexts.ctx_mut(), |ui| {
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

                ui.allocate_space(egui::Vec2::new(1.0, 100.0));

                ui.horizontal(|ui| {
                    ui.label("IP: ");
                    ui.text_edit_singleline(&mut ip_res.0);
                });

                ui.horizontal(|ui| {
                    ui.label("Username: ");
                    ui.text_edit_singleline(&mut username_res.0);
                });

                ui.allocate_space(egui::Vec2::new(1.0, 26.0));

                if ui.button("Start").clicked() {
                    commands.insert_resource(NextState(Some(GameState::Loading)));
                }

                ui.allocate_space(egui::Vec2::new(1.0, 26.0));

                if ui.button("Options").clicked() {
                    in_options.0 = !in_options.0;
                }

                ui.allocate_space(egui::Vec2::new(1.0, 100.0));
            });
            ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                ui.add(egui::Hyperlink::from_label_and_url(
                    "made by vixeliz",
                    "https://github.com/vixeliz/vinox/",
                ));
            });
        });

    if !options.dark_theme {
        catppuccin_egui::set_theme(contexts.ctx_mut(), catppuccin_egui::MOCHA);
    }
    egui::CentralPanel::default().show(contexts.ctx_mut(), |ui| {
        egui::warn_if_debug_build(ui);

        ui.separator();
    });
}

pub fn ui_events() {}

pub fn start(mut commands: Commands) {
    commands.spawn((Camera2dBundle::default(), Menu));
}
