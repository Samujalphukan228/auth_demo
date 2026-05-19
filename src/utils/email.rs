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
        "to": [{ "email": to_email }],
        "subject": "Verify your email address",
        "htmlContent": format!(
            r#"
            <!DOCTYPE html>
            <html>
            <head>
                <meta charset="utf-8">
                <meta name="viewport" content="width=device-width, initial-scale=1.0">
            </head>
            <body style="margin: 0; padding: 0; background-color: #ffffff; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif;">
                <table width="100%" cellpadding="0" cellspacing="0" role="presentation" style="background-color: #ffffff;">
                    <tr>
                        <td align="center" style="padding: 40px 20px;">
                            <table width="100%" cellpadding="0" cellspacing="0" role="presentation" style="max-width: 420px;">
                                
                                <!-- Header -->
                                <tr>
                                    <td align="center" style="padding-bottom: 32px;">
                                        <h1 style="margin: 0; font-size: 28px; font-weight: 600; color: #111111;">Your Product</h1>
                                    </td>
                                </tr>
                                
                                <!-- Main Card -->
                                <tr>
                                    <td style="background-color: #ffffff; border: 1px solid #e5e5e5; border-radius: 12px; padding: 48px 40px; text-align: center;">
                                        <h2 style="margin: 0 0 16px 0; font-size: 24px; font-weight: 600; color: #111111;">Welcome</h2>
                                        <p style="margin: 0 0 32px 0; font-size: 17px; line-height: 1.5; color: #444444;">
                                            Click the button below to verify your email address.
                                        </p>
                                        
                                        <!-- Centered Button -->
                                        <table cellpadding="0" cellspacing="0" role="presentation" style="margin: 0 auto 32px auto;">
                                            <tr>
                                                <td align="center">
                                                    <a href="{url}" 
                                                       style="display: inline-block; 
                                                              background-color: #111111; 
                                                              color: #ffffff; 
                                                              font-size: 17px; 
                                                              font-weight: 600; 
                                                              padding: 14px 32px; 
                                                              border-radius: 8px; 
                                                              text-decoration: none;">
                                                        Verify Email
                                                    </a>
                                                </td>
                                            </tr>
                                        </table>
                                        
                                        <p style="margin: 0; font-size: 15px; line-height: 1.5; color: #666666;">
                                            This link expires in 15 minutes.<br>
                                            If you didn’t create an account, you can safely ignore this email.
                                        </p>
                                    </td>
                                </tr>
                                
                                <!-- Footer -->
                                <tr>
                                    <td align="center" style="padding-top: 32px;">
                                        <p style="margin: 0; font-size: 13px; color: #999999;">
                                            © Your Company. All rights reserved.
                                        </p>
                                    </td>
                                </tr>
                            </table>
                        </td>
                    </tr>
                </table>
            </body>
            </html>
            "#,
            url = verification_link
        )
    });

    send_email(client, api_key, body).await
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
        "to": [{ "email": to_email }],
        "subject": "Reset your password",
        "htmlContent": format!(
            r#"
            <!DOCTYPE html>
            <html>
            <head>
                <meta charset="utf-8">
                <meta name="viewport" content="width=device-width, initial-scale=1.0">
            </head>
            <body style="margin: 0; padding: 0; background-color: #ffffff; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif;">
                <table width="100%" cellpadding="0" cellspacing="0" role="presentation" style="background-color: #ffffff;">
                    <tr>
                        <td align="center" style="padding: 40px 20px;">
                            <table width="100%" cellpadding="0" cellspacing="0" role="presentation" style="max-width: 420px;">
                                
                                <!-- Header -->
                                <tr>
                                    <td align="center" style="padding-bottom: 32px;">
                                        <h1 style="margin: 0; font-size: 28px; font-weight: 600; color: #111111;">Your Product</h1>
                                    </td>
                                </tr>
                                
                                <!-- Main Card -->
                                <tr>
                                    <td style="background-color: #ffffff; border: 1px solid #e5e5e5; border-radius: 12px; padding: 48px 40px; text-align: center;">
                                        <h2 style="margin: 0 0 16px 0; font-size: 24px; font-weight: 600; color: #111111;">Password Reset</h2>
                                        <p style="margin: 0 0 32px 0; font-size: 17px; line-height: 1.5; color: #444444;">
                                            You requested to reset your password.<br>
                                            Click the button below to continue.
                                        </p>
                                        
                                        <!-- Centered Button -->
                                        <table cellpadding="0" cellspacing="0" role="presentation" style="margin: 0 auto 32px auto;">
                                            <tr>
                                                <td align="center">
                                                    <a href="{url}" 
                                                       style="display: inline-block; 
                                                              background-color: #111111; 
                                                              color: #ffffff; 
                                                              font-size: 17px; 
                                                              font-weight: 600; 
                                                              padding: 14px 32px; 
                                                              border-radius: 8px; 
                                                              text-decoration: none;">
                                                        Reset Password
                                                    </a>
                                                </td>
                                            </tr>
                                        </table>
                                        
                                        <p style="margin: 0; font-size: 15px; line-height: 1.5; color: #666666;">
                                            This link expires in 15 minutes.<br>
                                            If you didn’t request this, you can safely ignore this email.
                                        </p>
                                    </td>
                                </tr>
                                
                                <!-- Footer -->
                                <tr>
                                    <td align="center" style="padding-top: 32px;">
                                        <p style="margin: 0; font-size: 13px; color: #999999;">
                                            © Your Company. All rights reserved.
                                        </p>
                                    </td>
                                </tr>
                            </table>
                        </td>
                    </tr>
                </table>
            </body>
            </html>
            "#,
            url = reset_link
        )
    });

    send_email(client, api_key, body).await
}

// Helper function to reduce code duplication
async fn send_email(
    client: Client,
    api_key: &str,
    body: serde_json::Value,
) -> Result<(), AppError> {
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