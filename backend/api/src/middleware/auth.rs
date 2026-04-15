use async_trait::async_trait;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{AppState, ErrorResponse};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub device_id: String,
    pub exp: usize,
}

pub fn generate_token(user_id: Uuid, device_id: &str, secret: &str) -> Result<String, AuthError> {
    let exp = (Utc::now() + Duration::days(30)).timestamp() as usize;
    let claims = Claims {
        sub: user_id.to_string(),
        device_id: device_id.to_string(),
        exp,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AuthError::TokenCreation(e.to_string()))
}

pub fn validate_token(token: &str, secret: &str) -> Result<Claims, AuthError> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|e| AuthError::InvalidToken(e.to_string()))?;
    Ok(token_data.claims)
}

#[derive(Debug)]
pub struct AuthUser {
    pub user_id: Uuid,
    pub device_id: String,
}

#[derive(Debug)]
pub enum AuthError {
    MissingHeader,
    InvalidToken(String),
    TokenCreation(String),
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, msg) = match &self {
            AuthError::MissingHeader => (StatusCode::UNAUTHORIZED, "missing authorization header"),
            AuthError::InvalidToken(_) => (StatusCode::UNAUTHORIZED, "invalid or expired token"),
            AuthError::TokenCreation(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "failed to create token")
            }
        };
        let body = Json(ErrorResponse {
            error: msg.to_string(),
            details: match self {
                AuthError::InvalidToken(d) | AuthError::TokenCreation(d) => Some(d),
                _ => None,
            },
        });
        (status, body).into_response()
    }
}

#[async_trait]
impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let header_val = parts
            .headers
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .ok_or(AuthError::MissingHeader)?;

        let token = header_val
            .strip_prefix("Bearer ")
            .ok_or(AuthError::MissingHeader)?;

        let claims = validate_token(token, &state.jwt_secret)?;

        let user_id = Uuid::parse_str(&claims.sub)
            .map_err(|e| AuthError::InvalidToken(e.to_string()))?;

        Ok(AuthUser {
            user_id,
            device_id: claims.device_id,
        })
    }
}
