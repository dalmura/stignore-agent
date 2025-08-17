mod config;
mod filesystem;
mod models;
mod tasks;

use axum::{
    Router,
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware,
    response::Response,
    routing::get,
    routing::post,
};
use tracing_subscriber::fmt;

use std::env;

async fn auth_middleware(
    State(data): State<config::Data>,
    request: Request<Body>,
    next: middleware::Next,
) -> Result<Response, StatusCode> {
    // Skip auth for help endpoint
    if request.uri().path() == "/" {
        return Ok(next.run(request).await);
    }

    // Check for X-API-Key header
    let auth_header = request
        .headers()
        .get("X-API-Key")
        .and_then(|header| header.to_str().ok());

    match auth_header {
        Some(provided_key) if provided_key == data.agent.api_key => Ok(next.run(request).await),
        _ => {
            tracing::warn!("Unauthorized access attempt to {}", request.uri().path());
            Err(StatusCode::UNAUTHORIZED)
        }
    }
}

#[tokio::main]
async fn main() {
    /* initialize tracing */
    fmt::init();

    /* load config */
    let args: Vec<String> = env::args().collect();
    let config_filename = &args[1];

    let data = config::load_config(config_filename);

    /* configure application routes */
    let app = Router::new()
        .route("/", get(tasks::help))
        .route("/api/v1/categories", get(tasks::category_list))
        .route("/api/v1/categories/{id}", get(tasks::category_info))
        .route("/api/v1/items", post(tasks::post_item_info))
        .route("/api/v1/ignore", post(tasks::post_ignore))
        .route("/api/v1/ignore-status", post(tasks::post_ignore_status))
        .route(
            "/api/v1/ignore-status-bulk",
            post(tasks::post_ignore_status_bulk),
        )
        .route("/api/v1/delete", post(tasks::post_delete))
        .layer(middleware::from_fn_with_state(
            data.clone(),
            auth_middleware,
        ))
        .with_state(data.clone());

    /* bind to the port and listen */
    let addr = format!("127.0.0.1:{}", data.agent.port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::info!("listening on {}", &addr);
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
