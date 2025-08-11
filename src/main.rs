mod config;
mod filesystem;
mod models;
mod tasks;

use axum::{routing::get, routing::post, Router};
use tracing_subscriber::fmt;

use std::env;

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
        .route("/api/v1/items/{*path}", get(tasks::get_item_info))
        .route("/api/v1/items", post(tasks::post_item_info))
        .route("/api/v1/ignore", post(tasks::post_ignore))
        .with_state(data.clone());

    /* bind to the port and listen */
    let addr = format!("127.0.0.1:{}", data.agent.port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::info!("listening on {}", &addr);
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
