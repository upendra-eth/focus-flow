use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::types::DashboardResult;

const DASHBOARD_SYSTEM_PROMPT: &str = r#"You are analyzing a user's daily life entries to generate a personal dashboard. The user has ADHD — be encouraging, highlight wins, keep it real.

You will receive a JSON array of the user's entries for the day (each with category, title, content, structured_data, tags).

Generate a response in STRICT JSON (no markdown, no backticks):
{
  "summary_text": "A warm 2-3 sentence summary of the day. Lead with positives.",
  "mood_score": 1-10 (infer from entries, null if insufficient data),
  "energy_score": 1-10 (infer from entries, null if insufficient data),
  "categories_breakdown": {"daily_note": 2, "financial": 1, ...},
  "highlights": ["highlight 1", "highlight 2", "highlight 3"],
  "financial_summary": {"total_expense": 0.0, "total_income": 0.0, "top_category": "food"} or null if no financial entries
}

RULES:
- mood_score/energy_score: use context clues from entries. If a day_recap has mood/energy, use those. Otherwise infer from sentiment.
- highlights: pick the top 3 most notable/positive things from the day. If fewer than 3, include what you have.
- financial_summary: only include if there are financial entries. Sum amounts by type.
- categories_breakdown: count entries per category.
- If there are zero entries, return a gentle "No entries recorded today" summary with null scores."#;

#[derive(Debug, thiserror::Error)]
pub enum DashboardError {
    #[error("Gemini API error: {0}")]
    ApiError(String),
    #[error("network error: {0}")]
    NetworkError(#[from] reqwest::Error),
    #[error("parse error: {0}")]
    ParseError(String),
}

#[derive(Debug, Serialize)]
struct GeminiGenerateRequest {
    contents: Vec<GeminiContent>,
    #[serde(rename = "generationConfig")]
    generation_config: GeminiGenerationConfig,
    #[serde(rename = "systemInstruction", skip_serializing_if = "Option::is_none")]
    system_instruction: Option<GeminiContent>,
}

#[derive(Debug, Serialize)]
struct GeminiGenerationConfig {
    #[serde(rename = "responseMimeType")]
    response_mime_type: String,
    temperature: f32,
}

#[derive(Debug, Serialize)]
struct GeminiContent {
    #[serde(skip_serializing_if = "Option::is_none")]
    role: Option<String>,
    parts: Vec<GeminiPart>,
}

#[derive(Debug, Serialize)]
struct GeminiPart {
    text: String,
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

pub struct DashboardGenerator {
    api_key: String,
    client: reqwest::Client,
    model: String,
}

impl DashboardGenerator {
    pub fn new(api_key: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            client: reqwest::Client::new(),
            model: "gemini-2.0-flash".to_string(),
        }
    }

    #[instrument(skip(self, entries_json), fields(model = %self.model, date = %date))]
    pub async fn generate_daily_dashboard(
        &self,
        entries_json: &serde_json::Value,
        date: &str,
    ) -> Result<DashboardResult, DashboardError> {
        if self.api_key.trim().is_empty() {
            return Err(DashboardError::ApiError(
                "GEMINI_API_KEY is empty".to_string(),
            ));
        }

        let user_message = format!(
            "Generate a daily dashboard for {date}.\n\nEntries:\n{entries}",
            entries = serde_json::to_string_pretty(entries_json).unwrap_or_default(),
        );

        let request = GeminiGenerateRequest {
            contents: vec![GeminiContent {
                role: Some("user".to_string()),
                parts: vec![GeminiPart {
                    text: user_message,
                }],
            }],
            generation_config: GeminiGenerationConfig {
                response_mime_type: "application/json".to_string(),
                temperature: 0.5,
            },
            system_instruction: Some(GeminiContent {
                role: None,
                parts: vec![GeminiPart {
                    text: DASHBOARD_SYSTEM_PROMPT.to_string(),
                }],
            }),
        };

        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.model, self.api_key
        );

        let response = self.client.post(&url).json(&request).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "unknown".to_string());
            return Err(DashboardError::ApiError(format!(
                "status {status}: {body}"
            )));
        }

        let gemini_resp: GeminiGenerateResponse = response
            .json()
            .await
            .map_err(|e| DashboardError::ParseError(e.to_string()))?;

        let text = gemini_resp
            .candidates
            .and_then(|c| c.into_iter().next())
            .and_then(|c| c.content)
            .and_then(|c| c.parts)
            .and_then(|p| p.into_iter().next())
            .and_then(|p| p.text)
            .ok_or_else(|| {
                DashboardError::ParseError("empty response from Gemini".to_string())
            })?;

        let parsed: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| DashboardError::ParseError(e.to_string()))?;

        Ok(DashboardResult {
            summary_text: parsed
                .get("summary_text")
                .and_then(|v| v.as_str())
                .unwrap_or("No summary available.")
                .to_string(),
            mood_score: parsed
                .get("mood_score")
                .and_then(|v| v.as_i64())
                .map(|v| v as i32),
            energy_score: parsed
                .get("energy_score")
                .and_then(|v| v.as_i64())
                .map(|v| v as i32),
            categories_breakdown: parsed
                .get("categories_breakdown")
                .cloned()
                .unwrap_or(serde_json::json!({})),
            highlights: parsed
                .get("highlights")
                .cloned()
                .unwrap_or(serde_json::json!([])),
            financial_summary: parsed.get("financial_summary").cloned(),
        })
    }
}
