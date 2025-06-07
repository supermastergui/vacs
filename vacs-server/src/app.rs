use crate::config;
use crate::state::AppState;
use crate::ws::ws_handler;
use axum::Router;
use axum::routing::any;
use std::sync::Arc;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::{DefaultMakeSpan, TraceLayer};

pub fn create_app() -> Router<Arc<AppState>> {
    Router::new().route("/ws", any(ws_handler)).layer((
        TraceLayer::new_for_http().make_span_with(DefaultMakeSpan::default().include_headers(true)),
        TimeoutLayer::new(config::SERVER_SHUTDOWN_TIMEOUT),
    ))
}
