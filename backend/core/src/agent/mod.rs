use std::sync::Arc;

use base64::Engine;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use focusflow_ai::agent::{AgentChat, AgentMessage, AgentResponse};
use focusflow_ai::dashboard_generator::DashboardGenerator;
use focusflow_db::models::{DailyDashboard, LifeEntry};
use focusflow_db::postgres::PostgresRepo;

#[derive(Debug, thiserror::Error)]
pub enum AgentServiceError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("agent error: {0}")]
    Agent(#[from] focusflow_ai::agent::AgentError),
    #[error("dashboard error: {0}")]
    Dashboard(#[from] focusflow_ai::dashboard_generator::DashboardError),
    #[error("not found: {0}")]
    NotFound(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResult {
    pub conversation_id: Uuid,
    pub agent_response: AgentResponse,
    pub saved_entries: Vec<LifeEntry>,
}

pub struct AgentService {
    db: Arc<PostgresRepo>,
    agent: Arc<AgentChat>,
    dashboard_gen: Arc<DashboardGenerator>,
}

impl AgentService {
    pub fn new(
        db: Arc<PostgresRepo>,
        agent: Arc<AgentChat>,
        dashboard_gen: Arc<DashboardGenerator>,
    ) -> Self {
        Self {
            db,
            agent,
            dashboard_gen,
        }
    }

    /// Send a chat message. Creates a new conversation if conversation_id is None.
    /// Images are tuples of (filename, mime_type, raw_bytes).
    pub async fn chat(
        &self,
        user_id: Uuid,
        conversation_id: Option<Uuid>,
        text: &str,
        images: Vec<(String, String, Vec<u8>)>,
    ) -> Result<ChatResult, AgentServiceError> {
        let conversation = match conversation_id {
            Some(id) => {
                self.db
                    .get_conversation(id)
                    .await?
                    .ok_or_else(|| AgentServiceError::NotFound(format!("conversation {id}")))?
            }
            None => {
                let title = text.chars().take(80).collect::<String>();
                self.db
                    .create_conversation(user_id, Some(&title))
                    .await?
            }
        };

        // Load recent messages for context
        let recent_messages = self
            .db
            .get_conversation_messages(conversation.id, 20)
            .await?;

        let history: Vec<AgentMessage> = recent_messages
            .iter()
            .map(|m| AgentMessage {
                role: m.role.clone(),
                text: m.content.clone(),
                image_base64: None,
            })
            .collect();

        // Base64-encode images and collect mime types
        let image_pairs: Vec<(String, String)> = images
            .iter()
            .map(|(_filename, mime, bytes)| {
                let b64 = base64::engine::general_purpose::STANDARD.encode(bytes);
                (mime.clone(), b64)
            })
            .collect();

        let image_refs: Vec<(&str, &str)> = image_pairs
            .iter()
            .map(|(m, d)| (m.as_str(), d.as_str()))
            .collect();

        let enable_search = should_enable_search(text);

        let agent_response = self
            .agent
            .chat(&history, text, &image_refs, enable_search)
            .await?;

        // Save user message
        let image_url_strings: Option<Vec<String>> = if images.is_empty() {
            None
        } else {
            Some(images.iter().map(|(f, _, _)| f.clone()).collect())
        };

        let user_msg = self
            .db
            .save_message(
                conversation.id,
                "user",
                text,
                image_url_strings.as_deref(),
                serde_json::json!({}),
            )
            .await?;

        // Save assistant response
        self.db
            .save_message(
                conversation.id,
                "model",
                &agent_response.reply,
                None,
                serde_json::json!({
                    "web_search_used": agent_response.web_search_used,
                    "entries_count": agent_response.entries.len(),
                }),
            )
            .await?;

        // Save extracted entries
        let today = chrono::Utc::now().date_naive();
        let mut saved_entries = Vec::new();

        for entry in &agent_response.entries {
            let saved = self
                .db
                .save_life_entry(
                    user_id,
                    &entry.category,
                    &entry.title,
                    &entry.content,
                    Some(entry.structured_data.clone()),
                    Some(user_msg.id),
                    None,
                    &entry.tags,
                    today,
                )
                .await?;
            saved_entries.push(saved);
        }

        Ok(ChatResult {
            conversation_id: conversation.id,
            agent_response,
            saved_entries,
        })
    }

    pub async fn get_entries(
        &self,
        user_id: Uuid,
        date: Option<NaiveDate>,
        category: Option<&str>,
        limit: i64,
    ) -> Result<Vec<LifeEntry>, AgentServiceError> {
        Ok(self.db.get_life_entries(user_id, date, category, limit).await?)
    }

    pub async fn get_or_generate_dashboard(
        &self,
        user_id: Uuid,
        date: NaiveDate,
    ) -> Result<DailyDashboard, AgentServiceError> {
        if let Some(existing) = self.db.get_dashboard(user_id, date).await? {
            return Ok(existing);
        }

        let entries = self
            .db
            .get_life_entries(user_id, Some(date), None, 200)
            .await?;

        let entries_json: serde_json::Value = serde_json::to_value(&entries)
            .unwrap_or(serde_json::json!([]));

        let date_str = date.to_string();
        let result = self
            .dashboard_gen
            .generate_daily_dashboard(&entries_json, &date_str)
            .await?;

        let dashboard = self
            .db
            .save_dashboard(
                user_id,
                date,
                &result.summary_text,
                result.mood_score,
                result.energy_score,
                result.categories_breakdown,
                result.highlights,
                result.financial_summary,
            )
            .await?;

        Ok(dashboard)
    }
}

fn should_enable_search(text: &str) -> bool {
    let lower = text.to_lowercase();
    let triggers = [
        "what is",
        "how much",
        "search for",
        "look up",
        "find out",
        "current price",
        "latest news",
        "what's the",
        "who is",
        "when is",
        "where is",
        "google",
    ];
    triggers.iter().any(|t| lower.contains(t))
}
