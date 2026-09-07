use anyhow::Result;
use axum_cookie::CookieManager;

pub fn extract_tokens(cookies: CookieManager) -> Result<(String, String)> {
    let access_token = match cookies.get("access_token") {
        Some(v) => v.to_string(),
        None => return Err(anyhow::Error::msg("access token not found")),
    };
    let refresh_token = match cookies.get("refresh_token") {
        Some(v) => v.to_string(),
        None => return Err(anyhow::Error::msg("refresh token not found")),
    };

    Ok((access_token, refresh_token))
}