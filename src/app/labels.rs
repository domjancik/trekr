use crate::actions::ActionSource;
use crate::mapping::MappingSourceKind;
use crate::transport::{LaunchQuantizeMode, QuantizeMode};
use sdl3::pixels::Color;

use super::types::MappingBadge;

pub(super) fn mapping_source_label(source: MappingSourceKind) -> &'static str {
    match source {
        MappingSourceKind::Key => "Key",
        MappingSourceKind::Midi => "MIDI",
        MappingSourceKind::Osc => "OSC",
    }
}

pub(super) fn compact_scope_label(scope: &str) -> &str {
    match scope {
        "Active Track" => "Act Track",
        "Armed/Active" => "Armed/Act",
        "Global" => "Global",
        "Relative" => "Relative",
        "Absolute" => "Absolute",
        other => other,
    }
}

pub(super) fn quantize_label(quantize: QuantizeMode) -> &'static str {
    match quantize {
        QuantizeMode::Off => "Off",
        QuantizeMode::Pulse => "Pulse",
        QuantizeMode::Sixteenth => "1/16",
        QuantizeMode::Eighth => "1/8",
        QuantizeMode::Quarter => "1/4",
        QuantizeMode::Bar => "Bar",
    }
}

pub(super) fn launch_quantize_label(quantize: LaunchQuantizeMode) -> &'static str {
    match quantize {
        LaunchQuantizeMode::Off => "Off",
        LaunchQuantizeMode::Sixteenth => "1/16",
        LaunchQuantizeMode::Eighth => "1/8",
        LaunchQuantizeMode::Quarter => "1/4",
        LaunchQuantizeMode::Bar => "Bar",
        LaunchQuantizeMode::LoopEnd => "LoopEnd",
    }
}

pub(super) fn action_source_label(source: ActionSource) -> &'static str {
    match source {
        ActionSource::Keyboard => "Keyboard",
        ActionSource::Pointer => "Pointer",
        ActionSource::Midi => "MIDI",
        ActionSource::Touch => "Touch",
        ActionSource::Remote => "Remote",
        ActionSource::Internal => "Internal",
    }
}

pub(super) fn mapping_source_sort_key(source_kind: MappingSourceKind) -> usize {
    match source_kind {
        MappingSourceKind::Key => 0,
        MappingSourceKind::Midi => 1,
        MappingSourceKind::Osc => 2,
    }
}

pub(super) fn badge_kind_prefix(source_kind: MappingSourceKind) -> &'static str {
    match source_kind {
        MappingSourceKind::Key => "K",
        MappingSourceKind::Midi => "M",
        MappingSourceKind::Osc => "O",
    }
}

pub(super) fn mapping_badge_palette(badge: &MappingBadge) -> (Color, Color) {
    match (badge.built_in, badge.source_kind) {
        (true, MappingSourceKind::Key) => (Color::RGB(64, 84, 126), Color::RGB(244, 244, 236)),
        (true, MappingSourceKind::Midi) => (Color::RGB(88, 94, 116), Color::RGB(236, 240, 246)),
        (true, MappingSourceKind::Osc) => (Color::RGB(84, 90, 112), Color::RGB(236, 240, 246)),
        (false, MappingSourceKind::Key) => (Color::RGB(88, 128, 76), Color::RGB(246, 248, 232)),
        (false, MappingSourceKind::Midi) => (Color::RGB(170, 104, 62), Color::RGB(250, 242, 228)),
        (false, MappingSourceKind::Osc) => (Color::RGB(148, 82, 104), Color::RGB(248, 238, 244)),
    }
}

pub(super) fn compact_badge_text(text: &str, max_len: usize) -> String {
    let compact = text
        .replace("Shift+", "S+")
        .replace("Space", "Spc")
        .replace("Left", "Lf")
        .replace("Right", "Rt")
        .replace("Active", "Act");
    if compact.chars().count() <= max_len {
        compact
    } else {
        compact.chars().take(max_len).collect()
    }
}
