use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct LauncherState {
    pub repo_url: Option<String>,
    pub installs: Vec<InstalledBuild>,
    pub last_selected_branch: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledBuild {
    pub branch: String,
    pub commit: String,
    pub source_dir: PathBuf,
    pub binary_path: PathBuf,
    pub installed_at_unix_secs: u64,
}
