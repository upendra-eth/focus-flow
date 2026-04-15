use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;

use crate::middleware::auth::AuthUser;
use crate::{AppState, ErrorResponse};

pub async fn get_widget_state(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let widget = state
        .task_service
        .get_widget_state(auth.user_id)
        .await
        .map_err(|e| {
            tracing::error!(error = ?e, "failed to get widget state");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "failed to get widget state".into(),
                    details: Some(e.to_string()),
                }),
            )
        })?;

    Ok(Json(widget))
}
