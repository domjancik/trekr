use crate::mapping::MappingEntry;
use crate::pages::AppPageState;
use crate::project::Project;
use crate::ui::TimelineFlow;
use crate::undo::UndoHistory;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PersistedAppState {
    pub project: Project,
    pub page_state: AppPageState,
    pub timeline_flow: TimelineFlow,
    pub mappings: Vec<MappingEntry>,
    pub transport_ticks: u64,
    pub playhead_ticks: u64,
}

impl Default for PersistedAppState {
    fn default() -> Self {
        Self {
            project: Project::demo(),
            page_state: AppPageState::default(),
            timeline_flow: TimelineFlow::DownwardColumns,
            mappings: Vec::new(),
            transport_ticks: 0,
            playhead_ticks: 0,
        }
    }
}

pub fn load(path: &Path) -> Result<PersistedAppState, Box<dyn std::error::Error>> {
    let contents = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&contents)?)
}

pub fn save(path: &Path, state: &PersistedAppState) -> Result<(), Box<dyn std::error::Error>> {
    write_json_atomic(path, state)?;
    Ok(())
}

pub fn undo_history_path(state_path: &Path) -> PathBuf {
    let file_name = state_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("state.json");
    state_path.with_file_name(format!("{file_name}.undo-history.json"))
}

pub fn load_undo_history(path: &Path) -> UndoHistory {
    let Ok(contents) = fs::read_to_string(path) else {
        return UndoHistory::default();
    };

    match serde_json::from_str::<UndoHistory>(&contents) {
        Ok(history) if history.version == UndoHistory::default().version => history,
        Ok(_) => UndoHistory::default(),
        Err(_) => {
            let corrupt_path = corrupt_sibling_path(path);
            let _ = fs::rename(path, corrupt_path);
            UndoHistory::default()
        }
    }
}

pub fn save_undo_history(
    path: &Path,
    history: &UndoHistory,
) -> Result<(), Box<dyn std::error::Error>> {
    write_json_atomic(path, history)?;
    Ok(())
}

fn write_json_atomic<T: Serialize>(
    path: &Path,
    value: &T,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let temp_path = path.with_extension(format!(
        "{}tmp",
        path.extension()
            .and_then(|extension| extension.to_str())
            .map(|extension| format!("{extension}."))
            .unwrap_or_default()
    ));
    let contents = serde_json::to_string_pretty(value)?;
    fs::write(&temp_path, contents)?;
    if path.exists() {
        fs::remove_file(path)?;
    }
    fs::rename(temp_path, path)?;
    Ok(())
}

fn corrupt_sibling_path(path: &Path) -> PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("undo-history.json");
    path.with_file_name(format!("{file_name}.{timestamp}.corrupt"))
}

#[cfg(test)]
mod tests {
    use super::{load_undo_history, save_undo_history, undo_history_path};
    use crate::undo::UndoHistory;
    use std::fs;

    #[test]
    fn undo_history_round_trips_in_separate_file() {
        let base = std::env::temp_dir().join(format!(
            "trekr-undo-history-test-{}.json",
            std::process::id()
        ));
        let undo_path = undo_history_path(&base);
        let mut history = UndoHistory::default();
        history.max_entries = 42;

        save_undo_history(&undo_path, &history).expect("save history");
        let loaded = load_undo_history(&undo_path);
        assert_eq!(loaded.max_entries, 42);

        let _ = fs::remove_file(&undo_path);
    }
}
