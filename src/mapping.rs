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
    pub source_label: String,
    pub target_label: String,
    pub scope_label: String,
    pub enabled: bool,
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
    "Song Loop",
    "Track Loop",
    "Clear Track",
    "Clear All",
    "Track Arm",
    "Track Mute",
    "Track Solo",
    "Passthrough",
    "Select Track",
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

pub fn demo_mappings() -> Vec<MappingEntry> {
    vec![
        MappingEntry {
            source_kind: MappingSourceKind::Key,
            source_label: "Space".to_string(),
            target_label: "Play/Stop".to_string(),
            scope_label: "Global".to_string(),
            enabled: true,
        },
        MappingEntry {
            source_kind: MappingSourceKind::Key,
            source_label: "Shift+R".to_string(),
            target_label: "Record Mode".to_string(),
            scope_label: "Global".to_string(),
            enabled: true,
        },
        MappingEntry {
            source_kind: MappingSourceKind::Key,
            source_label: "R".to_string(),
            target_label: "Record".to_string(),
            scope_label: "Armed/Active".to_string(),
            enabled: true,
        },
        MappingEntry {
            source_kind: MappingSourceKind::Key,
            source_label: "C".to_string(),
            target_label: "Clear Track".to_string(),
            scope_label: "Active Track".to_string(),
            enabled: true,
        },
        MappingEntry {
            source_kind: MappingSourceKind::Key,
            source_label: "Shift+C".to_string(),
            target_label: "Clear All".to_string(),
            scope_label: "Global".to_string(),
            enabled: true,
        },
        MappingEntry {
            source_kind: MappingSourceKind::Key,
            source_label: "G".to_string(),
            target_label: "Song Loop".to_string(),
            scope_label: "Global".to_string(),
            enabled: true,
        },
        MappingEntry {
            source_kind: MappingSourceKind::Key,
            source_label: "L".to_string(),
            target_label: "Track Loop".to_string(),
            scope_label: "Active Track".to_string(),
            enabled: true,
        },
        MappingEntry {
            source_kind: MappingSourceKind::Midi,
            source_label: "CC20".to_string(),
            target_label: "Track Arm".to_string(),
            scope_label: "Active Track".to_string(),
            enabled: true,
        },
        MappingEntry {
            source_kind: MappingSourceKind::Key,
            source_label: "A".to_string(),
            target_label: "Track Arm".to_string(),
            scope_label: "Active Track".to_string(),
            enabled: true,
        },
        MappingEntry {
            source_kind: MappingSourceKind::Key,
            source_label: "M".to_string(),
            target_label: "Track Mute".to_string(),
            scope_label: "Active Track".to_string(),
            enabled: true,
        },
        MappingEntry {
            source_kind: MappingSourceKind::Key,
            source_label: "S".to_string(),
            target_label: "Track Solo".to_string(),
            scope_label: "Active Track".to_string(),
            enabled: true,
        },
        MappingEntry {
            source_kind: MappingSourceKind::Key,
            source_label: "I".to_string(),
            target_label: "Passthrough".to_string(),
            scope_label: "Active Track".to_string(),
            enabled: true,
        },
        MappingEntry {
            source_kind: MappingSourceKind::Key,
            source_label: "[ ]".to_string(),
            target_label: "Set Track Loop".to_string(),
            scope_label: "Active Track".to_string(),
            enabled: true,
        },
        MappingEntry {
            source_kind: MappingSourceKind::Key,
            source_label: "Shift+[ ]".to_string(),
            target_label: "Set Song Loop".to_string(),
            scope_label: "Global".to_string(),
            enabled: true,
        },
        MappingEntry {
            source_kind: MappingSourceKind::Key,
            source_label: ", .".to_string(),
            target_label: "Nudge Loop".to_string(),
            scope_label: "Active Track".to_string(),
            enabled: true,
        },
        MappingEntry {
            source_kind: MappingSourceKind::Key,
            source_label: "Shift+, .".to_string(),
            target_label: "Nudge Song Loop".to_string(),
            scope_label: "Global".to_string(),
            enabled: true,
        },
        MappingEntry {
            source_kind: MappingSourceKind::Key,
            source_label: "- =".to_string(),
            target_label: "Resize Loop".to_string(),
            scope_label: "Active Track".to_string(),
            enabled: true,
        },
        MappingEntry {
            source_kind: MappingSourceKind::Key,
            source_label: "/ \\".to_string(),
            target_label: "Half/Double Loop".to_string(),
            scope_label: "Active Track".to_string(),
            enabled: true,
        },
        MappingEntry {
            source_kind: MappingSourceKind::Key,
            source_label: "Left/Right".to_string(),
            target_label: "Select Track".to_string(),
            scope_label: "Relative".to_string(),
            enabled: true,
        },
        MappingEntry {
            source_kind: MappingSourceKind::Key,
            source_label: "1-9".to_string(),
            target_label: "Select Track".to_string(),
            scope_label: "Absolute".to_string(),
            enabled: true,
        },
        MappingEntry {
            source_kind: MappingSourceKind::Key,
            source_label: "Tab/F1-F5".to_string(),
            target_label: "Pages/Overlay".to_string(),
            scope_label: "Global".to_string(),
            enabled: true,
        },
        MappingEntry {
            source_kind: MappingSourceKind::Midi,
            source_label: "Note C2".to_string(),
            target_label: "Record Hold".to_string(),
            scope_label: "Global".to_string(),
            enabled: false,
        },
        MappingEntry {
            source_kind: MappingSourceKind::Midi,
            source_label: "CC21".to_string(),
            target_label: "Track Loop".to_string(),
            scope_label: "Active Track".to_string(),
            enabled: false,
        },
        MappingEntry {
            source_kind: MappingSourceKind::Midi,
            source_label: "CC22".to_string(),
            target_label: "Track Mute".to_string(),
            scope_label: "Active Track".to_string(),
            enabled: false,
        },
        MappingEntry {
            source_kind: MappingSourceKind::Osc,
            source_label: "/transport/play".to_string(),
            target_label: "Play/Stop".to_string(),
            scope_label: "Global".to_string(),
            enabled: true,
        },
        MappingEntry {
            source_kind: MappingSourceKind::Osc,
            source_label: "/track/active/arm".to_string(),
            target_label: "Track Arm".to_string(),
            scope_label: "Active Track".to_string(),
            enabled: false,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::{
        MappingSourceKind, cycle_mapping_scope_label, cycle_mapping_source_kind,
        cycle_mapping_target_label, default_source_label, demo_mappings,
    };

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
        assert_eq!(
            cycle_mapping_target_label("Play/Stop", -1),
            "Link Start/Stop"
        );
        assert_eq!(cycle_mapping_scope_label("Global", -1), "Absolute");
    }
}
