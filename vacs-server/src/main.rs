use axum::extract::{ConnectInfo, State, ws};
use axum::{
    Router,
    extract::ws::{WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::any,
};
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast, mpsc, watch};
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use vacs_shared::signaling;

#[derive(Clone)]
struct ClientWithSender {
    client: signaling::Client,
    sender: mpsc::Sender<signaling::Message>,
}

struct AppState {
    /// Key: CID
    clients: RwLock<HashMap<String, ClientWithSender>>,
    broadcast: broadcast::Sender<signaling::Message>,
    shutdown_rx: watch::Receiver<()>,
}

impl AppState {
    fn new(shutdown_rx: watch::Receiver<()>) -> Self {
        let (broadcast_sender, _) = broadcast::channel(10);
        Self {
            clients: RwLock::new(HashMap::new()),
            broadcast: broadcast_sender,
            shutdown_rx,
        }
    }
}

/// Represents the outcome of [`receive_message`], indicating whether the message received should be handled, skipped or receiving errored.
enum MessageResult {
    /// A valid application-message that can be processed.
    ApplicationMessage(signaling::Message),
    /// A control message (e.g., Ping, Pong) that should be skipped.
    ControlMessage,
    /// The client has disconnected.
    Disconnected,
    /// An error occurred while receiving the message.
    Error(anyhow::Error),
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!(
                    "{}=trace,tower_http=debug,axum::rejection=trace",
                    env!("CARGO_CRATE_NAME")
                )
                .into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let (shutdown_tx, shutdown_rx) = watch::channel(());

    let app_state = Arc::new(AppState::new(shutdown_rx.clone()));

    let app = Router::new()
        .route("/ws", any(ws_handler))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        )
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, addr, state))
}

async fn handle_socket(socket: WebSocket, addr: SocketAddr, state: Arc<AppState>) {
    let span = tracing::trace_span!("handle_socket", ?addr, client_id = tracing::field::Empty);
    let _enter = span.enter();

    tracing::trace!("Handling new websocket connection");

    let (mut websocket_sender, mut websocket_receiver) = socket.split();

    let client_id = match handle_login(&mut websocket_receiver, &mut websocket_sender).await {
        Some(id) => id,
        None => return,
    };
    let mut client_receiver = match register_client(&state, &client_id, &mut websocket_sender).await
    {
        Ok(receiver) => receiver,
        Err(err) => {
            tracing::warn!(?err, "Failed to register client");
            return;
        }
    };

    let mut broadcast_receiver = state.broadcast.subscribe();

    handle_client_interaction(
        &state,
        &client_id,
        &mut websocket_receiver,
        &mut websocket_sender,
        &mut client_receiver,
        &mut broadcast_receiver,
    )
    .await;

    disconnect_client(&state, &client_id).await;
}

async fn send_message(
    websocket_sender: &mut SplitSink<WebSocket, ws::Message>,
    message: &signaling::Message,
) -> anyhow::Result<()> {
    let serialized_message = signaling::Message::serialize(&message)
        .map_err(|e| anyhow::anyhow!(e).context("Failed to serialize message"))?;
    websocket_sender
        .send(ws::Message::from(serialized_message))
        .await
        .map_err(|e| anyhow::anyhow!(e).context("Failed to send message"))?;
    Ok(())
}

async fn receive_message(websocket_receiver: &mut SplitStream<WebSocket>) -> MessageResult {
    match websocket_receiver.next().await {
        Some(Ok(ws::Message::Text(raw_message))) => {
            tracing::trace!(?raw_message, "Received websocket text message");
            match signaling::Message::deserialize(&raw_message) {
                Ok(message) => MessageResult::ApplicationMessage(message),
                Err(err) => MessageResult::Error(
                    anyhow::anyhow!(err).context("Failed to deserialize message"),
                ),
            }
        }
        Some(Ok(ws::Message::Ping(msg))) => {
            tracing::trace!(?msg, "Received websocket ping message");
            MessageResult::ControlMessage
        }
        Some(Ok(ws::Message::Pong(_))) => {
            tracing::trace!("Received websocket pong message");
            MessageResult::ControlMessage
        }
        Some(Ok(ws::Message::Close(reason))) => {
            tracing::debug!(?reason, "Received websocket close message");
            MessageResult::Disconnected
        }
        Some(Ok(other)) => {
            tracing::trace!(?other, "Received unexpected websocket message");
            MessageResult::Error(anyhow::anyhow!("Received unexpected websocket message"))
        }
        Some(Err(err)) => {
            MessageResult::Error(anyhow::anyhow!(err).context("Failed to receive message"))
        }
        None => {
            tracing::debug!("Client receiver closed, disconnecting");
            MessageResult::Disconnected
        }
    }
}

async fn handle_login(
    websocket_receiver: &mut SplitStream<WebSocket>,
    websocket_sender: &mut SplitSink<WebSocket, ws::Message>,
) -> Option<String> {
    tracing::trace!("Handling login flow");
    let login_timeout = tokio::time::Duration::from_secs(10);

    tokio::time::timeout(login_timeout, async {
        loop {
            return match receive_message(websocket_receiver).await {
                MessageResult::ApplicationMessage(signaling::Message::Login { id, token }) => {
                    if token.is_empty() {
                        tracing::trace!("Received login with empty token");
                        let login_failure_message = signaling::Message::LoginFailure {
                            reason: signaling::LoginFailureReason::InvalidCredentials,
                        };
                        if let Err(err) =
                            send_message(websocket_sender, &login_failure_message).await
                        {
                            tracing::warn!(?err, "Failed to send login failure message");
                        }
                        return None;
                    }
                    tracing::Span::current().record("client_id", &id);
                    tracing::trace!("Login flow completed");
                    Some(id)
                }
                MessageResult::ApplicationMessage(message) => {
                    tracing::debug!(?message, "Received unexpected message during login flow");
                    let login_failure_message = signaling::Message::LoginFailure {
                        reason: signaling::LoginFailureReason::InvalidLoginFlow,
                    };
                    if let Err(err) = send_message(websocket_sender, &login_failure_message).await {
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
    })
    .await
    .unwrap_or_else(|_| {
        tracing::debug!("Login timed out");
        None
    })
}

async fn register_client(
    state: &Arc<AppState>,
    client_id: &str,
    websocket_sender: &mut SplitSink<WebSocket, ws::Message>,
) -> anyhow::Result<mpsc::Receiver<signaling::Message>> {
    tracing::trace!("Registering client");

    if state.clients.read().await.contains_key(client_id) {
        tracing::debug!("Client already connected, rejecting new websocket connection");
        let login_failure_message = signaling::Message::LoginFailure {
            reason: signaling::LoginFailureReason::IdTaken,
        };
        send_message(websocket_sender, &login_failure_message).await?;
        return Err(anyhow::anyhow!("Client already connected"));
    }

    let (client_sender, client_receiver) = mpsc::channel(100);
    let client = signaling::Client {
        id: client_id.to_string(),
        display_name: client_id.to_string(),
        status: signaling::ClientStatus::Connected,
    };

    state.clients.write().await.insert(
        client_id.to_string(),
        ClientWithSender {
            sender: client_sender,
            client: client.clone(),
        },
    );

    if state.broadcast.receiver_count() > 0 {
        if let Err(err) = state
            .broadcast
            .send(signaling::Message::ClientUpdate { client })
        {
            tracing::warn!(?err, "Failed to broadcast client connected message");
        }
    } else {
        tracing::debug!("No broadcast receivers, skipping client connected message");
    }

    tracing::trace!("Client registered");
    Ok(client_receiver)
}

async fn handle_client_interaction(
    state: &Arc<AppState>,
    client_id: &str,
    websocket_receiver: &mut SplitStream<WebSocket>,
    websocket_sender: &mut SplitSink<WebSocket, ws::Message>,
    client_receiver: &mut mpsc::Receiver<signaling::Message>,
    broadcast_receiver: &mut broadcast::Receiver<signaling::Message>,
) {
    let mut shutdown_rx = state.shutdown_rx.clone();

    tracing::debug!("Starting to handle client interaction");

    // Complete login flow by sending list of currently connected clients to the new client
    let clients = state
        .clients
        .read()
        .await
        .values()
        .cloned()
        .map(|c| c.client)
        .collect();
    if let Err(err) = send_message(
        websocket_sender,
        &signaling::Message::ClientList { clients },
    )
    .await
    {
        tracing::warn!(?err, "Failed to send client list");
    }

    loop {
        tokio::select! {
            biased;

            _ = shutdown_rx.changed() => {
                tracing::debug!("Shutdown signal received:");
                break;
            }

            message_result = receive_message(websocket_receiver) => {
                match message_result {
                    MessageResult::ApplicationMessage(message) => {
                        handle_application_message(state, client_id, websocket_sender, &message).await;
                    }
                    MessageResult::ControlMessage => {
                        continue;
                    }
                    MessageResult::Disconnected => {
                        tracing::debug!("Client disconnected");
                        break;
                    }
                    MessageResult::Error(err) => {
                        tracing::warn!(?err, "Failed to receive WebSocket message");
                        break;
                    }
                }
            }

            message = client_receiver.recv() => {
                if let Some(message) = message {
                    if let Err(err) = send_message(websocket_sender, &message).await {
                        tracing::warn!(?err, "Failed to send direct message");
                    }
                } else {
                    tracing::debug!("Client receiver closed, disconnecting client");
                    break;
                }
            }

            message = broadcast_receiver.recv() => {
                match message {
                    Ok(message) => {
                        if let Err(err) = send_message(websocket_sender, &message).await {
                            tracing::warn!(?err, "Failed to send broadcast message");
                        }
                    }
                    Err(err) => {
                        tracing::debug!(?err, "Broadcast receiver closed, disconnecting client");
                        break;
                    }
                }
            }
        }
    }

    tracing::debug!("Finished handling client interaction");
}

async fn disconnect_client(state: &Arc<AppState>, client_id: &str) {
    tracing::debug!("Disconnecting client");

    if let Some(mut client) = state.clients.write().await.remove(client_id) {
        client.client.status = signaling::ClientStatus::Disconnected;
        let client_disconnected_message = signaling::Message::ClientUpdate {
            client: client.client,
        };

        if let Err(err) = state.broadcast.send(client_disconnected_message) {
            tracing::warn!(?err, "Failed to broadcast client disconnected message");
        }
    }

    tracing::debug!("Client disconnected");
}

async fn handle_application_message(
    state: &Arc<AppState>,
    client_id: &str,
    websocket_sender: &mut SplitSink<WebSocket, ws::Message>,
    message: &signaling::Message,
) {
    tracing::trace!(?message, "Handling application message");

    match message {
        signaling::Message::ListClients => {
            tracing::trace!("Returning list of clients");
            let clients = state
                .clients
                .read()
                .await
                .values()
                .cloned()
                .map(|c| c.client)
                .collect();
            if let Err(err) = send_message(
                websocket_sender,
                &signaling::Message::ClientList { clients },
            )
            .await
            {
                tracing::warn!(?err, "Failed to send client list");
            }
        }
        signaling::Message::Logout => {
            tracing::trace!("Logging out client");
            disconnect_client(state, client_id).await;
        }
        signaling::Message::CallOffer { peer_id, sdp } => {
            tracing::trace!(?peer_id, "Received call offer");
            match state.clients.read().await.get(peer_id) {
                Some(client) => {
                    if let Err(err) = client
                        .sender
                        .send(signaling::Message::CallOffer {
                            peer_id: client_id.to_string(),
                            sdp: sdp.to_owned(),
                        })
                        .await
                    {
                        tracing::warn!(?peer_id, ?err, "Failed to send call offer");
                        // TODO inform client about failed call offer
                    }
                }
                None => {
                    tracing::trace!(?peer_id, "Peer not found");
                    if let Err(err) =
                        send_message(websocket_sender, &signaling::Message::PeerNotFound {}).await
                    {
                        tracing::warn!(?peer_id, ?err, "Failed to send peer not found reply");
                    }
                }
            }
        }
        signaling::Message::CallAnswer { peer_id, sdp } => {
            tracing::trace!(?peer_id, "Received call answer");
            match state.clients.read().await.get(peer_id) {
                Some(client) => {
                    if let Err(err) = client
                        .sender
                        .send(signaling::Message::CallAnswer {
                            peer_id: client_id.to_string(),
                            sdp: sdp.to_owned(),
                        })
                        .await
                    {
                        tracing::warn!(?peer_id, ?err, "Failed to send call answer");
                        // TODO inform client about failed call answer
                    }
                }
                None => {
                    tracing::trace!(?peer_id, "Peer not found");
                    if let Err(err) =
                        send_message(websocket_sender, &signaling::Message::PeerNotFound {}).await
                    {
                        tracing::warn!(?peer_id, ?err, "Failed to send peer not found reply");
                    }
                }
            }
        }
        signaling::Message::CallReject { peer_id } => {
            tracing::trace!(?peer_id, "Received call rejection");
            match state.clients.read().await.get(peer_id) {
                Some(client) => {
                    if let Err(err) = client
                        .sender
                        .send(signaling::Message::CallReject {
                            peer_id: client_id.to_string(),
                        })
                        .await
                    {
                        tracing::warn!(?peer_id, ?err, "Failed to send call rejection");
                        // TODO inform client about failed call rejection
                    }
                }
                None => {
                    tracing::trace!(?peer_id, "Peer not found");
                    if let Err(err) =
                        send_message(websocket_sender, &signaling::Message::PeerNotFound {}).await
                    {
                        tracing::warn!(?peer_id, ?err, "Failed to send peer not found reply");
                    }
                }
            }
        }
        signaling::Message::CallIceCandidate { peer_id, candidate } => {
            tracing::trace!(?peer_id, "Received call ICE candidate");
            match state.clients.read().await.get(peer_id) {
                Some(client) => {
                    if let Err(err) = client
                        .sender
                        .send(signaling::Message::CallIceCandidate {
                            peer_id: client_id.to_string(),
                            candidate: candidate.to_owned(),
                        })
                        .await
                    {
                        tracing::warn!(?peer_id, ?err, "Failed to send ICE candidate");
                        // TODO inform client about failed ICE candidate
                    }
                }
                None => {
                    tracing::trace!(?peer_id, "Peer not found");
                    if let Err(err) =
                        send_message(websocket_sender, &signaling::Message::PeerNotFound {}).await
                    {
                        tracing::warn!(?peer_id, ?err, "Failed to send peer not found reply");
                    }
                }
            }
        }
        signaling::Message::CallEnd { peer_id } => {
            tracing::trace!(?peer_id, "Received call end");
            match state.clients.read().await.get(peer_id) {
                Some(client) => {
                    if let Err(err) = client
                        .sender
                        .send(signaling::Message::CallReject {
                            peer_id: client_id.to_string(),
                        })
                        .await
                    {
                        tracing::warn!(?peer_id, ?err, "Failed to send call end");
                        // TODO inform client about failed call end
                    }
                }
                None => {
                    tracing::trace!(?peer_id, "Peer not found");
                    if let Err(err) =
                        send_message(websocket_sender, &signaling::Message::PeerNotFound {}).await
                    {
                        tracing::warn!(?peer_id, ?err, "Failed to send peer not found reply");
                    }
                }
            }
        }
        _ => {}
    }
}
