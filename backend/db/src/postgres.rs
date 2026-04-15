use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use chrono::NaiveDate;

use crate::models::{
    BehavioralSignal, Conversation, ConversationMessage, CreateSignal, CreateTask, DailyDashboard,
    ImageUpload, LifeEntry, ProfilingAnswer, ProfilingQuestion, Task, ThoughtEntry, User,
    WeeklyInsight,
};

#[derive(Clone)]
pub struct PostgresRepo {
    pub pool: PgPool,
}

impl PostgresRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_user(&self, device_id: &str) -> Result<User, sqlx::Error> {
        sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (device_id)
            VALUES ($1)
            RETURNING *
            "#,
        )
        .bind(device_id)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn get_user_by_device_id(
        &self,
        device_id: &str,
    ) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE device_id = $1")
            .bind(device_id)
            .fetch_optional(&self.pool)
            .await
    }

    pub async fn get_user(&self, id: Uuid) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
    }

    pub async fn update_last_active(&self, user_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE users SET last_active_at = now() WHERE id = $1")
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn create_task(
        &self,
        user_id: Uuid,
        task: CreateTask,
    ) -> Result<Task, sqlx::Error> {
        sqlx::query_as::<_, Task>(
            r#"
            INSERT INTO tasks (user_id, title, description, priority, source, due_at, tags, ai_metadata)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            "#,
        )
        .bind(user_id)
        .bind(&task.title)
        .bind(&task.description)
        .bind(&task.priority)
        .bind(&task.source)
        .bind(task.due_at)
        .bind(&task.tags)
        .bind(&task.ai_metadata)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn get_tasks(
        &self,
        user_id: Uuid,
        status: Option<&str>,
    ) -> Result<Vec<Task>, sqlx::Error> {
        match status {
            Some(s) => {
                sqlx::query_as::<_, Task>(
                    "SELECT * FROM tasks WHERE user_id = $1 AND status = $2 ORDER BY created_at DESC",
                )
                .bind(user_id)
                .bind(s)
                .fetch_all(&self.pool)
                .await
            }
            None => {
                sqlx::query_as::<_, Task>(
                    "SELECT * FROM tasks WHERE user_id = $1 ORDER BY created_at DESC",
                )
                .bind(user_id)
                .fetch_all(&self.pool)
                .await
            }
        }
    }

    pub async fn update_task_status(
        &self,
        task_id: Uuid,
        status: &str,
    ) -> Result<Task, sqlx::Error> {
        let completed_at = if status == "completed" {
            Some(Utc::now())
        } else {
            None
        };

        sqlx::query_as::<_, Task>(
            r#"
            UPDATE tasks
            SET status = $1, completed_at = COALESCE($2, completed_at)
            WHERE id = $3
            RETURNING *
            "#,
        )
        .bind(status)
        .bind(completed_at)
        .bind(task_id)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn get_completed_today(&self, user_id: Uuid) -> Result<i64, sqlx::Error> {
        let row: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*)
            FROM tasks
            WHERE user_id = $1
              AND status = 'completed'
              AND completed_at >= CURRENT_DATE
            "#,
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.0)
    }

    /// Count consecutive days (backwards from today) with at least one completed task.
    pub async fn get_streak_days(&self, user_id: Uuid) -> Result<i32, sqlx::Error> {
        let row: (i32,) = sqlx::query_as(
            r#"
            WITH daily AS (
                SELECT DISTINCT (completed_at AT TIME ZONE 'UTC')::date AS d
                FROM tasks
                WHERE user_id = $1 AND status = 'completed' AND completed_at IS NOT NULL
            ),
            numbered AS (
                SELECT d, d - (ROW_NUMBER() OVER (ORDER BY d DESC))::int AS grp
                FROM daily
            )
            SELECT COALESCE(COUNT(*)::int, 0)
            FROM numbered
            WHERE grp = (
                SELECT grp FROM numbered WHERE d = CURRENT_DATE
                LIMIT 1
            )
            "#,
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
        .unwrap_or((0,));

        Ok(row.0)
    }

    pub async fn get_all_active_questions(
        &self,
    ) -> Result<Vec<ProfilingQuestion>, sqlx::Error> {
        sqlx::query_as::<_, ProfilingQuestion>(
            "SELECT * FROM profiling_questions WHERE active = true ORDER BY priority ASC",
        )
        .fetch_all(&self.pool)
        .await
    }

    pub async fn get_answered_question_ids(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<Uuid>, sqlx::Error> {
        let rows: Vec<(Uuid,)> = sqlx::query_as(
            "SELECT question_id FROM profiling_answers WHERE user_id = $1",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|(id,)| id).collect())
    }

    pub async fn save_answer(
        &self,
        user_id: Uuid,
        question_id: Uuid,
        answer_value: &str,
        raw_input: Option<&str>,
        source: &str,
        confidence: f32,
    ) -> Result<ProfilingAnswer, sqlx::Error> {
        sqlx::query_as::<_, ProfilingAnswer>(
            r#"
            INSERT INTO profiling_answers (user_id, question_id, answer_value, raw_input, source, confidence)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (user_id, question_id)
            DO UPDATE SET answer_value = EXCLUDED.answer_value,
                         raw_input = EXCLUDED.raw_input,
                         source = EXCLUDED.source,
                         confidence = EXCLUDED.confidence,
                         answered_at = now()
            RETURNING *
            "#,
        )
        .bind(user_id)
        .bind(question_id)
        .bind(answer_value)
        .bind(raw_input)
        .bind(source)
        .bind(confidence)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn record_signal(
        &self,
        user_id: Uuid,
        signal: CreateSignal,
    ) -> Result<BehavioralSignal, sqlx::Error> {
        sqlx::query_as::<_, BehavioralSignal>(
            r#"
            INSERT INTO behavioral_signals (user_id, signal_type, payload)
            VALUES ($1, $2, $3)
            RETURNING *
            "#,
        )
        .bind(user_id)
        .bind(&signal.signal_type)
        .bind(&signal.payload)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn get_recent_signals(
        &self,
        user_id: Uuid,
        signal_type: Option<&str>,
        hours: i64,
    ) -> Result<Vec<BehavioralSignal>, sqlx::Error> {
        match signal_type {
            Some(st) => {
                sqlx::query_as::<_, BehavioralSignal>(
                    r#"
                    SELECT * FROM behavioral_signals
                    WHERE user_id = $1
                      AND signal_type = $2
                      AND recorded_at >= now() - ($3 || ' hours')::interval
                    ORDER BY recorded_at DESC
                    "#,
                )
                .bind(user_id)
                .bind(st)
                .bind(hours.to_string())
                .fetch_all(&self.pool)
                .await
            }
            None => {
                sqlx::query_as::<_, BehavioralSignal>(
                    r#"
                    SELECT * FROM behavioral_signals
                    WHERE user_id = $1
                      AND recorded_at >= now() - ($2 || ' hours')::interval
                    ORDER BY recorded_at DESC
                    "#,
                )
                .bind(user_id)
                .bind(hours.to_string())
                .fetch_all(&self.pool)
                .await
            }
        }
    }

    pub async fn save_thought_entry(
        &self,
        user_id: Uuid,
        raw_transcript: &str,
        processed_text: Option<&str>,
        sentiment: Option<&str>,
    ) -> Result<ThoughtEntry, sqlx::Error> {
        sqlx::query_as::<_, ThoughtEntry>(
            r#"
            INSERT INTO thought_entries (user_id, raw_transcript, processed_text, sentiment)
            VALUES ($1, $2, $3, $4)
            RETURNING *
            "#,
        )
        .bind(user_id)
        .bind(raw_transcript)
        .bind(processed_text)
        .bind(sentiment)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn get_latest_insight(
        &self,
        user_id: Uuid,
    ) -> Result<Option<WeeklyInsight>, sqlx::Error> {
        sqlx::query_as::<_, WeeklyInsight>(
            "SELECT * FROM weekly_insights WHERE user_id = $1 ORDER BY week_start DESC LIMIT 1",
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn save_insight(
        &self,
        insight: &WeeklyInsight,
    ) -> Result<WeeklyInsight, sqlx::Error> {
        sqlx::query_as::<_, WeeklyInsight>(
            r#"
            INSERT INTO weekly_insights (id, user_id, week_start, summary_text, patterns, recommendations, tasks_completed, streak_days)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (user_id, week_start)
            DO UPDATE SET summary_text = EXCLUDED.summary_text,
                         patterns = EXCLUDED.patterns,
                         recommendations = EXCLUDED.recommendations,
                         tasks_completed = EXCLUDED.tasks_completed,
                         streak_days = EXCLUDED.streak_days,
                         generated_at = now()
            RETURNING *
            "#,
        )
        .bind(insight.id)
        .bind(insight.user_id)
        .bind(insight.week_start)
        .bind(&insight.summary_text)
        .bind(&insight.patterns)
        .bind(&insight.recommendations)
        .bind(insight.tasks_completed)
        .bind(insight.streak_days)
        .fetch_one(&self.pool)
        .await
    }

    // ── Conversations ──────────────────────────────────────────────

    pub async fn create_conversation(
        &self,
        user_id: Uuid,
        title: Option<&str>,
    ) -> Result<Conversation, sqlx::Error> {
        sqlx::query_as::<_, Conversation>(
            r#"
            INSERT INTO conversations (user_id, title)
            VALUES ($1, $2)
            RETURNING *
            "#,
        )
        .bind(user_id)
        .bind(title)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn get_conversation(
        &self,
        id: Uuid,
    ) -> Result<Option<Conversation>, sqlx::Error> {
        sqlx::query_as::<_, Conversation>("SELECT * FROM conversations WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
    }

    pub async fn get_user_conversations(
        &self,
        user_id: Uuid,
        limit: i64,
    ) -> Result<Vec<Conversation>, sqlx::Error> {
        sqlx::query_as::<_, Conversation>(
            "SELECT * FROM conversations WHERE user_id = $1 ORDER BY updated_at DESC LIMIT $2",
        )
        .bind(user_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
    }

    // ── Messages ───────────────────────────────────────────────────

    pub async fn save_message(
        &self,
        conversation_id: Uuid,
        role: &str,
        content: &str,
        image_urls: Option<&[String]>,
        metadata: serde_json::Value,
    ) -> Result<ConversationMessage, sqlx::Error> {
        sqlx::query_as::<_, ConversationMessage>(
            r#"
            INSERT INTO conversation_messages (conversation_id, role, content, image_urls, metadata)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#,
        )
        .bind(conversation_id)
        .bind(role)
        .bind(content)
        .bind(image_urls)
        .bind(metadata)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn get_conversation_messages(
        &self,
        conversation_id: Uuid,
        limit: i64,
    ) -> Result<Vec<ConversationMessage>, sqlx::Error> {
        sqlx::query_as::<_, ConversationMessage>(
            r#"
            SELECT * FROM conversation_messages
            WHERE conversation_id = $1
            ORDER BY created_at ASC
            LIMIT $2
            "#,
        )
        .bind(conversation_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
    }

    // ── Life Entries ───────────────────────────────────────────────

    pub async fn save_life_entry(
        &self,
        user_id: Uuid,
        category: &str,
        title: &str,
        content: &str,
        structured_data: Option<serde_json::Value>,
        source_message_id: Option<Uuid>,
        image_urls: Option<&[String]>,
        tags: &[String],
        entry_date: NaiveDate,
    ) -> Result<LifeEntry, sqlx::Error> {
        sqlx::query_as::<_, LifeEntry>(
            r#"
            INSERT INTO life_entries (user_id, category, title, content, structured_data, source_message_id, image_urls, tags, entry_date)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
            "#,
        )
        .bind(user_id)
        .bind(category)
        .bind(title)
        .bind(content)
        .bind(structured_data)
        .bind(source_message_id)
        .bind(image_urls)
        .bind(tags)
        .bind(entry_date)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn get_life_entries(
        &self,
        user_id: Uuid,
        date: Option<NaiveDate>,
        category: Option<&str>,
        limit: i64,
    ) -> Result<Vec<LifeEntry>, sqlx::Error> {
        match (date, category) {
            (Some(d), Some(c)) => {
                sqlx::query_as::<_, LifeEntry>(
                    r#"
                    SELECT * FROM life_entries
                    WHERE user_id = $1 AND entry_date = $2 AND category = $3
                    ORDER BY created_at DESC LIMIT $4
                    "#,
                )
                .bind(user_id)
                .bind(d)
                .bind(c)
                .bind(limit)
                .fetch_all(&self.pool)
                .await
            }
            (Some(d), None) => {
                sqlx::query_as::<_, LifeEntry>(
                    r#"
                    SELECT * FROM life_entries
                    WHERE user_id = $1 AND entry_date = $2
                    ORDER BY created_at DESC LIMIT $3
                    "#,
                )
                .bind(user_id)
                .bind(d)
                .bind(limit)
                .fetch_all(&self.pool)
                .await
            }
            (None, Some(c)) => {
                sqlx::query_as::<_, LifeEntry>(
                    r#"
                    SELECT * FROM life_entries
                    WHERE user_id = $1 AND category = $2
                    ORDER BY created_at DESC LIMIT $3
                    "#,
                )
                .bind(user_id)
                .bind(c)
                .bind(limit)
                .fetch_all(&self.pool)
                .await
            }
            (None, None) => {
                sqlx::query_as::<_, LifeEntry>(
                    r#"
                    SELECT * FROM life_entries
                    WHERE user_id = $1
                    ORDER BY created_at DESC LIMIT $2
                    "#,
                )
                .bind(user_id)
                .bind(limit)
                .fetch_all(&self.pool)
                .await
            }
        }
    }

    // ── Dashboard ──────────────────────────────────────────────────

    pub async fn get_dashboard(
        &self,
        user_id: Uuid,
        date: NaiveDate,
    ) -> Result<Option<DailyDashboard>, sqlx::Error> {
        sqlx::query_as::<_, DailyDashboard>(
            "SELECT * FROM daily_dashboards WHERE user_id = $1 AND dashboard_date = $2",
        )
        .bind(user_id)
        .bind(date)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn save_dashboard(
        &self,
        user_id: Uuid,
        date: NaiveDate,
        summary_text: &str,
        mood_score: Option<i32>,
        energy_score: Option<i32>,
        categories_breakdown: serde_json::Value,
        highlights: serde_json::Value,
        financial_summary: Option<serde_json::Value>,
    ) -> Result<DailyDashboard, sqlx::Error> {
        sqlx::query_as::<_, DailyDashboard>(
            r#"
            INSERT INTO daily_dashboards (user_id, dashboard_date, summary_text, mood_score, energy_score, categories_breakdown, highlights, financial_summary)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (user_id, dashboard_date)
            DO UPDATE SET summary_text = EXCLUDED.summary_text,
                         mood_score = EXCLUDED.mood_score,
                         energy_score = EXCLUDED.energy_score,
                         categories_breakdown = EXCLUDED.categories_breakdown,
                         highlights = EXCLUDED.highlights,
                         financial_summary = EXCLUDED.financial_summary,
                         generated_at = now()
            RETURNING *
            "#,
        )
        .bind(user_id)
        .bind(date)
        .bind(summary_text)
        .bind(mood_score)
        .bind(energy_score)
        .bind(categories_breakdown)
        .bind(highlights)
        .bind(financial_summary)
        .fetch_one(&self.pool)
        .await
    }

    // ── Images ─────────────────────────────────────────────────────

    pub async fn save_image_upload(
        &self,
        user_id: Uuid,
        filename: &str,
        mime_type: &str,
        size_bytes: i64,
        storage_path: &str,
        ai_description: Option<&str>,
    ) -> Result<ImageUpload, sqlx::Error> {
        sqlx::query_as::<_, ImageUpload>(
            r#"
            INSERT INTO image_uploads (user_id, filename, mime_type, size_bytes, storage_path, ai_description)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            "#,
        )
        .bind(user_id)
        .bind(filename)
        .bind(mime_type)
        .bind(size_bytes)
        .bind(storage_path)
        .bind(ai_description)
        .fetch_one(&self.pool)
        .await
    }
}
