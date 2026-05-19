use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;
use validator::ValidationErrors;

#[derive(Debug, Error)]
pub enum AppError {
    // 400
    #[error("Bad request: {0}")]
    BadRequest(String),

    // 401
    #[error("Unauthorized")]
    Unauthorized,

    // 403
    #[allow(dead_code)]
    #[error("Forbidden")]
    Forbidden,

    // 404
    #[error("Not found: {0}")]
    NotFound(String),

    // 409
    #[error("Conflict: {0}")]
    Conflict(String),

    // 429
    #[error("Too many requests")]
    TooManyRequests,

    // 500
    #[error("Internal server error")]
    InternalServerError,

    // 422
    #[error("Validation error: {0}")]
    ValidationError(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, self.to_string()),
            AppError::Forbidden => (StatusCode::FORBIDDEN, self.to_string()),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, msg.clone()),
            AppError::TooManyRequests => (StatusCode::TOO_MANY_REQUESTS, self.to_string()),
            AppError::InternalServerError => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            AppError::ValidationError(msg) => (StatusCode::UNPROCESSABLE_ENTITY, msg.clone()),
        };

        let body = Json(json!({
            "error": message,
            "status": status.as_u16(),
        }));

        (status, body).into_response()
    }
}

// convert sqlx errors into AppError
impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        match e {
            sqlx::Error::RowNotFound => AppError::NotFound("Resource not found".to_string()),
            _ => {
                tracing::error!("Database error: {e}");
                AppError::InternalServerError
            }
        }
    }
}

// convert redis errors into AppError
impl From<deadpool_redis::PoolError> for AppError {
    fn from(e: deadpool_redis::PoolError) -> Self {
        tracing::error!("Redis pool error: {e}");
        AppError::InternalServerError
    }
}

impl From<redis::RedisError> for AppError {
    fn from(e: redis::RedisError) -> Self {
        tracing::error!("Redis error: {e}");
        AppError::InternalServerError
    }
}

impl From<ValidationErrors> for AppError {
    fn from(e: ValidationErrors) -> Self {
        let messages: Vec<String> = e
            .field_errors()
            .into_iter()
            .flat_map(|(_, errors)| {
                errors.iter().map(|e| {
                    e.message
                        .clone()
                        .unwrap_or_default()
                        .to_string()
                })
            })
            .collect();

        AppError::ValidationError(messages.join(", "))
    }
}