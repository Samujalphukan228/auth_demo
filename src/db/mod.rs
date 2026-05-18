use sqlx::{MySqlPool, mysql::MySqlPoolOptions};
use std::time::Duration;

#[allow(dead_code)]
fn secs(n: u64) -> Duration {
    Duration::from_secs(n)
}

#[allow(dead_code)]
fn mins(n: u64) -> Duration {
    Duration::from_secs(n * 60)
}

pub async fn connect(database_url: &str) -> MySqlPool {
    MySqlPoolOptions::new()
        .max_connections(10)
        .min_connections(2)
        .acquire_timeout(secs(5))
        .idle_timeout(mins(10))
        .max_lifetime(mins(30))
        .connect(database_url)
        .await
        .unwrap_or_else(|e| {
            eprintln!("Failed to connect to database: {e}");
            std::process::exit(1);
        })
}