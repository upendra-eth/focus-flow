use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub score: f32,
    pub payload: serde_json::Value,
}

#[derive(Clone)]
pub struct QdrantClient {
    url: String,
    http: reqwest::Client,
}

#[derive(thiserror::Error, Debug)]
pub enum QdrantError {
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("qdrant returned non-success status {status}: {body}")]
    Api { status: u16, body: String },
}

impl QdrantClient {
    pub fn new(url: &str) -> Self {
        Self {
            url: url.trim_end_matches('/').to_string(),
            http: reqwest::Client::new(),
        }
    }

    pub async fn upsert_embedding(
        &self,
        collection: &str,
        id: &str,
        vector: Vec<f32>,
        payload: serde_json::Value,
    ) -> Result<(), QdrantError> {
        let body = serde_json::json!({
            "points": [{
                "id": id,
                "vector": vector,
                "payload": payload,
            }]
        });

        let resp = self
            .http
            .put(format!(
                "{}/collections/{collection}/points",
                self.url
            ))
            .json(&body)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            return Err(QdrantError::Api { status, body });
        }

        Ok(())
    }

    pub async fn search(
        &self,
        collection: &str,
        vector: Vec<f32>,
        limit: usize,
    ) -> Result<Vec<SearchResult>, QdrantError> {
        let body = serde_json::json!({
            "vector": vector,
            "limit": limit,
            "with_payload": true,
        });

        let resp = self
            .http
            .post(format!(
                "{}/collections/{collection}/points/search",
                self.url
            ))
            .json(&body)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let text = resp.text().await.unwrap_or_default();
            return Err(QdrantError::Api { status, body: text });
        }

        #[derive(Deserialize)]
        struct QdrantHit {
            id: serde_json::Value,
            score: f32,
            payload: Option<serde_json::Value>,
        }

        #[derive(Deserialize)]
        struct QdrantResponse {
            result: Vec<QdrantHit>,
        }

        let data: QdrantResponse = resp.json().await?;

        let results = data
            .result
            .into_iter()
            .map(|hit| SearchResult {
                id: match hit.id {
                    serde_json::Value::String(s) => s,
                    other => other.to_string(),
                },
                score: hit.score,
                payload: hit.payload.unwrap_or(serde_json::Value::Null),
            })
            .collect();

        Ok(results)
    }
}
