use deadpool_redis::Pool as RedisPool;
use redis::AsyncCommands;
use sqlx::MySqlPool;
use uuid::Uuid;
use crate::errors::AppError;
use crate::features::auth::dto::{SessionResponse, UserResponse};
use crate::features::auth::repository::AuthRepository;
use crate::utils::{hash, jwt, email};
use crate::config::AppConfig;

pub struct AuthService;

impl AuthService {
    // register a new user
    pub async fn register(
    db: &MySqlPool,
    redis: &RedisPool,
    config: &AppConfig,
    user_email: &str,
    password: &str,
) -> Result<String, AppError> {
    let user_email = crate::utils::normalize_email(user_email); // ← add this
    let user_email = user_email.as_str();  

    let existing = AuthRepository::find_email_status(db, user_email).await?;

    match existing {
        // email exists and is verified → reject
        Some((_, 1)) => {
            return Err(AppError::Conflict("Email already registered".to_string()));
        }

        // email exists but unverified → resend verification email automatically
        Some((user_id, _)) => {
            // rate limit resend — 2 emails per 10 min per email
            let resend_key = format!("resend_verify:{}", user_email);
            let mut conn = redis.get().await?;
            let attempts: i64 = conn.incr(&resend_key, 1).await?;
            if attempts == 1 {
                conn.expire::<_, ()>(&resend_key, 600).await?; // 10 min
            }
            if attempts > 2 {
                return Err(AppError::TooManyRequests);
            }

            let token = Uuid::new_v4().to_string();
            let redis_key = format!("verify:{}", token);
            conn.set_ex::<_, _, ()>(&redis_key, &user_id, 900).await?;

            let link = format!("{}/auth/verify?token={}", config.app_url, token);
            email::send_verification_email(
                &config.brevo_api_key,
                &config.brevo_sender_email,
                &config.brevo_sender_name,
                user_email,
                &link,
            )
            .await?;

            return Ok("Verification email resent. Please check your inbox.".to_string());
        }

        // email does not exist → create user
        None => {
            let password_hash = hash::hash_password(password)?;
            let user_id = Uuid::new_v4().to_string();
            AuthRepository::create_user(db, &user_id, user_email, &password_hash).await?;

            let token = Uuid::new_v4().to_string();
            let redis_key = format!("verify:{}", token);
            let mut conn = redis.get().await?;
            conn.set_ex::<_, _, ()>(&redis_key, &user_id, 900).await?;

            let link = format!("{}/auth/verify?token={}", config.app_url, token);

            // if email fails → rollback user creation
            if let Err(e) = email::send_verification_email(
                &config.brevo_api_key,
                &config.brevo_sender_email,
                &config.brevo_sender_name,
                user_email,
                &link,
            )
            .await
            {
                AuthRepository::delete_user(db, &user_id).await?;
                return Err(e);
            }

            return Ok(
                "Registration successful. Please check your email to verify your account."
                    .to_string(),
            );
        }
    }
}

    // verify email
    pub async fn verify_email(
        db: &MySqlPool,
        redis: &RedisPool,
        token: &str,
    ) -> Result<(), AppError> {
        let redis_key = format!("verify:{}", token);
        let mut conn = redis.get().await?;

        let user_id: Option<String> = conn.get(&redis_key).await?;
        let user_id = user_id.ok_or(AppError::BadRequest(
            "Invalid or expired verification link".to_string(),
        ))?;

        AuthRepository::verify_user(db, &user_id).await?;
        conn.del::<_, ()>(&redis_key).await?;

        Ok(())
    }

    // login
    pub async fn login(
        db: &MySqlPool,
        redis: &RedisPool,
        config: &AppConfig,
        user_email: &str,
        password: &str,
        ip: &str,
        device: &str,
    ) -> Result<(String, String), AppError> {
        let user_email = crate::utils::normalize_email(user_email);
        let user_email = user_email.as_str();

        let rate_key = format!("login_attempts:{}", ip);
        let mut conn = redis.get().await?;
        let attempts: i64 = conn.incr(&rate_key, 1).await?;
        if attempts == 1 {
            conn.expire::<_, ()>(&rate_key, 900).await?;
        }
        if attempts > 5 {
            return Err(AppError::TooManyRequests);
        }

        let user = AuthRepository::find_login_data(db, user_email)
            .await?
            .ok_or(AppError::Unauthorized)?;

        if !hash::verify_password(password, &user.password_hash)? {
            return Err(AppError::Unauthorized);
        }

        if user.is_verified == 0 {
            return Err(AppError::BadRequest("Email not verified".to_string()));
        }

        let access_token = jwt::generate_access_token(&user.id, &config.jwt_secret)?;
        let refresh_token = jwt::generate_refresh_token(&user.id, &config.jwt_refresh_secret)?;

        let token_id = Uuid::new_v4().to_string();
        let token_hash = format!("{:x}", md5::compute(&refresh_token));
        sqlx::query!(
            "INSERT INTO refresh_tokens (id, user_id, token_hash, device, ip, expires_at)
             VALUES (?, ?, ?, ?, ?, DATE_ADD(NOW(), INTERVAL 30 DAY))",
            token_id,
            user.id,
            token_hash,
            device,
            ip
        )
        .execute(db)
        .await?;

        conn.del::<_, ()>(&rate_key).await?;

        Ok((access_token, refresh_token))
    }

    // refresh
    pub async fn refresh(
        db: &MySqlPool,
        config: &AppConfig,
        refresh_token: &str,
    ) -> Result<(String, String), AppError> {
        let user_id = jwt::verify_refresh_token(refresh_token, &config.jwt_refresh_secret)?;

        let token_hash = format!("{:x}", md5::compute(refresh_token));
        let exists = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM refresh_tokens
             WHERE token_hash = ? AND user_id = ? AND expires_at > NOW()",
            token_hash,
            user_id
        )
        .fetch_one(db)
        .await?;

        if exists == 0 {
            return Err(AppError::Unauthorized);
        }

        let access_token = jwt::generate_access_token(&user_id, &config.jwt_secret)?;
        Ok((access_token, refresh_token.to_string()))
    }

    // logout current device
    pub async fn logout(
        db: &MySqlPool,
        redis: &RedisPool,
        access_token: &str,
        refresh_token: &str,
        user_id: &str,
    ) -> Result<(), AppError> {
        let blocklist_key = format!("blocklist:{}", access_token);
        let mut conn = redis.get().await?;
        conn.set_ex::<_, _, ()>(&blocklist_key, "1", 900).await?;

        let token_hash = format!("{:x}", md5::compute(refresh_token));
        sqlx::query!(
            "DELETE FROM refresh_tokens WHERE token_hash = ? AND user_id = ?",
            token_hash,
            user_id
        )
        .execute(db)
        .await?;

        Ok(())
    }

    // get all active sessions
    pub async fn get_sessions(
        db: &MySqlPool,
        user_id: &str,
        current_refresh_token: &str,
    ) -> Result<Vec<SessionResponse>, AppError> {
        let current_hash = format!("{:x}", md5::compute(current_refresh_token));
        let sessions = AuthRepository::get_sessions(db, user_id).await?;

        let response = sessions
            .into_iter()
            .map(|s| SessionResponse {
                id: s.id,
                device: s.device,
                ip: s.ip,
                created_at: s.created_at,
                is_current: s.token_hash == current_hash,
            })
            .collect();

        Ok(response)
    }

    // logout specific session — requires password
    pub async fn logout_session(
        db: &MySqlPool,
        _redis: &RedisPool,
        user_id: &str,
        session_id: &str,
        password: &str,
    ) -> Result<(), AppError> {
        let password_hash = AuthRepository::find_password_hash(db, user_id)
            .await?
            .ok_or(AppError::Unauthorized)?;

        if !hash::verify_password(password, &password_hash)? {
            return Err(AppError::Unauthorized);
        }

        let deleted = AuthRepository::delete_session(db, session_id, user_id).await?;
        if !deleted {
            return Err(AppError::NotFound("Session not found".to_string()));
        }

        Ok(())
    }

    // logout all sessions — requires password
    pub async fn logout_all(
        db: &MySqlPool,
        redis: &RedisPool,
        user_id: &str,
        password: &str,
        current_access_token: &str,
    ) -> Result<(), AppError> {
        let password_hash = AuthRepository::find_password_hash(db, user_id)
            .await?
            .ok_or(AppError::Unauthorized)?;

        if !hash::verify_password(password, &password_hash)? {
            return Err(AppError::Unauthorized);
        }

        let blocklist_key = format!("blocklist:{}", current_access_token);
        let mut conn = redis.get().await?;
        conn.set_ex::<_, _, ()>(&blocklist_key, "1", 900).await?;

        AuthRepository::delete_all_sessions(db, user_id).await?;

        Ok(())
    }

    // get user profile
    pub async fn me(
        db: &MySqlPool,
        user_id: &str,
    ) -> Result<UserResponse, AppError> {
        let user = AuthRepository::find_profile(db, user_id)
            .await?
            .ok_or(AppError::NotFound("User not found".to_string()))?;

        Ok(UserResponse {
            id: user.id,
            email: user.email,
            is_verified: user.is_verified == 1,
        })
    }

    // forgot password — send reset email
    pub async fn forgot_password(
    db: &MySqlPool,
    redis: &RedisPool,
    config: &AppConfig,
    user_email: &str,
) -> Result<(), AppError> {
    let user_email = crate::utils::normalize_email(user_email);
    let user_email = user_email.as_str();

    // rate limit — 2 reset emails per 10 min per email
    let rate_key = format!("forgot_password:{}", user_email);
    let mut conn = redis.get().await?;
    let attempts: i64 = conn.incr(&rate_key, 1).await?;
    if attempts == 1 {
        conn.expire::<_, ()>(&rate_key, 600).await?; // 10 min
    }
    if attempts > 2 {
        // still return success — never reveal if email exists
        return Ok(());
    }

    let user_id = AuthRepository::find_id_by_email(db, user_email).await?;

    if let Some(user_id) = user_id {
        let token = Uuid::new_v4().to_string();
        let redis_key = format!("reset:{}", token);
        conn.set_ex::<_, _, ()>(&redis_key, &user_id, 900).await?;

        let link = format!("{}/auth/reset-password?token={}", config.app_url, token);
        email::send_reset_email(
            &config.brevo_api_key,
            &config.brevo_sender_email,
            &config.brevo_sender_name,
            user_email,
            &link,
        )
        .await?;
    }

    Ok(())
}

    // validate reset token
    pub async fn validate_reset_token(
        redis: &RedisPool,
        token: &str,
    ) -> Result<(), AppError> {
        let redis_key = format!("reset:{}", token);
        let mut conn = redis.get().await?;

        let exists: Option<String> = conn.get(&redis_key).await?;
        exists.ok_or(AppError::BadRequest(
            "Invalid or expired reset link".to_string(),
        ))?;

        Ok(())
    }

    // reset password
    pub async fn reset_password(
        db: &MySqlPool,
        redis: &RedisPool,
        token: &str,
        new_password: &str,
    ) -> Result<(), AppError> {
        let redis_key = format!("reset:{}", token);
        let mut conn = redis.get().await?;

        let user_id: Option<String> = conn.get(&redis_key).await?;
        let user_id = user_id.ok_or(AppError::BadRequest(
            "Invalid or expired reset link".to_string(),
        ))?;

        let password_hash = hash::hash_password(new_password)?;
        AuthRepository::update_password(db, &user_id, &password_hash).await?;
        conn.del::<_, ()>(&redis_key).await?;
        AuthRepository::delete_all_sessions(db, &user_id).await?;

        Ok(())
    }
}