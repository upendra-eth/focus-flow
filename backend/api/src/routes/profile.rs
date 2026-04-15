use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;
use uuid::Uuid;

use crate::middleware::auth::AuthUser;
use crate::{AppState, ErrorResponse};

#[derive(Debug, Deserialize)]
pub struct SubmitAnswerRequest {
    pub question_id: Uuid,
    pub answer_value: String,
    pub raw_input: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SkipQuestionRequest {
    pub question_id: Uuid,
}

pub async fn next_question(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let question = state
        .profiling_engine
        .select_next_question(auth.user_id)
        .await
        .map_err(|e| {
            tracing::error!(error = ?e, "failed to select next question");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "failed to select question".into(),
                    details: Some(e.to_string()),
                }),
            )
        })?;

    Ok(Json(question))
}

pub async fn submit_answer(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<SubmitAnswerRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let answer = state
        .profiling_engine
        .submit_answer(
            auth.user_id,
            body.question_id,
            &body.answer_value,
            body.raw_input.as_deref(),
            "manual",
            1.0,
        )
        .await
        .map_err(|e| {
            tracing::error!(error = ?e, "failed to submit answer");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "failed to submit answer".into(),
                    details: Some(e.to_string()),
                }),
            )
        })?;

    Ok(Json(answer))
}

pub async fn skip_question(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<SkipQuestionRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    state
        .profiling_engine
        .skip_question(auth.user_id, body.question_id)
        .await
        .map_err(|e| {
            tracing::error!(error = ?e, "failed to skip question");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "failed to skip question".into(),
                    details: Some(e.to_string()),
                }),
            )
        })?;

    Ok(Json(serde_json::json!({ "success": true })))
}
