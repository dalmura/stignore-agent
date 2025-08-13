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

/// Checks if an item ID (hash) is present in the .stignore file.
/// This is used when the actual file doesn't exist locally but we want to check
/// if it's been ignored by its hash ID.
///
/// # Parameters
/// * `category_base_path` - The base directory of the category (e.g., "/home/user/media/movies")
/// * `category_id` - The category ID used to generate the category hash
/// * `item_id` - The hash ID of the item to check
///
/// # Returns
/// * `bool` - True if the item ID is found in .stignore, false otherwise
pub fn is_item_id_ignored(
    category_base_path: &std::path::Path,
    category_id: &str,
    item_id: &str,
) -> bool {
    let stignore_path = category_base_path.join(".stignore");

    tracing::debug!(
        "is_item_id_ignored called with category_base_path: {:?}, category_id: '{}', item_id: '{}'",
        category_base_path,
        category_id,
        item_id
    );
    tracing::debug!("Looking for .stignore file at: {:?}", stignore_path);

    // Read .stignore file if it exists
    let ignore_content = match std::fs::read_to_string(&stignore_path) {
        Ok(content) => {
            tracing::debug!(
                "Successfully read .stignore file, content length: {} bytes",
                content.len()
            );
            tracing::debug!("Raw .stignore content:\n{}", content);
            content
        }
        Err(e) => {
            tracing::debug!("Could not read .stignore file: {}", e);
            return false; // No .stignore file means nothing is ignored
        }
    };

    // Convert each path in .stignore to its corresponding item ID and compare
    let mut found_match = false;
    for (line_num, line) in ignore_content.lines().enumerate() {
        let trimmed_line = line.trim();
        tracing::debug!(
            "Line {}: '{}' (trimmed: '{}')",
            line_num + 1,
            line,
            trimmed_line
        );

        // Skip empty lines
        if trimmed_line.is_empty() {
            continue;
        }

        // First check if it's already a hash ID (for manual additions)
        if trimmed_line == item_id {
            tracing::debug!("Found direct hash match on line {}", line_num + 1);
            found_match = true;
            break;
        }

        // Convert the path to an item ID by processing each path component
        let path_item_id = path_to_item_id(trimmed_line, category_id);
        tracing::debug!(
            "Converted path '{}' to item_id: '{}'",
            trimmed_line,
            path_item_id
        );

        if path_item_id == item_id {
            tracing::debug!(
                "Found converted path match on line {}: path '{}' -> id '{}'",
                line_num + 1,
                trimmed_line,
                path_item_id
            );
            found_match = true;
            break;
        }
    }

    tracing::debug!("is_item_id_ignored result: {}", found_match);
    found_match
}

/// Converts a filesystem path to its corresponding item ID by processing each component
/// in the hierarchy and generating IDs progressively, starting with the category hash.
fn path_to_item_id(path: &str, category_id: &str) -> String {
    tracing::debug!(
        "Converting path '{}' to item ID with category_id '{}'",
        path,
        category_id
    );

    // Generate the category hash first
    let category_hash = generate_id(category_id, None);
    tracing::debug!("Generated category hash: '{}'", category_hash);

    // Remove leading slash and split into components
    let path_clean = path.strip_prefix('/').unwrap_or(path);
    let components: Vec<&str> = path_clean.split('/').filter(|s| !s.is_empty()).collect();

    tracing::debug!("Path components: {:?}", components);

    if components.is_empty() {
        tracing::debug!("No path components, returning category hash");
        return category_hash;
    }

    // Generate item ID by walking through the hierarchy, starting with category hash as parent
    let mut current_parent_id = category_hash;
    let mut final_id = String::new();

    for (i, component) in components.iter().enumerate() {
        let item_id = generate_id(component, Some(&current_parent_id));

        tracing::debug!(
            "Component {}: '{}' with parent_id '{}' -> id '{}'",
            i,
            component,
            current_parent_id,
            item_id
        );

        final_id = item_id.clone();
        current_parent_id = item_id;
    }

    tracing::debug!("Final item ID for path '{}': '{}'", path, final_id);
    final_id
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

    // Check if the path is already ignored (check both path and potential item ID)
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
