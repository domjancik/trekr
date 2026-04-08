use crate::launcher::models::LauncherState;
use std::fs;
use std::path::{Path, PathBuf};

pub const DEFAULT_STATE_PATH: &str = "artifacts/state/launcher-state.json";

pub fn default_state_path() -> PathBuf {
    PathBuf::from(DEFAULT_STATE_PATH)
}

pub fn load(path: &Path) -> Result<LauncherState, Box<dyn std::error::Error>> {
    let contents = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&contents)?)
}

pub fn save(path: &Path, state: &LauncherState) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let contents = serde_json::to_string_pretty(state)?;
    fs::write(path, contents)?;
    Ok(())
}
