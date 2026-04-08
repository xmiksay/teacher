use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use axum::{Json, extract::State, http::StatusCode};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};
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

    // Issue token
    let (token, expires_at) = generate_token();
    let token_model = auth_token::ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(user_id),
        token: Set(token.clone()),
        expires_at: Set(expires_at),
        created_at: Set(now),
    };
    token_model
        .insert(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(AuthResponse {
        user_id,
        token,
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

    // Issue new token
    let now = Utc::now().fixed_offset();
    let (token, expires_at) = generate_token();
    let token_model = auth_token::ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(user.id),
        token: Set(token.clone()),
        expires_at: Set(expires_at),
        created_at: Set(now),
    };
    token_model
        .insert(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(AuthResponse {
        user_id: user.id,
        token,
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
