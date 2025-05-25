use axum::{
    extract::State,
    Json, Router,
    routing::get,
};
use serde::{Serialize, Deserialize};
use sqlx::Row;

use crate::{
    ctx::Ctx,
    error::{Error, Result},
    model::manager::ModelManager,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct DayCount {
    pub day: String,    // e.g. "2025-05-18"
    pub count: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StatisticsResponse {
    pub total_users: i64,
    pub total_tasks: i64,
    pub total_documents: i64,
    pub total_conversations: i64,
    pub total_messages: i64,
    pub avg_messages_per_conversation: f64,

    // New:
    pub total_pipeline_runs: i64,
    pub avg_pipeline_duration_ms: f64,

    pub avg_tasks_per_user: f64,
    pub avg_docs_per_user: f64,

    pub messages_last_7_days: Vec<DayCount>,
}

pub async fn get_statistics(
    State(mm): State<ModelManager>,
    _ctx: Ctx,
) -> Result<Json<StatisticsResponse>> {
    // 1) core counts
    let total_users: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM \"user\"")
        .fetch_one(mm.db()).await?;
    let total_tasks: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM task")
        .fetch_one(mm.db()).await?;
    let total_documents: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM document")
        .fetch_one(mm.db()).await?;
    let total_conversations: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM conversation")
        .fetch_one(mm.db()).await?;
    let total_messages: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM message")
        .fetch_one(mm.db()).await?;

    let avg_messages_per_conversation = if total_conversations > 0 {
        total_messages as f64 / total_conversations as f64
    } else { 0.0 };

    // 2) pipeline stats
    let total_pipeline_runs: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM pipeline_log")
            .fetch_one(mm.db()).await
            .unwrap_or(0);

    let avg_pipeline_duration_ms: Option<f64> =
        sqlx::query_scalar("SELECT AVG(duration_ms)::double precision FROM pipeline_log")
            .fetch_one(mm.db()).await
            .unwrap_or(None);

    // 3) user‐level averages
     let avg_tasks_per_user: Option<f64> =
         sqlx::query_scalar(
             r#"
           SELECT AVG(task_count)::double precision
           FROM (
             SELECT u.id,
                    COUNT(t.id) AS task_count
             FROM "user" u
             LEFT JOIN task t ON t.created_by = u.id
             GROUP BY u.id
           ) sub
         "#
         )
            .fetch_one(mm.db()).await
            .unwrap_or(None);

     let avg_docs_per_user: Option<f64> =
         sqlx::query_scalar(
             r#"
           SELECT AVG(doc_count)::double precision
           FROM (
             SELECT u.id,
                    COUNT(d.id) AS doc_count
             FROM "user" u
             LEFT JOIN document d ON d.uploaded_by = u.id
             GROUP BY u.id
           ) sub
         "#
         )
            .fetch_one(mm.db()).await
            .unwrap_or(None);

    // 4) messages per day over last 7 days
    let rows = sqlx::query(
        "SELECT to_char(created_at::date, 'YYYY-MM-DD') AS day, COUNT(*)
         FROM message
         WHERE created_at >= now() - interval '7 days'
         GROUP BY day
         ORDER BY day"
    )
        .fetch_all(mm.db()).await?;

    let messages_last_7_days = rows.into_iter().map(|r| {
        DayCount {
            day: r.try_get::<String, _>("day").unwrap(),
            count: r.try_get::<i64, _>("count").unwrap(),
        }
    }).collect();

    Ok(Json(StatisticsResponse {
        total_users,
        total_tasks,
        total_documents,
        total_conversations,
        total_messages,
        avg_messages_per_conversation,

        total_pipeline_runs,
        avg_pipeline_duration_ms: avg_pipeline_duration_ms.unwrap_or(0.0),

        avg_tasks_per_user: avg_tasks_per_user.unwrap_or(0.0),
        avg_docs_per_user: avg_docs_per_user.unwrap_or(0.0),

        messages_last_7_days,
    }))
}

pub fn routes(mm: ModelManager) -> Router {
    Router::new()
        .route("/statistics", get(get_statistics))
        .with_state(mm)
}
