use reqwest::multipart;
use serde::Deserialize;
use tracing::instrument;

use crate::types::TranscriptionResult;

#[derive(Debug, thiserror::Error)]
pub enum WhisperError {
    #[error("OpenAI API error: {0}")]
    ApiError(String),
    #[error("network error: {0}")]
    NetworkError(#[from] reqwest::Error),
    #[error("failed to parse response: {0}")]
    ParseError(String),
}

#[derive(Debug, Deserialize)]
struct WhisperResponse {
    text: String,
    language: Option<String>,
    duration: Option<f64>,
}

pub struct WhisperClient {
    api_key: String,
    client: reqwest::Client,
    model: String,
}

impl WhisperClient {
    pub fn new(api_key: &str) -> Self {
        Self::new_with_model(api_key, "whisper-1")
    }

    pub fn new_with_model(api_key: &str, model: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            client: reqwest::Client::new(),
            model: model.to_string(),
        }
    }

    #[instrument(skip(self, audio_data), fields(filename = %filename, model = %self.model))]
    pub async fn transcribe(
        &self,
        audio_data: Vec<u8>,
        filename: &str,
    ) -> Result<TranscriptionResult, WhisperError> {
        let file_part = multipart::Part::bytes(audio_data)
            .file_name(filename.to_string())
            .mime_str("audio/mpeg")
            .map_err(|e| WhisperError::ParseError(e.to_string()))?;

        let form = multipart::Form::new()
            .part("file", file_part)
            .text("model", self.model.clone())
            .text("response_format", "verbose_json");

        let response = self
            .client
            .post("https://api.openai.com/v1/audio/transcriptions")
            .bearer_auth(&self.api_key)
            .multipart(form)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "unknown".to_string());
            return Err(WhisperError::ApiError(format!(
                "status {status}: {body}"
            )));
        }

        let whisper_resp: WhisperResponse = response
            .json()
            .await
            .map_err(|e| WhisperError::ParseError(e.to_string()))?;

        Ok(TranscriptionResult {
            text: whisper_resp.text,
            language: whisper_resp.language,
            duration_seconds: whisper_resp.duration.unwrap_or(0.0) as f32,
        })
    }
}
