use crate::config;
use crate::state::AppState;
use crate::ws::ws_handler;
use axum::routing::any;
use axum::Router;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::{DefaultMakeSpan, TraceLayer};

pub fn create_app() -> Router<Arc<AppState>> {
    Router::new().route("/ws", any(ws_handler)).layer((
        TraceLayer::new_for_http().make_span_with(DefaultMakeSpan::default().include_headers(true)),
        TimeoutLayer::new(config::SERVER_SHUTDOWN_TIMEOUT),
    ))
}

pub async fn serve(listener: TcpListener, app: Router<Arc<AppState>>, state: Arc<AppState>) {
    axum::serve(
        listener,
        app.with_state(state)
            .into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap()
}
