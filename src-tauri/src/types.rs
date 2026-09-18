use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageInfo {
    pub name: String,
    pub code: String,
    pub depot_id: u64,
    pub manifest_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseDepotInfo {
    pub depot_id: u64,
    pub manifest_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallerConstants {
    pub languages: Vec<LanguageInfo>,
    pub required_free_bytes: u64,
    pub base_depot: BaseDepotInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreflightResult {
    pub valid: bool,
    pub path: Option<String>,
    pub exists: bool,
    pub writeable: bool,
    pub has_game: bool,
    pub has_dawn: bool,
    pub dawn_version: Option<String>,
    pub free_bytes: u64,
    pub free_gb: f64,
    pub required_bytes: u64,
    pub required_gb: u64,
    pub has_enough_space: bool,
    pub warning: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderValidationResult {
    pub valid: bool,
    pub has_packages: bool,
    pub exe_path: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressPayload {
    pub percent: u32,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SteamGuardPayload {
    #[serde(rename = "type")]
    pub guard_type: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResult {
    pub success: bool,
    pub message: Option<String>,
    pub error: Option<String>,
    pub cancelled: Option<bool>,
    pub count: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthErrorPayload {
    pub error_type: String,
    pub message: String,
}
