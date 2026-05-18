use axum::{
    extract::FromRequestParts,
    http::request::Parts,
};
use axum_extra::extract::CookieJar;
use redis::AsyncCommands;
use crate::errors::AppError;
use crate::state::AppState;
use crate::utils::jwt;

pub struct AuthGuard {
    pub user_id: String,
    pub access_token: String,
}

impl FromRequestParts<AppState> for AuthGuard {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        // read access token from cookie
        let jar = CookieJar::from_request_parts(parts, state).await.unwrap();

        let token = jar
            .get("access_token")
            .map(|c| c.value().to_string())
            .ok_or(AppError::Unauthorized)?;

        // verify JWT
        let user_id = jwt::verify_access_token(&token, &state.config.jwt_secret)?;

        // check blocklist
        let blocklist_key = format!("blocklist:{}", token);
        let mut conn = state.redis.get().await?;
        let blacklisted: Option<String> = conn.get(&blocklist_key).await?;
        if blacklisted.is_some() {
            return Err(AppError::Unauthorized);
        }

        Ok(AuthGuard { user_id, access_token: token })
    }
}