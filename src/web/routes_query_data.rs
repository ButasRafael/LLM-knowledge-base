
use axum::{
    extract::State,
    Json, Router,
    routing::post,
};
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::{Ctx, Result};
use crate::model::manager::ModelManager;

#[derive(Debug, Deserialize)]
pub struct QueryRequest {
    pub prompt: String,
}

#[derive(Debug, Serialize)]
pub struct QueryResponse {
    pub retrieved_contexts: Vec<String>,
}

#[tracing::instrument]
pub async fn query_data_handler(
    State(mm): State<ModelManager>,
    ctx: Ctx,
    Json(payload): Json<QueryRequest>,
) -> Result<Json<QueryResponse>> {
    println!("->> query_data_handler - prompt: {:?}", payload.prompt);

    let retrieved_docs = mm.query_data(&ctx, &payload.prompt).await?;

    info!("Retrieved docs from Qdrant: {:?}", retrieved_docs);

    Ok(Json(QueryResponse {
        retrieved_contexts: retrieved_docs,
    }))
}

pub fn routes(mm: ModelManager) -> Router {
    Router::new()
        .route("/query/data", post(query_data_handler))
        .with_state(mm)
}
