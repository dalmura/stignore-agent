use crate::config;
use crate::filesystem;
use crate::responses::*;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    Json,
};

pub async fn help() -> Html<&'static str> {
    Html("Please visit <a href='https://github.com/dalmura/stignore-agent'>the documentation</a> for further information")
}

// GET categories
// Returns all configured categories that the agent is configured for!
pub async fn category_list(State(data): State<config::Data>) -> impl IntoResponse {
    let items = data
        .categories
        .iter()
        .map(|c| {
            let parent_id = filesystem::generate_id(&c.id, None);

            let category_path = filesystem::build_path(&data.agent.base_path, &c.relative_path);
            let children = filesystem::build_items(&category_path, Some(&parent_id), false);

            filesystem::ItemGroup {
                id: parent_id,
                name: c.name.clone(),
                size_kb: children.iter().map(|c| c.size_kb).sum(),
                items: children,
                leaf: false,
            }
        })
        .collect();

    (StatusCode::OK, Json(CategoryListingResponse { items }))
}

// GET category info
// Returns specific info for a given category
pub async fn category_info(
    State(data): State<config::Data>,
    Path(category_id): Path<String>,
) -> Response {
    match data.categories.iter().find(|x| x.id == category_id) {
        Some(category) => {
            let category_path =
                filesystem::build_path(&data.agent.base_path, &category.relative_path);

            let parent_id = filesystem::generate_id(&category.id, None);

            (
                StatusCode::OK,
                Json(CategoryInfoResponse {
                    name: category.name.clone(),
                    items: filesystem::build_items(&category_path, Some(&parent_id), false),
                }),
            )
                .into_response()
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(NotFoundResponse {
                message: format!("Category ID {} not found", category_id),
            }),
        )
            .into_response(),
    }
}

// GET itemgroup info
// Returns specific into for a given itemgroup
// We must be given a series of correct itemgroup IDs to traverse
pub async fn item_info(State(data): State<config::Data>, Path(path): Path<String>) -> Response {
    let start = std::path::Path::new(&data.agent.base_path);
    let item_path: Vec<&str> = path.split('/').collect();
    tracing::info!("Finding {:?}", &item_path);

    match filesystem::get_item(start, &item_path) {
        Some(item) => (StatusCode::OK, Json(ItemInfoResponse { item })).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(NotFoundResponse {
                message: format!("Item Path '{:?}' not found", &item_path),
            }),
        )
            .into_response(),
    }
}
