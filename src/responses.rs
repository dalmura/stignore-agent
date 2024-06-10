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
pub(crate) struct ItemInfoResponse {
    pub item: filesystem::ItemGroup,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct NotFoundResponse {
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct DebugPathResponse {
    pub path: String,
}
