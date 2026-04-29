use axum::{
    Json,
    extract::State,
};
use sea_orm::{EntityTrait, QueryFilter, QueryOrder, QuerySelect, ColumnTrait, DbErr};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::AppState;
use crate::auth::AuthUser;
use crate::entities::{lesson, lesson_message, user_language_profile, vocabulary, weak_point};
use crate::tool_loop;

#[derive(Deserialize)]
pub struct LessonRequest {
    pub profile_id: Uuid,
    pub lesson_id: Option<Uuid>,
    pub messages: Vec<Message>,
    pub loop_mode: Option<bool>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Serialize)]
pub struct LessonResponse {
    pub reply: String,
}

pub async fn chat(
    _auth: AuthUser,
    State(state): State<AppState>,
    Json(input): Json<LessonRequest>,
) -> Result<Json<LessonResponse>, (axum::http::StatusCode, String)> {
    let profile = user_language_profile::Entity::find_by_id(input.profile_id)
        .one(&state.db)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((axum::http::StatusCode::NOT_FOUND, "Profile not found".to_string()))?;

    let weak_points = weak_point::Entity::find()
        .filter(weak_point::Column::ProfileId.eq(input.profile_id))
        .filter(weak_point::Column::Active.eq(true))
        .all(&state.db)
        .await
        .map_err(|e: DbErr| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let lru_vocab = vocabulary::Entity::find()
        .filter(vocabulary::Column::ProfileId.eq(input.profile_id))
        .order_by_asc(vocabulary::Column::LastPracticed)
        .limit(20)
        .all(&state.db)
        .await
        .map_err(|e: DbErr| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Load all vocabulary linked to the current lesson (no limit)
    let lesson_vocab = if let Some(lid) = input.lesson_id {
        vocabulary::Entity::find()
            .filter(vocabulary::Column::LessonId.eq(lid))
            .order_by_asc(vocabulary::Column::LastPracticed)
            .all(&state.db)
            .await
            .map_err(|e: DbErr| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    } else {
        vec![]
    };

    let system_prompt = build_system_prompt(&profile, &weak_points, &lru_vocab, &lesson_vocab);

    let tools = serde_json::json!([
        {
            "name": "add_vocabulary",
            "description": "Save a new target-language word to the student's vocabulary so it can be revisited in future lessons. Invoke whenever the student encounters, asks about, or makes a lexical mistake on a word that is not already tracked. Also invoke for each related form you teach (infinitive, key conjugations, irregular forms). The persistence layer records the word, translation, and context — the student does not need to be told the word was saved.",
            "input_schema": {
                "type": "object",
                "properties": {
                    "word": {"type": "string", "description": "The word in the target language"},
                    "translation": {"type": "string", "description": "Translation in the student's explanation language"},
                    "context": {"type": "string", "description": "The sentence where the word was encountered or the mistake was made"}
                },
                "required": ["word", "translation"]
            }
        },
        {
            "name": "bump_vocabulary",
            "description": "Mark a tracked vocabulary word as needing more practice. Invoke when the student repeats a mistake on a word that is already in their vocabulary list (spelling, gender, conjugation, etc.).",
            "input_schema": {
                "type": "object",
                "properties": {
                    "word": {"type": "string", "description": "The exact word as stored in the vocabulary list"}
                },
                "required": ["word"]
            }
        },
        {
            "name": "add_weak_point",
            "description": "Record a recurring grammar or usage pattern the student struggles with (e.g. 'subjuntivo', 'ser vs estar', 'past participle agreement'). Invoke for patterns, not for individual words — single words go through the vocabulary tools.",
            "input_schema": {
                "type": "object",
                "properties": {
                    "type": {"type": "string", "enum": ["grammar", "vocabulary", "phrase"], "description": "grammar for grammar patterns, phrase for common expressions/idioms, vocabulary only for word-class mistakes"},
                    "detail": {"type": "string", "description": "Concise description of the pattern, e.g. 'subjuntivo', 'ser vs estar'"}
                },
                "required": ["type", "detail"]
            }
        },
        {
            "name": "resolve_weak_point",
            "description": "Mark a weak point as resolved when the student has consistently produced the correct form across several turns.",
            "input_schema": {
                "type": "object",
                "properties": {
                    "detail": {"type": "string", "description": "The exact weak point detail as previously recorded"}
                },
                "required": ["detail"]
            }
        },
        {
            "name": "set_topic_preference",
            "description": "Update the student's tutor style or explanation language preference when they explicitly ask for a change.",
            "input_schema": {
                "type": "object",
                "properties": {
                    "style": {"type": "string", "description": "New tutor style"},
                    "explanation_language": {"type": "string", "description": "New explanation language"}
                }
            }
        }
    ]);

    let all_api_messages: Vec<serde_json::Value> = input
        .messages
        .iter()
        // Filter out empty assistant messages that confuse smaller models
        .filter(|m| !(m.role == "assistant" && m.content.trim().is_empty()))
        .map(|m| serde_json::json!({"role": &m.role, "content": &m.content}))
        .collect();

    // Loop mode: send only first message + last 3 to save tokens
    let api_messages = if input.loop_mode.unwrap_or(false) && all_api_messages.len() > 4 {
        let mut trimmed = vec![all_api_messages[0].clone()];
        trimmed.extend_from_slice(&all_api_messages[all_api_messages.len() - 3..]);
        trimmed
    } else {
        all_api_messages
    };

    let reply = tool_loop::run(
        &state,
        &system_prompt,
        api_messages,
        tools,
        input.profile_id,
        input.lesson_id,
    )
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Persist conversation to lesson (skip empty replies)
    if let Some(lesson_id) = input.lesson_id && !reply.trim().is_empty() {
        let now = chrono::Utc::now().fixed_offset();
        let is_greeting = input.messages.last()
            .map(|m| m.content.starts_with("[lesson:greeting]"))
            .unwrap_or(false);

        // Insert the user's last message and the assistant reply as lesson_message rows
        if let Some(user_msg) = input.messages.last() {
            use sea_orm::ActiveModelTrait;

            // Skip persisting the hidden greeting prompt
            if !is_greeting {
                let user_row = lesson_message::ActiveModel {
                    id: sea_orm::Set(Uuid::new_v4()),
                    lesson_id: sea_orm::Set(lesson_id),
                    role: sea_orm::Set(user_msg.role.clone()),
                    content: sea_orm::Set(user_msg.content.clone()),
                    created_at: sea_orm::Set(now),
                };
                let _ = user_row.insert(&state.db).await;
            }

            let assistant_row = lesson_message::ActiveModel {
                id: sea_orm::Set(Uuid::new_v4()),
                lesson_id: sea_orm::Set(lesson_id),
                role: sea_orm::Set("assistant".to_string()),
                content: sea_orm::Set(reply.clone()),
                created_at: sea_orm::Set(now + chrono::Duration::milliseconds(1)),
            };
            let _ = assistant_row.insert(&state.db).await;
        }

        // Generate title from first user message if this is the first exchange
        if input.messages.len() <= 1 {
            let title = if is_greeting {
                "New lesson".to_string()
            } else {
                input
                    .messages
                    .first()
                    .map(|m| {
                        let t = m.content.chars().take(60).collect::<String>();
                        if m.content.len() > 60 { format!("{t}...") } else { t }
                    })
                    .unwrap_or_else(|| "New lesson".to_string())
            };

            if let Ok(Some(existing)) = lesson::Entity::find_by_id(lesson_id).one(&state.db).await {
                let mut active: lesson::ActiveModel = existing.into();
                active.title = sea_orm::Set(title);
                active.updated_at = sea_orm::Set(now);
                use sea_orm::ActiveModelTrait;
                let _ = active.update(&state.db).await;
            }
        } else if let Ok(Some(existing)) = lesson::Entity::find_by_id(lesson_id).one(&state.db).await {
            let mut active: lesson::ActiveModel = existing.into();
            active.updated_at = sea_orm::Set(now);
            use sea_orm::ActiveModelTrait;
            let _ = active.update(&state.db).await;
        }
    }

    Ok(Json(LessonResponse { reply }))
}

fn build_system_prompt(
    profile: &user_language_profile::Model,
    weak_points: &[weak_point::Model],
    lru_vocab: &[vocabulary::Model],
    lesson_vocab: &[vocabulary::Model],
) -> String {
    let wp_list = if weak_points.is_empty() {
        "None identified yet.".to_string()
    } else {
        weak_points
            .iter()
            .map(|wp| format!("- [{}] {}", wp.r#type, wp.detail))
            .collect::<Vec<_>>()
            .join("\n")
    };

    let vocab_list = if lru_vocab.is_empty() {
        "No vocabulary tracked yet.".to_string()
    } else {
        lru_vocab
            .iter()
            .map(|v| {
                format!(
                    "- {} → {} (errors: {})",
                    v.word, v.translation, v.error_count
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

    let personal_note_section = if profile.personal_note.is_empty() {
        String::new()
    } else {
        format!(
            "\n## Student's Personal Learning Preferences\n{}\n",
            profile.personal_note
        )
    };

    let lesson_vocab_list = if lesson_vocab.is_empty() {
        String::new()
    } else {
        let items = lesson_vocab
            .iter()
            .map(|v| {
                format!(
                    "- {} → {} (errors: {})",
                    v.word, v.translation, v.error_count
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        format!(
            "\n## Vocabulary Added This Lesson\n{items}\n"
        )
    };

    format!(
        r#"You are a language tutor for {target_language}.

Student level: {level}
Explanation language: {explanation_language}
Tutor style: {style}
{personal_note_section}
## Weak Points
{wp_list}

## Vocabulary Needing Practice (LRU order — least recently practiced first)
{vocab_list}
{lesson_vocab_list}
## Instructions
- Conduct the lesson naturally in {target_language}, adjusting complexity to {level} level.
- ONLY correct the student's actual messages. NEVER correct your own sentences or examples.
- When the student makes a mistake, correct it inline using this format:
  **Original:** <what the student said>
  **Corrected:** <correct version with **bold** on the fixed parts>
  **Mistakes:**
  1. `<wrong>` → `<right>` — <brief explanation>
- Do NOT generate incorrect examples and then correct them. If you want to teach, show only the correct form.
- Subtly incorporate weak points into the conversation to help the student practice them.
- When explaining grammar or vocabulary, use {explanation_language} language.
- Match the {style} tutor personality throughout.
- When teaching a verb, also cover its key related forms (infinitive, common conjugations at this level, important irregular forms) so the student builds a complete picture.
"#,
        target_language = profile.language,
        level = profile.level,
        explanation_language = profile.explanation_language,
        style = profile.style,
        personal_note_section = personal_note_section,
        wp_list = wp_list,
        vocab_list = vocab_list,
        lesson_vocab_list = lesson_vocab_list,
    )
}
