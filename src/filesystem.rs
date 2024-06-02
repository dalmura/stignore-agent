use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/* generic functions */
pub fn build_path(base_path: &String, category_path: &String) -> PathBuf {
    Path::new(base_path).join(category_path)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct ItemGroup {
    pub id: String,
    pub name: String,
    pub size_kb: u64,
    pub count: u32,
    pub items: Vec<ItemGroup>,
}

fn get_entry_size(entry: PathBuf) -> u64 {
    match fs::read_dir(entry) {
        Err(why) => {
            println!("ERROR: Unable to list path: {:?}", why.kind());
            0
        }
        Ok(entries) => entries.map(|i| i.unwrap().metadata().unwrap().len()).sum(),
    }
}

fn entry_to_item(entry: fs::DirEntry) -> ItemGroup {
    let children = build_items(entry.path());

    let children_size = match children.len() == 0 {
        true => get_entry_size(entry.path()),
        false => children.iter().map(|c| c.size_kb).sum(),
    };

    ItemGroup {
        id: entry.file_name().into_string().unwrap(),
        name: entry.file_name().into_string().unwrap(),
        size_kb: children_size,
        count: children.len() as u32,
        items: children,
    }
}

pub fn build_items(category_path: PathBuf) -> Vec<ItemGroup> {
    match fs::read_dir(category_path) {
        Err(why) => {
            println!("ERROR: Unable to list path: {:?}", why.kind());
            vec![]
        }
        Ok(paths) => paths
            .filter(|i| i.as_ref().unwrap().file_type().unwrap().is_dir())
            .map(|i| entry_to_item(i.unwrap()))
            .collect(),
    }
}
