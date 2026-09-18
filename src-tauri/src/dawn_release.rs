use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use tauri::{AppHandle, Emitter};

use crate::types::ProgressPayload;

const GITHUB_LATEST_RELEASE_URL: &str = "https://api.github.com/repos/isinternets/Dawn/releases/latest";

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct GitHubRelease {
    pub tag_name: String,
    pub name: Option<String>,
    pub html_url: Option<String>,
    pub assets: Vec<GitHubAsset>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GitHubAsset {
    pub name: String,
    pub size: u64,
    pub browser_download_url: String,
}

pub fn get_releases_dir() -> PathBuf {
    #[cfg(windows)]
    {
        let base = std::env::var("LOCALAPPDATA")
            .or_else(|_| std::env::var("APPDATA"))
            .unwrap_or_else(|_| "C:\\AppData\\Local".to_string());
        PathBuf::from(base).join("DawnInstaller").join("releases")
    }
    #[cfg(not(windows))]
    {
        let base = std::env::var("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
                PathBuf::from(home).join(".local").join("share")
            });
        base.join("DawnInstaller").join("releases")
    }
}

pub fn is_valid_release_dir(dir: &Path) -> bool {
    dir.is_dir()
        && dir.join("release.json").exists()
        && (dir.join("payload").join("steam_api64.dll").exists()
            || dir.join("steam_api64.dll").exists())
}

pub fn get_latest_cached_release_dir() -> Option<PathBuf> {
    let releases_dir = get_releases_dir();
    if !releases_dir.exists() {
        return None;
    }

    let mut valid_dirs: Vec<PathBuf> = Vec::new();
    if let Ok(entries) = fs::read_dir(&releases_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if is_valid_release_dir(&path) {
                valid_dirs.push(path);
            }
        }
    }

    valid_dirs.sort_by(|a, b| b.file_name().cmp(&a.file_name()));
    valid_dirs.into_iter().next()
}

const GITHUB_WEB_RELEASES_LATEST: &str = "https://github.com/isinternets/Dawn/releases/latest";

pub async fn resolve_tag_from_web_redirect() -> Result<GitHubRelease, String> {
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64)")
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|e| format!("Client error: {}", e))?;

    let response = client
        .get(GITHUB_WEB_RELEASES_LATEST)
        .send()
        .await
        .map_err(|e| format!("Web redirect request failed: {}", e))?;

    if let Some(loc) = response.headers().get("location") {
        let loc_str = loc.to_str().map_err(|_| "Invalid location header")?;
        if let Some(tag) = loc_str.split("/tag/").nth(1) {
            let tag_clean = tag.trim_matches('/').to_string();
            let version_num = tag_clean.trim_start_matches('v').to_string();
            let zip_name = format!("Dawn-{}.zip", version_num);
            let zip_url = format!(
                "https://github.com/isinternets/Dawn/releases/download/{}/{}",
                tag_clean, zip_name
            );
            let sha_name = format!("Dawn-{}.zip.sha256", version_num);
            let sha_url = format!(
                "https://github.com/isinternets/Dawn/releases/download/{}/{}",
                tag_clean, sha_name
            );

            return Ok(GitHubRelease {
                tag_name: tag_clean,
                name: Some(format!("Dawn {}", version_num)),
                html_url: Some(loc_str.to_string()),
                assets: vec![
                    GitHubAsset {
                        name: zip_name,
                        size: 4_000_000,
                        browser_download_url: zip_url,
                    },
                    GitHubAsset {
                        name: sha_name,
                        size: 100,
                        browser_download_url: sha_url,
                    },
                ],
            });
        }
    }
    Err("Could not resolve tag from GitHub web redirect".to_string())
}

pub async fn fetch_latest_release() -> Result<GitHubRelease, String> {
    let client = reqwest::Client::builder()
        .user_agent("DawnLauncher/1.0 (Windows NT 10.0; Win64; x64)")
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let api_res = client
        .get(GITHUB_LATEST_RELEASE_URL)
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await;

    if let Ok(response) = api_res {
        if response.status().is_success() {
            if let Ok(release) = response.json::<GitHubRelease>().await {
                return Ok(release);
            }
        }
    }

    // Fallback: Web URL redirect hook (bypasses unauthenticated API rate limits)
    resolve_tag_from_web_redirect().await
}

pub async fn ensure_latest_dawn_release(app: &AppHandle) -> Result<PathBuf, String> {
    let _ = app.emit(
        "depot:output",
        "[DAWN] Checking https://github.com/isinternets/Dawn/releases for latest updates...\r\n",
    );
    let _ = app.emit(
        "installer:progress",
        ProgressPayload {
            percent: 94,
            status: "Checking for latest Dawn release...".to_string(),
        },
    );

    match fetch_latest_release().await {
        Ok(release) => {
            let tag = release.tag_name.clone();
            let _ = app.emit(
                "depot:output",
                format!("[DAWN] Found latest release: {} on GitHub\r\n", tag),
            );

            let zip_asset = release
                .assets
                .iter()
                .find(|a| a.name.ends_with(".zip") && !a.name.ends_with(".zip.sha256"))
                .cloned()
                .ok_or_else(|| format!("Release {} does not contain a .zip asset", tag))?;

            let sha_asset = release
                .assets
                .iter()
                .find(|a| a.name.ends_with(".zip.sha256"))
                .cloned();

            let releases_dir = get_releases_dir();
            let target_dir = releases_dir.join(&tag);

            if is_valid_release_dir(&target_dir) {
                let _ = app.emit(
                    "depot:output",
                    format!("[DAWN] Latest release {} is already cached and verified.\r\n", tag),
                );
                return Ok(target_dir);
            }

            let mb = (zip_asset.size as f64) / (1024.0 * 1024.0);
            let _ = app.emit(
                "depot:output",
                format!("[DAWN] Downloading {} ({:.2} MB)...\r\n", zip_asset.name, mb),
            );
            let _ = app.emit(
                "installer:progress",
                ProgressPayload {
                    percent: 96,
                    status: format!("Downloading Dawn {} ({:.1} MB)...", tag, mb),
                },
            );

            tokio::fs::create_dir_all(&target_dir)
                .await
                .map_err(|e| format!("Failed to create directory {:?}: {}", target_dir, e))?;

            let expected_sha = if let Some(sha) = sha_asset {
                let client = reqwest::Client::builder()
                    .user_agent("DawnLauncher/1.0")
                    .build()
                    .unwrap_or_default();
                if let Ok(resp) = client.get(&sha.browser_download_url).send().await {
                    if resp.status().is_success() {
                        if let Ok(text) = resp.text().await {
                            text.split_whitespace().next().map(|s| s.to_lowercase())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            };

            let client = reqwest::Client::builder()
                .user_agent("DawnLauncher/1.0")
                .build()
                .map_err(|e| format!("Failed to build client: {}", e))?;

            let response = client
                .get(&zip_asset.browser_download_url)
                .send()
                .await
                .map_err(|e| format!("Failed to download {}: {}", zip_asset.name, e))?;

            if !response.status().is_success() {
                return Err(format!("Download failed with HTTP {}", response.status()));
            }

            let bytes = response
                .bytes()
                .await
                .map_err(|e| format!("Failed to read stream: {}", e))?;

            if let Some(expected) = expected_sha {
                let mut hasher = Sha256::new();
                hasher.update(&bytes);
                let actual = format!("{:x}", hasher.finalize());
                if actual.to_lowercase() != expected.to_lowercase() {
                    return Err(format!(
                        "Checksum verification failed for {}: expected {}, got {}",
                        zip_asset.name, expected, actual
                    ));
                }
                let _ = app.emit("depot:output", "[DAWN] SHA-256 verified successfully.\r\n");
            }

            let zip_path = target_dir.join(&zip_asset.name);
            tokio::fs::write(&zip_path, &bytes)
                .await
                .map_err(|e| format!("Failed to write zip: {}", e))?;

            let _ = app.emit("depot:output", "[DAWN] Extracting release files...\r\n");
            let _ = app.emit(
                "installer:progress",
                ProgressPayload {
                    percent: 98,
                    status: format!("Extracting Dawn {}...", tag),
                },
            );

            #[cfg(windows)]
            let extracted = {
                use std::os::windows::process::CommandExt;
                let mut tar_cmd = Command::new("tar");
                tar_cmd.args(["-xf", zip_path.to_str().unwrap(), "-C", target_dir.to_str().unwrap()]);
                tar_cmd.creation_flags(0x08000000);
                match tar_cmd.output() {
                    Ok(out) if out.status.success() => true,
                    _ => {
                        let ps_script = format!(
                            "Expand-Archive -LiteralPath '{}' -DestinationPath '{}' -Force",
                            zip_path.to_str().unwrap(),
                            target_dir.to_str().unwrap()
                        );
                        let mut ps_cmd = Command::new("powershell");
                        ps_cmd.args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", &ps_script]);
                        ps_cmd.creation_flags(0x08000000);
                        matches!(ps_cmd.output(), Ok(out) if out.status.success())
                    }
                }
            };

            #[cfg(not(windows))]
            let extracted = {
                let mut unzip_cmd = Command::new("unzip");
                unzip_cmd.args(["-o", zip_path.to_str().unwrap(), "-d", target_dir.to_str().unwrap()]);
                if matches!(unzip_cmd.output(), Ok(ref out) if out.status.success()) {
                    true
                } else {
                    // Python 3 fallback (present on NixOS and virtually all Linux distributions)
                    let py_script = format!(
                        "import zipfile; zipfile.ZipFile('{}').extractall('{}')",
                        zip_path.to_str().unwrap(),
                        target_dir.to_str().unwrap()
                    );
                    let mut py_cmd = Command::new("python3");
                    py_cmd.args(["-c", &py_script]);
                    if matches!(py_cmd.output(), Ok(ref out) if out.status.success()) {
                        true
                    } else {
                        // Tar fallback (works if bsdtar is installed)
                        let mut tar_cmd = Command::new("tar");
                        tar_cmd.args(["-xf", zip_path.to_str().unwrap(), "-C", target_dir.to_str().unwrap()]);
                        matches!(tar_cmd.output(), Ok(ref out) if out.status.success())
                    }
                }
            };

            let _ = tokio::fs::remove_file(&zip_path).await;

            if !extracted || !is_valid_release_dir(&target_dir) {
                return Err(format!(
                    "Failed to extract or validate release archive at {:?}",
                    target_dir
                ));
            }

            let _ = app.emit(
                "depot:output",
                format!("[DAWN] Successfully downloaded and prepared Dawn {}\r\n", tag),
            );
            Ok(target_dir)
        }
        Err(err) => {
            let _ = app.emit(
                "depot:output",
                format!(
                    "[WARN] GitHub release check failed ({}); checking local cache...\r\n",
                    err
                ),
            );

            if let Some(cached) = get_latest_cached_release_dir() {
                let _ = app.emit(
                    "depot:output",
                    format!("[DAWN] Using cached release at {:?}\r\n", cached),
                );
                return Ok(cached);
            }

            let bundled = crate::installer::get_bundled_payload_dir();
            let _ = app.emit(
                "depot:output",
                format!("[DAWN] Using bundled release at {:?}\r\n", bundled),
            );
            Ok(bundled)
        }
    }
}

pub async fn deploy_dawn_to_game(
    app: &AppHandle,
    release_dir: &Path,
    game_root: &str,
) -> Result<(), String> {
    let target = Path::new(game_root);

    // 1. Check if Destiny 2 is running
    let mut sys = sysinfo::System::new();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
    let is_running = sys.processes().values().any(|p| {
        let name = p.name().to_string_lossy().to_lowercase();
        name == "destiny2" || name == "destiny2.exe"
    });
    if is_running {
        return Err("Destiny 2 is currently running. Please close the game before installing or updating Dawn.".to_string());
    }

    let _ = app.emit(
        "depot:output",
        format!("[DAWN] Deploying Dawn files to game directory {:?}...\r\n", target),
    );

    // 2. Backup original steam_api64.dll to .dawn/backup/ if not already backed up
    let backup_dir = target.join(".dawn").join("backup");
    let _ = fs::create_dir_all(&backup_dir);
    let backup_dll = backup_dir.join("steam_api64.dll");

    if !backup_dll.exists() {
        let original_candidates = [
            target.join("bin").join("x64").join("steam_api64.dll"),
            target.join("steam_api64.dll"),
        ];
        for orig in &original_candidates {
            if orig.is_file() {
                if fs::copy(orig, &backup_dll).is_ok() {
                    let _ = app.emit("depot:output", "[DAWN] Successfully backed up original steam_api64.dll\r\n");
                    break;
                }
            }
        }
    }

    // 3. Resolve payload directory
    let payload = if release_dir.join("payload").exists() {
        release_dir.join("payload")
    } else {
        release_dir.to_path_buf()
    };

    if !payload.exists() {
        return Err(format!("Payload folder not found at {:?}", payload));
    }

    // 4. Copy payload contents to game root
    let _ = app.emit("depot:output", "[DAWN] Copying mod payload files...\r\n");
    if let Err(e) = crate::installer::copy_dir_all(&payload, target) {
        let _ = app.emit("depot:output", format!("[WARN] Copy notification: {}\r\n", e));
    }

    // 5. Ensure Dawn directory and steam_api64.dll are in bin/x64 as well
    let dawn_sub = payload.join("Dawn");
    let bin_dawn = target.join("bin").join("x64").join("Dawn");
    if dawn_sub.exists() {
        let _ = crate::installer::copy_dir_all(&dawn_sub, &bin_dawn);
    }

    let steam_dll = payload.join("steam_api64.dll");
    let bin_steam_dll = target.join("bin").join("x64").join("steam_api64.dll");
    if steam_dll.exists() {
        if let Some(parent) = bin_steam_dll.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::copy(&steam_dll, &bin_steam_dll);
        let _ = fs::copy(&steam_dll, target.join("steam_api64.dll"));
    }

    // 6. Copy release metadata
    let rel_json = release_dir.join("release.json");
    if rel_json.exists() {
        let dawn_meta = target.join(".dawn");
        let _ = fs::create_dir_all(&dawn_meta);
        let _ = fs::copy(&rel_json, dawn_meta.join("release.json"));
    }

    let _ = app.emit("depot:output", "[DAWN] Dawn mod deployed successfully!\r\n");
    Ok(())
}

pub async fn get_latest_dawn_version() -> Option<String> {
    // 1. Try remote GitHub latest release
    if let Ok(rel) = fetch_latest_release().await {
        let tag = rel.tag_name.trim().trim_start_matches(['v', 'V']).to_string();
        if !tag.is_empty() {
            return Some(tag);
        }
    }

    // 2. Check local cached releases
    if let Some(cached_dir) = get_latest_cached_release_dir() {
        let rel_json = cached_dir.join("release.json");
        if let Ok(content) = fs::read_to_string(&rel_json) {
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(ver) = parsed
                    .get("release")
                    .or_else(|| parsed.get("version"))
                    .and_then(|v| v.as_str())
                {
                    return Some(ver.trim().trim_start_matches(['v', 'V']).to_string());
                }
            }
        }
        if let Some(folder_name) = cached_dir.file_name().and_then(|n| n.to_str()) {
            let clean = folder_name.trim().trim_start_matches(['v', 'V']);
            if !clean.is_empty() {
                return Some(clean.to_string());
            }
        }
    }

    // 3. Check bundled payload
    let bundled = crate::installer::get_bundled_payload_dir();
    let rel_json = bundled.join("release.json");
    if let Ok(content) = fs::read_to_string(&rel_json) {
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(ver) = parsed
                .get("release")
                .or_else(|| parsed.get("version"))
                .and_then(|v| v.as_str())
            {
                return Some(ver.trim().trim_start_matches(['v', 'V']).to_string());
            }
        }
    }

    None
}
