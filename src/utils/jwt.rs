use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use crate::errors::AppError;

// claims inside the access token
#[derive(Debug, Serialize, Deserialize)]
pub struct AccessClaims {
    pub sub: String,    // user id
    pub exp: usize,     // expiry timestamp
    pub iat: usize,     // issued at timestamp
}

// claims inside the refresh token
#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshClaims {
    pub sub: String,    // user id
    pub exp: usize,     // expiry timestamp
    pub iat: usize,     // issued at timestamp
}

// generate access token — expires in 15 minutes
pub fn generate_access_token(user_id: &str, secret: &str) -> Result<String, AppError> {
    let now = chrono::Utc::now().timestamp() as usize;
    let exp = now + 60 * 15; // 15 minutes

    let claims = AccessClaims {
        sub: user_id.to_string(),
        exp,
        iat: now,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| {
        tracing::error!("Failed to generate access token: {e}");
        AppError::InternalServerError
    })
}

// generate refresh token — expires in 30 days
pub fn generate_refresh_token(user_id: &str, secret: &str) -> Result<String, AppError> {
    let now = chrono::Utc::now().timestamp() as usize;
    let exp = now + 60 * 60 * 24 * 30; // 30 days

    let claims = RefreshClaims {
        sub: user_id.to_string(),
        exp,
        iat: now,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| {
        tracing::error!("Failed to generate refresh token: {e}");
        AppError::InternalServerError
    })
}

// verify and decode access token → returns user id
pub fn verify_access_token(token: &str, secret: &str) -> Result<String, AppError> {
    let token_data = decode::<AccessClaims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|e| {
        tracing::error!("Invalid access token: {e}");
        AppError::Unauthorized
    })?;

    Ok(token_data.claims.sub)
}

// verify and decode refresh token → returns user id
pub fn verify_refresh_token(token: &str, secret: &str) -> Result<String, AppError> {
    let token_data = decode::<RefreshClaims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|e| {
        tracing::error!("Invalid refresh token: {e}");
        AppError::Unauthorized
    })?;

    Ok(token_data.claims.sub)
}