use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::middleware::auth::generate_token;
use crate::{AppState, ErrorResponse};

#[derive(Debug, Deserialize)]
pub struct DeviceAuthRequest {
    pub device_id: String,
}

#[derive(Debug, Serialize)]
pub struct DeviceAuthResponse {
    pub token: String,
    pub user: focusflow_db::models::User,
}

pub async fn device_auth(
    State(state): State<AppState>,
    Json(body): Json<DeviceAuthRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let user = match state.db.get_user_by_device_id(&body.device_id).await {
        Ok(Some(user)) => user,
        Ok(None) => state.db.create_user(&body.device_id).await.map_err(|e| {
            tracing::error!(error = %e, "failed to create user");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "failed to create user".into(),
                    details: Some(e.to_string()),
                }),
            )
        })?,
        Err(e) => {
            tracing::error!(error = %e, "failed to look up user");
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "database error".into(),
                    details: Some(e.to_string()),
                }),
            ));
        }
    };

    let token = generate_token(user.id, &body.device_id, &state.jwt_secret).map_err(|e| {
        tracing::error!(error = ?e, "failed to generate token");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "failed to generate token".into(),
                details: None,
            }),
        )
    })?;

    if let Err(e) = state.db.update_last_active(user.id).await {
        tracing::warn!(error = %e, "failed to update last_active");
    }

    Ok(Json(DeviceAuthResponse { token, user }))
}
