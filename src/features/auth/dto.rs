use serde::{Deserialize, Serialize};
use chrono::NaiveDateTime;
use validator::Validate;

// POST /auth/register
#[derive(Debug, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(email(message = "Invalid email address"))]
    pub email: String,

    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub password: String,
}

// POST /auth/login
#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email(message = "Invalid email address"))]
    pub email: String,

    #[validate(length(min = 1, message = "Password is required"))]
    pub password: String,
}

// POST /auth/forgot-password
#[derive(Debug, Deserialize, Validate)]
pub struct ForgotPasswordRequest {
    #[validate(email(message = "Invalid email address"))]
    pub email: String,
}

// POST /auth/reset-password
#[derive(Debug, Deserialize, Validate)]
pub struct ResetPasswordRequest {
    #[validate(length(min = 1, message = "Token is required"))]
    pub token: String,

    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub new_password: String,
}

// DELETE /auth/sessions/:id and DELETE /auth/sessions
#[derive(Debug, Deserialize, Validate)]
pub struct SessionActionRequest {
    #[validate(length(min = 1, message = "Password is required"))]
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