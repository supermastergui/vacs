use cpal::{BuildStreamError, PlayStreamError, StreamError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AudioStartError {
    #[error("Audio device is not available")]
    DeviceNotAvailable,
    #[error("Unsupported config")]
    UnsupportedConfig,
    #[error("Audio device is busy or permission was denied")]
    DeviceBusyOrDenied,
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl From<BuildStreamError> for AudioStartError {
    fn from(e: BuildStreamError) -> Self {
        use BuildStreamError::*;
        match e {
            DeviceNotAvailable => AudioStartError::DeviceNotAvailable,
            StreamConfigNotSupported | InvalidArgument => AudioStartError::UnsupportedConfig,
            StreamIdOverflow => AudioStartError::Other(anyhow::anyhow!("Stream ID overflow")),
            BackendSpecific { err } => {
                tracing::debug!(?err, "Backend specific cpal build stream error");
                AudioStartError::Other(anyhow::anyhow!(err.description))
            }
        }
    }
}

impl From<PlayStreamError> for AudioStartError {
    fn from(e: PlayStreamError) -> Self {
        use PlayStreamError::*;
        match e {
            DeviceNotAvailable => AudioStartError::DeviceNotAvailable,
            BackendSpecific { err } => {
                tracing::debug!(?err, "Backend specific cpal play stream error");
                AudioStartError::Other(anyhow::anyhow!(err.description))
            }
        }
    }
}

#[derive(Debug, Error)]
pub enum AudioRuntimeError {
    #[error("Audio device was disconnected")]
    DeviceLost,
    #[error("{0}")]
    Other(String),
}

impl From<StreamError> for AudioRuntimeError {
    fn from(e: StreamError) -> Self {
        use cpal::StreamError::*;
        match e {
            DeviceNotAvailable => AudioRuntimeError::DeviceLost,
            BackendSpecific { err } => {
                tracing::debug!(?err, "Backend specific cpal stream error");
                AudioRuntimeError::Other(err.description)
            }
        }
    }
}
