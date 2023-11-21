use crate::config;
use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse},
    Json,
};

pub async fn help() -> Html<&'static str> {
    Html("Please visit <a href='https://github.com/dalmura/stignore-agent'>the documentation</a> for further information")
}

pub async fn category_listing(State(data): State<config::Data>) -> impl IntoResponse {
    (StatusCode::IM_A_TEAPOT, Json(data))
}
