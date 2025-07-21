mod auth;
mod commands;
mod config;
mod error;
mod secrets;
mod signaling;
mod state;

use crate::state::AppState;
use tauri::{Manager, RunEvent};

pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Info)
                .level_for("vacs_client_lib", log::LevelFilter::Trace)
                .build(),
        )
        .plugin(tauri_plugin_single_instance::init(|app, argv, _| {
            if let Some(url) = argv.get(1) {
                let app = app.clone();
                let url = url.clone();
                tauri::async_runtime::spawn(async move {
                    auth::handle_auth_callback(&app, &url)
                        .await
                        .expect("Failed to handle auth callback");
                });
            }
        }))
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            use tauri_plugin_deep_link::DeepLinkExt;
            app.deep_link().register_all()?;

            app.manage(AppState::new()?);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::app::frontend_ready,
            commands::auth::open_auth_url,
            commands::auth::check_auth_session,
        ])
        .build(tauri::generate_context!())
        .expect("Failed to build tauri application")
        .run(move |app_handle, event| if let RunEvent::ExitRequested { .. } = event {
            app_handle
                .state::<AppState>()
                .persist()
                .expect("Failed to persist app state");
        });
}
