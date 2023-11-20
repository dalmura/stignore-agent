mod config;

use axum::{
    extract::State,
    routing::get,
    http::StatusCode,
    response::{Html, IntoResponse},
    Json, Router,
};
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    // load config
    let data = config::load_config("./config.toml");

    let app = Router::new()
        .route("/", get(help))
        .route("/api/v1/discover", get(category_listing))
        .with_state(data.clone());

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    let addr = SocketAddr::from(([127, 0, 0, 1], data.agent.port));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn help() -> Html<&'static str> {
    Html("Please visit <a href='https://github.com/dalmura/stignore-agent'>the documentation</a> for further information")
}

async fn category_listing(
    State(data): State<config::Data>,
) ->  impl IntoResponse {
    (StatusCode::IM_A_TEAPOT, Json(data))
}