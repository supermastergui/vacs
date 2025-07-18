use crate::config::AuthConfig;
use crate::ws::message::{receive_message, send_message, MessageResult};
use axum::extract::ws;
use axum::extract::ws::WebSocket;
use futures_util::stream::{SplitSink, SplitStream};
use std::sync::Arc;
use std::time::Duration;
use tracing::instrument;
use vacs_protocol::{LoginFailureReason, SignalingMessage};
use vacs_vatsim::user::UserService;

#[instrument(level = "debug", skip_all)]
pub async fn handle_login(
    auth_config: &AuthConfig,
    vatsim_user_service: Arc<dyn UserService>,
    websocket_receiver: &mut SplitStream<WebSocket>,
    websocket_sender: &mut SplitSink<WebSocket, ws::Message>,
) -> Option<String> {
    tracing::trace!("Handling login flow");
    match tokio::time::timeout(Duration::from_millis(auth_config.login_flow_timeout_millis), async {
        loop {
            return match receive_message(websocket_receiver).await {
                MessageResult::ApplicationMessage(SignalingMessage::Login { token }) => {
                    match vatsim_user_service.get_cid(&token).await {
                        Ok(cid) => {
                            tracing::trace!(?cid, "Login flow completed");
                            Some(cid)
                        },
                        Err(err) => {
                            tracing::debug!(?err, "Login flow failed");
                            let login_failure_message = SignalingMessage::LoginFailure {
                                reason: LoginFailureReason::InvalidCredentials,
                            };
                            if let Err(err) =
                                send_message(websocket_sender, login_failure_message).await
                            {
                                tracing::warn!(?err, "Failed to send login failure message");
                            }
                            None
                        }
                    }
                }
                MessageResult::ApplicationMessage(message) => {
                    tracing::debug!(msg = ?message, "Received unexpected message during login flow");
                    let login_failure_message = SignalingMessage::LoginFailure {
                        reason: LoginFailureReason::Unauthorized,
                    };
                    if let Err(err) = send_message(websocket_sender, login_failure_message).await {
                        tracing::warn!(?err, "Failed to send login failure message");
                    }
                    None
                }
                MessageResult::ControlMessage => {
                    tracing::trace!("Skipping control message during login");
                    continue;
                }
                MessageResult::Disconnected => {
                    tracing::debug!("Client disconnected during login flow");
                    None
                }
                MessageResult::Error(err) => {
                    tracing::warn!(?err, "Received error while handling login flow");
                    None
                }
            };
        }
    }).await {
        Ok(Some(id)) => Some(id),
        Ok(None) => None,
        Err(_) => {
            tracing::debug!("Login flow timed out");
            let login_timeout_message = SignalingMessage::LoginFailure {
                reason: LoginFailureReason::Timeout,
            };
            if let Err(err) = send_message(websocket_sender, login_timeout_message).await {
                tracing::warn!(?err, "Failed to send login timeout message");
            }
            None
        }
    }
}
