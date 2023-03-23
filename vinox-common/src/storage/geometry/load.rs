use directories::ProjectDirs;
use std::fs;

use walkdir::WalkDir;

use super::descriptor::{BlockGeo, GeometryDescriptor};

pub fn block_geo() -> Option<BlockGeo> {
    if let Some(proj_dirs) = ProjectDirs::from("com", "vinox", "vinox") {
        let final_path = proj_dirs.data_dir().join("assets/geometry/block/block.ron");
        if let Ok(ron_string) = fs::read_to_string(final_path) {
            let ron_result = ron::from_str::<GeometryDescriptor>(ron_string.as_str());
            if let Ok(block) = ron_result {
                Some(block.element)
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    }
}

pub fn load_all_geo() -> Vec<GeometryDescriptor> {
    let mut result = Vec::new();
    if let Some(proj_dirs) = ProjectDirs::from("com", "vinox", "vinox") {
        for entry in WalkDir::new(proj_dirs.data_dir().join("assets/geometry"))
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
