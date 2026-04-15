use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::middleware::auth::AuthUser;
use crate::{AppState, ErrorResponse};

#[derive(Debug, Deserialize)]
pub struct RecordSignalsRequest {
    pub signals: Vec<SignalInput>,
}

#[derive(Debug, Deserialize)]
pub struct SignalInput {
    pub signal_type: String,
    pub payload: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct RecordSignalsResponse {
    pub recorded: usize,
}

pub async fn record_signals(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<RecordSignalsRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let mut count = 0usize;

    for signal in &body.signals {
        match state
            .signal_service
            .record(auth.user_id, &signal.signal_type, signal.payload.clone())
            .await
        {
            Ok(_) => count += 1,
            Err(e) => {
                tracing::warn!(
                    error = ?e,
                    signal_type = %signal.signal_type,
                    "failed to record signal, skipping"
                );
            }
        }
    }

    Ok(Json(RecordSignalsResponse { recorded: count }))
}
