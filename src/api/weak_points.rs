use axum::{
    Json,
    extract::{Path, State},
};
use sea_orm::{ActiveModelTrait, EntityTrait, QueryFilter, ColumnTrait, Set};
use serde::Deserialize;
use uuid::Uuid;

use crate::AppState;
use crate::auth::AuthUser;
use crate::entities::weak_point;

#[derive(Deserialize)]
pub struct CreateWeakPoint {
    pub profile_id: Uuid,
    #[serde(rename = "type")]
    pub r#type: String,
    pub detail: String,
}

pub async fn list_weak_points(
    _auth: AuthUser,
    State(state): State<AppState>,
    Path(profile_id): Path<Uuid>,
) -> Result<Json<Vec<weak_point::Model>>, (axum::http::StatusCode, String)> {
    let weak_points = weak_point::Entity::find()
        .filter(weak_point::Column::ProfileId.eq(profile_id))
        .all(&state.db)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(weak_points))
}

pub async fn create_weak_point(
    _auth: AuthUser,
    State(state): State<AppState>,
    Json(input): Json<CreateWeakPoint>,
) -> Result<Json<weak_point::Model>, (axum::http::StatusCode, String)> {
    let model = weak_point::ActiveModel {
        id: Set(Uuid::new_v4()),
        profile_id: Set(input.profile_id),
        r#type: Set(input.r#type),
        detail: Set(input.detail),
        active: Set(true),
    };

    let result = model
        .insert(&state.db)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(result))
}

pub async fn delete_weak_point(
    _auth: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<(), (axum::http::StatusCode, String)> {
    weak_point::Entity::delete_by_id(id)
        .exec(&state.db)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(())
}
