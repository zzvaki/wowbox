use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameInstallation {
    pub id: String,
    pub flavor: String,
    pub label: String,
    pub product_folder: String,
    pub path: String,
    pub addons_path: String,
    pub addon_count: usize,
    pub available: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddonInfo {
    pub id: String,
    pub title: String,
    pub notes: String,
    pub author: String,
    pub version: String,
    pub interface_version: String,
    pub source: String,
    pub source_id: Option<String>,
    pub folder_name: String,
    pub folders: Vec<String>,
    pub path: String,
    pub status: String,
    pub latest_version: Option<String>,
    pub latest_file_id: Option<String>,
    pub latest_download_url: Option<String>,
    pub website_url: Option<String>,
    pub error: Option<String>,
    pub modified_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCheckResult {
    pub addon_id: String,
    pub status: String,
    pub latest_version: Option<String>,
    pub latest_file_id: Option<String>,
    pub download_url: Option<String>,
    pub website_url: Option<String>,
    pub error: Option<String>,
}

impl UpdateCheckResult {
    pub fn untracked(addon_id: String) -> Self {
        Self {
            addon_id,
            status: "untracked".into(),
            latest_version: None,
            latest_file_id: None,
            download_url: None,
            website_url: None,
            error: None,
        }
    }

    pub fn error(addon_id: String, error: impl Into<String>) -> Self {
        Self {
            addon_id,
            status: "error".into(),
            latest_version: None,
            latest_file_id: None,
            download_url: None,
            website_url: None,
            error: Some(error.into()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRequest {
    pub addon: AddonInfo,
    pub download_url: String,
    pub api_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateResult {
    pub addon_id: String,
    pub version: String,
    pub backup_path: String,
    pub installed_folders: Vec<String>,
}
