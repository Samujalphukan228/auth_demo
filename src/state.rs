use sqlx::MySqlPool;
use deadpool_redis::Pool as RedisPool;
use crate::config::AppConfig;

#[derive(Clone)]
pub struct AppState {
    pub db: MySqlPool,
    pub redis: RedisPool,
    pub config: AppConfig,
}

impl AppState {
    pub fn new(db: MySqlPool, redis: RedisPool, config: AppConfig) -> Self {
        Self { db, redis, config }
    }
}