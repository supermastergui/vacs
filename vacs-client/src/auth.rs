use crate::config::BackendEndpoint;
use crate::error::{Error, HandleUnauthorizedExt};
use crate::state::AppState;
use anyhow::Context;
use serde_json::Value;
use tauri::{AppHandle, Emitter, Manager};
use url::Url;
use vacs_protocol::http::auth::{AuthExchangeToken, InitVatsimLogin, UserInfo};

pub async fn open_auth_url(app_state: &AppState) -> Result<(), Error> {
    let auth_url = app_state
        .http_get::<InitVatsimLogin>(BackendEndpoint::InitAuth, None)
        .await?
        .url;

    log::info!("Opening auth URL: {auth_url}");

    tauri_plugin_opener::open_url(auth_url, None::<&str>)
        .context("Failed to open auth URL with the default browser")?;

    Ok(())
}

#[vacs_macros::log_err]
pub async fn handle_auth_callback(app: &AppHandle, url: &str) -> Result<(), Error> {
    let url = Url::parse(url).context("Failed to parse auth callback URL")?;

    let mut code = None;
    let mut state = None;

    for (key, value) in url.query_pairs() {
        match &*key {
            "code" => code = Some(value),
            "state" => state = Some(value),
            _ => {}
        }
    }

    let code = code.context("Auth callback URL does not contain code")?;
    let state = state.context("Auth callback URL does not contain code")?;

    let cid = app
        .state::<AppState>()
        .http_post::<UserInfo, AuthExchangeToken>(
            BackendEndpoint::ExchangeCode,
            None,
            Some(AuthExchangeToken {
                code: code.to_string(),
                state: state.to_string(),
            }),
        )
        .await?
        .cid;

    log::info!("Successfully authenticated as CID {cid}");
    app.emit("auth:authenticated", cid).ok();

    Ok(())
}

pub async fn check_auth_session(app: &AppHandle) -> Result<(), Error> {
    log::debug!("Fetching user info");
    let response = app.state::<AppState>()
        .http_get::<UserInfo>(BackendEndpoint::UserInfo, None)
        .await;

    match response {
        Ok(user_info) => {
            log::info!("Authenticated as CID {}", user_info.cid);
            app.emit("auth:authenticated", user_info.cid).ok();
            Ok(())
        }
        Err(Error::Unauthorized) => {
            log::info!("Not authenticated");
            app.emit("auth:unauthenticated", Value::Null).ok();
            Ok(())
        }
        Err(err) => {
            log::info!("Not authenticated");
            app.emit("auth:unauthenticated", Value::Null).ok();
            Err(err)
        }
    }
}

pub async fn logout(app: &AppHandle) -> Result<(), Error> {
    log::debug!("Logging out");

    let app_state = app.state::<AppState>();
    app_state
        .http_post::<(), ()>(BackendEndpoint::Logout, None, None)
        .await
        .handle_unauthorized(app)?;

    app_state
        .clear_cookie_store()
        .context("Failed to clear cookie store")?;

    log::info!("Successfully logged out");
    app.emit("auth:unauthenticated", Value::Null).ok();

    Ok(())
}
