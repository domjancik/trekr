use crate::midi_fx::{
    LiveMidiFxEvent, LiveMidiFxState, MidiFx, playback_timing_lookback_ticks,
    process_live_chain_tick, reset_live_fx_timing, transform_notes,
};
use crate::midi_io::{MidiEventPriority, MidiOutputRuntime, MidiPortRef};
use crate::project::{MidiNote, Project, Track};
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use super::super::note_runtime::{occurrence_note_events, scheduled_note_occurrences};

const PLAYBACK_RUNTIME_POLL_INTERVAL: Duration = Duration::from_millis(1);

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MidiRuntimeState {
    pub project: Project,
    pub transport_ticks: u64,
    pub playhead_ticks: u64,
    pub default_output_port: Option<MidiPortRef>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct MidiRuntimeSnapshot {
    pub transport_ticks: u64,
    pub playhead_ticks: u64,
}

enum MidiRuntimeCommand {
    SyncState(MidiRuntimeState),
    Shutdown,
}

pub(crate) struct MidiRuntime {
    command_sender: Sender<MidiRuntimeCommand>,
    latest_snapshot: Arc<Mutex<MidiRuntimeSnapshot>>,
}

impl MidiRuntime {
    pub(crate) fn start_if_enabled(initial_state: MidiRuntimeState) -> Option<Self> {
        if cfg!(test) {
            return None;
        }
        if std::env::var("TREKR_MIDI_RUNTIME")
            .ok()
            .is_some_and(|value| value == "0")
        {
            return None;
        }
        Some(Self::start(initial_state))
    }

    #[cfg(test)]
    pub(in crate::app) fn from_snapshot_for_test(snapshot: MidiRuntimeSnapshot) -> Self {
        let (command_sender, _command_receiver) = mpsc::channel();
        Self {
            command_sender,
            latest_snapshot: Arc::new(Mutex::new(snapshot)),
        }
    }

    fn start(initial_state: MidiRuntimeState) -> Self {
        let (command_sender, command_receiver) = mpsc::channel();
        let latest_snapshot = Arc::new(Mutex::new(MidiRuntimeSnapshot {
            transport_ticks: initial_state.transport_ticks,
            playhead_ticks: initial_state.playhead_ticks,
        }));
        let thread_snapshot = Arc::clone(&latest_snapshot);
        thread::Builder::new()
            .name("trekr-midi-runtime".to_string())
            .spawn(move || {
                let mut runtime = MidiRuntimeWorker::new(initial_state, thread_snapshot);
                runtime.run(command_receiver);
            })
            .expect("midi runtime should start");

        Self {
            command_sender,
            latest_snapshot,
        }
    }

    pub(crate) fn sync_state(&self, state: MidiRuntimeState) {
        let _ = self
            .command_sender
            .send(MidiRuntimeCommand::SyncState(state));
    }

    pub(crate) fn latest_snapshot(&self) -> MidiRuntimeSnapshot {
        self.latest_snapshot
            .lock()
            .map(|snapshot| *snapshot)
            .unwrap_or_default()
    }
}

impl Drop for MidiRuntime {
    fn drop(&mut self) {
        let _ = self.command_sender.send(MidiRuntimeCommand::Shutdown);
    }
}

struct MidiRuntimeWorker {
    project: Project,
    default_output_port: Option<MidiPortRef>,
    transport_ticks: u64,
    playhead_ticks: u64,
    output_fx_live_states: Vec<LiveMidiFxState>,
    midi_output: MidiOutputRuntime,
    latest_snapshot: Arc<Mutex<MidiRuntimeSnapshot>>,
    last_tick_at: Instant,
    last_poll_wake_at: Instant,
}

impl MidiRuntimeWorker {
    fn new(
        initial_state: MidiRuntimeState,
        latest_snapshot: Arc<Mutex<MidiRuntimeSnapshot>>,
    ) -> Self {
        let mut worker = Self {
            output_fx_live_states: vec![
                LiveMidiFxState::default();
                initial_state.project.tracks.len()
            ],
            project: initial_state.project,
            default_output_port: initial_state.default_output_port,
            transport_ticks: initial_state.transport_ticks,
            playhead_ticks: initial_state.playhead_ticks,
            midi_output: MidiOutputRuntime::default(),
            latest_snapshot,
            last_tick_at: Instant::now(),
            last_poll_wake_at: Instant::now(),
        };
        worker.prewarm_outputs();
        worker
    }

    fn run(&mut self, command_receiver: Receiver<MidiRuntimeCommand>) {
        loop {
            let wake_now = Instant::now();
            log_runtime(format!(
                "wake delta_ms={} playing={} transport_ticks={} playhead_ticks={}",
                wake_now
                    .saturating_duration_since(self.last_poll_wake_at)
                    .as_millis(),
                self.project.transport.playing,
                self.transport_ticks,
                self.playhead_ticks
            ));
            self.last_poll_wake_at = wake_now;

            while let Ok(command) = command_receiver.try_recv() {
                if self.apply_command(command) {
                    return;
                }
            }

            self.tick();

            match command_receiver.recv_timeout(PLAYBACK_RUNTIME_POLL_INTERVAL) {
                Ok(command) => {
                    if self.apply_command(command) {
                        return;
                    }
                }
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => return,
            }
        }
    }

    fn apply_command(&mut self, command: MidiRuntimeCommand) -> bool {
        match command {
            MidiRuntimeCommand::SyncState(state) => {
                let was_active = self.background_playback_active();
                let previous_transport_ticks = self.transport_ticks;
                let next_active = state.project.transport.playing
                    && !state.project.transport.recording
                    && !state.project.transport.link_enabled;
                let should_seek = !was_active
                    || !next_active
                    || runtime_seek_requested(
                        previous_transport_ticks,
                        state.transport_ticks,
                        state.project.transport.ppqn,
                    );
                let playback_shape_changed = self.project != state.project
                    || self.default_output_port != state.default_output_port;

                self.project = state.project;
                self.default_output_port = state.default_output_port;
                self.ensure_output_fx_live_state_len();
                if should_seek {
                    self.transport_ticks = state.transport_ticks;
                    self.playhead_ticks = state.playhead_ticks;
                    self.reset_output_fx_timing(self.transport_ticks);
                } else {
                    self.playhead_ticks = self.song_playhead_for_transport(self.transport_ticks);
                }
                self.prewarm_outputs();
                if was_active && (!next_active || should_seek || playback_shape_changed) {
                    self.silence_all_tracks();
                }
                if playback_shape_changed {
                    self.reset_output_fx_timing(self.transport_ticks);
                }
                if should_seek {
                    self.last_tick_at = Instant::now();
                }
                self.publish_snapshot();
                log_runtime(format!(
                    "sync active={} seek={} shape_changed={} transport_ticks={} playhead_ticks={}",
                    next_active,
                    should_seek,
                    playback_shape_changed,
                    self.transport_ticks,
                    self.playhead_ticks
                ));
                false
            }
            MidiRuntimeCommand::Shutdown => true,
        }
    }

    fn tick(&mut self) {
        if !self.background_playback_active() {
            self.publish_snapshot();
            return;
        }

        let now = Instant::now();
        let delta = now.saturating_duration_since(self.last_tick_at);
        self.last_tick_at = now;

        let ticks_per_second = self.project.transport.ticks_per_second();
        let advanced_ticks =
            (delta.as_nanos() as u128 * u128::from(ticks_per_second)) / 1_000_000_000_u128;
        if advanced_ticks == 0 {
            return;
        }

        let previous_ticks = self.transport_ticks;
        let advanced_ticks = advanced_ticks as u64;
        let window_started_at = now.checked_sub(delta).unwrap_or(now);
        self.transport_ticks = self.transport_ticks.saturating_add(advanced_ticks);
        self.playhead_ticks = self.song_playhead_for_transport(self.transport_ticks);
        self.process_queued_stored_loop_recalls(previous_ticks, self.transport_ticks);
        self.dispatch_midi_notes(previous_ticks, advanced_ticks, window_started_at, now);
        self.dispatch_live_output_events(
            previous_ticks,
            self.transport_ticks,
            window_started_at,
            now,
        );
        self.publish_snapshot();
    }

    fn background_playback_active(&self) -> bool {
        self.project.transport.playing
            && !self.project.transport.recording
            && !self.project.transport.link_enabled
    }

    fn publish_snapshot(&self) {
        if let Ok(mut snapshot) = self.latest_snapshot.lock() {
            *snapshot = MidiRuntimeSnapshot {
                transport_ticks: self.transport_ticks,
                playhead_ticks: self.playhead_ticks,
            };
        }
    }

    fn prewarm_outputs(&mut self) {
        let mut ports = Vec::new();
        if let Some(port) = self.default_output_port.clone() {
            ports.push(port);
        }
        for track in &self.project.tracks {
            if let Some(port) = track
                .routing
                .output_port
                .cloned_resolved(self.default_output_port.as_ref())
            {
                if !ports.iter().any(|existing| existing == &port) {
                    ports.push(port);
                }
            }
        }
        for port in ports {
            let _ = self.midi_output.prewarm_port(&port);
        }
    }

    fn song_playhead_for_transport(&self, transport_ticks: u64) -> u64 {
        if !self.project.transport.loop_enabled || self.project.loop_region.length_ticks == 0 {
            return transport_ticks;
        }

        let loop_region = self.project.loop_region;
        let relative = transport_ticks.saturating_sub(loop_region.start_ticks);
        loop_region.start_ticks + (relative % loop_region.length_ticks.max(1))
    }

    fn process_queued_stored_loop_recalls(
        &mut self,
        previous_transport_ticks: u64,
        current_transport_ticks: u64,
    ) {
        if current_transport_ticks <= previous_transport_ticks {
            return;
        }
        let ppqn = self.project.transport.ppqn;
        for track in &mut self.project.tracks {
            if track.active_take.is_some() {
                continue;
            }
            let _ = track.resolve_queued_stored_loop_recall_if_due(
                previous_transport_ticks,
                current_transport_ticks,
                ppqn,
            );
        }
    }

    fn ensure_output_fx_live_state_len(&mut self) {
        let track_count = self.project.tracks.len();
        if self.output_fx_live_states.len() < track_count {
            self.output_fx_live_states
                .resize(track_count, LiveMidiFxState::default());
        }
    }

    fn reset_output_fx_timing(&mut self, current_ticks: u64) {
        self.ensure_output_fx_live_state_len();
        for state in &mut self.output_fx_live_states {
            reset_live_fx_timing(state, current_ticks);
        }
    }

    fn silence_all_tracks(&mut self) {
        let ports_and_channels: Vec<(MidiPortRef, u8)> = self
            .project
            .tracks
            .iter()
            .filter_map(|track| {
                track
                    .routing
                    .output_port
                    .cloned_resolved(self.default_output_port.as_ref())
                    .zip(track.routing.output_channel)
            })
            .collect();

        for (port, channel) in ports_and_channels {
            let _ = self.midi_output.send_all_notes_off(&port, channel);
        }
    }

    fn record_context(&self, track: &Track) -> Option<crate::project::RecordContext> {
        if track.state.loop_enabled {
            Some(crate::project::RecordContext {
                range: track.loop_region,
                wrap_basis_ticks: 0,
                extend_clip_on_wrap: self.project.transport.loop_recording_extends_clip,
            })
        } else if self.project.transport.loop_enabled {
            Some(crate::project::RecordContext {
                range: self.project.loop_region,
                wrap_basis_ticks: self.project.loop_region.start_ticks,
                extend_clip_on_wrap: self.project.transport.loop_recording_extends_clip,
            })
        } else {
            None
        }
    }

    fn playback_loop_range_for_track(&self, track: &Track) -> Option<crate::timeline::LoopRegion> {
        if track.state.loop_enabled {
            Some(track.loop_region)
        } else {
            self.project
                .transport
                .loop_enabled
                .then_some(self.project.loop_region)
        }
    }

    fn track_emits_clone_source(&self, source_track_index: usize) -> bool {
        let Some(source) = self.project.tracks.get(source_track_index) else {
            return false;
        };
        if source.state.muted {
            return false;
        }
        let any_solo = self.project.tracks.iter().any(|track| track.state.soloed);
        !any_solo || source.state.soloed
    }

    fn effective_track_clone_playback_notes_recursive(
        &self,
        track_index: usize,
        previous_ticks: u64,
        advanced_ticks: u64,
        visited: &mut [bool],
    ) -> Vec<MidiNote> {
        let Some(track) = self.project.tracks.get(track_index) else {
            return Vec::new();
        };
        let mut notes = Vec::new();
        for slot in track
            .midi_fx
            .input_fx
            .iter()
            .flatten()
            .filter(|slot| slot.enabled)
        {
            let MidiFx::TrackClone { source_track } = slot.effect else {
                notes = transform_notes(
                    &notes,
                    &[Some(slot.clone())],
                    self.project.global_harmony.root,
                );
                continue;
            };
            if source_track == track_index || !self.track_emits_clone_source(source_track) {
                continue;
            }
            notes.extend(self.effective_track_pre_output_playback_notes_recursive(
                source_track,
                previous_ticks,
                advanced_ticks,
                visited,
            ));
        }
        notes
    }

    fn effective_track_pre_output_playback_notes_recursive(
        &self,
        track_index: usize,
        previous_ticks: u64,
        advanced_ticks: u64,
        visited: &mut [bool],
    ) -> Vec<MidiNote> {
        let Some(track) = self.project.tracks.get(track_index) else {
            return Vec::new();
        };
        if visited.get(track_index).copied().unwrap_or(true) {
            return Vec::new();
        }
        visited[track_index] = true;
        let native_notes = scheduled_note_occurrences(
            track,
            &track.midi_notes,
            previous_ticks,
            advanced_ticks,
            self.playback_loop_range_for_track(track),
        );
        let cloned_notes = self.effective_track_clone_playback_notes_recursive(
            track_index,
            previous_ticks,
            advanced_ticks,
            visited,
        );
        let mut notes = native_notes;
        notes.extend(cloned_notes);
        visited[track_index] = false;
        notes.sort_by_key(|note| (note.start_ticks, note.pitch, note.length_ticks));
        notes
    }

    fn dispatch_midi_notes(
        &mut self,
        previous_ticks: u64,
        advanced_ticks: u64,
        window_started_at: Instant,
        window_ended_at: Instant,
    ) {
        if advanced_ticks == 0 {
            return;
        }

        let track_events: Vec<(Option<MidiPortRef>, u8, Vec<(u64, bool, u8, u8)>)> = self
            .project
            .tracks
            .iter()
            .enumerate()
            .map(|(track_index, track)| {
                let channel = track.routing.output_channel.unwrap_or(1).clamp(1, 16);
                let port = track
                    .routing
                    .output_port
                    .cloned_resolved(self.default_output_port.as_ref());
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
                (port, channel, events)
            })
            .collect();

        for (port, channel, events) in track_events {
            let Some(port) = port else {
                continue;
            };
            for (event_ticks, note_on, pitch, velocity) in events {
                let due_at = event_dispatch_due_at(
                    previous_ticks,
                    advanced_ticks,
                    event_ticks,
                    window_started_at,
                    window_ended_at,
                );
                let result = if note_on {
                    self.midi_output.schedule_note_on_at(
                        &port,
                        channel,
                        pitch,
                        velocity,
                        due_at,
                        MidiEventPriority::Playback,
                    )
                } else {
                    self.midi_output.schedule_note_off_at(
                        &port,
                        channel,
                        pitch,
                        due_at,
                        MidiEventPriority::NoteOff,
                    )
                };
                if result.is_err() {
                    self.prewarm_outputs();
                }
            }
        }
    }

    fn dispatch_live_output_events(
        &mut self,
        previous_ticks: u64,
        current_ticks: u64,
        window_started_at: Instant,
        window_ended_at: Instant,
    ) {
        if current_ticks <= previous_ticks {
            return;
        }

        self.ensure_output_fx_live_state_len();
        for track_index in 0..self.project.tracks.len() {
            let Some(track_view) = self.project.tracks.get(track_index) else {
                continue;
            };
            let output_chain = track_view.midi_fx.output_fx.clone();
            let output_port = track_view
                .routing
                .output_port
                .cloned_resolved(self.default_output_port.as_ref());
            let output_channel = track_view.routing.output_channel;

            let output_events = if let Some(state) = self.output_fx_live_states.get_mut(track_index)
            {
                process_live_chain_tick(
                    &output_chain,
                    state,
                    previous_ticks,
                    current_ticks,
                    self.project.global_harmony.root,
                )
            } else {
                Vec::new()
            };

            if let (Some(port), Some(channel)) = (output_port.as_ref(), output_channel) {
                for (event_ticks, event) in output_events {
                    let due_at = event_dispatch_due_at(
                        previous_ticks,
                        current_ticks.saturating_sub(previous_ticks),
                        event_ticks,
                        window_started_at,
                        window_ended_at,
                    );
                    let result = match event {
                        LiveMidiFxEvent::NoteOn { pitch, velocity } => {
                            self.midi_output.schedule_note_on_at(
                                port,
                                channel.clamp(1, 16),
                                pitch,
                                velocity,
                                due_at,
                                MidiEventPriority::DelayedFx,
                            )
                        }
                        LiveMidiFxEvent::NoteOff { pitch } => {
                            self.midi_output.schedule_note_off_at(
                                port,
                                channel.clamp(1, 16),
                                pitch,
                                due_at,
                                MidiEventPriority::NoteOff,
                            )
                        }
                    };
                    if result.is_err() {
                        self.prewarm_outputs();
                    }
                }
            }
        }
    }
}

fn runtime_seek_requested(previous_ticks: u64, requested_ticks: u64, ppqn: u16) -> bool {
    let tolerance_ticks = u64::from(ppqn.max(1) / 8).max(1);
    previous_ticks.abs_diff(requested_ticks) > tolerance_ticks
}

fn event_dispatch_due_at(
    previous_ticks: u64,
    advanced_ticks: u64,
    event_ticks: u64,
    window_started_at: Instant,
    window_ended_at: Instant,
) -> Instant {
    if advanced_ticks == 0 {
        return window_ended_at;
    }
    let window_duration = window_ended_at.saturating_duration_since(window_started_at);
    if window_duration.is_zero() {
        return window_ended_at;
    }
    let offset_ticks = event_ticks
        .saturating_sub(previous_ticks)
        .min(advanced_ticks);
    let offset_nanos = window_duration
        .as_nanos()
        .saturating_mul(offset_ticks as u128)
        / advanced_ticks as u128;
    window_started_at + Duration::from_nanos(offset_nanos.min(u64::MAX as u128) as u64)
}

fn log_runtime(message: String) {
    if std::env::var("TREKR_MIDI_RUNTIME_LOG")
        .ok()
        .is_none_or(|value| value == "0")
    {
        return;
    }
    let line = format!("[midiruntime] {message}");
    eprintln!("{line}");
    append_env_log(
        "TREKR_MIDI_RUNTIME_LOG_PATH",
        "trekr-midi-runtime.log",
        &line,
    );
}

fn append_env_log(path_var: &str, default_name: &str, line: &str) {
    let path = std::env::var(path_var).unwrap_or_else(|_| default_name.to_string());
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
        let _ = writeln!(file, "{line}");
    }
}
