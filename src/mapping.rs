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
        MappingSourceKind, cycle_mapping_scope_label, cycle_mapping_source_device_label,
        cycle_mapping_source_kind, cycle_mapping_target_label, default_mapping_source_device,
        default_source_label, demo_mappings,
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
    }
}
