use crate::config;
use crate::filesystem;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};

pub async fn help() -> Html<&'static str> {
    Html("Please visit <a href='https://github.com/dalmura/stignore-agent'>the documentation</a> for further information")
}

// GET categories
#[derive(Debug, Serialize, Deserialize, Clone)]
struct CategoryListingResponse {
    categories: Vec<String>,
}

pub async fn category_listing(State(data): State<config::Data>) -> impl IntoResponse {
    let categories: Vec<String> = data.categories.iter().map(|x| x.id.clone()).collect();
    (StatusCode::OK, Json(CategoryListingResponse { categories }))
}

// GET category info
#[derive(Debug, Serialize, Deserialize, Clone)]
struct CategoryInfoResponse {
    name: String,
    items: Vec<filesystem::CategoryItem>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct CategoryInfoNotFoundResponse {
    message: String,
}

pub async fn category_info(
    State(data): State<config::Data>,
    Path(category_id): Path<String>,
) -> Response {
    match data.categories.iter().find(|x| x.id == category_id) {
        Some(category) => {
            let category_path =
                filesystem::build_path(&data.agent.base_path, &category.relative_path);

            (
                StatusCode::OK,
                Json(CategoryInfoResponse {
                    name: category.name.clone(),
                    items: filesystem::get_category_items(category_path, category.id.clone()),
                }),
            )
                .into_response()
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(CategoryInfoNotFoundResponse {
                message: "Category ID not found".to_string(),
            }),
        )
            .into_response(),
    }
}
