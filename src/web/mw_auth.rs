use crate::ctx::Ctx;
use crate::web::{set_token_cookie, AUTH_TOKEN};
use crate::{Error, Result};
use async_trait::async_trait;
use axum::body::Body;
use axum::extract::{FromRequestParts, State};
use axum::http::request::Parts;
use axum::http::Request;
use axum::middleware::Next;
use axum::response::Response;
use serde::Serialize;
use tower_cookies::{Cookie, Cookies};
use crate::crypt::token::{validate_token, Token};
use crate::model::manager::ModelManager;
use crate::model::user::{UserBmc, UserForAuth};

type CtxExtResult = core::result::Result<Ctx, CtxExtError>;

#[derive(Clone, Serialize, Debug)]
pub enum CtxExtError {
    TokenNotInCookie,
    TokenWrongFormat,

    UserNotFound,
    ModelAccessError(String),
    FailValidate,
    CannotSetTokenCookie,

    CtxNotInRequestExt,
    CtxCreateFail(String),

    CtxCannotNewRootCtx,
}


pub async fn mw_require_auth(
    ctx: Result<Ctx>,
    req: Request<Body>,
    next: Next,
) -> Result<Response> {
    println!("->> {:<12} - mw_require_auth - {ctx:?}", "MIDDLEWARE");

    ctx?;

    Ok(next.run(req).await)
}

pub async fn mw_require_admin(
    ctx: Result<Ctx>,
    req: Request<Body>,
    next: Next,
) -> Result<Response> {
    println!("->> {:<12} - mw_require_admin - {ctx:?}", "MIDDLEWARE");

    let c = ctx?; 

    if !c.is_admin() {
        return Err(Error::ServiceError("You must be an admin to access this resource.".into()));
    }
    Ok(next.run(req).await)
}


async fn _ctx_resolve(mm:State<ModelManager>, cookies:&Cookies)->CtxExtResult{
    let token = cookies
        .get(AUTH_TOKEN)
        .map(|c| c.value().to_string())
        .ok_or(CtxExtError::TokenNotInCookie)?;

    let token:Token = token.parse().map_err(|_| CtxExtError::TokenWrongFormat)?;

    let user:UserForAuth = UserBmc::first_by_username(&Ctx::root_ctx(), &mm, &token.identifier)
        .await
        .map_err(|e| CtxExtError::ModelAccessError(e.to_string()))?
        .ok_or(CtxExtError::UserNotFound)?;

    validate_token(&token, &user.token_salt.to_string())
        .map_err(|_| CtxExtError::FailValidate)?;

    set_token_cookie(cookies, &user.username, &user.token_salt.to_string())
        .map_err(|_| CtxExtError::CannotSetTokenCookie)?;

    Ctx::new(user.id, &user.role).map_err(|e| CtxExtError::CtxCreateFail(e.to_string()))
}

pub async fn mw_ctx_resolver(
    mm: State<ModelManager>,
    cookies: Cookies,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response> {
    println!("->> {:<12} - mw_ctx_resolver", "MIDDLEWARE");

    let ctx_result = _ctx_resolve(mm, &cookies).await;
    if ctx_result.is_err() && !matches!(ctx_result, Err(CtxExtError::TokenNotInCookie))
    {
        cookies.remove(Cookie::from(AUTH_TOKEN))
    }

    req.extensions_mut().insert(ctx_result);

    Ok(next.run(req).await)
}

#[async_trait]
impl<S: Send + Sync> FromRequestParts<S> for Ctx {
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self> {
        println!("{:<12} - Ctx", "EXTRACTOR");

        parts
            .extensions
            .get::<CtxExtResult>()
            .ok_or(Error::CtxExt(CtxExtError::CtxNotInRequestExt))?
            .clone()
            .map_err(Error::CtxExt)
    }
}
