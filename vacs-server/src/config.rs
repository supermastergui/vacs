use serde::Deserialize;
use std::time::Duration;

pub const BROADCAST_CHANNEL_CAPACITY: usize = 100;
pub const CLIENT_CHANNEL_CAPACITY: usize = 100;
pub const SERVER_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(30);
pub const LOGIN_FLOW_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub bind_addr: String,
}
