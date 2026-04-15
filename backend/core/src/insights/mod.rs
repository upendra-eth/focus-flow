use std::sync::Arc;

use chrono::{Datelike, Utc};
use uuid::Uuid;

use focusflow_ai::insights_generator::InsightsGenerator;
use focusflow_ai::types::InsightInput;
use focusflow_db::models::WeeklyInsight;
use focusflow_db::postgres::PostgresRepo;

#[derive(Debug, thiserror::Error)]
pub enum InsightError {
    #[error("database error: {0}")]
    DbError(#[from] sqlx::Error),
    #[error("AI generation error: {0}")]
    AiError(#[from] focusflow_ai::insights_generator::InsightError),
}

pub struct InsightService {
    db: Arc<PostgresRepo>,
    ai: Arc<InsightsGenerator>,
}

impl InsightService {
    pub fn new(db: Arc<PostgresRepo>, ai: Arc<InsightsGenerator>) -> Self {
        Self { db, ai }
    }

    pub async fn generate_weekly_insight(
        &self,
        user_id: Uuid,
    ) -> Result<WeeklyInsight, InsightError> {
        let tasks_completed = self.db.get_completed_today(user_id).await? as i32;
        let streak_days = self.db.get_streak_days(user_id).await?;

        // Gather journal entries from the past 7 days
        let thought_signals = self
            .db
            .get_recent_signals(user_id, Some("thought_entry"), 24 * 7)
            .await?;
        let journal_entries: Vec<String> = thought_signals
            .iter()
            .filter_map(|s| {
                s.payload
                    .get("summary")
                    .and_then(|v| v.as_str())
                    .map(String::from)
            })
            .collect();

        // Gather profile answers (question_text, answer_value) from answered signals
        let answer_signals = self
            .db
            .get_recent_signals(user_id, Some("question_answered"), 24 * 7)
            .await?;
        let profile_answers: Vec<(String, String)> = answer_signals
            .iter()
            .filter_map(|s| {
                let q = s.payload.get("question_text")?.as_str()?;
                let a = s.payload.get("answer_value")?.as_str()?;
                Some((q.to_string(), a.to_string()))
            })
            .collect();

        // Behavioral signals summary
        let all_signals = self
            .db
            .get_recent_signals(user_id, None, 24 * 7)
            .await?;
        let signal_summary = summarize_signals(&all_signals);

        let input = InsightInput {
            user_id: user_id.to_string(),
            tasks_completed,
            streak_days,
            journal_entries,
            profile_answers,
            signal_summary,
        };

        let generated = self.ai.generate_weekly_insight(input).await?;

        let today = Utc::now().date_naive();
        let week_start = today - chrono::Duration::days(today.weekday().num_days_from_monday() as i64);

        let insight = WeeklyInsight {
            id: Uuid::new_v4(),
            user_id,
            week_start,
            summary_text: generated.summary_text,
            patterns: generated.patterns,
            recommendations: generated.recommendations,
            tasks_completed,
            streak_days,
            generated_at: Utc::now(),
        };

        let saved = self.db.save_insight(&insight).await?;

        tracing::info!(user_id = %user_id, week_start = %week_start, "weekly insight generated");
        Ok(saved)
    }

    pub async fn get_latest_insight(
        &self,
        user_id: Uuid,
    ) -> Result<Option<WeeklyInsight>, InsightError> {
        let insight = self.db.get_latest_insight(user_id).await?;
        Ok(insight)
    }
}

fn summarize_signals(signals: &[focusflow_db::models::BehavioralSignal]) -> serde_json::Value {
    let mut counts = std::collections::HashMap::<&str, u64>::new();
    for s in signals {
        *counts.entry(s.signal_type.as_str()).or_default() += 1;
    }

    serde_json::json!({
        "total_signals": signals.len(),
        "counts": counts,
    })
}
