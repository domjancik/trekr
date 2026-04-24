use crate::actions::AppAction;
use crate::midi_io::{MidiInputEvent, MidiInputMessage};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MappingTarget {
    TransportPlay,
    TransportRecord,
    TrackArm,
    TrackMute,
    LoopSet,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MidiMapping {
    pub target: MappingTarget,
    pub track_index: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MappingSourceKind {
    Key,
    Midi,
    Osc,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MappingEntry {
    pub source_kind: MappingSourceKind,
    #[serde(default = "default_mapping_source_device")]
    pub source_device_label: String,
    pub source_label: String,
    pub target_label: String,
    pub scope_label: String,
    pub enabled: bool,
}

impl MappingEntry {
    pub fn default_new() -> Self {
        Self {
            source_kind: MappingSourceKind::Key,
            source_device_label: default_mapping_source_device(),
            source_label: default_source_label(MappingSourceKind::Key).to_string(),
            target_label: "Play/Stop".to_string(),
            scope_label: default_scope_label("Play/Stop", 0),
            enabled: false,
        }
    }
}

const KEY_SOURCE_OPTIONS: &[&str] = &[
    "Space",
    "R",
    "Shift+R",
    "C",
    "Shift+C",
    "G",
    "L",
    "A",
    "M",
    "S",
    "I",
    "[ ]",
    ", .",
    "- =",
    "/ \\",
    "Left/Right",
    "T Shift+T",
    "V J K",
    "U O H P Y B",
    "Z X D F",
    "Numpad1",
    "Numpad2",
    "Numpad3",
    "Numpad4",
    "Numpad5",
    "Numpad6",
    "Numpad7",
    "Numpad8",
    "Alt+1",
    "Alt+2",
    "Alt+3",
    "Alt+4",
    "Alt+5",
    "Alt+6",
    "Alt+7",
    "Alt+8",
    "Ctrl+Z",
    "Ctrl+Y",
    "Ctrl+Shift+Z",
    "Alt+Z",
    "Alt+Shift+Z",
    "Alt+X",
    "Alt+Shift+X",
    "Alt+C",
    "Alt+Shift+C",
    "Tab/F1-F6",
];

const MIDI_SOURCE_OPTIONS: &[&str] =
    &["Note C2", "Note D2", "CC20", "CC21", "CC22", "CC23", "CC24"];

const OSC_SOURCE_OPTIONS: &[&str] = &[
    "/transport/play",
    "/transport/record",
    "/track/active/arm",
    "/track/active/mute",
    "/track/active/loop",
];

const TARGET_OPTIONS: &[&str] = &[
    "Play/Stop",
    "Record",
    "Record Mode",
    "Loop Recording Wrap",
    "Song Loop",
    "Reset Song Loop",
    "Track Loop",
    "Recall Stored Loop Slot 1",
    "Recall Stored Loop Slot 2",
    "Recall Stored Loop Slot 3",
    "Recall Stored Loop Slot 4",
    "Recall Stored Loop Slot 5",
    "Recall Stored Loop Slot 6",
    "Recall Stored Loop Slot 7",
    "Recall Stored Loop Slot 8",
    "Store Current Loop To Slot 1",
    "Store Current Loop To Slot 2",
    "Store Current Loop To Slot 3",
    "Store Current Loop To Slot 4",
    "Store Current Loop To Slot 5",
    "Store Current Loop To Slot 6",
    "Store Current Loop To Slot 7",
    "Store Current Loop To Slot 8",
    "Clear Stored Loop Slot 1",
    "Clear Stored Loop Slot 2",
    "Clear Stored Loop Slot 3",
    "Clear Stored Loop Slot 4",
    "Clear Stored Loop Slot 5",
    "Clear Stored Loop Slot 6",
    "Clear Stored Loop Slot 7",
    "Clear Stored Loop Slot 8",
    "Clear Track",
    "Clear All",
    "Track Arm",
    "Track Mute",
    "Track Solo",
    "Passthrough",
    "Recording View",
    "Select Next Recording Clip",
    "Select Previous Recording Clip",
    "Recording Clip Mute",
    "Delete Recording Clip",
    "Focused Track View",
    "Select Track",
    "Select Notes At Playhead",
    "Select Notes At Playhead Add",
    "Deselect Track Notes",
    "Select Next Note",
    "Select Previous Note",
    "Focus First Selected Note",
    "Focus Last Selected Note",
    "Extend Note Selection Forward",
    "Extend Note Selection Backward",
    "Extend Note Selection Both",
    "Contract Note Selection",
    "Nudge Selected Notes Earlier",
    "Nudge Selected Notes Later",
    "Nudge Selected Notes Up",
    "Nudge Selected Notes Down",
    "Previous Page Item",
    "Next Page Item",
    "Adjust Page Item Backward",
    "Adjust Page Item Forward",
    "Activate Page Item",
    "Cancel",
    "Mappings Write Mode",
    "Add Mapping",
    "Remove Mapping",
    "Previous Mapping Field",
    "Next Mapping Field",
    "Undo",
    "Redo",
    "Undo Timeline",
    "Redo Timeline",
    "Undo Mappings",
    "Redo Mappings",
    "Undo UI",
    "Redo UI",
    "Pages/Overlay",
    "Link Enable",
    "Link Start/Stop",
];

const SCOPE_OPTIONS: &[&str] = &[
    "Global",
    "Active Track",
    "Armed/Active",
    "Relative",
    "Absolute",
];

pub fn default_mapping_source_device() -> String {
    "Any MIDI".to_string()
}

pub fn cycle_mapping_source_kind(current: MappingSourceKind, delta: i32) -> MappingSourceKind {
    let options = [
        MappingSourceKind::Key,
        MappingSourceKind::Midi,
        MappingSourceKind::Osc,
    ];
    let current_index = options
        .iter()
        .position(|candidate| *candidate == current)
        .unwrap_or(0) as i32;
    options[(current_index + delta).rem_euclid(options.len() as i32) as usize]
}

pub fn default_source_label(kind: MappingSourceKind) -> &'static str {
    source_options(kind).first().copied().unwrap_or("Space")
}

pub fn cycle_mapping_source_label(
    kind: MappingSourceKind,
    current: &str,
    delta: i32,
) -> &'static str {
    cycle_label(source_options(kind), current, delta)
}

pub fn cycle_mapping_target_label(current: &str, delta: i32) -> &'static str {
    cycle_label(TARGET_OPTIONS, current, delta)
}

pub fn mapping_target_labels() -> &'static [&'static str] {
    TARGET_OPTIONS
}

pub fn cycle_mapping_scope_label(current: &str, delta: i32) -> &'static str {
    cycle_label(SCOPE_OPTIONS, current, delta)
}

pub fn default_scope_label(target_label: &str, track_count: usize) -> String {
    scope_options_for_target(target_label, track_count)
        .first()
        .cloned()
        .unwrap_or_else(|| "Global".to_string())
}

pub fn mapping_scope_valid_for_target(
    target_label: &str,
    scope_label: &str,
    track_count: usize,
) -> bool {
    scope_options_for_target(target_label, track_count)
        .iter()
        .any(|candidate| candidate == scope_label)
}

pub fn cycle_mapping_scope_value(
    current: &str,
    delta: i32,
    target_label: &str,
    track_count: usize,
) -> String {
    let options = scope_options_for_target(target_label, track_count);
    let current_index = options
        .iter()
        .position(|candidate| candidate == current)
        .unwrap_or(0) as i32;
    options[(current_index + delta).rem_euclid(options.len() as i32) as usize].clone()
}

pub fn cycle_mapping_source_device_label(current: &str, devices: &[String], delta: i32) -> String {
    let mut options = vec![default_mapping_source_device()];
    for device in devices {
        if !options.iter().any(|candidate| candidate == device) {
            options.push(device.clone());
        }
    }

    let current_index = options
        .iter()
        .position(|candidate| candidate == current)
        .unwrap_or(0) as i32;
    options[(current_index + delta).rem_euclid(options.len() as i32) as usize].clone()
}

fn source_options(kind: MappingSourceKind) -> &'static [&'static str] {
    match kind {
        MappingSourceKind::Key => KEY_SOURCE_OPTIONS,
        MappingSourceKind::Midi => MIDI_SOURCE_OPTIONS,
        MappingSourceKind::Osc => OSC_SOURCE_OPTIONS,
    }
}

fn cycle_label<'a>(options: &'a [&'a str], current: &str, delta: i32) -> &'a str {
    let current_index = options
        .iter()
        .position(|candidate| *candidate == current)
        .unwrap_or(0) as i32;
    options[(current_index + delta).rem_euclid(options.len() as i32) as usize]
}

pub fn search_mapping_targets(query: &str) -> Vec<&'static str> {
    let trimmed = query.trim();
    let mut scored = TARGET_OPTIONS
        .iter()
        .filter_map(|label| target_query_score(label, trimmed).map(|score| (*label, score)))
        .collect::<Vec<_>>();

    scored.sort_by(|(left_label, left_score), (right_label, right_score)| {
        left_score
            .cmp(right_score)
            .then_with(|| left_label.len().cmp(&right_label.len()))
            .then_with(|| left_label.cmp(right_label))
    });
    scored.into_iter().map(|(label, _)| label).collect()
}

fn target_query_score(label: &str, query: &str) -> Option<(u8, usize)> {
    if query.is_empty() {
        return Some((4, label.len()));
    }

    let label_lower = label.to_ascii_lowercase();
    let query_lower = query.to_ascii_lowercase();

    if label_lower == query_lower {
        return Some((0, 0));
    }
    if let Some(index) = label_lower.find(&query_lower) {
        let boundary = index == 0
            || label_lower
                .as_bytes()
                .get(index.saturating_sub(1))
                .is_some_and(|byte| *byte == b' ' || *byte == b'/' || *byte == b'(');
        return Some((if boundary { 1 } else { 2 }, index));
    }

    fuzzy_match_distance(&label_lower, &query_lower).map(|distance| (3, distance))
}

fn fuzzy_match_distance(label: &str, query: &str) -> Option<usize> {
    let mut search_start = 0_usize;
    let mut first_index = None;
    let mut last_index = 0_usize;

    for query_char in query.chars() {
        let next = label[search_start..]
            .char_indices()
            .find(|(_, label_char)| *label_char == query_char)?;
        let absolute_index = search_start + next.0;
        first_index.get_or_insert(absolute_index);
        last_index = absolute_index;
        search_start = absolute_index + next.1.len_utf8();
    }

    Some(last_index.saturating_sub(first_index.unwrap_or(0)))
}

fn scope_options_for_target(target_label: &str, track_count: usize) -> Vec<String> {
    match target_label {
        "Previous Page Item"
        | "Next Page Item"
        | "Adjust Page Item Backward"
        | "Adjust Page Item Forward"
        | "Activate Page Item"
        | "Cancel"
        | "Mappings Write Mode"
        | "Add Mapping"
        | "Remove Mapping"
        | "Previous Mapping Field"
        | "Next Mapping Field"
        | "Play/Stop"
        | "Record Mode"
        | "Loop Recording Wrap"
        | "Song Loop"
        | "Set Song Loop"
        | "Reset Song Loop"
        | "Clear All"
        | "Undo"
        | "Redo"
        | "Undo Timeline"
        | "Redo Timeline"
        | "Undo Mappings"
        | "Redo Mappings"
        | "Undo UI"
        | "Redo UI"
        | "Pages/Overlay"
        | "Link Enable"
        | "Link Start/Stop"
        | "Focused Track View" => vec!["Global".to_string()],
        "Record" | "Record Hold" => {
            let mut options = vec!["Armed/Active".to_string(), "Active Track".to_string()];
            options.extend(absolute_track_scopes(track_count));
            options
        }
        "Select Track" => {
            let mut options = vec!["Relative".to_string()];
            options.extend(absolute_track_scopes(track_count));
            options
        }
        "Select Notes At Playhead"
        | "Select Notes At Playhead Add"
        | "Deselect Track Notes"
        | "Select Next Note"
        | "Select Previous Note"
        | "Focus First Selected Note"
        | "Focus Last Selected Note"
        | "Extend Note Selection Forward"
        | "Extend Note Selection Backward"
        | "Extend Note Selection Both"
        | "Contract Note Selection"
        | "Nudge Selected Notes Earlier"
        | "Nudge Selected Notes Later"
        | "Nudge Selected Notes Up"
        | "Nudge Selected Notes Down" => {
            let mut options = vec!["Active Track".to_string()];
            options.extend(absolute_track_scopes(track_count));
            options
        }
        "Recording View"
        | "Select Next Recording Clip"
        | "Select Previous Recording Clip"
        | "Recording Clip Mute"
        | "Delete Recording Clip" => {
            let mut options = vec!["Active Track".to_string()];
            options.extend(absolute_track_scopes(track_count));
            options
        }
        "Track Loop"
        | "Set Track Loop"
        | "Recall Stored Loop Slot 1"
        | "Recall Stored Loop Slot 2"
        | "Recall Stored Loop Slot 3"
        | "Recall Stored Loop Slot 4"
        | "Recall Stored Loop Slot 5"
        | "Recall Stored Loop Slot 6"
        | "Recall Stored Loop Slot 7"
        | "Recall Stored Loop Slot 8"
        | "Store Current Loop To Slot 1"
        | "Store Current Loop To Slot 2"
        | "Store Current Loop To Slot 3"
        | "Store Current Loop To Slot 4"
        | "Store Current Loop To Slot 5"
        | "Store Current Loop To Slot 6"
        | "Store Current Loop To Slot 7"
        | "Store Current Loop To Slot 8"
        | "Clear Stored Loop Slot 1"
        | "Clear Stored Loop Slot 2"
        | "Clear Stored Loop Slot 3"
        | "Clear Stored Loop Slot 4"
        | "Clear Stored Loop Slot 5"
        | "Clear Stored Loop Slot 6"
        | "Clear Stored Loop Slot 7"
        | "Clear Stored Loop Slot 8"
        | "Clear Track"
        | "Track Arm"
        | "Track Mute"
        | "Track Solo"
        | "Passthrough" => {
            let mut options = vec!["Active Track".to_string()];
            options.extend(absolute_track_scopes(track_count));
            options
        }
        _ => SCOPE_OPTIONS
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
    }
}

fn absolute_track_scopes(track_count: usize) -> Vec<String> {
    (0..track_count.max(1))
        .map(|index| format!("Track {}", index + 1))
        .collect()
}

pub fn demo_mappings() -> Vec<MappingEntry> {
    vec![
        entry(MappingSourceKind::Key, "Space", "Play/Stop", "Global", true),
        entry(
            MappingSourceKind::Key,
            "Shift+R",
            "Record Mode",
            "Global",
            true,
        ),
        entry(MappingSourceKind::Key, "R", "Record", "Armed/Active", true),
        entry(
            MappingSourceKind::Key,
            "C",
            "Clear Track",
            "Active Track",
            true,
        ),
        entry(
            MappingSourceKind::Key,
            "Shift+C",
            "Clear All",
            "Global",
            true,
        ),
        entry(MappingSourceKind::Key, "G", "Song Loop", "Global", true),
        entry(
            MappingSourceKind::Key,
            "L",
            "Track Loop",
            "Active Track",
            true,
        ),
        midi_entry("Any MIDI", "CC20", "Track Arm", "Active Track", true),
        entry(
            MappingSourceKind::Key,
            "A",
            "Track Arm",
            "Active Track",
            true,
        ),
        entry(
            MappingSourceKind::Key,
            "M",
            "Track Mute",
            "Active Track",
            true,
        ),
        entry(
            MappingSourceKind::Key,
            "S",
            "Track Solo",
            "Active Track",
            true,
        ),
        entry(
            MappingSourceKind::Key,
            "I",
            "Passthrough",
            "Active Track",
            true,
        ),
        entry(
            MappingSourceKind::Key,
            "[ ]",
            "Set Track Loop",
            "Active Track",
            true,
        ),
        entry(
            MappingSourceKind::Key,
            "Shift+[ ]",
            "Set Song Loop",
            "Global",
            true,
        ),
        entry(
            MappingSourceKind::Key,
            ", .",
            "Nudge Loop",
            "Active Track",
            true,
        ),
        entry(
            MappingSourceKind::Key,
            "Shift+, .",
            "Nudge Song Loop",
            "Global",
            true,
        ),
        entry(
            MappingSourceKind::Key,
            "- =",
            "Resize Loop",
            "Active Track",
            true,
        ),
        entry(
            MappingSourceKind::Key,
            "/ \\",
            "Half/Double Loop",
            "Active Track",
            true,
        ),
        entry(
            MappingSourceKind::Key,
            "Left/Right",
            "Select Track",
            "Relative",
            true,
        ),
        entry(
            MappingSourceKind::Key,
            "T Shift+T",
            "Select Notes At Playhead",
            "Active Track",
            true,
        ),
        entry(
            MappingSourceKind::Key,
            "V J K",
            "Select Next Note",
            "Active Track",
            true,
        ),
        entry(
            MappingSourceKind::Key,
            "U O H P Y B",
            "Extend Note Selection Forward",
            "Active Track",
            true,
        ),
        entry(
            MappingSourceKind::Key,
            "Z X D F",
            "Nudge Selected Notes Up",
            "Active Track",
            true,
        ),
        entry(
            MappingSourceKind::Key,
            "1-9",
            "Select Track",
            "Absolute",
            true,
        ),
        entry(
            MappingSourceKind::Key,
            "Tab/F1-F5",
            "Pages/Overlay",
            "Global",
            true,
        ),
        midi_entry("Any MIDI", "Note C2", "Record Hold", "Global", false),
        midi_entry("Any MIDI", "CC21", "Track Loop", "Active Track", false),
        midi_entry("Any MIDI", "CC22", "Track Mute", "Active Track", false),
        entry(
            MappingSourceKind::Osc,
            "/transport/play",
            "Play/Stop",
            "Global",
            true,
        ),
        entry(
            MappingSourceKind::Osc,
            "/track/active/arm",
            "Track Arm",
            "Active Track",
            false,
        ),
    ]
}

pub fn mapping_entry_to_actions(entry: &MappingEntry, event: &MidiInputEvent) -> Vec<AppAction> {
    let absolute_track_index = parse_absolute_track_scope(&entry.scope_label);
    match entry.target_label.as_str() {
        "Previous Page Item" => vec![AppAction::SelectPreviousPageItem],
        "Next Page Item" => vec![AppAction::SelectNextPageItem],
        "Adjust Page Item Backward" => vec![AppAction::AdjustPageItemBackward],
        "Adjust Page Item Forward" => vec![AppAction::AdjustPageItemForward],
        "Activate Page Item" => vec![AppAction::ActivatePageItem],
        "Cancel" => vec![AppAction::CancelCurrentMode],
        "Mappings Write Mode" => vec![AppAction::ToggleMappingsWriteMode],
        "Add Mapping" => vec![AppAction::AddMappingRow],
        "Remove Mapping" => vec![AppAction::RemoveSelectedMapping],
        "Previous Mapping Field" => vec![AppAction::SelectPreviousPageField],
        "Next Mapping Field" => vec![AppAction::SelectNextPageField],
        "Play/Stop" => vec![AppAction::TogglePlayback],
        "Record" => vec![AppAction::ToggleRecording],
        "Record Hold" => hold_mapping_actions(
            absolute_track_index,
            event,
            AppAction::StartRecording,
            AppAction::StopRecording,
        ),
        "Record Mode" => vec![AppAction::CycleRecordMode],
        "Loop Recording Wrap" => vec![AppAction::ToggleLoopRecordingExtension],
        "Song Loop" | "Set Song Loop" => vec![AppAction::ToggleGlobalLoop],
        "Reset Song Loop" => vec![AppAction::ResetGlobalLoop],
        "Track Loop" | "Set Track Loop" => {
            track_scoped_actions(absolute_track_index, AppAction::ToggleCurrentTrackLoop)
        }
        label if recall_stored_loop_slot_action(label).is_some() => track_scoped_actions(
            absolute_track_index,
            recall_stored_loop_slot_action(label).expect("stored loop recall action checked"),
        ),
        label if store_stored_loop_slot_action(label).is_some() => track_scoped_actions(
            absolute_track_index,
            store_stored_loop_slot_action(label).expect("stored loop store action checked"),
        ),
        label if clear_stored_loop_slot_action(label).is_some() => track_scoped_actions(
            absolute_track_index,
            clear_stored_loop_slot_action(label).expect("stored loop clear action checked"),
        ),
        "Clear Track" => {
            track_scoped_actions(absolute_track_index, AppAction::ClearCurrentTrackContent)
        }
        "Clear All" => vec![AppAction::ClearAllTrackContent],
        "Track Arm" => track_scoped_actions(absolute_track_index, AppAction::ToggleCurrentTrackArm),
        "Track Mute" => {
            track_scoped_actions(absolute_track_index, AppAction::ToggleCurrentTrackMute)
        }
        "Track Solo" => {
            track_scoped_actions(absolute_track_index, AppAction::ToggleCurrentTrackSolo)
        }
        "Passthrough" => track_scoped_actions(
            absolute_track_index,
            AppAction::ToggleCurrentTrackPassthrough,
        ),
        "Recording View" => track_scoped_actions(
            absolute_track_index,
            AppAction::ToggleCurrentTrackRecordingView,
        ),
        "Select Next Recording Clip" => {
            track_scoped_actions(absolute_track_index, AppAction::SelectNextRecordingClip)
        }
        "Select Previous Recording Clip" => {
            track_scoped_actions(absolute_track_index, AppAction::SelectPreviousRecordingClip)
        }
        "Recording Clip Mute" => track_scoped_actions(
            absolute_track_index,
            AppAction::ToggleSelectedRecordingClipMute,
        ),
        "Delete Recording Clip" => {
            track_scoped_actions(absolute_track_index, AppAction::DeleteSelectedRecordingClip)
        }
        "Focused Track View" => vec![AppAction::ToggleFocusedTrackView],
        "Select Track" => absolute_track_index
            .map(AppAction::SelectTrack)
            .or_else(|| match entry.scope_label.as_str() {
                "Relative" => Some(AppAction::SelectNextTrack),
                _ => None,
            })
            .into_iter()
            .collect(),
        "Select Notes At Playhead" => {
            track_scoped_actions(absolute_track_index, AppAction::SelectNotesAtPlayhead)
        }
        "Select Notes At Playhead Add" => hold_mapping_actions(
            absolute_track_index,
            event,
            AppAction::BeginNoteAdditiveSelectionHold,
            AppAction::EndNoteAdditiveSelectionHold,
        )
        .into_iter()
        .chain(
            is_mapping_press_event(event)
                .then(|| {
                    track_scoped_actions(absolute_track_index, AppAction::SelectNotesAtPlayhead)
                })
                .into_iter()
                .flatten(),
        )
        .collect(),
        "Deselect Track Notes" => {
            track_scoped_actions(absolute_track_index, AppAction::DeselectTrackNotes)
        }
        "Select Next Note" => track_scoped_actions(absolute_track_index, AppAction::SelectNextNote),
        "Select Previous Note" => {
            track_scoped_actions(absolute_track_index, AppAction::SelectPreviousNote)
        }
        "Focus First Selected Note" => {
            track_scoped_actions(absolute_track_index, AppAction::FocusFirstSelectedNote)
        }
        "Focus Last Selected Note" => {
            track_scoped_actions(absolute_track_index, AppAction::FocusLastSelectedNote)
        }
        "Extend Note Selection Forward" => {
            track_scoped_actions(absolute_track_index, AppAction::ExtendNoteSelectionForward)
        }
        "Extend Note Selection Backward" => {
            track_scoped_actions(absolute_track_index, AppAction::ExtendNoteSelectionBackward)
        }
        "Extend Note Selection Both" => {
            track_scoped_actions(absolute_track_index, AppAction::ExtendNoteSelectionBoth)
        }
        "Contract Note Selection" => {
            track_scoped_actions(absolute_track_index, AppAction::ContractNoteSelection)
        }
        "Nudge Selected Notes Earlier" => {
            track_scoped_actions(absolute_track_index, AppAction::NudgeSelectedNotesEarlier)
        }
        "Nudge Selected Notes Later" => {
            track_scoped_actions(absolute_track_index, AppAction::NudgeSelectedNotesLater)
        }
        "Nudge Selected Notes Up" => {
            track_scoped_actions(absolute_track_index, AppAction::NudgeSelectedNotesUp)
        }
        "Nudge Selected Notes Down" => {
            track_scoped_actions(absolute_track_index, AppAction::NudgeSelectedNotesDown)
        }
        "Undo" => vec![AppAction::Undo],
        "Redo" => vec![AppAction::Redo],
        "Undo Timeline" => vec![AppAction::UndoTimeline],
        "Redo Timeline" => vec![AppAction::RedoTimeline],
        "Undo Mappings" => vec![AppAction::UndoMappings],
        "Redo Mappings" => vec![AppAction::RedoMappings],
        "Undo UI" => vec![AppAction::UndoUi],
        "Redo UI" => vec![AppAction::RedoUi],
        "Pages/Overlay" => vec![AppAction::ToggleMappingsOverlay],
        "Link Enable" => vec![AppAction::ToggleLinkEnabled],
        "Link Start/Stop" => vec![AppAction::ToggleLinkStartStopSync],
        _ => Vec::new(),
    }
}

pub fn mapping_entry_targets_action(entry: &MappingEntry, action: AppAction) -> bool {
    entry.enabled
        && mapping_entry_possible_actions(entry)
            .into_iter()
            .any(|candidate| candidate == action)
}

pub fn parse_absolute_track_scope(scope_label: &str) -> Option<usize> {
    let scope = scope_label.trim();
    scope
        .strip_prefix("Track ")
        .and_then(|suffix| suffix.parse::<usize>().ok())
        .and_then(|index| index.checked_sub(1))
}

fn track_scoped_actions(
    absolute_track_index: Option<usize>,
    toggle_action: AppAction,
) -> Vec<AppAction> {
    absolute_track_index
        .map(|index| vec![AppAction::SelectTrack(index), toggle_action])
        .unwrap_or_else(|| vec![toggle_action])
}

fn mapping_entry_possible_actions(entry: &MappingEntry) -> Vec<AppAction> {
    let absolute_track_index = parse_absolute_track_scope(&entry.scope_label);
    match entry.target_label.as_str() {
        "Previous Page Item" => vec![AppAction::SelectPreviousPageItem],
        "Next Page Item" => vec![AppAction::SelectNextPageItem],
        "Adjust Page Item Backward" => vec![AppAction::AdjustPageItemBackward],
        "Adjust Page Item Forward" => vec![AppAction::AdjustPageItemForward],
        "Activate Page Item" => vec![AppAction::ActivatePageItem],
        "Cancel" => vec![AppAction::CancelCurrentMode],
        "Mappings Write Mode" => vec![AppAction::ToggleMappingsWriteMode],
        "Add Mapping" => vec![AppAction::AddMappingRow],
        "Remove Mapping" => vec![AppAction::RemoveSelectedMapping],
        "Previous Mapping Field" => vec![AppAction::SelectPreviousPageField],
        "Next Mapping Field" => vec![AppAction::SelectNextPageField],
        "Play/Stop" => vec![AppAction::TogglePlayback],
        "Record" | "Record Hold" => vec![AppAction::ToggleRecording],
        "Record Mode" => vec![AppAction::CycleRecordMode],
        "Loop Recording Wrap" => vec![AppAction::ToggleLoopRecordingExtension],
        "Song Loop" | "Set Song Loop" => vec![AppAction::ToggleGlobalLoop],
        "Reset Song Loop" => vec![AppAction::ResetGlobalLoop],
        "Track Loop" | "Set Track Loop" => {
            track_scoped_actions(absolute_track_index, AppAction::ToggleCurrentTrackLoop)
        }
        label if recall_stored_loop_slot_action(label).is_some() => track_scoped_actions(
            absolute_track_index,
            recall_stored_loop_slot_action(label).expect("stored loop recall action checked"),
        ),
        label if store_stored_loop_slot_action(label).is_some() => track_scoped_actions(
            absolute_track_index,
            store_stored_loop_slot_action(label).expect("stored loop store action checked"),
        ),
        label if clear_stored_loop_slot_action(label).is_some() => track_scoped_actions(
            absolute_track_index,
            clear_stored_loop_slot_action(label).expect("stored loop clear action checked"),
        ),
        "Clear Track" => {
            track_scoped_actions(absolute_track_index, AppAction::ClearCurrentTrackContent)
        }
        "Clear All" => vec![AppAction::ClearAllTrackContent],
        "Track Arm" => track_scoped_actions(absolute_track_index, AppAction::ToggleCurrentTrackArm),
        "Track Mute" => {
            track_scoped_actions(absolute_track_index, AppAction::ToggleCurrentTrackMute)
        }
        "Track Solo" => {
            track_scoped_actions(absolute_track_index, AppAction::ToggleCurrentTrackSolo)
        }
        "Passthrough" => track_scoped_actions(
            absolute_track_index,
            AppAction::ToggleCurrentTrackPassthrough,
        ),
        "Recording View" => track_scoped_actions(
            absolute_track_index,
            AppAction::ToggleCurrentTrackRecordingView,
        ),
        "Select Next Recording Clip" => {
            track_scoped_actions(absolute_track_index, AppAction::SelectNextRecordingClip)
        }
        "Select Previous Recording Clip" => {
            track_scoped_actions(absolute_track_index, AppAction::SelectPreviousRecordingClip)
        }
        "Recording Clip Mute" => track_scoped_actions(
            absolute_track_index,
            AppAction::ToggleSelectedRecordingClipMute,
        ),
        "Delete Recording Clip" => {
            track_scoped_actions(absolute_track_index, AppAction::DeleteSelectedRecordingClip)
        }
        "Focused Track View" => vec![AppAction::ToggleFocusedTrackView],
        "Select Track" => absolute_track_index
            .map(AppAction::SelectTrack)
            .or_else(|| match entry.scope_label.as_str() {
                "Relative" => Some(AppAction::SelectNextTrack),
                _ => None,
            })
            .into_iter()
            .collect(),
        "Select Notes At Playhead" | "Select Notes At Playhead Add" => {
            track_scoped_actions(absolute_track_index, AppAction::SelectNotesAtPlayhead)
        }
        "Deselect Track Notes" => {
            track_scoped_actions(absolute_track_index, AppAction::DeselectTrackNotes)
        }
        "Select Next Note" => track_scoped_actions(absolute_track_index, AppAction::SelectNextNote),
        "Select Previous Note" => {
            track_scoped_actions(absolute_track_index, AppAction::SelectPreviousNote)
        }
        "Focus First Selected Note" => {
            track_scoped_actions(absolute_track_index, AppAction::FocusFirstSelectedNote)
        }
        "Focus Last Selected Note" => {
            track_scoped_actions(absolute_track_index, AppAction::FocusLastSelectedNote)
        }
        "Extend Note Selection Forward" => {
            track_scoped_actions(absolute_track_index, AppAction::ExtendNoteSelectionForward)
        }
        "Extend Note Selection Backward" => {
            track_scoped_actions(absolute_track_index, AppAction::ExtendNoteSelectionBackward)
        }
        "Extend Note Selection Both" => {
            track_scoped_actions(absolute_track_index, AppAction::ExtendNoteSelectionBoth)
        }
        "Contract Note Selection" => {
            track_scoped_actions(absolute_track_index, AppAction::ContractNoteSelection)
        }
        "Nudge Selected Notes Earlier" => {
            track_scoped_actions(absolute_track_index, AppAction::NudgeSelectedNotesEarlier)
        }
        "Nudge Selected Notes Later" => {
            track_scoped_actions(absolute_track_index, AppAction::NudgeSelectedNotesLater)
        }
        "Nudge Selected Notes Up" => {
            track_scoped_actions(absolute_track_index, AppAction::NudgeSelectedNotesUp)
        }
        "Nudge Selected Notes Down" => {
            track_scoped_actions(absolute_track_index, AppAction::NudgeSelectedNotesDown)
        }
        "Undo" => vec![AppAction::Undo],
        "Redo" => vec![AppAction::Redo],
        "Undo Timeline" => vec![AppAction::UndoTimeline],
        "Redo Timeline" => vec![AppAction::RedoTimeline],
        "Undo Mappings" => vec![AppAction::UndoMappings],
        "Redo Mappings" => vec![AppAction::RedoMappings],
        "Undo UI" => vec![AppAction::UndoUi],
        "Redo UI" => vec![AppAction::RedoUi],
        "Pages/Overlay" => vec![AppAction::ToggleMappingsOverlay],
        "Link Enable" => vec![AppAction::ToggleLinkEnabled],
        "Link Start/Stop" => vec![AppAction::ToggleLinkStartStopSync],
        _ => Vec::new(),
    }
}

fn recall_stored_loop_slot_action(target_label: &str) -> Option<AppAction> {
    match target_label {
        "Recall Stored Loop Slot 1" => Some(AppAction::RecallStoredLoopSlot1),
        "Recall Stored Loop Slot 2" => Some(AppAction::RecallStoredLoopSlot2),
        "Recall Stored Loop Slot 3" => Some(AppAction::RecallStoredLoopSlot3),
        "Recall Stored Loop Slot 4" => Some(AppAction::RecallStoredLoopSlot4),
        "Recall Stored Loop Slot 5" => Some(AppAction::RecallStoredLoopSlot5),
        "Recall Stored Loop Slot 6" => Some(AppAction::RecallStoredLoopSlot6),
        "Recall Stored Loop Slot 7" => Some(AppAction::RecallStoredLoopSlot7),
        "Recall Stored Loop Slot 8" => Some(AppAction::RecallStoredLoopSlot8),
        _ => None,
    }
}

fn store_stored_loop_slot_action(target_label: &str) -> Option<AppAction> {
    match target_label {
        "Store Current Loop To Slot 1" => Some(AppAction::StoreCurrentLoopToSlot1),
        "Store Current Loop To Slot 2" => Some(AppAction::StoreCurrentLoopToSlot2),
        "Store Current Loop To Slot 3" => Some(AppAction::StoreCurrentLoopToSlot3),
        "Store Current Loop To Slot 4" => Some(AppAction::StoreCurrentLoopToSlot4),
        "Store Current Loop To Slot 5" => Some(AppAction::StoreCurrentLoopToSlot5),
        "Store Current Loop To Slot 6" => Some(AppAction::StoreCurrentLoopToSlot6),
        "Store Current Loop To Slot 7" => Some(AppAction::StoreCurrentLoopToSlot7),
        "Store Current Loop To Slot 8" => Some(AppAction::StoreCurrentLoopToSlot8),
        _ => None,
    }
}

fn clear_stored_loop_slot_action(target_label: &str) -> Option<AppAction> {
    match target_label {
        "Clear Stored Loop Slot 1" => Some(AppAction::ClearStoredLoopSlot1),
        "Clear Stored Loop Slot 2" => Some(AppAction::ClearStoredLoopSlot2),
        "Clear Stored Loop Slot 3" => Some(AppAction::ClearStoredLoopSlot3),
        "Clear Stored Loop Slot 4" => Some(AppAction::ClearStoredLoopSlot4),
        "Clear Stored Loop Slot 5" => Some(AppAction::ClearStoredLoopSlot5),
        "Clear Stored Loop Slot 6" => Some(AppAction::ClearStoredLoopSlot6),
        "Clear Stored Loop Slot 7" => Some(AppAction::ClearStoredLoopSlot7),
        "Clear Stored Loop Slot 8" => Some(AppAction::ClearStoredLoopSlot8),
        _ => None,
    }
}

pub fn mapping_entry_key_actions(entry: &MappingEntry) -> Vec<AppAction> {
    mapping_entry_possible_actions(entry)
}

fn is_mapping_press_event(event: &MidiInputEvent) -> bool {
    match event.message {
        MidiInputMessage::NoteOn { .. } => true,
        MidiInputMessage::NoteOff { .. } => false,
        MidiInputMessage::ControlChange { value, .. } => value > 0,
    }
}

fn hold_mapping_actions(
    absolute_track_index: Option<usize>,
    event: &MidiInputEvent,
    start_action: AppAction,
    stop_action: AppAction,
) -> Vec<AppAction> {
    if is_mapping_press_event(event) {
        track_scoped_actions(absolute_track_index, start_action)
    } else {
        vec![stop_action]
    }
}

fn entry(
    source_kind: MappingSourceKind,
    source_label: &str,
    target_label: &str,
    scope_label: &str,
    enabled: bool,
) -> MappingEntry {
    MappingEntry {
        source_kind,
        source_device_label: default_mapping_source_device(),
        source_label: source_label.to_string(),
        target_label: target_label.to_string(),
        scope_label: scope_label.to_string(),
        enabled,
    }
}

fn midi_entry(
    source_device_label: &str,
    source_label: &str,
    target_label: &str,
    scope_label: &str,
    enabled: bool,
) -> MappingEntry {
    MappingEntry {
        source_kind: MappingSourceKind::Midi,
        source_device_label: source_device_label.to_string(),
        source_label: source_label.to_string(),
        target_label: target_label.to_string(),
        scope_label: scope_label.to_string(),
        enabled,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        MappingEntry, MappingSourceKind, cycle_mapping_scope_label, cycle_mapping_scope_value,
        cycle_mapping_source_device_label, cycle_mapping_source_kind, cycle_mapping_target_label,
        default_mapping_source_device, default_scope_label, default_source_label, demo_mappings,
        mapping_entry_targets_action, mapping_entry_to_actions, mapping_scope_valid_for_target,
        parse_absolute_track_scope, search_mapping_targets,
    };
    use crate::actions::AppAction;
    use crate::midi_io::{MidiInputEvent, MidiInputMessage, MidiPortRef};

    #[test]
    fn demo_mappings_cover_key_midi_and_osc_sources() {
        let mappings = demo_mappings();

        assert!(
            mappings
                .iter()
                .any(|entry| entry.source_kind == MappingSourceKind::Key)
        );
        assert!(
            mappings
                .iter()
                .any(|entry| entry.source_kind == MappingSourceKind::Midi)
        );
        assert!(
            mappings
                .iter()
                .any(|entry| entry.source_kind == MappingSourceKind::Osc)
        );
    }

    #[test]
    fn mapping_cycle_helpers_wrap() {
        assert_eq!(
            cycle_mapping_source_kind(MappingSourceKind::Key, -1),
            MappingSourceKind::Osc
        );
        assert_eq!(default_source_label(MappingSourceKind::Midi), "Note C2");
        assert_eq!(default_mapping_source_device(), "Any MIDI");
        assert_eq!(
            cycle_mapping_target_label("Play/Stop", -1),
            "Link Start/Stop"
        );
        assert_eq!(cycle_mapping_scope_label("Global", -1), "Absolute");
        assert_eq!(
            cycle_mapping_source_device_label("Any MIDI", &["Port A".to_string()], 1),
            "Port A"
        );
        assert_eq!(default_scope_label("Track Arm", 4), "Active Track");
        assert_eq!(
            cycle_mapping_scope_value("Active Track", 1, "Track Arm", 4),
            "Track 1"
        );
        assert_eq!(
            cycle_mapping_scope_value("Track 4", 1, "Track Arm", 4),
            "Active Track"
        );
    }

    #[test]
    fn default_new_mapping_starts_disabled() {
        let entry = MappingEntry::default_new();

        assert_eq!(entry.source_kind, MappingSourceKind::Key);
        assert_eq!(entry.target_label, "Play/Stop");
        assert_eq!(entry.scope_label, "Global");
        assert!(!entry.enabled);
    }

    #[test]
    fn target_lookup_search_matches_slot_queries_and_fuzzy_terms() {
        let slot_results = search_mapping_targets("slot 4");
        assert_eq!(
            slot_results.first().copied(),
            Some("Clear Stored Loop Slot 4")
        );
        assert!(slot_results.contains(&"Recall Stored Loop Slot 4"));
        assert!(slot_results.contains(&"Store Current Loop To Slot 4"));

        let arm_results = search_mapping_targets("arm");
        assert_eq!(arm_results.first().copied(), Some("Track Arm"));
    }

    #[test]
    fn target_scope_validation_matches_catalog_rules() {
        assert!(mapping_scope_valid_for_target("Track Arm", "Track 3", 4));
        assert!(mapping_scope_valid_for_target(
            "Track Arm",
            "Active Track",
            4
        ));
        assert!(!mapping_scope_valid_for_target("Play/Stop", "Track 3", 4));
        assert!(mapping_scope_valid_for_target("Play/Stop", "Global", 4));
    }

    #[test]
    fn mapping_entries_expand_track_scopes_into_actions() {
        let entry = MappingEntry {
            source_kind: MappingSourceKind::Midi,
            source_device_label: "Port A".to_string(),
            source_label: "CC20".to_string(),
            target_label: "Track Arm".to_string(),
            scope_label: "Track 3".to_string(),
            enabled: true,
        };
        let event = MidiInputEvent {
            port: MidiPortRef::new("Port A"),
            channel: 1,
            message: MidiInputMessage::ControlChange {
                controller: 20,
                value: 127,
            },
        };

        assert_eq!(
            mapping_entry_to_actions(&entry, &event),
            vec![AppAction::SelectTrack(2), AppAction::ToggleCurrentTrackArm]
        );
        assert!(mapping_entry_targets_action(
            &entry,
            AppAction::ToggleCurrentTrackArm
        ));
        assert_eq!(parse_absolute_track_scope("Track 3"), Some(2));
    }

    #[test]
    fn stored_loop_targets_expand_with_track_scope() {
        let entry = MappingEntry {
            source_kind: MappingSourceKind::Key,
            source_device_label: default_mapping_source_device(),
            source_label: "Numpad2".to_string(),
            target_label: "Recall Stored Loop Slot 2".to_string(),
            scope_label: "Track 4".to_string(),
            enabled: true,
        };
        let event = MidiInputEvent {
            port: MidiPortRef::new("Port A"),
            channel: 1,
            message: MidiInputMessage::ControlChange {
                controller: 20,
                value: 127,
            },
        };

        assert_eq!(
            mapping_entry_to_actions(&entry, &event),
            vec![AppAction::SelectTrack(3), AppAction::RecallStoredLoopSlot2]
        );
        assert!(mapping_entry_targets_action(
            &entry,
            AppAction::RecallStoredLoopSlot2
        ));
    }

    #[test]
    fn editor_navigation_targets_resolve_to_page_actions() {
        let event = MidiInputEvent {
            port: MidiPortRef::new("Port A"),
            channel: 1,
            message: MidiInputMessage::ControlChange {
                controller: 20,
                value: 127,
            },
        };
        let cases = [
            ("Previous Page Item", AppAction::SelectPreviousPageItem),
            ("Next Page Item", AppAction::SelectNextPageItem),
            (
                "Adjust Page Item Backward",
                AppAction::AdjustPageItemBackward,
            ),
            ("Adjust Page Item Forward", AppAction::AdjustPageItemForward),
            ("Activate Page Item", AppAction::ActivatePageItem),
            ("Cancel", AppAction::CancelCurrentMode),
            ("Mappings Write Mode", AppAction::ToggleMappingsWriteMode),
            ("Add Mapping", AppAction::AddMappingRow),
            ("Remove Mapping", AppAction::RemoveSelectedMapping),
            ("Previous Mapping Field", AppAction::SelectPreviousPageField),
            ("Next Mapping Field", AppAction::SelectNextPageField),
        ];

        for (target_label, action) in cases {
            let entry = MappingEntry {
                source_kind: MappingSourceKind::Midi,
                source_device_label: "Port A".to_string(),
                source_label: "CC20".to_string(),
                target_label: target_label.to_string(),
                scope_label: "Global".to_string(),
                enabled: true,
            };
            assert_eq!(mapping_entry_to_actions(&entry, &event), vec![action]);
            assert!(mapping_entry_targets_action(&entry, action));
        }
    }
}
