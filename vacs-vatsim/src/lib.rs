#[cfg(feature = "oauth")]
pub mod oauth;

#[cfg(feature = "user")]
pub mod user;

#[cfg(feature = "slurper")]
pub mod slurper;

/// User-Agent string used for all HTTP requests.
static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
