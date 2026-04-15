use std::sync::Arc;

use chrono::{DateTime, Duration, Utc};
use rand::prelude::IndexedRandom;
use uuid::Uuid;

use focusflow_db::models::{BehavioralSignal, CreateSignal, ProfilingAnswer, ProfilingQuestion};
use focusflow_db::postgres::PostgresRepo;

#[derive(Debug, thiserror::Error)]
pub enum ProfilingError {
    #[error("database error: {0}")]
    DbError(#[from] sqlx::Error),
}

#[derive(Debug, Clone)]
pub enum ProfilingState {
    Dormant,
    Selecting,
    WaitingForAnswer {
        question_id: Uuid,
        asked_at: DateTime<Utc>,
    },
    Cooldown {
        interactions_remaining: u8,
    },
}

#[derive(Debug, Clone)]
pub enum ProfilingTrigger {
    TaskCompleted,
    SessionStart { session_count_today: u32 },
    ThoughtDumpCompleted,
    Scheduled,
    InsightViewed,
}

pub struct ProfilingEngine {
    db: Arc<PostgresRepo>,
}

impl ProfilingEngine {
    pub fn new(db: Arc<PostgresRepo>) -> Self {
        Self { db }
    }

    /// Decide whether the system should present a profiling question right now.
    pub async fn should_ask_question(
        &self,
        user_id: Uuid,
        trigger: ProfilingTrigger,
    ) -> bool {
        let fatigue = match self.calculate_fatigue(user_id).await {
            Ok(f) => f,
            Err(e) => {
                tracing::warn!(error = %e, "fatigue calculation failed; skipping question");
                return false;
            }
        };

        if fatigue > 0.7 {
            tracing::debug!(user_id = %user_id, fatigue, "fatigue too high, suppressing question");
            return false;
        }

        match trigger {
            ProfilingTrigger::TaskCompleted => true,
            ProfilingTrigger::SessionStart { session_count_today } => session_count_today >= 3,
            ProfilingTrigger::ThoughtDumpCompleted => true,
            ProfilingTrigger::Scheduled => true,
            ProfilingTrigger::InsightViewed => true,
        }
    }

    /// Pick the best next question using the scoring algorithm from the architecture.
    pub async fn select_next_question(
        &self,
        user_id: Uuid,
    ) -> Result<Option<ProfilingQuestion>, ProfilingError> {
        // 1. Gather user state
        let answered_ids = self.db.get_answered_question_ids(user_id).await?;

        let skipped_signals = self
            .db
            .get_recent_signals(user_id, Some("question_skipped"), 24 * 7)
            .await?;
        let skipped_ids: Vec<Uuid> = skipped_signals
            .iter()
            .filter_map(|s| {
                s.payload
                    .get("question_id")
                    .and_then(|v| v.as_str())
                    .and_then(|s| Uuid::parse_str(s).ok())
            })
            .collect();

        // 2. Fatigue gate
        let fatigue = self.calculate_fatigue(user_id).await?;
        if fatigue > 0.7 {
            return Ok(None);
        }

        // 3. Get all active questions, filter answered + recently skipped
        let all_questions = self.db.get_all_active_questions().await?;
        let candidates: Vec<&ProfilingQuestion> = all_questions
            .iter()
            .filter(|q| !answered_ids.contains(&q.id))
            .filter(|q| !skipped_ids.contains(&q.id))
            .filter(|q| match q.depends_on {
                Some(dep) => answered_ids.contains(&dep),
                None => true,
            })
            .collect();

        if candidates.is_empty() {
            return Ok(None);
        }

        // 4. Determine recent answer categories for diversity bonus
        let recent_answered_signals = self
            .db
            .get_recent_signals(user_id, Some("question_answered"), 24 * 7)
            .await?;
        let recent_categories: Vec<String> = recent_answered_signals
            .iter()
            .take(3)
            .filter_map(|s| {
                s.payload
                    .get("category")
                    .and_then(|v| v.as_str())
                    .map(String::from)
            })
            .collect();

        // 5. Score candidates
        let mut scored: Vec<(&ProfilingQuestion, f64)> = candidates
            .iter()
            .map(|q| {
                let mut score: f64 = 0.0;

                // Priority weight: lower priority number = ask sooner
                score += (100 - q.priority) as f64 * 0.3;

                // Category diversity bonus
                if !recent_categories.contains(&q.category) {
                    score += 0.2;
                }

                (*q, score)
            })
            .collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // 6. Weighted random choice among top 3
        let top_n = scored.iter().take(3).collect::<Vec<_>>();
        let chosen = top_n.choose(&mut rand::rng());

        Ok(chosen.map(|(q, _)| (*q).clone()))
    }

    pub async fn submit_answer(
        &self,
        user_id: Uuid,
        question_id: Uuid,
        answer_value: &str,
        raw_input: Option<&str>,
        source: &str,
        confidence: f32,
    ) -> Result<ProfilingAnswer, ProfilingError> {
        let answer = self
            .db
            .save_answer(user_id, question_id, answer_value, raw_input, source, confidence)
            .await?;

        // Record behavioral signal
        let signal = CreateSignal {
            signal_type: "question_answered".to_string(),
            payload: serde_json::json!({
                "question_id": question_id.to_string(),
                "source": source,
                "confidence": confidence,
            }),
        };
        if let Err(e) = self.db.record_signal(user_id, signal).await {
            tracing::warn!(error = %e, "failed to record question_answered signal");
        }

        tracing::info!(
            user_id = %user_id,
            question_id = %question_id,
            "profiling answer submitted"
        );

        Ok(answer)
    }

    pub async fn skip_question(
        &self,
        user_id: Uuid,
        question_id: Uuid,
    ) -> Result<(), ProfilingError> {
        let signal = CreateSignal {
            signal_type: "question_skipped".to_string(),
            payload: serde_json::json!({
                "question_id": question_id.to_string(),
            }),
        };
        self.db.record_signal(user_id, signal).await?;

        tracing::info!(
            user_id = %user_id,
            question_id = %question_id,
            "profiling question skipped"
        );

        Ok(())
    }

    /// Fatigue score: 0.0 (fresh) → 1.0 (exhausted). Above 0.7 means don't ask.
    pub async fn calculate_fatigue(&self, user_id: Uuid) -> Result<f32, ProfilingError> {
        let mut fatigue: f32 = 0.0;

        // ── Factor 1: questions asked today (answered + skipped in last 24h) ──
        let answered_today = self
            .db
            .get_recent_signals(user_id, Some("question_answered"), 24)
            .await?;
        let skipped_today = self
            .db
            .get_recent_signals(user_id, Some("question_skipped"), 24)
            .await?;
        let questions_today = answered_today.len() + skipped_today.len();

        if questions_today >= 3 {
            fatigue += 0.4;
        } else if questions_today >= 2 {
            fatigue += 0.25;
        } else if questions_today >= 1 {
            fatigue += 0.1;
        }

        // ── Factor 2: 7-day skip rate ──
        let answered_week = self
            .db
            .get_recent_signals(user_id, Some("question_answered"), 24 * 7)
            .await?;
        let skipped_week = self
            .db
            .get_recent_signals(user_id, Some("question_skipped"), 24 * 7)
            .await?;
        let total_week = answered_week.len() + skipped_week.len();
        if total_week > 0 {
            let skip_rate = skipped_week.len() as f32 / total_week as f32;
            fatigue += skip_rate * 0.3;
        }

        // ── Factor 3: session duration < 30 s → user is busy ──
        let app_opens = self
            .db
            .get_recent_signals(user_id, Some("app_open"), 24)
            .await?;
        if let Some(last_open) = app_opens.first() {
            let elapsed = Utc::now() - last_open.recorded_at;
            if elapsed < Duration::seconds(30) {
                fatigue += 0.2;
            }
        }

        // ── Factor 4: last skip within 2 hours ──
        if let Some(last_skip) = skipped_today.first() {
            let since_skip = Utc::now() - last_skip.recorded_at;
            if since_skip < Duration::hours(2) {
                fatigue += 0.2;
            }
        }

        // ── Factor 5: recent frustrated/anxious sentiment from thought entries ──
        fatigue += self.sentiment_fatigue_component(user_id, &app_opens).await;

        Ok(fatigue.min(1.0))
    }

    /// Check recent thought-entry signals for negative sentiment.
    async fn sentiment_fatigue_component(
        &self,
        user_id: Uuid,
        _recent_signals: &[BehavioralSignal],
    ) -> f32 {
        let thought_signals = self
            .db
            .get_recent_signals(user_id, Some("thought_entry"), 24)
            .await
            .unwrap_or_default();

        for s in &thought_signals {
            if let Some(sentiment) = s.payload.get("sentiment").and_then(|v| v.as_str()) {
                if sentiment == "frustrated" || sentiment == "anxious" {
                    return 0.15;
                }
            }
        }

        0.0
    }
}
