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

    let system_prompt = build_system_prompt(&profile, &weak_points, &lru_vocab);

    let mcp_base_url = format!("{}/mcp", state.self_url);

    let tools = serde_json::json!([
        {
            "name": "add_vocabulary",
            "description": "Add a new word to the student's vocabulary list. Use when the student asks about a word or makes a lexical mistake.",
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
            "description": "Mark a known vocabulary word as needing more practice. Use when the student repeats a mistake on a word already in their vocabulary.",
            "input_schema": {
                "type": "object",
                "properties": {
                    "word": {"type": "string", "description": "The word to bump"}
                },
                "required": ["word"]
            }
        },
        {
            "name": "add_weak_point",
            "description": "Record a recurring grammar or usage pattern the student struggles with.",
            "input_schema": {
                "type": "object",
                "properties": {
                    "type": {"type": "string", "enum": ["grammar", "vocabulary", "phrase"], "description": "Category of the weak point"},
                    "detail": {"type": "string", "description": "Description of the weak point, e.g. 'subjuntivo', 'ser vs estar'"}
                },
                "required": ["type", "detail"]
            }
        },
        {
            "name": "resolve_weak_point",
            "description": "Mark a weak point as resolved when the student consistently uses the form correctly.",
            "input_schema": {
                "type": "object",
                "properties": {
                    "detail": {"type": "string", "description": "The weak point detail to resolve"}
                },
                "required": ["detail"]
            }
        },
        {
            "name": "set_topic_preference",
            "description": "Update the student's tutor style or explanation language preference.",
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

    let reply = call_claude_with_tools(
        &state,
        &system_prompt,
        api_messages,
        tools,
        input.profile_id,
        &mcp_base_url,
    )
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Persist conversation to lesson (skip empty replies)
    if let Some(lesson_id) = input.lesson_id && !reply.trim().is_empty() {
        let now = chrono::Utc::now().fixed_offset();

        // Insert the user's last message and the assistant reply as lesson_message rows
        if let Some(user_msg) = input.messages.last() {
            use sea_orm::ActiveModelTrait;

            let user_row = lesson_message::ActiveModel {
                id: sea_orm::Set(Uuid::new_v4()),
                lesson_id: sea_orm::Set(lesson_id),
                role: sea_orm::Set(user_msg.role.clone()),
                content: sea_orm::Set(user_msg.content.clone()),
                created_at: sea_orm::Set(now),
            };
            let _ = user_row.insert(&state.db).await;

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
            let title = input
                .messages
                .first()
                .map(|m| {
                    let t = m.content.chars().take(60).collect::<String>();
                    if m.content.len() > 60 { format!("{t}...") } else { t }
                })
                .unwrap_or_else(|| "New lesson".to_string());

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

## Instructions
- NEVER reply with an empty message. Always provide meaningful content in every response.
- Conduct the lesson naturally in {target_language}, adjusting complexity to {level} level.
- When the student makes a mistake, correct it inline using this format:
  **Original:** <what they said>
  **Corrected:** <correct version with **bold** on the fixed parts>
  **Mistakes:**
  1. `<wrong>` → `<right>` — <brief explanation>
- Subtly incorporate weak points into the conversation to help the student practice them.
- When you add a new word to the student's vocabulary, briefly mention the word and its translation to the student so they know it was saved.
- Use the other tools (bump_vocabulary, add_weak_point, resolve_weak_point, set_topic_preference) silently — do not mention them to the student.
- You MUST always include a text response to the student. Never respond with only tool calls and no text.
- When explaining grammar or vocabulary, use {explanation_language}.
- Match the {style} tutor personality throughout."#,
        target_language = profile.language,
        level = profile.level,
        explanation_language = profile.explanation_language,
        style = profile.style,
        personal_note_section = personal_note_section,
        wp_list = wp_list,
        vocab_list = vocab_list,
    )
}

/// Calls the Claude API with tools, handles tool_use responses by executing them locally,
/// and returns the final text response.
async fn call_claude_with_tools(
    state: &AppState,
    system_prompt: &str,
    messages: Vec<serde_json::Value>,
    tools: serde_json::Value,
    profile_id: Uuid,
    mcp_base_url: &str,
) -> anyhow::Result<String> {
    let client = &state.http_client;
    let mut conversation = messages;

    loop {
        let body = serde_json::json!({
            "model": &state.claude_model,
            "max_tokens": 4096,
            "system": system_prompt,
            "messages": conversation,
            "tools": tools,
        });

        let resp = client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &state.anthropic_api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await?;

        let status = resp.status();
        let resp_body: serde_json::Value = resp.json().await?;

        if !status.is_success() {
            anyhow::bail!("Claude API error {}: {}", status, resp_body);
        }

        let _stop_reason = resp_body["stop_reason"].as_str().unwrap_or("");
        let content = resp_body["content"].as_array().ok_or_else(|| anyhow::anyhow!("No content in response"))?;

        // Collect text parts and tool_use parts
        let mut text_parts = Vec::new();
        let mut tool_uses = Vec::new();

        for block in content {
            match block["type"].as_str() {
                Some("text") => {
                    if let Some(text) = block["text"].as_str() {
                        text_parts.push(text.to_string());
                    }
                }
                Some("tool_use") => {
                    tool_uses.push(block.clone());
                }
                _ => {}
            }
        }

        if !tool_uses.is_empty() {
            // Add assistant message with the full content
            conversation.push(serde_json::json!({
                "role": "assistant",
                "content": content,
            }));

            // Execute each tool and collect results
            let mut tool_results = Vec::new();
            for tool_use in &tool_uses {
                let tool_name = tool_use["name"].as_str().unwrap_or("");
                let tool_id = tool_use["id"].as_str().unwrap_or("");
                let tool_input = &tool_use["input"];

                let result = execute_tool(state, tool_name, tool_input, profile_id, mcp_base_url).await;

                tool_results.push(serde_json::json!({
                    "type": "tool_result",
                    "tool_use_id": tool_id,
                    "content": match &result {
                        Ok(v) => v.to_string(),
                        Err(e) => format!("Error: {e}"),
                    },
                }));
            }

            conversation.push(serde_json::json!({
                "role": "user",
                "content": tool_results,
            }));

            continue;
        }

        // No more tool calls — return the text if we have any
        let text = text_parts.join("\n");
        if text.is_empty() {
            // Claude ended without producing text — nudge it to respond
            conversation.push(serde_json::json!({
                "role": "assistant",
                "content": content,
            }));
            conversation.push(serde_json::json!({
                "role": "user",
                "content": "Continue.",
            }));
            continue;
        }
        return Ok(text);
    }
}

async fn execute_tool(
    state: &AppState,
    tool_name: &str,
    input: &serde_json::Value,
    profile_id: Uuid,
    _mcp_base_url: &str,
) -> anyhow::Result<serde_json::Value> {
    let db = &state.db;

    match tool_name {
        "add_vocabulary" => {
            let word = input["word"].as_str().unwrap_or("").to_string();
            let translation = input["translation"].as_str().unwrap_or("").to_string();
            let context = input["context"].as_str().map(|s| s.to_string());

            let model = crate::entities::vocabulary::ActiveModel {
                id: sea_orm::Set(Uuid::new_v4()),
                profile_id: sea_orm::Set(profile_id),
                word: sea_orm::Set(word.clone()),
                translation: sea_orm::Set(translation),
                added_by: sea_orm::Set("claude".to_string()),
                context: sea_orm::Set(context),
                last_practiced: sea_orm::Set(chrono::Utc::now().into()),
                error_count: sea_orm::Set(0),
            };

            use sea_orm::ActiveModelTrait;
            model.insert(db).await?;
            Ok(serde_json::json!({"status": "added", "word": word}))
        }
        "bump_vocabulary" => {
            let word = input["word"].as_str().unwrap_or("").to_string();

            let vocab = crate::entities::vocabulary::Entity::find()
                .filter(crate::entities::vocabulary::Column::ProfileId.eq(profile_id))
                .filter(crate::entities::vocabulary::Column::Word.eq(&word))
                .one(db)
                .await?
                .ok_or_else(|| anyhow::anyhow!("Word not found: {word}"))?;

            let mut active: crate::entities::vocabulary::ActiveModel = vocab.into();
            active.last_practiced = sea_orm::Set(chrono::Utc::now().into());
            active.error_count = sea_orm::Set(active.error_count.unwrap() + 1);

            use sea_orm::ActiveModelTrait;
            active.update(db).await?;
            Ok(serde_json::json!({"status": "bumped", "word": word}))
        }
        "add_weak_point" => {
            let wp_type = input["type"].as_str().unwrap_or("grammar").to_string();
            let detail = input["detail"].as_str().unwrap_or("").to_string();

            let model = crate::entities::weak_point::ActiveModel {
                id: sea_orm::Set(Uuid::new_v4()),
                profile_id: sea_orm::Set(profile_id),
                r#type: sea_orm::Set(wp_type),
                detail: sea_orm::Set(detail.clone()),
                active: sea_orm::Set(true),
            };

            use sea_orm::ActiveModelTrait;
            model.insert(db).await?;
            Ok(serde_json::json!({"status": "added", "detail": detail}))
        }
        "resolve_weak_point" => {
            let detail = input["detail"].as_str().unwrap_or("").to_string();

            let wp = crate::entities::weak_point::Entity::find()
                .filter(crate::entities::weak_point::Column::ProfileId.eq(profile_id))
                .filter(crate::entities::weak_point::Column::Detail.eq(&detail))
                .filter(crate::entities::weak_point::Column::Active.eq(true))
                .one(db)
                .await?
                .ok_or_else(|| anyhow::anyhow!("Weak point not found: {detail}"))?;

            let mut active: crate::entities::weak_point::ActiveModel = wp.into();
            active.active = sea_orm::Set(false);

            use sea_orm::ActiveModelTrait;
            active.update(db).await?;
            Ok(serde_json::json!({"status": "resolved", "detail": detail}))
        }
        "set_topic_preference" => {
            let profile = crate::entities::user_language_profile::Entity::find_by_id(profile_id)
                .one(db)
                .await?
                .ok_or_else(|| anyhow::anyhow!("Profile not found"))?;

            let mut active: crate::entities::user_language_profile::ActiveModel = profile.into();

            if let Some(style) = input["style"].as_str() {
                active.style = sea_orm::Set(style.to_string());
            }
            if let Some(lang) = input["explanation_language"].as_str() {
                active.explanation_language = sea_orm::Set(lang.to_string());
            }

            use sea_orm::ActiveModelTrait;
            active.update(db).await?;
            Ok(serde_json::json!({"status": "updated"}))
        }
        _ => Ok(serde_json::json!({"error": format!("Unknown tool: {tool_name}")})),
    }
}
