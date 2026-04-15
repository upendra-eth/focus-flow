use serde::{Deserialize, Serialize};
use tracing::instrument;

const AGENT_SYSTEM_PROMPT: &str = r#"You are a personal AI life assistant for someone with ADHD. The user will share thoughts, daily events, expenses, ideas, or just vent. Your job:

1. RESPOND conversationally — be warm, concise, never preachy. Match the user's energy.
2. CATEGORIZE what they shared into one or more life entries. Categories:
   - daily_note: general thoughts
   - mind_dump: unstructured venting/brain dump
   - day_recap: what they did today
   - financial: money-related (extract amount, type: expense/income, subcategory)
   - health: exercise, sleep, food, medication
   - idea: creative ideas or project concepts
   - learning: things learned or read
   - relationship: social interactions
   - goal: aspirations, plans
   - gratitude: things they're thankful for
3. EXTRACT structured data per category (see JSON schema below).
4. If images are attached, describe what you see and incorporate into your response.
5. If you need current information (prices, news, facts), use web search.

RESPOND IN STRICT JSON (no markdown):
{
  "reply": "your conversational response",
  "entries": [
    {
      "category": "...",
      "title": "short title",
      "content": "entry content",
      "structured_data": { ... category-specific fields ... },
      "tags": ["tag1", "tag2"]
    }
  ],
  "web_search_used": false
}

Structured data examples:
- financial: {"amount": 45.50, "type": "expense", "subcategory": "food", "description": "lunch"}
- health: {"type": "exercise", "activity": "walking", "duration_min": 30}
- day_recap: {"mood": "good", "energy": 7, "highlights": ["finished report", "called mom"]}
- For others: {} is fine if no specific structure applies.

If the user just says hi or something casual, reply normally with entries: []."#;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMessage {
    pub role: String,
    pub text: String,
    pub image_base64: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResponse {
    pub reply: String,
    pub entries: Vec<ExtractedEntry>,
    pub web_search_used: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedEntry {
    pub category: String,
    pub title: String,
    pub content: String,
    pub structured_data: serde_json::Value,
    pub tags: Vec<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum AgentError {
    #[error("Gemini API error: {0}")]
    ApiError(String),
    #[error("network error: {0}")]
    NetworkError(#[from] reqwest::Error),
    #[error("parse error: {0}")]
    ParseError(String),
}

// --- Gemini request/response structs ---

#[derive(Debug, Serialize)]
struct GeminiGenerateRequest {
    contents: Vec<GeminiContent>,
    #[serde(rename = "generationConfig")]
    generation_config: GeminiGenerationConfig,
    #[serde(rename = "systemInstruction", skip_serializing_if = "Option::is_none")]
    system_instruction: Option<GeminiContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<serde_json::Value>>,
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

pub struct AgentChat {
    api_key: String,
    client: reqwest::Client,
    model: String,
}

impl AgentChat {
    pub fn new(api_key: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            client: reqwest::Client::new(),
            model: "gemini-2.0-flash".to_string(),
        }
    }

    /// Multi-turn chat with optional image attachments and web search grounding.
    #[instrument(skip(self, history, user_text, images), fields(model = %self.model, history_len = history.len(), enable_search))]
    pub async fn chat(
        &self,
        history: &[AgentMessage],
        user_text: &str,
        images: &[(&str, &str)], // (mime_type, base64_data)
        enable_search: bool,
    ) -> Result<AgentResponse, AgentError> {
        if self.api_key.trim().is_empty() {
            return Err(AgentError::ApiError("GEMINI_API_KEY is empty".to_string()));
        }

        let mut contents: Vec<GeminiContent> = Vec::new();

        // Replay conversation history as alternating user/model turns
        for msg in history {
            let role = match msg.role.as_str() {
                "model" => "model",
                _ => "user",
            };

            let mut parts = vec![GeminiPart::Text {
                text: msg.text.clone(),
            }];

            if let Some(ref imgs) = msg.image_base64 {
                for img_b64 in imgs {
                    parts.push(GeminiPart::InlineData {
                        inline_data: GeminiInlineData {
                            mime_type: "image/jpeg".to_string(),
                            data: img_b64.clone(),
                        },
                    });
                }
            }

            contents.push(GeminiContent {
                role: Some(role.to_string()),
                parts,
            });
        }

        // Build current user message parts
        let mut user_parts = vec![GeminiPart::Text {
            text: user_text.to_string(),
        }];

        for (mime_type, b64_data) in images {
            user_parts.push(GeminiPart::InlineData {
                inline_data: GeminiInlineData {
                    mime_type: mime_type.to_string(),
                    data: b64_data.to_string(),
                },
            });
        }

        contents.push(GeminiContent {
            role: Some("user".to_string()),
            parts: user_parts,
        });

        let tools = if enable_search {
            Some(vec![serde_json::json!({"googleSearchRetrieval": {}})])
        } else {
            None
        };

        let request = GeminiGenerateRequest {
            contents,
            generation_config: GeminiGenerationConfig {
                response_mime_type: "application/json".to_string(),
                temperature: 0.7,
            },
            system_instruction: Some(GeminiContent {
                role: None,
                parts: vec![GeminiPart::Text {
                    text: AGENT_SYSTEM_PROMPT.to_string(),
                }],
            }),
            tools,
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
            return Err(AgentError::ApiError(format!("status {status}: {body}")));
        }

        let gemini_resp: GeminiGenerateResponse = response
            .json()
            .await
            .map_err(|e| AgentError::ParseError(e.to_string()))?;

        let text = gemini_resp
            .candidates
            .and_then(|c| c.into_iter().next())
            .and_then(|c| c.content)
            .and_then(|c| c.parts)
            .and_then(|p| p.into_iter().next())
            .and_then(|p| p.text)
            .ok_or_else(|| AgentError::ParseError("empty response from Gemini".to_string()))?;

        match serde_json::from_str::<AgentResponse>(&text) {
            Ok(parsed) => Ok(parsed),
            Err(e) => {
                tracing::warn!(
                    error = %e,
                    raw = %text,
                    "failed to parse agent JSON, returning raw text as reply"
                );
                Ok(AgentResponse {
                    reply: text,
                    entries: vec![],
                    web_search_used: enable_search,
                })
            }
        }
    }
}
