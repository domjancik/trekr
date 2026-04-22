use super::*;

pub(super) fn scheduled_note_occurrences(
    track: &Track,
    notes: &[MidiNote],
    previous_ticks: u64,
    advanced_ticks: u64,
    loop_range: Option<crate::timeline::LoopRegion>,
) -> Vec<MidiNote> {
    if advanced_ticks == 0 || track.state.muted {
        return Vec::new();
    }

    let segments = loop_range
        .map(|range| ranged_segments(previous_ticks, advanced_ticks, range))
        .unwrap_or_else(|| {
            vec![(
                previous_ticks,
                previous_ticks.saturating_add(advanced_ticks),
            )]
        });

    let mut occurrences = Vec::new();
    let mut transport_cursor = previous_ticks;
    for (segment_start, segment_end) in segments {
        for note in notes {
            if track.recording_clip_is_muted(note.recording_clip_id) {
                continue;
            }
            let overlap_start = note.start_ticks.max(segment_start);
            let overlap_end = note.end_ticks().min(segment_end);
            if overlap_start >= overlap_end {
                continue;
            }
            occurrences.push(MidiNote {
                pitch: note.pitch,
                start_ticks: if note.start_ticks >= segment_start {
                    transport_cursor.saturating_add(note.start_ticks.saturating_sub(segment_start))
                } else {
                    transport_cursor.saturating_sub(segment_start.saturating_sub(note.start_ticks))
                },
                length_ticks: note.length_ticks,
                velocity: note.velocity,
                recording_clip_id: note.recording_clip_id,
            });
        }
        transport_cursor =
            transport_cursor.saturating_add(segment_end.saturating_sub(segment_start));
    }

    occurrences.sort_by_key(|note| (note.start_ticks, note.pitch, note.length_ticks));
    occurrences
}

pub(super) fn occurrence_note_events(
    track: &Track,
    notes: &[MidiNote],
    previous_ticks: u64,
    advanced_ticks: u64,
) -> Vec<(u64, bool, u8, u8)> {
    if track.state.muted {
        Vec::new()
    } else {
        occurrence_note_events_unmuted(notes, previous_ticks, advanced_ticks)
    }
}

pub(super) fn occurrence_note_events_unmuted(
    notes: &[MidiNote],
    previous_ticks: u64,
    advanced_ticks: u64,
) -> Vec<(u64, bool, u8, u8)> {
    let end_ticks = previous_ticks.saturating_add(advanced_ticks);
    let mut events = Vec::with_capacity(notes.len().saturating_mul(2));
    for note in notes {
        if note.start_ticks >= previous_ticks && note.start_ticks < end_ticks {
            events.push((note.start_ticks, true, note.pitch, note.velocity));
        }
        let note_end = note.end_ticks();
        if note_end >= previous_ticks && note_end < end_ticks {
            events.push((note_end, false, note.pitch, note.velocity));
        }
    }
    events.sort_by_key(|event| (event.0, event.1));
    events
}

fn ranged_segments(
    previous_ticks: u64,
    advanced_ticks: u64,
    range: crate::timeline::LoopRegion,
) -> Vec<(u64, u64)> {
    if range.length_ticks == 0 || advanced_ticks == 0 {
        return Vec::new();
    }

    let mut segments = Vec::new();
    let mut remaining = advanced_ticks;
    let mut cursor = range.start_ticks + (previous_ticks % range.length_ticks);
    let end = range.end_ticks();

    while remaining > 0 {
        let next_boundary = end.min(cursor.saturating_add(remaining));
        segments.push((cursor, next_boundary));
        let consumed = next_boundary.saturating_sub(cursor);
        if consumed >= remaining {
            break;
        }

        remaining = remaining.saturating_sub(consumed);
        cursor = range.start_ticks;
    }

    segments
}

pub(super) fn ticks_per_second_for_tempo(tempo_bpm: f64, ppqn: u16) -> u64 {
    let clamped_bpm = tempo_bpm.clamp(20.0, 400.0);
    ((clamped_bpm * f64::from(ppqn.max(1))) / 60.0).round() as u64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::{RecordContext, TrackKind};
    use crate::transport::QuantizeMode;

    #[test]
    fn ticks_per_second_for_tempo_matches_expected_values() {
        assert_eq!(ticks_per_second_for_tempo(120.0, 960), 1_920);
        assert_eq!(ticks_per_second_for_tempo(90.0, 960), 1_440);
    }

    #[test]
    fn scheduled_note_events_include_recorded_take_notes_before_commit() {
        let transport = crate::transport::Transport {
            quantize: QuantizeMode::Off,
            ..crate::transport::Transport::default()
        };
        let mut track = Track::new_empty("Track 1", TrackKind::Midi);
        track.state.loop_enabled = true;
        track.loop_region = crate::timeline::LoopRegion::new(960, 960);
        track.begin_recording(1_680);
        track.record_note_on(64, 100, 1_700);
        track.record_note_off(64, 1_760);

        let preview_notes = track.playback_preview_notes(
            transport,
            2_670,
            Some(RecordContext {
                range: track.loop_region,
                wrap_basis_ticks: 0,
                extend_clip_on_wrap: true,
            }),
        );
        let preview_occurrences = scheduled_note_occurrences(
            &track,
            preview_notes.as_slice(),
            2_650,
            20,
            Some(track.loop_region),
        );
        let events = occurrence_note_events(&track, preview_occurrences.as_slice(), 2_650, 20);

        assert!(events.iter().any(|event| *event == (2_660, true, 64, 100)));
    }

    #[test]
    fn scheduled_note_events_ignore_pending_take_notes_until_note_off() {
        let transport = crate::transport::Transport {
            quantize: QuantizeMode::Off,
            ..crate::transport::Transport::default()
        };
        let mut track = Track::new_empty("Track 1", TrackKind::Midi);
        track.state.loop_enabled = true;
        track.loop_region = crate::timeline::LoopRegion::new(960, 960);
        track.begin_recording(1_680);
        track.record_note_on(64, 100, 1_700);

        let preview_notes = track.playback_preview_notes(
            transport,
            2_670,
            Some(RecordContext {
                range: track.loop_region,
                wrap_basis_ticks: 0,
                extend_clip_on_wrap: true,
            }),
        );
        let preview_occurrences = scheduled_note_occurrences(
            &track,
            preview_notes.as_slice(),
            2_650,
            20,
            Some(track.loop_region),
        );
        let events = occurrence_note_events(&track, preview_occurrences.as_slice(), 2_650, 20);

        assert!(events.is_empty());
    }
}
