mod config;
mod tasks;

use axum::{routing::get, Router};
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    // load config
    let data = config::load_config("./config.toml");

    let app = Router::new()
        .route("/", get(tasks::help))
        .route("/api/v1/discover", get(tasks::category_listing))
        .with_state(data.clone());

    let addr = SocketAddr::from(([127, 0, 0, 1], data.agent.port));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
