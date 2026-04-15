use base64::Engine;
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::types::TranscriptionResult;

#[derive(Debug, thiserror::Error)]
pub enum WhisperError {
    #[error("Gemini API error: {0}")]
    ApiError(String),
    #[error("network error: {0}")]
    NetworkError(#[from] reqwest::Error),
    #[error("failed to parse response: {0}")]
    ParseError(String),
}

#[derive(Debug, Serialize)]
struct GeminiGenerateRequest {
    contents: Vec<GeminiContent>,
    #[serde(rename = "generationConfig")]
    generation_config: GeminiGenerationConfig,
}

#[derive(Debug, Serialize)]
struct GeminiGenerationConfig {
    #[serde(rename = "responseMimeType")]
    response_mime_type: String,
    temperature: f32,
}

#[derive(Debug, Serialize)]
struct GeminiContent {
    parts: Vec<GeminiPart>,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
enum GeminiPart {
    Text { text: String },
    InlineData { inline_data: GeminiInlineData },
}

#[derive(Debug, Serialize)]
struct GeminiInlineData {
    #[serde(rename = "mimeType")]
    mime_type: String,
    data: String,
}

#[derive(Debug, Deserialize)]
struct GeminiGenerateResponse {
    candidates: Option<Vec<GeminiCandidate>>,
}

#[derive(Debug, Deserialize)]
struct GeminiCandidate {
    content: Option<GeminiCandidateContent>,
}

#[derive(Debug, Deserialize)]
struct GeminiCandidateContent {
    parts: Option<Vec<GeminiCandidatePart>>,
}

#[derive(Debug, Deserialize)]
struct GeminiCandidatePart {
    text: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GeminiTranscriptionJson {
    transcript: String,
    language: Option<String>,
    duration_seconds: Option<f32>,
}

pub struct WhisperClient {
    api_key: String,
    client: reqwest::Client,
    model: String,
}

impl WhisperClient {
    pub fn new(api_key: &str) -> Self {
        Self::new_with_model(api_key, "gemini-2.5-flash")
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
        if self.api_key.trim().is_empty() {
            return Err(WhisperError::ApiError(
                "GEMINI_API_KEY is empty".to_string(),
            ));
        }

        // Gemini is multimodal; we send audio as inline data and request strict JSON back.
        // We use `audio/mpeg` as a generic audio mime type; Android will often send m4a/mp4.
        let mime_type = if filename.ends_with(".m4a") || filename.ends_with(".mp4") {
            "audio/mp4"
        } else if filename.ends_with(".wav") {
            "audio/wav"
        } else {
            "audio/mpeg"
        };

        let audio_b64 = base64::engine::general_purpose::STANDARD.encode(audio_data);

        let prompt = "Transcribe the user's audio. Respond ONLY with valid JSON (no markdown): {\"transcript\":\"...\",\"language\":null|\"en\",\"duration_seconds\":number|null}. Keep transcript as plain text.";

        let request = GeminiGenerateRequest {
            contents: vec![GeminiContent {
                parts: vec![
                    GeminiPart::Text {
                        text: prompt.to_string(),
                    },
                    GeminiPart::InlineData {
                        inline_data: GeminiInlineData {
                            mime_type: mime_type.to_string(),
                            data: audio_b64,
                        },
                    },
                ],
            }],
            generation_config: GeminiGenerationConfig {
                response_mime_type: "application/json".to_string(),
                temperature: 0.1,
            },
        };

        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.model, self.api_key
        );

        let response = self.client.post(url).json(&request).send().await?;

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

        let resp: GeminiGenerateResponse = response
            .json()
            .await
            .map_err(|e| WhisperError::ParseError(e.to_string()))?;

        let text = resp
            .candidates
            .and_then(|c| c.into_iter().next())
            .and_then(|c| c.content)
            .and_then(|c| c.parts)
            .and_then(|p| p.into_iter().next())
            .and_then(|p| p.text)
            .ok_or_else(|| WhisperError::ParseError("empty response from Gemini".to_string()))?;

        let parsed: GeminiTranscriptionJson =
            serde_json::from_str(&text).map_err(|e| WhisperError::ParseError(e.to_string()))?;

        Ok(TranscriptionResult {
            text: parsed.transcript,
            language: parsed.language,
            duration_seconds: parsed.duration_seconds.unwrap_or(0.0),
        })
    }
}
