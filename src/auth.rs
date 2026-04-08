use axum::{
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
};
use chrono::Utc;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use uuid::Uuid;

use crate::AppState;
use crate::entities::auth_token;

/// Extractor that validates a Bearer token from the Authorization header.
/// Resolves to the authenticated user's ID.
pub struct AuthUser {
    pub user_id: Uuid,
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = (StatusCode, String);

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let header = parts
            .headers
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .ok_or((StatusCode::UNAUTHORIZED, "Missing authorization header".to_string()))?;

        let token = header
            .strip_prefix("Bearer ")
            .ok_or((StatusCode::UNAUTHORIZED, "Invalid authorization format".to_string()))?;

        let record = auth_token::Entity::find()
            .filter(auth_token::Column::Token.eq(token))
            .one(&state.db)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
            .ok_or((StatusCode::UNAUTHORIZED, "Invalid token".to_string()))?;

        if record.expires_at < Utc::now().fixed_offset() {
            return Err((StatusCode::UNAUTHORIZED, "Token expired".to_string()));
        }

        Ok(AuthUser {
            user_id: record.user_id,
        })
    }
}
