use bevy_quinnet::client::Client;
use brigadier_rs::*;
use egui_notify::Toasts;
use std::{collections::BTreeMap, convert::Infallible};
use vinox_common::{networking::protocol::ClientMessage, physics::simulate::CollidesWithWorld};

use bevy::{math::Vec3A, pbr::wireframe::WireframeConfig, prelude::*, render::primitives::Aabb};
use bevy_egui::{
    egui::{Align2, FontId},
    *,
};

use crate::states::{
    components::GameOptions,
    game::{networking::components::ChatMessages, world::chunks::ControlledPlayer},
};

#[derive(Resource, Default, Deref, DerefMut)]
pub struct ConsoleOpen(pub bool);

#[derive(Resource, Default, Deref, DerefMut)]
pub struct Toast(pub Toasts);

#[allow(clippy::too_many_arguments)]
pub fn create_ui(
    mut commands: Commands,
    mut player_query: Query<(Entity, &mut Aabb), With<ControlledPlayer>>,
    collider_query: Query<&CollidesWithWorld>,
    mut client: ResMut<Client>,
    is_open: Res<ConsoleOpen>, // mut username_res: ResMut<UserName>,
    mut current_message: Local<String>,
    mut messages: ResMut<ChatMessages>,
    mut contexts: EguiContexts,
    mut toast: ResMut<Toast>,
    options: Res<GameOptions>,
    mut wireframe_config: ResMut<WireframeConfig>,
) {
    if !options.dark_theme {
        catppuccin_egui::set_theme(contexts.ctx_mut(), catppuccin_egui::MOCHA);
    }
    toast.show(contexts.ctx_mut());
    if **is_open {
        let parser = literal("/wireframe")
            .then(boolean("bool").build_exec(|_ctx: (), bar| {
                println!("Toggled wireframe to {bar}");
                Ok::<(), Infallible>(())
            }))
            .build_exec(|_ctx: ()| {
                println!("Called foo with no arguments");
                Ok::<(), Infallible>(())
            });
        let parser_spec = literal("/spectator")
            .then(boolean("bool").build_exec(|_ctx: (), bar| {
                println!("Toggled spectator to {bar}");
                Ok::<(), Infallible>(())
            }))
            .build_exec(|_ctx: ()| {
                println!("Called foo with no arguments");
                Ok::<(), Infallible>(())
            });
        let parser_tp = literal("/tp")
            .then(integer_i64("x").build_exec(|_ctx: (), bar| Ok::<(), Infallible>(())))
            .then(integer_i64("y").build_exec(|_ctx: (), bar| Ok::<(), Infallible>(())))
            .then(integer_i64("z").build_exec(|_ctx: (), bar| Ok::<(), Infallible>(())))
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
                                    // TODO: Switch this to a better system
                                    if let Ok((player, mut player_transform)) =
                                        player_query.get_single_mut()
                                    {
                                        if let Ok((result, _)) = parser.parse((), &current_message)
                                        {
                                            messages
                                                .push(("Console".to_string(), result.to_string()));
                                            wireframe_config.global = !wireframe_config.global;
                                        } else if let Ok((result, _)) =
                                            parser_spec.parse((), &current_message)
                                        {
                                            messages
                                                .push(("Console".to_string(), result.to_string()));
                                            if collider_query.get(player).is_ok() {
                                                commands
                                                    .entity(player)
                                                    .remove::<CollidesWithWorld>();
                                            } else {
                                                commands.entity(player).insert(CollidesWithWorld);
                                            }
                                        } else if let Ok((result, _)) =
                                            parser_tp.parse((), &current_message)
                                        {
                                            let (mut x, mut y, mut z) = (0, 0, 0);
                                            for (idx, num) in result.split_whitespace().enumerate()
                                            {
                                                if let Ok(conv) = num.parse() {
                                                    match idx {
                                                        0 => {
                                                            x = conv;
                                                        }
                                                        1 => {
                                                            y = conv;
                                                        }
                                                        2 => {
                                                            z = conv;
                                                        }
                                                        _ => {}
                                                    }
                                                }
                                            }
                                            player_transform.center =
                                                Vec3A::new(x as f32, y as f32, z as f32);
                                            messages.push((
                                                "Console".to_string(),
                                                format!("{x}, {y}, {z}"),
                                            ));
                                        } else {
                                            client.connection_mut().try_send_message(
                                                ClientMessage::ChatMessage {
                                                    message: current_message.to_string(),
                                                },
                                            );
                                            current_message.clear();
                                        }
                                    }
                                }
                            });
                        });

                    egui::ScrollArea::vertical()
                        .auto_shrink([false; 2])
                        .max_width(2000.0)
                        .show(ui, |ui| {
                            for (username, message) in messages.iter() {
                                ui.label(format!("{username}: {message}"));
                            }
                        });
                });
            });
    }
}
