use crate::types::{BaseDepotInfo, InstallerConstants, LanguageInfo};

pub const STEAM_APP_ID: u64 = 1085660;
pub const GAME_EXECUTABLE: &str = "destiny2.exe";

/// Expected Destiny 2 build for Dawn: Build 86657 (Season of Arrivals, Aug 2020)
pub const EXPECTED_BUILD_ID: u32 = 86657;
pub const EXPECTED_EXE_SIZE: u64 = 122_984_224; // Exact size of build 86657 destiny2.exe in manifest 7180122903232116872
pub const EXPECTED_BUILD_STRING: &str = "86657";
pub const EXPECTED_FILE_VERSION: &str = "86657.20.08.23.1800.d2_rc";

pub const BASE_DEPOT_ID: u64 = 1085661;
pub const BASE_MANIFEST_ID: &str = "7180122903232116872";

pub const REQUIRED_FREE_BYTES: u64 = 110 * 1024 * 1024 * 1024; // 110 GiB

pub const DEPOT_DOWNLOADER_VERSION: &str = "3.4.0";

#[cfg(windows)]
pub const DEPOT_DOWNLOADER_URL: &str = "https://github.com/SteamRE/DepotDownloader/releases/download/DepotDownloader_3.4.0/DepotDownloader-windows-x64.zip";
#[cfg(not(windows))]
pub const DEPOT_DOWNLOADER_URL: &str = "https://github.com/SteamRE/DepotDownloader/releases/download/DepotDownloader_3.4.0/DepotDownloader-linux-x64.zip";

#[cfg(windows)]
pub const DEPOT_DOWNLOADER_SHA256: &str = "41c9e9f0df54b3ad02e67a11726756e5c73283bd7c2e1b04acfa5ae4c2ed3767";
#[cfg(not(windows))]
pub const DEPOT_DOWNLOADER_SHA256: &str = "";

#[cfg(windows)]
pub const DEPOT_DOWNLOADER_EXE: &str = "DepotDownloader.exe";
#[cfg(not(windows))]
pub const DEPOT_DOWNLOADER_EXE: &str = "DepotDownloader";

#[cfg(windows)]
pub const DEPOT_DOWNLOADER_ZIP: &str = "DepotDownloader-windows-x64.zip";
#[cfg(not(windows))]
pub const DEPOT_DOWNLOADER_ZIP: &str = "DepotDownloader-linux-x64.zip";

pub struct StaticLang {
    pub name: &'static str,
    pub code: &'static str,
    pub depot_id: u64,
    pub manifest_id: &'static str,
}

pub const STATIC_LANGUAGES: &[StaticLang] = &[
    StaticLang { name: "English", code: "english", depot_id: 1085662, manifest_id: "2210332166360342287" },
    StaticLang { name: "French", code: "french", depot_id: 1085663, manifest_id: "2934940253687559290" },
    StaticLang { name: "German", code: "german", depot_id: 1085664, manifest_id: "2207989571290186153" },
    StaticLang { name: "Italian", code: "italian", depot_id: 1085665, manifest_id: "6668232053215128229" },
    StaticLang { name: "Japanese", code: "japanese", depot_id: 1085666, manifest_id: "7430022397683116838" },
    StaticLang { name: "Portuguese (Brazil)", code: "brazilian", depot_id: 1085667, manifest_id: "9037238175838085860" },
    StaticLang { name: "Spanish (Spain)", code: "spanish", depot_id: 1085668, manifest_id: "3424833900894552134" },
    StaticLang { name: "Russian", code: "russian", depot_id: 1085669, manifest_id: "4539277942371480381" },
    StaticLang { name: "Polish", code: "polish", depot_id: 1085670, manifest_id: "6407581507105256731" },
    StaticLang { name: "Chinese (Simplified)", code: "schinese", depot_id: 1085671, manifest_id: "4397663774546719308" },
    StaticLang { name: "Chinese (Traditional)", code: "tchinese", depot_id: 1085672, manifest_id: "3906738704604711877" },
    StaticLang { name: "Spanish (Latin America)", code: "latam", depot_id: 1085673, manifest_id: "4773170998099699561" },
    StaticLang { name: "Korean", code: "koreana", depot_id: 1085674, manifest_id: "7148196199569436690" },
];

pub fn get_constants() -> InstallerConstants {
    InstallerConstants {
        languages: STATIC_LANGUAGES
            .iter()
            .map(|l| LanguageInfo {
                name: l.name.to_string(),
                code: l.code.to_string(),
                depot_id: l.depot_id,
                manifest_id: l.manifest_id.to_string(),
            })
            .collect(),
        required_free_bytes: REQUIRED_FREE_BYTES,
        base_depot: BaseDepotInfo {
            depot_id: BASE_DEPOT_ID,
            manifest_id: BASE_MANIFEST_ID.to_string(),
        },
    }
}
