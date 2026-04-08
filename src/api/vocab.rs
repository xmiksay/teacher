use axum::{
    Json,
    extract::{Path, Query, State},
};
use sea_orm::{ActiveModelTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect, ColumnTrait, Set};
use serde::Deserialize;
use uuid::Uuid;

use crate::AppState;
use crate::auth::AuthUser;
use crate::entities::vocabulary;

#[derive(Deserialize)]
pub struct CreateVocab {
    pub profile_id: Uuid,
    pub word: String,
    pub translation: String,
    pub added_by: Option<String>,
    pub context: Option<String>,
}

#[derive(Deserialize)]
pub struct VocabQuery {
    pub limit: Option<u64>,
    pub offset: Option<u64>,
}

pub async fn create_vocab(
    _auth: AuthUser,
    State(state): State<AppState>,
    Json(input): Json<CreateVocab>,
) -> Result<Json<vocabulary::Model>, (axum::http::StatusCode, String)> {
    let model = vocabulary::ActiveModel {
        id: Set(Uuid::new_v4()),
        profile_id: Set(input.profile_id),
        word: Set(input.word),
        translation: Set(input.translation),
        added_by: Set(input.added_by.unwrap_or_else(|| "user".to_string())),
        context: Set(input.context),
        last_practiced: Set(chrono::Utc::now().into()),
        error_count: Set(0),
    };

    let result = model
        .insert(&state.db)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(result))
}

pub async fn list_vocab(
    _auth: AuthUser,
    State(state): State<AppState>,
    Path(profile_id): Path<Uuid>,
    Query(query): Query<VocabQuery>,
) -> Result<Json<Vec<vocabulary::Model>>, (axum::http::StatusCode, String)> {
    let mut select = vocabulary::Entity::find()
        .filter(vocabulary::Column::ProfileId.eq(profile_id))
        .order_by_asc(vocabulary::Column::LastPracticed);

    if let Some(offset) = query.offset {
        select = select.offset(offset);
    }

    let limit = query.limit.unwrap_or(50);
    select = select.limit(limit);

    let vocab = select
        .all(&state.db)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(vocab))
}

pub async fn delete_vocab(
    _auth: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<(), (axum::http::StatusCode, String)> {
    vocabulary::Entity::delete_by_id(id)
        .exec(&state.db)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(())
}
