mod app_updater;
mod commands;
mod constants;
mod dawn_release;
mod depot_service;
mod installer;
mod logger;
mod types;

use std::sync::Arc;
use depot_service::ActiveDownloadState;

#[cfg(windows)]
pub fn round_window_corners(window: &tauri::WebviewWindow) {
    if let Ok(hwnd) = window.hwnd() {
        let preference: u32 = 2; // DWMWCP_ROUND (Windows 11 rounded corners)
        const DWMWA_WINDOW_CORNER_PREFERENCE: u32 = 33;
        #[link(name = "dwmapi")]
        extern "system" {
            fn DwmSetWindowAttribute(
                hwnd: *mut std::ffi::c_void,
                dw_attribute: u32,
                pv_attribute: *const std::ffi::c_void,
                cb_attribute: u32,
            ) -> i32;
        }
        unsafe {
            let _ = DwmSetWindowAttribute(
                hwnd.0 as *mut std::ffi::c_void,
                DWMWA_WINDOW_CORNER_PREFERENCE,
                &preference as *const u32 as *const std::ffi::c_void,
                std::mem::size_of::<u32>() as u32,
            );
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let download_state = Arc::new(ActiveDownloadState::new());

    tauri::Builder::default()
        .manage(download_state)
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::select_game_folder,
            commands::get_installer_constants,
            commands::validate_preflight,
            commands::validate_game_folder,
            commands::detect_game_folder,
            commands::start_depot_download,
            commands::cancel_depot_download,
            commands::send_console_input,
            commands::install_dawn,
            commands::restore_dawn,
            commands::uninstall_dawn,
            commands::clear_cache,
            commands::launch_game,
            commands::open_folder,
            commands::open_docs,
            commands::minimize_window,
            commands::close_window,
            commands::get_debug_logs,
            commands::open_log_file,
            commands::clear_debug_logs,
            commands::get_latest_dawn_version,
            commands::get_app_version,
            commands::get_platform,
            commands::check_app_update,
            commands::install_app_update,
        ])
        .setup(|app| {
            #[cfg(windows)]
            {
                use tauri::Manager;
                if let Some(window) = app.get_webview_window("main") {
                    round_window_corners(&window);
                }
            }

            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
