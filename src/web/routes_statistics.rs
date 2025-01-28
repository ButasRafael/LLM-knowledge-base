//MOCK, IMPLEMENT REAL STATISTICS AFTER IMPLEMENTING THE PIPELINES
use axum::{
    extract::State,
    Json, Router,
    routing::get,
};
use serde::{Serialize, Deserialize};
use tracing::info;
use sqlx::Row;

use crate::{
    ctx::Ctx,
    error::{Error, Result},
    model::manager::ModelManager,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct StatisticsResponse {
    pub total_users: i64,
    pub total_tasks: i64,
    pub total_documents: i64,
    // Add more fields if you like, e.g. total queries, LLM usage, etc.(after implementation, now just use these three fields from the database)
}

pub async fn get_statistics(
    State(mm): State<ModelManager>,
    _ctx: Ctx,
) -> Result<Json<StatisticsResponse>> {
    println!("->> {:<12} - get_statistics", "HANDLER");

    let total_users = sqlx::query("SELECT COUNT(*) AS count FROM \"user\"")
        .fetch_one(mm.db())
        .await?
        .try_get::<i64, _>("count")?;

    let total_tasks = sqlx::query("SELECT COUNT(*) AS count FROM task")
        .fetch_one(mm.db())
        .await?
        .try_get::<i64, _>("count")?;

    let total_documents = sqlx::query("SELECT COUNT(*) AS count FROM document")
        .fetch_one(mm.db())
        .await?
        .try_get::<i64, _>("count")?;

    let response = StatisticsResponse {
        total_users,
        total_tasks,
        total_documents,
    };

    info!("Statistics: {:?}", response);

    Ok(Json(response))
}

pub fn routes(mm: ModelManager) -> Router {
    Router::new()
        .route("/statistics", get(get_statistics))
        .with_state(mm)
}
