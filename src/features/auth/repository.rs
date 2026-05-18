use sqlx::MySqlPool;
use chrono::NaiveDateTime;
use crate::errors::AppError;

#[derive(Debug, sqlx::FromRow)]
pub struct UserLoginRow {
    pub id: String,
    pub password_hash: String,
    pub is_verified: i8,
}

#[derive(Debug, sqlx::FromRow)]
pub struct UserProfileRow {
    pub id: String,
    pub email: String,
    pub is_verified: i8,
}

#[derive(Debug, sqlx::FromRow)]
pub struct SessionRow {
    pub id: String,
    pub device: Option<String>,
    pub ip: Option<String>,
    pub created_at: NaiveDateTime,
    pub token_hash: String,
}

pub struct AuthRepository;

impl AuthRepository {
    // insert a new user
    pub async fn create_user(
        db: &MySqlPool,
        id: &str,
        email: &str,
        password_hash: &str,
    ) -> Result<(), AppError> {
        sqlx::query!(
            "INSERT INTO users (id, email, password_hash) VALUES (?, ?, ?)",
            id,
            email,
            password_hash
        )
        .execute(db)
        .await?;

        Ok(())
    }

    // delete user by id — used for rollback if email fails
    pub async fn delete_user(
        db: &MySqlPool,
        id: &str,
    ) -> Result<(), AppError> {
        sqlx::query!(
            "DELETE FROM users WHERE id = ?",
            id
        )
        .execute(db)
        .await?;

        Ok(())
    }

    // check email and return (id, is_verified) if exists
    pub async fn find_email_status(
        db: &MySqlPool,
        email: &str,
    ) -> Result<Option<(String, i8)>, AppError> {
        let row = sqlx::query!(
            "SELECT id, is_verified FROM users WHERE email = ?",
            email
        )
        .fetch_optional(db)
        .await?;

        Ok(row.map(|r| (r.id, r.is_verified)))
    }

    // login — only fetch what's needed to authenticate
    pub async fn find_login_data(
        db: &MySqlPool,
        email: &str,
    ) -> Result<Option<UserLoginRow>, AppError> {
        let row = sqlx::query_as!(
            UserLoginRow,
            "SELECT id, password_hash, is_verified FROM users WHERE email = ?",
            email
        )
        .fetch_optional(db)
        .await?;

        Ok(row)
    }

    // /me — only fetch profile fields, never password hash
    pub async fn find_profile(
        db: &MySqlPool,
        id: &str,
    ) -> Result<Option<UserProfileRow>, AppError> {
        let row = sqlx::query_as!(
            UserProfileRow,
            "SELECT id, email, is_verified FROM users WHERE id = ?",
            id
        )
        .fetch_optional(db)
        .await?;

        Ok(row)
    }

    // mark user as verified
    pub async fn verify_user(
        db: &MySqlPool,
        id: &str,
    ) -> Result<(), AppError> {
        sqlx::query!(
            "UPDATE users SET is_verified = TRUE WHERE id = ?",
            id
        )
        .execute(db)
        .await?;

        Ok(())
    }

    // get all active sessions for a user
    pub async fn get_sessions(
        db: &MySqlPool,
        user_id: &str,
    ) -> Result<Vec<SessionRow>, AppError> {
        let rows = sqlx::query_as!(
            SessionRow,
            "SELECT id, device, ip, created_at, token_hash
             FROM refresh_tokens
             WHERE user_id = ? AND expires_at > NOW()
             ORDER BY created_at DESC",
            user_id
        )
        .fetch_all(db)
        .await?;

        Ok(rows)
    }

    // delete a specific session by id and user_id
    pub async fn delete_session(
        db: &MySqlPool,
        session_id: &str,
        user_id: &str,
    ) -> Result<bool, AppError> {
        let result = sqlx::query!(
            "DELETE FROM refresh_tokens WHERE id = ? AND user_id = ?",
            session_id,
            user_id
        )
        .execute(db)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    // delete all sessions for a user
    pub async fn delete_all_sessions(
        db: &MySqlPool,
        user_id: &str,
    ) -> Result<(), AppError> {
        sqlx::query!(
            "DELETE FROM refresh_tokens WHERE user_id = ?",
            user_id
        )
        .execute(db)
        .await?;

        Ok(())
    }

    // find password hash for password confirmation
    pub async fn find_password_hash(
        db: &MySqlPool,
        user_id: &str,
    ) -> Result<Option<String>, AppError> {
        let row = sqlx::query_scalar!(
            "SELECT password_hash FROM users WHERE id = ?",
            user_id
        )
        .fetch_optional(db)
        .await?;

        Ok(row)
    }

    // find user id by email — for forgot password
    pub async fn find_id_by_email(
        db: &MySqlPool,
        email: &str,
    ) -> Result<Option<String>, AppError> {
        let row = sqlx::query_scalar!(
            "SELECT id FROM users WHERE email = ?",
            email
        )
        .fetch_optional(db)
        .await?;

        Ok(row)
    }

    // update password
    pub async fn update_password(
        db: &MySqlPool,
        user_id: &str,
        password_hash: &str,
    ) -> Result<(), AppError> {
        sqlx::query!(
            "UPDATE users SET password_hash = ? WHERE id = ?",
            password_hash,
            user_id
        )
        .execute(db)
        .await?;

        Ok(())
    }
}