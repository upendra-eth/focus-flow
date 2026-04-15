use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::types::{GeneratedInsight, InsightInput, MotivationalMessage};

const WEEKLY_INSIGHT_SYSTEM_PROMPT: &str = r#"You are a compassionate, evidence-informed ADHD coach analyzing a user's weekly data. Your goal is to surface patterns the user might not see themselves and provide genuinely actionable recommendations.

You will receive:
- Number of tasks completed and current streak
- Journal/thought-dump entries from the week
- Profile answers about the user's ADHD traits
- Behavioral signal summaries (app usage patterns, task completion times, etc.)

Generate a response in STRICT JSON format (no markdown, no backticks):
{
  "summary_text": "A warm, 2-3 sentence narrative summary of the week. Acknowledge wins first, then gently note challenges. Never guilt-trip. Use 'you' language.",
  "patterns": {
    "time_blindness": 0.0-1.0,
    "analysis_paralysis": 0.0-1.0,
    "emotional_flooding": 0.0-1.0,
    "task_initiation_difficulty": 0.0-1.0,
    "hyperfocus_tendency": 0.0-1.0,
    "consistency_struggle": 0.0-1.0,
    "energy_management": 0.0-1.0
  },
  "recommendations": [
    {
      "action": "One specific, small action the user can try",
      "reason": "Why this might help, connected to observed patterns",
      "difficulty": "easy" | "medium"
    }
  ]
}

RULES:
- Pattern scores are 0.0 (not observed) to 1.0 (strongly observed this week).
- Only score patterns you have evidence for; use 0.0 for no data.
- Provide 2-4 recommendations, prioritizing "easy" ones.
- Recommendations must be concrete (not "try to be more organized" but "set a 15-minute timer before starting your most dreaded task").
- ALWAYS lead with something positive, even if the week was rough.
- Never use phrases like "you should", "you need to", or "you failed to"."#;

const MOTIVATIONAL_SYSTEM_PROMPT: &str = r#"You are generating short motivational messages for someone with ADHD. These appear on their home screen widget.

RULES:
- Each message is 8-15 words maximum.
- Warm, honest tone. Not cheesy, not condescending.
- Never guilt-inducing ("You haven't done anything today" is FORBIDDEN).
- Acknowledge that simple things can be hard and that's okay.
- Match energy to their current state (streak, completions today).
- Vary the tone: some encouraging, some validating, some gently funny.

Respond in STRICT JSON format (no markdown, no backticks):
[
  {"message": "the motivational text", "tone": "encouraging" | "validating" | "playful" | "calm"}
]"#;

#[derive(Debug, thiserror::Error)]
pub enum InsightError {
    #[error("OpenAI API error: {0}")]
    ApiError(String),
    #[error("network error: {0}")]
    NetworkError(#[from] reqwest::Error),
    #[error("failed to parse LLM response: {0}")]
    ParseError(String),
}

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
}

#[derive(Debug, Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatMessageResponse,
}

#[derive(Debug, Deserialize)]
struct ChatMessageResponse {
    content: Option<String>,
}

pub struct InsightsGenerator {
    api_key: String,
    client: reqwest::Client,
    model: String,
}

impl InsightsGenerator {
    pub fn new(api_key: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            client: reqwest::Client::new(),
            model: "gpt-4o".to_string(),
        }
    }

    #[instrument(skip(self, input), fields(user_id = %input.user_id, model = %self.model))]
    pub async fn generate_weekly_insight(
        &self,
        input: InsightInput,
    ) -> Result<GeneratedInsight, InsightError> {
        let user_message = self.build_insight_user_message(&input);

        let content = self
            .chat_completion(WEEKLY_INSIGHT_SYSTEM_PROMPT, &user_message, 0.7)
            .await?;

        let parsed: serde_json::Value =
            serde_json::from_str(&content).map_err(|e| InsightError::ParseError(e.to_string()))?;

        Ok(GeneratedInsight {
            summary_text: parsed
                .get("summary_text")
                .and_then(|v| v.as_str())
                .unwrap_or("No summary available this week.")
                .to_string(),
            patterns: parsed
                .get("patterns")
                .cloned()
                .unwrap_or(serde_json::json!({})),
            recommendations: parsed
                .get("recommendations")
                .cloned()
                .unwrap_or(serde_json::json!([])),
        })
    }

    #[instrument(skip(self), fields(model = %self.model, count))]
    pub async fn generate_motivational_messages(
        &self,
        completed_today: i32,
        streak_days: i32,
        count: usize,
    ) -> Result<Vec<MotivationalMessage>, InsightError> {
        let user_message = format!(
            "Generate {count} motivational messages.\n\
             User context:\n\
             - Tasks completed today: {completed_today}\n\
             - Current streak: {streak_days} days"
        );

        let content = self
            .chat_completion(MOTIVATIONAL_SYSTEM_PROMPT, &user_message, 0.9)
            .await?;

        let messages: Vec<MotivationalMessage> =
            serde_json::from_str(&content).map_err(|e| InsightError::ParseError(e.to_string()))?;

        Ok(messages)
    }

    fn build_insight_user_message(&self, input: &InsightInput) -> String {
        let journals = if input.journal_entries.is_empty() {
            "No journal entries this week.".to_string()
        } else {
            input
                .journal_entries
                .iter()
                .enumerate()
                .map(|(i, e)| format!("{}. {}", i + 1, e))
                .collect::<Vec<_>>()
                .join("\n")
        };

        let profile = if input.profile_answers.is_empty() {
            "No profile answers available.".to_string()
        } else {
            input
                .profile_answers
                .iter()
                .map(|(q, a)| format!("Q: {q}\nA: {a}"))
                .collect::<Vec<_>>()
                .join("\n\n")
        };

        format!(
            "WEEKLY DATA:\n\
             - Tasks completed: {completed}\n\
             - Current streak: {streak} days\n\n\
             JOURNAL ENTRIES:\n{journals}\n\n\
             PROFILE ANSWERS:\n{profile}\n\n\
             BEHAVIORAL SIGNALS:\n{signals}",
            completed = input.tasks_completed,
            streak = input.streak_days,
            signals = serde_json::to_string_pretty(&input.signal_summary).unwrap_or_default(),
        )
    }

    async fn chat_completion(
        &self,
        system_prompt: &str,
        user_message: &str,
        temperature: f32,
    ) -> Result<String, InsightError> {
        let request = ChatRequest {
            model: self.model.clone(),
            messages: vec![
                ChatMessage {
                    role: "system".to_string(),
                    content: system_prompt.to_string(),
                },
                ChatMessage {
                    role: "user".to_string(),
                    content: user_message.to_string(),
                },
            ],
            temperature,
        };

        let response = self
            .client
            .post("https://api.openai.com/v1/chat/completions")
            .bearer_auth(&self.api_key)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "unknown".to_string());
            return Err(InsightError::ApiError(format!(
                "status {status}: {body}"
            )));
        }

        let chat_resp: ChatResponse = response
            .json()
            .await
            .map_err(|e| InsightError::ParseError(e.to_string()))?;

        chat_resp
            .choices
            .first()
            .and_then(|c| c.message.content.clone())
            .ok_or_else(|| InsightError::ParseError("empty response from LLM".to_string()))
    }
}
