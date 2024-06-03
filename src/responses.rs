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
pub(crate) struct CategoryInfoNotFoundResponse {
    pub message: String,
}
