use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub device_id: String,
    pub email: Option<String>,
    pub display_name: Option<String>,
    pub timezone: String,
    pub created_at: DateTime<Utc>,
    pub last_active_at: DateTime<Utc>,
    pub onboarding_done: bool,
    pub settings: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Task {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub priority: String,
    pub source: String,
    pub due_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub tags: Vec<String>,
    pub ai_metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ProfilingQuestion {
    pub id: Uuid,
    pub category: String,
    pub question_text: String,
    pub question_type: String,
    pub options: Option<serde_json::Value>,
    pub priority: i32,
    pub depends_on: Option<Uuid>,
    pub skip_if: Option<serde_json::Value>,
    pub tags: Vec<String>,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ProfilingAnswer {
    pub id: Uuid,
    pub user_id: Uuid,
    pub question_id: Uuid,
    pub answer_value: String,
    pub raw_input: Option<String>,
    pub source: String,
    pub confidence: f32,
    pub answered_at: DateTime<Utc>,
    pub session_context: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct BehavioralSignal {
    pub id: Uuid,
    pub user_id: Uuid,
    pub signal_type: String,
    pub payload: serde_json::Value,
    pub recorded_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct WeeklyInsight {
    pub id: Uuid,
    pub user_id: Uuid,
    pub week_start: NaiveDate,
    pub summary_text: String,
    pub patterns: serde_json::Value,
    pub recommendations: serde_json::Value,
    pub tasks_completed: i32,
    pub streak_days: i32,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ThoughtEntry {
    pub id: Uuid,
    pub user_id: Uuid,
    pub raw_transcript: String,
    pub processed_text: Option<String>,
    pub sentiment: Option<String>,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetState {
    pub completed_today: i32,
    pub streak_days: i32,
    pub motivational_message: String,
    pub last_completed_task_title: Option<String>,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTask {
    pub title: String,
    pub description: Option<String>,
    pub priority: String,
    pub source: String,
    pub due_at: Option<DateTime<Utc>>,
    pub tags: Vec<String>,
    pub ai_metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSignal {
    pub signal_type: String,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Conversation {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ConversationMessage {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub role: String,
    pub content: String,
    pub image_urls: Option<Vec<String>>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct LifeEntry {
    pub id: Uuid,
    pub user_id: Uuid,
    pub category: String,
    pub title: String,
    pub content: String,
    pub structured_data: Option<serde_json::Value>,
    pub source_message_id: Option<Uuid>,
    pub image_urls: Option<Vec<String>>,
    pub tags: Vec<String>,
    pub entry_date: chrono::NaiveDate,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DailyDashboard {
    pub id: Uuid,
    pub user_id: Uuid,
    pub dashboard_date: chrono::NaiveDate,
    pub summary_text: String,
    pub mood_score: Option<i32>,
    pub energy_score: Option<i32>,
    pub categories_breakdown: serde_json::Value,
    pub highlights: serde_json::Value,
    pub financial_summary: Option<serde_json::Value>,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ImageUpload {
    pub id: Uuid,
    pub user_id: Uuid,
    pub filename: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub storage_path: String,
    pub ai_description: Option<String>,
    pub created_at: DateTime<Utc>,
}
