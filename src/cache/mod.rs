use deadpool_redis::{Config, Pool,Runtime};
use std::time::Duration;

#[allow(dead_code)]
fn secs(n: u64) -> Duration {
    Duration::from_secs(n)
}

pub fn connect(redis_url: &str) -> Pool {
    Config::from_url(redis_url)
        .builder()
        .unwrap_or_else(|e| {
            eprintln!("Failed to build Redis pool: {e}");
            std::process::exit(1);
        })
        .max_size(10)
        .wait_timeout(Some(secs(5)))
        .recycle_timeout(Some(secs(5)))
        .runtime(Runtime::Tokio1)
        .build()
        .unwrap_or_else(|e| {
            eprintln!("Failed to create Redis pool: {e}");
            std::process::exit(1);
        })
}