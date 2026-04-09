use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use sea_orm::{ActiveModelTrait, EntityTrait, QueryFilter, ColumnTrait, Set};
use serde::Deserialize;
use uuid::Uuid;

use crate::AppState;
use crate::auth::AuthUser;
use crate::entities::user_language_profile;

#[derive(Deserialize)]
pub struct CreateProfile {
    pub language: String,
    pub level: Option<String>,
    pub style: Option<String>,
    pub explanation_language: Option<String>,
    pub personal_note: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateProfile {
    pub level: Option<String>,
    pub style: Option<String>,
    pub explanation_language: Option<String>,
    pub personal_note: Option<String>,
}

pub async fn create_profile(
    auth: AuthUser,
    State(state): State<AppState>,
    Json(input): Json<CreateProfile>,
) -> Result<Json<user_language_profile::Model>, (axum::http::StatusCode, String)> {
    let model = user_language_profile::ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(auth.user_id),
        language: Set(input.language),
        level: Set(input.level.unwrap_or_else(|| "A1".to_string())),
        style: Set(input.style.unwrap_or_else(|| "friendly".to_string())),
        explanation_language: Set(input.explanation_language.unwrap_or_else(|| "en".to_string())),
        personal_note: Set(input.personal_note.unwrap_or_default()),
    };

    let result = model
        .insert(&state.db)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(result))
}

pub async fn get_profile(
    _auth: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<user_language_profile::Model>, (axum::http::StatusCode, String)> {
    let profile = user_language_profile::Entity::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((axum::http::StatusCode::NOT_FOUND, "Profile not found".to_string()))?;

    Ok(Json(profile))
}

pub async fn list_profiles(
    auth: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<user_language_profile::Model>>, (axum::http::StatusCode, String)> {
    let profiles = user_language_profile::Entity::find()
        .filter(user_language_profile::Column::UserId.eq(auth.user_id))
        .all(&state.db)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(profiles))
}

pub async fn update_profile(
    _auth: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateProfile>,
) -> Result<Json<user_language_profile::Model>, (axum::http::StatusCode, String)> {
    let profile = user_language_profile::Entity::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((axum::http::StatusCode::NOT_FOUND, "Profile not found".to_string()))?;

    let mut active: user_language_profile::ActiveModel = profile.into();

    if let Some(level) = input.level {
        active.level = Set(level);
    }
    if let Some(style) = input.style {
        active.style = Set(style);
    }
    if let Some(lang) = input.explanation_language {
        active.explanation_language = Set(lang);
    }
    if let Some(note) = input.personal_note {
        active.personal_note = Set(note);
    }

    let updated = active
        .update(&state.db)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(updated))
}

pub async fn delete_profile(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    let profile = user_language_profile::Entity::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Profile not found".to_string()))?;

    if profile.user_id != auth.user_id {
        return Err((StatusCode::FORBIDDEN, "Not your profile".to_string()));
    }

    user_language_profile::Entity::delete_by_id(id)
        .exec(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}
