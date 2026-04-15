use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use chrono::{NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use focusflow_ai::agent::AgentMessage;

use crate::middleware::auth::AuthUser;
use crate::{AppState, ErrorResponse};

// ---------------------------------------------------------------------------
// Request / Response types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct AgentChatRequest {
    pub conversation_id: Option<Uuid>,
    pub text: String,
    pub images: Option<Vec<ImageAttachment>>,
}

#[derive(Debug, Deserialize)]
pub struct ImageAttachment {
    pub mime_type: String,
    pub data: String,
}

#[derive(Debug, Serialize)]
pub struct AgentChatResponse {
    pub conversation_id: Uuid,
    pub reply: String,
    pub entries_created: Vec<EntryBrief>,
    pub web_search_used: bool,
}

#[derive(Debug, Serialize)]
pub struct EntryBrief {
    pub id: Uuid,
    pub category: String,
    pub title: String,
}

#[derive(Debug, Deserialize)]
pub struct EntriesQuery {
    pub date: Option<String>,
    pub category: Option<String>,
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct DashboardQuery {
    pub date: Option<String>,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn err(status: StatusCode, msg: &str, details: Option<String>) -> (StatusCode, Json<ErrorResponse>) {
    (
        status,
        Json(ErrorResponse {
            error: msg.to_string(),
            details,
        }),
    )
}

fn internal(msg: &str, e: impl std::fmt::Display) -> (StatusCode, Json<ErrorResponse>) {
    tracing::error!(error = %e, "{msg}");
    err(StatusCode::INTERNAL_SERVER_ERROR, msg, Some(e.to_string()))
}

fn parse_date(s: &str) -> Result<NaiveDate, (StatusCode, Json<ErrorResponse>)> {
    NaiveDate::parse_from_str(s, "%Y-%m-%d").map_err(|e| {
        err(
            StatusCode::BAD_REQUEST,
            "invalid date format, expected YYYY-MM-DD",
            Some(e.to_string()),
        )
    })
}

// ---------------------------------------------------------------------------
// POST /api/v1/agent/chat
// ---------------------------------------------------------------------------

pub async fn agent_chat(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<AgentChatRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    if body.text.trim().is_empty() {
        return Err(err(StatusCode::BAD_REQUEST, "text cannot be empty", None));
    }

    let pool = &state.db.pool;

    // 1. Load or create conversation
    let conversation_id = if let Some(cid) = body.conversation_id {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM conversations WHERE id = $1 AND user_id = $2)",
        )
        .bind(cid)
        .bind(auth.user_id)
        .fetch_one(pool)
        .await
        .map_err(|e| internal("failed to check conversation", e))?;

        if !exists {
            return Err(err(StatusCode::NOT_FOUND, "conversation not found", None));
        }
        cid
    } else {
        let title = body.text.chars().take(60).collect::<String>();
        let row: (Uuid,) = sqlx::query_as(
            "INSERT INTO conversations (user_id, title) VALUES ($1, $2) RETURNING id",
        )
        .bind(auth.user_id)
        .bind(&title)
        .fetch_one(pool)
        .await
        .map_err(|e| internal("failed to create conversation", e))?;
        row.0
    };

    // 2. Load conversation history
    let history_rows: Vec<(String, String, Option<Vec<String>>)> = sqlx::query_as(
        r#"SELECT role, content, image_urls
           FROM conversation_messages
           WHERE conversation_id = $1
           ORDER BY created_at ASC
           LIMIT 50"#,
    )
    .bind(conversation_id)
    .fetch_all(pool)
    .await
    .map_err(|e| internal("failed to load history", e))?;

    let history: Vec<AgentMessage> = history_rows
        .into_iter()
        .map(|(role, content, imgs)| AgentMessage {
            role: if role == "assistant" {
                "model".to_string()
            } else {
                "user".to_string()
            },
            text: content,
            image_base64: imgs,
        })
        .collect();

    // 3. Build image tuples for the agent
    let images: Vec<(String, String)> = body
        .images
        .unwrap_or_default()
        .into_iter()
        .map(|img| (img.mime_type, img.data))
        .collect();

    let image_refs: Vec<(&str, &str)> = images
        .iter()
        .map(|(m, d)| (m.as_str(), d.as_str()))
        .collect();

    // 4. Call the agent
    let agent_response = state
        .agent
        .chat(&history, &body.text, &image_refs, false)
        .await
        .map_err(|e| internal("agent chat failed", e))?;

    // 5. Save user message
    let image_urls: Option<Vec<String>> = if images.is_empty() {
        None
    } else {
        Some(images.iter().map(|(m, _)| format!("inline:{m}")).collect())
    };

    let user_msg_id: (Uuid,) = sqlx::query_as(
        r#"INSERT INTO conversation_messages (conversation_id, role, content, image_urls)
           VALUES ($1, 'user', $2, $3)
           RETURNING id"#,
    )
    .bind(conversation_id)
    .bind(&body.text)
    .bind(&image_urls.as_deref())
    .fetch_one(pool)
    .await
    .map_err(|e| internal("failed to save user message", e))?;

    // 6. Save assistant message
    sqlx::query(
        r#"INSERT INTO conversation_messages (conversation_id, role, content, metadata)
           VALUES ($1, 'assistant', $2, $3)"#,
    )
    .bind(conversation_id)
    .bind(&agent_response.reply)
    .bind(serde_json::json!({
        "entries_count": agent_response.entries.len(),
        "web_search_used": agent_response.web_search_used,
    }))
    .execute(pool)
    .await
    .map_err(|e| internal("failed to save assistant message", e))?;

    // 7. Update conversation timestamp & title
    sqlx::query("UPDATE conversations SET updated_at = now() WHERE id = $1")
        .bind(conversation_id)
        .execute(pool)
        .await
        .ok();

    // 8. Save extracted life entries
    let today = Utc::now().date_naive();
    let mut entries_created = Vec::new();

    for entry in &agent_response.entries {
        let row: (Uuid,) = sqlx::query_as(
            r#"INSERT INTO life_entries
               (user_id, category, title, content, structured_data, source_message_id, tags, entry_date)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
               RETURNING id"#,
        )
        .bind(auth.user_id)
        .bind(&entry.category)
        .bind(&entry.title)
        .bind(&entry.content)
        .bind(&entry.structured_data)
        .bind(user_msg_id.0)
        .bind(&entry.tags)
        .bind(today)
        .fetch_one(pool)
        .await
        .map_err(|e| internal("failed to save life entry", e))?;

        entries_created.push(EntryBrief {
            id: row.0,
            category: entry.category.clone(),
            title: entry.title.clone(),
        });
    }

    Ok(Json(AgentChatResponse {
        conversation_id,
        reply: agent_response.reply,
        entries_created,
        web_search_used: agent_response.web_search_used,
    }))
}

// ---------------------------------------------------------------------------
// GET /api/v1/agent/entries
// ---------------------------------------------------------------------------

pub async fn get_entries(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(query): Query<EntriesQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let pool = &state.db.pool;
    let limit = query.limit.unwrap_or(50).min(200);

    let date = query.date.as_deref().map(parse_date).transpose()?;

    let rows: Vec<serde_json::Value> = sqlx::query_scalar(
        r#"SELECT json_build_object(
               'id', id,
               'category', category,
               'title', title,
               'content', content,
               'structured_data', structured_data,
               'tags', tags,
               'entry_date', entry_date,
               'created_at', created_at
           )
           FROM life_entries
           WHERE user_id = $1
             AND ($2::date IS NULL OR entry_date = $2)
             AND ($3::text IS NULL OR category = $3)
           ORDER BY entry_date DESC, created_at DESC
           LIMIT $4"#,
    )
    .bind(auth.user_id)
    .bind(date)
    .bind(query.category.as_deref())
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(|e| internal("failed to fetch entries", e))?;

    Ok(Json(rows))
}

// ---------------------------------------------------------------------------
// GET /api/v1/agent/dashboard
// ---------------------------------------------------------------------------

pub async fn get_dashboard(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(query): Query<DashboardQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let pool = &state.db.pool;
    let date = match query.date.as_deref() {
        Some(d) => parse_date(d)?,
        None => Utc::now().date_naive(),
    };

    let dashboard: Option<serde_json::Value> = sqlx::query_scalar(
        r#"SELECT json_build_object(
               'summary_text', summary_text,
               'mood_score', mood_score,
               'energy_score', energy_score,
               'categories_breakdown', categories_breakdown,
               'highlights', highlights,
               'financial_summary', financial_summary
           )
           FROM daily_dashboards
           WHERE user_id = $1 AND dashboard_date = $2"#,
    )
    .bind(auth.user_id)
    .bind(date)
    .fetch_optional(pool)
    .await
    .map_err(|e| internal("failed to fetch dashboard", e))?;

    Ok(Json(dashboard))
}

// ---------------------------------------------------------------------------
// GET /api/v1/agent/conversations
// ---------------------------------------------------------------------------

pub async fn list_conversations(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let pool = &state.db.pool;

    let rows: Vec<serde_json::Value> = sqlx::query_scalar(
        r#"SELECT json_build_object(
               'id', id,
               'title', title,
               'updated_at', updated_at
           )
           FROM conversations
           WHERE user_id = $1
           ORDER BY updated_at DESC
           LIMIT 20"#,
    )
    .bind(auth.user_id)
    .fetch_all(pool)
    .await
    .map_err(|e| internal("failed to fetch conversations", e))?;

    Ok(Json(rows))
}
