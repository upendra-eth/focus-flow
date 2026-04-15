pub mod middleware;
pub mod routes;
pub mod ws;

use std::sync::Arc;

use serde::{Deserialize, Serialize};

use focusflow_ai::agent::AgentChat;
use focusflow_ai::classifier::IntentClassifier;
use focusflow_ai::embeddings::EmbeddingClient;
use focusflow_ai::whisper::WhisperClient;
use focusflow_core::insights::InsightService;
use focusflow_core::profiling::ProfilingEngine;
use focusflow_core::signals::SignalService;
use focusflow_core::tasks::TaskService;
use focusflow_db::postgres::PostgresRepo;
use focusflow_db::redis_cache::RedisCache;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<PostgresRepo>,
    pub cache: Arc<RedisCache>,
    pub task_service: Arc<TaskService>,
    pub profiling_engine: Arc<ProfilingEngine>,
    pub insight_service: Arc<InsightService>,
    pub signal_service: Arc<SignalService>,
    pub whisper: Arc<WhisperClient>,
    pub classifier: Arc<IntentClassifier>,
    pub embeddings: Arc<EmbeddingClient>,
    pub agent: Arc<AgentChat>,
    pub jwt_secret: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}
