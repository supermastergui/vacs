mod auth;
mod root;
mod ws;

use crate::state::AppState;
use axum::Router;
use axum_login::{AuthManagerLayer, AuthnBackend};
use std::sync::Arc;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
use tower_sessions::SessionStore;
use tower_sessions::service::SignedCookie;

pub fn create_app<B, S>(auth_layer: AuthManagerLayer<B, S, SignedCookie>) -> Router<Arc<AppState>>
where
    B: AuthnBackend + Send + Sync + 'static + Clone,
    S: SessionStore + Send + Sync + 'static + Clone,
{
    Router::new()
        .nest("/auth", auth::routes())
        .nest("/ws", ws::routes().merge(crate::ws::routes()))
        .merge(root::routes())
        .layer(TraceLayer::new_for_http().make_span_with(DefaultMakeSpan::default()))
        .layer(TimeoutLayer::new(crate::config::SERVER_SHUTDOWN_TIMEOUT))
        .layer(auth_layer)
}
