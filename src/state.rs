use crate::mapping::MappingEntry;
use crate::pages::AppPageState;
use crate::project::Project;
use crate::ui::TimelineFlow;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedAppState {
    pub project: Project,
    pub page_state: AppPageState,
    pub timeline_flow: TimelineFlow,
    pub mappings: Vec<MappingEntry>,
}

pub fn load(path: &Path) -> Result<PersistedAppState, Box<dyn std::error::Error>> {
    let contents = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&contents)?)
}

pub fn save(path: &Path, state: &PersistedAppState) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let contents = serde_json::to_string_pretty(state)?;
    fs::write(path, contents)?;
    Ok(())
}
