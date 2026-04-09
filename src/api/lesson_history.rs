use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::AppState;
use crate::auth::AuthUser;
use crate::entities::{lesson, lesson_message, user_language_profile};

#[derive(Serialize)]
pub struct LessonSummary {
    pub id: Uuid,
    pub profile_id: Uuid,
    pub title: String,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
    pub message_count: usize,
}

#[derive(Serialize)]
pub struct LessonDetail {
    pub id: Uuid,
    pub profile_id: Uuid,
    pub title: String,
    pub messages: Vec<MessageDetail>,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
}

#[derive(Serialize, Deserialize)]
pub struct MessageDetail {
    pub id: Uuid,
    pub role: String,
    pub content: String,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
}

#[derive(Deserialize)]
pub struct CreateLessonRequest {
    pub profile_id: Uuid,
}

pub async fn list_lessons(
    _auth: AuthUser,
    State(state): State<AppState>,
    Path(profile_id): Path<Uuid>,
) -> Result<Json<Vec<LessonSummary>>, (StatusCode, String)> {
    // Verify profile exists
    user_language_profile::Entity::find_by_id(profile_id)
        .one(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Profile not found".to_string()))?;

    let lessons = lesson::Entity::find()
        .filter(lesson::Column::ProfileId.eq(profile_id))
        .order_by_desc(lesson::Column::UpdatedAt)
        .all(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut summaries = Vec::new();
    for l in lessons {
        let message_count = lesson_message::Entity::find()
            .filter(lesson_message::Column::LessonId.eq(l.id))
            .all(&state.db)
            .await
            .map(|msgs| msgs.len())
            .unwrap_or(0);

        summaries.push(LessonSummary {
            id: l.id,
            profile_id: l.profile_id,
            title: l.title,
            created_at: l.created_at,
            updated_at: l.updated_at,
            message_count,
        });
    }

    Ok(Json(summaries))
}

pub async fn create_lesson(
    _auth: AuthUser,
    State(state): State<AppState>,
    Json(input): Json<CreateLessonRequest>,
) -> Result<Json<LessonDetail>, (StatusCode, String)> {
    // Verify profile exists
    user_language_profile::Entity::find_by_id(input.profile_id)
        .one(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Profile not found".to_string()))?;

    let now = chrono::Utc::now().fixed_offset();
    let id = Uuid::new_v4();

    let model = lesson::ActiveModel {
        id: Set(id),
        profile_id: Set(input.profile_id),
        title: Set("New lesson".to_string()),
        created_at: Set(now),
        updated_at: Set(now),
    };

    let lesson = model
        .insert(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(LessonDetail {
        id: lesson.id,
        profile_id: lesson.profile_id,
        title: lesson.title,
        messages: vec![],
        created_at: lesson.created_at,
        updated_at: lesson.updated_at,
    }))
}

pub async fn get_lesson(
    _auth: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<LessonDetail>, (StatusCode, String)> {
    let lesson = lesson::Entity::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Lesson not found".to_string()))?;

    let messages = lesson_message::Entity::find()
        .filter(lesson_message::Column::LessonId.eq(lesson.id))
        .order_by_asc(lesson_message::Column::CreatedAt)
        .all(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let message_details: Vec<MessageDetail> = messages
        .into_iter()
        .map(|m| MessageDetail {
            id: m.id,
            role: m.role,
            content: m.content,
            created_at: m.created_at,
        })
        .collect();

    Ok(Json(LessonDetail {
        id: lesson.id,
        profile_id: lesson.profile_id,
        title: lesson.title,
        messages: message_details,
        created_at: lesson.created_at,
        updated_at: lesson.updated_at,
    }))
}

pub async fn delete_lesson(
    _auth: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    let result = lesson::Entity::delete_by_id(id)
        .exec(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if result.rows_affected == 0 {
        return Err((StatusCode::NOT_FOUND, "Lesson not found".to_string()));
    }

    Ok(StatusCode::NO_CONTENT)
}

pub async fn delete_message(
    _auth: AuthUser,
    State(state): State<AppState>,
    Path((_lesson_id, message_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, (StatusCode, String)> {
    let result = lesson_message::Entity::delete_by_id(message_id)
        .exec(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if result.rows_affected == 0 {
        return Err((StatusCode::NOT_FOUND, "Message not found".to_string()));
    }

    Ok(StatusCode::NO_CONTENT)
}
