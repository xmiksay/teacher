pub mod api;
pub mod auth;
pub mod entities;
pub mod mcp;
pub mod migration;

use sea_orm::DatabaseConnection;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub http_client: reqwest::Client,
    pub anthropic_api_key: String,
    pub claude_model: String,
    pub self_url: String,
}
