use crate::signaling::error::SignalingError;
pub(crate) use crate::signaling::transport::SignalingTransport;
use crate::signaling::{ClientInfo, Message};
use std::time::Duration;
use tokio::sync::watch;
use tracing::instrument;

pub struct SignalingClientBuilder<T: SignalingTransport> {
    transport: T,
    login_timeout: Duration,
    shutdown_rx: watch::Receiver<()>,
}

impl<T: SignalingTransport> SignalingClientBuilder<T> {
    #[instrument(level = "debug", skip(transport, shutdown_rx))]
    pub fn new(transport: T, shutdown_rx: watch::Receiver<()>) -> Self {
        Self {
            transport,
            login_timeout: Duration::from_secs(5),
            shutdown_rx,
        }
    }

    #[instrument(level = "debug", skip(self))]
    pub fn with_login_timeout(mut self, timeout: Duration) -> Self {
        self.login_timeout = timeout;
        self
    }

    #[instrument(level = "debug", skip(self))]
    pub fn build(self) -> SignalingClient<T> {
        SignalingClient {
            transport: self.transport,
            login_timeout: self.login_timeout,
            shutdown_rx: self.shutdown_rx,
        }
    }
}

pub struct SignalingClient<T: SignalingTransport> {
    transport: T,
    login_timeout: Duration,
    shutdown_rx: watch::Receiver<()>,
}

impl<T: SignalingTransport> SignalingClient<T> {
    #[instrument(level = "debug", skip(transport, shutdown_rx))]
    pub fn new(transport: T, shutdown_rx: watch::Receiver<()>) -> Self {
        SignalingClientBuilder::new(transport, shutdown_rx).build()
    }

    pub fn builder(transport: T, shutdown_rx: watch::Receiver<()>) -> SignalingClientBuilder<T> {
        SignalingClientBuilder::new(transport, shutdown_rx)
    }

    #[instrument(level = "debug", skip(self), err)]
    pub async fn send(&mut self, msg: Message) -> Result<(), SignalingError> {
        tracing::debug!("Sending message to server");
        tokio::select! {
            biased;
            _ = self.shutdown_rx.changed() => {
                tracing::debug!("Shutdown signal received, aborting send");
                Err(SignalingError::Timeout("Shutdown signal received".to_string()))
            }
            result = self.transport.send(msg) => result,
        }
    }

    #[instrument(level = "debug", skip(self), err)]
    pub async fn recv(&mut self) -> Result<Message, SignalingError> {
        tracing::debug!("Waiting for message from server");
        tokio::select! {
            biased;
            _ = self.shutdown_rx.changed() => {
                tracing::debug!("Shutdown signal received, aborting recv");
                Err(SignalingError::Timeout("Shutdown signal received".to_string()))
            }
            msg = self.transport.recv() => msg,
        }
    }

    #[instrument(level = "debug", skip(self), err)]
    pub async fn recv_with_timeout(
        &mut self,
        timeout: Duration,
    ) -> Result<Message, SignalingError> {
        tracing::debug!("Waiting for message from server");
        let recv_result = tokio::select! {
            biased;
            _ = self.shutdown_rx.changed() => {
                tracing::debug!("Shutdown signal received, aborting recv");
                return Err(SignalingError::Timeout("Shutdown signal received".to_string()))
            }
            res = tokio::time::timeout(timeout, self.transport.recv()) => res,
        };

        match recv_result {
            Ok(Ok(msg)) => Ok(msg),
            Ok(Err(err)) => Err(err),
            Err(_) => {
                tracing::warn!("Timeout waiting for message");
                Err(SignalingError::Timeout(
                    "Timeout waiting for message".to_string(),
                ))
            }
        }
    }

    #[instrument(level = "info", skip(self, token), fields(%id))]
    pub async fn login(
        &mut self,
        id: &str,
        token: &str,
    ) -> Result<Vec<ClientInfo>, SignalingError> {
        tracing::debug!("Sending Login message to server");
        self.send(Message::Login {
            id: id.to_string(),
            token: token.to_string(),
        })
        .await?;

        tracing::debug!(login_timeout = ?self.login_timeout, "Awaiting authentication response from server");
        match self.recv_with_timeout(self.login_timeout).await? {
            Message::ClientList { clients } => {
                tracing::info!(num_clients = ?clients.len(), "Login successful, received client list");
                Ok(clients)
            }
            Message::LoginFailure { reason } => {
                tracing::warn!(?reason, "Login failed");
                Err(SignalingError::LoginError(reason))
            }
            Message::Error { reason, peer_id } => {
                tracing::error!(?reason, ?peer_id, "Server returned error");
                Err(SignalingError::ServerError(reason))
            }
            other => {
                tracing::error!(?other, "Received unexpected message from server");
                Err(SignalingError::ProtocolError(
                    "Expected ClientList after Login".to_string(),
                ))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signaling::transport::mock::MockTransport;
    use crate::signaling::{ErrorReason, LoginFailureReason};
    use pretty_assertions::assert_matches;
    use test_log::test;
    use tokio::sync::watch;

    fn test_client_list() -> Vec<ClientInfo> {
        vec![
            ClientInfo {
                id: "client1".to_string(),
                display_name: "Client 1".to_string(),
            },
            ClientInfo {
                id: "client2".to_string(),
                display_name: "Client 2".to_string(),
            },
            ClientInfo {
                id: "client3".to_string(),
                display_name: "Client 3".to_string(),
            },
        ]
    }

    #[test(tokio::test)]
    async fn send() {
        let (mock, mut handle) = MockTransport::new();
        let (_shutdown_tx, shutdown_rx) = watch::channel(());
        let mut client = SignalingClient::new(mock, shutdown_rx);
        let msg = Message::Login {
            id: "test".to_string(),
            token: "test".to_string(),
        };

        let result = client.send(msg.clone()).await;
        assert!(result.is_ok());

        let sent_msg = handle.outgoing_rx.recv().await.unwrap();
        assert_eq!(sent_msg, msg);
    }

    #[test(tokio::test)]
    async fn send_shutdown() {
        let (mock, _handle) = MockTransport::new();
        let (shutdown_tx, shutdown_rx) = watch::channel(());
        let mut client = SignalingClient::new(mock, shutdown_rx);
        let msg = Message::Login {
            id: "test".to_string(),
            token: "test".to_string(),
        };

        shutdown_tx.send(()).unwrap();
        let result = client.send(msg.clone()).await;
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "timeout: Shutdown signal received".to_string()
        );
    }

    #[test(tokio::test)]
    async fn send_disconnected() {
        let (mock, handle) = MockTransport::new();
        let (_shutdown_tx, shutdown_rx) = watch::channel(());
        let mut client = SignalingClient::new(mock, shutdown_rx);
        let msg = Message::Login {
            id: "test".to_string(),
            token: "test".to_string(),
        };

        drop(handle.outgoing_rx); // Simulate the outgoing channel being closed

        let send_result = client.send(msg).await;
        assert!(send_result.is_err());
        assert_matches!(send_result, Err(SignalingError::Transport(_)));
    }

    #[test(tokio::test)]
    async fn recv() {
        let (mock, handle) = MockTransport::new();
        let (_shutdown_tx, shutdown_rx) = watch::channel(());
        let mut client = SignalingClient::new(mock, shutdown_rx);
        let msg = Message::ClientList {
            clients: test_client_list(),
        };

        let result = handle.incoming_tx.send(msg.clone()).await;
        assert!(result.is_ok());

        let recv_result = client.recv().await;
        assert!(recv_result.is_ok());
        assert_eq!(recv_result.unwrap(), msg);
    }

    #[test(tokio::test)]
    async fn recv_shutdown() {
        let (mock, handle) = MockTransport::new();
        let (shutdown_tx, shutdown_rx) = watch::channel(());
        let mut client = SignalingClient::new(mock, shutdown_rx);
        let msg = Message::Login {
            id: "test".to_string(),
            token: "test".to_string(),
        };

        let result = handle.incoming_tx.send(msg.clone()).await;
        assert!(result.is_ok());

        shutdown_tx.send(()).unwrap();
        let recv_result = client.recv().await;
        assert!(recv_result.is_err());
        assert_eq!(
            recv_result.unwrap_err().to_string(),
            "timeout: Shutdown signal received".to_string()
        );
    }

    #[test(tokio::test)]
    async fn recv_with_timeout() {
        let (mock, handle) = MockTransport::new();
        let (_shutdown_tx, shutdown_rx) = watch::channel(());
        let mut client = SignalingClient::new(mock, shutdown_rx);
        let msg = Message::CallAnswer {
            peer_id: "client1".to_string(),
            sdp: "sdp".to_string(),
        };

        let result = handle.incoming_tx.send(msg.clone()).await;
        assert!(result.is_ok());

        let recv_result = client.recv_with_timeout(Duration::from_millis(100)).await;
        assert!(recv_result.is_ok());
        assert_eq!(recv_result.unwrap(), msg);
    }

    #[test(tokio::test)]
    async fn recv_with_timeout_expired() {
        let (mock, _handle) = MockTransport::new();
        let (_shutdown_tx, shutdown_rx) = watch::channel(());
        let mut client = SignalingClient::new(mock, shutdown_rx);

        let recv_result = client.recv_with_timeout(Duration::from_millis(100)).await;
        assert!(recv_result.is_err());
        assert_eq!(
            recv_result.unwrap_err().to_string(),
            "timeout: Timeout waiting for message".to_string()
        );
    }

    #[test(tokio::test)]
    async fn recv_with_timeout_shutdown() {
        let (mock, handle) = MockTransport::new();
        let (shutdown_tx, shutdown_rx) = watch::channel(());
        let mut client = SignalingClient::new(mock, shutdown_rx);
        let msg = Message::Login {
            id: "test".to_string(),
            token: "test".to_string(),
        };

        let result = handle.incoming_tx.send(msg.clone()).await;
        assert!(result.is_ok());

        shutdown_tx.send(()).unwrap();
        let recv_result = client.recv_with_timeout(Duration::from_millis(100)).await;
        assert!(recv_result.is_err());
        assert_eq!(
            recv_result.unwrap_err().to_string(),
            "timeout: Shutdown signal received".to_string()
        );
    }

    #[test(tokio::test)]
    async fn recv_server_error() {
        let (mock, handle) = MockTransport::new();
        let (_shutdown_tx, shutdown_rx) = watch::channel(());
        let mut client = SignalingClient::new(mock, shutdown_rx);
        let msg = Message::Error {
            reason: ErrorReason::Internal("something failed".to_string()),
            peer_id: None,
        };

        let result = handle.incoming_tx.send(msg.clone()).await;
        assert!(result.is_ok());

        let recv_result = client.recv().await;
        assert!(recv_result.is_ok());
        assert_eq!(recv_result.unwrap(), msg);
    }

    #[test(tokio::test)]
    async fn recv_peer_connection_error() {
        let (mock, handle) = MockTransport::new();
        let (_shutdown_tx, shutdown_rx) = watch::channel(());
        let mut client = SignalingClient::new(mock, shutdown_rx);
        let msg = Message::Error {
            reason: ErrorReason::PeerConnection,
            peer_id: Some("client1".to_string()),
        };

        let result = handle.incoming_tx.send(msg.clone()).await;
        assert!(result.is_ok());

        let recv_result = client.recv().await;
        assert!(recv_result.is_ok());
        assert_eq!(recv_result.unwrap(), msg);
    }

    #[test(tokio::test)]
    async fn recv_disconnected() {
        let (mock, handle) = MockTransport::new();
        let (_shutdown_tx, shutdown_rx) = watch::channel(());
        let mut client = SignalingClient::new(mock, shutdown_rx);

        drop(handle.incoming_tx); // Simulate the incoming channel being closed

        let recv_result = client.recv().await;
        assert!(recv_result.is_err());
        assert_matches!(recv_result, Err(SignalingError::Disconnected));
    }

    #[test(tokio::test)]
    async fn login() {
        let (mock, mut handle) = MockTransport::new();
        let (_shutdown_tx, shutdown_rx) = watch::channel(());
        let mut client = SignalingClient::new(mock, shutdown_rx);
        let test_clients = test_client_list();
        let msg = Message::ClientList {
            clients: test_clients.clone(),
        };

        let result = handle.incoming_tx.send(msg.clone()).await;
        assert!(result.is_ok());

        let login_result = client.login("client1", "token1").await;
        assert!(login_result.is_ok());
        assert_eq!(login_result.unwrap(), test_clients);

        let sent_message = handle.outgoing_rx.recv().await;
        assert_matches!(sent_message, Some(Message::Login { ref id, ref token }) if id == "client1" && token == "token1");
    }

    #[test(tokio::test)]
    async fn login_timeout() {
        let (mock, mut handle) = MockTransport::new();
        let (_shutdown_tx, shutdown_rx) = watch::channel(());
        let mut client = SignalingClientBuilder::new(mock, shutdown_rx)
            .with_login_timeout(Duration::from_millis(100))
            .build();

        let login_result = client.login("client1", "token1").await;
        assert!(login_result.is_err());
        assert_matches!(login_result, Err(SignalingError::Timeout(_)));

        let sent_message = handle.outgoing_rx.recv().await;
        assert_matches!(sent_message, Some(Message::Login { ref id, ref token }) if id == "client1" && token == "token1");
    }

    #[test(tokio::test)]
    async fn login_unauthorized() {
        let (mock, mut handle) = MockTransport::new();
        let (_shutdown_tx, shutdown_rx) = watch::channel(());
        let mut client = SignalingClient::new(mock, shutdown_rx);
        let msg = Message::LoginFailure {
            reason: LoginFailureReason::Unauthorized,
        };

        let result = handle.incoming_tx.send(msg.clone()).await;
        assert!(result.is_ok());

        let login_result = client.login("client1", "token1").await;
        assert!(login_result.is_err());
        assert_matches!(
            login_result,
            Err(SignalingError::LoginError(LoginFailureReason::Unauthorized))
        );

        let sent_message = handle.outgoing_rx.recv().await;
        assert_matches!(sent_message, Some(Message::Login { ref id, ref token }) if id == "client1" && token == "token1");
    }

    #[test(tokio::test)]
    async fn login_invalid_credentials() {
        let (mock, mut handle) = MockTransport::new();
        let (_shutdown_tx, shutdown_rx) = watch::channel(());
        let mut client = SignalingClient::new(mock, shutdown_rx);
        let msg = Message::LoginFailure {
            reason: LoginFailureReason::InvalidCredentials,
        };

        let result = handle.incoming_tx.send(msg.clone()).await;
        assert!(result.is_ok());

        let login_result = client.login("client1", "token1").await;
        assert!(login_result.is_err());
        assert_matches!(
            login_result,
            Err(SignalingError::LoginError(
                LoginFailureReason::InvalidCredentials
            ))
        );

        let sent_message = handle.outgoing_rx.recv().await;
        assert_matches!(sent_message, Some(Message::Login { ref id, ref token }) if id == "client1" && token == "token1");
    }

    #[test(tokio::test)]
    async fn login_duplicate_id() {
        let (mock, mut handle) = MockTransport::new();
        let (_shutdown_tx, shutdown_rx) = watch::channel(());
        let mut client = SignalingClient::new(mock, shutdown_rx);
        let msg = Message::LoginFailure {
            reason: LoginFailureReason::DuplicateId,
        };

        let result = handle.incoming_tx.send(msg.clone()).await;
        assert!(result.is_ok());

        let login_result = client.login("client1", "token1").await;
        assert!(login_result.is_err());
        assert_matches!(
            login_result,
            Err(SignalingError::LoginError(LoginFailureReason::DuplicateId))
        );

        let sent_message = handle.outgoing_rx.recv().await;
        assert_matches!(sent_message, Some(Message::Login { ref id, ref token }) if id == "client1" && token == "token1");
    }

    #[test(tokio::test)]
    async fn login_unexpected_message() {
        let (mock, mut handle) = MockTransport::new();
        let (_shutdown_tx, shutdown_rx) = watch::channel(());
        let mut client = SignalingClient::new(mock, shutdown_rx);
        let msg = Message::CallAnswer {
            peer_id: "client1".to_string(),
            sdp: "sdp".to_string(),
        };

        let result = handle.incoming_tx.send(msg.clone()).await;
        assert!(result.is_ok());

        let login_result = client.login("client1", "token1").await;
        assert!(login_result.is_err());
        assert_matches!(login_result, Err(SignalingError::ProtocolError(_)));

        let sent_message = handle.outgoing_rx.recv().await;
        assert_matches!(sent_message, Some(Message::Login { ref id, ref token }) if id == "client1" && token == "token1");
    }

    #[test(tokio::test)]
    async fn login_server_error() {
        let (mock, mut handle) = MockTransport::new();
        let (_shutdown_tx, shutdown_rx) = watch::channel(());
        let mut client = SignalingClient::new(mock, shutdown_rx);
        let msg = Message::Error {
            reason: ErrorReason::Internal("something failed".to_string()),
            peer_id: None,
        };

        let result = handle.incoming_tx.send(msg.clone()).await;
        assert!(result.is_ok());

        let login_result = client.login("client1", "token1").await;
        assert!(login_result.is_err());
        assert_matches!(login_result, Err(SignalingError::ServerError(_)));

        let sent_message = handle.outgoing_rx.recv().await;
        assert_matches!(sent_message, Some(Message::Login { ref id, ref token }) if id == "client1" && token == "token1");
    }
}
