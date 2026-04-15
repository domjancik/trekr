use crate::mapping::MappingEntry;
use crate::pages::AppPageState;
use crate::project::Project;
use crate::ui::TimelineFlow;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const VERSION_RETENTION_LIMIT: usize = 100;

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

pub fn serialize(state: &PersistedAppState) -> Result<String, Box<dyn std::error::Error>> {
    Ok(serde_json::to_string_pretty(state)?)
}

pub fn save(path: &Path, state: &PersistedAppState) -> Result<(), Box<dyn std::error::Error>> {
    let contents = serialize(state)?;
    write_atomic(path, &contents)?;
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaveResult {
    pub serialized_state: String,
    pub version_path: Option<PathBuf>,
}

pub fn save_with_version(
    path: &Path,
    state: &PersistedAppState,
    previous_serialized_state: Option<&str>,
) -> Result<SaveResult, Box<dyn std::error::Error>> {
    let contents = serialize(state)?;
    save_serialized_with_version(path, &contents, previous_serialized_state)
}

pub fn save_serialized_with_version(
    path: &Path,
    contents: &str,
    previous_serialized_state: Option<&str>,
) -> Result<SaveResult, Box<dyn std::error::Error>> {
    write_atomic(path, contents)?;

    let version_path = if previous_serialized_state == Some(contents) {
        None
    } else {
        let path = write_version_snapshot(path, contents)?;
        prune_old_versions(path.parent().unwrap_or_else(|| Path::new(".")))?;
        Some(path)
    };

    Ok(SaveResult {
        serialized_state: contents.to_string(),
        version_path,
    })
}

pub fn load_latest_recoverable_version(
    working_file: &Path,
) -> Result<Option<(PersistedAppState, PathBuf)>, Box<dyn std::error::Error>> {
    let versions_dir = versions_dir_for(working_file);
    if !versions_dir.exists() {
        return Ok(None);
    }

    let mut entries = fs::read_dir(&versions_dir)?
        .filter_map(Result::ok)
        .filter(|entry| {
            entry
                .file_type()
                .map(|kind| kind.is_file())
                .unwrap_or(false)
        })
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .and_then(|value| value.to_str())
                .is_some_and(|value| value.eq_ignore_ascii_case("json"))
        })
        .collect::<Vec<_>>();
    entries.sort();
    entries.reverse();

    for path in entries {
        if let Ok(state) = load(&path) {
            return Ok(Some((state, path)));
        }
    }

    Ok(None)
}

pub fn versions_dir_for(path: &Path) -> PathBuf {
    let file_stem = path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("state");
    let mut directory = path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf();
    directory.push(format!("{file_stem}.versions"));
    directory
}

fn write_version_snapshot(
    path: &Path,
    contents: &str,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let versions_dir = versions_dir_for(path);
    fs::create_dir_all(&versions_dir)?;

    let timestamp = timestamp_filename()?;
    let mut version_path = versions_dir.join(format!("{timestamp}.json"));
    let mut suffix = 1_usize;
    while version_path.exists() {
        version_path = versions_dir.join(format!("{timestamp}-{suffix:02}.json"));
        suffix += 1;
    }

    write_atomic(&version_path, contents)?;
    Ok(version_path)
}

fn prune_old_versions(versions_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let mut entries = fs::read_dir(versions_dir)?
        .filter_map(Result::ok)
        .filter(|entry| {
            entry
                .file_type()
                .map(|kind| kind.is_file())
                .unwrap_or(false)
        })
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .and_then(|value| value.to_str())
                .is_some_and(|value| value.eq_ignore_ascii_case("json"))
        })
        .collect::<Vec<_>>();
    if entries.len() <= VERSION_RETENTION_LIMIT {
        return Ok(());
    }

    entries.sort();
    let overflow = entries.len().saturating_sub(VERSION_RETENTION_LIMIT);
    for path in entries.into_iter().take(overflow) {
        fs::remove_file(path)?;
    }
    Ok(())
}

fn write_atomic(path: &Path, contents: &str) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let temp_path = temp_path_for(path)?;
    fs::write(&temp_path, contents)?;
    match fs::rename(&temp_path, path) {
        Ok(()) => Ok(()),
        Err(rename_error) => {
            if path.exists() {
                fs::remove_file(path)?;
                fs::rename(&temp_path, path)?;
                Ok(())
            } else {
                Err(Box::new(rename_error))
            }
        }
    }
}

fn temp_path_for(path: &Path) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "state path is missing a file name",
            )
        })?;
    let unique = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    Ok(path.with_file_name(format!(".{file_name}.{unique}.tmp")))
}

fn timestamp_filename() -> Result<String, Box<dyn std::error::Error>> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?;
    let total_seconds = now.as_secs() as i64;
    let milliseconds = now.subsec_millis();
    let days = total_seconds.div_euclid(86_400);
    let seconds_of_day = total_seconds.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    let hour = seconds_of_day / 3_600;
    let minute = (seconds_of_day % 3_600) / 60;
    let second = seconds_of_day % 60;
    Ok(format!(
        "{year:04}-{month:02}-{day:02}T{hour:02}-{minute:02}-{second:02}-{milliseconds:03}Z"
    ))
}

fn civil_from_days(days_since_epoch: i64) -> (i32, u32, u32) {
    let z = days_since_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    let year = y + i64::from(month <= 2);
    (year as i32, month as u32, day as u32)
}

#[cfg(test)]
mod tests {
    use super::{
        PersistedAppState, load_latest_recoverable_version, save_serialized_with_version,
        versions_dir_for,
    };
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_temp_dir() -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before epoch")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("trekr-state-tests-{unique}"));
        fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    #[test]
    fn save_with_version_skips_duplicate_snapshot_when_contents_match_previous() {
        let dir = unique_temp_dir();
        let path = dir.join("last-run.json");
        let state = PersistedAppState::default();
        let first = serde_json::to_string_pretty(&state).expect("serialize");

        let first_result =
            save_serialized_with_version(&path, &first, None).expect("initial save succeeds");
        assert!(first_result.version_path.is_some());

        let second_result = save_serialized_with_version(&path, &first, Some(&first))
            .expect("duplicate save succeeds");
        assert!(second_result.version_path.is_none());
    }

    #[test]
    fn recovery_loads_latest_valid_version_when_present() {
        let dir = unique_temp_dir();
        let working_file = dir.join("last-run.json");
        let versions_dir = versions_dir_for(&working_file);
        fs::create_dir_all(&versions_dir).expect("create versions dir");

        fs::write(
            versions_dir.join("2026-01-01T00-00-00-000Z.json"),
            "{invalid",
        )
        .expect("write invalid version");
        let state = PersistedAppState::default();
        fs::write(
            versions_dir.join("2026-01-02T00-00-00-000Z.json"),
            serde_json::to_string_pretty(&state).expect("serialize"),
        )
        .expect("write valid version");

        let recovered = load_latest_recoverable_version(&working_file)
            .expect("recovery succeeds")
            .expect("version exists");
        assert_eq!(
            serde_json::to_string_pretty(&recovered.0).expect("serialize recovered"),
            serde_json::to_string_pretty(&state).expect("serialize state")
        );
        assert!(
            recovered
                .1
                .file_name()
                .and_then(|value| value.to_str())
                .is_some_and(|value| value.contains("2026-01-02"))
        );
    }
}
