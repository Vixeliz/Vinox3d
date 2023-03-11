use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Resource, Default, Clone)]
pub struct LoadableAssets {
    pub block_textures: HashMap<String, [Handle<Image>; 6]>,
    pub entity_models: HashMap<String, Handle<Scene>>,
    pub block_atlas: Handle<TextureAtlas>,
}
