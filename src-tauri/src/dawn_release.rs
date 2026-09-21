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
    #[serde(default)]
    pub body: Option<String>,
    #[serde(default)]
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
    if !dir.is_dir() {
        return false;
    }
    let has_dll = dir.join("payload").join("steam_api64.dll").exists()
        || dir.join("steam_api64.dll").exists()
        || dir.join("bin").join("x64").join("steam_api64.dll").exists();
    let has_meta_or_dawn = dir.join("release.json").exists()
        || dir.join(".dawn").join("release.json").exists()
        || dir.join("payload").join("Dawn").exists()
        || dir.join("Dawn").exists()
        || dir.join("payload").exists();
    has_dll && has_meta_or_dawn
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
            let version_num = tag_clean.trim_start_matches(['v', 'V']).to_string();

            let mut assets = Vec::new();

            // 1. Try querying GitHub's expanded_assets HTML endpoint for exact asset names and URLs
            let expanded_url = format!(
                "https://github.com/isinternets/Dawn/releases/expanded_assets/{}",
                tag_clean
            );
            let fetch_client = reqwest::Client::builder()
                .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64)")
                .build()
                .unwrap_or_default();

            if let Ok(exp_res) = fetch_client.get(&expanded_url).send().await {
                if exp_res.status().is_success() {
                    if let Ok(html) = exp_res.text().await {
                        let prefix = format!("/releases/download/{}/", tag_clean);
                        let mut cursor = 0;
                        while let Some(pos) = html[cursor..].find(&prefix) {
                            let start = cursor + pos + prefix.len();
                            if let Some(end) = html[start..].find('"') {
                                let filename = html[start..start + end].trim();
                                if !filename.is_empty() {
                                    let download_url = format!(
                                        "https://github.com/isinternets/Dawn/releases/download/{}/{}",
                                        tag_clean, filename
                                    );
                                    if !assets.iter().any(|a: &GitHubAsset| a.browser_download_url == download_url) {
                                        assets.push(GitHubAsset {
                                            name: filename.to_string(),
                                            size: 0,
                                            browser_download_url: download_url,
                                        });
                                    }
                                }
                                cursor = start + end;
                            } else {
                                break;
                            }
                        }
                    }
                }
            }

            // 2. If expanded_assets didn't return assets, generate all possible naming patterns
            if assets.is_empty() {
                let candidates = [
                    format!("{}.zip", version_num),
                    format!("Dawn-{}.zip", version_num),
                    format!("{}.zip", tag_clean),
                    format!("Dawn-{}.zip", tag_clean),
                    "Dawn.zip".to_string(),
                ];
                for cand in &candidates {
                    assets.push(GitHubAsset {
                        name: cand.clone(),
                        size: 4_000_000,
                        browser_download_url: format!(
                            "https://github.com/isinternets/Dawn/releases/download/{}/{}",
                            tag_clean, cand
                        ),
                    });
                }
                let sha_candidates = [
                    "SHA256SUMS.txt".to_string(),
                    format!("{}.zip.sha256", version_num),
                    format!("Dawn-{}.zip.sha256", version_num),
                ];
                for sha in &sha_candidates {
                    assets.push(GitHubAsset {
                        name: sha.clone(),
                        size: 100,
                        browser_download_url: format!(
                            "https://github.com/isinternets/Dawn/releases/download/{}/{}",
                            tag_clean, sha
                        ),
                    });
                }
            }

            return Ok(GitHubRelease {
                tag_name: tag_clean,
                name: Some(format!("Dawn {}", version_num)),
                html_url: Some(loc_str.to_string()),
                body: None,
                assets,
            });
        }
    }
    Err("Could not resolve tag from GitHub web redirect".to_string())
}

pub fn extract_assets_from_markdown_body(body: &str, assets: &mut Vec<GitHubAsset>) {
    for line in body.lines() {
        let trimmed = line.trim();
        let mut rem = trimmed;
        while let Some(start_bracket) = rem.find('[') {
            if let Some(end_bracket_offset) = rem[start_bracket..].find(']') {
                let end_bracket = start_bracket + end_bracket_offset;
                if rem.len() > end_bracket + 1 && rem.as_bytes()[end_bracket + 1] == b'(' {
                    if let Some(end_paren_offset) = rem[end_bracket + 2..].find(')') {
                        let end_paren = end_bracket + 2 + end_paren_offset;
                        let label = rem[start_bracket + 1..end_bracket].trim();
                        let url = rem[end_bracket + 2..end_paren].trim();
                        if url.starts_with("http") {
                            let label_lower = label.to_lowercase();
                            let url_lower = url.to_lowercase();
                            if label_lower.ends_with(".zip")
                                || label_lower.ends_with(".sha256")
                                || label_lower.contains("sha256")
                                || label_lower.contains("checksum")
                                || url_lower.ends_with(".zip")
                            {
                                let filename = if label_lower.ends_with(".zip") || label_lower.contains("sha256") {
                                    label.to_string()
                                } else {
                                    url.rsplit('/').next().unwrap_or("release.zip").to_string()
                                };
                                if !assets.iter().any(|a| a.browser_download_url == url) {
                                    assets.push(GitHubAsset {
                                        name: filename,
                                        size: 0,
                                        browser_download_url: url.to_string(),
                                    });
                                }
                            }
                        }
                        rem = &rem[end_paren + 1..];
                        continue;
                    }
                }
            }
            break;
        }
    }
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
            if let Ok(mut release) = response.json::<GitHubRelease>().await {
                let has_zip = release.assets.iter().any(|a| {
                    let n = a.name.to_lowercase();
                    n.ends_with(".zip") && !n.ends_with(".zip.sha256") && !n.contains("source")
                });
                if !has_zip {
                    if let Some(body) = &release.body {
                        extract_assets_from_markdown_body(body, &mut release.assets);
                    }
                }
                return Ok(release);
            }
        }
    }

    // Fallback: Web URL redirect hook (bypasses unauthenticated API rate limits)
    resolve_tag_from_web_redirect().await
}

async fn download_and_extract_release(app: &AppHandle, release: &GitHubRelease) -> Result<PathBuf, String> {
    let tag = release.tag_name.clone();
    let _ = app.emit(
        "depot:output",
        format!("[DAWN] Found latest release: {} on GitHub\r\n", tag),
    );

    let releases_dir = get_releases_dir();
    let target_dir = releases_dir.join(&tag);

    if is_valid_release_dir(&target_dir) {
        let _ = app.emit(
            "depot:output",
            format!("[DAWN] Latest release {} is already cached and verified.\r\n", tag),
        );
        return Ok(target_dir);
    }

    let mut zip_candidates: Vec<GitHubAsset> = release
        .assets
        .iter()
        .filter(|a| {
            let n = a.name.to_lowercase();
            n.ends_with(".zip") && !n.ends_with(".zip.sha256") && !n.contains("source")
        })
        .cloned()
        .collect();

    let version_num = tag.trim_start_matches(['v', 'V']).to_string();
    let fallback_names = [
        format!("{}.zip", version_num),
        format!("Dawn-{}.zip", version_num),
        format!("{}.zip", tag),
        format!("Dawn-{}.zip", tag),
        "Dawn.zip".to_string(),
    ];
    for name in &fallback_names {
        let url = format!(
            "https://github.com/isinternets/Dawn/releases/download/{}/{}",
            tag, name
        );
        if !zip_candidates.iter().any(|a| a.browser_download_url == url) {
            zip_candidates.push(GitHubAsset {
                name: name.clone(),
                size: 0,
                browser_download_url: url,
            });
        }
    }

    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64)")
        .build()
        .map_err(|e| format!("Failed to build client: {}", e))?;

    let mut downloaded: Option<(GitHubAsset, Vec<u8>)> = None;
    for cand in &zip_candidates {
        let mb = (cand.size as f64) / (1024.0 * 1024.0);
        let size_desc = if mb > 0.1 { format!(" ({:.2} MB)", mb) } else { String::new() };
        let _ = app.emit(
            "depot:output",
            format!("[DAWN] Downloading {}{}...\r\n", cand.name, size_desc),
        );
        let _ = app.emit(
            "installer:progress",
            ProgressPayload {
                percent: 96,
                status: format!("Downloading Dawn {}{}...", tag, size_desc),
            },
        );

        match client.get(&cand.browser_download_url).send().await {
            Ok(resp) if resp.status().is_success() => {
                match resp.bytes().await {
                    Ok(b) if !b.is_empty() => {
                        downloaded = Some((cand.clone(), b.to_vec()));
                        break;
                    }
                    _ => {}
                }
            }
            Ok(resp) => {
                let _ = app.emit(
                    "depot:output",
                    format!("[DAWN] Candidate {} returned HTTP {}\r\n", cand.name, resp.status()),
                );
            }
            Err(e) => {
                let _ = app.emit(
                    "depot:output",
                    format!("[DAWN] Download failed for {}: {}\r\n", cand.name, e),
                );
            }
        }
    }

    let (zip_asset, bytes) = downloaded.ok_or_else(|| {
        format!("Failed to download player .zip for Dawn {}", tag)
    })?;

    tokio::fs::create_dir_all(&target_dir)
        .await
        .map_err(|e| format!("Failed to create directory {:?}: {}", target_dir, e))?;

    let mut expected_sha: Option<String> = None;
    if let Some(sha_asset) = release.assets.iter().find(|a| {
        let n = a.name.to_lowercase();
        n.ends_with(".sha256") || n.contains("sha256") || n.contains("checksum")
    }) {
        if let Ok(resp) = client.get(&sha_asset.browser_download_url).send().await {
            if resp.status().is_success() {
                if let Ok(text) = resp.text().await {
                    for line in text.lines() {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 2 {
                            let hash = parts[0];
                            let fname = parts[1].trim_start_matches('*');
                            if fname == zip_asset.name || fname.ends_with(&zip_asset.name) || zip_asset.name.ends_with(fname) {
                                expected_sha = Some(hash.to_lowercase());
                                break;
                            }
                        }
                    }
                    if expected_sha.is_none() {
                        expected_sha = text.split_whitespace().next().map(|s| s.to_lowercase());
                    }
                }
            }
        }
    }

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
                let mut tar_cmd = Command::new("tar");
                tar_cmd.args(["-xf", zip_path.to_str().unwrap(), "-C", target_dir.to_str().unwrap()]);
                matches!(tar_cmd.output(), Ok(ref out) if out.status.success())
            }
        }
    };

    let _ = tokio::fs::remove_file(&zip_path).await;

    // Ensure target_dir has release.json recording tag_name and installed_tag
    let rel_path = target_dir.join("release.json");
    if !rel_path.exists() {
        let minimal = serde_json::json!({
            "schema": 1,
            "release": tag,
            "tag_name": tag,
            "installed_tag": tag,
            "gameBuild": 86657,
            "runtimeDirectory": "Dawn"
        });
        let _ = fs::write(&rel_path, serde_json::to_string_pretty(&minimal).unwrap_or_default());
    } else if let Ok(content) = fs::read_to_string(&rel_path) {
        let mut rel_val = serde_json::from_str::<serde_json::Value>(&content)
            .unwrap_or_else(|_| serde_json::json!({}));
        if let Some(obj) = rel_val.as_object_mut() {
            obj.insert("tag_name".to_string(), serde_json::json!(tag));
            obj.insert("installed_tag".to_string(), serde_json::json!(tag));
        }
        let _ = fs::write(&rel_path, serde_json::to_string_pretty(&rel_val).unwrap_or_default());
    }

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

    let download_error = match fetch_latest_release().await {
        Ok(release) => match download_and_extract_release(app, &release).await {
            Ok(dir) => return Ok(dir),
            Err(err) => Some(err),
        },
        Err(err) => Some(err),
    };

    if let Some(ref err) = download_error {
        let _ = app.emit(
            "depot:output",
            format!(
                "[WARN] Remote Dawn release retrieval failed ({}); checking local cache...\r\n",
                err
            ),
        );
        crate::logger::log_msg(
            "WARN",
            &format!("Remote Dawn release retrieval failed: {}", err),
            Some(app),
        );
    }

    if let Some(cached) = get_latest_cached_release_dir() {
        let _ = app.emit(
            "depot:output",
            format!("[DAWN] Using cached release at {:?}\r\n", cached),
        );
        return Ok(cached);
    }

    let bundled = crate::installer::get_bundled_payload_dir();
    if is_valid_release_dir(&bundled) {
        let _ = app.emit(
            "depot:output",
            format!("[DAWN] Using bundled release at {:?}\r\n", bundled),
        );
        return Ok(bundled);
    }

    Err(format!(
        "Failed to download Dawn online ({}) and no local cache or bundled release is available.",
        download_error.unwrap_or_else(|| "Unknown error".to_string())
    ))
}

pub fn merge_directories_preserving_user_data(from: &Path, to: &Path) -> Result<(), String> {
    if !from.is_dir() {
        return Ok(());
    }
    fs::create_dir_all(to).map_err(|e| format!("Failed to create directory {:?}: {}", to, e))?;

    for entry in fs::read_dir(from).map_err(|e| format!("Failed to read directory {:?}: {}", from, e))? {
        let entry = entry.map_err(|e| format!("Failed to read dir entry: {}", e))?;
        let from_path = entry.path();
        let to_path = to.join(entry.file_name());

        if from_path.is_dir() {
            merge_directories_preserving_user_data(&from_path, &to_path)?;
        } else if from_path.is_file() {
            if !to_path.exists() {
                let _ = fs::copy(&from_path, &to_path);
            } else {
                let overwrite = match (fs::metadata(&from_path), fs::metadata(&to_path)) {
                    (Ok(meta_from), Ok(meta_to)) => {
                        let from_modified = meta_from.modified().ok();
                        let to_modified = meta_to.modified().ok();
                        if let (Some(from_time), Some(to_time)) = (from_modified, to_modified) {
                            from_time >= to_time || meta_from.len() != meta_to.len()
                        } else {
                            true
                        }
                    }
                    _ => true,
                };
                if overwrite {
                    let _ = fs::copy(&from_path, &to_path);
                }
            }
        }
    }
    Ok(())
}

pub fn deploy_preserving_user_settings(from: &Path, to: &Path) -> Result<(), String> {
    if !from.is_dir() {
        return Ok(());
    }
    fs::create_dir_all(to).map_err(|e| format!("Failed to create directory {:?}: {}", to, e))?;

    for entry in fs::read_dir(from).map_err(|e| format!("Failed to read directory {:?}: {}", from, e))? {
        let entry = entry.map_err(|e| format!("Failed to read dir entry: {}", e))?;
        let from_path = entry.path();
        let to_path = to.join(entry.file_name());

        if from_path.is_dir() {
            deploy_preserving_user_settings(&from_path, &to_path)?;
        } else if from_path.is_file() {
            let fname = entry.file_name();
            let fname_str = fname.to_string_lossy();
            let is_user_progress = fname_str == "settings.json"
                || fname_str == "player.json"
                || fname_str == "hud.json"
                || fname_str == "movement.json"
                || fname_str.ends_with(".db")
                || fname_str.ends_with(".sqlite");

            if is_user_progress && to_path.exists() {
                continue;
            }

            let _ = fs::copy(&from_path, &to_path);
        }
    }
    Ok(())
}

pub fn migrate_legacy_root_dawn(target: &Path, app: Option<&AppHandle>) -> Result<bool, String> {
    let root_dawn = target.join("Dawn");
    let root_steam_dll = target.join("steam_api64.dll");

    if !root_dawn.is_dir() && !root_steam_dll.is_file() {
        return Ok(false);
    }

    let bin_x64 = target.join("bin").join("x64");
    let bin_dawn = bin_x64.join("Dawn");
    let bin_steam_dll = bin_x64.join("steam_api64.dll");

    // 1. If root Dawn directory exists, backup and merge
    if root_dawn.is_dir() {
        if let Some(a) = app {
            let _ = a.emit(
                "depot:output",
                "[MIGRATE] Migrating legacy root Dawn directory to bin/x64/Dawn...\r\n",
            );
        }
        crate::logger::log_msg(
            "INFO",
            &format!("Migrating legacy root Dawn at {:?} to {:?}", root_dawn, bin_dawn),
            app,
        );

        // A. Safety backup to .dawn/root_migration_backup
        let backup_dir = target.join(".dawn").join("root_migration_backup");
        let _ = fs::create_dir_all(&backup_dir);
        let _ = crate::installer::copy_dir_all(&root_dawn, &backup_dir);

        // B. Ensure bin/x64/Dawn exists
        let _ = fs::create_dir_all(&bin_dawn);

        // C. Recursively merge all contents and subdirectories of root_dawn into bin_dawn
        merge_directories_preserving_user_data(&root_dawn, &bin_dawn)?;

        // D. Remove root Dawn folder
        let _ = fs::remove_dir_all(&root_dawn);
        if let Some(a) = app {
            let _ = a.emit(
                "depot:output",
                "[MIGRATE] Root Dawn directory successfully migrated to bin/x64/Dawn and cleaned up.\r\n",
            );
        }
    }

    // 2. If root steam_api64.dll exists, ensure bin/x64/steam_api64.dll exists, then clean up root
    if root_steam_dll.is_file() {
        if !bin_steam_dll.is_file() {
            let _ = fs::create_dir_all(&bin_x64);
            let _ = fs::copy(&root_steam_dll, &bin_steam_dll);
            crate::logger::log_msg("INFO", "Moved root steam_api64.dll to bin/x64/steam_api64.dll", app);
        }

        // If the root DLL is the mod proxy (not genuine Steam DLL), remove it so only bin/x64 is used
        if !crate::installer::is_genuine_steam_dll(&root_steam_dll) {
            let _ = fs::remove_file(&root_steam_dll);
            crate::logger::log_msg("INFO", "Removed legacy steam_api64.dll proxy from game root", app);
        }
    }

    Ok(true)
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

    // 2. Perform silent migration of legacy root files if present
    let _ = migrate_legacy_root_dawn(target, Some(app));

    let _ = app.emit(
        "depot:output",
        format!("[DAWN] Deploying Dawn files to bin/x64 in {:?}...\r\n", target),
    );

    // 3. Backup original steam_api64.dll to .dawn/backup/ if not already backed up
    let backup_dir = target.join(".dawn").join("backup");
    let _ = fs::create_dir_all(&backup_dir);
    let backup_dll = backup_dir.join("steam_api64.dll");

    if !backup_dll.exists() {
        let original_candidates = [
            target.join("bin").join("x64").join("steam_api64.dll"),
            target.join("steam_api64.dll"),
        ];
        for orig in &original_candidates {
            if orig.is_file() && crate::installer::is_genuine_steam_dll(orig) {
                if fs::copy(orig, &backup_dll).is_ok() {
                    let _ = app.emit("depot:output", "[DAWN] Successfully backed up original steam_api64.dll\r\n");
                    break;
                }
            }
        }
    }

    // 4. Resolve payload directory
    let payload = if release_dir.join("payload").exists() {
        release_dir.join("payload")
    } else {
        release_dir.to_path_buf()
    };

    if !payload.exists() {
        return Err(format!("Payload folder not found at {:?}", payload));
    }

    let bin_x64 = target.join("bin").join("x64");
    let _ = fs::create_dir_all(&bin_x64);

    // 5. Deploy Dawn mod directory ONLY to bin/x64/Dawn, preserving user settings if existing
    let dawn_sub = payload.join("Dawn");
    let bin_dawn = bin_x64.join("Dawn");
    if dawn_sub.exists() {
        let _ = app.emit("depot:output", "[DAWN] Deploying Dawn files to bin/x64/Dawn...\r\n");
        if bin_dawn.exists() {
            deploy_preserving_user_settings(&dawn_sub, &bin_dawn)?;
        } else {
            let _ = crate::installer::copy_dir_all(&dawn_sub, &bin_dawn);
        }
    }

    // 6. Deploy steam_api64.dll ONLY to bin/x64/steam_api64.dll
    let steam_dll = payload.join("steam_api64.dll");
    let bin_steam_dll = bin_x64.join("steam_api64.dll");
    if steam_dll.exists() {
        let _ = fs::copy(&steam_dll, &bin_steam_dll);
    }

    // Ensure root directory has no lingering Dawn folder or proxy steam_api64.dll
    let root_dawn = target.join("Dawn");
    if root_dawn.exists() {
        let _ = fs::remove_dir_all(&root_dawn);
    }
    let root_steam_dll = target.join("steam_api64.dll");
    if root_steam_dll.exists() && !crate::installer::is_genuine_steam_dll(&root_steam_dll) {
        let _ = fs::remove_file(&root_steam_dll);
    }

    // Purge steam_appid.txt from root and bin/x64 (triggers 'Problem reading game content' in Destiny 2)
    let appid_root = target.join("steam_appid.txt");
    if appid_root.exists() {
        let _ = fs::remove_file(&appid_root);
    }
    let appid_bin = bin_x64.join("steam_appid.txt");
    if appid_bin.exists() {
        let _ = fs::remove_file(&appid_bin);
    }

    // 7. Copy release metadata and record installed tag
    let dawn_meta = target.join(".dawn");
    let _ = fs::create_dir_all(&dawn_meta);
    let rel_json = release_dir.join("release.json");
    let mut rel_val = if rel_json.exists() {
        serde_json::from_str::<serde_json::Value>(&fs::read_to_string(&rel_json).unwrap_or_default())
            .unwrap_or_else(|_| serde_json::json!({}))
    } else {
        serde_json::json!({})
    };
    if let Some(obj) = rel_val.as_object_mut() {
        if let Some(tag) = release_dir.file_name().and_then(|f| f.to_str()) {
            obj.insert("tag_name".to_string(), serde_json::json!(tag));
            obj.insert("installed_tag".to_string(), serde_json::json!(tag));
            if !obj.contains_key("release") {
                obj.insert("release".to_string(), serde_json::json!(tag));
            }
        }
    }
    let formatted = serde_json::to_string_pretty(&rel_val).unwrap_or_default();
    let _ = fs::write(dawn_meta.join("release.json"), &formatted);
    let _ = fs::write(bin_x64.join("release.json"), &formatted);
    let _ = fs::write(target.join("release.json"), &formatted);

    crate::installer::ensure_launch_scripts(target);
    crate::installer::ensure_vc_runtime_files(target, Some(app));
    crate::installer::unblock_game_files(target);
    crate::installer::ensure_cvars_windowed_fullscreen(Some(app));

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deploy_preserving_user_settings() {
        let temp_dir = std::env::temp_dir().join(format!("dawn_test_preserve_{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp_dir);
        let from_dir = temp_dir.join("release_payload");
        let to_dir = temp_dir.join("game_dawn");

        let _ = fs::create_dir_all(&from_dir);
        let _ = fs::create_dir_all(&to_dir);

        // User already has custom settings.json
        fs::write(to_dir.join("settings.json"), b"{\"user_setting\": 123}").unwrap();
        // Release payload has default settings.json and a new script
        fs::write(from_dir.join("settings.json"), b"{\"default_setting\": 0}").unwrap();
        let _ = fs::create_dir_all(from_dir.join("scripts"));
        fs::write(from_dir.join("scripts").join("new_script.lua"), b"-- new script").unwrap();

        let res = deploy_preserving_user_settings(&from_dir, &to_dir);
        assert!(res.is_ok());

        // Settings should be preserved
        let settings_content = fs::read_to_string(to_dir.join("settings.json")).unwrap();
        assert_eq!(settings_content, "{\"user_setting\": 123}");

        // New script should be deployed
        assert!(to_dir.join("scripts").join("new_script.lua").is_file());

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_migrate_legacy_root_dawn() {
        let temp_dir = std::env::temp_dir().join(format!("dawn_test_migrate_{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp_dir);
        let _ = fs::create_dir_all(&temp_dir);

        // Dummy destiny2.exe in root
        fs::write(temp_dir.join("destiny2.exe"), b"dummy exe").unwrap();

        // Dummy legacy root Dawn with user files and nested folders
        let root_dawn = temp_dir.join("Dawn");
        let _ = fs::create_dir_all(root_dawn.join("scripts"));
        let _ = fs::create_dir_all(root_dawn.join("custom").join("subfolder"));
        fs::write(root_dawn.join("settings.json"), b"{\"user_progress\": 999}").unwrap();
        fs::write(root_dawn.join("player.json"), b"{\"character\": \"hunter\"}").unwrap();
        fs::write(root_dawn.join("scripts").join("my_macro.lua"), b"-- user macro").unwrap();
        fs::write(root_dawn.join("custom").join("subfolder").join("data.txt"), b"saved data").unwrap();

        // Legacy proxy DLL in root
        fs::write(temp_dir.join("steam_api64.dll"), b"dummy proxy dll").unwrap();

        // Execute migration
        let res = migrate_legacy_root_dawn(&temp_dir, None);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), true);

        // 1. Root Dawn directory should be removed
        assert!(!temp_dir.join("Dawn").exists(), "Root Dawn folder must be removed");

        // 2. Root proxy steam_api64.dll should be removed
        assert!(!temp_dir.join("steam_api64.dll").exists(), "Root steam_api64.dll proxy must be removed");

        // 3. Files must now be in bin/x64/Dawn
        let bin_dawn = temp_dir.join("bin").join("x64").join("Dawn");
        assert!(bin_dawn.is_dir(), "bin/x64/Dawn must exist");
        assert_eq!(
            fs::read_to_string(bin_dawn.join("settings.json")).unwrap(),
            "{\"user_progress\": 999}"
        );
        assert_eq!(
            fs::read_to_string(bin_dawn.join("player.json")).unwrap(),
            "{\"character\": \"hunter\"}"
        );
        assert_eq!(
            fs::read_to_string(bin_dawn.join("scripts").join("my_macro.lua")).unwrap(),
            "-- user macro"
        );
        assert_eq!(
            fs::read_to_string(bin_dawn.join("custom").join("subfolder").join("data.txt")).unwrap(),
            "saved data"
        );

        // 4. steam_api64.dll must be in bin/x64
        assert!(temp_dir.join("bin").join("x64").join("steam_api64.dll").is_file());

        // 5. Backup in .dawn/root_migration_backup must exist
        let backup = temp_dir.join(".dawn").join("root_migration_backup");
        assert!(backup.join("settings.json").is_file());
        assert!(backup.join("scripts").join("my_macro.lua").is_file());

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
