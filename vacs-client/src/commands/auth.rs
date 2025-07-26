use crate::auth;
use crate::error::Error;
use crate::state::AppState;
use tauri::{AppHandle, Manager};

#[tauri::command]
#[vacs_macros::log_err]
pub async fn open_auth_url(app: AppHandle) -> Result<(), Error> {
    auth::open_auth_url(&app.state::<AppState>()).await
}

#[tauri::command]
#[vacs_macros::log_err]
pub async fn check_auth_session(app: AppHandle) -> Result<(), Error> {
    auth::check_auth_session(&app).await
}

#[tauri::command]
#[vacs_macros::log_err]
pub async fn logout(app: AppHandle) -> Result<(), Error> {
    auth::logout(&app).await
}
