use anyhow::Result;
use axum_cookie::{CookieManager, cookie::{Cookie, cookie::SameSite}};

use crate::config::AuthConfig;

const ACCESS_TOKEN_KEY: &str = "access_token";
const REFRESH_TOKEN_KEY: &str = "refresh_token";

pub fn extract_tokens(cookies: &CookieManager) -> Result<(String, String)> {
    let access_token = match cookies.get(ACCESS_TOKEN_KEY) {
        Some(v) => v.value().to_string(),
        None => return Err(anyhow::Error::msg("access token not found")),
    };
    let refresh_token = match cookies.get(REFRESH_TOKEN_KEY) {
        Some(v) => v.value().to_string(),
        None => return Err(anyhow::Error::msg("refresh token not found")),
    };

    Ok((access_token, refresh_token))
}

pub fn add_token_cookies(
    cookies: &CookieManager,
    (access_token, refresh_token): (String, String),
    auth_conf: &AuthConfig,
) {
    let cookie = Cookie::builder(ACCESS_TOKEN_KEY, access_token)
        .path("/")
        .max_age(auth_conf.access_token_expiration_time)
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Strict)
        .build();
    cookies.add(cookie);

    let cookie = Cookie::builder(REFRESH_TOKEN_KEY, refresh_token)
        .path("/")
        .max_age(auth_conf.refresh_token_expiration_time)
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Strict)
        .build();
    cookies.add(cookie);
}

pub fn remove_token_cookies(cookies: &CookieManager) {
    cookies.remove(ACCESS_TOKEN_KEY);
    cookies.remove(REFRESH_TOKEN_KEY);
}