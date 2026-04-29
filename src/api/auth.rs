use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use axum::{Json, extract::State, http::StatusCode};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::AppState;
use crate::entities::{auth_token, user};

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub user_id: Uuid,
    pub token: String,
    pub expires_at: chrono::DateTime<chrono::FixedOffset>,
}

pub async fn register(
    State(state): State<AppState>,
    Json(input): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, String)> {
    if input.username.is_empty() || input.password.len() < 8 {
        return Err((
            StatusCode::BAD_REQUEST,
            "Username required, password must be at least 8 characters".to_string(),
        ));
    }

    // Check if username already taken
    let existing = user::Entity::find()
        .filter(user::Column::Username.eq(&input.username))
        .one(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if existing.is_some() {
        return Err((StatusCode::CONFLICT, "Username already taken".to_string()));
    }

    let now = Utc::now().fixed_offset();
    let user_id = Uuid::new_v4();

    // Hash password with argon2 (salt is embedded in PHC string)
    let salt = SaltString::generate(&mut OsRng);
    let password_hash = Argon2::default()
        .hash_password(input.password.as_bytes(), &salt)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .to_string();

    // Create user with password
    let user_model = user::ActiveModel {
        id: Set(user_id),
        username: Set(input.username),
        password_hash: Set(password_hash),
        created_at: Set(now),
    };
    user_model
        .insert(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Issue token — store only the SHA-256 hash, return the raw token to the user
    let (raw_token, expires_at) = generate_token();
    let token_model = auth_token::ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(user_id),
        token: Set(hash_token(&raw_token)),
        expires_at: Set(expires_at),
        created_at: Set(now),
    };
    token_model
        .insert(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(AuthResponse {
        user_id,
        token: raw_token,
        expires_at,
    }))
}

pub async fn login(
    State(state): State<AppState>,
    Json(input): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, String)> {
    // Find user by username
    let user = user::Entity::find()
        .filter(user::Column::Username.eq(&input.username))
        .one(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::UNAUTHORIZED, "Invalid credentials".to_string()))?;

    // Verify password
    let parsed_hash = PasswordHash::new(&user.password_hash)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Argon2::default()
        .verify_password(input.password.as_bytes(), &parsed_hash)
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid credentials".to_string()))?;

    // Issue new token — store only the SHA-256 hash, return the raw token to the user
    let now = Utc::now().fixed_offset();
    let (raw_token, expires_at) = generate_token();
    let token_model = auth_token::ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(user.id),
        token: Set(hash_token(&raw_token)),
        expires_at: Set(expires_at),
        created_at: Set(now),
    };
    token_model
        .insert(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(AuthResponse {
        user_id: user.id,
        token: raw_token,
        expires_at,
    }))
}

fn generate_token() -> (String, chrono::DateTime<chrono::FixedOffset>) {
    use rand::Rng;
    use std::fmt::Write;

    let mut rng = rand::rng();
    let token_bytes: [u8; 32] = rng.random();
    let mut token = String::with_capacity(64);
    for byte in token_bytes {
        write!(token, "{byte:02x}").unwrap();
    }
    let expires_at = (Utc::now() + chrono::Duration::days(30)).fixed_offset();

    (token, expires_at)
}

/// SHA-256 hash a raw token for storage. Tokens are high-entropy,
/// so a fast hash is sufficient (no need for argon2).
pub fn hash_token(raw_token: &str) -> String {
    use std::fmt::Write;
    let hash = Sha256::digest(raw_token.as_bytes());
    let mut hex = String::with_capacity(64);
    for byte in hash {
        write!(hex, "{byte:02x}").unwrap();
    }
    hex
}
