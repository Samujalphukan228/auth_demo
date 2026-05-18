use reqwest::Client;
use serde_json::json;
use crate::errors::AppError;

pub async fn send_verification_email(
    api_key: &str,
    sender_email: &str,
    sender_name: &str,
    to_email: &str,
    verification_link: &str,
) -> Result<(), AppError> {
    let client = Client::new();

    let body = json!({
        "sender": {
            "name": sender_name,
            "email": sender_email
        },
        "to": [
            { "email": to_email }
        ],
        "subject": "Verify your email",
        "htmlContent": format!(
            "<h2>Welcome!</h2>
             <p>Click the link below to verify your email address.</p>
             <a href='{url}' style='
                display: inline-block;
                padding: 12px 24px;
                background-color: #4F46E5;
                color: white;
                text-decoration: none;
                border-radius: 6px;
                font-weight: bold;
             '>Verify Email</a>
             <p>This link expires in 15 minutes.</p>
             <p>If you did not create an account, ignore this email.</p>",
            url = verification_link
        )
    });

    let response = client
        .post("https://api.brevo.com/v3/smtp/email")
        .header("api-key", api_key)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| {
            tracing::error!("Failed to send email: {e}");
            AppError::InternalServerError
        })?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        tracing::error!("Brevo API error {status}: {text}");
        return Err(AppError::InternalServerError);
    }

    Ok(())
}


pub async fn send_reset_email(
    api_key: &str,
    sender_email: &str,
    sender_name: &str,
    to_email: &str,
    reset_link: &str,
) -> Result<(), AppError> {
    let client = Client::new();

    let body = json!({
        "sender": {
            "name": sender_name,
            "email": sender_email
        },
        "to": [
            { "email": to_email }
        ],
        "subject": "Reset your password",
        "htmlContent": format!(
            "<h2>Password Reset</h2>
             <p>You requested a password reset. Click the link below to set a new password.</p>
             <a href='{url}' style='
                display: inline-block;
                padding: 12px 24px;
                background-color: #E53E3E;
                color: white;
                text-decoration: none;
                border-radius: 6px;
                font-weight: bold;
             '>Reset Password</a>
             <p>This link expires in 15 minutes.</p>
             <p>If you did not request a password reset, ignore this email.</p>",
            url = reset_link
        )
    });

    let response = client
        .post("https://api.brevo.com/v3/smtp/email")
        .header("api-key", api_key)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| {
            tracing::error!("Failed to send reset email: {e}");
            AppError::InternalServerError
        })?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        tracing::error!("Brevo API error {status}: {text}");
        return Err(AppError::InternalServerError);
    }

    Ok(())
}