use crate::filesystem;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct CategoryListingResponse {
    pub items: Vec<filesystem::ItemGroup>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct CategoryInfoResponse {
    pub name: String,
    pub items: Vec<filesystem::ItemGroup>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct ItemInfoRequest {
    pub item_path: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct ItemInfoResponse {
    pub item: filesystem::ItemGroup,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct NotFoundResponse {
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct IgnoreRequest {
    pub category_id: String,
    pub folder_path: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct IgnoreResponse {
    pub success: bool,
    pub message: String,
    pub ignored_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct IgnoreStatusRequest {
    pub category_id: String,
    pub folder_path: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct IgnoreStatusResponse {
    pub ignored: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct BulkIgnoreStatusRequest {
    pub items: Vec<IgnoreStatusRequest>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct BulkIgnoreStatusItem {
    pub category_id: String,
    pub folder_path: Vec<String>,
    pub ignored: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct BulkIgnoreStatusResponse {
    pub items: Vec<BulkIgnoreStatusItem>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct DeleteRequest {
    pub category_id: String,
    pub folder_path: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct DeleteResponse {
    pub success: bool,
    pub message: String,
    pub deleted_path: Option<String>,
}
