use crate::auth;
use crate::error::Error;
use crate::state::AppState;
use anyhow::Context;
use tauri::{AppHandle, Manager};

#[tauri::command]
pub async fn open_auth_url(app: AppHandle) -> Result<(), Error> {
    auth::open_auth_url(&app.state::<AppState>())
        .await
        .context("Failed to open auth url")?;
    Ok(())
}

#[tauri::command]
pub async fn check_auth_session(app: AppHandle) -> Result<bool, Error> {
    let authenticated = auth::check_auth_session(&app)
        .await
        .context("Failed to check auth session")?;
    Ok(authenticated)
}
