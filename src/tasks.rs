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
pub async fn category_list(State(data): State<config::Data>) -> impl IntoResponse {
    let categories = data
        .categories
        .iter()
        .map(|c| {
            let category_path = filesystem::build_path(&data.agent.base_path, &c.relative_path);
            let children = filesystem::build_items(category_path);

            filesystem::ItemGroup {
                id: c.id.clone(),
                name: c.name.clone(),
                size_kb: children.iter().map(|c| c.size_kb).sum(),
                count: children.len() as u32,
                items: children,
            }
        })
        .collect();

    (StatusCode::OK, Json(CategoryListingResponse { categories }))
}

// GET category info
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
                    items: filesystem::build_items(category_path),
                }),
            )
                .into_response()
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(CategoryInfoNotFoundResponse {
                message: format!("Category ID {} not found", category_id),
            }),
        )
            .into_response(),
    }
}
