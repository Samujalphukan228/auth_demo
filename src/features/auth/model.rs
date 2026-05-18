use serde::{Deserialize, Serialize};
use chrono::NaiveDateTime;

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: String,
    pub email: String,
    pub password_hash: String,
    pub is_verified: i8,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}