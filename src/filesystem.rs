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
        size_kb: entry.metadata().unwrap().len() / 1024,
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

pub fn get_item(start: &Path, path: &[&str], parent_id: Option<&str>) -> Option<ItemGroup> {
    if path.is_empty() {
        tracing::debug!("Path is empty, nothing to do here?");
        return None;
    }

    tracing::debug!("entered get_item");
    tracing::debug!("start: {:?}", start);
    tracing::debug!("parent_id: {:?}", parent_id);

    let item_id = path[0];
    tracing::debug!("picked first item_id {}", item_id);

    let children = build_items(start, parent_id, false);
    tracing::debug!("found {} children", &children.len());

    tracing::debug!("Looking for child id {}", item_id);
    let found = children
        .iter()
        .find(|child| {
            tracing::debug!("comparing against child {}", &child.id);
            child.id == item_id
        })
        .map(|c| c.to_owned());

    if found.is_some() {
        tracing::debug!("Found a matching child");
    } else {
        tracing::debug!("Didn't find any matching children");
    }

    match path.len() {
        1 => found,
        _ => match found {
            Some(child) => {
                let start_here = start.join(child.name);
                tracing::debug!("entering get_item recursively");
                get_item(start_here.as_path(), &path[1..], Some(item_id))
            }
            None => None,
        },
    }
}

/// Resolves an item path (array of IDs) to the actual filesystem path on disk.
///
/// This function is similar to `get_item` but returns the concrete filesystem path
/// instead of an ItemGroup structure. It's used when you need the actual file/directory
/// path for operations like writing to .stignore files.
///
/// # Parameters
/// * `start` - The starting directory path (typically the base path)
/// * `path` - Array of item IDs representing the path to traverse (e.g., ["movies_id", "movie_dir_id"])
/// * `parent_id` - Parent context for ID generation (used for building items at each level)
///
/// # Returns
/// * `Some(PathBuf)` - The resolved filesystem path if all IDs in the path are found
/// * `None` - If any part of the path cannot be resolved or path is empty
///
/// # Example
/// ```
/// // Item path: ["movies_id", "movie1_id"]
/// // Base path: "/home/user/media"
/// // Returns: Some(PathBuf("/home/user/media/movies/Movie 1 (2023)"))
/// ```
pub fn resolve_item_filesystem_path(
    start: &Path,
    path: &[&str],
    parent_id: Option<&str>,
) -> Option<PathBuf> {
    // Empty path cannot be resolved
    if path.is_empty() {
        return None;
    }

    // Get the current item ID to find (first in the path)
    let item_id = path[0];

    // Build items in the current directory to search through
    let children = build_items(start, parent_id, false);

    // Find the child item that matches our target ID
    let found = children.iter().find(|child| child.id == item_id);

    match found {
        Some(child) => {
            // Construct the filesystem path by joining the child's name to current path
            let child_path = start.join(&child.name);

            match path.len() {
                // Base case: if this is the final item in path, return the resolved path
                1 => Some(child_path),
                // Recursive case: continue resolving with remaining path segments
                _ => resolve_item_filesystem_path(&child_path, &path[1..], Some(item_id)),
            }
        }
        // Item ID not found in current directory
        None => None,
    }
}
