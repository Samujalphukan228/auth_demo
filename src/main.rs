mod app;
mod config;
mod state;
mod errors;
mod db;
mod cache;
mod middleware;
mod utils;
mod routes;
mod features;

use tracing::info;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    info!("Starting auth_demo...");

    let (app, config) = app::create_app().await;

    let addr = format!("0.0.0.0:{}", config.port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .unwrap_or_else(|e| {
            eprintln!("Failed to bind port: {e}");
            std::process::exit(1);
        });

    info!("Server running on http://{}", addr);

    axum::serve(listener, app)
        .await
        .unwrap_or_else(|e| {
            eprintln!("Server error: {e}");
            std::process::exit(1);
        });
}