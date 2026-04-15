use redis::AsyncCommands;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::models::WidgetState;

#[derive(thiserror::Error, Debug)]
pub enum CacheError {
    #[error("redis error: {0}")]
    Redis(#[from] redis::RedisError),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub struct RedisCache {
    conn: Mutex<redis::aio::MultiplexedConnection>,
}

impl RedisCache {
    pub fn new(conn: redis::aio::MultiplexedConnection) -> Self {
        Self {
            conn: Mutex::new(conn),
        }
    }

    fn widget_key(user_id: Uuid) -> String {
        format!("widget:{user_id}")
    }

    fn session_key(user_id: Uuid) -> String {
        format!("session:{user_id}")
    }

    pub async fn get_widget_state(
        &self,
        user_id: Uuid,
    ) -> Result<Option<WidgetState>, CacheError> {
        let mut conn = self.conn.lock().await;
        let raw: Option<String> = conn.get(Self::widget_key(user_id)).await?;
        match raw {
            Some(json) => Ok(Some(serde_json::from_str(&json)?)),
            None => Ok(None),
        }
    }

    pub async fn set_widget_state(
        &self,
        user_id: Uuid,
        state: &WidgetState,
    ) -> Result<(), CacheError> {
        let json = serde_json::to_string(state)?;
        let mut conn = self.conn.lock().await;
        conn.set_ex::<_, _, ()>(Self::widget_key(user_id), &json, 3600).await?;
        Ok(())
    }

    pub async fn invalidate_widget(&self, user_id: Uuid) -> Result<(), CacheError> {
        let mut conn = self.conn.lock().await;
        conn.del::<_, ()>(Self::widget_key(user_id)).await?;
        Ok(())
    }

    pub async fn store_session(
        &self,
        user_id: Uuid,
        device_id: &str,
        ttl_secs: u64,
    ) -> Result<(), CacheError> {
        let mut conn = self.conn.lock().await;
        conn.set_ex::<_, _, ()>(Self::session_key(user_id), device_id, ttl_secs)
            .await?;
        Ok(())
    }

    pub async fn get_session(&self, user_id: Uuid) -> Result<Option<String>, CacheError> {
        let mut conn = self.conn.lock().await;
        let val: Option<String> = conn.get(Self::session_key(user_id)).await?;
        Ok(val)
    }
}
