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
            tracing::warn!("Unable to list path: {:?}", why.kind());
            vec![]
        }
    }
}

pub fn get_item(start: &Path, path: &[&str], parent_id: Option<&str>) -> Option<ItemGroup> {
    if path.is_empty() {
        return None;
    }

    let item_id = path[0];
    let children = build_items(start, parent_id, false);
    let found = children
        .iter()
        .find(|child| child.id == item_id)
        .map(|c| c.to_owned());

    match path.len() {
        1 => found,
        _ => match found {
            Some(child) => {
                let start_here = start.join(child.name);
                get_item(start_here.as_path(), &path[1..], Some(item_id))
            }
            None => None,
        },
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

/// Result of deleting a path from filesystem
#[derive(Debug, Clone)]
pub enum DeleteResult {
    Success {
        deleted_path: String,
        message: String,
    },
    NotFound {
        requested_path: String,
    },
    Error {
        message: String,
    },
}

/// Checks if a folder path is ignored in the .stignore file.
/// This function works with folder names directly and supports non-existent folders.
///
/// # Parameters
/// * `category_base_path` - The base directory of the category (e.g., "/home/user/media/movies")
/// * `folder_path` - The folder path string to check (e.g., "/Movie Name (2023)")
///
/// # Returns
/// * `bool` - True if the folder path is ignored, false otherwise
pub fn is_path_ignored(category_base_path: &std::path::Path, folder_path: &str) -> bool {
    let stignore_path = category_base_path.join(".stignore");

    // Normalize the path to ensure consistency
    let normalized_path = if folder_path.starts_with('/') {
        folder_path.to_string()
    } else {
        format!("/{}", folder_path)
    };

    // Read .stignore file if it exists
    let ignore_content = match std::fs::read_to_string(&stignore_path) {
        Ok(content) => content,
        Err(_) => return false, // No .stignore file means nothing is ignored
    };

    // Check if the path is in the ignore list
    ignore_content
        .lines()
        .any(|line| line.trim() == normalized_path)
}

/// Adds a folder path to the .stignore file in the specified category directory.
/// This function works with folder names directly and supports non-existent folders.
///
/// # Parameters
/// * `category_base_path` - The base directory of the category (e.g., "/home/user/media/movies")
/// * `folder_path` - The folder path string to ignore (e.g., "/Movie Name (2023)")
/// * `category_name` - Name of the category for success messages
///
/// # Returns
/// * `StignoreResult` - Success, already ignored, or error result
pub fn add_to_stignore(
    category_base_path: &std::path::Path,
    folder_path: &str,
    category_name: &str,
) -> StignoreResult {
    let stignore_path = category_base_path.join(".stignore");

    // Ensure the path starts with '/' for consistency
    let normalized_path = if folder_path.starts_with('/') {
        folder_path.to_string()
    } else {
        format!("/{}", folder_path)
    };

    // Read existing .stignore or create new content
    let mut ignore_content = std::fs::read_to_string(&stignore_path).unwrap_or_default();

    // Check if the path is already ignored
    if ignore_content
        .lines()
        .any(|line| line.trim() == normalized_path)
    {
        return StignoreResult::AlreadyIgnored {
            ignored_path: normalized_path,
        };
    }

    // Add the path to ignore content
    if !ignore_content.is_empty() && !ignore_content.ends_with('\n') {
        ignore_content.push('\n');
    }
    ignore_content.push_str(&normalized_path);
    ignore_content.push('\n');

    // Write back to .stignore
    match std::fs::write(&stignore_path, ignore_content) {
        Ok(_) => StignoreResult::Success {
            ignored_path: normalized_path.clone(),
            message: format!(
                "Successfully added '{}' to .stignore in category '{}'",
                normalized_path, category_name
            ),
        },
        Err(err) => StignoreResult::Error {
            message: format!("Failed to write .stignore file: {}", err),
        },
    }
}

/// Deletes a folder path from the filesystem in the specified category directory.
/// This function actually removes files and directories from disk.
///
/// # Parameters
/// * `category_base_path` - The base directory of the category (e.g., "/home/user/media/movies")
/// * `folder_path` - The folder path to delete (as a vector of path components)
/// * `category_name` - Name of the category for success messages
///
/// # Returns
/// * `DeleteResult` - Success, not found, or error result
pub fn delete_from_filesystem(
    category_base_path: &std::path::Path,
    folder_path: &[String],
    category_name: &str,
) -> DeleteResult {
    // Build the complete path to delete
    let mut full_path = category_base_path.to_path_buf();
    for component in folder_path {
        full_path = full_path.join(component);
    }

    // Convert to display string for messages
    let folder_path_str = format!("/{}", folder_path.join("/"));

    // Check if the path exists
    if !full_path.exists() {
        return DeleteResult::NotFound {
            requested_path: folder_path_str,
        };
    }

    // Attempt to delete the path
    let result = if full_path.is_dir() {
        std::fs::remove_dir_all(&full_path)
    } else {
        std::fs::remove_file(&full_path)
    };

    match result {
        Ok(_) => DeleteResult::Success {
            deleted_path: folder_path_str.clone(),
            message: format!(
                "Successfully deleted '{}' from category '{}'",
                folder_path_str, category_name
            ),
        },
        Err(err) => DeleteResult::Error {
            message: format!("Failed to delete '{}': {}", folder_path_str, err),
        },
    }
}
