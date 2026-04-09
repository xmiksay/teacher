use axum::{
    Json,
    extract::{Path, State},
};
use sea_orm::{ActiveModelTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect, ColumnTrait, Set, DbErr};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::AppState;
use crate::entities::{user_language_profile, vocabulary, weak_point};

/// MCP tool handlers — called by the Claude API during lessons.
/// These are exposed as regular REST endpoints that Claude calls via MCP toolset.

#[derive(Serialize)]
pub struct ProfileContext {
    pub language: String,
    pub level: String,
    pub style: String,
    pub explanation_language: String,
    pub weak_points: Vec<WeakPointInfo>,
    pub recent_vocab: Vec<VocabInfo>,
}

#[derive(Serialize)]
pub struct WeakPointInfo {
    pub r#type: String,
    pub detail: String,
}

#[derive(Serialize)]
pub struct VocabInfo {
    pub word: String,
    pub translation: String,
    pub error_count: i32,
}

pub async fn get_profile(
    State(state): State<AppState>,
    Path(profile_id): Path<Uuid>,
) -> Result<Json<ProfileContext>, (axum::http::StatusCode, String)> {
    let profile = user_language_profile::Entity::find_by_id(profile_id)
        .one(&state.db)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((axum::http::StatusCode::NOT_FOUND, "Profile not found".to_string()))?;

    let weak_points = weak_point::Entity::find()
        .filter(weak_point::Column::ProfileId.eq(profile_id))
        .filter(weak_point::Column::Active.eq(true))
        .all(&state.db)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .into_iter()
        .map(|wp| WeakPointInfo {
            r#type: wp.r#type,
            detail: wp.detail,
        })
        .collect();

    let recent_vocab = vocabulary::Entity::find()
        .filter(vocabulary::Column::ProfileId.eq(profile_id))
        .order_by_asc(vocabulary::Column::LastPracticed)
        .limit(20)
        .all(&state.db)
        .await
        .map_err(|e: DbErr| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .into_iter()
        .map(|v| VocabInfo {
            word: v.word,
            translation: v.translation,
            error_count: v.error_count,
        })
        .collect();

    Ok(Json(ProfileContext {
        language: profile.language,
        level: profile.level,
        style: profile.style,
        explanation_language: profile.explanation_language,
        weak_points,
        recent_vocab,
    }))
}

#[derive(Deserialize)]
pub struct AddVocabularyInput {
    pub word: String,
    pub translation: String,
    pub context: Option<String>,
}

pub async fn add_vocabulary(
    State(state): State<AppState>,
    Path(profile_id): Path<Uuid>,
    Json(input): Json<AddVocabularyInput>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let model = vocabulary::ActiveModel {
        id: Set(Uuid::new_v4()),
        profile_id: Set(profile_id),
        word: Set(input.word.clone()),
        translation: Set(input.translation),
        added_by: Set("claude".to_string()),
        context: Set(input.context),
        last_practiced: Set(chrono::Utc::now().into()),
        error_count: Set(0),
        lesson_id: Set(None),
    };

    model
        .insert(&state.db)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({"status": "added", "word": input.word})))
}

pub async fn bump_vocabulary(
    State(state): State<AppState>,
    Path((profile_id, word)): Path<(Uuid, String)>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let vocab = vocabulary::Entity::find()
        .filter(vocabulary::Column::ProfileId.eq(profile_id))
        .filter(vocabulary::Column::Word.eq(&word))
        .one(&state.db)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((axum::http::StatusCode::NOT_FOUND, "Word not found".to_string()))?;

    let mut active: vocabulary::ActiveModel = vocab.into();
    active.last_practiced = Set(chrono::Utc::now().into());
    active.error_count = Set(active.error_count.unwrap() + 1);

    active
        .update(&state.db)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({"status": "bumped", "word": word})))
}

#[derive(Deserialize)]
pub struct AddWeakPointInput {
    pub r#type: String,
    pub detail: String,
}

pub async fn add_weak_point(
    State(state): State<AppState>,
    Path(profile_id): Path<Uuid>,
    Json(input): Json<AddWeakPointInput>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let model = weak_point::ActiveModel {
        id: Set(Uuid::new_v4()),
        profile_id: Set(profile_id),
        r#type: Set(input.r#type),
        detail: Set(input.detail.clone()),
        active: Set(true),
    };

    model
        .insert(&state.db)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({"status": "added", "detail": input.detail})))
}

pub async fn resolve_weak_point(
    State(state): State<AppState>,
    Path((profile_id, detail)): Path<(Uuid, String)>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let wp = weak_point::Entity::find()
        .filter(weak_point::Column::ProfileId.eq(profile_id))
        .filter(weak_point::Column::Detail.eq(&detail))
        .filter(weak_point::Column::Active.eq(true))
        .one(&state.db)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((axum::http::StatusCode::NOT_FOUND, "Weak point not found".to_string()))?;

    let mut active: weak_point::ActiveModel = wp.into();
    active.active = Set(false);

    active
        .update(&state.db)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({"status": "resolved", "detail": detail})))
}

#[derive(Deserialize)]
pub struct SetTopicPreferenceInput {
    pub style: Option<String>,
    pub explanation_language: Option<String>,
}

pub async fn set_topic_preference(
    State(state): State<AppState>,
    Path(profile_id): Path<Uuid>,
    Json(input): Json<SetTopicPreferenceInput>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let profile = user_language_profile::Entity::find_by_id(profile_id)
        .one(&state.db)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((axum::http::StatusCode::NOT_FOUND, "Profile not found".to_string()))?;

    let mut active: user_language_profile::ActiveModel = profile.into();

    if let Some(style) = input.style {
        active.style = Set(style);
    }
    if let Some(lang) = input.explanation_language {
        active.explanation_language = Set(lang);
    }

    active
        .update(&state.db)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({"status": "updated"})))
}
