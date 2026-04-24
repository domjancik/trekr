use crate::actions::ActionSource;
use crate::mapping::MappingSourceKind;
use crate::pages::MappingField;
use crate::routing::MidiChannelFilter;
use crate::transport::{LaunchQuantizeMode, QuantizeMode};
use sdl3::pixels::Color;

use super::types::MappingBadge;

pub(crate) fn mapping_source_label(source: MappingSourceKind) -> &'static str {
    match source {
        MappingSourceKind::Key => "Key",
        MappingSourceKind::Midi => "MIDI",
        MappingSourceKind::Osc => "OSC",
    }
}

pub(crate) fn compact_scope_label(scope: &str) -> &str {
    match scope {
        "Active Track" => "Act Track",
        "Armed/Active" => "Armed/Act",
        "Global" => "Global",
        "Relative" => "Relative",
        "Absolute" => "Absolute",
        other => other,
    }
}

pub(crate) fn quantize_label(quantize: QuantizeMode) -> &'static str {
    match quantize {
        QuantizeMode::Off => "Off",
        QuantizeMode::Pulse => "Pulse",
        QuantizeMode::Sixteenth => "1/16",
        QuantizeMode::Eighth => "1/8",
        QuantizeMode::Quarter => "1/4",
        QuantizeMode::Bar => "Bar",
    }
}

pub(crate) fn launch_quantize_label(quantize: LaunchQuantizeMode) -> &'static str {
    match quantize {
        LaunchQuantizeMode::Off => "Off",
        LaunchQuantizeMode::Sixteenth => "1/16",
        LaunchQuantizeMode::Eighth => "1/8",
        LaunchQuantizeMode::Quarter => "1/4",
        LaunchQuantizeMode::Bar => "Bar",
        LaunchQuantizeMode::LoopEnd => "LoopEnd",
    }
}

pub(crate) fn action_source_label(source: ActionSource) -> &'static str {
    match source {
        ActionSource::Keyboard => "Keyboard",
        ActionSource::Pointer => "Pointer",
        ActionSource::Midi => "MIDI",
        ActionSource::Touch => "Touch",
        ActionSource::Remote => "Remote",
        ActionSource::Internal => "Internal",
    }
}

pub(crate) fn mapping_source_sort_key(source_kind: MappingSourceKind) -> usize {
    match source_kind {
        MappingSourceKind::Key => 0,
        MappingSourceKind::Midi => 1,
        MappingSourceKind::Osc => 2,
    }
}

pub(crate) fn badge_kind_prefix(source_kind: MappingSourceKind) -> &'static str {
    match source_kind {
        MappingSourceKind::Key => "K",
        MappingSourceKind::Midi => "M",
        MappingSourceKind::Osc => "O",
    }
}

pub(crate) fn mapping_badge_palette(badge: &MappingBadge) -> (Color, Color) {
    match (badge.built_in, badge.source_kind) {
        (true, MappingSourceKind::Key) => (Color::RGB(64, 84, 126), Color::RGB(244, 244, 236)),
        (true, MappingSourceKind::Midi) => (Color::RGB(88, 94, 116), Color::RGB(236, 240, 246)),
        (true, MappingSourceKind::Osc) => (Color::RGB(84, 90, 112), Color::RGB(236, 240, 246)),
        (false, MappingSourceKind::Key) => (Color::RGB(88, 128, 76), Color::RGB(246, 248, 232)),
        (false, MappingSourceKind::Midi) => (Color::RGB(170, 104, 62), Color::RGB(250, 242, 228)),
        (false, MappingSourceKind::Osc) => (Color::RGB(148, 82, 104), Color::RGB(248, 238, 244)),
    }
}

pub(crate) fn compact_badge_text(text: &str, max_len: usize) -> String {
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

pub(crate) fn mapping_field_index(field: MappingField) -> usize {
    match field {
        MappingField::SourceKind => 0,
        MappingField::SourceDevice => 1,
        MappingField::SourceValue => 2,
        MappingField::Target => 3,
        MappingField::Scope => 4,
        MappingField::Enabled => 5,
    }
}

pub(crate) fn input_channel_label(channel: MidiChannelFilter) -> String {
    match channel {
        MidiChannelFilter::Omni => "all".to_string(),
        MidiChannelFilter::Channel(value) => value.to_string(),
    }
}

pub(crate) fn output_channel_label(channel: Option<u8>) -> String {
    channel
        .map(|value| value.to_string())
        .unwrap_or_else(|| "none".to_string())
}

pub(crate) fn on_off(value: bool) -> &'static str {
    if value { "on" } else { "off" }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn channel_labels_match_compact_shell_status_format() {
        assert_eq!(input_channel_label(MidiChannelFilter::Omni), "all");
        assert_eq!(
            input_channel_label(MidiChannelFilter::Channel(12)),
            "12".to_string()
        );
        assert_eq!(output_channel_label(None), "none");
        assert_eq!(output_channel_label(Some(3)), "3".to_string());
    }

    #[test]
    fn on_off_uses_short_status_words() {
        assert_eq!(on_off(true), "on");
        assert_eq!(on_off(false), "off");
    }
}
