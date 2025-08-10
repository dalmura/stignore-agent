use crate::config;
use crate::filesystem;
use crate::models::*;
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
// Returns specific info for a given itemgroup
// We must be given a series of correct itemgroup IDs to traverse
pub async fn get_item_info(State(data): State<config::Data>, Path(path): Path<String>) -> Response {
    let start = std::path::Path::new(&data.agent.base_path);
    let item_path: Vec<&str> = path.split('/').collect();
    tracing::info!("Finding {:?}", &item_path);

    match filesystem::get_item(start, &item_path, None) {
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

// POST itemgroup info
// Returns specific info for a given itemgroup
// We must be given a series of correct itemgroup IDs to traverse
pub async fn post_item_info(
    State(data): State<config::Data>,
    Json(payload): Json<ItemInfoRequest>,
) -> Response {
    let start = std::path::Path::new(&data.agent.base_path);
    let item_path: Vec<&str> = payload.item_path.iter().map(AsRef::as_ref).collect();

    for item in &item_path {
        tracing::debug!("Finding {}", *item);
    }

    match filesystem::get_item(start, item_path.as_slice(), None) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{AgentConfig, Category, Data};
    use axum::http::StatusCode;
    use axum::Router;
    use axum_test::TestServer;
    use std::fs;
    use tempfile::TempDir;

    // Test constants
    const MOVIES_ID: &str = "db15c6cdb24182a5027ee83634f7226e90f5e2b3593baa0672c93d89cb21e4f383f575bbb41e35fd76d3cd424275b7b0831d1437503fe1ab78229131a4183d30";
    const NONEXISTENT_ID: &str = "nonexistent_id";

    struct TestDirectoryPaths {
        movie1: std::path::PathBuf,
        movie2: std::path::PathBuf,
        show1_season1: std::path::PathBuf,
        show1_season2: std::path::PathBuf,
        show2_season1: std::path::PathBuf,
        show3_season1: std::path::PathBuf,
        show3_season2: std::path::PathBuf,
        show3_season3: std::path::PathBuf,
    }

    fn create_directory_structure(temp_dir: &TempDir) -> TestDirectoryPaths {
        let movies_dir = temp_dir.path().join("movies");
        let tv_dir = temp_dir.path().join("tv");
        fs::create_dir_all(&movies_dir).unwrap();
        fs::create_dir_all(&tv_dir).unwrap();

        // Create Movies structure
        let movie1_dir = movies_dir.join("Movie 1 (2023)");
        let movie2_dir = movies_dir.join("Movie 2 (2024)");
        fs::create_dir_all(&movie1_dir).unwrap();
        fs::create_dir_all(&movie2_dir).unwrap();

        // Create TV show structure
        let shows = [
            ("Show 1 (2021)", vec!["Season 1", "Season 2"]),
            ("Show 2 (2022)", vec!["Season 1"]),
            ("Show 3 (2023)", vec!["Season 1", "Season 2", "Season 3"]),
        ];

        for (show_name, seasons) in shows {
            let show_dir = tv_dir.join(show_name);
            fs::create_dir_all(&show_dir).unwrap();

            for season in seasons {
                fs::create_dir_all(show_dir.join(season)).unwrap();
            }
        }

        TestDirectoryPaths {
            movie1: movie1_dir,
            movie2: movie2_dir,
            show1_season1: tv_dir.join("Show 1 (2021)").join("Season 1"),
            show1_season2: tv_dir.join("Show 1 (2021)").join("Season 2"),
            show2_season1: tv_dir.join("Show 2 (2022)").join("Season 1"),
            show3_season1: tv_dir.join("Show 3 (2023)").join("Season 1"),
            show3_season2: tv_dir.join("Show 3 (2023)").join("Season 2"),
            show3_season3: tv_dir.join("Show 3 (2023)").join("Season 3"),
        }
    }

    fn create_test_files(paths: &TestDirectoryPaths) {
        // Create movie files
        fs::write(paths.movie1.join("Movie 1 (2023).mkv"), "test movie 1 content").unwrap();
        fs::write(paths.movie2.join("Movie 2 (2024).mp4"), "test movie 2 content").unwrap();

        // Create TV show files
        let tv_episodes = [
            (&paths.show1_season1, vec!["S01E01 - Ep 1.mkv", "S01E02 - Ep 2.mkv"]),
            (&paths.show1_season2, vec!["S02E01 - Ep 1.mkv", "S02E02 - Ep 2.mkv", "S02E03 - Ep 3.mkv"]),
            (&paths.show2_season1, vec!["S01E01 - Ep 1.mkv"]),
            (&paths.show3_season1, vec!["S01E01 - Ep 1.mkv"]),
            (&paths.show3_season2, vec!["S02E01 - Ep 1.mkv", "S02E02 - Ep 2.mkv"]),
            (&paths.show3_season3, vec!["S03E01 - Ep 1.mkv", "S03E02 - Ep 2.mkv", "S03E03 - Ep 3.mkv"]),
        ];

        for (season_path, episodes) in tv_episodes {
            for episode in episodes {
                fs::write(season_path.join(episode), "test episode content").unwrap();
            }
        }
    }

    fn create_test_data() -> (Data, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path().to_string_lossy().to_string();

        let paths = create_directory_structure(&temp_dir);
        create_test_files(&paths);

        let data = Data {
            agent: AgentConfig {
                name: "Test Agent".to_string(),
                port: 3000,
                base_path,
            },
            categories: vec![
                Category {
                    id: "movies".to_string(),
                    name: "Movies".to_string(),
                    relative_path: "movies".to_string(),
                },
                Category {
                    id: "tv".to_string(),
                    name: "TV Shows".to_string(),
                    relative_path: "tv".to_string(),
                },
            ],
        };

        (data, temp_dir)
    }

    fn create_test_router(data: Data) -> Router {
        Router::new()
            .route("/", axum::routing::get(help))
            .route("/api/v1/categories", axum::routing::get(category_list))
            .route("/api/v1/categories/{id}", axum::routing::get(category_info))
            .route("/api/v1/items/{*path}", axum::routing::get(get_item_info))
            .route("/api/v1/items", axum::routing::post(post_item_info))
            .with_state(data)
    }

    async fn setup_test_server() -> (TestServer, TempDir) {
        let (data, temp_dir) = create_test_data();
        let app = create_test_router(data);
        let server = TestServer::new(app).unwrap();
        (server, temp_dir)
    }

    // Helper endpoint tests
    #[tokio::test]
    async fn test_help_endpoint() {
        let (server, _temp_dir) = setup_test_server().await;

        let response = server.get("/").await;
        response.assert_status(StatusCode::OK);
        let text = response.text();
        assert!(text.contains("documentation"));
    }

    // Category endpoint tests
    #[tokio::test]
    async fn test_category_list() {
        let (server, _temp_dir) = setup_test_server().await;

        let response = server.get("/api/v1/categories").await;
        response.assert_status(StatusCode::OK);

        let json: CategoryListingResponse = response.json();
        assert_eq!(json.items.len(), 2);
        assert!(json.items.iter().any(|item| item.name == "Movies"));
        assert!(json.items.iter().any(|item| item.name == "TV Shows"));
    }

    #[tokio::test]
    async fn test_category_info_found() {
        let (server, _temp_dir) = setup_test_server().await;

        let response = server.get("/api/v1/categories/movies").await;
        response.assert_status(StatusCode::OK);

        let json: CategoryInfoResponse = response.json();
        assert_eq!(json.name, "Movies");
        assert_eq!(json.items.len(), 2); // Movie 1 (2023) and Movie 2 (2024) directories
    }

    #[tokio::test]
    async fn test_category_info_not_found() {
        let (server, _temp_dir) = setup_test_server().await;

        let response = server.get("/api/v1/categories/nonexistent").await;
        response.assert_status(StatusCode::NOT_FOUND);

        let json: NotFoundResponse = response.json();
        assert!(json.message.contains("Category ID nonexistent not found"));
    }

    // Item endpoint tests (GET)
    #[tokio::test]
    async fn test_get_item_info_success() {
        let (server, _temp_dir) = setup_test_server().await;

        let response = server.get(&format!("/api/v1/items/{}", MOVIES_ID)).await;
        response.assert_status(StatusCode::OK);

        let json: ItemInfoResponse = response.json();
        assert_eq!(json.item.name, "movies");
        assert_eq!(json.item.items.len(), 2);
    }

    #[tokio::test]
    async fn test_get_item_info_not_found() {
        let (server, _temp_dir) = setup_test_server().await;

        let response = server.get(&format!("/api/v1/items/{}", NONEXISTENT_ID)).await;
        response.assert_status(StatusCode::NOT_FOUND);

        let json: NotFoundResponse = response.json();
        assert!(json.message.contains("not found"));
    }

    // Item endpoint tests (POST)
    #[tokio::test]
    async fn test_post_item_info_success() {
        let (server, _temp_dir) = setup_test_server().await;

        let request_body = ItemInfoRequest {
            item_path: vec![MOVIES_ID.to_string()],
        };

        let response = server.post("/api/v1/items").json(&request_body).await;
        response.assert_status(StatusCode::OK);

        let json: ItemInfoResponse = response.json();
        assert_eq!(json.item.name, "movies");
        assert_eq!(json.item.items.len(), 2);
    }

    #[tokio::test]
    async fn test_post_item_info_not_found() {
        let (server, _temp_dir) = setup_test_server().await;

        let request_body = ItemInfoRequest {
            item_path: vec![NONEXISTENT_ID.to_string()],
        };

        let response = server.post("/api/v1/items").json(&request_body).await;
        response.assert_status(StatusCode::NOT_FOUND);

        let json: NotFoundResponse = response.json();
        assert!(json.message.contains("not found"));
    }
}
