use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Auth error: {0}")]
    Auth(String),
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Network error: {0}")]
    Network(String),
    #[error(transparent)]
    Generic(#[from] anyhow::Error),
}

impl serde::Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        FrontendError::from(self).serialize(serializer)
    }
}

#[derive(Debug, Serialize)]
pub struct FrontendError {
    title: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    timeout_ms: Option<u16>,
}

impl FrontendError {
    pub fn new(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            timeout_ms: None,
        }
    }

    pub fn new_with_timeout(
        title: impl Into<String>,
        message: impl Into<String>,
        timeout_ms: u16,
    ) -> Self {
        Self::new(title, message).with_timeout(timeout_ms)
    }

    pub fn with_timeout(mut self, timeout_ms: u16) -> Self {
        self.timeout_ms = Some(timeout_ms);
        self
    }
}

impl From<Error> for FrontendError {
    fn from(err: Error) -> Self {
        FrontendError::from(&err)
    }
}

impl From<&Error> for FrontendError {
    fn from(err: &Error) -> Self {
        match err {
            Error::Auth(err) => FrontendError::new("Auth error", err),
            Error::Unauthorized => {
                FrontendError::new_with_timeout("Auth error", "Unauthorized", 5000)
            }
            Error::Network(err) => FrontendError::new("Network error", err),
            Error::Generic(err) => FrontendError::new("Error", err.to_string()),
        }
    }
}
