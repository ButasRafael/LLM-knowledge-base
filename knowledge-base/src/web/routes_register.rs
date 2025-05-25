use axum::{
    extract::State,
    routing::post,
    Json, Router,
};
use serde::Deserialize;
use serde_json::json;
use tracing::info;

use crate::{
    ctx::Ctx,
    model::manager::ModelManager,
    model::user::{UserBmc, UserForCreate},
    Result,
};

#[derive(Debug, Deserialize)]
struct RegisterPayload {
    username:  String,
    #[serde(rename = "pwd_clear")]   // keep same field name the UI already sends
    password:  String,
}

pub async fn register_handler(
    State(mm): State<ModelManager>,
    Json(payload): Json<RegisterPayload>,
) -> Result<Json<serde_json::Value>> {
    let ctx = Ctx::root_ctx();

    let new_user = UserForCreate {
        username:  payload.username,
        pwd_clear: payload.password,
        role:      None,
    };

    let user_id = UserBmc::create(&ctx, &mm, new_user).await?;
    info!("New account created: id={user_id}");

    Ok(Json(json!({ "id": user_id })))
}

pub fn routes(mm: ModelManager) -> Router {
    Router::new()
        .route("/register", post(register_handler))
        .with_state(mm)
}
