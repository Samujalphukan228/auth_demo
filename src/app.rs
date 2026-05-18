use axum::Router;
use tracing::info;
use crate::config::AppConfig;
use crate::state::AppState;
use ezconfig_rs::Config;

pub async fn create_app() -> (Router, AppConfig) {
    let config = AppConfig::load().unwrap_or_else(|e| {
        eprintln!("Config error: {e}");
        std::process::exit(1);
    });

    let db_pool = crate::db::connect(&config.database_url).await;
    info!("Connected to MariaDB ✓");

    let redis_pool = crate::cache::connect(&config.redis_url);
    info!("Connected to Redis ✓");

    let state = AppState::new(db_pool, redis_pool, config.clone());

    let router = crate::routes::init(state);

    (router, config)
}