use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tauri::{AppHandle, Emitter};

use crate::constants::{
    EXPECTED_BUILD_ID, EXPECTED_BUILD_STRING, EXPECTED_EXE_SIZE, EXPECTED_FILE_VERSION,
    GAME_EXECUTABLE, REQUIRED_FREE_BYTES,
};
use crate::types::{CommandResult, FolderValidationResult, PreflightResult, ProgressPayload};

pub fn get_bundled_payload_dir() -> PathBuf {
    let current_exe = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("."));
    let app_dir = current_exe.parent().unwrap_or(Path::new("."));

    // Check various relative locations for bundled payload across packaged and dev structures
    let mut candidates = vec![
        // Production adjacent and resources
        app_dir.join("bundle").join("dawn-release"),
        app_dir.join("_up_").join("bundle").join("dawn-release"),
        app_dir.join("resources").join("bundle").join("dawn-release"),
        app_dir.join("resources").join("dawn-release"),
        // 1 level up (target/release)
        app_dir.join("..").join("bundle").join("dawn-release"),
        app_dir.join("..").join("_up_").join("bundle").join("dawn-release"),
        // 2 levels up
        app_dir.join("..").join("..").join("bundle").join("dawn-release"),
        // 3 levels up (dev mode: src-tauri/target/debug -> repo root)
        app_dir.join("..").join("..").join("..").join("bundle").join("dawn-release"),
        // 4 levels up
        app_dir.join("..").join("..").join("..").join("..").join("bundle").join("dawn-release"),
    ];

    // Current working directory and parent paths (e.g. running from repo root or src-tauri)
    if let Ok(cwd) = std::env::current_dir() {
        candidates.push(cwd.join("bundle").join("dawn-release"));
        candidates.push(cwd.join("..").join("bundle").join("dawn-release"));
        candidates.push(cwd.join("..").join("..").join("bundle").join("dawn-release"));
        candidates.push(cwd.join("resources").join("bundle").join("dawn-release"));
    }

    candidates.push(PathBuf::from("bundle").join("dawn-release"));
    candidates.push(PathBuf::from("..").join("bundle").join("dawn-release"));

    for c in &candidates {
        if c.exists() && (c.join("payload").exists() || c.join("release.json").exists()) {
            return c.clone();
        }
    }

    // Fallback if payload/release.json check fails but folder exists
    for c in &candidates {
        if c.exists() {
            return c.clone();
        }
    }

    app_dir.join("bundle").join("dawn-release")
}

#[cfg(windows)]
pub fn get_available_disk_space(path: &Path) -> u64 {
    use std::os::windows::ffi::OsStrExt;
    let mut wide: Vec<u16> = path.as_os_str().encode_wide().collect();
    wide.push(0);

    let mut free_bytes_caller: u64 = 0;
    let mut total_bytes: u64 = 0;
    let mut total_free: u64 = 0;

    extern "system" {
        fn GetDiskFreeSpaceExW(
            lpDirectoryName: *const u16,
            lpFreeBytesAvailableToCaller: *mut u64,
            lpTotalNumberOfBytes: *mut u64,
            lpTotalNumberOfFreeBytes: *mut u64,
        ) -> i32;
    }

    unsafe {
        if GetDiskFreeSpaceExW(
            wide.as_ptr(),
            &mut free_bytes_caller,
            &mut total_bytes,
            &mut total_free,
        ) != 0 {
            free_bytes_caller
        } else {
            0
        }
    }
}

#[cfg(not(windows))]
pub fn get_available_disk_space(path: &Path) -> u64 {
    use std::ffi::CString;
    use std::mem::MaybeUninit;

    let c_path = match CString::new(path.to_str().unwrap_or("/")) {
        Ok(c) => c,
        Err(_) => return 0,
    };

    let mut stat: MaybeUninit<libc::statvfs> = MaybeUninit::uninit();
    let res = unsafe { libc::statvfs(c_path.as_ptr(), stat.as_mut_ptr()) };
    if res == 0 {
        let s = unsafe { stat.assume_init() };
        (s.f_bavail as u64) * (s.f_frsize as u64)
    } else {
        0
    }
}

pub fn is_installer_directory(p: &Path) -> bool {
    let Ok(current_exe) = std::env::current_exe() else {
        return false;
    };
    let Some(installer_dir) = current_exe.parent() else {
        return false;
    };

    let norm_target = p.canonicalize().unwrap_or_else(|_| p.to_path_buf());
    let norm_installer = installer_dir.canonicalize().unwrap_or_else(|_| installer_dir.to_path_buf());

    let target_str = norm_target
        .to_string_lossy()
        .trim_start_matches(r"\\?\")
        .trim_end_matches(['/', '\\'])
        .to_lowercase();
    let inst_str = norm_installer
        .to_string_lossy()
        .trim_start_matches(r"\\?\")
        .trim_end_matches(['/', '\\'])
        .to_lowercase();

    if !target_str.is_empty() && target_str == inst_str {
        return true;
    }

    if let Some(exe_name) = current_exe.file_name() {
        if p.join(exe_name).is_file() {
            return true;
        }
    }

    false
}

pub fn is_steamapps_directory(p: &Path) -> bool {
    let path_lower = p.to_string_lossy().to_lowercase();
    path_lower.contains("steamapps")
}

#[cfg(windows)]
pub fn get_file_version_string(path: &Path) -> Option<String> {
    use std::os::windows::ffi::OsStrExt;
    let mut wide: Vec<u16> = path.as_os_str().encode_wide().collect();
    wide.push(0);

    #[link(name = "version")]
    extern "system" {
        fn GetFileVersionInfoSizeW(lptstrFilename: *const u16, lpdwHandle: *mut u32) -> u32;
        fn GetFileVersionInfoW(
            lptstrFilename: *const u16,
            dwHandle: u32,
            dwLen: u32,
            lpData: *mut u8,
        ) -> i32;
    }

    unsafe {
        let mut handle = 0u32;
        let size = GetFileVersionInfoSizeW(wide.as_ptr(), &mut handle);
        if size == 0 {
            return None;
        }

        let mut buf = vec![0u8; size as usize];
        if GetFileVersionInfoW(wide.as_ptr(), 0, size, buf.as_mut_ptr()) == 0 {
            return None;
        }

        let wide_needle: Vec<u16> = EXPECTED_BUILD_STRING.encode_utf16().collect();
        let u16_slice = std::slice::from_raw_parts(buf.as_ptr() as *const u16, size as usize / 2);
        if u16_slice.windows(wide_needle.len()).any(|w| w == wide_needle) {
            return Some(EXPECTED_BUILD_STRING.to_string());
        }
    }
    None
}

#[cfg(not(windows))]
#[allow(dead_code)]
pub fn get_file_version_string(_path: &Path) -> Option<String> {
    None
}

/// Verifies whether a given path is specifically the Destiny 2 Build 86657 executable required by Dawn.
/// Rejects modern live/retail builds (which are ~140MB+ and have newer version headers).
pub fn is_build_86657(exe_path: &Path) -> bool {
    if !exe_path.is_file() {
        return false;
    }

    // 1. Primary check: exact file size match (122,984,224 bytes from build 86657 depot manifest)
    if let Ok(meta) = fs::metadata(exe_path) {
        if meta.len() == EXPECTED_EXE_SIZE {
            return true;
        }
    }

    // 2. Secondary check: PE file version info containing "86657"
    #[cfg(windows)]
    {
        if let Some(ver) = get_file_version_string(exe_path) {
            if ver.contains(EXPECTED_BUILD_STRING) || ver.contains(EXPECTED_FILE_VERSION) {
                return true;
            }
        }
    }

    // 3. Cross-platform fallback: scan initial bytes for UTF-16LE / ASCII "86657"
    if let Ok(file) = fs::File::open(exe_path) {
        use std::io::Read;
        let mut buffer = Vec::new();
        let mut take = file.take(64 * 1024 * 1024);
        if take.read_to_end(&mut buffer).is_ok() {
            let wide_needle: Vec<u8> = EXPECTED_BUILD_STRING
                .encode_utf16()
                .flat_map(|c| c.to_le_bytes())
                .collect();
            if buffer.windows(wide_needle.len()).any(|w| w == wide_needle.as_slice())
                || buffer.windows(EXPECTED_BUILD_STRING.len()).any(|w| w == EXPECTED_BUILD_STRING.as_bytes())
            {
                return true;
            }
        }
    }

    false
}

pub fn validate_preflight(target_path: String) -> PreflightResult {
    let trimmed = target_path.trim();
    if trimmed.is_empty() {
        return PreflightResult {
            valid: false,
            path: None,
            exists: false,
            writeable: false,
            has_game: false,
            has_dawn: false,
            dawn_version: None,
            free_bytes: 0,
            free_gb: 0.0,
            required_bytes: REQUIRED_FREE_BYTES,
            required_gb: 110,
            has_enough_space: false,
            warning: None,
            error: Some("Please select a destination folder.".to_string()),
        };
    }

    let p = Path::new(trimmed);
    // Disallow pure root drive (e.g. C:\)
    if let Some(parent) = p.parent() {
        if parent == Path::new("") && p.to_string_lossy().ends_with(':') {
            return PreflightResult {
                valid: false,
                path: Some(trimmed.to_string()),
                exists: false,
                writeable: false,
                has_game: false,
                has_dawn: false,
                dawn_version: None,
                free_bytes: 0,
                free_gb: 0.0,
                required_bytes: REQUIRED_FREE_BYTES,
                required_gb: 110,
                has_enough_space: false,
                warning: None,
                error: Some("Choose a subfolder on the drive (e.g. D:\\Games\\Destiny2), not the root drive itself.".to_string()),
            };
        }
    }

    // Disallow selecting the installer's own folder
    if is_installer_directory(p) {
        let err = "Cannot install into the folder where the installer is located. Please create and choose a separate folder for Destiny 2 + Dawn (e.g. C:\\Games\\Dawn).".to_string();
        crate::logger::log_msg("WARN", &format!("Preflight rejected folder '{}': is installer directory", trimmed), None);
        return PreflightResult {
            valid: false,
            path: Some(trimmed.to_string()),
            exists: true,
            writeable: false,
            has_game: false,
            has_dawn: false,
            dawn_version: None,
            free_bytes: 0,
            free_gb: 0.0,
            required_bytes: REQUIRED_FREE_BYTES,
            required_gb: 110,
            has_enough_space: false,
            warning: None,
            error: Some(err),
        };
    }

    // Disallow selecting a Steam steamapps retail folder
    if is_steamapps_directory(p) {
        let err = "Cannot install into a Steam 'steamapps' folder. Dawn cannot be installed over your retail Destiny 2. Please choose a separate empty folder outside of Steam (e.g. C:\\Games\\Dawn).".to_string();
        crate::logger::log_msg("WARN", &format!("Preflight rejected folder '{}': is steamapps directory", trimmed), None);
        return PreflightResult {
            valid: false,
            path: Some(trimmed.to_string()),
            exists: true,
            writeable: false,
            has_game: false,
            has_dawn: false,
            dawn_version: None,
            free_bytes: 0,
            free_gb: 0.0,
            required_bytes: REQUIRED_FREE_BYTES,
            required_gb: 110,
            has_enough_space: false,
            warning: None,
            error: Some(err),
        };
    }

    let exists = p.exists();
    if !exists {
        if let Err(e) = fs::create_dir_all(p) {
            return PreflightResult {
                valid: false,
                path: Some(trimmed.to_string()),
                exists: false,
                writeable: false,
                has_game: false,
                has_dawn: false,
                dawn_version: None,
                free_bytes: 0,
                free_gb: 0.0,
                required_bytes: REQUIRED_FREE_BYTES,
                required_gb: 110,
                has_enough_space: false,
                warning: None,
                error: Some(format!("Cannot access folder: {}", e)),
            };
        }
    }

    // Write permission probe
    let probe_file = p.join(format!(".dawn_write_test_{}.tmp", std::time::SystemTime::now().elapsed().unwrap_or_default().as_millis()));
    let writeable = match fs::write(&probe_file, b"dawn_probe") {
        Ok(_) => {
            let _ = fs::remove_file(&probe_file);
            true
        }
        Err(e) => {
            return PreflightResult {
                valid: false,
                path: Some(trimmed.to_string()),
                exists: true,
                writeable: false,
                has_game: false,
                has_dawn: false,
                dawn_version: None,
                free_bytes: 0,
                free_gb: 0.0,
                required_bytes: REQUIRED_FREE_BYTES,
                required_gb: 110,
                has_enough_space: false,
                warning: None,
                error: Some(format!("Folder is write-protected or lacks permissions: {}", e)),
            };
        }
    };

    let free_bytes = get_available_disk_space(p);
    let free_gb = (free_bytes as f64) / (1024.0 * 1024.0 * 1024.0);
    let required_gb = REQUIRED_FREE_BYTES / (1024 * 1024 * 1024);

    let exe_path = p.join(GAME_EXECUTABLE);
    let has_any_exe = exe_path.is_file();
    let is_correct_build = if has_any_exe { is_build_86657(&exe_path) } else { false };
    let has_game = is_correct_build;
    let has_enough_space = has_game || free_bytes >= REQUIRED_FREE_BYTES;

    let warning = if has_any_exe && !is_correct_build {
        Some(format!(
            "Notice: Found destiny2.exe, but it is a newer live retail build (not Build {EXPECTED_BUILD_ID}). Dawn requires Season of Arrivals Build {EXPECTED_BUILD_ID}. Fresh installation into a dedicated folder will be performed."
        ))
    } else if !has_enough_space {
        Some(format!(
            "Insufficient disk space: {:.1} GB free. Fresh install requires at least ~{} GB.",
            free_gb, required_gb
        ))
    } else {
        None
    };

    if has_game {
        // Silently migrate legacy root Dawn directory & proxy DLL to bin/x64 on startup/selection
        let _ = crate::dawn_release::migrate_legacy_root_dawn(&p, None);
    }

    let has_dawn = (p.join("bin").join("x64").join("Dawn").is_dir()
        && p.join("bin").join("x64").join("steam_api64.dll").is_file())
        || (p.join("Dawn").is_dir() && (p.join("steam_api64.dll").is_file() || p.join("bin").join("x64").join("steam_api64.dll").is_file()));

    let mut dawn_version = None;
    let rel_meta = p.join(".dawn").join("release.json");
    let rel_root = p.join("release.json");
    for meta_path in [rel_meta, rel_root] {
        if meta_path.is_file() {
            if let Ok(content) = fs::read_to_string(&meta_path) {
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(ver) = parsed
                        .get("tag_name")
                        .or_else(|| parsed.get("installed_tag"))
                        .or_else(|| parsed.get("release"))
                        .or_else(|| parsed.get("version"))
                        .and_then(|v| v.as_str())
                    {
                        let clean = ver.trim().trim_start_matches(['v', 'V']).to_string();
                        if !clean.is_empty() {
                            dawn_version = Some(clean);
                            break;
                        }
                    }
                }
            }
        }
    }

    crate::logger::log_msg(
        "PREFLIGHT",
        &format!(
            "Path '{}' evaluated: valid=true, has_game={}, has_dawn={} ({:?}), space={:.1}GB/{}GB",
            trimmed, has_game, has_dawn, dawn_version, free_gb, required_gb
        ),
        None,
    );

    PreflightResult {
        valid: true,
        path: Some(trimmed.to_string()),
        exists: true,
        writeable,
        has_game,
        has_dawn,
        dawn_version,
        free_bytes,
        free_gb: (free_gb * 100.0).round() / 100.0,
        required_bytes: REQUIRED_FREE_BYTES,
        required_gb,
        has_enough_space,
        warning,
        error: None,
    }
}

pub fn validate_game_folder(folder_path: String) -> FolderValidationResult {
    let p = Path::new(&folder_path);
    if !p.exists() {
        return FolderValidationResult {
            valid: false,
            has_packages: false,
            exe_path: None,
            message: "Directory does not exist".to_string(),
        };
    }

    if is_installer_directory(p) {
        return FolderValidationResult {
            valid: false,
            has_packages: false,
            exe_path: None,
            message: "Cannot choose the folder where the installer is located. Please choose a separate game folder.".to_string(),
        };
    }

    if is_steamapps_directory(p) {
        return FolderValidationResult {
            valid: false,
            has_packages: false,
            exe_path: None,
            message: "Cannot install into a Steam 'steamapps' folder. Dawn cannot be installed over retail Destiny 2.".to_string(),
        };
    }

    let exe = p.join(GAME_EXECUTABLE);
    if !exe.exists() {
        return FolderValidationResult {
            valid: false,
            has_packages: false,
            exe_path: None,
            message: "destiny2.exe not found in this folder".to_string(),
        };
    }

    // Per-version check: Reject live/retail Destiny 2 builds
    if !is_build_86657(&exe) {
        let size_mb = fs::metadata(&exe).map(|m| m.len() / (1024 * 1024)).unwrap_or(0);
        return FolderValidationResult {
            valid: false,
            has_packages: false,
            exe_path: Some(exe.to_string_lossy().to_string()),
            message: format!(
                "destiny2.exe ({size_mb} MB) is not Build {EXPECTED_BUILD_ID}. This appears to be the current live/retail build; Dawn requires the Season of Arrivals Build {EXPECTED_BUILD_ID} files."
            ),
        };
    }

    let packages = p.join("packages");
    let has_packages = packages.exists();

    FolderValidationResult {
        valid: true,
        has_packages,
        exe_path: Some(exe.to_string_lossy().to_string()),
        message: if has_packages {
            format!("Valid Destiny 2 Build {EXPECTED_BUILD_ID} installation found")
        } else {
            format!("Destiny 2 Build {EXPECTED_BUILD_ID} found (packages folder missing)")
        },
    }
}

#[cfg(windows)]
fn get_steam_libraries() -> Vec<PathBuf> {
    use std::os::windows::process::CommandExt;
    let mut libraries = Vec::new();
    let mut reg_cmd = Command::new("reg");
    reg_cmd.args(["query", r"HKCU\Software\Valve\Steam", "/v", "SteamPath"]).creation_flags(0x08000000);
    let Ok(out) = reg_cmd.output()
    else {
        return libraries;
    };
    let text = String::from_utf8_lossy(&out.stdout);
    let Some(steam) = text.lines().find_map(|l| l.split("REG_SZ").nth(1)).map(|p| PathBuf::from(p.trim())) else {
        return libraries;
    };
    libraries.push(steam.clone());
    let vdf = fs::read_to_string(steam.join("steamapps").join("libraryfolders.vdf")).unwrap_or_default();
    for line in vdf.lines() {
        if let Some(rest) = line.trim().strip_prefix("\"path\"") {
            let clean = rest.trim().trim_matches('"').replace("\\\\", "\\");
            libraries.push(PathBuf::from(clean));
        }
    }
    libraries
}

#[cfg(not(windows))]
fn get_steam_libraries() -> Vec<PathBuf> {
    let mut libraries = Vec::new();
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let home_path = PathBuf::from(home);

    let steam_roots = [
        home_path.join(".local").join("share").join("Steam"),
        home_path.join(".steam").join("steam"),
        home_path.join(".steam").join("root"),
        home_path.join(".var").join("app").join("com.valvesoftware.Steam").join(".local").join("share").join("Steam"),
        PathBuf::from("/run/media/mmcblk0p1"),
        PathBuf::from("/run/media/deck"),
    ];

    for root in &steam_roots {
        if root.is_dir() {
            if !libraries.contains(root) {
                libraries.push(root.clone());
            }
            let vdf_path = root.join("steamapps").join("libraryfolders.vdf");
            if let Ok(content) = fs::read_to_string(&vdf_path) {
                for line in content.lines() {
                    let trimmed = line.trim();
                    if let Some(rest) = trimmed.strip_prefix("\"path\"") {
                        let clean = rest.trim().trim_matches('"').replace("\\\\", "/");
                        let lib_p = PathBuf::from(clean);
                        if lib_p.is_dir() && !libraries.contains(&lib_p) {
                            libraries.push(lib_p);
                        }
                    }
                }
            }
        }
    }

    libraries
}

pub fn detect_game_folder() -> Option<String> {
    let mut candidates: Vec<PathBuf> = Vec::new();

    // 1. Check existing Dawn folders on fixed drives first
    let drives = ["C:", "D:", "E:", "F:", "G:", "H:"];
    for drive in drives {
        candidates.push(PathBuf::from(format!(r#"{}\Dawn"#, drive)));
        candidates.push(PathBuf::from(format!(r#"{}\Games\Dawn"#, drive)));
        candidates.push(PathBuf::from(format!(r#"{}\Destiny 2"#, drive)));
        candidates.push(PathBuf::from(format!(r#"{}\Games\Destiny 2"#, drive)));
    }

    // 2. Steam library folders
    for lib in get_steam_libraries() {
        candidates.push(lib.join("steamapps").join("common").join("Dawn"));
        candidates.push(lib.join("steamapps").join("common").join("Destiny 2"));
    }

    // Standard Steam default paths
    for drive in ["C:", "D:", "E:"] {
        candidates.push(PathBuf::from(format!(r#"{}\Program Files (x86)\Steam\steamapps\common\Destiny 2"#, drive)));
        candidates.push(PathBuf::from(format!(r#"{}\Program Files\Steam\steamapps\common\Destiny 2"#, drive)));
        candidates.push(PathBuf::from(format!(r#"{}\SteamLibrary\steamapps\common\Destiny 2"#, drive)));
    }

    // Scan candidates: STRICT PER-VERSION CHECK
    // Only accept if destiny2.exe exists AND is verified Build 86657!
    // Never auto-select the current live retail build of Destiny 2.
    for p in candidates {
        if p.exists() {
            let exe = p.join(GAME_EXECUTABLE);
            if is_build_86657(&exe) {
                return Some(p.to_string_lossy().to_string());
            }
        }
    }

    None
}

pub fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_all(&from, &to)?;
        } else {
            fs::copy(&from, &to)?;
        }
    }
    Ok(())
}

#[allow(dead_code)]
pub async fn install_bundled_dawn(app: &AppHandle, game_root: &str) {
    let bundled = get_bundled_payload_dir();
    let payload = bundled.join("payload");
    let target = Path::new(game_root);

    let _ = app.emit("depot:output", format!("Deploying mod files from {:?}...\r\n", payload));

    if payload.exists() {
        let _ = crate::dawn_release::migrate_legacy_root_dawn(target, Some(app));

        let bin_x64 = target.join("bin").join("x64");
        let _ = fs::create_dir_all(&bin_x64);

        let dawn_sub = payload.join("Dawn");
        let bin_dawn = bin_x64.join("Dawn");
        if dawn_sub.exists() {
            if bin_dawn.exists() {
                let _ = crate::dawn_release::deploy_preserving_user_settings(&dawn_sub, &bin_dawn);
            } else {
                let _ = copy_dir_all(&dawn_sub, &bin_dawn);
            }
        }

        let steam_dll = payload.join("steam_api64.dll");
        let bin_steam_dll = bin_x64.join("steam_api64.dll");
        if steam_dll.exists() {
            let _ = fs::copy(&steam_dll, &bin_steam_dll);
        }

        let root_dawn = target.join("Dawn");
        if root_dawn.exists() {
            let _ = fs::remove_dir_all(&root_dawn);
        }
        let root_steam_dll = target.join("steam_api64.dll");
        if root_steam_dll.exists() && !is_genuine_steam_dll(&root_steam_dll) {
            let _ = fs::remove_file(&root_steam_dll);
        }

        let _ = app.emit("depot:output", "Dawn payload deployment to bin/x64 finished!\r\n");
    }
}

pub async fn install_dawn(app: AppHandle, game_root: String) -> CommandResult {
    let _ = app.emit("installer:progress", ProgressPayload {
        percent: 5,
        status: "Checking for latest Dawn release...".to_string(),
    });

    let release_dir = match crate::dawn_release::ensure_latest_dawn_release(&app).await {
        Ok(dir) => dir,
        Err(e) => {
            let _ = app.emit("depot:output", format!("[WARN] {}. Checking cached or bundled fallback...\r\n", e));
            if let Some(cached) = crate::dawn_release::get_latest_cached_release_dir() {
                cached
            } else {
                get_bundled_payload_dir()
            }
        }
    };

    let _ = app.emit("installer:progress", ProgressPayload {
        percent: 50,
        status: "Deploying Dawn files...".to_string(),
    });

    match crate::dawn_release::deploy_dawn_to_game(&app, &release_dir, &game_root).await {
        Ok(()) => {
            let _ = app.emit("installer:progress", ProgressPayload {
                percent: 100,
                status: "Dawn installed successfully!".to_string(),
            });
            CommandResult {
                success: true,
                message: Some("Installation completed successfully!".to_string()),
                error: None,
                cancelled: Some(false),
                count: None,
            }
        }
        Err(e) => {
            let _ = app.emit("installer:progress", ProgressPayload {
                percent: 100,
                status: format!("Installation error: {}", e),
            });
            CommandResult {
                success: false,
                message: None,
                error: Some(e),
                cancelled: Some(false),
                count: None,
            }
        }
    }
}

/// Validates that a file is a genuine x64 Steam Client API DLL and NOT a proxy or Dawn DLL.
/// Inspects the PE headers (MZ signature, PE\0\0, AMD64 machine, IMAGE_FILE_DLL flag)
/// and confirms the version string contains 'Steam Client API'.
pub fn is_genuine_steam_dll(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }
    let data = match fs::read(path) {
        Ok(d) => d,
        Err(_) => return false,
    };
    if data.len() < 256 {
        return false;
    }
    // Check MZ signature
    if data[0] != b'M' || data[1] != b'Z' {
        return false;
    }
    // Offset 0x3C contains the offset to the PE header
    let pe_offset = u32::from_le_bytes([data[0x3C], data[0x3D], data[0x3E], data[0x3F]]) as usize;
    if pe_offset < 64 || pe_offset + 24 > data.len() {
        return false;
    }
    // Check PE signature: 'P', 'E', 0, 0
    if &data[pe_offset..pe_offset + 4] != b"PE\0\0" {
        return false;
    }
    // Machine architecture: 0x8664 = IMAGE_FILE_MACHINE_AMD64
    let machine = u16::from_le_bytes([data[pe_offset + 4], data[pe_offset + 5]]);
    if machine != 0x8664 {
        return false;
    }
    // Characteristics at offset + 22: bit 0x2000 = IMAGE_FILE_DLL
    let characteristics = u16::from_le_bytes([data[pe_offset + 22], data[pe_offset + 23]]);
    if (characteristics & 0x2000) == 0 {
        return false;
    }

    // Check that ProductName is "Steam Client API" (stored as UTF-16LE in PE resource section)
    let steam_needle_utf16: Vec<u8> = "Steam Client API"
        .encode_utf16()
        .flat_map(|c| c.to_le_bytes())
        .collect();

    data.windows(steam_needle_utf16.len())
        .any(|w| w == steam_needle_utf16.as_slice())
        || data.windows(16).any(|w| w == b"Steam Client API")
}

/// Discovers a genuine original Steam Client API DLL from backup candidates
/// (.dawn/original/, .dawn/backup/, and .dawn/release-backups/).
pub fn find_original_steam_dll(game_root: &Path, relative: &str) -> Option<PathBuf> {
    let mut candidates: Vec<PathBuf> = Vec::new();

    // 1. .dawn/original/<relative>
    candidates.push(game_root.join(".dawn").join("original").join(relative));
    if relative != "steam_api64.dll" {
        candidates.push(game_root.join(".dawn").join("original").join("steam_api64.dll"));
    }

    // 2. .dawn/backup/steam_api64.dll & .dawn/backup/<relative>
    candidates.push(game_root.join(".dawn").join("backup").join(relative));
    if relative != "steam_api64.dll" {
        candidates.push(game_root.join(".dawn").join("backup").join("steam_api64.dll"));
    }

    // 3. .dawn/release-backups/ and .dawn/backup/ subdirectories
    for loc in &[".dawn/release-backups", ".dawn/backup"] {
        let history = game_root.join(loc);
        if history.is_dir() {
            if let Ok(entries) = fs::read_dir(&history) {
                let mut dirs: Vec<PathBuf> = entries
                    .flatten()
                    .map(|e| e.path())
                    .filter(|p| p.is_dir())
                    .collect();
                dirs.sort();

                for dir in dirs {
                    if *loc == ".dawn/release-backups" {
                        let journal_path = dir.join("journal.json");
                        if journal_path.is_file() {
                            if let Ok(content) = fs::read_to_string(&journal_path) {
                                if let Ok(journal) = serde_json::from_str::<serde_json::Value>(&content) {
                                    if journal.get("state").and_then(|s| s.as_str()) == Some("complete") {
                                        candidates.push(dir.join("previous").join(relative));
                                    }
                                }
                            }
                        }
                    } else {
                        candidates.push(dir.join(relative));
                    }
                }
            }
        }
    }

    // 4. Test each candidate
    for candidate in candidates {
        if candidate.is_file() && is_genuine_steam_dll(&candidate) {
            return Some(candidate);
        }
    }

    None
}

/// Fully uninstalls Dawn mod natively without PowerShell dependencies.
/// Restores original steam_api64.dll, removes mod directories, proxy DLLs/PDBs, and purges .dawn.
pub async fn uninstall_dawn(app: AppHandle, game_root: String) -> CommandResult {
    let p = Path::new(&game_root);

    let _ = app.emit("depot:output", "\r\n[DAWN] Starting Dawn uninstallation...\r\n");

    // 1. Verify destiny2 is closed
    let mut sys = sysinfo::System::new();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
    let is_running = sys.processes().values().any(|proc| {
        let name = proc.name().to_string_lossy().to_lowercase();
        name == "destiny2" || name == "destiny2.exe"
    });
    if is_running {
        let _ = app.emit("depot:output", "[DAWN] Error: Destiny 2 is running. Please close the game before uninstalling.\r\n");
        return CommandResult {
            success: false,
            message: None,
            error: Some("Close Destiny 2 before uninstalling Dawn.".to_string()),
            cancelled: Some(false),
            count: None,
        };
    }

    // 2. Locate genuine original steam_api64.dll
    let original_dll = find_original_steam_dll(p, "steam_api64.dll");
    let mut restored = false;

    if let Some(ref orig) = original_dll {
        let _ = app.emit("depot:output", format!("[DAWN] Genuine Steam DLL found at: {}\r\n", orig.display()));

        let targets = [
            p.join("bin").join("x64").join("steam_api64.dll"),
            p.join("steam_api64.dll"),
        ];

        for target in &targets {
            if let Some(parent) = target.parent() {
                let _ = fs::create_dir_all(parent);
            }
            match fs::copy(orig, target) {
                Ok(_) => {
                    restored = true;
                    let _ = app.emit("depot:output", format!("[DAWN] Restored original DLL to: {}\r\n", target.display()));
                }
                Err(e) => {
                    let _ = app.emit("depot:output", format!("[DAWN] Warning: Failed copying to {}: {}\r\n", target.display(), e));
                }
            }
        }
    } else {
        let _ = app.emit("depot:output", "[DAWN] Warning: No genuine Steam DLL backup found in .dawn/\r\n");
    }

    // 3. Remove proxy DLLs, PDBs, and Dawn release files
    let remove_files = [
        p.join("steam_api64.pdb"),
        p.join("bin").join("x64").join("steam_api64.pdb"),
        p.join("dxgi.dll"),
        p.join("dxgi.pdb"),
        p.join("bin").join("x64").join("dxgi.dll"),
        p.join("bin").join("x64").join("dxgi.pdb"),
        p.join("release.json"),
        p.join("launch-destiny.cmd"),
        p.join("launch-destiny.sh"),
    ];

    for f in &remove_files {
        if f.exists() {
            let _ = fs::remove_file(f);
            let _ = app.emit("depot:output", format!("[DAWN] Removed {}\r\n", f.display()));
        }
    }

    // If no genuine DLL was restored and the existing steam_api64.dll is a Dawn proxy, remove it to avoid broken state
    if !restored {
        let targets = [
            p.join("bin").join("x64").join("steam_api64.dll"),
            p.join("steam_api64.dll"),
        ];
        for target in &targets {
            if target.is_file() && !is_genuine_steam_dll(target) {
                let _ = fs::remove_file(target);
                let _ = app.emit("depot:output", format!("[DAWN] Removed Dawn proxy DLL: {}\r\n", target.display()));
            }
        }
    }

    // 4. Remove Dawn directories
    let remove_dirs = [
        p.join("Dawn"),
        p.join("bin").join("x64").join("Dawn"),
        p.join(".dawn"),
    ];

    for dir in &remove_dirs {
        if dir.exists() {
            let _ = fs::remove_dir_all(dir);
            let _ = app.emit("depot:output", format!("[DAWN] Purged {}\r\n", dir.display()));
        }
    }

    // 5. Clear cache
    let _ = clear_cache(game_root);

    let message = if restored {
        "Dawn uninstalled. Original steam_api64.dll restored successfully and Dawn files purged.".to_string()
    } else {
        "Dawn uninstalled. Warning: No genuine Steam DLL backup found. Please verify game files in Steam before launching.".to_string()
    };

    let _ = app.emit("depot:output", format!("[DAWN] {}\r\n", message));

    CommandResult {
        success: true,
        message: Some(message),
        error: None,
        cancelled: Some(false),
        count: None,
    }
}

pub async fn restore_dawn(app: AppHandle, game_root: String) -> CommandResult {
    uninstall_dawn(app, game_root).await
}

pub fn clear_cache(game_root: String) -> CommandResult {
    let targets = [
        PathBuf::from(&game_root).join("Dawn").join("cache"),
        PathBuf::from(&game_root).join("bin").join("x64").join("Dawn").join("cache"),
    ];

    let mut cleared = 0;
    for t in targets {
        if t.exists() {
            if let Ok(entries) = fs::read_dir(t) {
                for entry in entries.flatten() {
                    let p = entry.path();
                    if p.extension().map(|ext| ext == "bin").unwrap_or(false) {
                        if fs::remove_file(p).is_ok() {
                            cleared += 1;
                        }
                    }
                }
            }
        }
    }

    // Purge steam_appid.txt if present
    let appid_root = PathBuf::from(&game_root).join("steam_appid.txt");
    if appid_root.is_file() {
        let _ = fs::remove_file(&appid_root);
        cleared += 1;
    }
    let appid_bin = PathBuf::from(&game_root).join("bin").join("x64").join("steam_appid.txt");
    if appid_bin.is_file() {
        let _ = fs::remove_file(&appid_bin);
        cleared += 1;
    }

    CommandResult {
        success: true,
        message: Some(format!("Cleared {} cache files", cleared)),
        error: None,
        cancelled: Some(false),
        count: Some(cleared),
    }
}

pub fn update_dawn_language(install_root: &str, language_code: &str) {
    let candidate_dirs = [
        PathBuf::from(install_root).join("bin").join("x64").join("Dawn"),
        PathBuf::from(install_root).join("Dawn"),
        PathBuf::from(install_root).join("Restoration"),
    ];

    for dir in &candidate_dirs {
        let settings_path = dir.join("settings.json");
        if settings_path.exists() {
            if let Ok(content) = fs::read_to_string(&settings_path) {
                if let Ok(mut json) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(steam) = json.get_mut("steam") {
                        if let Some(obj) = steam.as_object_mut() {
                            obj.insert("language".to_string(), serde_json::Value::String(language_code.to_string()));
                        }
                    } else if let Some(root_obj) = json.as_object_mut() {
                        let mut steam_map = serde_json::Map::new();
                        steam_map.insert("language".to_string(), serde_json::Value::String(language_code.to_string()));
                        root_obj.insert("steam".to_string(), serde_json::Value::Object(steam_map));
                    }
                    if let Ok(new_content) = serde_json::to_string_pretty(&json) {
                        let _ = fs::write(&settings_path, new_content);
                    }
                }
            }
        } else if dir.exists() {
            let mut root_obj = serde_json::Map::new();
            let mut steam_map = serde_json::Map::new();
            steam_map.insert("language".to_string(), serde_json::Value::String(language_code.to_string()));
            root_obj.insert("steam".to_string(), serde_json::Value::Object(steam_map));
            if let Ok(new_content) = serde_json::to_string_pretty(&serde_json::Value::Object(root_obj)) {
                let _ = fs::write(&settings_path, new_content);
            }
        }
    }
}

pub fn check_vc_redist_installed() -> bool {
    #[cfg(windows)]
    {
        let sys_root = std::env::var("SystemRoot").unwrap_or_else(|_| "C:\\Windows".to_string());
        let sys32 = Path::new(&sys_root).join("System32");
        let vcruntime = sys32.join("vcruntime140.dll");
        let vcruntime_1 = sys32.join("vcruntime140_1.dll");
        let msvcp = sys32.join("msvcp140.dll");
        vcruntime.is_file() && vcruntime_1.is_file() && msvcp.is_file()
    }
    #[cfg(not(windows))]
    {
        true
    }
}

pub fn ensure_vc_runtime_files(game_root: &Path, app: Option<&AppHandle>) {
    #[cfg(windows)]
    {
        let sys_root = std::env::var("SystemRoot").unwrap_or_else(|_| "C:\\Windows".to_string());
        let sys32 = Path::new(&sys_root).join("System32");
        let bin_x64 = game_root.join("bin").join("x64");
        let _ = fs::create_dir_all(&bin_x64);

        let runtime_dlls = ["vcruntime140.dll", "vcruntime140_1.dll", "msvcp140.dll"];

        for dll_name in &runtime_dlls {
            let sys_file = sys32.join(dll_name);
            let root_dest = game_root.join(dll_name);
            let bin_dest = bin_x64.join(dll_name);

            if sys_file.is_file() {
                let should_copy_root = if !root_dest.is_file() {
                    true
                } else if let (Ok(src_meta), Ok(dst_meta)) = (sys_file.metadata(), root_dest.metadata()) {
                    dst_meta.len() == 0 || (src_meta.len() != dst_meta.len() && src_meta.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH) > dst_meta.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH))
                } else {
                    false
                };

                if should_copy_root {
                    if let Ok(_) = fs::copy(&sys_file, &root_dest) {
                        crate::logger::log_msg("INFO", &format!("Deployed validated 64-bit {} from System32 to game root", dll_name), app);
                    }
                }

                let should_copy_bin = if !bin_dest.is_file() {
                    true
                } else if let Ok(dst_meta) = bin_dest.metadata() {
                    dst_meta.len() == 0
                } else {
                    false
                };

                if should_copy_bin {
                    if let Ok(_) = fs::copy(&sys_file, &bin_dest) {
                        crate::logger::log_msg("INFO", &format!("Deployed validated 64-bit {} from System32 to bin/x64", dll_name), app);
                    }
                }
            } else {
                crate::logger::log_msg("WARN", &format!("System32 is missing {}! Visual C++ 2015-2022 (x64) Redistributable may need reinstallation.", dll_name), app);
            }
        }
    }
    #[cfg(not(windows))]
    {
        let _ = (game_root, app);
    }
}

pub fn unblock_game_files(game_root: &Path) {
    #[cfg(windows)]
    {
        let files_to_unblock = [
            game_root.join(GAME_EXECUTABLE),
            game_root.join("steam_api64.dll"),
            game_root.join("bin").join("x64").join("steam_api64.dll"),
            game_root.join("vcruntime140.dll"),
            game_root.join("vcruntime140_1.dll"),
            game_root.join("msvcp140.dll"),
            game_root.join("bin").join("x64").join("vcruntime140.dll"),
            game_root.join("bin").join("x64").join("vcruntime140_1.dll"),
            game_root.join("bin").join("x64").join("msvcp140.dll"),
            game_root.join("launch-destiny.cmd"),
        ];

        for f in &files_to_unblock {
            if f.exists() {
                let stream = format!("{}:Zone.Identifier", f.to_string_lossy());
                let _ = fs::remove_file(stream);
            }
        }

        let bin_dawn = game_root.join("bin").join("x64").join("Dawn");
        if bin_dawn.is_dir() {
            if let Ok(entries) = fs::read_dir(&bin_dawn) {
                for entry in entries.flatten() {
                    let p = entry.path();
                    if p.is_file() {
                        let stream = format!("{}:Zone.Identifier", p.to_string_lossy());
                        let _ = fs::remove_file(stream);
                    }
                }
            }
        }
    }
    #[cfg(not(windows))]
    {
        let _ = game_root;
    }
}

pub fn ensure_cvars_windowed_fullscreen(app: Option<&AppHandle>) {
    #[cfg(windows)]
    {
        let appdata = match std::env::var("APPDATA") {
            Ok(v) if !v.trim().is_empty() => v,
            _ => return,
        };

        let prefs_dir = PathBuf::from(appdata).join("Bungie").join("DestinyPC").join("prefs");
        if let Err(_) = fs::create_dir_all(&prefs_dir) {
            return;
        }
        let cvars_path = prefs_dir.join("cvars.xml");

        if !cvars_path.exists() {
            let default_xml = "<?xml version=\"1.0\"?>\r\n<body>\r\n\t<namespace name=\"graphics\">\r\n\t\t<cvar name=\"window_mode\" value=\"2\" />\r\n\t</namespace>\r\n</body>\r\n";
            let _ = fs::write(&cvars_path, default_xml);
            if let Some(a) = app {
                let _ = a.emit("depot:output", "[DAWN] Initialized display preferences (Windowed Fullscreen) in cvars.xml\r\n");
            }
        } else if let Ok(mut content) = fs::read_to_string(&cvars_path) {
            let mut modified = false;
            if content.contains("window_mode") {
                if content.contains(r#"<cvar name="window_mode" value="0""#) {
                    content = content.replace(r#"<cvar name="window_mode" value="0""#, r#"<cvar name="window_mode" value="2""#);
                    modified = true;
                } else if content.contains(r#"<cvar name="window_mode" value="1""#) {
                    content = content.replace(r#"<cvar name="window_mode" value="1""#, r#"<cvar name="window_mode" value="2""#);
                    modified = true;
                }
            } else if let Some(pos) = content.find("<namespace name=\"graphics\">") {
                let insert_pos = pos + "<namespace name=\"graphics\">".len();
                content.insert_str(insert_pos, "\r\n\t\t<cvar name=\"window_mode\" value=\"2\" />");
                modified = true;
            }

            if modified {
                let _ = fs::write(&cvars_path, content);
                if let Some(a) = app {
                    let _ = a.emit("depot:output", "[DAWN] Configured cvars.xml with window_mode=2 (Windowed Fullscreen)\r\n");
                }
            }
        }
    }
    #[cfg(not(windows))]
    {
        let _ = app;
    }
}

pub fn ensure_launch_scripts(game_root: &Path) {
    #[cfg(windows)]
    {
        let cmd_path = game_root.join("launch-destiny.cmd");
        if !cmd_path.exists() {
            let script = "@echo off\r\ncd /d \"%~dp0\"\r\nset DAWN_FOREST_BASELINE=1\r\nif not exist \"%~dp0destiny2.exe\" (\r\n    echo [ERROR] destiny2.exe was not found in %~dp0\r\n    pause\r\n    exit /b 1\r\n)\r\nstart \"\" \"%~dp0destiny2.exe\" %*\r\n";
            let _ = fs::write(&cmd_path, script);
        }
    }

    #[cfg(not(windows))]
    {
        let sh_path = game_root.join("launch-destiny.sh");
        if !sh_path.exists() {
            let script = r#"#!/usr/bin/env sh
# Dawn Launcher - Destiny 2 (Build 86657) Launch Script
set -e
GAME_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$GAME_DIR"

export DAWN_FOREST_BASELINE=1

# Check for steam-run (recommended on NixOS / SteamOS)
RUNNER=""
if command -v steam-run >/dev/null 2>&1; then
    RUNNER="steam-run"
elif [ -x "/run/current-system/sw/bin/steam-run" ]; then
    RUNNER="/run/current-system/sw/bin/steam-run"
fi

# Locate Proton or Wine
PROTON_CANDIDATE=""
for cand in \
    "$HOME/.local/share/Steam/steamapps/common/Proton - Experimental/proton" \
    "$HOME/.local/share/Steam/steamapps/common/Proton 9.0/proton" \
    "$HOME/.local/share/Steam/steamapps/common/Proton 8.0/proton" \
    "$HOME/.steam/steam/steamapps/common/Proton - Experimental/proton" \
    "$HOME/.steam/steam/steamapps/common/Proton 9.0/proton" \
    "$HOME/.steam/root/steamapps/common/Proton - Experimental/proton"; do
    if [ -f "$cand" ]; then
        PROTON_CANDIDATE="$cand"
        break
    fi
done

if [ -n "$PROTON_CANDIDATE" ]; then
    STEAM_ROOT="$(dirname "$(dirname "$(dirname "$(dirname "$PROTON_CANDIDATE")")")")"
    export STEAM_COMPAT_CLIENT_INSTALL_PATH="$STEAM_ROOT"
    export STEAM_COMPAT_DATA_PATH="$STEAM_ROOT/steamapps/compatdata/1085660"
    mkdir -p "$STEAM_COMPAT_DATA_PATH"

    echo "[DAWN] Launching via Proton: $PROTON_CANDIDATE"
    if [ -n "$RUNNER" ]; then
        exec $RUNNER "$PROTON_CANDIDATE" run "$GAME_DIR/destiny2.exe" "$@"
    else
        exec "$PROTON_CANDIDATE" run "$GAME_DIR/destiny2.exe" "$@"
    fi
elif command -v wine >/dev/null 2>&1; then
    echo "[DAWN] Launching via Wine"
    if [ -n "$RUNNER" ]; then
        exec $RUNNER wine "$GAME_DIR/destiny2.exe" "$@"
    else
        exec wine "$GAME_DIR/destiny2.exe" "$@"
    fi
else
    echo "[ERROR] Neither Proton nor Wine was found in standard locations or PATH."
    echo "Please install Proton via Steam or install Wine, or edit this script to specify your runner."
    exit 1
fi
"#;
            let _ = fs::write(&sh_path, script);
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Ok(meta) = fs::metadata(&sh_path) {
                    let mut perms = meta.permissions();
                    perms.set_mode(0o755);
                    let _ = fs::set_permissions(&sh_path, perms);
                }
            }
        }
    }
}

pub fn launch_game(app: AppHandle, game_root: String, language_code: Option<String>) -> CommandResult {
    let trimmed = game_root.trim().trim_matches('"');
    let mut p = PathBuf::from(trimmed);
    // If user pointed inside bin/x64, adjust to parent game root
    if p.file_name().and_then(|f| f.to_str()) == Some("x64") {
        if let Some(parent) = p.parent() {
            if parent.file_name().and_then(|f| f.to_str()) == Some("bin") {
                if let Some(root) = parent.parent() {
                    p = root.to_path_buf();
                }
            }
        }
    }

    if let Some(ref lang) = language_code {
        update_dawn_language(&p.to_string_lossy(), lang);
    }

    let exe_path = p.join(GAME_EXECUTABLE);

    if !exe_path.exists() {
        let msg = "destiny2.exe not found in game folder".to_string();
        let _ = app.emit("depot:output", format!("[LAUNCH ERROR] {}\r\n", msg));
        crate::logger::log_msg("ERROR", &msg, Some(&app));
        return CommandResult {
            success: false,
            message: None,
            error: Some(msg),
            cancelled: Some(false),
            count: None,
        };
    }

    if !is_build_86657(&exe_path) {
        let msg = format!(
            "Refusing to launch: destiny2.exe is not Build {EXPECTED_BUILD_ID}. Dawn cannot run on modern retail builds."
        );
        let _ = app.emit("depot:output", format!("[LAUNCH ERROR] {}\r\n", msg));
        crate::logger::log_msg("ERROR", &msg, Some(&app));
        return CommandResult {
            success: false,
            message: None,
            error: Some(msg),
            cancelled: Some(false),
            count: None,
        };
    }

    // Silently migrate legacy root Dawn directory & proxy DLL to bin/x64 if still present
    let _ = crate::dawn_release::migrate_legacy_root_dawn(&p, Some(&app));

    // Ensure Dawn mod proxy steam_api64.dll exists in bin/x64
    let bin_dll = p.join("bin").join("x64").join("steam_api64.dll");
    if !bin_dll.exists() {
        let msg = "Dawn mod proxy (steam_api64.dll) was not found in bin/x64. Windows Defender or your antivirus may have quarantined it, or Dawn was not installed. Please reinstall Dawn Mod or Verify Files, and check your antivirus protection history.".to_string();
        let _ = app.emit("depot:output", format!("[LAUNCH ERROR] {}\r\n", msg));
        crate::logger::log_msg("ERROR", &msg, Some(&app));
        return CommandResult {
            success: false,
            message: None,
            error: Some(msg),
            cancelled: Some(false),
            count: None,
        };
    }

    // Ensure root steam_api64.dll proxy is purged so destiny2.exe does not conflict with bin/x64
    let root_dll = p.join("steam_api64.dll");
    if root_dll.exists() && !is_genuine_steam_dll(&root_dll) {
        let _ = fs::remove_file(&root_dll);
        crate::logger::log_msg("INFO", "Cleaned up legacy steam_api64.dll proxy from game root", Some(&app));
    }

    // Ensure steam_appid.txt is purged: Destiny 2 anti-tamper specifically detects steam_appid.txt
    // and throws the error "Problem reading game content, please close destiny 2..."
    let appid_root = p.join("steam_appid.txt");
    if appid_root.exists() {
        let _ = fs::remove_file(&appid_root);
        crate::logger::log_msg("INFO", "Purged steam_appid.txt from game root (prevents 'Problem reading game content')", Some(&app));
    }
    let appid_bin = p.join("bin").join("x64").join("steam_appid.txt");
    if appid_bin.exists() {
        let _ = fs::remove_file(&appid_bin);
        crate::logger::log_msg("INFO", "Purged steam_appid.txt from bin/x64", Some(&app));
    }

    #[cfg(windows)]
    {
        if !check_vc_redist_installed() {
            let msg = "Microsoft Visual C++ 2015-2022 (x64) Redistributable is missing or incomplete (vcruntime140.dll, vcruntime140_1.dll, or msvcp140.dll missing from System32). Note: The 64-bit (x64) version is required even if you have the 32-bit (x86) version installed. Please install it from Microsoft: https://aka.ms/vs/17/release/vc_redist.x64.exe".to_string();
            let _ = app.emit("depot:output", format!("[LAUNCH ERROR] {}\r\n", msg));
            crate::logger::log_msg("ERROR", &msg, Some(&app));
            return CommandResult {
                success: false,
                message: None,
                error: Some(msg),
                cancelled: Some(false),
                count: None,
            };
        }

        ensure_vc_runtime_files(&p, Some(&app));
        unblock_game_files(&p);
        ensure_cvars_windowed_fullscreen(Some(&app));
    }

    ensure_launch_scripts(&p);

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        let _ = app.emit("depot:output", format!("[LAUNCH] Launching Destiny 2 directly from {:?}...\r\n", exe_path));
        crate::logger::log_msg(
            "INFO",
            &format!("Launching Destiny 2 from dir {:?}, exe: {:?}, DAWN_FOREST_BASELINE=1", p, exe_path),
            Some(&app),
        );

        let mut cmd = Command::new(&exe_path);
        cmd.current_dir(&p)
            .env("DAWN_FOREST_BASELINE", "1")
            .creation_flags(0x00000200); // CREATE_NEW_PROCESS_GROUP

        match cmd.spawn() {
            Ok(mut child) => {
                let pid = child.id();
                crate::logger::log_msg("INFO", &format!("Destiny 2 process spawned with PID {}", pid), Some(&app));

                // Check for immediate startup crash (missing runtime, DLL not found, bad format)
                std::thread::sleep(std::time::Duration::from_millis(750));
                match child.try_wait() {
                    Ok(Some(status)) => {
                        let raw_code = status.code().unwrap_or(-1);
                        let ucode = raw_code as u32;
                        let err_desc = match ucode {
                            0xC0000135 => "STATUS_DLL_NOT_FOUND (A required DLL was not found. Please install Visual C++ 2015-2022 x64 Redistributable and DirectX runtimes: https://aka.ms/vs/17/release/vc_redist.x64.exe)",
                            0xC000007B => "STATUS_INVALID_IMAGE_FORMAT (A 32/64-bit DLL conflict or corrupted file was detected)",
                            0xC0000005 => "STATUS_ACCESS_VIOLATION (Game crashed during early initialization. Check graphics drivers and ensure cvars.xml has window_mode=2)",
                            0xC0000409 => "STATUS_STACK_BUFFER_OVERRUN",
                            _ => "Process terminated immediately after launch",
                        };
                        let msg = format!("destiny2.exe exited immediately with code 0x{:08X} ({})", ucode, err_desc);
                        let _ = app.emit("depot:output", format!("[LAUNCH ERROR] {}\r\n", msg));
                        crate::logger::log_msg("ERROR", &msg, Some(&app));
                        CommandResult {
                            success: false,
                            message: None,
                            error: Some(msg),
                            cancelled: Some(false),
                            count: None,
                        }
                    }
                    Ok(None) => {
                        let msg = format!("Game launched successfully (PID: {})", pid);
                        let _ = app.emit("depot:output", format!("[LAUNCH] {}\r\n", msg));
                        crate::logger::log_msg("INFO", &msg, Some(&app));
                        CommandResult {
                            success: true,
                            message: Some(msg),
                            error: None,
                            cancelled: Some(false),
                            count: None,
                        }
                    }
                    Err(e) => {
                        let msg = format!("Game spawned (PID: {}), check status: {}", pid, e);
                        let _ = app.emit("depot:output", format!("[LAUNCH] {}\r\n", msg));
                        crate::logger::log_msg("INFO", &msg, Some(&app));
                        CommandResult {
                            success: true,
                            message: Some(msg),
                            error: None,
                            cancelled: Some(false),
                            count: None,
                        }
                    }
                }
            }
            Err(e) => {
                let os_code = e.raw_os_error().unwrap_or(0);
                let friendly = match os_code {
                    193 => "ERROR_BAD_EXE_FORMAT: destiny2.exe is not a valid 64-bit executable. The file may be corrupted.",
                    5 => "ERROR_ACCESS_DENIED: Access was denied launching destiny2.exe. Check folder permissions or antivirus.",
                    225 => "ERROR_VIRUS_INFECTED: Windows Defender or Antivirus blocked the game. Please add an exclusion in Windows Security.",
                    1260 => "ERROR_KM_DRIVER_BLOCKED: Windows Smart App Control or Group Policy blocked execution.",
                    _ => "Failed to spawn destiny2.exe",
                };
                let msg = format!("Failed to launch game: {} ({})", e, friendly);
                let _ = app.emit("depot:output", format!("[LAUNCH ERROR] {}\r\n", msg));
                crate::logger::log_msg("ERROR", &msg, Some(&app));
                CommandResult {
                    success: false,
                    message: None,
                    error: Some(msg),
                    cancelled: Some(false),
                    count: None,
                }
            }
        }
    }

    #[cfg(not(windows))]
    {
        let msg = "Linux launch is not supported. Please run the game on Windows or through a configured Wine/Proton runner.".to_string();
        let _ = app.emit("depot:output", format!("[LAUNCH] {}\r\n", msg));
        crate::logger::log_msg("WARN", &msg, Some(&app));
        CommandResult {
            success: false,
            message: None,
            error: Some(msg),
            cancelled: Some(false),
            count: None,
        }
    }
}

pub fn open_folder(target_path: String) -> bool {
    let p = Path::new(&target_path);
    if p.exists() {
        #[cfg(windows)]
        {
            let _ = Command::new("explorer").arg(p).spawn();
        }
        #[cfg(not(windows))]
        {
            let _ = Command::new("xdg-open").arg(p).spawn();
        }
        true
    } else {
        false
    }
}

pub fn open_docs() -> bool {
    let bundled = get_bundled_payload_dir();
    let readme = bundled.join("READ-ME.txt");
    let target_url = "https://github.com/isinternets/Dawn";

    #[cfg(windows)]
    {
        if readme.exists() {
            let _ = Command::new("explorer").arg(readme).spawn();
        } else {
            let _ = Command::new("explorer").arg(target_url).spawn();
        }
    }
    #[cfg(not(windows))]
    {
        if readme.exists() {
            let _ = Command::new("xdg-open").arg(readme).spawn();
        } else {
            let _ = Command::new("xdg-open").arg(target_url).spawn();
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ensure_launch_scripts() {
        let temp_dir = std::env::temp_dir().join(format!("dawn_test_scripts_{}", std::process::id()));
        let _ = fs::create_dir_all(&temp_dir);

        ensure_launch_scripts(&temp_dir);

        #[cfg(windows)]
        {
            let cmd_path = temp_dir.join("launch-destiny.cmd");
            assert!(cmd_path.is_file(), "launch-destiny.cmd should be created on Windows");
            let content = fs::read_to_string(&cmd_path).unwrap();
            assert!(content.contains("destiny2.exe"));
            assert!(content.contains("DAWN_FOREST_BASELINE=1"));
        }

        #[cfg(not(windows))]
        {
            let sh_path = temp_dir.join("launch-destiny.sh");
            assert!(sh_path.is_file(), "launch-destiny.sh should be created on Linux");
            let content = fs::read_to_string(&sh_path).unwrap();
            assert!(content.contains("destiny2.exe"));
        }

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_unblock_game_files() {
        let temp_dir = std::env::temp_dir().join(format!("dawn_test_unblock_{}", std::process::id()));
        let _ = fs::create_dir_all(&temp_dir);
        let test_file = temp_dir.join("destiny2.exe");
        let _ = fs::write(&test_file, b"test payload");

        unblock_game_files(&temp_dir);
        assert!(test_file.exists());

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_check_vc_redist_installed() {
        // Function should run without panic on any platform
        let installed = check_vc_redist_installed();
        #[cfg(windows)]
        {
            // On the Windows development machine, VC++ redistributable is installed
            assert!(installed, "VC++ Redistributable should be detected on dev machine");
        }
        #[cfg(not(windows))]
        {
            assert!(installed);
        }
    }

    #[test]
    fn test_is_steamapps_directory() {
        assert!(is_steamapps_directory(Path::new(r"C:\Program Files (x86)\Steam\steamapps\common\Destiny 2")));
        assert!(is_steamapps_directory(Path::new(r"D:\SteamLibrary\SteamApps\common\Destiny 2")));
        assert!(is_steamapps_directory(Path::new("/home/user/.steam/steam/steamapps/common/Destiny 2")));
        assert!(!is_steamapps_directory(Path::new(r"C:\Games\Dawn")));
        assert!(!is_steamapps_directory(Path::new(r"D:\Destiny2-Dawn")));
        assert!(!is_steamapps_directory(Path::new("/home/user/Games/Dawn")));
    }

    #[test]
    fn test_is_installer_directory() {
        if let Ok(current_exe) = std::env::current_exe() {
            if let Some(parent) = current_exe.parent() {
                assert!(is_installer_directory(parent), "Current exe directory must be identified as installer directory");
            }
        }

        let temp_dir = std::env::temp_dir().join(format!("dawn_test_non_installer_{}", std::process::id()));
        let _ = fs::create_dir_all(&temp_dir);
        assert!(!is_installer_directory(&temp_dir), "Empty temp directory must NOT be identified as installer directory");

        if let Ok(current_exe) = std::env::current_exe() {
            if let Some(exe_name) = current_exe.file_name() {
                let dummy_exe = temp_dir.join(exe_name);
                let _ = fs::write(&dummy_exe, b"dummy");
                assert!(is_installer_directory(&temp_dir), "Directory containing installer executable must be identified as installer directory");
            }
        }

        let _ = fs::remove_dir_all(&temp_dir);
    }
}


