use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::Command as TokioCommand;
use tokio::sync::Mutex;
use sha2::{Digest, Sha256};
use tauri::{AppHandle, Emitter};

use crate::constants::{
    BASE_DEPOT_ID, BASE_MANIFEST_ID, DEPOT_DOWNLOADER_EXE, DEPOT_DOWNLOADER_SHA256,
    DEPOT_DOWNLOADER_URL, DEPOT_DOWNLOADER_VERSION, DEPOT_DOWNLOADER_ZIP, STATIC_LANGUAGES,
    STEAM_APP_ID,
};
use crate::types::{AuthErrorPayload, CommandResult, ProgressPayload, SteamGuardPayload};

pub struct ActiveDownloadState {
    pub active_pid: Arc<Mutex<Option<u32>>>,
    pub child_stdin: Arc<Mutex<Option<tokio::process::ChildStdin>>>,
    pub is_cancelled: Arc<AtomicBool>,
}

impl ActiveDownloadState {
    pub fn new() -> Self {
        Self {
            active_pid: Arc::new(Mutex::new(None)),
            child_stdin: Arc::new(Mutex::new(None)),
            is_cancelled: Arc::new(AtomicBool::new(false)),
        }
    }
}

pub fn get_tools_dir() -> PathBuf {
    #[cfg(windows)]
    {
        let base = std::env::var("LOCALAPPDATA")
            .or_else(|_| std::env::var("APPDATA"))
            .unwrap_or_else(|_| "C:\\AppData\\Local".to_string());
        PathBuf::from(base)
            .join("DawnInstaller")
            .join("tools")
            .join("DepotDownloader")
            .join(DEPOT_DOWNLOADER_VERSION)
    }
    #[cfg(not(windows))]
    {
        let base = std::env::var("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
                PathBuf::from(home).join(".local").join("share")
            });
        base.join("DawnInstaller")
            .join("tools")
            .join("DepotDownloader")
            .join(DEPOT_DOWNLOADER_VERSION)
    }
}

pub async fn ensure_depot_downloader(app: &AppHandle) -> Result<PathBuf, String> {
    let exe_path = std::env::var_os("PATH").and_then(|paths| {
        std::env::split_paths(&paths).filter_map(|dir| {
            let exe_path = dir.join(DEPOT_DOWNLOADER_EXE);
            if exe_path.is_file() {
                Some(exe_path)
            } else {
                None
            }
        }).next()
    });

    if let Some(exe) = exe_path {
        return Ok(exe);
    }

    let tools_dir = get_tools_dir();
    let exe_path = tools_dir.join(DEPOT_DOWNLOADER_EXE);

    if exe_path.exists() {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(meta) = std::fs::metadata(&exe_path) {
                let mut perms = meta.permissions();
                perms.set_mode(0o755);
                let _ = std::fs::set_permissions(&exe_path, perms);
            }
        }
        return Ok(exe_path);
    }

    tokio::fs::create_dir_all(&tools_dir)
        .await
        .map_err(|e| format!("Failed to create tools directory: {}", e))?;

    let archive_path = tools_dir.join(DEPOT_DOWNLOADER_ZIP);

    let _ = app.emit("installer:progress", ProgressPayload {
        percent: 0,
        status: format!("Downloading DepotDownloader {}...", DEPOT_DOWNLOADER_VERSION),
    });

    let client = reqwest::Client::builder()
        .user_agent("DawnInstaller/1.0")
        .build()
        .map_err(|e| format!("Failed to initialize HTTP client: {}", e))?;

    let response = client
        .get(DEPOT_DOWNLOADER_URL)
        .send()
        .await
        .map_err(|e| format!("Failed to download DepotDownloader: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Download failed with HTTP {}", response.status()));
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read download stream: {}", e))?;

    tokio::fs::write(&archive_path, &bytes)
        .await
        .map_err(|e| format!("Failed to write archive: {}", e))?;

    // Verify SHA-256 if hash is configured
    if !DEPOT_DOWNLOADER_SHA256.is_empty() {
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let computed_hash = format!("{:x}", hasher.finalize());

        if computed_hash.to_lowercase() != DEPOT_DOWNLOADER_SHA256.to_lowercase() {
            let _ = tokio::fs::remove_file(&archive_path).await;
            return Err(format!(
                "Checksum mismatch! Expected {}, got {}",
                DEPOT_DOWNLOADER_SHA256, computed_hash
            ));
        }
    }

    let _ = app.emit("installer:progress", ProgressPayload {
        percent: 5,
        status: "Extracting DepotDownloader...".to_string(),
    });

    let mut tar_cmd = std::process::Command::new("tar");
    tar_cmd.args(["-xf", archive_path.to_str().unwrap(), "-C", tools_dir.to_str().unwrap()]);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        tar_cmd.creation_flags(0x08000000);
    }
    let tar_res = tar_cmd.output();

    let extracted = match tar_res {
        Ok(out) if out.status.success() => true,
        _ => {
            #[cfg(windows)]
            {
                use std::os::windows::process::CommandExt;
                let ps_script = format!(
                    "Expand-Archive -LiteralPath '{}' -DestinationPath '{}' -Force",
                    archive_path.to_str().unwrap(),
                    tools_dir.to_str().unwrap()
                );
                let mut ps_cmd = std::process::Command::new("powershell");
                ps_cmd.args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", &ps_script]);
                ps_cmd.creation_flags(0x08000000);
                let ps_res = ps_cmd.output();
                matches!(ps_res, Ok(out) if out.status.success())
            }
            #[cfg(not(windows))]
            {
                let mut unzip_cmd = std::process::Command::new("unzip");
                unzip_cmd.args(["-o", archive_path.to_str().unwrap(), "-d", tools_dir.to_str().unwrap()]);
                let unzip_res = unzip_cmd.output();
                if matches!(unzip_res, Ok(ref out) if out.status.success()) {
                    true
                } else {
                    let py_script = format!(
                        "import zipfile; zipfile.ZipFile('{}').extractall('{}')",
                        archive_path.to_str().unwrap(),
                        tools_dir.to_str().unwrap()
                    );
                    let mut py_cmd = std::process::Command::new("python3");
                    py_cmd.args(["-c", &py_script]);
                    matches!(py_cmd.output(), Ok(out) if out.status.success())
                }
            }
        }
    };

    let _ = tokio::fs::remove_file(&archive_path).await;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(meta) = std::fs::metadata(&exe_path) {
            let mut perms = meta.permissions();
            perms.set_mode(0o755);
            let _ = std::fs::set_permissions(&exe_path, perms);
        }
    }

    if !extracted || !exe_path.exists() {
        return Err(format!("Failed to extract {}", DEPOT_DOWNLOADER_EXE));
    }

    Ok(exe_path)
}

pub fn parse_qr_from_bytes(raw: &[u8]) -> Option<String> {
    let marker = b"Use the Steam Mobile App to sign in with this QR code:";
    let marker_idx = raw.windows(marker.len()).rposition(|w| w == marker)?;

    let after = &raw[marker_idx + marker.len()..];

    // Split by lines on raw bytes
    let mut byte_lines: Vec<&[u8]> = Vec::new();
    let mut curr = 0;
    for (i, &b) in after.iter().enumerate() {
        if b == b'\n' {
            let mut end = i;
            if end > curr && after[end - 1] == b'\r' {
                end -= 1;
            }
            byte_lines.push(&after[curr..end]);
            curr = i + 1;
        }
    }
    if curr < after.len() {
        byte_lines.push(&after[curr..]);
    }

    let n = 37;
    let cell_size = 6;
    let size = n * cell_size;

    // Look for the first line that is a QR code row (length >= 74, beginning with top quiet zone)
    let mut start_idx = None;
    for (i, line) in byte_lines.iter().enumerate() {
        if line.len() >= 74 {
            start_idx = Some(i);
            break;
        }
    }

    let start = start_idx?;

    // Must have all 37 rows completely received before generating an SVG
    // (Prevents emitting incomplete rows which caused rapid flickering)
    if byte_lines.len() < start + n {
        return None;
    }

    let mut rects = String::new();
    let mut dark_modules = 0;

    for (r, line) in byte_lines[start..start + n].iter().enumerate() {
        if line.len() < 74 {
            // Incomplete row still buffering
            return None;
        }
        for col in (0..74).step_by(2) {
            let b = line[col];
            if b == 0xDB || b == 219 || b == b'#' {
                let x = (col / 2) * cell_size;
                let y = r * cell_size;
                rects.push_str(&format!(
                    r##"<rect x="{}" y="{}" width="{}" height="{}" fill="#0f172a" />"##,
                    x, y, cell_size, cell_size
                ));
                dark_modules += 1;
            }
        }
    }

    // A valid Steam QR code contains at least 150 dark modules
    if dark_modules < 150 {
        return None;
    }

    Some(format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {size} {size}" width="{size}" height="{size}" shape-rendering="crispEdges"><rect width="{size}" height="{size}" fill="#ffffff" rx="10" />{rects}</svg>"##
    ))
}

pub async fn run_depot_download(
    app: AppHandle,
    state: Arc<ActiveDownloadState>,
    install_root: String,
    language_code: String,
    auth_method: String,
    steam_username: Option<String>,
    steam_password: Option<String>,
) -> CommandResult {
    state.is_cancelled.store(false, Ordering::SeqCst);
    crate::logger::log_msg(
        "INFO",
        &format!(
            "Download initiated: install_root='{}', language='{}', auth_method='{}'",
            install_root, language_code, auth_method
        ),
        Some(&app),
    );

    let exe_path = match ensure_depot_downloader(&app).await {
        Ok(p) => p,
        Err(e) => {
            let _ = app.emit("depot:output", format!("\r\n[ERROR] {}\r\n", e));
            return CommandResult {
                success: false,
                message: None,
                error: Some(e),
                cancelled: Some(false),
                count: None,
            };
        }
    };

    let lang_info = STATIC_LANGUAGES
        .iter()
        .find(|l| l.code == language_code)
        .unwrap_or(&STATIC_LANGUAGES[0]);

    let target_dir = Path::new(&install_root);
    let _ = tokio::fs::create_dir_all(target_dir).await;

    // Single unified download session for Base Game + Selected Language Depot
    let _ = app.emit(
        "depot:output",
        format!(
            "\r\n[STEP 1/2] Downloading Base Game & {} Language Depot...\r\n",
            lang_info.name
        ),
    );

    let depots = [
        (BASE_DEPOT_ID, BASE_MANIFEST_ID),
        (lang_info.depot_id, lang_info.manifest_id),
    ];

    let download_res = execute_depot_step(
        &app,
        &state,
        &exe_path,
        &install_root,
        &depots,
        &auth_method,
        steam_username.as_deref(),
        steam_password.as_deref(),
        &format!("Game & {}", lang_info.name),
        0,
        90,
    )
    .await;

    if let Some(true) = download_res.cancelled {
        let _ = app.emit("depot:output", "\r\n[CANCELLED] Download was cancelled by user.\r\n");
        return download_res;
    }
    if !download_res.success {
        return download_res;
    }

    if state.is_cancelled.load(Ordering::SeqCst) {
        let _ = app.emit("depot:output", "\r\n[CANCELLED] Download was cancelled by user.\r\n");
        return CommandResult {
            success: false,
            message: None,
            error: Some("Download was cancelled by user.".to_string()),
            cancelled: Some(true),
            count: None,
        };
    }

    // Step 2: Fetch & Deploy Latest Dawn Mod Release
    let _ = app.emit(
        "depot:output",
        "\r\n--- Step 2/2: Fetching & Deploying Latest Dawn Mod Release ---\r\n",
    );

    let release_dir = match crate::dawn_release::ensure_latest_dawn_release(&app).await {
        Ok(dir) => dir,
        Err(e) => {
            let _ = app.emit(
                "depot:output",
                format!("[WARN] Failed to fetch latest release: {}. Using bundled fallback...\r\n", e),
            );
            crate::installer::get_bundled_payload_dir()
        }
    };

    if let Err(e) = crate::dawn_release::deploy_dawn_to_game(&app, &release_dir, &install_root).await {
        let _ = app.emit(
            "depot:output",
            format!("[ERROR] Deployment error: {}\r\n", e),
        );
    }

    // Step 3: Configure Dawn settings.json with the selected language so steam_api64.dll sets in-game language
    crate::installer::update_dawn_language(&install_root, &language_code);
    let _ = app.emit(
        "depot:output",
        format!("[CONFIG] Game language configured to '{}'\r\n", language_code),
    );

    CommandResult {
        success: true,
        message: Some(format!("Destiny 2 ({}) & Dawn installed successfully!", lang_info.name)),
        error: None,
        cancelled: Some(false),
        count: None,
    }
}

async fn execute_depot_step(
    app: &AppHandle,
    state: &Arc<ActiveDownloadState>,
    exe_path: &Path,
    install_root: &str,
    depots: &[(u64, &str)],
    auth_method: &str,
    steam_username: Option<&str>,
    steam_password: Option<&str>,
    step_label: &str,
    percent_start: u32,
    percent_end: u32,
) -> CommandResult {
    let mut args = vec![
        "-app".to_string(),
        STEAM_APP_ID.to_string(),
    ];

    args.push("-depot".to_string());
    for (d_id, _) in depots {
        args.push(d_id.to_string());
    }

    args.push("-manifest".to_string());
    for (_, m_id) in depots {
        args.push(m_id.to_string());
    }

    args.extend_from_slice(&[
        "-dir".to_string(),
        install_root.to_string(),
        "-os".to_string(),
        "windows".to_string(),
        "-osarch".to_string(),
        "64".to_string(),
        "-validate".to_string(),
        "-max-servers".to_string(),
        "20".to_string(),
        "-max-downloads".to_string(),
        "16".to_string(),
    ]);

    if auth_method == "qr" {
        args.push("-qr".to_string());
    } else if let (Some(u), Some(p)) = (steam_username, steam_password) {
        if !u.is_empty() {
            args.push("-username".to_string());
            args.push(u.to_string());
            args.push("-password".to_string());
            args.push(p.to_string());
            args.push("-remember-password".to_string());
        }
    }

    let mut cmd = TokioCommand::new(exe_path);
    cmd.args(&args)
        .current_dir(install_root)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    #[cfg(windows)]
    cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            return CommandResult {
                success: false,
                message: None,
                error: Some(format!("Failed to spawn DepotDownloader: {}", e)),
                cancelled: Some(false),
                count: None,
            };
        }
    };

    let pid = child.id().unwrap_or(0);
    *state.active_pid.lock().await = Some(pid);
    *state.child_stdin.lock().await = child.stdin.take();

    let mut stdout = child.stdout.take().expect("Failed to open stdout");
    let mut stderr = child.stderr.take().expect("Failed to open stderr");

    let progress_regex = regex::Regex::new(r"(\d+(\.\d+)?)%").unwrap();
    let mut raw_buffer = Vec::new();
    let mut last_qr = String::new();
    let mut stdout_buf = [0u8; 4096];
    let mut stderr_buf = [0u8; 4096];

    let mut detected_auth_error: Option<String> = None;
    let mut detected_auth_type: Option<String> = None;
    let mut detected_general_error: Option<String> = None;

    let depots_desc = depots
        .iter()
        .map(|(d, m)| format!("{}:{}", d, m))
        .collect::<Vec<_>>()
        .join(", ");

    crate::logger::log_msg(
        "INFO",
        &format!(
            "Starting {} [Depots: [{}], Auth: {}]",
            step_label, depots_desc, auth_method
        ),
        Some(app),
    );

    loop {
        tokio::select! {
            res = stdout.read(&mut stdout_buf) => {
                match res {
                    Ok(0) => break,
                    Ok(n) => {
                        let chunk = &stdout_buf[..n];
                        raw_buffer.extend_from_slice(chunk);
                        let text = String::from_utf8_lossy(chunk).to_string();
                        let _ = app.emit("depot:output", text.clone());

                        // Log to debug logger (skip pure progress spam)
                        for line in text.lines() {
                            let trimmed = line.trim();
                            if !trimmed.is_empty() {
                                if !progress_regex.is_match(trimmed) || trimmed.contains("Error") || trimmed.contains("Failed") {
                                    crate::logger::log_msg("DEPOT", trimmed, Some(app));
                                }
                            }
                        }

                        // Inspect for Steam authentication errors
                        let lower = text.to_lowercase();
                        if lower.contains("invalidpassword") {
                            detected_auth_type = Some("invalid_password".to_string());
                            detected_auth_error = Some("Incorrect Steam password or username. Please check your credentials and try again.".to_string());
                        } else if lower.contains("twofactorcodemismatch") {
                            detected_auth_type = Some("2fa_mismatch".to_string());
                            detected_auth_error = Some("Incorrect Steam Guard code entered. Please try again.".to_string());
                        } else if lower.contains("ratelimitexceeded") {
                            detected_auth_type = Some("rate_limit".to_string());
                            detected_auth_error = Some("Steam login rate limit exceeded. Please wait a few minutes before trying again.".to_string());
                        } else if lower.contains("accountlogondenied") {
                            detected_auth_type = Some("logon_denied".to_string());
                            detected_auth_error = Some("Steam Guard access denied. Please check your Steam Guard configuration.".to_string());
                        } else if lower.contains("timed out waiting for confirmation") || (lower.contains("timeout") && lower.contains("failed to authenticate")) {
                            detected_auth_type = Some("timeout".to_string());
                            detected_auth_error = Some("Authentication timed out waiting for Steam Guard confirmation. Please try again.".to_string());
                        } else if lower.contains("failed to authenticate with steam") && detected_auth_error.is_none() {
                            detected_auth_type = Some("auth_failed".to_string());
                            detected_auth_error = Some("Failed to authenticate with Steam. Please check your credentials.".to_string());
                        }

                        if (lower.contains("error:") || lower.contains("fatal:") || lower.contains("exception:")) && detected_general_error.is_none() {
                            detected_general_error = Some(text.trim().to_string());
                        }

                        // Check QR code
                        if let Some(svg) = parse_qr_from_bytes(&raw_buffer) {
                            if svg != last_qr {
                                last_qr = svg.clone();
                                let _ = app.emit("depot:qr-code", svg);
                            }
                        }

                        // Check Steam Guard ONLY for credentials logins (never in QR mode),
                        // and never trigger on the QR instruction text!
                        if auth_method != "qr" {
                            if lower.contains("2 factor auth code")
                                || lower.contains("steam guard code")
                                || lower.contains("verification code")
                                || lower.contains("auth code")
                                || lower.contains("two-factor") {
                                let _ = app.emit("depot:steam-guard", SteamGuardPayload {
                                    guard_type: "code".to_string(),
                                    message: if lower.contains("email") {
                                        "Enter the Steam Guard code sent to your email:".to_string()
                                    } else {
                                        "Enter your Steam Guard Mobile Authenticator code:".to_string()
                                    }
                                });
                            } else if lower.contains("approve the login")
                                || lower.contains("mobile confirmation")
                                || lower.contains("confirm on your phone")
                                || lower.contains("waiting for mobile") {
                                let _ = app.emit("depot:steam-guard", SteamGuardPayload {
                                    guard_type: "mobile_confirm".to_string(),
                                    message: "Please approve the login request on your Steam Mobile App".to_string()
                                });
                            }
                        }

                        // Check Progress
                        if let Some(caps) = progress_regex.captures(&text) {
                            if let Some(m) = caps.get(1) {
                                if let Ok(parsed_pct) = m.as_str().parse::<f64>() {
                                    let scaled = percent_start as f64
                                        + (parsed_pct / 100.0) * (percent_end - percent_start) as f64;
                                    let status = format!("{} {:.1}%", step_label, parsed_pct);
                                    let _ = app.emit("depot:progress", ProgressPayload {
                                        percent: scaled.round() as u32,
                                        status,
                                    });
                                }
                            }
                        }
                    }
                    Err(_) => break,
                }
            }
            res_err = stderr.read(&mut stderr_buf) => {
                match res_err {
                    Ok(0) => {},
                    Ok(n) => {
                        let text = String::from_utf8_lossy(&stderr_buf[..n]).to_string();
                        let _ = app.emit("depot:output", text.clone());
                        for line in text.lines() {
                            let trimmed = line.trim();
                            if !trimmed.is_empty() {
                                crate::logger::log_msg("DEPOT_ERR", trimmed, Some(app));
                            }
                        }

                        let lower = text.to_lowercase();
                        if lower.contains("invalidpassword") {
                            detected_auth_type = Some("invalid_password".to_string());
                            detected_auth_error = Some("Incorrect Steam password or username. Please check your credentials and try again.".to_string());
                        } else if lower.contains("twofactorcodemismatch") {
                            detected_auth_type = Some("2fa_mismatch".to_string());
                            detected_auth_error = Some("Incorrect Steam Guard code entered. Please try again.".to_string());
                        } else if lower.contains("ratelimitexceeded") {
                            detected_auth_type = Some("rate_limit".to_string());
                            detected_auth_error = Some("Steam login rate limit exceeded. Please wait a few minutes before trying again.".to_string());
                        } else if lower.contains("accountlogondenied") {
                            detected_auth_type = Some("logon_denied".to_string());
                            detected_auth_error = Some("Steam Guard access denied. Please check your Steam Guard configuration.".to_string());
                        } else if lower.contains("timed out waiting for confirmation") || (lower.contains("timeout") && lower.contains("failed to authenticate")) {
                            detected_auth_type = Some("timeout".to_string());
                            detected_auth_error = Some("Authentication timed out waiting for Steam Guard confirmation. Please try again.".to_string());
                        } else if lower.contains("failed to authenticate with steam") && detected_auth_error.is_none() {
                            detected_auth_type = Some("auth_failed".to_string());
                            detected_auth_error = Some("Failed to authenticate with Steam. Please check your credentials.".to_string());
                        }

                        if (lower.contains("error:") || lower.contains("fatal:") || lower.contains("exception:")) && detected_general_error.is_none() {
                            detected_general_error = Some(text.trim().to_string());
                        }
                    }
                    Err(_) => {}
                }
            }
            status = child.wait() => {
                let exit_code = status.map(|s| s.code().unwrap_or(-1)).unwrap_or(-1);
                *state.active_pid.lock().await = None;
                *state.child_stdin.lock().await = None;

                if state.is_cancelled.load(Ordering::SeqCst) {
                    crate::logger::log_msg("INFO", &format!("{} was cancelled by user.", step_label), Some(app));
                    return CommandResult {
                        success: false,
                        message: None,
                        error: Some("Download was cancelled by user.".to_string()),
                        cancelled: Some(true),
                        count: None,
                    };
                }

                if exit_code == 0 {
                    crate::logger::log_msg("INFO", &format!("{} completed successfully (exit code 0).", step_label), Some(app));
                    return CommandResult {
                        success: true,
                        message: Some(format!("{} completed successfully.", step_label)),
                        error: None,
                        cancelled: Some(false),
                        count: None,
                    };
                } else {
                    if let Some(err_msg) = detected_auth_error.clone() {
                        let auth_type = detected_auth_type.unwrap_or_else(|| "auth_error".to_string());
                        crate::logger::log_msg("ERROR", &format!("DepotDownloader auth error ({}): {}", auth_type, err_msg), Some(app));
                        let _ = app.emit("depot:auth-error", AuthErrorPayload {
                            error_type: auth_type,
                            message: err_msg.clone(),
                        });
                        return CommandResult {
                            success: false,
                            message: None,
                            error: Some(err_msg),
                            cancelled: Some(false),
                            count: None,
                        };
                    }

                    let err_msg = detected_general_error
                        .unwrap_or_else(|| format!("DepotDownloader exited with error code {}.", exit_code));
                    crate::logger::log_msg("ERROR", &format!("DepotDownloader failed: {}", err_msg), Some(app));
                    return CommandResult {
                        success: false,
                        message: None,
                        error: Some(err_msg),
                        cancelled: Some(false),
                        count: None,
                    };
                }
            }
        }

        if state.is_cancelled.load(Ordering::SeqCst) {
            let mut pid_guard = state.active_pid.lock().await;
            if let Some(pid) = *pid_guard {
                #[cfg(windows)]
                {
                    use std::os::windows::process::CommandExt;
                    let mut k = std::process::Command::new("taskkill");
                    k.args(["/PID", &pid.to_string(), "/T", "/F"]).creation_flags(0x08000000);
                    let _ = k.output();
                }
                #[cfg(not(windows))]
                {
                    unsafe {
                        libc::kill(pid as i32, libc::SIGKILL);
                    }
                }
                *pid_guard = None;
            }
            #[cfg(windows)]
            {
                use std::os::windows::process::CommandExt;
                let mut k = std::process::Command::new("taskkill");
                k.args(["/IM", "DepotDownloader.exe", "/T", "/F"]).creation_flags(0x08000000);
                let _ = k.output();
            }
            #[cfg(not(windows))]
            {
                let _ = std::process::Command::new("pkill")
                    .args(["-9", "-f", "DepotDownloader"])
                    .output();
            }
            crate::logger::log_msg("INFO", "Download cancelled by user; processes terminated.", Some(app));
            return CommandResult {
                success: false,
                message: None,
                error: Some("Download was cancelled by user.".to_string()),
                cancelled: Some(true),
                count: None,
            };
        }
    }

    *state.active_pid.lock().await = None;
    *state.child_stdin.lock().await = None;

    if state.is_cancelled.load(Ordering::SeqCst) {
        CommandResult {
            success: false,
            message: None,
            error: Some("Download was cancelled by user.".to_string()),
            cancelled: Some(true),
            count: None,
        }
    } else if let Some(err_msg) = detected_auth_error {
        let auth_type = detected_auth_type.unwrap_or_else(|| "auth_error".to_string());
        let _ = app.emit("depot:auth-error", AuthErrorPayload {
            error_type: auth_type,
            message: err_msg.clone(),
        });
        CommandResult {
            success: false,
            message: None,
            error: Some(err_msg),
            cancelled: Some(false),
            count: None,
        }
    } else {
        CommandResult {
            success: true,
            message: Some(format!("{} finished.", step_label)),
            error: None,
            cancelled: Some(false),
            count: None,
        }
    }
}

pub async fn send_input(state: &Arc<ActiveDownloadState>, input: &str) -> bool {
    let mut stdin_guard = state.child_stdin.lock().await;
    if let Some(stdin) = stdin_guard.as_mut() {
        let msg = format!("{}\r\n", input.trim());
        let _ = stdin.write_all(msg.as_bytes()).await;
        let _ = stdin.flush().await;
        true
    } else {
        false
    }
}

pub async fn cancel_download(state: &Arc<ActiveDownloadState>) {
    state.is_cancelled.store(true, Ordering::SeqCst);
    let mut pid_guard = state.active_pid.lock().await;
    let had_pid = pid_guard.is_some();
    if let Some(pid) = *pid_guard {
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            let mut k = std::process::Command::new("taskkill");
            k.args(["/PID", &pid.to_string(), "/T", "/F"]).creation_flags(0x08000000);
            let _ = k.output();
        }
        #[cfg(not(windows))]
        {
            unsafe {
                libc::kill(pid as i32, libc::SIGKILL);
            }
        }
        *pid_guard = None;
    }
    // Only kill DepotDownloader process instances if a process was actively running
    if had_pid {
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            let mut k = std::process::Command::new("taskkill");
            k.args(["/IM", "DepotDownloader.exe", "/T", "/F"]).creation_flags(0x08000000);
            let _ = k.output();
        }
        #[cfg(not(windows))]
        {
            let _ = std::process::Command::new("pkill")
                .args(["-9", "-f", "DepotDownloader"])
                .output();
        }
    }
    *state.child_stdin.lock().await = None;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_qr_from_bytes() {
        let mut data = Vec::new();
        data.extend_from_slice(b"Connecting to Steam3... Done!\r\nLogging in with QR code...\r\n");
        data.extend_from_slice(b"Use the Steam Mobile App to sign in with this QR code:\r\n");

        // 4 quiet zone rows of 74 spaces
        for _ in 0..4 {
            data.extend_from_slice(&[b' '; 74]);
            data.extend_from_slice(b"\r\n");
        }

        // 29 data rows with 0xDB (full blocks)
        for _ in 0..29 {
            let mut row = vec![b' '; 74];
            for c in (0..74).step_by(2) {
                row[c] = 0xDB;
            }
            data.extend_from_slice(&row);
            data.extend_from_slice(b"\r\n");
        }

        // 4 quiet zone rows of 74 spaces
        for _ in 0..4 {
            data.extend_from_slice(&[b' '; 74]);
            data.extend_from_slice(b"\r\n");
        }

        let result = parse_qr_from_bytes(&data);
        assert!(result.is_some(), "Expected QR code SVG to be generated");
        let svg = result.unwrap();
        assert!(svg.starts_with("<svg"), "Output must be an SVG element");
        assert!(svg.contains(r###"fill="#ffffff""###), "SVG must contain white background");
        assert!(svg.contains(r###"fill="#0f172a""###), "SVG must contain dark module rects");
    }
}
