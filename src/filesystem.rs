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
    pub items: Vec<ItemGroup>,
}

fn dir_to_item(entry: fs::DirEntry) -> ItemGroup {
    let children = match &build_items(entry.path(), false)[..] {
        [] => build_items(entry.path(), true),
        rest => rest.to_vec(),
    };

    ItemGroup {
        id: entry.file_name().into_string().unwrap(),
        name: entry.file_name().into_string().unwrap(),
        size_kb: children.iter().map(|c| c.size_kb).sum(),
        items: children,
    }
}

fn file_to_item(entry: fs::DirEntry) -> ItemGroup {
    ItemGroup {
        id: entry.file_name().into_string().unwrap(),
        name: entry.file_name().into_string().unwrap(),
        size_kb: entry.metadata().unwrap().len(),
        items: vec![],
    }
}

pub fn build_items(category_path: PathBuf, leaf: bool) -> Vec<ItemGroup> {
    match fs::read_dir(category_path) {
        Err(why) => {
            println!("ERROR: Unable to list path: {:?}", why.kind());
            vec![]
        }
        Ok(paths) => match leaf {
            true => paths.map(|i| file_to_item(i.unwrap())).collect(),
            false => paths
                .filter(|i| i.as_ref().unwrap().file_type().unwrap().is_dir())
                .map(|i| dir_to_item(i.unwrap()))
                .collect(),
        },
    }
}
