use std::sync::Arc;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use focusflow_db::models::{CreateSignal, CreateTask, Task, WidgetState};
use focusflow_db::postgres::PostgresRepo;
use focusflow_db::redis_cache::{CacheError, RedisCache};

#[derive(Debug, thiserror::Error)]
pub enum TaskError {
    #[error("database error: {0}")]
    DbError(#[from] sqlx::Error),
    #[error("cache error: {0}")]
    CacheError(#[from] CacheError),
    #[error("task not found")]
    NotFound,
}

pub struct TaskService {
    db: Arc<PostgresRepo>,
    cache: Arc<RedisCache>,
}

impl TaskService {
    pub fn new(db: Arc<PostgresRepo>, cache: Arc<RedisCache>) -> Self {
        Self { db, cache }
    }

    pub async fn create_task(
        &self,
        user_id: Uuid,
        title: String,
        description: Option<String>,
        priority: String,
        source: String,
        due_at: Option<DateTime<Utc>>,
        tags: Vec<String>,
        ai_metadata: Option<serde_json::Value>,
    ) -> Result<Task, TaskError> {
        let create = CreateTask {
            title,
            description,
            priority,
            source,
            due_at,
            tags,
            ai_metadata,
        };

        let task = self.db.create_task(user_id, create).await?;

        let signal = CreateSignal {
            signal_type: "task_created".to_string(),
            payload: serde_json::json!({ "task_id": task.id }),
        };
        if let Err(e) = self.db.record_signal(user_id, signal).await {
            tracing::warn!(error = %e, "failed to record task_created signal");
        }

        if let Err(e) = self.cache.invalidate_widget(user_id).await {
            tracing::warn!(error = %e, "failed to invalidate widget cache");
        }

        tracing::info!(user_id = %user_id, task_id = %task.id, "task created");
        Ok(task)
    }

    pub async fn complete_task(
        &self,
        user_id: Uuid,
        task_id: Uuid,
    ) -> Result<Task, TaskError> {
        let task = self
            .db
            .update_task_status(task_id, "completed")
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => TaskError::NotFound,
                other => TaskError::DbError(other),
            })?;

        let signal = CreateSignal {
            signal_type: "task_completed".to_string(),
            payload: serde_json::json!({ "task_id": task_id }),
        };
        if let Err(e) = self.db.record_signal(user_id, signal).await {
            tracing::warn!(error = %e, "failed to record task_completed signal");
        }

        // Eagerly update widget cache with fresh data
        match self.compute_widget_state(user_id, Some(&task.title)).await {
            Ok(state) => {
                if let Err(e) = self.cache.set_widget_state(user_id, &state).await {
                    tracing::warn!(error = %e, "failed to update widget cache after completion");
                }
            }
            Err(e) => {
                tracing::warn!(error = %e, "failed to compute widget state; invalidating cache");
                let _ = self.cache.invalidate_widget(user_id).await;
            }
        }

        tracing::info!(user_id = %user_id, task_id = %task_id, "task completed");
        Ok(task)
    }

    pub async fn get_user_tasks(
        &self,
        user_id: Uuid,
        status: Option<&str>,
    ) -> Result<Vec<Task>, TaskError> {
        let tasks = self.db.get_tasks(user_id, status).await?;
        Ok(tasks)
    }

    pub async fn get_widget_state(&self, user_id: Uuid) -> Result<WidgetState, TaskError> {
        if let Some(cached) = self.cache.get_widget_state(user_id).await? {
            return Ok(cached);
        }

        let state = self.compute_widget_state(user_id, None).await?;

        if let Err(e) = self.cache.set_widget_state(user_id, &state).await {
            tracing::warn!(error = %e, "failed to cache widget state");
        }

        Ok(state)
    }

    async fn compute_widget_state(
        &self,
        user_id: Uuid,
        last_completed_title: Option<&str>,
    ) -> Result<WidgetState, TaskError> {
        let completed_today = self.db.get_completed_today(user_id).await? as i32;
        let streak_days = self.db.get_streak_days(user_id).await?;

        let message = match completed_today {
            0 => "Ready when you are.".to_string(),
            1 => "You showed up. That's the hardest part.".to_string(),
            2..=4 => format!("{completed_today} done. Each one was a win."),
            _ => format!("{completed_today} tasks crushed. Incredible."),
        };

        Ok(WidgetState {
            completed_today,
            streak_days,
            motivational_message: message,
            last_completed_task_title: last_completed_title.map(String::from),
            last_updated: Utc::now(),
        })
    }
}
