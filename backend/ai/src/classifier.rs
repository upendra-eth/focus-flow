use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::types::{ClassificationResult, Intent, IntentData};

const CLASSIFICATION_SYSTEM_PROMPT: &str = r#"You are an intent classifier for an ADHD productivity app. The user spoke into their phone and this is the transcript. Classify into exactly ONE intent and extract structured data.

INTENTS:
1. "create_task" — user wants to do something, add a reminder, or complete an action
2. "thought_dump" — user is venting, reflecting, journaling, or thinking out loud
3. "profile_answer" — user is answering a question the app previously asked them
4. "unclear" — cannot determine intent, need clarification

DECISION PROCESS:
- If there is a pending profiling question AND the transcript semantically matches the question topic, classify as "profile_answer".
- If the transcript contains action verbs with objects ("call", "buy", "finish", "send", "schedule") or phrases like "I need to", "I should", "don't forget", "remind me", classify as "create_task".
- If the transcript is primarily emotional or reflective (sentiment markers, "I feel", "it's hard", venting), classify as "thought_dump".
- If the transcript is 3 words or fewer and unclassifiable, classify as "unclear".

RESPOND ONLY WITH VALID JSON (no markdown, no backticks):
{
  "intent": "create_task" | "thought_dump" | "profile_answer" | "unclear",
  "confidence": 0.0-1.0,
  "data": {
    // For create_task:
    "title": "short task title",
    "due_hint": "tomorrow" | "next week" | "no date" | ISO datetime string,
    "priority_hint": "low" | "medium" | "high" | "urgent"

    // For thought_dump:
    "summary": "1-sentence summary of what the user expressed",
    "sentiment": "positive" | "neutral" | "frustrated" | "anxious"

    // For profile_answer:
    "matched_question_id": "the question UUID or null",
    "extracted_answer": "normalized answer value"

    // For unclear:
    "raw_text": "the original transcript"
  }
}"#;

#[derive(Debug, thiserror::Error)]
pub enum ClassifierError {
    #[error("Gemini API error: {0}")]
    ApiError(String),
    #[error("network error: {0}")]
    NetworkError(#[from] reqwest::Error),
    #[error("failed to parse LLM response: {0}")]
    ParseError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassificationContext {
    pub pending_question: Option<(String, String)>,
    pub recent_task_titles: Vec<String>,
    pub time_of_day: String,
}

#[derive(Debug, Deserialize)]
struct RawClassification {
    intent: String,
    confidence: f32,
    data: serde_json::Value,
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

pub struct IntentClassifier {
    api_key: String,
    client: reqwest::Client,
    model: String,
}

impl IntentClassifier {
    pub fn new(api_key: &str) -> Self {
        Self::new_with_model(api_key, "gemini-2.0-flash")
    }

    pub fn new_with_model(api_key: &str, model: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            client: reqwest::Client::new(),
            model: model.to_string(),
        }
    }

    #[instrument(skip(self, transcript, context), fields(model = %self.model))]
    pub async fn classify(
        &self,
        transcript: &str,
        context: ClassificationContext,
    ) -> Result<ClassificationResult, ClassifierError> {
        if self.api_key.trim().is_empty() {
            return Err(ClassifierError::ApiError("GEMINI_API_KEY is empty".to_string()));
        }

        let user_message = self.build_user_message(transcript, &context);

        let request = GeminiGenerateRequest {
            contents: vec![GeminiContent {
                parts: vec![GeminiPart {
                    text: format!(
                        "SYSTEM:\\n{system}\\n\\nUSER:\\n{user}",
                        system = CLASSIFICATION_SYSTEM_PROMPT,
                        user = user_message
                    ),
                }],
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
            return Err(ClassifierError::ApiError(format!(
                "status {status}: {body}"
            )));
        }

        let gemini_resp: GeminiGenerateResponse = response
            .json()
            .await
            .map_err(|e| ClassifierError::ParseError(e.to_string()))?;

        let content = gemini_resp
            .candidates
            .and_then(|c| c.into_iter().next())
            .and_then(|c| c.content)
            .and_then(|c| c.parts)
            .and_then(|p| p.into_iter().next())
            .and_then(|p| p.text)
            .ok_or_else(|| ClassifierError::ParseError("empty response from Gemini".to_string()))?;

        self.parse_response(&content, transcript)
    }

    fn build_user_message(&self, transcript: &str, context: &ClassificationContext) -> String {
        let pending_q = match &context.pending_question {
            Some((_, text)) => text.as_str(),
            None => "none",
        };

        let recent = if context.recent_task_titles.is_empty() {
            "none".to_string()
        } else {
            context.recent_task_titles.join(", ")
        };

        format!(
            "USER CONTEXT:\n\
             - Pending profiling question: \"{pending_q}\"\n\
             - Recent tasks: {recent}\n\
             - Time of day: {time}\n\n\
             TRANSCRIPT:\n\"{transcript}\"",
            time = context.time_of_day,
        )
    }

    fn parse_response(
        &self,
        content: &str,
        original_transcript: &str,
    ) -> Result<ClassificationResult, ClassifierError> {
        let raw: RawClassification = match serde_json::from_str(content) {
            Ok(r) => r,
            Err(e) => {
                tracing::warn!(
                    error = %e,
                    content = content,
                    "failed to parse LLM classification output, falling back to Unclear"
                );
                return Ok(ClassificationResult {
                    intent: Intent::Unclear,
                    confidence: 0.0,
                    data: IntentData::Unclear {
                        raw_text: original_transcript.to_string(),
                    },
                });
            }
        };

        let intent = match raw.intent.as_str() {
            "create_task" => Intent::CreateTask,
            "thought_dump" => Intent::ThoughtDump,
            "profile_answer" => Intent::ProfileAnswer,
            _ => Intent::Unclear,
        };

        let data = match &intent {
            Intent::CreateTask => IntentData::Task {
                title: raw
                    .data
                    .get("title")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Untitled task")
                    .to_string(),
                due_hint: raw
                    .data
                    .get("due_hint")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                priority_hint: raw
                    .data
                    .get("priority_hint")
                    .and_then(|v| v.as_str())
                    .unwrap_or("medium")
                    .to_string(),
            },
            Intent::ThoughtDump => IntentData::ThoughtDump {
                summary: raw
                    .data
                    .get("summary")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                sentiment: raw
                    .data
                    .get("sentiment")
                    .and_then(|v| v.as_str())
                    .unwrap_or("neutral")
                    .to_string(),
            },
            Intent::ProfileAnswer => IntentData::ProfileAnswer {
                matched_question_id: raw
                    .data
                    .get("matched_question_id")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                extracted_answer: raw
                    .data
                    .get("extracted_answer")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
            },
            Intent::Unclear => IntentData::Unclear {
                raw_text: raw
                    .data
                    .get("raw_text")
                    .and_then(|v| v.as_str())
                    .unwrap_or(original_transcript)
                    .to_string(),
            },
        };

        Ok(ClassificationResult {
            intent,
            confidence: raw.confidence.clamp(0.0, 1.0),
            data,
        })
    }
}
