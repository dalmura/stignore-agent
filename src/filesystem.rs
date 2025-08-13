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

/// Result of adding a path to .stignore file
#[derive(Debug, Clone)]
pub enum StignoreResult {
    Success {
        ignored_path: String,
        message: String,
    },
    AlreadyIgnored {
        ignored_path: String,
    },
    Error {
        message: String,
    },
}

/// Checks if a filesystem path is already ignored in the .stignore file.
///
/// # Parameters
/// * `category_base_path` - The base directory of the category (e.g., "/home/user/media/movies")
/// * `file_path` - The full filesystem path to check if ignored
///
/// # Returns
/// * `bool` - True if the path is ignored, false otherwise
pub fn is_path_ignored(category_base_path: &std::path::Path, file_path: &std::path::Path) -> bool {
    let stignore_path = category_base_path.join(".stignore");

    // Calculate the path relative to the category
    let category_relative_path = match file_path.strip_prefix(category_base_path) {
        Ok(rel_path) => {
            let path_str = rel_path.to_string_lossy().to_string();
            // Ensure the path always starts with '/'
            if path_str.starts_with('/') {
                path_str
            } else {
                format!("/{}", path_str)
            }
        }
        Err(_) => return false,
    };

    // Read .stignore file if it exists
    let ignore_content = match std::fs::read_to_string(&stignore_path) {
        Ok(content) => content,
        Err(_) => return false, // No .stignore file means nothing is ignored
    };

    // Check if the path is in the ignore list
    ignore_content
        .lines()
        .any(|line| line.trim() == category_relative_path)
}

/// Adds a filesystem path to the .stignore file in the specified category directory.
///
/// This function handles all .stignore file operations including:
/// - Reading existing .stignore content
/// - Checking if the path is already ignored
/// - Adding new paths to ignore
/// - Writing back to the file
///
/// # Parameters
/// * `category_base_path` - The base directory of the category (e.g., "/home/user/media/movies")
/// * `file_path` - The full filesystem path to the item to ignore
/// * `category_name` - Name of the category for success messages
///
/// # Returns
/// * `StignoreResult` - Success, already ignored, or error result
pub fn add_to_stignore(
    category_base_path: &std::path::Path,
    file_path: &std::path::Path,
    category_name: &str,
) -> StignoreResult {
    let stignore_path = category_base_path.join(".stignore");

    // Calculate the path relative to the category
    let category_relative_path = match file_path.strip_prefix(category_base_path) {
        Ok(rel_path) => {
            let path_str = rel_path.to_string_lossy().to_string();
            // Ensure the path always starts with '/'
            if path_str.starts_with('/') {
                path_str
            } else {
                format!("/{}", path_str)
            }
        }
        Err(_) => {
            return StignoreResult::Error {
                message: "Failed to resolve category-relative path".to_string(),
            };
        }
    };

    // Read existing .stignore or create new content
    let mut ignore_content = std::fs::read_to_string(&stignore_path).unwrap_or_default();

    // Check if the path is already ignored
    if ignore_content
        .lines()
        .any(|line| line.trim() == category_relative_path)
    {
        return StignoreResult::AlreadyIgnored {
            ignored_path: category_relative_path,
        };
    }

    // Add the path to ignore content
    if !ignore_content.is_empty() && !ignore_content.ends_with('\n') {
        ignore_content.push('\n');
    }
    ignore_content.push_str(&category_relative_path);
    ignore_content.push('\n');

    // Write back to .stignore
    match std::fs::write(&stignore_path, ignore_content) {
        Ok(_) => StignoreResult::Success {
            ignored_path: category_relative_path.clone(),
            message: format!(
                "Successfully added '{}' to .stignore in category '{}'",
                category_relative_path, category_name
            ),
        },
        Err(err) => StignoreResult::Error {
            message: format!("Failed to write .stignore file: {}", err),
        },
    }
}
