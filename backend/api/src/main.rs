use std::sync::Arc;

use axum::routing::{get, post, patch};
use axum::Router;
use sqlx::postgres::PgPoolOptions;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

use focusflow_ai::classifier::IntentClassifier;
use focusflow_ai::embeddings::EmbeddingClient;
use focusflow_ai::insights_generator::InsightsGenerator;
use focusflow_ai::whisper::WhisperClient;
use focusflow_core::insights::InsightService;
use focusflow_core::profiling::ProfilingEngine;
use focusflow_core::signals::SignalService;
use focusflow_core::tasks::TaskService;
use focusflow_db::postgres::PostgresRepo;
use focusflow_db::redis_cache::RedisCache;

use focusflow_api::{routes, ws, AppState};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".into());
    let gemini_api_key = std::env::var("GEMINI_API_KEY").unwrap_or_default();
    let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let host = std::env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".into());
    let port = std::env::var("SERVER_PORT").unwrap_or_else(|_| "8080".into());

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await?;
    tracing::info!("connected to postgres");

    let redis_client = redis::Client::open(redis_url.as_str())?;
    let redis_conn = redis_client.get_multiplexed_async_connection().await?;
    tracing::info!("connected to redis");

    let db = Arc::new(PostgresRepo::new(pool));
    let cache = Arc::new(RedisCache::new(redis_conn));

    let task_service = Arc::new(TaskService::new(Arc::clone(&db), Arc::clone(&cache)));
    let profiling_engine = Arc::new(ProfilingEngine::new(Arc::clone(&db)));
    let insight_service = Arc::new(InsightService::new(
        Arc::clone(&db),
        Arc::new(InsightsGenerator::new(&gemini_api_key)),
    ));
    let signal_service = Arc::new(SignalService::new(Arc::clone(&db)));

    let whisper = Arc::new(WhisperClient::new(&gemini_api_key));
    let classifier = Arc::new(IntentClassifier::new(&gemini_api_key));
    let embeddings = Arc::new(EmbeddingClient::new(&gemini_api_key));

    let state = AppState {
        db,
        cache,
        task_service,
        profiling_engine,
        insight_service,
        signal_service,
        whisper,
        classifier,
        embeddings,
        jwt_secret,
    };

    let app = Router::new()
        .route("/api/v1/auth/device", post(routes::auth::device_auth))
        .route("/api/v1/voice/upload", post(routes::voice::upload_voice))
        .route(
            "/api/v1/tasks",
            get(routes::tasks::list_tasks).post(routes::tasks::create_task),
        )
        .route("/api/v1/tasks/:id", patch(routes::tasks::update_task))
        .route("/api/v1/widget/state", get(routes::widget::get_widget_state))
        .route(
            "/api/v1/profile/next-question",
            get(routes::profile::next_question),
        )
        .route("/api/v1/profile/answer", post(routes::profile::submit_answer))
        .route("/api/v1/profile/skip", post(routes::profile::skip_question))
        .route(
            "/api/v1/insights/latest",
            get(routes::insights::latest_insight),
        )
        .route("/api/v1/signals", post(routes::signals::record_signals))
        .route("/api/v1/ws", get(ws::handler::ws_handler))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = format!("{host}:{port}");
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("listening on {addr}");

    axum::serve(listener, app).await?;

    Ok(())
}
