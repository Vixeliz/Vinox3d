use directories::ProjectDirs;
use load_file::load_bytes;
use std::{collections::BTreeMap, path::PathBuf};
use vinox_server::create_server;

use bevy::{
    input::{keyboard::KeyboardInput, mouse::MouseButtonInput, ButtonState},
    prelude::*,
    window::{PresentMode, PrimaryWindow},
};
use bevy_egui::{
    egui::{
        self,
        epaint::Shadow,
        style::{Selection, Spacing, WidgetVisuals, Widgets},
        Color32, FontData, FontDefinitions, FontFamily, FontId, Margin, Rounding, Stroke, Visuals,
    },
    EguiContexts, EguiSettings,
};
use vinox_common::networking::protocol::NetworkIP;

use crate::states::{
    components::{save_game_options, GameActions, GameOptions, GameState, Menu, ProjectPath},
    game::networking::components::Password,
};

#[derive(Resource, Default, Deref, DerefMut)]
pub struct InOptions(pub bool);

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
    // egui_theme: Res<EguiTheme>,
) {
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
    mut password: ResMut<Password>,
    mut options: ResMut<GameOptions>,
    _asset_server: ResMut<AssetServer>,
    _rendered_texture_id: Local<egui::TextureId>,
    mut is_initialized: Local<bool>,
) {
    if !*is_initialized {
        *is_initialized = true;
        // *rendered_texture_id = contexts.add_image(asset_server.load("cover.png").clone_weak());
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

                ui.horizontal(|ui| {
                    ui.label("Password: ");
                    ui.text_edit_singleline(&mut **password);
                    // Pressing enter makes we lose focus
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

    egui::CentralPanel::default().show(contexts.ctx_mut(), |ui| {
        egui::warn_if_debug_build(ui);
        let ratio = 16.0 / 10.0;
        let (_width, _height) = (ui.available_height() * ratio, ui.available_height());
        ui.separator();
        // ui.add(egui::widgets::Image::new(
        //     *rendered_texture_id,
        //     [width, height],
        // ));
    });
}

pub fn ui_events() {}

pub fn start(mut commands: Commands, _options: Res<GameOptions>, mut contexts: EguiContexts) {
    let asset_path = if let Some(proj_dirs) = ProjectDirs::from("com", "vinox", "vinox") {
        let full_path = proj_dirs
            .data_dir()
            .join("assets")
            .join("CozetteVector.ttf");
        full_path
    } else {
        let mut path = PathBuf::new();
        path.push("assets");
        path
    };

    let asset_string = asset_path.as_os_str().to_str().unwrap();
    let font_bytes = load_bytes!(asset_string);
    const TITLE_FONT_NAME: &str = "Monocraft";
    // const TITLE_FONT_NAME: &str = "cozettevector";
    let mut fonts = FontDefinitions::default();
    fonts
        .font_data
        .insert(TITLE_FONT_NAME.into(), FontData::from_static(font_bytes));
    fonts
        .families
        .entry(FontFamily::Name(TITLE_FONT_NAME.into()))
        .or_default()
        .push(TITLE_FONT_NAME.into());
    fonts
        .families
        .entry(FontFamily::Monospace)
        .or_default()
        .push(TITLE_FONT_NAME.into());
    fonts
        .families
        .entry(FontFamily::Proportional)
        .or_default()
        .insert(0, TITLE_FONT_NAME.into());

    let dark_widgets = WidgetVisuals {
        rounding: Rounding::same(0.0),
        bg_fill: Color32::from_rgb(64, 64, 64),
        bg_stroke: Stroke::new(1.0, Color32::from_rgb(90, 90, 90)),
        fg_stroke: Stroke::new(2.0, Color32::from_rgb(200, 200, 200)),
        expansion: 1.0,
        weak_bg_fill: Color32::from_rgb(64, 64, 64),
    };
    let style = egui::Style {
        spacing: Spacing {
            button_padding: egui::Vec2::new(10.0, 10.0),
            item_spacing: egui::Vec2::new(10.0, 10.0),
            window_margin: Margin::same(10.0),
            interact_size: egui::Vec2::new(40.0, 18.0),
            combo_height: 200.0,
            indent: 28.0,
            slider_width: 150.0,
            text_edit_width: 280.0,
            scroll_bar_width: 8.0,
            tooltip_width: 500.0,
            ..Default::default()
        },
        wrap: Some(false),
        visuals: Visuals {
            dark_mode: true,
            faint_bg_color: Color32::from_rgb(64, 64, 64),
            extreme_bg_color: Color32::from_rgb(48, 48, 48),
            code_bg_color: Color32::from_rgb(72, 72, 72),
            selection: Selection {
                bg_fill: Color32::from_rgb(115, 115, 115),
                stroke: Default::default(),
            },
            widgets: Widgets {
                noninteractive: WidgetVisuals {
                    bg_fill: Color32::from_rgb(90, 90, 90),
                    ..dark_widgets
                },
                inactive: WidgetVisuals {
                    bg_fill: Color32::from_rgb(100, 100, 100),
                    ..dark_widgets
                },
                hovered: dark_widgets,
                active: dark_widgets,
                open: dark_widgets,
            },
            window_rounding: Rounding::same(0.0),
            window_shadow: Shadow::small_dark(),
            popup_shadow: Shadow::small_dark(),
            resize_corner_size: 12.0,
            clip_rect_margin: 3.0,
            button_frame: true,
            collapsing_header_frame: false,
            hyperlink_color: Color32::from_rgb(110, 100, 110),

            override_text_color: None,
            text_cursor_width: 0.0,
            text_cursor_preview: false,
            ..Default::default()
        },
        // interaction: Interaction {
        //     resize_grab_radius_corner: 10.0,
        //     resize_grab_radius_side: 8.0,
        //     show_tooltips_only_when_still: false,
        // },
        animation_time: 150.0,

        text_styles: {
            let mut texts = BTreeMap::new();
            texts.insert(
                egui::TextStyle::Small,
                FontId {
                    size: 20.0,
                    family: FontFamily::Name(TITLE_FONT_NAME.into()),
                },
            );
            texts.insert(
                egui::TextStyle::Body,
                FontId {
                    size: 20.0,
                    family: FontFamily::Name(TITLE_FONT_NAME.into()),
                },
            );
            texts.insert(
                egui::TextStyle::Heading,
                FontId {
                    size: 22.0,
                    family: FontFamily::Name(TITLE_FONT_NAME.into()),
                },
            );
            texts.insert(
                egui::TextStyle::Monospace,
                FontId {
                    size: 20.0,
                    family: FontFamily::Name(TITLE_FONT_NAME.into()),
                },
            );
            texts.insert(
                egui::TextStyle::Button,
                FontId {
                    size: 20.0,
                    family: FontFamily::Name(TITLE_FONT_NAME.into()),
                },
            );
            texts
        },

        override_text_style: None,
        // override_font_id: Some(FontId {
        //     size: 12.0,
        //     family: FontFamily::Name("Monocraft".into()),
        // }),
        override_font_id: None,

        debug: Default::default(),
        explanation_tooltips: false,
        ..Default::default()
    };

    contexts.ctx_mut().set_style(style);
    contexts.ctx_mut().set_fonts(fonts);
    commands.spawn((Camera2dBundle::default(), Menu));
}
