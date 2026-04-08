use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LauncherState {
    pub repo_url: Option<String>,
    pub installs: Vec<InstalledBuild>,
    pub tracked_branches: Vec<String>,
    pub last_selected_branch: Option<String>,
    pub default_window_mode: String,
    pub default_state_mode: String,
    pub default_project_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledBuild {
    pub branch: String,
    pub commit: String,
    pub source_dir: PathBuf,
    pub binary_path: PathBuf,
    pub installed_at_unix_secs: u64,
}

impl Default for LauncherState {
    fn default() -> Self {
        Self {
            repo_url: None,
            installs: Vec::new(),
            tracked_branches: vec!["main".to_string()],
            last_selected_branch: Some("main".to_string()),
            default_window_mode: "windowed".to_string(),
            default_state_mode: "persisted".to_string(),
            default_project_path: None,
        }
    }
}
