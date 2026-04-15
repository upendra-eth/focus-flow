use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::types::EmbeddingResult;

#[derive(Debug, thiserror::Error)]
pub enum EmbeddingError {
    #[error("OpenAI API error: {0}")]
    ApiError(String),
    #[error("network error: {0}")]
    NetworkError(#[from] reqwest::Error),
    #[error("failed to parse response: {0}")]
    ParseError(String),
    #[error("empty input provided")]
    EmptyInput,
}

#[derive(Debug, Serialize)]
struct EmbeddingRequest {
    model: String,
    input: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct EmbeddingResponse {
    data: Vec<EmbeddingDataItem>,
    model: String,
    usage: EmbeddingUsage,
}

#[derive(Debug, Deserialize)]
struct EmbeddingDataItem {
    embedding: Vec<f32>,
}

#[derive(Debug, Deserialize)]
struct EmbeddingUsage {
    total_tokens: u32,
}

pub struct EmbeddingClient {
    api_key: String,
    client: reqwest::Client,
    model: String,
}

impl EmbeddingClient {
    pub fn new(api_key: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            client: reqwest::Client::new(),
            model: "text-embedding-3-small".to_string(),
        }
    }

    #[instrument(skip(self, text), fields(model = %self.model, text_len = text.len()))]
    pub async fn embed(&self, text: &str) -> Result<EmbeddingResult, EmbeddingError> {
        if text.is_empty() {
            return Err(EmbeddingError::EmptyInput);
        }

        let request = EmbeddingRequest {
            model: self.model.clone(),
            input: serde_json::Value::String(text.to_string()),
        };

        let resp = self.call_api(&request).await?;

        let item = resp
            .data
            .into_iter()
            .next()
            .ok_or_else(|| EmbeddingError::ParseError("no embedding returned".to_string()))?;

        Ok(EmbeddingResult {
            vector: item.embedding,
            model: resp.model,
            usage_tokens: resp.usage.total_tokens,
        })
    }

    #[instrument(skip(self, texts), fields(model = %self.model, batch_size = texts.len()))]
    pub async fn embed_batch(
        &self,
        texts: &[&str],
    ) -> Result<Vec<EmbeddingResult>, EmbeddingError> {
        if texts.is_empty() {
            return Err(EmbeddingError::EmptyInput);
        }

        let input: Vec<serde_json::Value> = texts
            .iter()
            .map(|t| serde_json::Value::String(t.to_string()))
            .collect();

        let request = EmbeddingRequest {
            model: self.model.clone(),
            input: serde_json::Value::Array(input),
        };

        let resp = self.call_api(&request).await?;
        let per_token = resp.usage.total_tokens / resp.data.len().max(1) as u32;

        Ok(resp
            .data
            .into_iter()
            .map(|item| EmbeddingResult {
                vector: item.embedding,
                model: resp.model.clone(),
                usage_tokens: per_token,
            })
            .collect())
    }

    async fn call_api(
        &self,
        request: &EmbeddingRequest,
    ) -> Result<EmbeddingResponse, EmbeddingError> {
        let response = self
            .client
            .post("https://api.openai.com/v1/embeddings")
            .bearer_auth(&self.api_key)
            .json(request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "unknown".to_string());
            return Err(EmbeddingError::ApiError(format!(
                "status {status}: {body}"
            )));
        }

        response
            .json()
            .await
            .map_err(|e| EmbeddingError::ParseError(e.to_string()))
    }
}
