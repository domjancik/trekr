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
    use super::{MappingSourceKind, demo_mappings};

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
}
