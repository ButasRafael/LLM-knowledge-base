use axum::{
    extract::{Path, State},
    Json,
    Router,
    routing::{post, get, put, delete},
    http::StatusCode,
};
use serde::Deserialize;
use tracing::info;

use crate::{
    ctx::Ctx,
    model::user::{User, UserBmc, UserForCreate, UserForUpdate},
    model::manager::ModelManager,
    Error, Result,
};
use crate::model::user::{PasswordChange, UserForLogin};
use crate::web::mw_auth::CtxExtError::CtxNotInRequestExt;

#[tracing::instrument]
pub async fn create_user(
    State(mm): State<ModelManager>,
    ctx: Ctx,
    Json(user_c): Json<UserForCreate>,
) -> Result<Json<User>> {
    println!("->> {:<12} - create_user", "HANDLER");
    let id = UserBmc::create(&ctx, &mm, user_c).await?;
    let user = UserBmc::get(&ctx, &mm, id).await?;
    info!("User created: {:?}", user);
    Ok(Json(user))
}

#[tracing::instrument]
pub async fn list_users(
    State(mm): State<ModelManager>,
    ctx: Ctx,
) -> Result<Json<Vec<User>>> {
    println!("->> {:<12} - list_users", "HANDLER");
    let users = UserBmc::list(&ctx, &mm).await?;
    info!("Users listed");
    Ok(Json(users))
}

#[tracing::instrument]
pub async fn get_user(
    State(mm): State<ModelManager>,
    ctx: Ctx,
    Path(id): Path<i64>,
) -> Result<Json<User>> {
    println!("->> {:<12} - get_user", "HANDLER");
    let user = UserBmc::get(&ctx, &mm, id).await?;
    info!("User retrieved: {:?}", user);
    Ok(Json(user))
}

#[tracing::instrument]
pub async fn update_user(
    State(mm): State<ModelManager>,
    ctx: Ctx,
    Path(id): Path<i64>,
    Json(user_u): Json<UserForUpdate>,
) -> Result<StatusCode> {
    println!("->> {:<12} - update_user", "HANDLER");
    UserBmc::update(&ctx, &mm, id, user_u).await?;
    info!("User updated: id={}", id);
    Ok(StatusCode::NO_CONTENT)
}

#[tracing::instrument]
pub async fn update_password(
    State(mm): State<ModelManager>,
    ctx: Ctx,
    Path(id): Path<i64>,
    Json(payload): Json<PasswordChange>,
) -> Result<StatusCode> {
    println!("->> {:<12} - update_password", "HANDLER");

    if ctx.user_id() != id {
        return Err(Error::CtxExt(CtxNotInRequestExt));
    }

    let user_for_login = UserBmc::get::<UserForLogin>(&ctx, &mm, id).await?;

    if let Some(existing_pwd_hash) = user_for_login.pwd {
        crate::crypt::pwd::validate_pwd(
            &crate::crypt::EncryptContent {
                salt: user_for_login.pwd_salt.to_string(),
                content: payload.old_password,
            },
            &existing_pwd_hash
        ).map_err(|_| Error::PwdNotMatching)?;
    } else {
        return Err(Error::UserHasNoPwd { user_id: id });
    }

    UserBmc::update_pwd(&ctx, &mm, id, &payload.new_password).await?;

    Ok(StatusCode::NO_CONTENT)
}


#[tracing::instrument]
pub async fn delete_user(
    State(mm): State<ModelManager>,
    ctx: Ctx,
    Path(id): Path<i64>,
) -> Result<StatusCode> {
    println!("->> {:<12} - delete_user", "HANDLER");
    UserBmc::delete(&ctx, &mm, id).await?;
    info!("User deleted: id={}", id);
    Ok(StatusCode::NO_CONTENT)
}

#[tracing::instrument]
pub async fn get_me(
    State(mm): State<ModelManager>,
    ctx: Ctx,
) -> Result<Json<User>> {
    println!("->> {:<12} - get_me", "HANDLER");
    let user_id = ctx.user_id();
    let user = UserBmc::get(&ctx, &mm, user_id).await?;
    Ok(Json(user))
}

pub fn routes(mm: ModelManager) -> Router {
    Router::new()
        .route("/users", post(create_user))
        .route("/users", get(list_users))
        .route("/users/:id", get(get_user))
        .route("/users/:id", put(update_user))
        .route("/users/:id", delete(delete_user))
        .route("/users/me", get(get_me))
        .route("/users/:id/password", put(update_password))
        .with_state(mm)
}
