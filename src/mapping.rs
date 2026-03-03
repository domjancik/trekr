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
    "Clear Track",
    "Clear All",
    "Track Arm",
    "Track Mute",
    "Track Solo",
    "Passthrough",
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

pub fn cycle_mapping_scope_label(current: &str, delta: i32) -> &'static str {
    cycle_label(SCOPE_OPTIONS, current, delta)
}

pub fn default_scope_label(target_label: &str, track_count: usize) -> String {
    scope_options_for_target(target_label, track_count)
        .first()
        .cloned()
        .unwrap_or_else(|| "Global".to_string())
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

fn scope_options_for_target(target_label: &str, track_count: usize) -> Vec<String> {
    match target_label {
        "Play/Stop"
        | "Record Mode"
        | "Loop Recording Wrap"
        | "Song Loop"
        | "Set Song Loop"
        | "Reset Song Loop"
        | "Clear All"
        | "Pages/Overlay"
        | "Link Enable"
        | "Link Start/Stop" => vec!["Global".to_string()],
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
        "Track Loop" | "Set Track Loop" | "Clear Track" | "Track Arm" | "Track Mute"
        | "Track Solo" | "Passthrough" => {
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
        "Play/Stop" => vec![AppAction::TogglePlayback],
        "Record" | "Record Hold" => vec![AppAction::ToggleRecording],
        "Record Mode" => vec![AppAction::CycleRecordMode],
        "Loop Recording Wrap" => vec![AppAction::ToggleLoopRecordingExtension],
        "Song Loop" | "Set Song Loop" => vec![AppAction::ToggleGlobalLoop],
        "Reset Song Loop" => vec![AppAction::ResetGlobalLoop],
        "Track Loop" | "Set Track Loop" => {
            track_scoped_actions(absolute_track_index, AppAction::ToggleCurrentTrackLoop)
        }
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
        "Pages/Overlay" => vec![AppAction::ToggleMappingsOverlay],
        "Link Enable" => vec![AppAction::ToggleLinkEnabled],
        "Link Start/Stop" => vec![AppAction::ToggleLinkStartStopSync],
        _ => Vec::new(),
    }
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
        mapping_entry_targets_action, mapping_entry_to_actions, parse_absolute_track_scope,
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
}
