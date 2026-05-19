use axum::{
    extract::{Query, State},
    http::HeaderMap,
    response::IntoResponse,
    Json,
};
use axum_extra::extract::CookieJar;
use axum_extra::extract::cookie::{Cookie, SameSite};
use serde::Deserialize;
use crate::errors::AppError;
use crate::features::auth::dto::{
    LoginRequest, RegisterRequest, SessionActionRequest,
    ForgotPasswordRequest, ResetPasswordRequest,
};
use crate::features::auth::service::AuthService;
use crate::middleware::auth_guard::AuthGuard;
use crate::state::AppState;
use serde_json::json;
use validator::Validate;

fn extract_ip(headers: &HeaderMap) -> String {
    headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string()
}

fn extract_device(headers: &HeaderMap) -> String {
    headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string()
}

#[derive(Deserialize)]
pub struct VerifyQuery {
    pub token: String,
}

// POST /auth/register
pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> impl IntoResponse {
    payload.validate()?;
    let message = AuthService::register(
        &state.db,
        &state.redis,
        &state.config,
        &payload.email,
        &payload.password,
    )
    .await?;

    Ok::<_, AppError>(Json(json!({ "message": message })))
}

// GET /auth/verify?token=
pub async fn verify_email(
    State(state): State<AppState>,
    Query(params): Query<VerifyQuery>,
) -> impl IntoResponse {
    AuthService::verify_email(&state.db, &state.redis, &params.token).await?;

    Ok::<_, AppError>(Json(json!({
        "message": "Email verified successfully. You can now log in."
    })))
}

// POST /auth/login
pub async fn login(
    State(state): State<AppState>,
    headers: HeaderMap,
    jar: CookieJar,
    Json(payload): Json<LoginRequest>,
) -> impl IntoResponse {
    payload.validate()?;
    let ip = extract_ip(&headers);
    let device = extract_device(&headers);

    let (access_token, refresh_token) = AuthService::login(
        &state.db,
        &state.redis,
        &state.config,
        &payload.email,
        &payload.password,
        &ip,
        &device,
    )
    .await?;

    // build HttpOnly cookies
    let access_cookie = Cookie::build(("access_token", access_token))
        .http_only(true)
        .same_site(SameSite::Strict)
        .max_age(time::Duration::seconds(900)) // 15 min
        .secure(state.config.cookie_secure)
        .path("/")
        .build();

    let refresh_cookie = Cookie::build(("refresh_token", refresh_token))
        .http_only(true)
        .same_site(SameSite::Strict)
        .max_age(time::Duration::seconds(60 * 60 * 24 * 30)) // 30 days
        .secure(state.config.cookie_secure)
        .path("/")
        .build();

    Ok::<_, AppError>((
        jar.add(access_cookie).add(refresh_cookie),
        Json(json!({ "message": "Logged in successfully." })),
    ))
}

// POST /auth/refresh
pub async fn refresh(
    State(state): State<AppState>,
    headers: HeaderMap,
    jar: CookieJar,
) -> impl IntoResponse {
    let refresh_token = jar
        .get("refresh_token")
        .map(|c| c.value().to_string())
        .ok_or(AppError::Unauthorized)?;

    let ip = extract_ip(&headers);
    let device = extract_device(&headers);

    let (new_access_token, new_refresh_token) =
        AuthService::refresh(&state.db, &state.config, &refresh_token, &ip, &device).await?;

    let access_cookie = Cookie::build(("access_token", new_access_token))
        .http_only(true)
        .same_site(SameSite::Strict)
        .max_age(time::Duration::seconds(900))
        .secure(state.config.cookie_secure)
        .path("/")
        .build();

    let refresh_cookie = Cookie::build(("refresh_token", new_refresh_token))
        .http_only(true)
        .same_site(SameSite::Strict)
        .max_age(time::Duration::seconds(60 * 60 * 24 * 30))
        .secure(state.config.cookie_secure)
        .path("/")
        .build();

    Ok::<_, AppError>((
        jar.add(access_cookie).add(refresh_cookie),
        Json(json!({ "message": "Token refreshed." })),
    ))
}

// POST /auth/logout
pub async fn logout(
    State(state): State<AppState>,
    auth: AuthGuard,
    jar: CookieJar,
) -> impl IntoResponse {
    let refresh_token = jar
        .get("refresh_token")
        .map(|c| c.value().to_string())
        .ok_or(AppError::Unauthorized)?;

    AuthService::logout(
        &state.db,
        &state.redis,
        &auth.access_token,
        &refresh_token,
        &auth.user_id,
    )
    .await?;

    // clear both cookies
    let jar = jar
        .remove(Cookie::from("access_token"))
        .remove(Cookie::from("refresh_token"));

    Ok::<_, AppError>((
        jar,
        Json(json!({ "message": "Logged out successfully." })),
    ))
}

// GET /auth/sessions
pub async fn get_sessions(
    State(state): State<AppState>,
    auth: AuthGuard,
    jar: CookieJar,
) -> impl IntoResponse {
    let refresh_token = jar
        .get("refresh_token")
        .map(|c| c.value().to_string())
        .ok_or(AppError::Unauthorized)?;

    let sessions = AuthService::get_sessions(
        &state.db,
        &auth.user_id,
        &refresh_token,
    )
    .await?;

    Ok::<_, AppError>(Json(sessions))
}

// DELETE /auth/sessions/:session_id
pub async fn logout_session(
    State(state): State<AppState>,
    auth: AuthGuard,
    axum::extract::Path(session_id): axum::extract::Path<String>,
    Json(payload): Json<SessionActionRequest>,
) -> impl IntoResponse {
    payload.validate()?;
    AuthService::logout_session(
        &state.db,
        &state.redis,
        &auth.user_id,
        &session_id,
        &payload.password,
    )
    .await?;

    Ok::<_, AppError>(Json(json!({
        "message": "Session logged out successfully."
    })))
}

// DELETE /auth/sessions
pub async fn logout_all(
    State(state): State<AppState>,
    auth: AuthGuard,
    jar: CookieJar,
    Json(payload): Json<SessionActionRequest>,
) -> impl IntoResponse {
    payload.validate()?;
    AuthService::logout_all(
        &state.db,
        &state.redis,
        &auth.user_id,
        &payload.password,
        &auth.access_token,
    )
    .await?;

    let jar = jar
        .remove(Cookie::from("access_token"))
        .remove(Cookie::from("refresh_token"));

    Ok::<_, AppError>((
        jar,
        Json(json!({ "message": "All sessions logged out successfully." })),
    ))
}

// GET /me
pub async fn me(
    State(state): State<AppState>,
    auth: AuthGuard,
) -> impl IntoResponse {
    let user = AuthService::me(&state.db, &auth.user_id).await?;
    Ok::<_, AppError>(Json(user))
}


// POST /auth/forgot-password
pub async fn forgot_password(
    State(state): State<AppState>,
    Json(payload): Json<ForgotPasswordRequest>,
) -> impl IntoResponse {
    payload.validate()?;
    AuthService::forgot_password(
        &state.db,
        &state.redis,
        &state.config,
        &payload.email,
    )
    .await?;

    Ok::<_, AppError>(Json(json!({
        "message": "If that email exists you will receive a reset link shortly."
    })))
}

// GET /auth/reset-password?token=
pub async fn validate_reset_token(
    State(state): State<AppState>,
    Query(params): Query<VerifyQuery>,
) -> impl IntoResponse {
    AuthService::validate_reset_token(&state.redis, &params.token).await?;

    Ok::<_, AppError>(Json(json!({
        "message": "Token is valid. You may now reset your password."
    })))
}

// POST /auth/reset-password
pub async fn reset_password(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(payload): Json<ResetPasswordRequest>,
) -> impl IntoResponse {
    payload.validate()?;
    AuthService::reset_password(
        &state.db,
        &state.redis,
        &payload.token,
        &payload.new_password,
    )
    .await?;

    // clear cookies — user must login again
    let jar = jar
        .remove(Cookie::from("access_token"))
        .remove(Cookie::from("refresh_token"));

    Ok::<_, AppError>((
        jar,
        Json(json!({
            "message": "Password reset successfully. Please log in again."
        })),
    ))
}