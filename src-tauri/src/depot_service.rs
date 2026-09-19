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

    // QRCoder (used by DepotDownloader's DisplayQrCode => AsciiQRCode.GetLineByLineGraphic(1, drawQuietZones:true))
    // renders every QR module as two characters: dark="██", light="  ".
    // The on-disk byte width of those characters depends on the console output encoding:
    //   - Windows OEM code page: '█' = single byte 0xDB  (module = 2x0xDB)
    //   - Linux/macOS UTF-8:     '█' = 0xE2 0x96 0x88   (module = 2x that sequence)
    // So we detect tokens by their actual byte length rather than assuming 2 bytes/module,
    // making the parser encoding-agnostic.
    fn token_len(line: &[u8]) -> usize {
        if line.len() >= 3 && line[0] == 0xE2 && line[1] == 0x96 && line[2] == 0x88 {
            3
        } else {
            1
        }
    }
    fn is_dark_token(line: &[u8]) -> bool {
        if line[0] == 0xDB || line[0] == b'#' {
            return true;
        }
        line.len() >= 3 && line[0] == 0xE2 && line[1] == 0x96 && line[2] == 0x88
    }
    // Decode one rendered row into per-module dark flags, consuming two chars per module.
    // Modules are char pairs; because a UTF-8 block is 3 bytes and a space is 1 byte,
    // consuming exactly two "tokens" per module always lands on a module boundary.
    fn decode_row(line: &[u8]) -> Option<Vec<bool>> {
        if line.is_empty() {
            return None;
        }
        let mut modules = Vec::new();
        let mut i = 0;
        while i < line.len() {
            let dark = is_dark_token(&line[i..]);
            i += token_len(&line[i..]);
            if i < line.len() {
                i += token_len(&line[i..]);
            }
            modules.push(dark);
        }
        Some(modules)
    }

    // n = 37 modules square (version-3 QR incl. 4-module quiet zones baked into the matrix).
    // Find the first row that is a complete 37-module line (a fully-received top quiet zone).
    let n = 37;
    let start_idx = byte_lines
        .iter()
        .position(|l| decode_row(l).map(|m| m.len() >= n).unwrap_or(false))?;

    // Must have all n rows completely received before generating an SVG
    // (Prevents emitting incomplete rows which caused rapid flickering)
    if byte_lines.len() < start_idx + n {
        return None;
    }

    let cell_size = 6;
    let size = n * cell_size;

    let mut rects = String::new();
    let mut dark_modules = 0;

    for (r, line) in byte_lines[start_idx..start_idx + n].iter().enumerate() {
        let modules = decode_row(line)?;
        if modules.len() < n {
            // Incomplete row still buffering
            return None;
        }
        for (col, dark) in modules.iter().take(n).enumerate() {
            if *dark {
                let x = col * cell_size;
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

#[derive(Debug, PartialEq, Eq)]
pub enum AuthInspectionResult {
    None,
    AuthSuccess,
    AuthError { error_type: String, message: String },
    SteamGuardPrompt { guard_type: String, message: String },
}

pub fn inspect_depot_auth(
    rolling_lower: &str,
    auth_method: &str,
    auth_succeeded: bool,
    has_auth_error: bool,
    steam_guard_prompted: bool,
) -> AuthInspectionResult {
    if rolling_lower.contains("invalidpassword") {
        return AuthInspectionResult::AuthError {
            error_type: "invalid_password".to_string(),
            message: "Incorrect Steam password or username. Please check your credentials and try again.".to_string(),
        };
    }
    if rolling_lower.contains("twofactorcodemismatch") {
        return AuthInspectionResult::AuthError {
            error_type: "2fa_mismatch".to_string(),
            message: "Incorrect Steam Guard code entered. Please try again.".to_string(),
        };
    }
    if rolling_lower.contains("ratelimitexceeded") {
        return AuthInspectionResult::AuthError {
            error_type: "rate_limit".to_string(),
            message: "Steam login rate limit exceeded. Please wait a few minutes before trying again.".to_string(),
        };
    }
    if rolling_lower.contains("accountlogondenied") {
        return AuthInspectionResult::AuthError {
            error_type: "logon_denied".to_string(),
            message: "Steam Guard access denied. Please check your Steam Guard configuration.".to_string(),
        };
    }
    if rolling_lower.contains("timed out waiting for confirmation")
        || (rolling_lower.contains("timeout") && rolling_lower.contains("failed to authenticate"))
    {
        return AuthInspectionResult::AuthError {
            error_type: "timeout".to_string(),
            message: "Authentication timed out waiting for Steam Guard confirmation. Please try again.".to_string(),
        };
    }
    if rolling_lower.contains("failed to authenticate with steam") && !has_auth_error {
        return AuthInspectionResult::AuthError {
            error_type: "auth_failed".to_string(),
            message: "Failed to authenticate with Steam. Please check your credentials.".to_string(),
        };
    }

    // 2. Check Steam Guard prompts first so pending 2FA prompts take precedence
    if auth_method != "qr" && !auth_succeeded && !has_auth_error && !steam_guard_prompted {
        let is_email_guard = rolling_lower.contains("sent to your email")
            || (rolling_lower.contains("steam guard") && rolling_lower.contains("email"))
            || (rolling_lower.contains("authentication code") && rolling_lower.contains("email"));

        let is_app_guard = rolling_lower.contains("2 factor")
            || rolling_lower.contains("two-factor")
            || rolling_lower.contains("authenticator app")
            || rolling_lower.contains("auth code");

        let is_general_guard = rolling_lower.contains("steam guard")
            && (rolling_lower.contains("code") || rolling_lower.contains("enter"));

        let is_mobile_approval = rolling_lower.contains("approve the login")
            || rolling_lower.contains("mobile confirmation")
            || rolling_lower.contains("confirm on your phone")
            || rolling_lower.contains("waiting for mobile")
            || (rolling_lower.contains("waiting for confirmation") && !rolling_lower.contains("timed out"));

        if is_email_guard {
            return AuthInspectionResult::SteamGuardPrompt {
                guard_type: "code".to_string(),
                message: "Enter the Steam Guard code sent to your email address:".to_string(),
            };
        }
        if is_app_guard || is_general_guard {
            return AuthInspectionResult::SteamGuardPrompt {
                guard_type: "code".to_string(),
                message: if rolling_lower.contains("email") {
                    "Enter the Steam Guard code sent to your email address:".to_string()
                } else {
                    "Enter your Steam Guard Mobile Authenticator code:".to_string()
                },
            };
        }
        if is_mobile_approval {
            return AuthInspectionResult::SteamGuardPrompt {
                guard_type: "mobile_confirm".to_string(),
                message: "Please approve the login request on your Steam Mobile App".to_string(),
            };
        }
    }

    // 3. Check authentication success (must NOT match "Connecting to Steam3... Done!")
    if !auth_succeeded && !has_auth_error {
        let is_login_done = rolling_lower.contains("with qr code... done")
            || rolling_lower.contains("with qr code...done")
            || rolling_lower.contains("into steam3... done")
            || rolling_lower.contains("into steam3...done")
            || rolling_lower.contains("got login token")
            || rolling_lower.contains("got account info")
            || rolling_lower.contains("requesting app info")
            || rolling_lower.contains("processing depot")
            || rolling_lower.contains("requesting depot info")
            || rolling_lower.contains("requesting manifest");

        if is_login_done {
            return AuthInspectionResult::AuthSuccess;
        }
    }

    AuthInspectionResult::None
}

pub fn get_saved_steam_username() -> Option<String> {
    #[cfg(windows)]
    {
        let local_appdata = std::env::var("LOCALAPPDATA").ok()?;
        let iso_dir = PathBuf::from(local_appdata).join("IsolatedStorage");
        if !iso_dir.is_dir() {
            return None;
        }

        fn search_account_config(dir: &Path) -> Option<PathBuf> {
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        if let Some(found) = search_account_config(&path) {
                            return Some(found);
                        }
                    } else if path.file_name().map(|n| n == "account.config").unwrap_or(false) {
                        return Some(path);
                    }
                }
            }
            None
        }

        let config_path = search_account_config(&iso_dir)?;
        let raw_bytes = std::fs::read(&config_path).ok()?;
        let decompressed = miniz_oxide::inflate::decompress_to_vec(&raw_bytes).ok()?;

        for i in 0..decompressed.len().saturating_sub(4) {
            if decompressed[i] == 0x0A {
                let len = decompressed[i + 1] as usize;
                if len >= 3 && len <= 32 && i + 2 + len < decompressed.len() {
                    let candidate = &decompressed[i + 2..i + 2 + len];
                    if decompressed[i + 2 + len] == 0x12 {
                        if let Ok(s) = std::str::from_utf8(candidate) {
                            if s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                                return Some(s.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    None
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
        "-max-downloads".to_string(),
        "16".to_string(),
    ]);

    if auth_method == "qr" {
        args.push("-qr".to_string());
    } else if let (Some(u), Some(p)) = (steam_username, steam_password) {
        let trimmed_user = u.trim();
        if !trimmed_user.is_empty() {
            args.push("-username".to_string());
            args.push(trimmed_user.to_string());
            args.push("-password".to_string());
            args.push(p.to_string());
            args.push("-no-mobile".to_string());
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
    let mut auth_succeeded = false;
    let mut steam_guard_prompted = false;
    let mut rolling_stdout = String::new();

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

                        // Keep rolling window for prompt & state detection across chunk boundaries
                        rolling_stdout.push_str(&text);
                        if rolling_stdout.len() > 8192 {
                            let keep_from = rolling_stdout.len() - 4096;
                            rolling_stdout = rolling_stdout[keep_from..].to_string();
                        }

                        let lower = text.to_lowercase();
                        let rolling_lower = rolling_stdout.to_lowercase();

                        // Detect and emit Steam account name if provided by DepotDownloader (e.g. after QR login)
                        if let Some(pos) = lower.find("next time you can login with -username ") {
                            let after = &text[pos + "next time you can login with -username ".len()..];
                            if let Some(user) = after.split_whitespace().next() {
                                let clean_user = user.trim().trim_matches('"').trim_matches('\'');
                                let _ = app.emit("depot:steam-username", clean_user.to_string());
                            }
                        }

                        // Inspect authentication state (errors, success, Steam Guard prompts)
                        match inspect_depot_auth(
                            &rolling_lower,
                            auth_method,
                            auth_succeeded,
                            detected_auth_error.is_some(),
                            steam_guard_prompted,
                        ) {
                            AuthInspectionResult::AuthError { error_type, message } => {
                                detected_auth_type = Some(error_type);
                                detected_auth_error = Some(message);
                            }
                            AuthInspectionResult::AuthSuccess => {
                                auth_succeeded = true;
                                crate::logger::log_msg("INFO", "Steam authentication successful. Emitting depot:auth-success.", Some(app));
                                let _ = app.emit("depot:auth-success", ());
                            }
                            AuthInspectionResult::SteamGuardPrompt { guard_type, message } => {
                                steam_guard_prompted = true;
                                crate::logger::log_msg("INFO", &format!("Detected Steam Guard prompt ({}).", guard_type), Some(app));
                                let _ = app.emit("depot:steam-guard", SteamGuardPayload {
                                    guard_type,
                                    message,
                                });
                            }
                            AuthInspectionResult::None => {}
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
        let msg = if cfg!(windows) {
            format!("{}\r\n", input.trim())
        } else {
            format!("{}\n", input.trim())
        };
        if let Err(e) = stdin.write_all(msg.as_bytes()).await {
            eprintln!("[ERROR] Failed to write to child stdin: {}", e);
            return false;
        }
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
        data.extend_from_slice(b"Connecting to Steam3... Done!
Logging in with QR code...
");
        data.extend_from_slice(b"Use the Steam Mobile App to sign in with this QR code:
");

        // 4 quiet zone rows of 74 spaces
        for _ in 0..4 {
            data.extend_from_slice(&[b' '; 74]);
            data.extend_from_slice(b"
");
        }

        // 29 data rows with 0xDB (full blocks)
        for _ in 0..29 {
            let mut row = vec![b' '; 74];
            for c in (0..74).step_by(2) {
                row[c] = 0xDB;
            }
            data.extend_from_slice(&row);
            data.extend_from_slice(b"
");
        }

        // 4 quiet zone rows of 74 spaces
        for _ in 0..4 {
            data.extend_from_slice(&[b' '; 74]);
            data.extend_from_slice(b"
");
        }

        let result = parse_qr_from_bytes(&data);
        assert!(result.is_some(), "Expected QR code SVG to be generated");
        let svg = result.unwrap();
        assert!(svg.starts_with("<svg"), "Output must be an SVG element");
        assert!(svg.contains(r###"fill="#ffffff""###), "SVG must contain white background");
        assert!(svg.contains(r###"fill="#0f172a""###), "SVG must contain dark module rects");
    }

    #[test]
    fn test_parse_qr_from_bytes_linux_utf8() {
        // QRCoder emits the same 37x37 matrix on Linux, but the dark module char '█'
        // is encoded as UTF-8 (0xE2 0x96 0x88) instead of the Windows OEM single byte 0xDB,
        // and lines end with LF only (no CR). This simulates real DepotDownloader output
        // piped from a Linux console.
        let mut data = Vec::new();
        data.extend_from_slice(b"Connecting to Steam3... Done!
Logging in with QR code...
");
        data.extend_from_slice(b"Use the Steam Mobile App to sign in with this QR code:
");

        // A module is QRCoder's "██" (dark = 2x [0xE2 0x96 0x88]) or "  " (light = 2x space).
        let dark_module: &[u8] = &[0xE2, 0x96, 0x88, 0xE2, 0x96, 0x88]; // "██"
        let light_module: &[u8] = &[b' ', b' ']; // "  "

        // 4 quiet zone rows (all light)
        for _ in 0..4 {
            let mut row = Vec::new();
            for _ in 0..37 {
                row.extend_from_slice(light_module);
            }
            row.extend_from_slice(b"
");
            data.extend_from_slice(&row);
        }

        // 29 data rows: alternate dark/light modules (even index = dark)
        for _ in 0..29 {
            let mut row = Vec::new();
            for m in 0..37 {
                if m % 2 == 0 {
                    row.extend_from_slice(dark_module);
                } else {
                    row.extend_from_slice(light_module);
                }
            }
            row.extend_from_slice(b"
");
            data.extend_from_slice(&row);
        }

        // 4 quiet zone rows (all light)
        for _ in 0..4 {
            let mut row = Vec::new();
            for _ in 0..37 {
                row.extend_from_slice(light_module);
            }
            row.extend_from_slice(b"
");
            data.extend_from_slice(&row);
        }

        let result = parse_qr_from_bytes(&data);
        assert!(result.is_some(), "Expected QR code SVG to be generated for Linux UTF-8 output");
        let svg = result.unwrap();
        assert!(svg.starts_with("<svg"), "Output must be an SVG element");
        assert!(svg.contains(r###"fill="#ffffff""###), "SVG must contain white background");
        assert!(svg.contains(r###"fill="#0f172a""###), "SVG must contain dark module rects");
    }

    #[test]
    fn test_inspect_depot_auth_email_guard() {
        // DepotDownloader email prompt format
        let output = "Connecting to Steam3... Done!
Logging 'destinyuser' to Steam3... This account is protected by Steam Guard. Please enter the authentication code sent to your email address: ".to_lowercase();

        let res = inspect_depot_auth(&output, "credentials", false, false, false);
        match res {
            AuthInspectionResult::SteamGuardPrompt { guard_type, message } => {
                assert_eq!(guard_type, "code");
                assert!(message.to_lowercase().contains("email"));
            }
            other => panic!("Expected SteamGuardPrompt with email, got {:?}", other),
        }
    }

    #[test]
    fn test_inspect_depot_auth_app_2fa() {
        let output = "Connecting to Steam3... Done!
Logging 'destinyuser' to Steam3... Please enter your 2 factor auth code from your authenticator app: ".to_lowercase();

        let res = inspect_depot_auth(&output, "credentials", false, false, false);
        match res {
            AuthInspectionResult::SteamGuardPrompt { guard_type, message } => {
                assert_eq!(guard_type, "code");
                assert!(message.to_lowercase().contains("authenticator"));
            }
            other => panic!("Expected SteamGuardPrompt with authenticator, got {:?}", other),
        }
    }

    #[test]
    fn test_inspect_depot_auth_success() {
        // QR login success
        let qr_out = "Connecting to Steam3... Done!
Logging in with QR code... Done!
Got account info".to_lowercase();
        assert_eq!(
            inspect_depot_auth(&qr_out, "qr", false, false, false),
            AuthInspectionResult::AuthSuccess
        );

        // Password login success (no 2FA)
        let pass_out = "Connecting to Steam3... Done!
Logging 'destinyuser' into Steam3... Done!
Got login token".to_lowercase();
        assert_eq!(
            inspect_depot_auth(&pass_out, "credentials", false, false, false),
            AuthInspectionResult::AuthSuccess
        );

        // Password + Steam Guard success
        let guard_out = "Connecting to Steam3... Done!
Logging 'destinyuser' into Steam3... Done!
Got account info
Requesting app info...".to_lowercase();
        assert_eq!(
            inspect_depot_auth(&guard_out, "credentials", false, false, false),
            AuthInspectionResult::AuthSuccess
        );

        // Ensure "Connecting to Steam3... Done!" followed by ongoing "Logging 'destinyuser' into Steam3..."
        // does NOT prematurely trigger AuthSuccess before credentials verification finishes
        let ongoing_login = "Connecting to Steam3... Done!
Logging 'destinyuser' into Steam3...".to_lowercase();
        assert_eq!(
            inspect_depot_auth(&ongoing_login, "credentials", false, false, false),
            AuthInspectionResult::None
        );
    }

    #[test]
    fn test_inspect_depot_auth_errors() {
        let err_pass = "Failed to authenticate with Steam: InvalidPassword".to_lowercase();
        match inspect_depot_auth(&err_pass, "credentials", false, false, false) {
            AuthInspectionResult::AuthError { error_type, .. } => {
                assert_eq!(error_type, "invalid_password");
            }
            other => panic!("Expected AuthError invalid_password, got {:?}", other),
        }

        let err_2fa = "Failed to authenticate with Steam: TwoFactorCodeMismatch".to_lowercase();
        match inspect_depot_auth(&err_2fa, "credentials", false, false, false) {
            AuthInspectionResult::AuthError { error_type, .. } => {
                assert_eq!(error_type, "2fa_mismatch");
            }
            other => panic!("Expected AuthError 2fa_mismatch, got {:?}", other),
        }
    }

    #[test]
    fn test_get_saved_steam_username() {
        let user = get_saved_steam_username();
        // On environments with IsolatedStorage (like this dev machine), it finds the detected account
        if let Some(ref u) = user {
            assert!(!u.is_empty(), "Username should not be empty");
        }
    }
}
