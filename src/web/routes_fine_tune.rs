
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
pub struct FineTuneRequest {
    pub prompt: String,
    pub context: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct FineTuneResponse {
    pub response: String,
}

#[tracing::instrument]
pub async fn fine_tune_handler(
    State(mm): State<ModelManager>,
    _ctx: Ctx,
    Json(payload): Json<FineTuneRequest>,
) -> Result<Json<FineTuneResponse>> {
    println!("->> fine_tune_handler - prompt: {:?}", payload.prompt);

    let refined_prompt = mm.fine_tune_prompt(&payload.prompt, &payload.context).await?;

    let ollama_response = mm.ollama_client.get_response(&refined_prompt).await?;

    info!("Ollama final LLM response: {}", ollama_response);


    Ok(Json(FineTuneResponse {
        response: ollama_response,
    }))
}

pub fn routes(mm: ModelManager) -> Router {
    Router::new()
        .route("/fine-tune", post(fine_tune_handler))
        .with_state(mm)
}
