use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct AppConfig {
    pub api: ApiConfig,
    #[cfg(feature = "audio")]
    pub audio: AudioConfig,
    #[cfg(feature = "webrtc")]
    pub webrtc: WebrtcConfig,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct ApiConfig {
    pub url: String,
    pub key: String,
}

#[cfg(feature = "audio")]
#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct AudioConfig {
    pub input: AudioDeviceConfig,
    pub output: AudioDeviceConfig,
}

#[cfg(feature = "audio")]
#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct AudioDeviceConfig {
    pub host_name: Option<String>,
    pub device_name: Option<String>,
    pub channels: u16,
}

#[cfg(feature = "webrtc")]
#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct WebrtcConfig {
    pub ice_servers: Vec<String>,
}
