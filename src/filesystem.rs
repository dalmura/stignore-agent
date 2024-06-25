use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha512};
use std::fs;
use std::path::{Path, PathBuf};

/* generic functions */
pub fn build_path(base_path: &String, next_item: &String) -> PathBuf {
    Path::new(base_path).join(next_item)
}

pub fn generate_id(name: &str, parent_id: Option<&str>) -> String {
    let mut hasher = Sha512::new();

    if let Some(id) = parent_id {
        hasher.update(id);
    }

    hasher.update(name);

    format!("{:x}", hasher.finalize())
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct ItemGroup {
    pub id: String,
    pub name: String,
    pub size_kb: u64,
    pub items: Vec<ItemGroup>,
    pub leaf: bool,
}

fn dir_to_item(entry: fs::DirEntry, parent_id: Option<&str>) -> ItemGroup {
    let filename = entry.file_name().into_string().unwrap();
    let entry_path = entry.path();

    let item_id = generate_id(&filename, parent_id);

    let mut children = build_items(&entry_path, Some(&item_id), false);
    let mut leaf = false;

    if children.is_empty() {
        children = build_items(&entry_path, Some(&item_id), true);
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

fn file_to_item(entry: fs::DirEntry, parent_id: Option<&str>) -> ItemGroup {
    let filename = entry.file_name().into_string().unwrap();

    ItemGroup {
        id: generate_id(&filename, parent_id),
        name: filename,
        size_kb: entry.metadata().unwrap().len(),
        items: vec![],
        leaf: false,
    }
}

pub fn build_items(item_path: &Path, parent_id: Option<&str>, leaf: bool) -> Vec<ItemGroup> {
    tracing::info!("build_items with {:?} (leaf: {})", item_path, leaf);
    match fs::read_dir(item_path) {
        Ok(paths) => match leaf {
            true => paths.map(|i| file_to_item(i.unwrap(), parent_id)).collect(),
            false => paths
                .filter(|i| i.as_ref().unwrap().file_type().unwrap().is_dir())
                .map(|i| dir_to_item(i.unwrap(), parent_id))
                .collect(),
        },
        Err(why) => {
            println!("ERROR: Unable to list path: {:?}", why.kind());
            vec![]
        }
    }
}

pub fn get_item(start: &Path, path: &[&str]) -> Option<ItemGroup> {
    if path.is_empty() {
        tracing::info!("Path is empty, nothing to do here?");
        return None;
    }

    let item_id = path[0];
    tracing::info!("picked first item_id {}", &item_id);

    let children = build_items(start, None, false);
    tracing::info!("found {} children", &children.len());

    tracing::info!("Looking for child id {}", &item_id);
    let found = children
        .iter()
        .find(|child| {
            tracing::info!("comparing against child {:?}", &child.id);
            child.id == item_id
        })
        .map(|c| c.to_owned());

    if found.is_some() {
        tracing::info!("Found has Some");
    } else {
        tracing::info!("Didn't find any matching children");
    }

    match path.len() {
        1 => found,
        _ => match found {
            Some(child) => {
                let start_here = start.join(child.name);
                get_item(start_here.as_path(), &path[1..])
            }
            None => None,
        },
    }
}
