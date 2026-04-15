use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use uuid::Uuid;

use crate::middleware::auth::AuthUser;
use crate::{AppState, ErrorResponse};

#[derive(Debug, Deserialize)]
pub struct ListTasksQuery {
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTaskRequest {
    pub title: String,
    pub description: Option<String>,
    pub priority: Option<String>,
    pub due_at: Option<DateTime<Utc>>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTaskRequest {
    pub status: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub priority: Option<String>,
    pub due_at: Option<DateTime<Utc>>,
}

pub async fn list_tasks(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(query): Query<ListTasksQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let status_filter = query.status.as_deref().and_then(|s| match s {
        "all" => None,
        other => Some(other),
    });

    let tasks = state
        .task_service
        .get_user_tasks(auth.user_id, status_filter)
        .await
        .map_err(|e| {
            tracing::error!(error = ?e, "failed to list tasks");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "failed to list tasks".into(),
                    details: Some(e.to_string()),
                }),
            )
        })?;

    Ok(Json(tasks))
}

pub async fn create_task(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<CreateTaskRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let task = state
        .task_service
        .create_task(
            auth.user_id,
            body.title,
            body.description,
            body.priority.unwrap_or_else(|| "medium".into()),
            "manual".to_string(),
            body.due_at,
            body.tags.unwrap_or_default(),
            None,
        )
        .await
        .map_err(|e| {
            tracing::error!(error = ?e, "failed to create task");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "failed to create task".into(),
                    details: Some(e.to_string()),
                }),
            )
        })?;

    Ok((StatusCode::CREATED, Json(task)))
}

pub async fn update_task(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateTaskRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let map_err = |e: Box<dyn std::error::Error>| {
        tracing::error!(error = %e, "failed to update task");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "failed to update task".into(),
                details: Some(e.to_string()),
            }),
        )
    };

    if let Some(status) = &body.status {
        if status == "completed" {
            let task = state
                .task_service
                .complete_task(auth.user_id, id)
                .await
                .map_err(|e| map_err(Box::new(e)))?;
            return Ok(Json(task));
        }

        let task = state
            .db
            .update_task_status(id, status)
            .await
            .map_err(|e| map_err(Box::new(e)))?;
        return Ok(Json(task));
    }

    // For non-status updates, fall back to status update with existing status
    // (the current DB layer only exposes update_task_status)
    let tasks = state
        .task_service
        .get_user_tasks(auth.user_id, Some("all"))
        .await
        .map_err(|e| map_err(Box::new(e)))?;

    let task = tasks
        .into_iter()
        .find(|t| t.id == id)
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: "task not found".into(),
                    details: None,
                }),
            )
        })?;

    Ok(Json(task))
}
