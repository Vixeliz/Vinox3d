use leafwing_input_manager::{prelude::ActionState, InputManagerBundle};
use std::collections::BTreeMap;
use vinox_server::create_server;

use bevy::{
    input::{keyboard::KeyboardInput, mouse::MouseButtonInput, ButtonState},
    prelude::*,
    window::{PresentMode, PrimaryWindow},
};
use bevy_egui::{
    egui::{self, FontId, Rounding},
    EguiContexts, EguiSettings,
};
use vinox_common::networking::protocol::NetworkIP;

use crate::states::components::{
    save_game_options, GameActions, GameOptions, GameState, Menu, ProjectPath,
};

#[derive(Resource, Default, Deref, DerefMut)]
pub struct InOptions(pub bool);

pub fn configure_visuals(mut contexts: EguiContexts) {
    contexts.ctx_mut().set_visuals(egui::Visuals {
        window_rounding: Rounding::from(0.0),
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

pub fn save_options(options: Res<GameOptions>, project_path: Res<ProjectPath>) {
    if options.is_changed() {
        save_game_options(options.clone(), project_path.clone());
    }
}

pub fn options(
    mut contexts: EguiContexts,
    mut in_options: ResMut<InOptions>,
    mut options: ResMut<GameOptions>,
    mut current_change: Local<Option<GameActions>>,
    mut keys: EventReader<KeyboardInput>,
    mut mouse_buttons: EventReader<MouseButtonInput>,
    mut windows: Query<&mut Window>,
) {
    contexts.ctx_mut().set_style(egui::Style {
        text_styles: {
            let mut texts = BTreeMap::new();
            texts.insert(egui::style::TextStyle::Small, FontId::monospace(14.0));
            texts.insert(egui::style::TextStyle::Body, FontId::monospace(14.0));
            texts.insert(egui::style::TextStyle::Heading, FontId::monospace(16.0));
            texts.insert(egui::style::TextStyle::Monospace, FontId::monospace(14.0));
            texts.insert(egui::style::TextStyle::Button, FontId::monospace(14.0));
            texts
        },
        ..Default::default()
    });
    if **in_options {
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
            .open(&mut in_options)
            .show(contexts.ctx_mut(), |ui| {
                ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                    egui::ScrollArea::vertical()
                        .auto_shrink([false; 2])
                        .max_width(2000.0)
                        .show(ui, |ui| {
                            for (input, action) in options.input.iter() {
                                ui.horizontal(|ui| {
                                    ui.label(format!("{action:?}"));
                                    if let Some(input) = input.get_at(0) {
                                        if let Some(key) = input.raw_inputs().keycodes.get(0) {
                                            if ui.small_button(format!("{key:?}")).clicked() {
                                                *current_change = Some(action);
                                            }
                                        } else if let Some(mouse) =
                                            input.raw_inputs().mouse_buttons.get(0)
                                        {
                                            if ui.small_button(format!("{mouse:?}")).clicked() {
                                                *current_change = Some(action);
                                            }
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
                                ui.label("Standard Hotbar: ");
                                if ui
                                    .small_button(format!("{}", options.standard_bar))
                                    .clicked()
                                {
                                    options.standard_bar = !options.standard_bar;
                                }
                            });
                            ui.separator();
                            ui.horizontal(|ui| {
                                ui.label("Looking At Window: ");
                                if ui.small_button(format!("{}", options.looking_at)).clicked() {
                                    options.looking_at = !options.looking_at;
                                }
                            });
                            ui.separator();
                            ui.horizontal(|ui| {
                                ui.label("FOV: ");
                                ui.add(egui::Slider::new(&mut options.fov, 30.0..=120.0));
                            });
                            ui.separator();
                            ui.horizontal(|ui| {
                                ui.label("Max meshes per frame: ");
                                ui.add(egui::Slider::new(&mut options.meshes_frame, 64..=2048));
                            });
                            ui.separator();
                            ui.horizontal(|ui| {
                                ui.label("Vsync: ");
                                if ui.small_button(format!("{}", options.vsync)).clicked() {
                                    options.vsync = !options.vsync;
                                    let mut window = windows.single_mut();
                                    window.present_mode = if options.vsync {
                                        PresentMode::AutoVsync
                                    } else {
                                        PresentMode::AutoNoVsync
                                    };
                                }
                            });
                            ui.separator();
                        });
                });
            });
    }
}

#[allow(clippy::too_many_arguments)]
pub fn create_ui(
    mut commands: Commands,
    mut contexts: EguiContexts,
    mut ip_res: ResMut<NetworkIP>,
    mut in_options: ResMut<InOptions>,
    mut options: ResMut<GameOptions>,
    asset_server: ResMut<AssetServer>,
    mut rendered_texture_id: Local<egui::TextureId>,
    mut is_initialized: Local<bool>,
) {
    if !options.dark_theme {
        catppuccin_egui::set_theme(contexts.ctx_mut(), catppuccin_egui::MOCHA);
    }

    if !*is_initialized {
        *is_initialized = true;
        *rendered_texture_id = contexts.add_image(asset_server.load("cover.png").clone_weak());
    }
    egui::SidePanel::left("menu_side_panel")
        .default_width(250.0)
        .show(contexts.ctx_mut(), |ui| {
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                ui.heading("Vinox");

                ui.allocate_space(egui::Vec2::new(1.0, 100.0));

                ui.horizontal(|ui| {
                    ui.label("IP: ");
                    ui.text_edit_singleline(&mut ip_res.0);
                });

                ui.horizontal(|ui| {
                    ui.label("Username: ");
                    ui.text_edit_singleline(&mut options.user_name);
                });

                ui.allocate_space(egui::Vec2::new(1.0, 26.0));

                if ui.button("Start").clicked() {
                    commands.insert_resource(NextState(Some(GameState::Loading)));
                }

                ui.allocate_space(egui::Vec2::new(1.0, 26.0));

                if ui.button("Singleplayer").clicked() {
                    std::thread::spawn(|| {
                        create_server();
                    });
                    commands.insert_resource(NextState(Some(GameState::Loading)));
                }

                ui.allocate_space(egui::Vec2::new(1.0, 26.0));

                if ui.button("Options").clicked() {
                    **in_options = !**in_options;
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
        let ratio = 16.0 / 10.0;
        let (width, height) = (ui.available_height() * ratio, ui.available_height());
        ui.add(egui::widgets::Image::new(
            *rendered_texture_id,
            [width, height],
        ));
        ui.separator();
    });
}

pub fn ui_events() {}

pub fn start(mut commands: Commands, options: Res<GameOptions>) {
    commands.spawn((Camera2dBundle::default(), Menu));
}
