use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/* generic functions */
pub fn build_path(base_path: &String, category_path: &String) -> PathBuf {
    Path::new(base_path).join(category_path)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct CategoryItem {
    category: String,
    id: String,
    name: String,
    items: Vec<CategoryItem>,
}

fn entry_to_item(entry: fs::DirEntry, category_id: String) -> CategoryItem {
    CategoryItem {
        category: category_id.clone(),
        id: entry.file_name().into_string().unwrap(),
        name: entry.file_name().into_string().unwrap(),
        items: vec![],
    }
}

pub fn get_category_items(category_path: PathBuf, category_id: String) -> Vec<CategoryItem> {
    match fs::read_dir(category_path) {
        Err(why) => {
            println!("ERROR: Unable to list path: {:?}", why.kind());
            vec![]
        }
        Ok(paths) => paths
            .filter(|i| i.as_ref().unwrap().file_type().unwrap().is_dir())
            .map(|i| entry_to_item(i.unwrap(), category_id.clone()))
            .collect(),
    }
}
