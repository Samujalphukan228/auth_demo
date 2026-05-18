use serde::{Deserialize, Serialize};
use chrono::NaiveDateTime;

// POST /auth/register
#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
}

// POST /auth/login
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

// DELETE /auth/sessions/:id and DELETE /auth/sessions
#[derive(Debug, Deserialize)]
pub struct SessionActionRequest {
    pub password: String,
}

// single session info
#[derive(Debug, Serialize)]
pub struct SessionResponse {
    pub id: String,
    pub device: Option<String>,
    pub ip: Option<String>,
    pub created_at: NaiveDateTime,
    pub is_current: bool,
}

// response for /me
#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: String,
    pub email: String,
    pub is_verified: bool,
}

// POST /auth/forgot-password
#[derive(Debug, Deserialize)]
pub struct ForgotPasswordRequest {
    pub email: String,
}

// POST /auth/reset-password
#[derive(Debug, Deserialize)]
pub struct ResetPasswordRequest {
    pub token: String,
    pub new_password: String,
}