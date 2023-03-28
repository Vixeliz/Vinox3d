use directories::ProjectDirs;
use std::fs;

use walkdir::WalkDir;

use crate::storage::blocks::descriptor::BlockDescriptor;

use super::descriptor::{ItemDescriptor, MAX_STACK_SIZE};

pub fn load_all_items() -> Vec<ItemDescriptor> {
    let mut result = Vec::new();
    if let Some(proj_dirs) = ProjectDirs::from("com", "vinox", "vinox") {
        for entry in WalkDir::new(proj_dirs.data_dir().join("assets/items"))
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.path().extension().unwrap_or_default() == "ron" {
                if let Ok(ron_string) = fs::read_to_string(entry.path()) {
                    let ron_result = ron::from_str(ron_string.as_str());
                    if let Ok(block) = ron_result {
                        result.push(block);
                    } else {
                        println!("{ron_result:?}");
                    }
                }
            }
        }
    }
    result
}

pub fn item_from_block(block: BlockDescriptor) -> ItemDescriptor {
    let mut name = block.clone().namespace;
    name.push(':');
    name.push_str(&block.name);

    let texture = if let Some(textures) = block.textures {
        if let Some(texture) = textures.get(&Some("front".to_string())) {
            // let mut final_path = "../../blocks/".to_string();
            let mut final_path = "blocks/".to_string();
            final_path.push_str(&block.name);
            final_path.push('/');
            final_path.push_str(&texture.clone().unwrap());
            // println!("{final_path}");
            Some(final_path)
        } else {
            None
        }
    } else {
        None
    };
    ItemDescriptor {
        namespace: block.namespace,
        name: block.name,
        texture,
        max_durability: None,
        max_stack_size: Some(MAX_STACK_SIZE),
        tool_type: None,
        script: None,
        associated_block: Some(name),
    }
}
