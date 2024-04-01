use crate::config;
use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse},
    Json,
};
use serde::{Deserialize, Serialize};

pub async fn help() -> Html<&'static str> {
    Html("Please visit <a href='https://github.com/dalmura/stignore-agent'>the documentation</a> for further information")
}

// Category Listing Endpoint
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CategoryListingResponse {
    pub(crate) categories: Vec<String>,
}

pub async fn category_listing(State(data): State<config::Data>) -> impl IntoResponse {
    let categories: Vec<String> = data.categories.iter().map(|x| x.name.clone()).collect();
    (StatusCode::OK, Json(CategoryListingResponse { categories }))
}
