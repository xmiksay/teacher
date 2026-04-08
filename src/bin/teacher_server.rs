use axum::{Router, routing::{get, post, delete}, response::IntoResponse, http::{StatusCode, header}};
use rust_embed::Embed;
use sea_orm::Database;
use sea_orm_migration::MigratorTrait;
use tower_http::cors::CorsLayer;
use tracing_subscriber::EnvFilter;

use teacher_server::{AppState, api, mcp, migration};

#[derive(Embed)]
#[folder = "client/dist"]
struct Assets;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://teacher:teacher@localhost:5432/teacher".to_string());
    let anthropic_api_key = std::env::var("ANTHROPIC_API_KEY")
        .expect("ANTHROPIC_API_KEY must be set");
    let claude_model = std::env::var("CLAUDE_MODEL")
        .unwrap_or_else(|_| "claude-sonnet-4-20250514".to_string());
    let self_url = std::env::var("SELF_URL")
        .unwrap_or_else(|_| "http://localhost:3000".to_string());
    let listen_addr = std::env::var("LISTEN_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:3000".to_string());

    let db = Database::connect(&database_url).await?;
    tracing::info!("Connected to database");

    migration::Migrator::up(&db, None).await?;
    tracing::info!("Migrations applied");

    let state = AppState {
        db,
        http_client: reqwest::Client::new(),
        anthropic_api_key,
        claude_model,
        self_url,
    };

    let app = Router::new()
        // Auth (public)
        .route("/api/auth/register", post(api::auth::register))
        .route("/api/auth/login", post(api::auth::login))
        // Lesson
        .route("/api/lesson/chat", post(api::lesson::chat))
        // Lesson history
        .route("/api/lessons/{profile_id}", get(api::lesson_history::list_lessons))
        .route("/api/lessons", post(api::lesson_history::create_lesson))
        .route("/api/lessons/{id}/detail", get(api::lesson_history::get_lesson))
        .route("/api/lessons/{id}/delete", delete(api::lesson_history::delete_lesson))
        // Profiles
        .route("/api/profiles", get(api::profile::list_profiles).post(api::profile::create_profile))
        .route("/api/profiles/{id}", get(api::profile::get_profile).put(api::profile::update_profile))
        // Weak points
        .route("/api/weak-points/{profile_id}", get(api::weak_points::list_weak_points))
        // Vocabulary
        .route("/api/vocab", post(api::vocab::create_vocab))
        .route("/api/vocab/{profile_id}", get(api::vocab::list_vocab))
        .route("/api/vocab/{id}/delete", delete(api::vocab::delete_vocab))
        // MCP endpoints (for direct access / testing)
        .route("/mcp/profile/{profile_id}", get(mcp::get_profile))
        .route("/mcp/vocabulary/{profile_id}", post(mcp::add_vocabulary))
        .route("/mcp/vocabulary/{profile_id}/bump/{word}", post(mcp::bump_vocabulary))
        .route("/mcp/weak_point/{profile_id}", post(mcp::add_weak_point))
        .route("/mcp/weak_point/{profile_id}/resolve/{detail}", post(mcp::resolve_weak_point))
        .route("/mcp/preference/{profile_id}", post(mcp::set_topic_preference))
        .layer(CorsLayer::permissive())
        .with_state(state)
        .fallback(static_handler);

    let listener = tokio::net::TcpListener::bind(&listen_addr).await?;
    tracing::info!("Listening on {listen_addr}");
    axum::serve(listener, app).await?;

    Ok(())
}

async fn static_handler(uri: axum::http::Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/');

    // Try the exact path first, then fall back to index.html for SPA routing
    match Assets::get(path) {
        Some(file) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            (StatusCode::OK, [(header::CONTENT_TYPE, mime.as_ref().to_string())], file.data.to_vec()).into_response()
        }
        None => match Assets::get("index.html") {
            Some(file) => {
                (StatusCode::OK, [(header::CONTENT_TYPE, "text/html".to_string())], file.data.to_vec()).into_response()
            }
            None => (StatusCode::NOT_FOUND, "Not Found").into_response(),
        },
    }
}
