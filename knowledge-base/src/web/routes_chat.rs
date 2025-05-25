use axum::{
    routing::{get, post},
    Router,
    extract::{Path, State},
    Json,
};
use serde::{Deserialize, Serialize};
use tokio::spawn;
use axum::http::StatusCode;
use crate::{Ctx, Result};
use crate::model::manager::ModelManager;
use crate::model::chat::*;
use crate::utils::token;


#[derive(Serialize)]
struct ConvList {
    conversations: Vec<Conversation>,
}

#[derive(Deserialize)]
struct NewConv {
    title: Option<String>,
}
#[derive(Deserialize)]
struct SendReq {
    prompt: String,
}
#[derive(Serialize)]
struct SendRes {
    answer: String,
}

pub async fn create_conv(
    State(mm): State<ModelManager>,
    ctx: Ctx,
    Json(body): Json<NewConv>,
) -> Result<Json<Conversation>> {
    let id = ConversationBmc::create(
        &ctx,
        &mm,
        body.title.as_deref().unwrap_or("Untitled"),
    )
        .await?;
    Ok(Json(ConversationBmc::get(&ctx, &mm, id).await?))
}

pub async fn list_conv(
    State(mm): State<ModelManager>,
    ctx: Ctx,
) -> Result<Json<ConvList>> {
    let conversations = ConversationBmc::list_for_user(&ctx, &mm).await?;
    Ok(Json(ConvList { conversations }))
}

pub async fn list_msgs(
    State(mm): State<ModelManager>,
    ctx: Ctx,
    Path(id): Path<i64>,
) -> Result<Json<Vec<Message>>> {
    // enforce ownership
    ConversationBmc::get::<Conversation>(&ctx, &mm, id).await?;
    let msgs = MessageBmc::recent(&ctx, &mm, id, 32_768).await?;
    Ok(Json(msgs))
}

pub async fn send_msg(
    State(mm): State<ModelManager>,
    ctx: Ctx,
    Path(id): Path<i64>,
    Json(body): Json<SendReq>,
) -> Result<(StatusCode, Json<SendRes>)> {
    // 1) persist the user’s prompt immediately
    MessageBmc::add(&ctx, &mm, id, "user", &body.prompt, token::count(&body.prompt)).await?;

    // 2) fire off the LLM & persistence in background
    let mm2 = mm.clone();
    let ctx2 = ctx.clone();
    let prompt = body.prompt.clone();
    spawn(async move {
        if let Ok(answer) = mm2.fine_tune_prompt(&ctx2, &prompt).await {
            let _ = MessageBmc::add(
                &ctx2, &mm2, id, "assistant", &answer, token::count(&answer)
            ).await;
        }
    });

    // 3) immediately return 202 Accepted (no answer payload)
    Ok((StatusCode::ACCEPTED, Json(SendRes { answer: String::new() })))
}

pub fn routes(mm: ModelManager) -> Router {
    Router::new()
        .route("/chat/conversations", post(create_conv).get(list_conv))
        .route(
            "/chat/conversations/:id/messages",
            get(list_msgs).post(send_msg),
        )
        .with_state(mm)
}
