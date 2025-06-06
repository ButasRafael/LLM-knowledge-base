
use tower_cookies::{Cookie,Cookies};
use crate::error::{Result};
use crate::crypt::token::generate_token;

pub mod routes_login;
pub mod routes_task;
pub mod mw_auth;
pub mod routes_document;
pub mod routes_query_data;
pub mod routes_statistics;
pub mod routes_fine_tune;
pub mod routes_user;
pub mod routes_register;
pub mod routes_chat;

pub const AUTH_TOKEN:&str="auth-token";
fn set_token_cookie(cookies: &Cookies, user: &str, salt: &str) -> Result<()> {
    let token = generate_token(user, salt)?;

    let mut cookie = Cookie::new(AUTH_TOKEN, token.to_string());
    cookie.set_http_only(true);
    cookie.set_path("/");

    cookies.add(cookie);

    Ok(())
}

fn remove_token_cookie(cookies: &Cookies) -> Result<()> {
    let mut cookie = Cookie::from(AUTH_TOKEN);
    cookie.set_path("/");

    cookies.remove(cookie);

    Ok(())
}
