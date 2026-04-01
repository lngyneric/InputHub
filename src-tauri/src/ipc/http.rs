use crate::{
    agents,
    config::Config,
    db::Db,
    types::{LaunchRequest, LaunchResponse, TaskStatus},
};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use tracing::{info, warn};
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    db: Db,
    cfg: Config,
}

#[derive(Debug, Deserialize)]
struct ListQuery {
    #[serde(default = "default_limit")]
    limit: i64,
}
fn default_limit() -> i64 {
    50
}

pub async fn serve(addr: String, db: Db, cfg: Config) -> anyhow::Result<()> {
    let state = AppState { db, cfg };
    let app = Router::new()
        .route("/healthz", get(|| async { "ok" }))
        .route("/launch", post(launch))
        .route("/tasks", get(list_tasks))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!(%addr, "http listening");
    axum::serve(listener, app).await?;
    Ok(())
}

async fn launch(State(st): State<AppState>, Json(req): Json<LaunchRequest>) -> impl IntoResponse {
    let task_id = Uuid::new_v4();
    let raw = req.raw.clone();
    let source = if req.source.trim().is_empty() {
        "http".to_string()
    } else {
        req.source.clone()
    };
    let app = req.app.clone();

    if let Err(e) = st
        .db
        .insert_task(task_id, &raw, &source, &app, TaskStatus::Received)
        .await
    {
        warn!(error=%e, "db insert_task failed");
        return (StatusCode::INTERNAL_SERVER_ERROR, "db error").into_response();
    }

    // 异步执行（Phase 1 先做最简单路由；Phase 2/3 会逐步填充）
    let db = st.db.clone();
    let cfg = st.cfg.clone();
    tokio::spawn(async move {
        if let Err(e) = agents::execute_task(task_id, raw, db, cfg).await {
            warn!(task_id=%task_id, error=%e, "execute_task failed");
        }
    });

    (
        StatusCode::OK,
        Json(LaunchResponse {
            task_id,
            status: TaskStatus::Received,
        }),
    )
        .into_response()
}

async fn list_tasks(State(st): State<AppState>, Query(q): Query<ListQuery>) -> impl IntoResponse {
    match st.db.list_tasks(q.limit).await {
        Ok(tasks) => (StatusCode::OK, Json(tasks)).into_response(),
        Err(e) => {
            warn!(error=%e, "db list_tasks failed");
            (StatusCode::INTERNAL_SERVER_ERROR, "db error").into_response()
        }
    }
}
