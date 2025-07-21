#[tauri::command]
pub fn frontend_ready() {
    log::info!("Frontend ready");
}
