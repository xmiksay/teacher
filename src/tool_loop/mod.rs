//! Tool loop: drives the LLM through repeated request → tool execution → request
//! cycles until it returns a final text response with no further tool calls.
//!
//! Each iteration:
//! 1. Send the conversation + tool schema to the LLM (Claude or Ollama).
//! 2. If the response contains tool calls, execute them locally against the DB,
//!    append the assistant turn and tool results to the conversation, loop again.
//! 3. If the response is text-only, return it. If it's empty, nudge the LLM to continue.

use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use uuid::Uuid;

use crate::{AppState, LlmProvider};
use crate::entities::{user_language_profile, vocabulary, weak_point};

struct ToolCall {
    id: String,
    name: String,
    input: serde_json::Value,
}

struct LlmResponse {
    text_parts: Vec<String>,
    tool_calls: Vec<ToolCall>,
    /// Raw assistant content to echo back into the conversation for tool round-trips.
    raw_assistant: serde_json::Value,
}

/// Strip `<think>...</think>` blocks that reasoning models (Qwen, DeepSeek) emit.
fn strip_think_blocks(text: &str) -> String {
    let mut result = String::new();
    let mut rest = text;
    while let Some(start) = rest.find("<think>") {
        result.push_str(&rest[..start]);
        if let Some(end) = rest[start..].find("</think>") {
            rest = &rest[start + end + "</think>".len()..];
        } else {
            return result.trim().to_string();
        }
    }
    result.push_str(rest);
    result.trim().to_string()
}

/// Run the tool loop until the LLM returns a non-empty text response.
pub async fn run(
    state: &AppState,
    system_prompt: &str,
    messages: Vec<serde_json::Value>,
    tools: serde_json::Value,
    profile_id: Uuid,
    lesson_id: Option<Uuid>,
) -> anyhow::Result<String> {
    let client = &state.http_client;
    let mut conversation = messages;

    loop {
        let resp = match &state.llm {
            LlmProvider::Claude { .. } => {
                send_claude_request(client, &state.llm, system_prompt, &conversation, &tools).await?
            }
            LlmProvider::Ollama { .. } => {
                send_ollama_request(client, &state.llm, system_prompt, &conversation, &tools).await?
            }
        };

        if !resp.tool_calls.is_empty() {
            tracing::info!(
                "LLM returned {} tool call(s): {}",
                resp.tool_calls.len(),
                resp.tool_calls.iter().map(|tc| tc.name.as_str()).collect::<Vec<_>>().join(", ")
            );

            let mut results = Vec::new();
            for tc in &resp.tool_calls {
                let result = execute_tool(state, &tc.name, &tc.input, profile_id, lesson_id).await;
                let content = match &result {
                    Ok(v) => {
                        if matches!(state.llm, LlmProvider::Ollama { .. }) {
                            tracing::info!("Tool '{}' result: {v}", tc.name);
                        }
                        v.to_string()
                    }
                    Err(e) => {
                        tracing::error!("Tool '{}' failed: {e}", tc.name);
                        format!("Error: {e}")
                    }
                };
                results.push((tc.id.clone(), content));
            }

            match &state.llm {
                LlmProvider::Claude { .. } => {
                    conversation.push(serde_json::json!({
                        "role": "assistant",
                        "content": resp.raw_assistant,
                    }));
                    let tool_results: Vec<serde_json::Value> = results.iter().map(|(id, content)| {
                        serde_json::json!({
                            "type": "tool_result",
                            "tool_use_id": id,
                            "content": content,
                        })
                    }).collect();
                    conversation.push(serde_json::json!({
                        "role": "user",
                        "content": tool_results,
                    }));
                }
                LlmProvider::Ollama { .. } => {
                    conversation.push(serde_json::json!({
                        "role": "assistant",
                        "content": resp.raw_assistant,
                        "tool_calls": resp.tool_calls.iter().map(|tc| serde_json::json!({
                            "id": tc.id,
                            "type": "function",
                            "function": {
                                "name": tc.name,
                                "arguments": tc.input,
                            }
                        })).collect::<Vec<_>>(),
                    }));
                    for (id, content) in &results {
                        conversation.push(serde_json::json!({
                            "role": "tool",
                            "tool_call_id": id,
                            "content": content,
                        }));
                    }
                }
            }
            continue;
        }

        tracing::debug!("LLM returned text only, no tool calls");
        let text = resp.text_parts.join("\n");
        if text.is_empty() {
            // Nudge the LLM to produce a text reply.
            conversation.push(serde_json::json!({
                "role": "assistant",
                "content": resp.raw_assistant,
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

async fn send_claude_request(
    client: &reqwest::Client,
    provider: &LlmProvider,
    system_prompt: &str,
    conversation: &[serde_json::Value],
    tools: &serde_json::Value,
) -> anyhow::Result<LlmResponse> {
    let LlmProvider::Claude { api_key, model } = provider else {
        unreachable!()
    };

    // Claude API requires the first message to have role "user".
    // When a conversation is restored from the DB the hidden greeting prompt
    // is not persisted, so the first message can be "assistant".
    let mut messages = conversation.to_vec();
    if messages.first().is_some_and(|m| m["role"] != "user") {
        messages.insert(0, serde_json::json!({"role": "user", "content": "Hello, let's continue our lesson."}));
    }

    // Inject a tool-use reminder before the last user message so Claude doesn't
    // lose track of tools in long, text-only conversation histories.
    if messages.len() > 2 {
        let reminder = serde_json::json!({
            "role": "user",
            "content": "REMINDER: You have tools available (add_vocabulary, bump_vocabulary, add_weak_point, resolve_weak_point, set_topic_preference). When teaching new words, you MUST call add_vocabulary for each word. When the student makes a grammar mistake, call add_weak_point. Do NOT just describe actions in text — execute them with the tools."
        });
        if let Some(pos) = messages.iter().rposition(|m| m["role"] == "user") {
            messages.insert(pos, reminder);
            messages.insert(pos + 1, serde_json::json!({
                "role": "assistant",
                "content": "Understood, I will use the tools."
            }));
        }
    }

    let body = serde_json::json!({
        "model": model,
        "max_tokens": 4096,
        "system": system_prompt,
        "messages": messages,
        "tools": tools,
    });

    let body_bytes = serde_json::to_vec(&body)
        .map_err(|e| anyhow::anyhow!("Failed to serialize Claude request body: {e}"))?;

    tracing::debug!("Claude request: {} messages, body size: {} bytes", messages.len(), body_bytes.len());

    let resp = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .body(body_bytes)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("Claude HTTP request failed: {e:#}"))?;

    let status = resp.status();
    let resp_text = resp.text().await
        .map_err(|e| anyhow::anyhow!("Failed to read Claude response body: {e}"))?;

    if !status.is_success() {
        anyhow::bail!("Claude API error {}: {}", status, resp_text);
    }

    let resp_body: serde_json::Value = serde_json::from_str(&resp_text)
        .map_err(|e| anyhow::anyhow!("Failed to parse Claude response: {e}"))?;

    let content = resp_body["content"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("No content in response"))?;

    let mut text_parts = Vec::new();
    let mut tool_calls = Vec::new();

    for block in content {
        match block["type"].as_str() {
            Some("text") => {
                if let Some(text) = block["text"].as_str() {
                    text_parts.push(text.to_string());
                }
            }
            Some("tool_use") => {
                tool_calls.push(ToolCall {
                    id: block["id"].as_str().unwrap_or("").to_string(),
                    name: block["name"].as_str().unwrap_or("").to_string(),
                    input: block["input"].clone(),
                });
            }
            _ => {}
        }
    }

    Ok(LlmResponse {
        text_parts,
        tool_calls,
        raw_assistant: serde_json::json!(content),
    })
}

fn to_openai_tools(anthropic_tools: &serde_json::Value) -> serde_json::Value {
    let tools = anthropic_tools.as_array().unwrap();
    serde_json::json!(tools.iter().map(|t| {
        serde_json::json!({
            "type": "function",
            "function": {
                "name": t["name"],
                "description": t["description"],
                "parameters": t["input_schema"],
            }
        })
    }).collect::<Vec<_>>())
}

async fn send_ollama_request(
    client: &reqwest::Client,
    provider: &LlmProvider,
    system_prompt: &str,
    conversation: &[serde_json::Value],
    tools: &serde_json::Value,
) -> anyhow::Result<LlmResponse> {
    let LlmProvider::Ollama { base_url, model } = provider else {
        unreachable!()
    };

    let mut messages = vec![serde_json::json!({"role": "system", "content": system_prompt})];

    if conversation.first().is_some_and(|m| m["role"] != "user") {
        messages.push(serde_json::json!({"role": "user", "content": "Hello, let's continue our lesson."}));
    }
    messages.extend_from_slice(conversation);

    if let Some(pos) = messages.iter().rposition(|m| m["role"] == "user") {
        messages.insert(pos, serde_json::json!({
            "role": "system",
            "content": "REMINDER: You MUST use the provided tools via function calls. Call add_vocabulary for each new word. Call add_weak_point for grammar mistakes. Do NOT write tool names in text — execute them as function calls."
        }));
    }

    let openai_tools = to_openai_tools(tools);

    let body = serde_json::json!({
        "model": model,
        "messages": messages,
        "tools": openai_tools,
        "stream": false,
    });

    tracing::debug!("Ollama request to {base_url}/api/chat model={model} body={body}");

    let resp = client
        .post(format!("{base_url}/api/chat"))
        .header("content-type", "application/json")
        .json(&body)
        .send()
        .await?;

    let status = resp.status();
    let resp_body: serde_json::Value = resp.json().await?;

    if !status.is_success() {
        tracing::error!("Ollama API error {status}: {resp_body}");
        anyhow::bail!("Ollama API error {}: {}", status, resp_body);
    }

    tracing::debug!("Ollama raw response: {resp_body}");

    let message = &resp_body["message"];
    if message.is_null() {
        tracing::error!("Ollama response missing message field: {resp_body}");
        anyhow::bail!("Ollama response missing message field");
    }

    let raw_content = message["content"].as_str().unwrap_or("");
    let text_content = strip_think_blocks(raw_content);

    let mut text_parts = Vec::new();
    if !text_content.is_empty() {
        text_parts.push(text_content.clone());
    }

    let mut tool_calls = Vec::new();
    if let Some(tcs) = message["tool_calls"].as_array() {
        for tc in tcs {
            let func = &tc["function"];
            let name = func["name"].as_str().unwrap_or("");

            if name.is_empty() {
                tracing::warn!("Ollama returned tool_call with empty name: {tc}");
                continue;
            }

            let input = if func["arguments"].is_object() {
                func["arguments"].clone()
            } else if let Some(s) = func["arguments"].as_str() {
                serde_json::from_str(s).unwrap_or_else(|e| {
                    tracing::warn!("Ollama tool '{name}' invalid JSON arguments: {e}, raw: {s}");
                    func["arguments"].clone()
                })
            } else {
                tracing::warn!("Ollama tool '{name}' unexpected arguments type: {}", func["arguments"]);
                serde_json::json!({})
            };

            let id = format!("call_{}", Uuid::new_v4());

            tracing::info!("Ollama tool call: {name}({input})");
            tool_calls.push(ToolCall { id, name: name.to_string(), input });
        }
    }

    if tool_calls.is_empty() && text_content.is_empty() {
        tracing::warn!("Ollama returned neither text nor tool calls: {resp_body}");
    }

    Ok(LlmResponse {
        text_parts,
        tool_calls,
        raw_assistant: serde_json::json!(text_content),
    })
}

async fn execute_tool(
    state: &AppState,
    tool_name: &str,
    input: &serde_json::Value,
    profile_id: Uuid,
    lesson_id: Option<Uuid>,
) -> anyhow::Result<serde_json::Value> {
    use sea_orm::{ActiveModelTrait, Set};
    let db = &state.db;

    match tool_name {
        "add_vocabulary" => {
            let word = input["word"].as_str().unwrap_or("").to_string();
            let translation = input["translation"].as_str().unwrap_or("").to_string();
            let context = input["context"].as_str().map(|s| s.to_string());

            let model = vocabulary::ActiveModel {
                id: Set(Uuid::new_v4()),
                profile_id: Set(profile_id),
                word: Set(word.clone()),
                translation: Set(translation),
                added_by: Set("claude".to_string()),
                context: Set(context),
                last_practiced: Set(chrono::Utc::now().into()),
                error_count: Set(0),
                lesson_id: Set(lesson_id),
            };

            model.insert(db).await?;
            Ok(serde_json::json!({"status": "added", "word": word}))
        }
        "bump_vocabulary" => {
            let word = input["word"].as_str().unwrap_or("").to_string();

            let vocab = vocabulary::Entity::find()
                .filter(vocabulary::Column::ProfileId.eq(profile_id))
                .filter(vocabulary::Column::Word.eq(&word))
                .one(db)
                .await?
                .ok_or_else(|| anyhow::anyhow!("Word not found: {word}"))?;

            let mut active: vocabulary::ActiveModel = vocab.into();
            active.last_practiced = Set(chrono::Utc::now().into());
            active.error_count = Set(active.error_count.unwrap() + 1);

            active.update(db).await?;
            Ok(serde_json::json!({"status": "bumped", "word": word}))
        }
        "add_weak_point" => {
            let wp_type = input["type"].as_str().unwrap_or("grammar").to_string();
            let detail = input["detail"].as_str().unwrap_or("").to_string();

            let model = weak_point::ActiveModel {
                id: Set(Uuid::new_v4()),
                profile_id: Set(profile_id),
                r#type: Set(wp_type),
                detail: Set(detail.clone()),
                active: Set(true),
            };

            model.insert(db).await?;
            Ok(serde_json::json!({"status": "added", "detail": detail}))
        }
        "resolve_weak_point" => {
            let detail = input["detail"].as_str().unwrap_or("").to_string();

            let wp = weak_point::Entity::find()
                .filter(weak_point::Column::ProfileId.eq(profile_id))
                .filter(weak_point::Column::Detail.eq(&detail))
                .filter(weak_point::Column::Active.eq(true))
                .one(db)
                .await?
                .ok_or_else(|| anyhow::anyhow!("Weak point not found: {detail}"))?;

            let mut active: weak_point::ActiveModel = wp.into();
            active.active = Set(false);

            active.update(db).await?;
            Ok(serde_json::json!({"status": "resolved", "detail": detail}))
        }
        "set_topic_preference" => {
            let profile = user_language_profile::Entity::find_by_id(profile_id)
                .one(db)
                .await?
                .ok_or_else(|| anyhow::anyhow!("Profile not found"))?;

            let mut active: user_language_profile::ActiveModel = profile.into();

            if let Some(style) = input["style"].as_str() {
                active.style = Set(style.to_string());
            }
            if let Some(lang) = input["explanation_language"].as_str() {
                active.explanation_language = Set(lang.to_string());
            }

            active.update(db).await?;
            Ok(serde_json::json!({"status": "updated"}))
        }
        _ => Ok(serde_json::json!({"error": format!("Unknown tool: {tool_name}")})),
    }
}
