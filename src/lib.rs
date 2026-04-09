pub mod api;
pub mod auth;
pub mod entities;
pub mod mcp;
pub mod migration;

use sea_orm::DatabaseConnection;

#[derive(Clone)]
pub enum LlmProvider {
    Claude { api_key: String, model: String },
    Ollama { base_url: String, model: String },
}

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub http_client: reqwest::Client,
    pub llm: LlmProvider,
    pub self_url: String,
}
