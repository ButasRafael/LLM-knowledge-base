
use axum::{
    extract::State,
    Json,
    routing::post,
    Router,
};
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::{Ctx, Result};
use crate::model::manager::ModelManager;

#[derive(Debug, Deserialize)]
pub struct FineTuneRequest {
    pub prompt: String,
}

#[derive(Debug, Serialize)]
pub struct FineTuneResponse {
    pub response: String,
}

#[tracing::instrument(skip_all, name = "fine_tune_handler")]
pub async fn fine_tune_handler(
    State(mm): State<ModelManager>,
    _ctx: Ctx,
    Json(payload): Json<FineTuneRequest>,
) -> Result<Json<FineTuneResponse>> {
    info!("Received fine-tune request: {:?}", payload.prompt);

    let refined_answer = mm.fine_tune_prompt(&payload.prompt).await?;

    info!("Refined answer: {:?}", refined_answer);

    Ok(Json(FineTuneResponse {
        response: refined_answer,
    }))
}

pub fn routes(mm: ModelManager) -> Router {
    Router::new()
        .route("/fine-tune", post(fine_tune_handler))
        .with_state(mm)
}
