mod config;
mod filesystem;
mod responses;
mod tasks;

use axum::{routing::get, Router};
use tracing_subscriber::fmt;

#[tokio::main]
async fn main() {
    /* initialize tracing */
    fmt::init();

    /* load config */
    let data = config::load_config("./config.toml");

    /* configure application routes */
    let app = Router::new()
        .route("/", get(tasks::help))
        .route("/api/v1/categories", get(tasks::category_list))
        .route("/api/v1/categories/:id", get(tasks::category_info))
        .with_state(data.clone());

    /* bind to the port and listen */
    let addr = format!("127.0.0.1:{}", data.agent.port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::debug!("listening on {}", &addr);
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
