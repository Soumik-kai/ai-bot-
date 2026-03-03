use axum::{extract::State, Json};
use serde_json::Value;
use crate::config::AppState;
use tracing::{error, info};
use teloxide_core::types::{ChatId, MessageId};
use tokio::task;

pub async fn webhook_handler(
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> axum::response::Json<Value> {
    // Parse Telegram update JSON minimally
    // We keep parsing simple: handle message updates with text
    if let Some(message) = payload.get("message").or_else(|| payload.get("edited_message")) {
        if let Some(chat) = message.get("chat") {
            let chat_id = chat.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
            let is_private = chat.get("type").and_then(|t| t.as_str()) == Some("private");
            let from_id = message.get("from").and_then(|f| f.get("id")).and_then(|v| v.as_i64()).unwrap_or(0);
            let text = message.get("text").and_then(|t| t.as_str()).unwrap_or("");

            // Enforce group-only usage
            if is_private {
                match crate::db::is_admin_or_authorized(&state.db, from_id, state.cfg.allowed_group_id).await {
                    Ok(true) => { /* allowed */ }
                    Ok(false) | Err(_) => {
                        let _ = send_message(&state, chat_id, "Personal use restricted to admin/authorized IDs.").await;
                        return Json(json!({"ok": true}));
                    }
                }
            } else {
                if state.cfg.allowed_group_id != 0 && chat_id != state.cfg.allowed_group_id {
                    // ignore messages from other groups
                    return Json(json!({"ok": true}));
                }
            }

            // Activation: /ask command
            if text.starts_with("/ask") {
                let question = text.trim_start_matches("/ask").trim().to_string();
                if question.is_empty() {
                    let _ = send_message(&state, chat_id, "Usage: /ask <your question>").await;
                    return Json(json!({"ok": true}));
                }
                let state_clone = state.clone();
                let message_clone = message.clone();
                task::spawn(async move {
                    if let Err(e) = process_question(state_clone, message_clone, question).await {
                        error!("process_question error: {:?}", e);
                    }
                });
                return Json(json!({"ok": true}));
            }

            // Mention handling (crude)
            if text.contains(&format!("@{}", state.cfg.telegram_bot_username)) {
                let question = text.replace(&format!("@{}", state.cfg.telegram_bot_username), "").trim().to_string();
                if !question.is_empty() {
                    let state_clone = state.clone();
                    let message_clone = message.clone();
                    task::spawn(async move {
                        if let Err(e) = process_question(state_clone, message_clone, question).await {
                            error!("process_question error: {:?}", e);
                        }
                    });
                }
                return Json(json!({"ok": true}));
            }

            // Reply-to-bot handling: if reply_to_message.from.is_bot and bot username matches
            if let Some(reply_to) = message.get("reply_to_message") {
                if let Some(from) = reply_to.get("from") {
                    if from.get("is_bot").and_then(|b| b.as_bool()) == Some(true) {
                        // treat as follow-up
                        let question = text.to_string();
                        let state_clone = state.clone();
                        let message_clone = message.clone();
                        task::spawn(async move {
                            if let Err(e) = process_question(state_clone, message_clone, question).await {
                                error!("process_question error: {:?}", e);
                            }
                        });
                        return Json(json!({"ok": true}));
                    }
                }
            }
        }
    }

    Json(json!({"ok": true}))
}

async fn process_question(state: AppState, message: serde_json::Value, question: String) -> anyhow::Result<()> {
    let chat_id = message.get("chat").and_then(|c| c.get("id")).and_then(|v| v.as_i64()).unwrap_or(0);
    let message_id = message.get("message_id").and_then(|v| v.as_i64()).unwrap_or(0);

    // 1) send typing action
    let _ = send_chat_action(&state, chat_id, "typing").await;

    // 2) run search
    let search_results = match state.search.search(&question).await {
        Ok(r) => r,
        Err(e) => {
            error!("search failed: {:?}", e);
            let _ = send_message(&state, chat_id, "Search failed; try again later.").await;
            return Ok(());
        }
    };

    // 3) build prompt
    let prompt = build_prompt(&question, &search_results);

    // 4) call LLM with fallback
    let answer = match state.llm_pool.call_with_fallback(&prompt, 1024).await {
        Ok(a) => a,
        Err(e) => {
            error!("llm failed: {:?}", e);
            let _ = send_message(&state, chat_id, "All models are currently unavailable.").await;
            return Ok(());
        }
    };

    // 5) stream reply (simulate by editing)
    if let Err(e) = stream_reply(&state, chat_id, message_id, &answer).await {
        error!("stream failed: {:?}", e);
        let _ = send_message(&state, chat_id, &answer).await;
    }

    Ok(())
}

fn build_prompt(question: &str, search_results: &Vec<crate::adapters::search::SearchResult>) -> String {
    let mut prompt = String::new();
    prompt.push_str("You are an assistant answering current affairs questions.\n");
    prompt.push_str("User question:\n");
    prompt.push_str(question);
    prompt.push_str("\n\nSearch results:\n");
    for r in search_results.iter().take(5) {
        prompt.push_str(&format!("- {}: {} ({})\n", r.title, r.snippet, r.url));
    }
    prompt.push_str("\nAnswer concisely and cite sources inline.\n");
    prompt
}

async fn send_message(state: &AppState, chat_id: i64, text: &str) -> anyhow::Result<()> {
    let token = &state.cfg.telegram_token;
    let url = format!("https://api.telegram.org/{}/sendMessage", token);
    let body = serde_json::json!({
        "chat_id": chat_id,
        "text": text,
        "reply_to_message_id": null
    });
    let _ = state.http.post(&url).json(&body).send().await?;
    Ok(())
}

async fn send_chat_action(state: &AppState, chat_id: i64, action: &str) -> anyhow::Result<()> {
    let token = &state.cfg.telegram_token;
    let url = format!("https://api.telegram.org/{}/sendChatAction", token);
    let body = serde_json::json!({
        "chat_id": chat_id,
        "action": action
    });
    let _ = state.http.post(&url).json(&body).send().await?;
    Ok(())
}

async fn stream_reply(state: &AppState, chat_id: i64, reply_to: i64, text: &str) -> anyhow::Result<()> {
    // send initial placeholder
    let token = &state.cfg.telegram_token;
    let send_url = format!("https://api.telegram.org/{}/sendMessage", token);
    let body = serde_json::json!({
        "chat_id": chat_id,
        "text": "...",
        "reply_to_message_id": reply_to
    });
    let resp = state.http.post(&send_url).json(&body).send().await?;
    let resp_json: serde_json::Value = resp.json().await?;
    let message_id = resp_json.get("result").and_then(|r| r.get("message_id")).and_then(|v| v.as_i64()).unwrap_or(0);

    // edit in chunks
    let chunk = 300;
    let mut i = 0usize;
    while i < text.len() {
        let end = std::cmp::min(i + chunk, text.len());
        let part = &text[..end];
        let edit_url = format!("https://api.telegram.org/{}/editMessageText", token);
        let edit_body = serde_json::json!({
            "chat_id": chat_id,
            "message_id": message_id,
            "text": part
        });
        let _ = state.http.post(&edit_url).json(&edit_body).send().await?;
        i = end;
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    }
    Ok(())
}