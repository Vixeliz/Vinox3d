use bevy::{asset::LoadState, math::Vec3A, prelude::*, render::primitives::Aabb};
use bevy_egui::EguiUserTextures;
use bevy_quinnet::client::{
    certificate::CertificateVerificationMode,
    connection::{ConnectionConfiguration, ConnectionEvent},
    Client,
};
use std::time::Duration;
use vinox_common::{
    ecs::bundles::PlayerBundleBuilder,
    networking::protocol::NetworkIP,
    storage::{
        blocks::load::load_all_blocks,
        crafting::load::load_all_recipes,
        geometry::load::load_all_geo,
        items::load::{item_from_block, load_all_items},
    },
    world::chunks::storage::{trim_geo_identifier, BlockTable, ItemTable, RecipeTable},
};

use crate::states::{
    assets::load::LoadableAssets, components::GameState, game::rendering::meshing::GeometryTable,
};

#[derive(Resource, Default, Deref, DerefMut)]
pub struct AssetsLoading(pub Vec<HandleUntyped>);

//TODO: Right now we are building the client only as a multiplayer client. This is fine but eventually we need to have singleplayer.
// To achieve this we will just have the client start up a server. But for now I am just going to use a dedicated one for testing
pub fn new_client(ip_res: Res<NetworkIP>, mut client: ResMut<Client>) {
    let ip = if ip_res.0 == "localhost" {
        "127.0.0.1".to_string()
    } else {
        ip_res.0.clone()
    }
    .parse()
    .unwrap();
    client
        .open_connection(
            ConnectionConfiguration::from_ips(ip, 25565, "0.0.0.0".to_string().parse().unwrap(), 0),
            CertificateVerificationMode::SkipVerification,
        )
        .unwrap();
}

#[allow(clippy::too_many_arguments)]
pub fn switch(
    mut commands: Commands,
    loading: Res<AssetsLoading>,
    asset_server: Res<AssetServer>,
    mut loadable_assets: ResMut<LoadableAssets>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut textures: ResMut<Assets<Image>>,
    mut client: ResMut<Client>,
    mut connected_event: EventReader<ConnectionEvent>,
) {
    match asset_server.get_group_load_state(loading.iter().map(|h| h.id())) {
        LoadState::Failed => {
            commands.insert_resource(NextState(Some(GameState::Menu)));
        }
        LoadState::Loaded => {
            for _ in connected_event.iter() {
                client.connection_mut().set_default_channel(
                    bevy_quinnet::shared::channel::ChannelId::UnorderedReliable,
                );

                let mut texture_atlas_builder = TextureAtlasBuilder::default();
                for handle in loadable_assets.block_textures.values() {
                    for item in handle {
                        let Some(texture) = textures.get(item) else {
            warn!("{:?} did not resolve to an `Image` asset.", asset_server.get_handle_path(item));
            continue;
                    };
                        texture_atlas_builder.add_texture(item.clone(), texture);
                    }
                }
                let texture_atlas = texture_atlas_builder.finish(&mut textures).unwrap();
                let atlas_handle = texture_atlases.add(texture_atlas);
                loadable_assets.block_atlas = atlas_handle;
                commands.insert_resource(NextState(Some(GameState::Game)));
            }
        }
        _ => {
            // NotLoaded/Loading: not fully ready yet
        }
    }
}

pub fn timeout(
    mut commands: Commands,
    mut timer: Local<Timer>,
    time: Res<Time>,
    mut client: ResMut<Client>,
) {
    timer.set_mode(TimerMode::Repeating);
    timer.set_duration(Duration::from_secs_f32(10.));

    timer.tick(time.delta());
    if timer.just_finished() {
        client.close_all_connections().ok();
        commands.insert_resource(NextState(Some(GameState::Menu)));
    }
}

pub fn setup_resources(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut loading: ResMut<AssetsLoading>,
    mut block_table: ResMut<BlockTable>,
    mut item_table: ResMut<ItemTable>,
    mut recipe_table: ResMut<RecipeTable>,
    mut geo_table: ResMut<GeometryTable>,
    mut loadable_assets: ResMut<LoadableAssets>,
    mut egui_textures: ResMut<EguiUserTextures>,
) {
    let player_handle = asset_server.load("base_player.gltf#Scene0");
    loading.push(player_handle.clone_untyped());
    commands.insert_resource(PlayerBundleBuilder {
        default_model: player_handle,
        model_aabb: Aabb {
            half_extents: Vec3A::new(0.25, 1.0, 0.2),
            ..default()
        },
    });

    for block in load_all_blocks() {
        let mut name = block.clone().namespace;
        name.push(':');
        name.push_str(&block.name);
        if let Some(has_item) = block.has_item {
            if has_item {
                item_table.insert(name.clone(), item_from_block(block.clone()));
            }
        }

        block_table.insert(name, block);
    }
    for recipe in load_all_recipes() {
        let mut name = recipe.clone().namespace;
        name.push(':');
        name.push_str(&recipe.name);
        recipe_table.insert(name, recipe);
    }
    for geo in load_all_geo() {
        let mut name = geo.clone().namespace;
        name.push(':');
        name.push_str(&geo.name);
        geo_table.insert(name, geo);
    }
    for item in load_all_items() {
        let mut name = item.clone().namespace;
        name.push(':');
        name.push_str(&item.name);

        item_table.insert(name, item);
    }

    for item in item_table.values() {
        let mut name = item.clone().namespace;
        name.push(':');
        name.push_str(&item.name);
        if let Some(path) = item.texture.clone() {
            let mut suffix = item.name.clone();
            suffix.push('/');
            suffix.push_str(&path);
            let texture_handle = asset_server.load(suffix);
            loading.push(texture_handle.clone_untyped());
            loadable_assets
                .item_textures
                .insert(name.clone(), texture_handle);
        } else {
            let texture_handle: Handle<Image> = asset_server.load("outline.png");
            loadable_assets
                .item_textures
                .insert(name.clone(), texture_handle);
        }
    }
    let texture_handle: Handle<Image> = asset_server.load("outline.png");
    loadable_assets
        .item_textures
        .insert("empty".to_string(), texture_handle);
    for item_texture in loadable_assets.item_textures.values() {
        egui_textures.add_image(item_texture.clone_weak());
    }
}

pub fn load_blocks(
    asset_server: Res<AssetServer>,
    mut loading: ResMut<AssetsLoading>,
    block_table: Res<BlockTable>,
    mut loadable_assets: ResMut<LoadableAssets>,
    mut has_ran: Local<bool>,
) {
    if !(*has_ran) && block_table.is_changed() {
        for block_pair in &**block_table {
            let block = block_pair.1;
            let mut texture_array: Vec<Handle<Image>> = Vec::with_capacity(6);
            texture_array.resize(6, Handle::default());
            let mut block_identifier = block.namespace.to_owned();
            block_identifier.push(':');
            block_identifier.push_str(&block.name.to_owned());
            // If there is a front texture preset all faces to use it so someone can use the same texture for all just by providing the front
            if let Some(texture_path) = &block.textures {
                if let Some(front) = texture_path.get(&Some("front".to_string())) {
                    let mut path = "blocks/".to_string();
                    let name = trim_geo_identifier(block.clone().name);
                    path.push_str(name.as_str());
                    path.push('/');
                    path.push_str(front.as_ref().unwrap());
                    let texture_handle: Handle<Image> = asset_server.load(path.as_str());
                    loading.push(texture_handle.clone_untyped());
                    texture_array[0] = texture_handle.clone();
                    texture_array[1] = texture_handle.clone();
                    texture_array[2] = texture_handle.clone();
                    texture_array[3] = texture_handle.clone();
                    texture_array[4] = texture_handle.clone();
                    texture_array[5] = texture_handle.clone();
                }
            }
            for texture_path_and_type in block.textures.iter() {
                for texture_path_and_type in texture_path_and_type.iter() {
                    if let (Some(texture_path), Some(texture_type)) = &texture_path_and_type {
                        let mut path = "blocks/".to_string();
                        let name = trim_geo_identifier(block.clone().name);
                        path.push_str(name.as_str());
                        path.push('/');
                        path.push_str(texture_type);
                        let texture_handle: Handle<Image> = asset_server.load(path.as_str());
                        loading.push(texture_handle.clone_untyped());

                        match &**texture_path {
                            "up" => {
                                texture_array[0] = texture_handle;
                            }
                            "down" => {
                                texture_array[1] = texture_handle;
                            }
                            "left" => {
                                texture_array[2] = texture_handle;
                            }
                            "right" => {
                                texture_array[3] = texture_handle;
                            }
                            "front" => {
                                texture_array[4] = texture_handle;
                            }
                            "back" => {
                                texture_array[5] = texture_handle;
                            }
                            _ => {}
                        }
                    }
                }
            }
            let texture_array: [Handle<Image>; 6] =
                texture_array
                    .try_into()
                    .unwrap_or_else(|texture_array: Vec<Handle<Image>>| {
                        panic!(
                            "Expected a Vec of length {} but it was {}",
                            6,
                            texture_array.len()
                        )
                    });
            loadable_assets
                .block_textures
                .insert(block_identifier, texture_array);
        }
        *has_ran = true;
    }
}
