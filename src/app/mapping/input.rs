use super::*;

pub(crate) fn midi_learn_label(event: &MidiInputEvent) -> String {
    match event.message {
        MidiInputMessage::NoteOn { pitch, .. } | MidiInputMessage::NoteOff { pitch } => {
            format!("Note {} Ch{}", midi_note_name(pitch), event.channel)
        }
        MidiInputMessage::ControlChange { controller, .. } => {
            format!("CC{} Ch{}", controller, event.channel)
        }
    }
}

fn midi_note_name(pitch: u8) -> String {
    const NAMES: [&str; 12] = [
        "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
    ];
    let name = NAMES[(pitch % 12) as usize];
    let octave = (pitch / 12) as i16 - 1;
    format!("{name}{octave}")
}

fn midi_note_label(pitch: u8) -> String {
    format!("Note {}", midi_note_name(pitch))
}

pub(crate) fn midi_mapping_matches_event(entry: &MappingEntry, event: &MidiInputEvent) -> bool {
    if !entry.enabled || entry.source_kind != MappingSourceKind::Midi {
        return false;
    }

    if entry.source_device_label != default_mapping_source_device()
        && entry.source_device_label != event.port.name
    {
        return false;
    }

    match event.message {
        MidiInputMessage::NoteOn { pitch, .. } | MidiInputMessage::NoteOff { pitch } => {
            if matches!(event.message, MidiInputMessage::NoteOff { .. })
                && !midi_mapping_target_supports_release(entry.target_label.as_str())
            {
                return false;
            }
            entry.source_label == midi_note_label(pitch)
                || entry.source_label == format!("{} Ch{}", midi_note_label(pitch), event.channel)
        }
        MidiInputMessage::ControlChange { controller, value } => {
            if value == 0 && !midi_mapping_target_supports_release(entry.target_label.as_str()) {
                return false;
            }
            entry.source_label == format!("CC{controller}")
                || entry.source_label == format!("CC{controller} Ch{}", event.channel)
        }
    }
}

fn midi_mapping_target_supports_release(target_label: &str) -> bool {
    matches!(target_label, "Record Hold" | "Select Notes At Playhead Add")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn midi_learn_label_formats_note_and_cc_values() {
        let note = MidiInputEvent {
            channel: 2,
            port: MidiPortRef {
                name: "Port".to_string(),
            },
            message: MidiInputMessage::NoteOn {
                pitch: 60,
                velocity: 100,
            },
        };
        assert_eq!(midi_learn_label(&note), "Note C4 Ch2");

        let cc = MidiInputEvent {
            channel: 4,
            port: MidiPortRef {
                name: "Port".to_string(),
            },
            message: MidiInputMessage::ControlChange {
                controller: 74,
                value: 64,
            },
        };
        assert_eq!(midi_learn_label(&cc), "CC74 Ch4");
    }

    #[test]
    fn midi_mapping_match_requires_release_capable_target_for_note_off() {
        let entry = MappingEntry {
            source_kind: MappingSourceKind::Midi,
            source_device_label: "Port".to_string(),
            source_label: "Note C4 Ch1".to_string(),
            target_label: "Record Hold".to_string(),
            scope_label: "Global".to_string(),
            enabled: true,
        };
        let note_off = MidiInputEvent {
            channel: 1,
            port: MidiPortRef {
                name: "Port".to_string(),
            },
            message: MidiInputMessage::NoteOff { pitch: 60 },
        };
        assert!(midi_mapping_matches_event(&entry, &note_off));

        let non_release = MappingEntry {
            target_label: "ToggleCurrentTrackArm".to_string(),
            ..entry
        };
        assert!(!midi_mapping_matches_event(&non_release, &note_off));
    }
}
