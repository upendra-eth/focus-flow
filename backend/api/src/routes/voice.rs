use axum::extract::{Multipart, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use chrono::Utc;
use serde::Serialize;
use uuid::Uuid;

use focusflow_ai::classifier::ClassificationContext;
use focusflow_ai::types::{Intent, IntentData};

use crate::middleware::auth::AuthUser;
use crate::{AppState, ErrorResponse};

#[derive(Debug, Serialize)]
pub struct VoiceUploadResponse {
    pub transcript: String,
    pub intent: Intent,
    pub confidence: f32,
    pub action_taken: String,
    pub data: serde_json::Value,
}

pub async fn upload_voice(
    State(state): State<AppState>,
    auth: AuthUser,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let mut audio_bytes: Option<Vec<u8>> = None;
    let mut filename = "audio.webm".to_string();

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "invalid multipart data".into(),
                details: Some(e.to_string()),
            }),
        )
    })? {
        if field.name() == Some("audio") {
            if let Some(name) = field.file_name() {
                filename = name.to_string();
            }
            audio_bytes = Some(
                field
                    .bytes()
                    .await
                    .map_err(|e| {
                        (
                            StatusCode::BAD_REQUEST,
                            Json(ErrorResponse {
                                error: "failed to read audio data".into(),
                                details: Some(e.to_string()),
                            }),
                        )
                    })?
                    .to_vec(),
            );
        }
    }

    let audio_data = audio_bytes.ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "missing 'audio' field in multipart form".into(),
                details: None,
            }),
        )
    })?;

    let transcription = state
        .whisper
        .transcribe(audio_data, &filename)
        .await
        .map_err(|e| {
            tracing::error!(error = ?e, "whisper transcription failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "transcription failed".into(),
                    details: Some(e.to_string()),
                }),
            )
        })?;

    let pending_question = match state
        .profiling_engine
        .select_next_question(auth.user_id)
        .await
    {
        Ok(Some(q)) => Some((q.id.to_string(), q.question_text.clone())),
        _ => None,
    };

    let recent_tasks = state
        .task_service
        .get_user_tasks(auth.user_id, Some("pending"))
        .await
        .unwrap_or_default()
        .into_iter()
        .take(5)
        .map(|t| t.title)
        .collect();

    let hour: u32 = Utc::now().format("%H").to_string().parse().unwrap_or(12);
    let time_of_day = match hour {
        5..=11 => "morning",
        12..=16 => "afternoon",
        17..=20 => "evening",
        _ => "night",
    }
    .to_string();

    let context = ClassificationContext {
        pending_question,
        recent_task_titles: recent_tasks,
        time_of_day,
    };

    let classification = state
        .classifier
        .classify(&transcription.text, context)
        .await
        .map_err(|e| {
            tracing::error!(error = ?e, "classification failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "intent classification failed".into(),
                    details: Some(e.to_string()),
                }),
            )
        })?;

    let (action_taken, data) = route_intent(&state, &auth, &transcription.text, &classification).await?;

    Ok(Json(VoiceUploadResponse {
        transcript: transcription.text,
        intent: classification.intent,
        confidence: classification.confidence,
        action_taken,
        data,
    }))
}

async fn route_intent(
    state: &AppState,
    auth: &AuthUser,
    transcript: &str,
    classification: &focusflow_ai::types::ClassificationResult,
) -> Result<(String, serde_json::Value), (StatusCode, Json<ErrorResponse>)> {
    match (&classification.intent, &classification.data) {
        (Intent::CreateTask, IntentData::Task { title, priority_hint, .. }) => {
            let task = state
                .task_service
                .create_task(
                    auth.user_id,
                    title.clone(),
                    None,
                    priority_hint.clone(),
                    "voice".to_string(),
                    None,
                    vec![],
                    Some(serde_json::json!({
                        "source": "voice",
                        "confidence": classification.confidence,
                    })),
                )
                .await
                .map_err(|e| {
                    tracing::error!(error = ?e, "failed to create task from voice");
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ErrorResponse {
                            error: "failed to create task".into(),
                            details: Some(e.to_string()),
                        }),
                    )
                })?;
            Ok(("task_created".into(), serde_json::to_value(&task).unwrap_or_default()))
        }

        (Intent::ThoughtDump, IntentData::ThoughtDump { summary, sentiment }) => {
            let entry = state
                .db
                .save_thought_entry(auth.user_id, transcript, Some(summary.as_str()), Some(sentiment.as_str()))
                .await
                .map_err(|e| {
                    tracing::error!(error = ?e, "failed to save thought entry");
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ErrorResponse {
                            error: "failed to save thought".into(),
                            details: Some(e.to_string()),
                        }),
                    )
                })?;
            Ok(("thought_saved".into(), serde_json::to_value(&entry).unwrap_or_default()))
        }

        (Intent::ProfileAnswer, IntentData::ProfileAnswer { matched_question_id, extracted_answer }) => {
            if let Some(qid) = matched_question_id.as_ref().and_then(|s| Uuid::parse_str(s).ok()) {
                let answer = state
                    .profiling_engine
                    .submit_answer(
                        auth.user_id,
                        qid,
                        extracted_answer,
                        Some(transcript),
                        "voice",
                        classification.confidence,
                    )
                    .await
                    .map_err(|e| {
                        tracing::error!(error = ?e, "failed to submit profile answer");
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(ErrorResponse {
                                error: "failed to submit answer".into(),
                                details: Some(e.to_string()),
                            }),
                        )
                    })?;
                Ok(("profile_answer_submitted".into(), serde_json::to_value(&answer).unwrap_or_default()))
            } else {
                Ok(("clarification_needed".into(), serde_json::json!({
                    "message": "Could not match to a question. Please try again.",
                })))
            }
        }

        _ => {
            Ok(("clarification_needed".into(), serde_json::json!({
                "message": "I didn't quite catch that. Could you say it again?",
                "transcript": transcript,
            })))
        }
    }
}
