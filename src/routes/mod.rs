use axum::{Router, routing::{get, post, delete}};
use crate::state::AppState;
use crate::features::auth::handler;

pub fn init(state: AppState) -> Router {
    let auth_routes = Router::new()
        .route("/register", post(handler::register))
        .route("/verify", get(handler::verify_email))
        .route("/login", post(handler::login))
        .route("/refresh", post(handler::refresh))
        .route("/logout", post(handler::logout))
        .route("/sessions", get(handler::get_sessions))
        .route("/sessions", delete(handler::logout_all))
        .route("/sessions/{session_id}", delete(handler::logout_session))
        .route("/forgot-password", post(handler::forgot_password))
        .route("/reset-password", get(handler::validate_reset_token))
        .route("/reset-password", post(handler::reset_password));

    Router::new()
        .nest("/auth", auth_routes)
        .route("/me", get(handler::me))
        .with_state(state)
}