use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha512};
use std::fs;
use std::path::{Path, PathBuf};

/* generic functions */
pub fn build_path(base_path: &String, next_item: &String) -> PathBuf {
    Path::new(base_path).join(next_item)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct ItemGroup {
    pub id: String,
    pub name: String,
    pub size_kb: u64,
    pub items: Vec<ItemGroup>,
    pub leaf: bool,
}

fn dir_to_item(entry: fs::DirEntry, parent_id: String) -> ItemGroup {
    let filename = entry.file_name().into_string().unwrap();

    let mut hasher = Sha512::new();
    hasher.update(&parent_id);
    hasher.update(&filename);

    let item_id = format!("{:x}", hasher.finalize());

    let mut children = build_items(entry.path(), item_id.clone(), false);
    let mut leaf = false;

    if children.is_empty() {
        children = build_items(entry.path(), item_id.clone(), true);
        leaf = true;
    }

    ItemGroup {
        id: item_id.clone(),
        name: filename,
        size_kb: children.iter().map(|c| c.size_kb).sum(),
        items: children,
        leaf,
    }
}

fn file_to_item(entry: fs::DirEntry, parent_id: String) -> ItemGroup {
    let filename = entry.file_name().into_string().unwrap();

    let mut hasher = Sha512::new();
    hasher.update(&parent_id);
    hasher.update(&filename);

    ItemGroup {
        id: format!("{:x}", hasher.finalize()),
        name: filename,
        size_kb: entry.metadata().unwrap().len(),
        items: vec![],
        leaf: false,
    }
}

pub fn build_items(item_path: PathBuf, parent_id: String, leaf: bool) -> Vec<ItemGroup> {
    match fs::read_dir(item_path) {
        Err(why) => {
            println!("ERROR: Unable to list path: {:?}", why.kind());
            vec![]
        }
        Ok(paths) => match leaf {
            true => paths
                .map(|i| file_to_item(i.unwrap(), parent_id.clone()))
                .collect(),
            false => paths
                .filter(|i| i.as_ref().unwrap().file_type().unwrap().is_dir())
                .map(|i| dir_to_item(i.unwrap(), parent_id.clone()))
                .collect(),
        },
    }
}
