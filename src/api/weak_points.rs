use axum::{
    Json,
    extract::{Path, State},
};
use sea_orm::{EntityTrait, QueryFilter, ColumnTrait};
use uuid::Uuid;

use crate::AppState;
use crate::auth::AuthUser;
use crate::entities::weak_point;

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
