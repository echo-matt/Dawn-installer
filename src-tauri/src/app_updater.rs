use std::process::Command;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use futures_util::StreamExt;
use tokio::io::AsyncWriteExt;

const LAUNCHER_REPO_API: &str = "https://api.github.com/repos/echo-matt/Dawn-installer/releases/latest";
const LAUNCHER_REPO_REDIRECT: &str = "https://github.com/echo-matt/Dawn-installer/releases/latest";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppUpdateInfo {
    pub update_available: bool,
    pub current_version: String,
    pub latest_version: String,
    pub release_notes: Option<String>,
    pub asset_name: Option<String>,
    pub asset_url: Option<String>,
    pub html_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProgressPayload {
    pub percent: u32,
    pub status: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GitHubReleaseAsset {
    pub name: String,
    pub browser_download_url: String,
}

#[derive(Debug, Clone, Deserialize)]
struct GitHubReleaseResponse {
    tag_name: String,
    body: Option<String>,
    html_url: Option<String>,
    assets: Vec<GitHubReleaseAsset>,
}

/// Parses semantic version string (e.g. "v1.1.2" or "1.1.2") into (major, minor, patch)
pub fn parse_semver(v: &str) -> (u32, u32, u32) {
    let clean = v.trim().trim_start_matches(['v', 'V']);
    let parts: Vec<&str> = clean.split('.').collect();
    let major = parts.get(0).and_then(|s| s.parse().ok()).unwrap_or(0);
    let minor = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
    let patch = parts.get(2).and_then(|s| s.split('-').next().unwrap_or("0").parse().ok()).unwrap_or(0);
    (major, minor, patch)
}

/// Compares two semver strings: returns true if `latest` is strictly newer than `current`.
pub fn is_newer_version(latest: &str, current: &str) -> bool {
    let l = parse_semver(latest);
    let c = parse_semver(current);
    l > c
}

/// Selects the best installer asset for the current OS from a list of release assets.
pub fn select_best_asset(assets: &[GitHubReleaseAsset]) -> Option<(String, String)> {
    #[cfg(windows)]
    {
        // 1. Prefer NSIS setup exe (e.g. DAWN-Setup-v1.1.2.exe)
        for a in assets {
            let lower = a.name.to_lowercase();
            if lower.contains("setup") && lower.ends_with(".exe") {
                return Some((a.name.clone(), a.browser_download_url.clone()));
            }
        }
        // 2. Fallback to any .exe
        for a in assets {
            let lower = a.name.to_lowercase();
            if lower.ends_with(".exe") {
                return Some((a.name.clone(), a.browser_download_url.clone()));
            }
        }
        // 3. Fallback to .msi
        for a in assets {
            let lower = a.name.to_lowercase();
            if lower.ends_with(".msi") {
                return Some((a.name.clone(), a.browser_download_url.clone()));
            }
        }
    }

    #[cfg(not(windows))]
    {
        // On Linux, prefer AppImage
        for a in assets {
            let lower = a.name.to_lowercase();
            if lower.ends_with(".appimage") {
                return Some((a.name.clone(), a.browser_download_url.clone()));
            }
        }
        // Fallback to deb
        for a in assets {
            let lower = a.name.to_lowercase();
            if lower.ends_with(".deb") {
                return Some((a.name.clone(), a.browser_download_url.clone()));
            }
        }
    }

    // Default: first asset if any
    assets.first().map(|a| (a.name.clone(), a.browser_download_url.clone()))
}

pub async fn check_app_update(_app: AppHandle) -> Result<AppUpdateInfo, String> {
    let current_version = env!("CARGO_PKG_VERSION").to_string();

    let client = reqwest::Client::builder()
        .user_agent("DAWN-Launcher-Updater/1.0")
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    // Try GitHub API
    let response = client.get(LAUNCHER_REPO_API).send().await;

    let release = match response {
        Ok(res) if res.status().is_success() => {
            res.json::<GitHubReleaseResponse>().await.map_err(|e| format!("Failed to parse release JSON: {}", e))?
        }
        _ => {
            // Fallback to redirect inspection
            let redirect_client = reqwest::Client::builder()
                .user_agent("Mozilla/5.0")
                .redirect(reqwest::redirect::Policy::none())
                .build()
                .map_err(|e| format!("Redirect client error: {}", e))?;

            let head = redirect_client.get(LAUNCHER_REPO_REDIRECT).send().await
                .map_err(|e| format!("Failed to reach repository: {}", e))?;

            if let Some(loc) = head.headers().get("location") {
                let loc_str = loc.to_str().map_err(|_| "Invalid redirect location")?;
                let tag = loc_str.rsplit('/').next().unwrap_or("").trim_start_matches(['v', 'V']).to_string();
                GitHubReleaseResponse {
                    tag_name: tag,
                    body: None,
                    html_url: Some(loc_str.to_string()),
                    assets: vec![],
                }
            } else {
                return Err("Failed to query update endpoint".to_string());
            }
        }
    };

    let latest_version = release.tag_name.trim().trim_start_matches(['v', 'V']).to_string();
    let update_available = is_newer_version(&latest_version, &current_version);

    let (asset_name, asset_url) = select_best_asset(&release.assets)
        .map(|(n, u)| (Some(n), Some(u)))
        .unwrap_or((None, None));

    Ok(AppUpdateInfo {
        update_available,
        current_version,
        latest_version,
        release_notes: release.body,
        asset_name,
        asset_url,
        html_url: release.html_url,
    })
}

pub async fn install_app_update(
    app: AppHandle,
    asset_url: String,
    asset_name: String,
) -> Result<(), String> {
    let client = reqwest::Client::builder()
        .user_agent("DAWN-Launcher-Updater/1.0")
        .build()
        .map_err(|e| format!("HTTP client error: {}", e))?;

    let res = client
        .get(&asset_url)
        .send()
        .await
        .map_err(|e| format!("Download request failed: {}", e))?;

    if !res.status().is_success() {
        return Err(format!("Download failed with HTTP status {}", res.status()));
    }

    let total_size = res.content_length().unwrap_or(0);
    let temp_dir = std::env::temp_dir();
    let target_path = temp_dir.join(&asset_name);

    let mut file = tokio::fs::File::create(&target_path)
        .await
        .map_err(|e| format!("Failed to create destination file: {}", e))?;

    let mut stream = res.bytes_stream();
    let mut downloaded: u64 = 0;

    let _ = app.emit("app-update:progress", UpdateProgressPayload {
        percent: 0,
        status: format!("Downloading {}...", asset_name),
    });

    while let Some(chunk_res) = stream.next().await {
        let chunk = chunk_res.map_err(|e| format!("Download stream error: {}", e))?;
        file.write_all(&chunk)
            .await
            .map_err(|e| format!("Write error: {}", e))?;

        downloaded += chunk.len() as u64;
        let percent = if total_size > 0 {
            ((downloaded as f64 / total_size as f64) * 100.0).min(100.0) as u32
        } else {
            50
        };

        let _ = app.emit("app-update:progress", UpdateProgressPayload {
            percent,
            status: format!("Downloading {} ({} MB / {} MB)...",
                asset_name,
                downloaded / (1024 * 1024),
                total_size / (1024 * 1024)
            ),
        });
    }

    file.flush().await.map_err(|e| format!("Flush error: {}", e))?;
    drop(file);

    let _ = app.emit("app-update:progress", UpdateProgressPayload {
        percent: 100,
        status: "Download complete. Starting installer...".to_string(),
    });

    // Spawn installer and exit current process
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        let mut cmd = Command::new(&target_path);
        cmd.creation_flags(0x00000200); // CREATE_NEW_PROCESS_GROUP

        match cmd.spawn() {
            Ok(_) => {
                // Give the installer half a second to launch then exit the old launcher
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                app.exit(0);
                Ok(())
            }
            Err(e) => Err(format!("Failed to spawn installer: {}", e)),
        }
    }

    #[cfg(not(windows))]
    {
        let _ = target_path;
        Err("Automatic installation is not supported on Linux. Please install the downloaded package manually.".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_newer_version() {
        assert!(is_newer_version("1.1.2", "1.1.1"));
        assert!(is_newer_version("v1.2.0", "1.1.9"));
        assert!(is_newer_version("2.0.0", "1.99.99"));
        assert!(!is_newer_version("1.1.1", "1.1.1"));
        assert!(!is_newer_version("1.1.0", "1.1.1"));
        assert!(!is_newer_version("v1.0.5", "1.1.1"));
    }

    #[test]
    fn test_select_best_asset_windows() {
        let assets = vec![
            GitHubReleaseAsset {
                name: "DAWN-v1.1.2-Portable.zip".to_string(),
                browser_download_url: "http://example.com/zip".to_string(),
            },
            GitHubReleaseAsset {
                name: "DAWN-Setup-v1.1.2.exe".to_string(),
                browser_download_url: "http://example.com/setup".to_string(),
            },
            GitHubReleaseAsset {
                name: "DAWN-v1.1.2.msi".to_string(),
                browser_download_url: "http://example.com/msi".to_string(),
            },
        ];

        let selected = select_best_asset(&assets);
        assert!(selected.is_some());
        #[cfg(windows)]
        {
            let (name, url) = selected.unwrap();
            assert_eq!(name, "DAWN-Setup-v1.1.2.exe");
            assert_eq!(url, "http://example.com/setup");
        }
    }
}
