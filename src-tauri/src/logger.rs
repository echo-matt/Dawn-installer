use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter};

static LOG_BUFFER: Mutex<Vec<String>> = Mutex::new(Vec::new());

pub fn get_log_dir() -> PathBuf {
    let base = std::env::var("LOCALAPPDATA")
        .or_else(|_| std::env::var("APPDATA"))
        .unwrap_or_else(|_| "C:\\AppData\\Local".to_string());
    PathBuf::from(base).join("DawnInstaller").join("logs")
}

pub fn get_log_file_path() -> PathBuf {
    get_log_dir().join("dawn-launcher.log")
}

fn get_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
    let total_secs = now.as_secs();
    let millis = now.subsec_millis();
    let secs = total_secs % 60;
    let mins = (total_secs / 60) % 60;
    let hours = (total_secs / 3600) % 24;
    format!("{:02}:{:02}:{:02}.{:03}", hours, mins, secs, millis)
}

pub fn log_msg(level: &str, msg: &str, app: Option<&AppHandle>) {
    let ts = get_timestamp();
    let line = format!("[{}] [{}] {}", ts, level, msg);

    // 1. Memory buffer
    if let Ok(mut buf) = LOG_BUFFER.lock() {
        if buf.len() >= 1000 {
            buf.remove(0);
        }
        buf.push(line.clone());
    }

    // 2. File log
    let file_path = get_log_file_path();
    if let Some(parent) = file_path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(&file_path) {
        let _ = writeln!(f, "{}", line);
    }

    // 3. Emit event to frontend
    if let Some(app_handle) = app {
        let _ = app_handle.emit("debug:log", line);
    }
}

pub fn get_logs() -> Vec<String> {
    if let Ok(buf) = LOG_BUFFER.lock() {
        buf.clone()
    } else {
        Vec::new()
    }
}

pub fn clear_logs() {
    if let Ok(mut buf) = LOG_BUFFER.lock() {
        buf.clear();
    }
    let file_path = get_log_file_path();
    let _ = fs::write(file_path, b"");
}
