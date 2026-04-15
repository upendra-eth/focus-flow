use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;

use crate::middleware::auth::AuthUser;
use crate::{AppState, ErrorResponse};

pub async fn latest_insight(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let insight = state
        .insight_service
        .get_latest_insight(auth.user_id)
        .await
        .map_err(|e| {
            tracing::error!(error = ?e, "failed to get latest insight");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "failed to get insight".into(),
                    details: Some(e.to_string()),
                }),
            )
        })?;

    Ok(Json(insight))
}
