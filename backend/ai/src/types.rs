use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Intent {
    CreateTask,
    ThoughtDump,
    ProfileAnswer,
    Unclear,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassificationResult {
    pub intent: Intent,
    pub confidence: f32,
    pub data: IntentData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum IntentData {
    Task {
        title: String,
        due_hint: Option<String>,
        priority_hint: String,
    },
    ThoughtDump {
        summary: String,
        sentiment: String,
    },
    ProfileAnswer {
        matched_question_id: Option<String>,
        extracted_answer: String,
    },
    Unclear {
        raw_text: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionResult {
    pub text: String,
    pub language: Option<String>,
    pub duration_seconds: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingResult {
    pub vector: Vec<f32>,
    pub model: String,
    pub usage_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsightInput {
    pub user_id: String,
    pub tasks_completed: i32,
    pub streak_days: i32,
    pub journal_entries: Vec<String>,
    pub profile_answers: Vec<(String, String)>,
    pub signal_summary: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedInsight {
    pub summary_text: String,
    pub patterns: serde_json::Value,
    pub recommendations: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MotivationalMessage {
    pub message: String,
    pub tone: String,
}
