use std::sync::Arc;
use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;

use crate::constants::get_constants;
use crate::depot_service::{cancel_download, run_depot_download, send_input, ActiveDownloadState};
use crate::installer::{
    clear_cache as installer_clear_cache, detect_game_folder as installer_detect,
    install_dawn as installer_install, launch_game as installer_launch,
    open_docs as installer_docs, open_folder as installer_open,
    restore_dawn as installer_restore, uninstall_dawn as installer_uninstall,
    validate_game_folder as installer_validate_game, validate_preflight as installer_preflight,
};
use crate::types::{CommandResult, FolderValidationResult, InstallerConstants, PreflightResult};

#[tauri::command]
pub async fn select_game_folder(app: AppHandle) -> Option<String> {
    let folder = app.dialog().file().blocking_pick_folder();
    folder.map(|p| p.to_string())
}

#[tauri::command]
pub fn get_installer_constants() -> InstallerConstants {
    get_constants()
}

#[tauri::command]
pub fn validate_preflight(folder_path: String) -> PreflightResult {
    installer_preflight(folder_path)
}

#[tauri::command]
pub fn validate_game_folder(folder_path: String) -> FolderValidationResult {
    installer_validate_game(folder_path)
}

#[tauri::command]
pub fn detect_game_folder() -> Option<String> {
    installer_detect()
}

#[tauri::command]
pub async fn start_depot_download(
    app: AppHandle,
    state: State<'_, Arc<ActiveDownloadState>>,
    install_root: String,
    language_code: String,
    auth_method: Option<String>,
    steam_username: Option<String>,
    steam_password: Option<String>,
) -> Result<CommandResult, String> {
    let method = auth_method.unwrap_or_else(|| "qr".to_string());
    let state_clone = Arc::clone(&state);

    Ok(run_depot_download(
        app,
        state_clone,
        install_root,
        language_code,
        method,
        steam_username,
        steam_password,
    )
    .await)
}

#[tauri::command]
pub async fn cancel_depot_download(state: State<'_, Arc<ActiveDownloadState>>) -> Result<bool, String> {
    cancel_download(&state).await;
    Ok(true)
}

#[tauri::command]
pub async fn send_console_input(
    state: State<'_, Arc<ActiveDownloadState>>,
    text: String,
) -> Result<bool, String> {
    Ok(send_input(&state, &text).await)
}

#[tauri::command]
pub async fn install_dawn(app: AppHandle, game_root: String) -> Result<CommandResult, String> {
    Ok(installer_install(app, game_root).await)
}

#[tauri::command]
pub async fn restore_dawn(app: AppHandle, game_root: String) -> Result<CommandResult, String> {
    Ok(installer_restore(app, game_root).await)
}

#[tauri::command]
pub async fn uninstall_dawn(app: AppHandle, game_root: String) -> Result<CommandResult, String> {
    Ok(installer_uninstall(app, game_root).await)
}

#[tauri::command]
pub fn clear_cache(game_root: String) -> CommandResult {
    installer_clear_cache(game_root)
}

#[tauri::command]
pub fn launch_game(game_root: String, language_code: Option<String>) -> CommandResult {
    installer_launch(game_root, language_code)
}

#[tauri::command]
pub fn open_folder(target_path: String) -> bool {
    installer_open(target_path)
}

#[tauri::command]
pub fn open_docs() -> bool {
    installer_docs()
}

#[tauri::command]
pub async fn minimize_window(window: tauri::WebviewWindow) {
    let _ = window.minimize();
}

#[tauri::command]
pub async fn close_window(
    window: tauri::WebviewWindow,
    state: State<'_, Arc<ActiveDownloadState>>,
) -> Result<(), String> {
    cancel_download(&state).await;
    let _ = window.close();
    Ok(())
}

#[tauri::command]
pub fn get_debug_logs() -> Vec<String> {
    crate::logger::get_logs()
}

#[tauri::command]
pub fn open_log_file() -> Result<(), String> {
    let p = crate::logger::get_log_file_path();
    if !p.exists() {
        if let Some(parent) = p.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(&p, b"");
    }
    std::process::Command::new("explorer")
        .arg(&p)
        .spawn()
        .map_err(|e| format!("Failed to open log file: {}", e))?;
    Ok(())
}

#[tauri::command]
pub fn clear_debug_logs() -> Result<(), String> {
    crate::logger::clear_logs();
    Ok(())
}

#[tauri::command]
pub async fn get_latest_dawn_version() -> Option<String> {
    crate::dawn_release::get_latest_dawn_version().await
}


