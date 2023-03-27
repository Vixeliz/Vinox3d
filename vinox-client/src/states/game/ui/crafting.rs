use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use std::collections::{BTreeMap, HashMap};

use bevy::prelude::*;
use bevy_egui::EguiContexts;
use bevy_egui::{egui::FontId, *};
use vinox_common::storage::items::descriptor::ItemData;
use vinox_common::world::chunks::storage::{identifier_to_name, name_to_identifier, ItemTable};
use vinox_common::{ecs::bundles::Inventory, world::chunks::storage::RecipeTable};

use crate::states::{components::GameOptions, game::world::chunks::ControlledPlayer};

//TODO: This is pretty jank rn. Many edge cases aren't accounted for
pub fn craft(
    inventory: &mut Inventory,
    required_items: &HashMap<String, u32>,
    output_item: (String, u32),
    item_table: &ItemTable,
) {
    let mut modified_items = required_items.clone();
    let mut new_inventory = inventory.clone();
    for (row_num, bar_row) in new_inventory.clone().hotbar.0.iter().enumerate() {
        for (item_num, bar_item) in bar_row.iter().cloned().enumerate() {
            if let Some(bar_item) = bar_item {
                let identifier =
                    name_to_identifier(bar_item.clone().namespace, bar_item.clone().name);
                if modified_items.contains_key(&identifier) {
                    let amount_left = *modified_items.get(&identifier).unwrap();
                    if amount_left < bar_item.stack_size {
                        let mut new_item = bar_item.clone();
                        new_item.stack_size -= amount_left;
                        new_inventory.hotbar[row_num][item_num] = Some(new_item);
                        modified_items.remove(&identifier);
                    } else {
                        new_inventory.hotbar[row_num][item_num] = None;
                        if (amount_left as i32 - bar_item.stack_size as i32) <= 0 {
                            modified_items.remove(&identifier);
                        } else {
                            modified_items
                                .insert(identifier.clone(), amount_left - bar_item.stack_size);
                        }
                    }
                }
            }
        }
    }
    for (row_num, inv_row) in new_inventory.clone().slots.iter().enumerate() {
        for (item_num, inv_item) in inv_row.iter().cloned().enumerate() {
            if let Some(inv_item) = inv_item {
                let identifier =
                    name_to_identifier(inv_item.clone().namespace, inv_item.clone().name);
                if modified_items.contains_key(&identifier) {
                    let amount_left = *modified_items.get(&identifier).unwrap();
                    if amount_left < inv_item.stack_size {
                        let mut new_item = inv_item.clone();
                        new_item.stack_size -= amount_left;
                        new_inventory.slots[row_num][item_num] = Some(new_item);
                        modified_items.remove(&identifier);
                    } else {
                        new_inventory.slots[row_num][item_num] = None;
                        if (amount_left as i32 - inv_item.stack_size as i32) <= 0 {
                            modified_items.remove(&identifier);
                        } else {
                            modified_items
                                .insert(identifier.clone(), amount_left - inv_item.stack_size);
                        }
                    }
                }
            }
        }
    }
    if modified_items.is_empty() {
        if new_inventory
            .add_item(item_table.get(&output_item.0).unwrap())
            .is_ok()
        {
            *inventory = new_inventory.clone();
        }
    }
}

pub fn crafting_ui(
    recipe_table: Res<RecipeTable>,
    item_table: Res<ItemTable>,
    mut player_query: Query<&mut Inventory, With<ControlledPlayer>>,
    mut contexts: EguiContexts,
    options: Res<GameOptions>,
    mut current_search: Local<String>,
) {
    if !options.dark_theme {
        catppuccin_egui::set_theme(contexts.ctx_mut(), catppuccin_egui::MOCHA);
    }
    if let Ok(mut inventory) = player_query.get_single_mut() {
        if inventory.open {
            egui::Window::new("crafting").show(contexts.ctx_mut(), |ui| {
                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                    ui.ctx().set_style(egui::Style {
                        text_styles: {
                            let mut texts = BTreeMap::new();
                            texts.insert(egui::style::TextStyle::Small, FontId::proportional(18.0));
                            texts.insert(egui::style::TextStyle::Body, FontId::proportional(18.0));
                            texts.insert(
                                egui::style::TextStyle::Heading,
                                FontId::proportional(20.0),
                            );
                            texts
                                .insert(egui::style::TextStyle::Monospace, FontId::monospace(18.0));
                            texts
                                .insert(egui::style::TextStyle::Button, FontId::proportional(18.0));
                            texts
                        },
                        ..Default::default()
                    });
                    let mut sorted_recipe_table = Vec::new();
                    ui.horizontal(|ui| {
                        ui.label("Search: ");
                        ui.text_edit_singleline(&mut *current_search);
                    });
                    let matcher = SkimMatcherV2::default();

                    for recipe in recipe_table.values() {
                        let score = matcher.fuzzy_match(&recipe.name, &current_search);
                        sorted_recipe_table.push((score, recipe));
                    }
                    sorted_recipe_table.sort_unstable_by_key(|k| k.0);

                    egui::ScrollArea::vertical()
                        .auto_shrink([false; 2])
                        .max_width(2000.0)
                        .show(ui, |ui| {
                            for (_, recipe) in sorted_recipe_table.iter().rev() {
                                ui.horizontal(|ui| {
                                    ui.label(format!("{}: x{}", recipe.name, recipe.output_item.1))
                                        .on_hover_ui(|ui| {
                                            for (required_item, item_amount) in recipe
                                                .required_items
                                                .clone()
                                                .unwrap_or(HashMap::new())
                                                .iter()
                                            {
                                                if let Some((_, name)) =
                                                    identifier_to_name(required_item.clone())
                                                {
                                                    ui.label(format!("{name}: x{item_amount}"));
                                                }
                                            }
                                        });
                                    if ui.button("Craft").clicked() {
                                        if let Some(required_items) = recipe.required_items.clone()
                                        {
                                            craft(
                                                &mut inventory,
                                                &required_items,
                                                recipe.output_item.clone(),
                                                &item_table,
                                            );
                                        }
                                    }
                                });
                            }
                        });
                });
            });
        }
    }
}
