#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MappingTarget {
    TransportPlay,
    TransportRecord,
    TrackArm,
    TrackMute,
    LoopSet,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MidiMapping {
    pub target: MappingTarget,
    pub track_index: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MappingSourceKind {
    Key,
    Midi,
    Osc,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MappingEntry {
    pub source_kind: MappingSourceKind,
    pub source_label: &'static str,
    pub target_label: &'static str,
    pub scope_label: &'static str,
    pub enabled: bool,
}

pub fn demo_mappings() -> Vec<MappingEntry> {
    vec![
        MappingEntry {
            source_kind: MappingSourceKind::Key,
            source_label: "Space",
            target_label: "Play/Stop",
            scope_label: "Global",
            enabled: true,
        },
        MappingEntry {
            source_kind: MappingSourceKind::Key,
            source_label: "L",
            target_label: "Track Loop",
            scope_label: "Active Track",
            enabled: true,
        },
        MappingEntry {
            source_kind: MappingSourceKind::Midi,
            source_label: "CC20",
            target_label: "Track Arm",
            scope_label: "Active Track",
            enabled: true,
        },
        MappingEntry {
            source_kind: MappingSourceKind::Midi,
            source_label: "Note C2",
            target_label: "Record Hold",
            scope_label: "Global",
            enabled: false,
        },
        MappingEntry {
            source_kind: MappingSourceKind::Osc,
            source_label: "/transport/play",
            target_label: "Play/Stop",
            scope_label: "Global",
            enabled: true,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::{MappingSourceKind, demo_mappings};

    #[test]
    fn demo_mappings_cover_key_midi_and_osc_sources() {
        let mappings = demo_mappings();

        assert!(mappings
            .iter()
            .any(|entry| entry.source_kind == MappingSourceKind::Key));
        assert!(mappings
            .iter()
            .any(|entry| entry.source_kind == MappingSourceKind::Midi));
        assert!(mappings
            .iter()
            .any(|entry| entry.source_kind == MappingSourceKind::Osc));
    }
}
