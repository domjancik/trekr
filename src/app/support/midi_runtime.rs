use crate::midi_fx::{
    LiveMidiFxEvent, LiveMidiFxState, MidiFx, MidiFxSlot, playback_timing_lookback_ticks,
    process_live_chain_event, process_live_chain_tick, reset_live_fx_timing, transform_notes,
};
use crate::midi_io::{
    MidiInputEvent, MidiInputMessage, MidiOutputCommandMeta, MidiOutputObservedEvent,
    MidiOutputOrigin, MidiOutputRuntime, MidiPortRef,
};
use crate::project::{MidiNote, Project, RecordContext, Track};
use crate::routing::{MidiChannelFilter, TrackPortSelection};
use crate::timeline::RecordingTake;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, VecDeque};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering as AtomicOrdering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use super::super::note_runtime::{
    occurrence_note_events, scheduled_note_occurrences, ticks_per_second_for_tempo,
};

const PLAYBACK_LOOKAHEAD_MS: u64 = 150;
const IDLE_WAKE_INTERVAL_MS: u64 = 2;
const DIAG_SUMMARY_INTERVAL: Duration = Duration::from_secs(1);

#[derive(Debug, Clone)]
pub(crate) struct MidiRuntimeStateSync {
    pub project: Project,
    pub transport_ticks: u64,
    pub playhead_ticks: u64,
    pub live_fx_ticks: u64,
    pub default_input_port: Option<MidiPortRef>,
    pub default_output_port: Option<MidiPortRef>,
}

#[derive(Debug, Clone)]
pub(crate) struct MidiRuntimeUiSnapshot {
    pub transport_ticks: u64,
    pub playhead_ticks: u64,
    pub live_fx_ticks: u64,
    pub recording_takes: Arc<Vec<Option<RecordingTake>>>,
    #[allow(dead_code)]
    pub updated_at: Instant,
}

impl Default for MidiRuntimeUiSnapshot {
    fn default() -> Self {
        Self {
            transport_ticks: 0,
            playhead_ticks: 0,
            live_fx_ticks: 0,
            recording_takes: Arc::new(Vec::new()),
            updated_at: Instant::now(),
        }
    }
}

#[derive(Debug, Default)]
struct RuntimeMetrics {
    callback_to_runtime_total_ns: AtomicU64,
    callback_to_runtime_max_ns: AtomicU64,
    callback_to_output_total_ns: AtomicU64,
    callback_to_output_max_ns: AtomicU64,
    callback_to_output_count: AtomicU64,
    due_miss_total_ns: AtomicU64,
    due_miss_max_ns: AtomicU64,
    due_miss_count: AtomicU64,
    runtime_input_count: AtomicU64,
    output_send_count: AtomicU64,
    playback_send_count: AtomicU64,
    live_send_count: AtomicU64,
    queue_depth_max: AtomicU64,
}

impl RuntimeMetrics {
    fn observe_callback_to_runtime(&self, elapsed: Duration) {
        let nanos = saturating_nanos_u64(elapsed);
        self.runtime_input_count
            .fetch_add(1, AtomicOrdering::Relaxed);
        self.callback_to_runtime_total_ns
            .fetch_add(nanos, AtomicOrdering::Relaxed);
        atomic_update_max(&self.callback_to_runtime_max_ns, nanos);
    }

    fn observe_output(&self, observed: &MidiOutputObservedEvent) {
        self.output_send_count.fetch_add(1, AtomicOrdering::Relaxed);
        match observed.origin {
            MidiOutputOrigin::Playback => {
                self.playback_send_count
                    .fetch_add(1, AtomicOrdering::Relaxed);
            }
            MidiOutputOrigin::LiveImmediate | MidiOutputOrigin::LiveScheduled => {
                self.live_send_count.fetch_add(1, AtomicOrdering::Relaxed);
            }
            MidiOutputOrigin::Direct | MidiOutputOrigin::Panic => {}
        }
        if let Some(callback_received_at) = observed.callback_received_at {
            let elapsed = observed
                .sent_at
                .saturating_duration_since(callback_received_at);
            let nanos = saturating_nanos_u64(elapsed);
            self.callback_to_output_total_ns
                .fetch_add(nanos, AtomicOrdering::Relaxed);
            self.callback_to_output_count
                .fetch_add(1, AtomicOrdering::Relaxed);
            atomic_update_max(&self.callback_to_output_max_ns, nanos);
        }
        if let Some(due_at) = observed.due_at {
            if observed.sent_at > due_at {
                let late = observed.sent_at.saturating_duration_since(due_at);
                let nanos = saturating_nanos_u64(late);
                self.due_miss_total_ns
                    .fetch_add(nanos, AtomicOrdering::Relaxed);
                self.due_miss_count.fetch_add(1, AtomicOrdering::Relaxed);
                atomic_update_max(&self.due_miss_max_ns, nanos);
            }
        }
    }

    fn observe_queue_depth(&self, depth: usize) {
        atomic_update_max(&self.queue_depth_max, depth as u64);
    }

    fn summary(&self) -> String {
        let runtime_inputs = self.runtime_input_count.load(AtomicOrdering::Relaxed);
        let output_sends = self.output_send_count.load(AtomicOrdering::Relaxed);
        let callback_to_output_count = self.callback_to_output_count.load(AtomicOrdering::Relaxed);
        let due_miss_count = self.due_miss_count.load(AtomicOrdering::Relaxed);
        format!(
            "trekr midi runtime: inputs={} outputs={} live={} playback={} cb_to_runtime_avg_ms={:.3} cb_to_runtime_max_ms={:.3} cb_to_output_avg_ms={:.3} cb_to_output_max_ms={:.3} due_miss_avg_ms={:.3} due_miss_max_ms={:.3} due_miss_count={} queue_depth_max={}",
            runtime_inputs,
            output_sends,
            self.live_send_count.load(AtomicOrdering::Relaxed),
            self.playback_send_count.load(AtomicOrdering::Relaxed),
            avg_ms(
                self.callback_to_runtime_total_ns
                    .load(AtomicOrdering::Relaxed),
                runtime_inputs
            ),
            nanos_to_ms(
                self.callback_to_runtime_max_ns
                    .load(AtomicOrdering::Relaxed)
            ),
            avg_ms(
                self.callback_to_output_total_ns
                    .load(AtomicOrdering::Relaxed),
                callback_to_output_count
            ),
            nanos_to_ms(self.callback_to_output_max_ns.load(AtomicOrdering::Relaxed)),
            avg_ms(
                self.due_miss_total_ns.load(AtomicOrdering::Relaxed),
                due_miss_count
            ),
            nanos_to_ms(self.due_miss_max_ns.load(AtomicOrdering::Relaxed)),
            due_miss_count,
            self.queue_depth_max.load(AtomicOrdering::Relaxed),
        )
    }
}

pub(crate) struct MidiRuntime {
    sender: Sender<MidiRuntimeCommand>,
    input_sender: Sender<MidiInputEvent>,
    snapshot: Arc<Mutex<MidiRuntimeUiSnapshot>>,
    active: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}
impl MidiRuntime {
    pub(crate) fn new(midi_output: MidiOutputRuntime) -> Self {
        let (sender, receiver) = mpsc::channel();
        let (input_sender, input_receiver) = mpsc::channel();
        let (completion_sender, completion_receiver) = mpsc::channel();
        let snapshot = Arc::new(Mutex::new(MidiRuntimeUiSnapshot::default()));
        let active = Arc::new(AtomicBool::new(true));
        let thread_snapshot = snapshot.clone();
        let thread_active = active.clone();
        let thread = thread::Builder::new()
            .name("trekr-midi-runtime".to_string())
            .spawn(move || {
                MidiRuntimeEngine::new(
                    midi_output,
                    receiver,
                    input_receiver,
                    completion_sender,
                    completion_receiver,
                    thread_snapshot,
                    thread_active,
                )
                .run();
            })
            .expect("midi runtime thread should start");
        Self {
            sender,
            input_sender,
            snapshot,
            active,
            thread: Some(thread),
        }
    }

    pub(crate) fn is_enabled(&self) -> bool {
        self.active.load(AtomicOrdering::Relaxed)
    }

    pub(crate) fn input_sender(&self) -> Sender<MidiInputEvent> {
        self.input_sender.clone()
    }

    pub(crate) fn sync_state(&self, state: MidiRuntimeStateSync) {
        let _ = self.sender.send(MidiRuntimeCommand::SyncState(state));
    }

    pub(crate) fn snapshot(&self) -> MidiRuntimeUiSnapshot {
        self.snapshot
            .lock()
            .map(|snapshot| snapshot.clone())
            .unwrap_or_default()
    }

    pub(crate) fn capture_snapshot(&self) -> MidiRuntimeUiSnapshot {
        let (sender, receiver) = mpsc::channel();
        let _ = self
            .sender
            .send(MidiRuntimeCommand::CaptureSnapshot(sender));
        receiver.recv().unwrap_or_else(|_| self.snapshot())
    }

    #[cfg(test)]
    pub(crate) fn wait_until_idle(&self) {
        let (sender, receiver) = mpsc::channel();
        let _ = self.sender.send(MidiRuntimeCommand::Flush(sender));
        let _ = receiver.recv_timeout(Duration::from_secs(1));
    }
}

impl Drop for MidiRuntime {
    fn drop(&mut self) {
        let _ = self.sender.send(MidiRuntimeCommand::Shutdown);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

enum MidiRuntimeCommand {
    SyncState(MidiRuntimeStateSync),
    CaptureSnapshot(Sender<MidiRuntimeUiSnapshot>),
    #[cfg(test)]
    Flush(Sender<()>),
    Shutdown,
}

struct MidiRuntimeEngine {
    midi_output: MidiOutputRuntime,
    commands: Receiver<MidiRuntimeCommand>,
    input_events: Receiver<MidiInputEvent>,
    completion_sender: Sender<MidiOutputObservedEvent>,
    output_events: Receiver<MidiOutputObservedEvent>,
    snapshot: Arc<Mutex<MidiRuntimeUiSnapshot>>,
    active: Arc<AtomicBool>,
    metrics: Arc<RuntimeMetrics>,
    state: Option<RuntimeState>,
    scheduler: BinaryHeap<ScheduledMidiEvent>,
    sequence: u64,
    last_diag_at: Instant,
    diag_enabled: bool,
}

impl MidiRuntimeEngine {
    fn new(
        midi_output: MidiOutputRuntime,
        commands: Receiver<MidiRuntimeCommand>,
        input_events: Receiver<MidiInputEvent>,
        completion_sender: Sender<MidiOutputObservedEvent>,
        output_events: Receiver<MidiOutputObservedEvent>,
        snapshot: Arc<Mutex<MidiRuntimeUiSnapshot>>,
        active: Arc<AtomicBool>,
    ) -> Self {
        Self {
            midi_output,
            commands,
            input_events,
            completion_sender,
            output_events,
            snapshot,
            active,
            metrics: Arc::new(RuntimeMetrics::default()),
            state: None,
            scheduler: BinaryHeap::new(),
            sequence: 1,
            last_diag_at: Instant::now(),
            diag_enabled: std::env::var("TREKR_MIDI_RUNTIME_LOG")
                .ok()
                .is_some_and(|value| value != "0"),
        }
    }

    fn run(mut self) {
        while self.active.load(AtomicOrdering::Relaxed) {
            if self.drain_pending_commands() {
                break;
            }
            while let Ok(event) = self.output_events.try_recv() {
                self.metrics.observe_output(&event);
            }
            while let Ok(event) = self.input_events.try_recv() {
                self.handle_input_event(event);
            }

            let now = Instant::now();
            self.advance_clock(now);
            self.dispatch_due_events(now);
            self.publish_snapshot(now);
            self.maybe_print_summary(now);

            let timeout = self
                .scheduler
                .peek()
                .map(|next| next.due_at.saturating_duration_since(now))
                .unwrap_or_else(|| Duration::from_millis(IDLE_WAKE_INTERVAL_MS));
            match self
                .commands
                .recv_timeout(timeout.min(Duration::from_millis(IDLE_WAKE_INTERVAL_MS)))
            {
                Ok(command) => {
                    if self.handle_command(command) {
                        break;
                    }
                }
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => break,
            }
        }
    }

    fn drain_pending_commands(&mut self) -> bool {
        while let Ok(command) = self.commands.try_recv() {
            if self.handle_command(command) {
                return true;
            }
        }
        false
    }

    fn handle_command(&mut self, command: MidiRuntimeCommand) -> bool {
        match command {
            MidiRuntimeCommand::SyncState(state) => self.sync_state(state),
            MidiRuntimeCommand::CaptureSnapshot(sender) => {
                while let Ok(event) = self.input_events.try_recv() {
                    self.handle_input_event(event);
                }
                let now = Instant::now();
                self.advance_clock(now);
                self.dispatch_due_events(now);
                while let Ok(event) = self.output_events.try_recv() {
                    self.metrics.observe_output(&event);
                }
                let snapshot = self.build_snapshot(now);
                if let Ok(mut target) = self.snapshot.lock() {
                    *target = snapshot.clone();
                }
                let _ = sender.send(snapshot);
            }
            #[cfg(test)]
            MidiRuntimeCommand::Flush(sender) => {
                while let Ok(event) = self.input_events.try_recv() {
                    self.handle_input_event(event);
                }
                let now = Instant::now();
                self.advance_clock(now);
                self.dispatch_due_events(now);
                while let Ok(event) = self.output_events.try_recv() {
                    self.metrics.observe_output(&event);
                }
                self.publish_snapshot(now);
                let _ = sender.send(());
            }
            MidiRuntimeCommand::Shutdown => return true,
        }
        false
    }

    fn sync_state(&mut self, sync: MidiRuntimeStateSync) {
        let now = Instant::now();
        let previous_playing = self
            .state
            .as_ref()
            .map(|state| state.project.transport.playing)
            .unwrap_or(false);
        let should_reset = self
            .state
            .as_ref()
            .is_none_or(|state| state.project.transport.ppqn != sync.project.transport.ppqn);
        let mut state = RuntimeState::from_sync(sync, now);
        if should_reset {
            state.reset_live_states();
        } else if let Some(previous) = self.state.take() {
            state.copy_live_states_from(previous);
        }
        if previous_playing && !state.project.transport.playing {
            self.scheduler.clear();
            self.sequence = self.sequence.saturating_add(1);
            self.schedule_panic_for_all_tracks(&state, now);
        } else if !previous_playing && state.project.transport.playing {
            self.scheduler.clear();
        }
        self.prewarm_outputs(&state);
        self.state = Some(state);
    }

    fn advance_clock(&mut self, now: Instant) {
        let Some(mut state) = self.state.take() else {
            return;
        };

        if state.project.transport.playing {
            let current_ticks = state.transport_ticks_at(now);
            if current_ticks > state.transport_ticks {
                let previous_ticks = state.transport_ticks;
                state.transport_ticks = current_ticks;
                state.playhead_ticks = state.song_playhead_for_transport(current_ticks);
                self.state = Some(state);
                self.advance_live_fx_for_state(previous_ticks, current_ticks);
                self.schedule_playback_up_to(now);
                return;
            }
        } else {
            let current_live_ticks = state.live_ticks_at(now);
            if current_live_ticks > state.live_fx_ticks {
                let previous_ticks = state.live_fx_ticks;
                state.live_fx_ticks = current_live_ticks;
                self.state = Some(state);
                self.advance_live_fx_for_state(previous_ticks, current_live_ticks);
                return;
            }
        }
        self.state = Some(state);
    }

    fn handle_input_event(&mut self, event: MidiInputEvent) {
        self.metrics.observe_callback_to_runtime(
            Instant::now().saturating_duration_since(event.received_at()),
        );
        let note_event = match event.message {
            MidiInputMessage::NoteOn { pitch, velocity } => {
                LiveMidiFxEvent::NoteOn { pitch, velocity }
            }
            MidiInputMessage::NoteOff { pitch } => LiveMidiFxEvent::NoteOff { pitch },
            MidiInputMessage::ControlChange { .. } => return,
        };
        let Some(mut state) = self.state.take() else {
            return;
        };
        let input_ticks = if state.project.transport.playing {
            state.transport_ticks_at(event.received_at())
        } else {
            state.live_ticks_at(event.received_at())
        };

        let matching_tracks: Vec<usize> = state
            .project
            .tracks
            .iter()
            .enumerate()
            .filter(|(_, track)| {
                state.resolve_input_port(&track.routing.input_port) == Some(&event.port)
                    && matches_input_channel(track.routing.input_channel, event.channel)
            })
            .map(|(index, _)| index)
            .collect();
        for track_index in matching_tracks {
            let Some(track_view) = state.project.tracks.get(track_index) else {
                continue;
            };
            let input_chain = track_view.midi_fx.input_fx.clone();
            let output_chain = track_view.midi_fx.output_fx.clone();
            let record_mode = track_view.midi_fx.record_input_fx_mode;
            let passthrough = track_view.state.passthrough;
            let monitor_input_fx = track_view.midi_fx.monitor_input_fx;
            let output_port = state
                .resolve_output_port(&track_view.routing.output_port)
                .cloned();
            let output_channel = track_view.routing.output_channel;
            let processed =
                if let Some(live_state) = state.input_fx_live_states.get_mut(track_index) {
                    process_live_chain_event(
                        &input_chain,
                        live_state,
                        note_event.clone(),
                        input_ticks,
                        state.project.global_harmony.root,
                    )
                } else {
                    vec![note_event.clone()]
                };
            self.record_live_input_events(
                &mut state,
                track_index,
                record_mode,
                &note_event,
                &processed,
                input_ticks,
            );
            let monitor_source = if monitor_input_fx {
                processed.clone()
            } else {
                vec![note_event.clone()]
            };
            self.propagate_live_clone_events(
                &mut state,
                track_index,
                &processed,
                input_ticks,
                event.received_at(),
            );
            if passthrough {
                self.send_live_monitor_events(
                    &mut state,
                    track_index,
                    &output_chain,
                    output_port.as_ref(),
                    output_channel,
                    monitor_source,
                    input_ticks,
                    event.received_at(),
                    MidiOutputOrigin::LiveImmediate,
                );
            }
        }
        self.state = Some(state);
    }

    fn record_live_input_events(
        &mut self,
        state: &mut RuntimeState,
        track_index: usize,
        record_mode: crate::midi_fx::RecordInputFxMode,
        raw_event: &LiveMidiFxEvent,
        post_input_events: &[LiveMidiFxEvent],
        input_ticks: u64,
    ) {
        let Some(track) = state.project.tracks.get_mut(track_index) else {
            return;
        };
        if track.active_take.is_none() {
            return;
        }
        let record_events = if record_mode == crate::midi_fx::RecordInputFxMode::PostInputFx {
            post_input_events.to_vec()
        } else {
            vec![raw_event.clone()]
        };
        for record_event in record_events {
            match record_event {
                LiveMidiFxEvent::NoteOn { pitch, velocity } => {
                    track.record_note_on(pitch, velocity, input_ticks);
                }
                LiveMidiFxEvent::NoteOff { pitch } => {
                    track.record_note_off(pitch, input_ticks);
                }
            }
        }
        state.refresh_recording_takes_snapshot();
    }

    fn propagate_live_clone_events(
        &mut self,
        state: &mut RuntimeState,
        source_track_index: usize,
        source_events: &[LiveMidiFxEvent],
        current_ticks: u64,
        callback_received_at: Instant,
    ) {
        if source_events.is_empty() || !track_emits_clone_source(&state.project, source_track_index)
        {
            return;
        }

        #[derive(Clone)]
        struct CloneTarget {
            target_index: usize,
            record_mode: crate::midi_fx::RecordInputFxMode,
            monitor_input_fx: bool,
            output_port: Option<MidiPortRef>,
            output_channel: Option<u8>,
            output_chain: Vec<Option<MidiFxSlot>>,
        }

        let mut targets = Vec::new();
        for (target_index, track) in state.project.tracks.iter().enumerate() {
            if target_index == source_track_index {
                continue;
            }
            let clone_matches = track
                .midi_fx
                .input_fx
                .iter()
                .flatten()
                .filter(|slot| slot.enabled)
                .filter(|slot| {
                    matches!(
                        slot.effect,
                        MidiFx::TrackClone { source_track } if source_track == source_track_index
                    )
                })
                .count();
            if clone_matches == 0 {
                continue;
            }
            let base = CloneTarget {
                target_index,
                record_mode: track.midi_fx.record_input_fx_mode,
                monitor_input_fx: track.midi_fx.monitor_input_fx,
                output_port: state
                    .resolve_output_port(&track.routing.output_port)
                    .cloned(),
                output_channel: track.routing.output_channel,
                output_chain: track.midi_fx.output_fx.clone(),
            };
            for _ in 0..clone_matches {
                targets.push(base.clone());
            }
        }

        for target in targets {
            let post_input_events = if let (Some(track), Some(live_state)) = (
                state.project.tracks.get(target.target_index),
                state.input_fx_live_states.get_mut(target.target_index),
            ) {
                process_track_clone_live_events(
                    track,
                    source_track_index,
                    source_events,
                    live_state,
                    current_ticks,
                    state.project.global_harmony.root,
                )
            } else {
                Vec::new()
            };
            if let Some(track) = state.project.tracks.get_mut(target.target_index) {
                if track.active_take.is_some()
                    && target.record_mode == crate::midi_fx::RecordInputFxMode::PostInputFx
                {
                    for record_event in &post_input_events {
                        match *record_event {
                            LiveMidiFxEvent::NoteOn { pitch, velocity } => {
                                track.record_note_on(pitch, velocity, current_ticks);
                            }
                            LiveMidiFxEvent::NoteOff { pitch } => {
                                track.record_note_off(pitch, current_ticks);
                            }
                        }
                    }
                    state.refresh_recording_takes_snapshot();
                }
            }
            if target.monitor_input_fx {
                self.send_live_monitor_events(
                    state,
                    target.target_index,
                    &target.output_chain,
                    target.output_port.as_ref(),
                    target.output_channel,
                    post_input_events,
                    current_ticks,
                    callback_received_at,
                    MidiOutputOrigin::LiveImmediate,
                );
            }
        }
    }

    fn send_live_monitor_events(
        &mut self,
        state: &mut RuntimeState,
        track_index: usize,
        output_chain: &[Option<MidiFxSlot>],
        output_port: Option<&MidiPortRef>,
        output_channel: Option<u8>,
        events: Vec<LiveMidiFxEvent>,
        current_ticks: u64,
        callback_received_at: Instant,
        origin: MidiOutputOrigin,
    ) {
        let (Some(port), Some(channel)) = (output_port, output_channel) else {
            return;
        };
        for event in events {
            let processed =
                if let Some(live_state) = state.output_fx_live_states.get_mut(track_index) {
                    process_live_chain_event(
                        output_chain,
                        live_state,
                        event,
                        current_ticks,
                        state.project.global_harmony.root,
                    )
                } else {
                    Vec::new()
                };
            for item in processed {
                self.schedule_live_event(
                    state,
                    port.clone(),
                    channel,
                    item,
                    callback_received_at,
                    origin,
                );
            }
        }
    }

    fn schedule_live_event(
        &mut self,
        _state: &RuntimeState,
        port: MidiPortRef,
        channel: u8,
        event: LiveMidiFxEvent,
        callback_received_at: Instant,
        origin: MidiOutputOrigin,
    ) {
        match event {
            LiveMidiFxEvent::NoteOn { pitch, velocity } => {
                let due_at = Instant::now();
                let sequence = self.next_sequence();
                self.scheduler.push(ScheduledMidiEvent::note_on(
                    due_at,
                    port,
                    channel.clamp(1, 16),
                    pitch,
                    velocity,
                    priority_for_origin(origin, true),
                    sequence,
                    Some(callback_received_at),
                    origin,
                ));
            }
            LiveMidiFxEvent::NoteOff { pitch } => {
                let due_at = Instant::now();
                let sequence = self.next_sequence();
                self.scheduler.push(ScheduledMidiEvent::note_off(
                    due_at,
                    port,
                    channel.clamp(1, 16),
                    pitch,
                    priority_for_origin(origin, false),
                    sequence,
                    Some(callback_received_at),
                    origin,
                ));
            }
        }
        self.metrics.observe_queue_depth(self.scheduler.len());
    }

    fn advance_live_fx_for_state(&mut self, previous_ticks: u64, current_ticks: u64) {
        let Some(mut state) = self.state.take() else {
            return;
        };
        if current_ticks <= previous_ticks {
            self.state = Some(state);
            return;
        }
        for track_index in 0..state.project.tracks.len() {
            let Some(track) = state.project.tracks.get(track_index) else {
                continue;
            };
            let input_chain = track.midi_fx.input_fx.clone();
            let output_chain = track.midi_fx.output_fx.clone();
            let passthrough = track.state.passthrough;
            let monitor_input_fx = track.midi_fx.monitor_input_fx;
            let output_port = state
                .resolve_output_port(&track.routing.output_port)
                .cloned();
            let output_channel = track.routing.output_channel;

            let input_events =
                if let Some(live_state) = state.input_fx_live_states.get_mut(track_index) {
                    process_live_chain_tick(
                        &input_chain,
                        live_state,
                        previous_ticks,
                        current_ticks,
                        state.project.global_harmony.root,
                    )
                } else {
                    Vec::new()
                };
            if passthrough && monitor_input_fx {
                for (tick, event) in input_events {
                    self.send_live_monitor_events(
                        &mut state,
                        track_index,
                        &output_chain,
                        output_port.as_ref(),
                        output_channel,
                        vec![event],
                        tick,
                        Instant::now(),
                        MidiOutputOrigin::LiveScheduled,
                    );
                }
            }

            let output_events =
                if let Some(live_state) = state.output_fx_live_states.get_mut(track_index) {
                    process_live_chain_tick(
                        &output_chain,
                        live_state,
                        previous_ticks,
                        current_ticks,
                        state.project.global_harmony.root,
                    )
                } else {
                    Vec::new()
                };
            if let (Some(port), Some(channel)) = (output_port.as_ref(), output_channel) {
                for (_tick, event) in output_events {
                    self.schedule_live_event(
                        &state,
                        port.clone(),
                        channel,
                        event,
                        Instant::now(),
                        MidiOutputOrigin::LiveScheduled,
                    );
                }
            }
        }
        self.state = Some(state);
    }

    fn schedule_playback_up_to(&mut self, now: Instant) {
        let Some(mut state) = self.state.take() else {
            return;
        };
        if !state.project.transport.playing {
            self.state = Some(state);
            return;
        }
        let ticks_per_second = state.project.transport.ticks_per_second();
        let lookahead_ticks =
            ((u128::from(ticks_per_second) * u128::from(PLAYBACK_LOOKAHEAD_MS)) / 1_000) as u64;
        let target_ticks = state.transport_ticks.saturating_add(lookahead_ticks.max(1));
        if target_ticks <= state.scheduled_until_ticks {
            self.state = Some(state);
            return;
        }
        let previous_ticks = state.scheduled_until_ticks.max(state.transport_ticks);
        let advanced_ticks = target_ticks.saturating_sub(previous_ticks);
        if advanced_ticks == 0 {
            self.state = Some(state);
            return;
        }
        let track_events = collect_runtime_track_events(
            &state.project,
            previous_ticks,
            advanced_ticks,
            state.default_output_port.as_ref(),
        );
        for (port, channel, events) in track_events {
            let Some(port) = port else {
                continue;
            };
            for (event_ticks, note_on, pitch, velocity) in events {
                let due_at = tick_due_at(&state, event_ticks, now);
                if note_on {
                    let sequence = self.next_sequence();
                    self.scheduler.push(ScheduledMidiEvent::note_on(
                        due_at,
                        port.clone(),
                        channel,
                        pitch,
                        velocity,
                        ScheduledEventPriority::Playback,
                        sequence,
                        None,
                        MidiOutputOrigin::Playback,
                    ));
                } else {
                    let sequence = self.next_sequence();
                    self.scheduler.push(ScheduledMidiEvent::note_off(
                        due_at,
                        port.clone(),
                        channel,
                        pitch,
                        ScheduledEventPriority::NoteOff,
                        sequence,
                        None,
                        MidiOutputOrigin::Playback,
                    ));
                }
            }
        }
        state.scheduled_until_ticks = target_ticks;
        self.metrics.observe_queue_depth(self.scheduler.len());
        self.state = Some(state);
    }

    fn dispatch_due_events(&mut self, now: Instant) {
        while self
            .scheduler
            .peek()
            .is_some_and(|event| event.due_at <= now)
        {
            let Some(event) = self.scheduler.pop() else {
                break;
            };
            let meta = MidiOutputCommandMeta {
                origin: event.origin,
                sequence: event.sequence,
                callback_received_at: event.callback_received_at,
                due_at: Some(event.due_at),
                enqueued_at: Instant::now(),
                completion_sender: Some(self.completion_sender.clone()),
            };
            match event.payload {
                ScheduledPayload::NoteOn {
                    port,
                    channel,
                    pitch,
                    velocity,
                } => {
                    let _ = self.midi_output.send_note_on_with_meta(
                        &port,
                        channel,
                        pitch,
                        velocity,
                        Some(meta),
                    );
                }
                ScheduledPayload::NoteOff {
                    port,
                    channel,
                    pitch,
                } => {
                    let _ =
                        self.midi_output
                            .send_note_off_with_meta(&port, channel, pitch, Some(meta));
                }
                ScheduledPayload::AllNotesOff { port, channel } => {
                    let _ =
                        self.midi_output
                            .send_all_notes_off_with_meta(&port, channel, Some(meta));
                }
            }
        }
    }

    fn schedule_panic_for_all_tracks(&mut self, state: &RuntimeState, now: Instant) {
        for track in &state.project.tracks {
            if let Some((port, channel)) = state
                .resolve_output_port(&track.routing.output_port)
                .cloned()
                .zip(track.routing.output_channel)
            {
                let sequence = self.next_sequence();
                self.scheduler.push(ScheduledMidiEvent::all_notes_off(
                    now,
                    port,
                    channel.clamp(1, 16),
                    sequence,
                ));
            }
        }
    }

    fn prewarm_outputs(&self, state: &RuntimeState) {
        let mut outputs = VecDeque::new();
        for track in &state.project.tracks {
            if let Some(port) = state.resolve_output_port(&track.routing.output_port) {
                if !outputs
                    .iter()
                    .any(|existing: &MidiPortRef| existing == port)
                {
                    outputs.push_back(port.clone());
                }
            }
        }
        for port in outputs {
            let _ = self.midi_output.prewarm(&port);
        }
    }

    fn publish_snapshot(&self, now: Instant) {
        let snapshot = self.build_snapshot(now);
        if let Ok(mut target) = self.snapshot.lock() {
            *target = snapshot;
        }
    }

    fn build_snapshot(&self, now: Instant) -> MidiRuntimeUiSnapshot {
        if let Some(state) = self.state.as_ref() {
            MidiRuntimeUiSnapshot {
                transport_ticks: state.transport_ticks,
                playhead_ticks: state.playhead_ticks,
                live_fx_ticks: state.live_fx_ticks,
                recording_takes: state.recording_takes_snapshot.clone(),
                updated_at: now,
            }
        } else {
            MidiRuntimeUiSnapshot {
                updated_at: now,
                ..MidiRuntimeUiSnapshot::default()
            }
        }
    }

    fn maybe_print_summary(&mut self, now: Instant) {
        if !self.diag_enabled
            || now.saturating_duration_since(self.last_diag_at) < DIAG_SUMMARY_INTERVAL
        {
            return;
        }
        self.last_diag_at = now;
        eprintln!("{}", self.metrics.summary());
    }

    fn next_sequence(&mut self) -> u64 {
        let next = self.sequence;
        self.sequence = self.sequence.saturating_add(1);
        next
    }
}

struct RuntimeState {
    project: Project,
    transport_ticks: u64,
    playhead_ticks: u64,
    live_fx_ticks: u64,
    default_input_port: Option<MidiPortRef>,
    default_output_port: Option<MidiPortRef>,
    anchor_instant: Instant,
    anchor_transport_ticks: u64,
    anchor_live_ticks: u64,
    scheduled_until_ticks: u64,
    input_fx_live_states: Vec<LiveMidiFxState>,
    output_fx_live_states: Vec<LiveMidiFxState>,
    recording_takes_snapshot: Arc<Vec<Option<RecordingTake>>>,
}

impl RuntimeState {
    fn from_sync(sync: MidiRuntimeStateSync, now: Instant) -> Self {
        let track_count = sync.project.tracks.len();
        Self {
            project: sync.project,
            transport_ticks: sync.transport_ticks,
            playhead_ticks: sync.playhead_ticks,
            live_fx_ticks: sync.live_fx_ticks,
            default_input_port: sync.default_input_port,
            default_output_port: sync.default_output_port,
            anchor_instant: now,
            anchor_transport_ticks: sync.transport_ticks,
            anchor_live_ticks: sync.live_fx_ticks,
            scheduled_until_ticks: sync.transport_ticks,
            input_fx_live_states: vec![LiveMidiFxState::default(); track_count],
            output_fx_live_states: vec![LiveMidiFxState::default(); track_count],
            recording_takes_snapshot: Arc::new(Vec::new()),
        }
        .with_recording_snapshot()
    }

    fn copy_live_states_from(&mut self, previous: Self) {
        self.input_fx_live_states = previous.input_fx_live_states;
        self.output_fx_live_states = previous.output_fx_live_states;
        for (track, previous_track) in self.project.tracks.iter_mut().zip(previous.project.tracks) {
            track.active_take = if self.project.transport.recording {
                merge_runtime_take_state(
                    track.active_take.as_ref(),
                    previous_track.active_take.as_ref(),
                )
            } else {
                track.active_take.clone()
            };
        }
        self.scheduled_until_ticks = previous
            .scheduled_until_ticks
            .min(self.transport_ticks.max(previous.scheduled_until_ticks));
        self.refresh_recording_takes_snapshot();
    }

    fn reset_live_states(&mut self) {
        for state in &mut self.input_fx_live_states {
            reset_live_fx_timing(state, self.live_fx_ticks);
        }
        for state in &mut self.output_fx_live_states {
            reset_live_fx_timing(state, self.live_fx_ticks);
        }
    }

    fn with_recording_snapshot(mut self) -> Self {
        self.refresh_recording_takes_snapshot();
        self
    }

    fn refresh_recording_takes_snapshot(&mut self) {
        self.recording_takes_snapshot = Arc::new(
            self.project
                .tracks
                .iter()
                .map(|track| track.active_take.clone())
                .collect(),
        );
    }

    fn transport_ticks_at(&self, now: Instant) -> u64 {
        let delta = now.saturating_duration_since(self.anchor_instant);
        let ticks_per_second = self.project.transport.ticks_per_second();
        let advanced =
            (delta.as_nanos() as u128 * u128::from(ticks_per_second)) / 1_000_000_000_u128;
        self.anchor_transport_ticks.saturating_add(advanced as u64)
    }

    fn live_ticks_at(&self, now: Instant) -> u64 {
        let delta = now.saturating_duration_since(self.anchor_instant);
        let ticks_per_second = ticks_per_second_for_tempo(
            f64::from(self.project.transport.tempo_bpm),
            self.project.transport.ppqn,
        );
        let advanced =
            (delta.as_nanos() as u128 * u128::from(ticks_per_second)) / 1_000_000_000_u128;
        self.anchor_live_ticks.saturating_add(advanced as u64)
    }

    fn song_playhead_for_transport(&self, transport_ticks: u64) -> u64 {
        if !self.project.transport.loop_enabled || self.project.loop_region.length_ticks == 0 {
            return transport_ticks;
        }
        let loop_region = self.project.loop_region;
        let relative = transport_ticks.saturating_sub(loop_region.start_ticks);
        loop_region.start_ticks + (relative % loop_region.length_ticks.max(1))
    }

    fn resolve_input_port<'a>(
        &'a self,
        selection: &'a TrackPortSelection,
    ) -> Option<&'a MidiPortRef> {
        selection.resolve(self.default_input_port.as_ref())
    }

    fn resolve_output_port<'a>(
        &'a self,
        selection: &'a TrackPortSelection,
    ) -> Option<&'a MidiPortRef> {
        selection.resolve(self.default_output_port.as_ref())
    }
}

fn merge_runtime_take_state(
    synced: Option<&RecordingTake>,
    previous_runtime: Option<&RecordingTake>,
) -> Option<RecordingTake> {
    match (synced, previous_runtime) {
        (None, None) => None,
        (Some(synced), None) => Some(synced.clone()),
        (None, Some(previous_runtime)) => Some(previous_runtime.clone()),
        (Some(synced), Some(previous_runtime)) => {
            let mut merged = previous_runtime.clone();
            merged.pressed_at_ticks = merged.pressed_at_ticks.min(synced.pressed_at_ticks);
            merged.released_at_ticks = synced.released_at_ticks.or(merged.released_at_ticks);
            for recorded_note in &synced.recorded_notes {
                if !merged.recorded_notes.contains(recorded_note) {
                    merged.recorded_notes.push(*recorded_note);
                }
            }
            for pending_note in &synced.pending_notes {
                if !merged.pending_notes.contains(pending_note) {
                    merged.pending_notes.push(*pending_note);
                }
            }
            Some(merged)
        }
    }
}

#[derive(Clone)]
struct ScheduledMidiEvent {
    due_at: Instant,
    priority: ScheduledEventPriority,
    sequence: u64,
    origin: MidiOutputOrigin,
    callback_received_at: Option<Instant>,
    payload: ScheduledPayload,
}

#[derive(Clone)]
enum ScheduledPayload {
    NoteOn {
        port: MidiPortRef,
        channel: u8,
        pitch: u8,
        velocity: u8,
    },
    NoteOff {
        port: MidiPortRef,
        channel: u8,
        pitch: u8,
    },
    AllNotesOff {
        port: MidiPortRef,
        channel: u8,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum ScheduledEventPriority {
    Panic,
    LiveImmediate,
    NoteOff,
    Playback,
    DelayedFx,
}

impl ScheduledMidiEvent {
    fn note_on(
        due_at: Instant,
        port: MidiPortRef,
        channel: u8,
        pitch: u8,
        velocity: u8,
        priority: ScheduledEventPriority,
        sequence: u64,
        callback_received_at: Option<Instant>,
        origin: MidiOutputOrigin,
    ) -> Self {
        Self {
            due_at,
            priority,
            sequence,
            origin,
            callback_received_at,
            payload: ScheduledPayload::NoteOn {
                port,
                channel,
                pitch,
                velocity,
            },
        }
    }

    fn note_off(
        due_at: Instant,
        port: MidiPortRef,
        channel: u8,
        pitch: u8,
        priority: ScheduledEventPriority,
        sequence: u64,
        callback_received_at: Option<Instant>,
        origin: MidiOutputOrigin,
    ) -> Self {
        Self {
            due_at,
            priority,
            sequence,
            origin,
            callback_received_at,
            payload: ScheduledPayload::NoteOff {
                port,
                channel,
                pitch,
            },
        }
    }

    fn all_notes_off(due_at: Instant, port: MidiPortRef, channel: u8, sequence: u64) -> Self {
        Self {
            due_at,
            priority: ScheduledEventPriority::Panic,
            sequence,
            origin: MidiOutputOrigin::Panic,
            callback_received_at: None,
            payload: ScheduledPayload::AllNotesOff { port, channel },
        }
    }
}

impl Ord for ScheduledMidiEvent {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .due_at
            .cmp(&self.due_at)
            .then_with(|| other.priority.cmp(&self.priority))
            .then_with(|| other.sequence.cmp(&self.sequence))
    }
}

impl PartialOrd for ScheduledMidiEvent {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for ScheduledMidiEvent {
    fn eq(&self, other: &Self) -> bool {
        self.sequence == other.sequence
    }
}

impl Eq for ScheduledMidiEvent {}

fn priority_for_origin(origin: MidiOutputOrigin, note_on: bool) -> ScheduledEventPriority {
    match origin {
        MidiOutputOrigin::Panic => ScheduledEventPriority::Panic,
        MidiOutputOrigin::LiveImmediate => {
            if note_on {
                ScheduledEventPriority::LiveImmediate
            } else {
                ScheduledEventPriority::NoteOff
            }
        }
        MidiOutputOrigin::LiveScheduled => {
            if note_on {
                ScheduledEventPriority::DelayedFx
            } else {
                ScheduledEventPriority::NoteOff
            }
        }
        MidiOutputOrigin::Playback | MidiOutputOrigin::Direct => {
            if note_on {
                ScheduledEventPriority::Playback
            } else {
                ScheduledEventPriority::NoteOff
            }
        }
    }
}

fn tick_due_at(state: &RuntimeState, event_ticks: u64, now: Instant) -> Instant {
    let ticks_per_second = state.project.transport.ticks_per_second().max(1);
    let delta_ticks = event_ticks.saturating_sub(state.transport_ticks);
    let nanos =
        (u128::from(delta_ticks) * 1_000_000_000_u128) / u128::from(ticks_per_second.max(1));
    now + Duration::from_nanos(nanos.min(u128::from(u64::MAX)) as u64)
}

fn collect_runtime_track_events(
    project: &Project,
    previous_ticks: u64,
    advanced_ticks: u64,
    default_output_port: Option<&MidiPortRef>,
) -> Vec<(Option<MidiPortRef>, u8, Vec<(u64, bool, u8, u8)>)> {
    project
        .tracks
        .iter()
        .enumerate()
        .map(|(track_index, track)| {
            let channel = track.routing.output_channel.unwrap_or(1).clamp(1, 16);
            let port = track
                .routing
                .output_port
                .resolve(default_output_port)
                .cloned();
            let output_lookback = playback_timing_lookback_ticks(&track.midi_fx.output_fx);
            let lookback_padding = output_lookback.saturating_add(u64::from(output_lookback > 0));
            let source_previous_ticks = previous_ticks.saturating_sub(lookback_padding);
            let source_advanced_ticks = advanced_ticks.saturating_add(lookback_padding);
            let mut visited = vec![false; project.tracks.len()];
            let mut pre_output_notes = effective_track_pre_output_playback_notes_recursive(
                project,
                track_index,
                source_previous_ticks,
                source_advanced_ticks,
                &mut visited,
                previous_ticks,
            );
            let preview_notes = track.playback_preview_notes(
                project.transport,
                previous_ticks.saturating_add(advanced_ticks),
                record_context_for_track(project, track, previous_ticks),
            );
            let preview_occurrences = scheduled_note_occurrences(
                track,
                &preview_notes,
                source_previous_ticks,
                source_advanced_ticks,
                playback_loop_range_for_track(project, track),
            );
            pre_output_notes.extend(preview_occurrences);
            let transformed_notes = transform_notes(
                &pre_output_notes,
                &track.midi_fx.output_fx,
                project.global_harmony.root,
            );
            let events =
                occurrence_note_events(track, &transformed_notes, previous_ticks, advanced_ticks);
            (port, channel, events)
        })
        .collect()
}

fn effective_track_pre_output_playback_notes_recursive(
    project: &Project,
    track_index: usize,
    previous_ticks: u64,
    advanced_ticks: u64,
    visited: &mut [bool],
    transport_ticks: u64,
) -> Vec<MidiNote> {
    let Some(track) = project.tracks.get(track_index) else {
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
        playback_loop_range_for_track(project, track),
    );
    let clone_notes = effective_track_clone_playback_notes_recursive(
        project,
        track_index,
        previous_ticks,
        advanced_ticks,
        visited,
        transport_ticks,
    );
    let mut notes = native_notes;
    notes.extend(clone_notes);
    visited[track_index] = false;
    notes.sort_by_key(|note| (note.start_ticks, note.pitch, note.length_ticks));
    notes
}

fn effective_track_clone_playback_notes_recursive(
    project: &Project,
    track_index: usize,
    previous_ticks: u64,
    advanced_ticks: u64,
    visited: &mut [bool],
    transport_ticks: u64,
) -> Vec<MidiNote> {
    let Some(track) = project.tracks.get(track_index) else {
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
            notes = transform_notes(&notes, &[Some(slot.clone())], project.global_harmony.root);
            continue;
        };
        if source_track == track_index || !track_emits_clone_source(project, source_track) {
            continue;
        }
        notes.extend(effective_track_pre_output_playback_notes_recursive(
            project,
            source_track,
            previous_ticks,
            advanced_ticks,
            visited,
            transport_ticks,
        ));
    }
    let _ = transport_ticks;
    notes
}

fn playback_loop_range_for_track(
    project: &Project,
    track: &Track,
) -> Option<crate::timeline::LoopRegion> {
    if track.state.loop_enabled {
        Some(track.loop_region)
    } else {
        project
            .transport
            .loop_enabled
            .then_some(project.loop_region)
    }
}

fn record_context_for_track(
    project: &Project,
    track: &Track,
    _transport_ticks: u64,
) -> Option<RecordContext> {
    if track.state.loop_enabled {
        Some(RecordContext {
            range: track.loop_region,
            wrap_basis_ticks: 0,
            extend_clip_on_wrap: project.transport.loop_recording_extends_clip,
        })
    } else if project.transport.loop_enabled {
        Some(RecordContext {
            range: project.loop_region,
            wrap_basis_ticks: project.loop_region.start_ticks,
            extend_clip_on_wrap: project.transport.loop_recording_extends_clip,
        })
    } else {
        None
    }
}

fn track_emits_clone_source(project: &Project, source_track_index: usize) -> bool {
    let Some(source) = project.tracks.get(source_track_index) else {
        return false;
    };
    if source.state.muted {
        return false;
    }
    let any_solo = project.tracks.iter().any(|track| track.state.soloed);
    !any_solo || source.state.soloed
}

fn process_track_clone_live_events(
    track: &Track,
    source_track_index: usize,
    source_events: &[LiveMidiFxEvent],
    state: &mut LiveMidiFxState,
    current_ticks: u64,
    global_quantize_root: u8,
) -> Vec<LiveMidiFxEvent> {
    let mut events = Vec::new();
    for slot in track
        .midi_fx
        .input_fx
        .iter()
        .flatten()
        .filter(|slot| slot.enabled)
    {
        match slot.effect {
            MidiFx::TrackClone { source_track } if source_track == source_track_index => {
                events.extend(source_events.iter().cloned());
            }
            MidiFx::TrackClone { .. } => {}
            _ => {
                let mut transformed = Vec::new();
                for event in events {
                    transformed.extend(process_live_chain_event(
                        &[Some(slot.clone())],
                        state,
                        event,
                        current_ticks,
                        global_quantize_root,
                    ));
                }
                events = transformed;
                if events.is_empty() {
                    break;
                }
            }
        }
    }
    events
}

fn matches_input_channel(filter: MidiChannelFilter, channel: u8) -> bool {
    match filter {
        MidiChannelFilter::Omni => true,
        MidiChannelFilter::Channel(expected) => expected == channel,
    }
}

fn atomic_update_max(target: &AtomicU64, candidate: u64) {
    let mut current = target.load(AtomicOrdering::Relaxed);
    while candidate > current {
        match target.compare_exchange_weak(
            current,
            candidate,
            AtomicOrdering::Relaxed,
            AtomicOrdering::Relaxed,
        ) {
            Ok(_) => break,
            Err(next) => current = next,
        }
    }
}

fn saturating_nanos_u64(duration: Duration) -> u64 {
    duration.as_nanos().min(u128::from(u64::MAX)) as u64
}

fn nanos_to_ms(nanos: u64) -> f64 {
    nanos as f64 / 1_000_000.0
}

fn avg_ms(total_nanos: u64, count: u64) -> f64 {
    if count == 0 {
        0.0
    } else {
        nanos_to_ms(total_nanos / count.max(1))
    }
}
