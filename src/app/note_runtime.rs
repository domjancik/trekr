use super::*;

pub(crate) fn scheduled_note_occurrences(
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

pub(crate) fn occurrence_note_events(
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

pub(crate) fn occurrence_note_events_unmuted(
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

pub(crate) fn ticks_per_second_for_tempo(tempo_bpm: f64, ppqn: u16) -> u64 {
    let clamped_bpm = tempo_bpm.clamp(20.0, 400.0);
    ((clamped_bpm * f64::from(ppqn.max(1))) / 60.0).round() as u64
}

impl App {
    #[cfg(test)]
    pub(super) fn effective_track_output_notes(&self, track_index: usize) -> Vec<MidiNote> {
        let Some(track) = self.project.tracks.get(track_index) else {
            return Vec::new();
        };
        let mut visited = vec![false; self.project.tracks.len()];
        let notes = self.effective_track_pre_output_playback_notes_recursive(
            track_index,
            self.transport_ticks,
            self.project.transport.ppqn as u64,
            &mut visited,
        );
        transform_notes(
            &notes,
            &track.midi_fx.output_fx,
            self.project.global_harmony.root,
        )
    }

    pub(super) fn dispatch_midi_notes(&mut self, previous_ticks: u64, advanced_ticks: u64) {
        if advanced_ticks == 0 {
            return;
        }

        let track_events: Vec<(
            Option<MidiPortRef>,
            u8,
            Vec<(u64, bool, u8, u8)>,
            Vec<(u64, bool, u8, u8)>,
        )> = self
            .project
            .tracks
            .iter()
            .enumerate()
            .map(|(track_index, track)| {
                let channel = track.routing.output_channel.unwrap_or(1).clamp(1, 16);
                let port = track
                    .routing
                    .output_port
                    .cloned_resolved(self.default_output_port());
                let output_lookback = playback_timing_lookback_ticks(&track.midi_fx.output_fx);
                let lookback_padding =
                    output_lookback.saturating_add(u64::from(output_lookback > 0));
                let source_previous_ticks = previous_ticks.saturating_sub(lookback_padding);
                let source_advanced_ticks = advanced_ticks.saturating_add(lookback_padding);
                let mut visited = vec![false; self.project.tracks.len()];
                let mut pre_output_notes = self
                    .effective_track_pre_output_playback_notes_recursive(
                        track_index,
                        source_previous_ticks,
                        source_advanced_ticks,
                        &mut visited,
                    );
                let preview_notes = track.playback_preview_notes(
                    self.project.transport,
                    self.transport_ticks,
                    self.record_context(track),
                );
                let preview_occurrences = scheduled_note_occurrences(
                    track,
                    &preview_notes,
                    source_previous_ticks,
                    source_advanced_ticks,
                    self.playback_loop_range_for_track(track),
                );
                pre_output_notes.extend(preview_occurrences);
                let clone_notes = self.effective_track_clone_playback_notes_recursive(
                    track_index,
                    previous_ticks,
                    advanced_ticks,
                    &mut visited,
                );
                let transformed_notes = transform_notes(
                    &pre_output_notes,
                    &track.midi_fx.output_fx,
                    self.project.global_harmony.root,
                );
                let events = occurrence_note_events(
                    track,
                    &transformed_notes,
                    previous_ticks,
                    advanced_ticks,
                );
                let record_events = if track.active_take.is_some()
                    && track.midi_fx.record_input_fx_mode
                        == crate::midi_fx::RecordInputFxMode::PostInputFx
                {
                    occurrence_note_events_unmuted(&clone_notes, previous_ticks, advanced_ticks)
                } else {
                    Vec::new()
                };
                (port, channel, events, record_events)
            })
            .collect();

        for (track_index, (port, channel, events, record_events)) in
            track_events.into_iter().enumerate()
        {
            if let Some(track) = self.project.tracks.get_mut(track_index) {
                for (event_ticks, note_on, pitch, velocity) in &record_events {
                    if *note_on {
                        track.record_note_on(*pitch, *velocity, *event_ticks);
                    } else {
                        track.record_note_off(*pitch, *event_ticks);
                    }
                }
            }

            let Some(port) = port else {
                continue;
            };

            let mut refresh_needed = false;
            for (_, note_on, pitch, velocity) in events {
                let result = if note_on {
                    self.midi_output
                        .send_note_on(&port, channel, pitch, velocity)
                } else {
                    self.midi_output.send_note_off(&port, channel, pitch)
                };
                if result.is_err() {
                    refresh_needed = true;
                }
            }
            if refresh_needed {
                self.refresh_midi_devices_now();
            }
        }
    }

    pub(super) fn silence_all_tracks(&mut self) {
        let ports_and_channels: Vec<(MidiPortRef, u8)> = self
            .project
            .tracks
            .iter()
            .filter_map(|track| {
                track
                    .routing
                    .output_port
                    .cloned_resolved(self.default_output_port())
                    .zip(track.routing.output_channel)
            })
            .collect();

        for (port, channel) in ports_and_channels {
            if self.midi_output.send_all_notes_off(&port, channel).is_err() {
                self.refresh_midi_devices_now();
            }
        }
    }

    pub(super) fn silence_tracks_for_loop_change(&mut self) {
        self.silence_all_tracks();
    }

    pub(super) fn handle_timeline_fx_configuration_changed(&mut self) {
        self.silence_all_tracks();
        let current_ticks = if self.project.transport.playing {
            self.transport_ticks
        } else {
            self.live_fx_ticks
        };
        self.reset_live_fx_timing(current_ticks);
    }

    pub(super) fn reset_live_fx_timing(&mut self, current_ticks: u64) {
        for state in &mut self.input_fx_live_states {
            reset_live_fx_timing(state, current_ticks);
        }
        for state in &mut self.output_fx_live_states {
            reset_live_fx_timing(state, current_ticks);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::{RecordContext, TrackKind};
    use crate::transport::QuantizeMode;
    use std::sync::{Mutex, OnceLock};
    use std::time::{Duration, Instant};

    fn runtime_test_guard() -> std::sync::MutexGuard<'static, ()> {
        static RUNTIME_TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        RUNTIME_TEST_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn runtime_test_timeout() -> Duration {
        Duration::from_secs(5)
    }

    fn wait_for_sent_messages<F>(
        app: &mut App,
        timeout: Duration,
        predicate: F,
    ) -> Vec<(String, u8, u8, Option<u8>)>
    where
        F: Fn(&[(String, u8, u8, Option<u8>)]) -> bool,
    {
        let started_at = Instant::now();
        loop {
            let sent = app.midi_output.sent_messages();
            if predicate(sent.as_slice()) {
                return sent;
            }
            assert!(
                started_at.elapsed() < timeout,
                "expected MIDI runtime output within {:?}, got {:?}",
                timeout,
                sent
            );
            std::thread::sleep(Duration::from_millis(20));
        }
    }

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

    #[test]
    fn changing_global_loop_sends_all_notes_off() {
        let mut app = App::new();
        app.project.tracks[0].routing.output_port =
            TrackPortSelection::named(MidiPortRef::new("Out A"));
        app.project.tracks[0].routing.output_channel = Some(1);

        app.apply_action(AppAction::SetGlobalLoopStart);

        assert!(
            app.midi_output
                .sent_messages()
                .iter()
                .any(|(port, channel, pitch, velocity)| {
                    port == "Out A" && *channel == 1 && *pitch == 123 && velocity.is_none()
                })
        );
    }

    #[test]
    fn changing_track_loop_sends_all_notes_off() {
        let mut app = App::new();
        app.project.tracks[0].routing.output_port =
            TrackPortSelection::named(MidiPortRef::new("Out A"));
        app.project.tracks[0].routing.output_channel = Some(1);

        app.apply_action(AppAction::NudgeCurrentTrackLoopForward);

        assert!(
            app.midi_output
                .sent_messages()
                .iter()
                .any(|(port, channel, pitch, velocity)| {
                    port == "Out A" && *channel == 1 && *pitch == 123 && velocity.is_none()
                })
        );
    }

    #[test]
    fn shortening_duration_mid_playback_sends_all_notes_off_and_resets_fx_timing() {
        let mut app = App::new();
        app.project.clear_all_track_content();
        app.project.select_track(0);
        app.project.tracks[0].routing.output_port =
            TrackPortSelection::named(MidiPortRef::new("Out A"));
        app.project.tracks[0].routing.output_channel = Some(1);
        app.project.tracks[0]
            .midi_notes
            .push(MidiNote::new(60, 0, 960, 100));
        app.project.tracks[0].midi_fx.output_fx[0] = Some(MidiFxSlot {
            enabled: true,
            effect: MidiFx::Duration { ticks: 960 },
        });
        app.project.transport.playing = true;

        app.dispatch_midi_notes(0, 240);
        app.playhead_ticks = 240;
        app.transport_ticks = 240;
        app.page_state.current_page = AppPage::Timeline;
        app.page_state.selected_timeline_context = TimelineContext::OutputFx;
        app.page_state.selected_timeline_fx_field = TimelineFxField::ParamPrimary;
        app.set_selected_timeline_fx_row(MidiFxChainKind::Output, 0);

        app.adjust_page_item(-1);

        let sent = app.midi_output.sent_messages();
        assert!(sent.iter().any(|(port, channel, pitch, velocity)| {
            port == "Out A" && *channel == 1 && *pitch == 123 && velocity.is_none()
        }));
    }

    #[test]
    fn stopped_live_input_delay_uses_live_fx_clock_for_note_on_and_note_off() {
        let _guard = runtime_test_guard();
        let mut app = App::new();
        app.project.clear_all_track_content();
        app.project.tracks[0].routing.input_port =
            TrackPortSelection::named(MidiPortRef::new("In A"));
        app.project.tracks[0].routing.output_port =
            TrackPortSelection::named(MidiPortRef::new("Out A"));
        app.project.tracks[0].routing.output_channel = Some(1);
        app.project.tracks[0].state.passthrough = true;
        app.project.tracks[0].midi_fx.monitor_input_fx = true;
        app.project.tracks[0].midi_fx.input_fx[0] = Some(MidiFxSlot {
            enabled: true,
            effect: MidiFx::Delay { ticks: 240 },
        });

        let input_port = app.project.tracks[0]
            .routing
            .input_port
            .as_named_port()
            .cloned()
            .unwrap();
        app.inject_midi_input_event(MidiInputEvent {
            port: input_port.clone(),
            channel: 1,
            message: MidiInputMessage::NoteOn {
                pitch: 60,
                velocity: 100,
            },

            received_at: std::time::Instant::now(),
            backend_timestamp_micros: None,
            sequence: 0,
        });
        assert!(app.midi_output.sent_messages().is_empty());

        let sent = wait_for_sent_messages(&mut app, runtime_test_timeout(), |sent| {
            sent.iter().any(|(port, channel, pitch, velocity)| {
                port == "Out A" && *channel == 1 && *pitch == 60 && velocity.is_some()
            })
        });
        assert!(sent.iter().any(|(port, channel, pitch, velocity)| {
            port == "Out A" && *channel == 1 && *pitch == 60 && velocity.is_some()
        }));

        app.inject_midi_input_event(MidiInputEvent {
            port: input_port,
            channel: 1,
            message: MidiInputMessage::NoteOff { pitch: 60 },

            received_at: std::time::Instant::now(),
            backend_timestamp_micros: None,
            sequence: 0,
        });

        let sent = wait_for_sent_messages(&mut app, runtime_test_timeout(), |sent| {
            sent.iter().any(|(port, channel, pitch, velocity)| {
                port == "Out A" && *channel == 1 && *pitch == 60 && velocity.is_none()
            })
        });
        assert!(sent.iter().any(|(port, channel, pitch, velocity)| {
            port == "Out A" && *channel == 1 && *pitch == 60 && velocity.is_none()
        }));
    }

    #[test]
    fn stopped_live_input_duration_uses_live_fx_clock_when_note_starts_late() {
        let _guard = runtime_test_guard();
        let mut app = App::new();
        app.project.clear_all_track_content();
        app.project.tracks[0].routing.input_port =
            TrackPortSelection::named(MidiPortRef::new("In A"));
        app.project.tracks[0].routing.output_port =
            TrackPortSelection::named(MidiPortRef::new("Out A"));
        app.project.tracks[0].routing.output_channel = Some(1);
        app.project.tracks[0].state.passthrough = true;
        app.project.tracks[0].midi_fx.monitor_input_fx = true;
        app.project.tracks[0].midi_fx.input_fx[0] = Some(MidiFxSlot {
            enabled: true,
            effect: MidiFx::Duration { ticks: 240 },
        });
        app.live_fx_ticks = 960;
        app.force_sync_midi_runtime();

        let input_port = app.project.tracks[0]
            .routing
            .input_port
            .as_named_port()
            .cloned()
            .unwrap();
        app.inject_midi_input_event(MidiInputEvent {
            port: input_port,
            channel: 1,
            message: MidiInputMessage::NoteOn {
                pitch: 60,
                velocity: 100,
            },

            received_at: std::time::Instant::now(),
            backend_timestamp_micros: None,
            sequence: 0,
        });

        let sent = wait_for_sent_messages(&mut app, runtime_test_timeout(), |sent| {
            sent.iter().any(|(port, channel, pitch, velocity)| {
                port == "Out A" && *channel == 1 && *pitch == 60 && velocity.is_some()
            })
        });
        assert!(sent.iter().any(|(port, channel, pitch, velocity)| {
            port == "Out A" && *channel == 1 && *pitch == 60 && velocity.is_some()
        }));
        let sent = wait_for_sent_messages(&mut app, runtime_test_timeout(), |sent| {
            sent.iter().any(|(port, channel, pitch, velocity)| {
                port == "Out A" && *channel == 1 && *pitch == 60 && velocity.is_none()
            })
        });
        assert!(sent.iter().any(|(port, channel, pitch, velocity)| {
            port == "Out A" && *channel == 1 && *pitch == 60 && velocity.is_none()
        }));
    }

    #[test]
    fn muting_track_sends_all_notes_off() {
        let mut app = App::new();
        app.project.clear_all_track_content();
        app.project.select_track(0);
        app.project.tracks[0].routing.output_port =
            TrackPortSelection::named(MidiPortRef::new("Out A"));
        app.project.tracks[0].routing.output_channel = Some(1);
        app.project.tracks[0]
            .midi_notes
            .push(MidiNote::new(60, 0, 1_920, 100));

        app.dispatch_midi_notes(0, 960);
        app.apply_action(AppAction::ToggleCurrentTrackMute);

        let sent = app.midi_output.sent_messages();
        assert!(sent.iter().any(|(port, channel, pitch, velocity)| {
            port == "Out A" && *channel == 1 && *pitch == 60 && velocity.is_some()
        }));
        assert!(sent.iter().any(|(port, channel, pitch, velocity)| {
            port == "Out A" && *channel == 1 && *pitch == 123 && velocity.is_none()
        }));
    }

    #[test]
    fn track_clone_passthrough_sends_live_output_to_target_track() {
        let _guard = runtime_test_guard();
        let mut app = App::new();
        app.project.clear_all_track_content();
        app.project.tracks[0].routing.input_port =
            TrackPortSelection::named(MidiPortRef::new("Test Input"));
        app.project.tracks[0].midi_fx.input_fx = vec![None; MIDI_FX_SLOT_COUNT];
        app.project.tracks[1].state.passthrough = true;
        app.project.tracks[1].midi_fx.monitor_input_fx = true;
        app.project.tracks[1].routing.output_port =
            TrackPortSelection::named(MidiPortRef::new("Out B"));
        app.project.tracks[1].routing.output_channel = Some(2);
        app.project.tracks[1].midi_fx.input_fx[0] = Some(MidiFxSlot {
            enabled: true,
            effect: MidiFx::TrackClone { source_track: 0 },
        });
        app.project.tracks[1].midi_fx.input_fx[1] = Some(MidiFxSlot {
            enabled: true,
            effect: MidiFx::Transpose { semitones: 12 },
        });
        app.project.tracks[1].midi_fx.output_fx = vec![None; MIDI_FX_SLOT_COUNT];

        let input_port = app.project.tracks[0]
            .routing
            .input_port
            .as_named_port()
            .cloned()
            .unwrap();
        app.inject_midi_input_event(MidiInputEvent {
            port: input_port.clone(),
            channel: 1,
            message: MidiInputMessage::NoteOn {
                pitch: 60,
                velocity: 100,
            },

            received_at: std::time::Instant::now(),
            backend_timestamp_micros: None,
            sequence: 0,
        });
        let _sent = wait_for_sent_messages(&mut app, runtime_test_timeout(), |sent| {
            sent.iter().any(|(port, channel, pitch, velocity)| {
                port == "Out B" && *channel == 2 && *pitch == 72 && velocity.is_some()
            })
        });
        app.inject_midi_input_event(MidiInputEvent {
            port: input_port,
            channel: 1,
            message: MidiInputMessage::NoteOff { pitch: 60 },

            received_at: std::time::Instant::now(),
            backend_timestamp_micros: None,
            sequence: 0,
        });

        let sent = wait_for_sent_messages(&mut app, runtime_test_timeout(), |sent| {
            sent.iter().any(|(port, channel, pitch, velocity)| {
                port == "Out B" && *channel == 2 && *pitch == 72 && velocity.is_some()
            }) && sent.iter().any(|(port, channel, pitch, velocity)| {
                port == "Out B" && *channel == 2 && *pitch == 72 && velocity.is_none()
            })
        });
        assert_eq!(
            sent.iter()
                .filter(|(port, channel, pitch, velocity)| {
                    port == "Out B" && *channel == 2 && *pitch == 72 && velocity.is_some()
                })
                .count(),
            1
        );
        assert_eq!(
            sent.iter()
                .filter(|(port, channel, pitch, velocity)| {
                    port == "Out B" && *channel == 2 && *pitch == 72 && velocity.is_none()
                })
                .count(),
            1
        );
        assert!(
            sent.iter()
                .any(|(port, channel, pitch, velocity)| port == "Out B"
                    && *channel == 2
                    && *pitch == 72
                    && velocity.is_some())
        );
        assert!(
            sent.iter()
                .any(|(port, channel, pitch, velocity)| port == "Out B"
                    && *channel == 2
                    && *pitch == 72
                    && velocity.is_none())
        );
    }

    #[test]
    fn track_clone_monitor_fx_sends_live_output_without_passthrough() {
        let _guard = runtime_test_guard();
        let mut app = App::new();
        app.project.clear_all_track_content();
        app.project.tracks[0].routing.input_port =
            TrackPortSelection::named(MidiPortRef::new("Test Input"));
        app.project.tracks[0].midi_fx.input_fx = vec![None; MIDI_FX_SLOT_COUNT];
        app.project.tracks[1].state.passthrough = false;
        app.project.tracks[1].midi_fx.monitor_input_fx = true;
        app.project.tracks[1].routing.output_port =
            TrackPortSelection::named(MidiPortRef::new("Out B"));
        app.project.tracks[1].routing.output_channel = Some(2);
        app.project.tracks[1].routing.input_port =
            TrackPortSelection::named(MidiPortRef::new("Test Input"));
        app.project.tracks[1].routing.input_channel = MidiChannelFilter::Channel(1);
        app.project.tracks[1].midi_fx.input_fx[0] = Some(MidiFxSlot {
            enabled: true,
            effect: MidiFx::TrackClone { source_track: 0 },
        });
        app.project.tracks[1].midi_fx.input_fx[1] = Some(MidiFxSlot {
            enabled: true,
            effect: MidiFx::Transpose { semitones: 12 },
        });
        app.project.tracks[1].midi_fx.output_fx = vec![None; MIDI_FX_SLOT_COUNT];

        let input_port = app.project.tracks[0]
            .routing
            .input_port
            .as_named_port()
            .cloned()
            .unwrap();
        app.inject_midi_input_event(MidiInputEvent {
            port: input_port.clone(),
            channel: 1,
            message: MidiInputMessage::NoteOn {
                pitch: 60,
                velocity: 100,
            },

            received_at: std::time::Instant::now(),
            backend_timestamp_micros: None,
            sequence: 0,
        });
        let _sent = wait_for_sent_messages(&mut app, runtime_test_timeout(), |sent| {
            sent.iter().any(|(port, channel, pitch, velocity)| {
                port == "Out B" && *channel == 2 && *pitch == 72 && velocity.is_some()
            })
        });
        app.inject_midi_input_event(MidiInputEvent {
            port: input_port,
            channel: 1,
            message: MidiInputMessage::NoteOff { pitch: 60 },

            received_at: std::time::Instant::now(),
            backend_timestamp_micros: None,
            sequence: 0,
        });

        let sent = wait_for_sent_messages(&mut app, runtime_test_timeout(), |sent| {
            sent.iter().any(|(port, channel, pitch, velocity)| {
                port == "Out B" && *channel == 2 && *pitch == 72 && velocity.is_some()
            })
        });
        assert!(
            sent.iter()
                .any(|(port, channel, pitch, velocity)| port == "Out B"
                    && *channel == 2
                    && *pitch == 72
                    && velocity.is_some())
        );
        assert!(
            !sent
                .iter()
                .any(|(port, channel, pitch, velocity)| port == "Out B"
                    && *channel == 2
                    && *pitch == 60
                    && velocity.is_some())
        );
    }

    #[test]
    fn track_clone_uses_recorded_source_midi_for_playback_stream() {
        let mut app = App::new();
        app.project.clear_all_track_content();
        app.project.tracks[0]
            .midi_notes
            .push(MidiNote::new(48, 0, 480, 100));
        app.project.tracks[1].midi_fx.input_fx = vec![None; MIDI_FX_SLOT_COUNT];
        app.project.tracks[1].midi_fx.input_fx[0] = Some(MidiFxSlot {
            enabled: true,
            effect: MidiFx::TrackClone { source_track: 0 },
        });
        app.project.tracks[1].midi_fx.input_fx[1] = Some(MidiFxSlot {
            enabled: true,
            effect: MidiFx::Transpose { semitones: 12 },
        });
        app.project.tracks[1].midi_fx.output_fx = vec![None; MIDI_FX_SLOT_COUNT];

        let notes = app.effective_track_output_notes(1);
        assert!(notes.iter().any(|note| note.pitch == 60));
    }

    #[test]
    fn track_clone_records_into_post_input_fx_take() {
        let mut app = App::new();
        app.project.clear_all_track_content();
        app.project.select_track(1);
        app.project.tracks[0].routing.input_port =
            TrackPortSelection::named(MidiPortRef::new("Test Input"));
        app.project.tracks[0].midi_fx.input_fx = vec![None; MIDI_FX_SLOT_COUNT];
        app.project.tracks[1].state.armed = true;
        app.project.tracks[1].midi_fx.record_input_fx_mode =
            crate::midi_fx::RecordInputFxMode::PostInputFx;
        app.project.tracks[1].midi_fx.input_fx[0] = Some(MidiFxSlot {
            enabled: true,
            effect: MidiFx::TrackClone { source_track: 0 },
        });
        app.project.tracks[1].midi_fx.input_fx[1] = Some(MidiFxSlot {
            enabled: true,
            effect: MidiFx::Transpose { semitones: 12 },
        });
        app.transport_ticks = 0;
        app.playhead_ticks = 0;

        app.apply_action(AppAction::ToggleRecording);
        assert!(app.project.tracks[1].active_take.is_some());
        let input_port = app.project.tracks[0]
            .routing
            .input_port
            .as_named_port()
            .cloned()
            .unwrap();
        app.inject_midi_input_event(MidiInputEvent {
            port: input_port.clone(),
            channel: 1,
            message: MidiInputMessage::NoteOn {
                pitch: 60,
                velocity: 100,
            },

            received_at: std::time::Instant::now(),
            backend_timestamp_micros: None,
            sequence: 0,
        });
        app.transport_ticks = 960;
        app.playhead_ticks = 960;
        app.inject_midi_input_event(MidiInputEvent {
            port: input_port,
            channel: 1,
            message: MidiInputMessage::NoteOff { pitch: 60 },

            received_at: std::time::Instant::now(),
            backend_timestamp_micros: None,
            sequence: 0,
        });
        app.apply_action(AppAction::ToggleRecording);

        let target = &app.project.tracks[1];
        assert!(target.midi_notes.iter().any(|note| note.pitch == 72));
    }

    #[test]
    fn track_clone_records_playback_source_midi_into_post_input_fx_take() {
        let mut app = App::new();
        app.project.clear_all_track_content();
        app.project.select_track(1);
        app.project.tracks[0]
            .midi_notes
            .push(MidiNote::new(60, 0, 480, 100));
        app.project.tracks[1].state.armed = true;
        app.project.tracks[1].midi_fx.record_input_fx_mode =
            crate::midi_fx::RecordInputFxMode::PostInputFx;
        app.project.tracks[1].midi_fx.input_fx[0] = Some(MidiFxSlot {
            enabled: true,
            effect: MidiFx::TrackClone { source_track: 0 },
        });
        app.project.tracks[1].midi_fx.input_fx[1] = Some(MidiFxSlot {
            enabled: true,
            effect: MidiFx::Transpose { semitones: 12 },
        });
        app.transport_ticks = 0;
        app.playhead_ticks = 0;

        app.apply_action(AppAction::ToggleRecording);
        assert!(app.project.tracks[1].active_take.is_some());
        app.dispatch_midi_notes(0, 960);
        app.transport_ticks = 960;
        app.playhead_ticks = 960;
        app.apply_action(AppAction::ToggleRecording);

        let target = &app.project.tracks[1];
        assert!(target.midi_notes.iter().any(|note| note.pitch == 72));
    }

    #[test]
    fn active_recording_preserves_live_passthrough_across_multiple_notes() {
        let _guard = runtime_test_guard();
        let mut app = App::new();
        app.project.clear_all_track_content();
        app.project.select_track(0);
        app.project.tracks[0].state.armed = true;
        app.project.tracks[0].state.passthrough = true;
        app.project.tracks[0].midi_fx.monitor_input_fx = true;
        app.project.tracks[0].routing.input_port =
            TrackPortSelection::named(MidiPortRef::new("In A"));
        app.project.tracks[0].routing.output_port =
            TrackPortSelection::named(MidiPortRef::new("Out A"));
        app.project.tracks[0].routing.output_channel = Some(1);
        app.project.tracks[0].midi_fx.input_fx = vec![None; MIDI_FX_SLOT_COUNT];
        app.project.tracks[0].midi_fx.output_fx = vec![None; MIDI_FX_SLOT_COUNT];

        app.apply_action(AppAction::ToggleRecording);
        assert!(app.project.tracks[0].active_take.is_some());

        let input_port = app.project.tracks[0]
            .routing
            .input_port
            .as_named_port()
            .cloned()
            .unwrap();

        app.inject_midi_input_event(MidiInputEvent {
            port: input_port.clone(),
            channel: 1,
            message: MidiInputMessage::NoteOn {
                pitch: 60,
                velocity: 100,
            },
            received_at: std::time::Instant::now(),
            backend_timestamp_micros: None,
            sequence: 0,
        });
        let _sent = wait_for_sent_messages(&mut app, runtime_test_timeout(), |sent| {
            sent.iter().any(|(port, channel, pitch, velocity)| {
                port == "Out A" && *channel == 1 && *pitch == 60 && velocity.is_some()
            })
        });

        app.transport_ticks = 120;
        app.playhead_ticks = 120;
        app.inject_midi_input_event(MidiInputEvent {
            port: input_port.clone(),
            channel: 1,
            message: MidiInputMessage::NoteOff { pitch: 60 },
            received_at: std::time::Instant::now(),
            backend_timestamp_micros: None,
            sequence: 0,
        });

        app.transport_ticks = 240;
        app.playhead_ticks = 240;
        app.inject_midi_input_event(MidiInputEvent {
            port: input_port.clone(),
            channel: 1,
            message: MidiInputMessage::NoteOn {
                pitch: 64,
                velocity: 96,
            },
            received_at: std::time::Instant::now(),
            backend_timestamp_micros: None,
            sequence: 0,
        });
        let sent = wait_for_sent_messages(&mut app, runtime_test_timeout(), |sent| {
            sent.iter()
                .filter(|(port, channel, _pitch, _velocity)| port == "Out A" && *channel == 1)
                .count()
                >= 3
        });
        assert!(sent.iter().any(|(port, channel, pitch, velocity)| {
            port == "Out A" && *channel == 1 && *pitch == 64 && *velocity == Some(96)
        }));

        app.transport_ticks = 360;
        app.playhead_ticks = 360;
        app.inject_midi_input_event(MidiInputEvent {
            port: input_port,
            channel: 1,
            message: MidiInputMessage::NoteOff { pitch: 64 },
            received_at: std::time::Instant::now(),
            backend_timestamp_micros: None,
            sequence: 0,
        });

        let sent = wait_for_sent_messages(&mut app, runtime_test_timeout(), |sent| {
            sent.iter()
                .filter(|(port, channel, _pitch, _velocity)| port == "Out A" && *channel == 1)
                .count()
                >= 4
        });
        assert_eq!(
            sent.iter()
                .filter(|(port, channel, pitch, velocity)| {
                    port == "Out A" && *channel == 1 && *pitch == 60 && velocity.is_some()
                })
                .count(),
            1
        );
        assert_eq!(
            sent.iter()
                .filter(|(port, channel, pitch, velocity)| {
                    port == "Out A" && *channel == 1 && *pitch == 60 && velocity.is_none()
                })
                .count(),
            1
        );
        assert_eq!(
            sent.iter()
                .filter(|(port, channel, pitch, velocity)| {
                    port == "Out A" && *channel == 1 && *pitch == 64 && velocity.is_some()
                })
                .count(),
            1
        );
        assert_eq!(
            sent.iter()
                .filter(|(port, channel, pitch, velocity)| {
                    port == "Out A" && *channel == 1 && *pitch == 64 && velocity.is_none()
                })
                .count(),
            1
        );

        app.transport_ticks = 480;
        app.playhead_ticks = 480;
        app.apply_action(AppAction::ToggleRecording);

        let target = &app.project.tracks[0];
        assert!(target.active_take.is_none());
        assert!(target.midi_notes.iter().any(|note| note.pitch == 60));
        assert!(target.midi_notes.iter().any(|note| note.pitch == 64));
    }

    #[test]
    fn post_input_fx_recording_matches_live_passthrough_output() {
        let _guard = runtime_test_guard();
        let mut app = App::new();
        app.project.clear_all_track_content();
        app.project.select_track(0);
        app.project.tracks[0].state.armed = true;
        app.project.tracks[0].state.passthrough = true;
        app.project.tracks[0].midi_fx.monitor_input_fx = true;
        app.project.tracks[0].midi_fx.record_input_fx_mode =
            crate::midi_fx::RecordInputFxMode::PostInputFx;
        app.project.tracks[0].routing.input_port =
            TrackPortSelection::named(MidiPortRef::new("In A"));
        app.project.tracks[0].routing.output_port =
            TrackPortSelection::named(MidiPortRef::new("Out A"));
        app.project.tracks[0].routing.output_channel = Some(1);
        app.project.tracks[0].midi_fx.input_fx = vec![None; MIDI_FX_SLOT_COUNT];
        app.project.tracks[0].midi_fx.output_fx = vec![None; MIDI_FX_SLOT_COUNT];
        app.project.tracks[0].midi_fx.input_fx[0] = Some(MidiFxSlot {
            enabled: true,
            effect: MidiFx::Transpose { semitones: 12 },
        });

        app.apply_action(AppAction::ToggleRecording);
        assert!(app.project.tracks[0].active_take.is_some());

        let input_port = app.project.tracks[0]
            .routing
            .input_port
            .as_named_port()
            .cloned()
            .unwrap();
        app.inject_midi_input_event(MidiInputEvent {
            port: input_port.clone(),
            channel: 1,
            message: MidiInputMessage::NoteOn {
                pitch: 60,
                velocity: 100,
            },
            received_at: std::time::Instant::now(),
            backend_timestamp_micros: None,
            sequence: 0,
        });
        app.transport_ticks = 120;
        app.playhead_ticks = 120;
        app.inject_midi_input_event(MidiInputEvent {
            port: input_port,
            channel: 1,
            message: MidiInputMessage::NoteOff { pitch: 60 },
            received_at: std::time::Instant::now(),
            backend_timestamp_micros: None,
            sequence: 0,
        });

        let sent = wait_for_sent_messages(&mut app, runtime_test_timeout(), |sent| {
            sent.iter().any(|(port, channel, pitch, velocity)| {
                port == "Out A" && *channel == 1 && *pitch == 72 && velocity.is_some()
            }) && sent.iter().any(|(port, channel, pitch, velocity)| {
                port == "Out A" && *channel == 1 && *pitch == 72 && velocity.is_none()
            })
        });
        assert_eq!(
            sent.iter()
                .filter(|(port, channel, pitch, velocity)| {
                    port == "Out A" && *channel == 1 && *pitch == 72 && velocity.is_some()
                })
                .count(),
            1
        );
        assert_eq!(
            sent.iter()
                .filter(|(port, channel, pitch, velocity)| {
                    port == "Out A" && *channel == 1 && *pitch == 72 && velocity.is_none()
                })
                .count(),
            1
        );

        app.transport_ticks = 240;
        app.playhead_ticks = 240;
        app.apply_action(AppAction::ToggleRecording);

        let target = &app.project.tracks[0];
        assert!(target.midi_notes.iter().any(|note| note.pitch == 72));
        assert!(
            !target.midi_notes.iter().any(|note| note.pitch == 60),
            "expected committed recording to match post-input-fx stream"
        );
    }

    #[test]
    fn track_clone_follows_source_track_loop_phase() {
        let mut app = App::new();
        app.project.clear_all_track_content();
        app.project.transport.loop_enabled = false;
        app.project.tracks[0].state.loop_enabled = true;
        app.project.tracks[0].loop_region = crate::timeline::LoopRegion::new(960, 960);
        app.project.tracks[0]
            .midi_notes
            .push(MidiNote::new(60, 960, 480, 100));
        app.project.tracks[1].routing.output_port =
            TrackPortSelection::named(MidiPortRef::new("Out B"));
        app.project.tracks[1].routing.output_channel = Some(2);
        app.project.tracks[1].midi_fx.input_fx[0] = Some(MidiFxSlot {
            enabled: true,
            effect: MidiFx::TrackClone { source_track: 0 },
        });

        app.dispatch_midi_notes(1_920, 960);

        let sent = app.midi_output.sent_messages();
        assert!(
            sent.iter()
                .any(|(port, channel, pitch, velocity)| port == "Out B"
                    && *channel == 2
                    && *pitch == 60
                    && velocity.is_some())
        );
    }

    #[test]
    fn live_input_arp_passthrough_emits_timed_notes() {
        let _guard = runtime_test_guard();
        let mut app = App::new();
        app.project.clear_all_track_content();
        app.project.tracks[0].routing.input_port =
            TrackPortSelection::named(MidiPortRef::new("Test Input"));
        app.project.tracks[0].state.passthrough = true;
        app.project.tracks[0].routing.output_port =
            TrackPortSelection::named(MidiPortRef::new("Out A"));
        app.project.tracks[0].routing.output_channel = Some(1);
        app.project.tracks[0].midi_fx.input_fx[0] = Some(MidiFxSlot {
            enabled: true,
            effect: MidiFx::Arp {
                step_ticks: 240,
                order: crate::midi_fx::ArpOrder::Up,
                gate_percent: 100,
            },
        });

        let input_port = app.project.tracks[0]
            .routing
            .input_port
            .as_named_port()
            .cloned()
            .unwrap();
        app.inject_midi_input_event(MidiInputEvent {
            port: input_port.clone(),
            channel: 1,
            message: MidiInputMessage::NoteOn {
                pitch: 60,
                velocity: 100,
            },

            received_at: std::time::Instant::now(),
            backend_timestamp_micros: None,
            sequence: 0,
        });
        app.inject_midi_input_event(MidiInputEvent {
            port: input_port,
            channel: 1,
            message: MidiInputMessage::NoteOn {
                pitch: 64,
                velocity: 100,
            },

            received_at: std::time::Instant::now(),
            backend_timestamp_micros: None,
            sequence: 0,
        });

        let sent = wait_for_sent_messages(&mut app, runtime_test_timeout(), |sent| {
            sent.iter()
                .filter(|(port, channel, _pitch, velocity)| {
                    port == "Out A" && *channel == 1 && velocity.is_some()
                })
                .count()
                >= 2
        });
        assert!(
            sent.iter()
                .filter(|(port, channel, _pitch, velocity)| {
                    port == "Out A" && *channel == 1 && velocity.is_some()
                })
                .count()
                >= 2
        );
    }

    #[test]
    fn playback_output_delay_emits_note_off_after_source_note_window() {
        let mut app = App::new();
        app.project.clear_all_track_content();
        app.project.tracks[0].routing.output_port =
            TrackPortSelection::named(MidiPortRef::new("Out A"));
        app.project.tracks[0].routing.output_channel = Some(1);
        app.project.tracks[0]
            .midi_notes
            .push(MidiNote::new(60, 0, 240, 100));
        app.project.tracks[0].midi_fx.output_fx = vec![None; MIDI_FX_SLOT_COUNT];
        app.project.tracks[0].midi_fx.output_fx[0] = Some(MidiFxSlot {
            enabled: true,
            effect: MidiFx::Delay { ticks: 60 },
        });

        app.dispatch_midi_notes(0, 60);
        assert!(app.midi_output.sent_messages().is_empty());

        app.dispatch_midi_notes(60, 60);
        assert_eq!(
            app.midi_output.sent_messages(),
            vec![("Out A".to_string(), 1, 60, Some(100))]
        );

        app.dispatch_midi_notes(120, 60);
        app.dispatch_midi_notes(180, 60);
        app.dispatch_midi_notes(240, 60);
        assert_eq!(
            app.midi_output.sent_messages(),
            vec![("Out A".to_string(), 1, 60, Some(100))]
        );

        app.dispatch_midi_notes(300, 60);
        assert_eq!(
            app.midi_output.sent_messages(),
            vec![
                ("Out A".to_string(), 1, 60, Some(100)),
                ("Out A".to_string(), 1, 60, None),
            ]
        );
    }

    #[test]
    fn playback_output_delay_releases_notes_for_repeated_pattern_across_small_windows() {
        let mut app = App::new();
        app.project.clear_all_track_content();
        app.project.tracks[0].routing.output_port =
            TrackPortSelection::named(MidiPortRef::new("Out A"));
        app.project.tracks[0].routing.output_channel = Some(1);
        app.project.tracks[0].midi_notes = vec![
            MidiNote::new(60, 0, 240, 100),
            MidiNote::new(64, 480, 240, 96),
        ];
        app.project.tracks[0].midi_fx.output_fx = vec![None; MIDI_FX_SLOT_COUNT];
        app.project.tracks[0].midi_fx.output_fx[0] = Some(MidiFxSlot {
            enabled: true,
            effect: MidiFx::Delay { ticks: 60 },
        });

        for start in [
            0_u64, 60, 120, 180, 240, 300, 360, 420, 480, 540, 600, 660, 720, 780,
        ] {
            app.dispatch_midi_notes(start, 60);
        }

        let sent = app.midi_output.sent_messages();
        for (pitch, velocity) in [(60, Some(100)), (64, Some(96))] {
            assert_eq!(
                sent.iter()
                    .filter(|(port, channel, event_pitch, event_velocity)| {
                        port == "Out A"
                            && *channel == 1
                            && *event_pitch == pitch
                            && *event_velocity == velocity
                    })
                    .count(),
                1
            );
            assert_eq!(
                sent.iter()
                    .filter(|(port, channel, event_pitch, event_velocity)| {
                        port == "Out A"
                            && *channel == 1
                            && *event_pitch == pitch
                            && event_velocity.is_none()
                    })
                    .count(),
                1
            );
        }
    }

    #[test]
    fn playback_output_duration_releases_note_after_extended_length_window() {
        let mut app = App::new();
        app.project.clear_all_track_content();
        app.project.tracks[0].routing.output_port =
            TrackPortSelection::named(MidiPortRef::new("Out A"));
        app.project.tracks[0].routing.output_channel = Some(1);
        app.project.tracks[0]
            .midi_notes
            .push(MidiNote::new(60, 0, 60, 100));
        app.project.tracks[0].midi_fx.output_fx = vec![None; MIDI_FX_SLOT_COUNT];
        app.project.tracks[0].midi_fx.output_fx[0] = Some(MidiFxSlot {
            enabled: true,
            effect: MidiFx::Duration { ticks: 240 },
        });

        app.dispatch_midi_notes(0, 60);
        assert_eq!(
            app.midi_output.sent_messages(),
            vec![("Out A".to_string(), 1, 60, Some(100))]
        );

        app.dispatch_midi_notes(60, 60);
        app.dispatch_midi_notes(120, 60);
        app.dispatch_midi_notes(180, 60);
        assert_eq!(
            app.midi_output.sent_messages(),
            vec![("Out A".to_string(), 1, 60, Some(100))]
        );

        app.dispatch_midi_notes(240, 60);
        assert_eq!(
            app.midi_output.sent_messages(),
            vec![
                ("Out A".to_string(), 1, 60, Some(100)),
                ("Out A".to_string(), 1, 60, None),
            ]
        );
    }

    #[test]
    fn stopped_live_arp_ticks_without_advancing_playhead() {
        let _guard = runtime_test_guard();
        let mut app = App::new();
        app.project.clear_all_track_content();
        app.project.tracks[0].routing.input_port =
            TrackPortSelection::named(MidiPortRef::new("In A"));
        app.project.tracks[0].routing.output_port =
            TrackPortSelection::named(MidiPortRef::new("Out A"));
        app.project.tracks[0].routing.output_channel = Some(1);
        app.project.tracks[0].state.passthrough = true;
        app.project.tracks[0].midi_fx.monitor_input_fx = true;
        app.project.tracks[0].midi_fx.input_fx = vec![None; MIDI_FX_SLOT_COUNT];
        app.project.tracks[0].midi_fx.input_fx[0] = Some(MidiFxSlot {
            enabled: true,
            effect: MidiFx::Arp {
                step_ticks: 240,
                order: crate::midi_fx::ArpOrder::Up,
                gate_percent: 100,
            },
        });

        let input_port = app.project.tracks[0]
            .routing
            .input_port
            .as_named_port()
            .cloned()
            .unwrap();
        app.inject_midi_input_event(MidiInputEvent {
            port: input_port.clone(),
            channel: 1,
            message: MidiInputMessage::NoteOn {
                pitch: 60,
                velocity: 100,
            },

            received_at: std::time::Instant::now(),
            backend_timestamp_micros: None,
            sequence: 0,
        });
        app.inject_midi_input_event(MidiInputEvent {
            port: input_port,
            channel: 1,
            message: MidiInputMessage::NoteOn {
                pitch: 64,
                velocity: 100,
            },

            received_at: std::time::Instant::now(),
            backend_timestamp_micros: None,
            sequence: 0,
        });

        assert_eq!(app.transport_ticks, 0);
        assert_eq!(app.playhead_ticks, 0);
        let started_at = std::time::Instant::now();
        let sent = loop {
            app.wait_for_midi_runtime();
            let sent = app.midi_output.sent_messages();
            if sent.iter().any(|(port, channel, _pitch, velocity)| {
                port == "Out A" && *channel == 1 && velocity.is_some()
            }) {
                break sent;
            }
            assert!(
                started_at.elapsed() < runtime_test_timeout(),
                "expected stopped live arp runtime to emit a note within 1s, got {:?}",
                sent
            );
            std::thread::sleep(Duration::from_millis(20));
        };
        assert!(sent.iter().any(|(port, channel, _pitch, velocity)| {
            port == "Out A" && *channel == 1 && velocity.is_some()
        }));
    }
}
