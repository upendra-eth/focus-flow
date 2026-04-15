use std::sync::Arc;

use uuid::Uuid;

use focusflow_db::models::{BehavioralSignal, CreateSignal};
use focusflow_db::postgres::PostgresRepo;

#[derive(Debug, thiserror::Error)]
pub enum SignalError {
    #[error("database error: {0}")]
    DbError(#[from] sqlx::Error),
}

pub struct SignalService {
    db: Arc<PostgresRepo>,
}

impl SignalService {
    pub fn new(db: Arc<PostgresRepo>) -> Self {
        Self { db }
    }

    pub async fn record(
        &self,
        user_id: Uuid,
        signal_type: &str,
        payload: serde_json::Value,
    ) -> Result<BehavioralSignal, SignalError> {
        let signal = CreateSignal {
            signal_type: signal_type.to_string(),
            payload,
        };

        let saved = self.db.record_signal(user_id, signal).await?;
        tracing::info!(user_id = %user_id, signal_type, "behavioral signal recorded");
        Ok(saved)
    }

    pub async fn get_recent(
        &self,
        user_id: Uuid,
        signal_type: Option<&str>,
        hours: i64,
    ) -> Result<Vec<BehavioralSignal>, SignalError> {
        let signals = self.db.get_recent_signals(user_id, signal_type, hours).await?;
        Ok(signals)
    }

    pub async fn count_signals_in_window(
        &self,
        user_id: Uuid,
        signal_type: &str,
        hours: i64,
    ) -> Result<i64, SignalError> {
        let signals = self
            .db
            .get_recent_signals(user_id, Some(signal_type), hours)
            .await?;
        Ok(signals.len() as i64)
    }
}
