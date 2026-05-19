use axum::Router;
use http::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use http::Method;
use tower_http::cors::CorsLayer;
use tower_http::set_header::SetResponseHeaderLayer;
use tower_http::trace::TraceLayer;
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
use http::{HeaderValue, header};
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

    let cors = CorsLayer::new()
        .allow_origin(config.allowed_origin.parse::<HeaderValue>().unwrap_or(HeaderValue::from_static("*")))
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
        .allow_headers([CONTENT_TYPE, AUTHORIZATION, ACCEPT])
        .allow_credentials(true);

    let router = crate::routes::init(state)
        // assign unique request id to every request
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
        // propagate request id to response headers
        .layer(PropagateRequestIdLayer::x_request_id())
        // log every request
        .layer(TraceLayer::new_for_http())
        // cors
        .layer(cors)
        // security headers
        .layer(SetResponseHeaderLayer::if_not_present(
            header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::X_FRAME_OPTIONS,
            HeaderValue::from_static("DENY"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::HeaderName::from_static("x-xss-protection"),
            HeaderValue::from_static("1; mode=block"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::REFERRER_POLICY,
            HeaderValue::from_static("strict-origin-when-cross-origin"),
        ));

    (router, config)
}