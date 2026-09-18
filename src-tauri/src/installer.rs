use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tauri::{AppHandle, Emitter};

#[cfg(windows)]
use crate::constants::{EXPECTED_BUILD_STRING, EXPECTED_FILE_VERSION};
use crate::constants::{EXPECTED_BUILD_ID, EXPECTED_EXE_SIZE, GAME_EXECUTABLE, REQUIRED_FREE_BYTES};
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
            "Low disk space: {:.1} GB available. Fresh install requires ~{} GB.",
            free_gb, required_gb
        ))
    } else {
        None
    };

    let has_dawn = (p.join("Dawn").is_dir() || p.join("bin").join("x64").join("Dawn").is_dir())
        && (p.join("steam_api64.dll").is_file() || p.join("bin").join("x64").join("steam_api64.dll").is_file());

    let mut dawn_version = None;
    let rel_meta = p.join(".dawn").join("release.json");
    let rel_root = p.join("release.json");
    for meta_path in [rel_meta, rel_root] {
        if meta_path.is_file() {
            if let Ok(content) = fs::read_to_string(&meta_path) {
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(ver) = parsed
                        .get("release")
                        .or_else(|| parsed.get("version"))
                        .or_else(|| parsed.get("tag_name"))
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
        if let Err(e) = copy_dir_all(&payload, target) {
            let _ = app.emit("depot:output", format!("[WARN] Copy error: {}\r\n", e));
        }

        let dawn_sub = payload.join("Dawn");
        let bin_dawn = target.join("bin").join("x64").join("Dawn");
        if dawn_sub.exists() {
            let _ = copy_dir_all(&dawn_sub, &bin_dawn);
        }
        let _ = app.emit("depot:output", "Dawn payload deployment finished!\r\n");
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
            let _ = app.emit("depot:output", format!("[WARN] {}. Using bundled fallback...\r\n", e));
            get_bundled_payload_dir()
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
        PathBuf::from(install_root).join("Dawn"),
        PathBuf::from(install_root).join("bin").join("x64").join("Dawn"),
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

#[cfg(not(windows))]
fn find_proton_and_compatdata(_game_root: &Path) -> Result<(PathBuf, PathBuf, PathBuf), String> {
    let steam_libs = get_steam_libraries();
    let steam_root = steam_libs.first().cloned().unwrap_or_else(|| {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join(".local").join("share").join("Steam")
    });

    let proton_candidates = [
        "Proton - Experimental",
        "Proton 9.0 (Beta)",
        "Proton 9.0",
        "Proton 8.0",
        "Proton 7.0",
        "Proton: Next",
        "Proton Hotfix",
    ];

    let mut found_proton: Option<PathBuf> = None;
    for lib in &steam_libs {
        for name in &proton_candidates {
            let p = lib.join("steamapps").join("common").join(name).join("proton");
            if p.is_file() {
                found_proton = Some(p);
                break;
            }
        }
        if found_proton.is_some() {
            break;
        }
    }

    if found_proton.is_none() {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let compat_dir = PathBuf::from(home).join(".local").join("share").join("Steam").join("compatibilitytools.d");
        if let Ok(entries) = fs::read_dir(&compat_dir) {
            for entry in entries.flatten() {
                let p = entry.path().join("proton");
                if p.is_file() {
                    found_proton = Some(p);
                    break;
                }
            }
        }
    }

    let proton = found_proton.ok_or_else(|| {
        "Proton executable not found in Steam libraries. Please install Proton (e.g. Proton Experimental or Proton 9.0) in Steam.".to_string()
    })?;

    let mut found_compatdata: Option<PathBuf> = None;
    for lib in &steam_libs {
        let cdata = lib.join("steamapps").join("compatdata").join("1085660");
        if cdata.exists() {
            found_compatdata = Some(cdata);
            break;
        }
    }

    let compatdata = found_compatdata.unwrap_or_else(|| {
        steam_root.join("steamapps").join("compatdata").join("1085660")
    });

    Ok((proton, compatdata, steam_root))
}

pub fn launch_game(game_root: String, language_code: Option<String>) -> CommandResult {
    if let Some(ref lang) = language_code {
        update_dawn_language(&game_root, lang);
    }

    let p = Path::new(&game_root);
    let exe_path = p.join(GAME_EXECUTABLE);

    if !exe_path.exists() {
        return CommandResult {
            success: false,
            message: None,
            error: Some("destiny2.exe not found in game folder".to_string()),
            cancelled: Some(false),
            count: None,
        };
    }

    if !is_build_86657(&exe_path) {
        return CommandResult {
            success: false,
            message: None,
            error: Some(format!(
                "Refusing to launch: destiny2.exe is not Build {EXPECTED_BUILD_ID}. Dawn cannot run on modern retail builds."
            )),
            cancelled: Some(false),
            count: None,
        };
    }

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        let cmd_path = p.join("launch-destiny.cmd");
        let target = if cmd_path.exists() { cmd_path } else { exe_path };

        let mut cmd = Command::new(target);
        cmd.current_dir(p)
            .env("DAWN_FOREST_BASELINE", "1")
            .creation_flags(0x00000008); // DETACHED_PROCESS

        match cmd.spawn() {
            Ok(_) => CommandResult {
                success: true,
                message: Some("Game launched".to_string()),
                error: None,
                cancelled: Some(false),
                count: None,
            },
            Err(e) => CommandResult {
                success: false,
                message: None,
                error: Some(format!("Failed to launch game: {}", e)),
                cancelled: Some(false),
                count: None,
            },
        }
    }

    #[cfg(not(windows))]
    {
        let sh_path = p.join("launch-destiny.sh");
        if sh_path.is_file() {
            let mut cmd = Command::new("sh");
            cmd.arg(sh_path)
                .current_dir(p)
                .env("DAWN_FOREST_BASELINE", "1");
            match cmd.spawn() {
                Ok(_) => return CommandResult {
                    success: true,
                    message: Some("Game launched via launch-destiny.sh".to_string()),
                    error: None,
                    cancelled: Some(false),
                    count: None,
                },
                Err(e) => return CommandResult {
                    success: false,
                    message: None,
                    error: Some(format!("Failed to execute launch-destiny.sh: {}", e)),
                    cancelled: Some(false),
                    count: None,
                },
            }
        }

        match find_proton_and_compatdata(p) {
            Ok((proton, compatdata, steam_root)) => {
                let mut cmd = Command::new(&proton);
                cmd.arg("run")
                    .arg(&exe_path)
                    .current_dir(p)
                    .env("STEAM_COMPAT_CLIENT_INSTALL_PATH", &steam_root)
                    .env("STEAM_COMPAT_DATA_PATH", &compatdata)
                    .env("DAWN_FOREST_BASELINE", "1");

                match cmd.spawn() {
                    Ok(_) => CommandResult {
                        success: true,
                        message: Some("Game launched via Proton".to_string()),
                        error: None,
                        cancelled: Some(false),
                        count: None,
                    },
                    Err(e) => CommandResult {
                        success: false,
                        message: None,
                        error: Some(format!("Failed to spawn Proton: {}", e)),
                        cancelled: Some(false),
                        count: None,
                    },
                }
            }
            Err(err) => CommandResult {
                success: false,
                message: None,
                error: Some(err),
                cancelled: Some(false),
                count: None,
            },
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
