use axum::{extract::{Json, State}, response::IntoResponse, http::StatusCode, Router};
use axum::routing::post;
use serde::Deserialize;
use serde_json::{json, Value};
use tower_cookies::{Cookie, Cookies};
use tracing::info;

use crate::{
    web::AUTH_TOKEN,
    ctx::Ctx,
    model::user::{UserBmc, UserForCreate},
    model::manager::ModelManager,
    Error, Result,
};
use crate::crypt::{pwd, EncryptContent};
use crate::model::user::UserForLogin;
use crate::web::{remove_token_cookie, set_token_cookie};

#[derive(Debug, Deserialize)]
struct LoginPayload {
    username: String,
    password: String,
}

#[derive(Debug, Deserialize)]
struct LogoffPayload {
    logoff: bool,
}

#[tracing::instrument]
pub async fn api_login(
    State(mm): State<ModelManager>,
    cookies: Cookies,
    Json(payload): Json<LoginPayload>,
) -> Result<Json<Value>> {
    println!("->> {:<12} - api_login", "HANDLER");

    let LoginPayload { username, password } = payload;
    let root_ctx = Ctx::root_ctx();
    let user :UserForLogin = UserBmc::first_by_username(&root_ctx, &mm, &username).await?.ok_or(Error::UserNotFound)?;

    let user_id = user.id;
    let Some(pwd) = user.pwd else {
        return Err(Error::UserHasNoPwd{user_id});
    };

    pwd::validate_pwd(&EncryptContent {
        salt: user.pwd_salt.to_string(),
        content: password.clone(),
    }, &pwd).map_err(|_| Error::PwdNotMatching)?;

    set_token_cookie(&cookies, &user.username, &user.token_salt.to_string())?;

    let body = Json(json!({
        "result": {
            "success": true,
            "user_id": user.id,
            "username": user.username,
        }
    }));

    info!("Login success for user_id={}", user.id);
    Ok(body)
}

#[tracing::instrument]
async fn api_logoff(
    State(_mm): State<ModelManager>,
    cookies: Cookies,
    Json(payload): Json<LogoffPayload>,
) -> Result<Json<Value>> {
    println!("{:<12} - api_logoff_handler", "HANDLER");
    let should_logoff = payload.logoff;

    if should_logoff {
        remove_token_cookie(&cookies)?;
    }

    let body = Json(json!({
		"result": {
			"logged_off": should_logoff
		}
	}));

    info!("Logoff success");
    Ok(body)
}



pub fn routes(mm: ModelManager) -> Router {
    Router::new()
        .route("/api/login", post(api_login))
        .route("/api/logoff", post(api_logoff))
        .with_state(mm)
}
