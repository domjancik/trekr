use midir::{Ignore, MidiInput, MidiInputConnection};
#[cfg(not(test))]
use midir::{MidiOutput, MidiOutputConnection};
use serde::{Deserialize, Serialize};
#[cfg(not(test))]
use std::cmp::Reverse;
#[cfg(not(test))]
use std::collections::BinaryHeap;
use std::collections::HashMap;
#[cfg(not(test))]
use std::fs::OpenOptions;
#[cfg(not(test))]
use std::io::Write;
use std::sync::atomic::{AtomicU64, Ordering};
#[cfg(not(test))]
use std::sync::mpsc::RecvTimeoutError;
use std::sync::mpsc::{self, Receiver, Sender};
#[cfg(test)]
use std::sync::{Arc, Mutex};
#[cfg(not(test))]
use std::thread;
use std::time::Instant;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MidiPortRef {
    pub name: String,
}

impl MidiPortRef {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MidiMessageKind {
    Note,
    ControlChange,
    ProgramChange,
    PitchBend,
    ChannelPressure,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MidiInputMessage {
    NoteOn { pitch: u8, velocity: u8 },
    NoteOff { pitch: u8 },
    ControlChange { controller: u8, value: u8 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MidiInputEvent {
    pub port: MidiPortRef,
    pub channel: u8,
    pub message: MidiInputMessage,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MidiDeviceCatalog {
    pub inputs: Vec<MidiPortRef>,
    pub outputs: Vec<MidiPortRef>,
    pub selected_input: Option<usize>,
    pub selected_output: Option<usize>,
}

impl MidiDeviceCatalog {
    #[cfg(test)]
    pub fn scan() -> Self {
        Self::demo()
    }

    #[cfg(not(test))]
    pub fn scan() -> Self {
        Self::scan_internal(true)
    }

    #[cfg(test)]
    pub fn scan_live() -> Self {
        Self::demo()
    }

    #[cfg(not(test))]
    pub fn scan_live() -> Self {
        Self::scan_internal(false)
    }

    #[cfg(not(test))]
    fn scan_internal(allow_demo_fallback: bool) -> Self {
        let inputs: Vec<MidiPortRef> = match MidiInput::new("trekr-midi-inputs") {
            Ok(midi_in) => midi_in
                .ports()
                .into_iter()
                .filter_map(|port| midi_in.port_name(&port).ok())
                .map(|name| MidiPortRef { name })
                .collect(),
            Err(_) => Vec::new(),
        };
        let outputs: Vec<MidiPortRef> = match MidiOutput::new("trekr-midi-outputs") {
            Ok(midi_out) => midi_out
                .ports()
                .into_iter()
                .filter_map(|port| midi_out.port_name(&port).ok())
                .map(|name| MidiPortRef { name })
                .collect(),
            Err(_) => Vec::new(),
        };

        if allow_demo_fallback && inputs.is_empty() && outputs.is_empty() {
            return Self::demo();
        }

        let mut catalog = Self {
            selected_input: None,
            selected_output: None,
            inputs,
            outputs,
        };
        if !catalog.inputs.is_empty() {
            catalog.selected_input = Some(0);
        }
        if !catalog.outputs.is_empty() {
            catalog.selected_output = Some(0);
        }
        catalog
    }

    pub fn demo() -> Self {
        Self {
            inputs: vec![
                MidiPortRef::new("Keystep 37"),
                MidiPortRef::new("Launchpad Mini"),
                MidiPortRef::new("DIN In A"),
            ],
            outputs: vec![
                MidiPortRef::new("Digitone"),
                MidiPortRef::new("Volca FM"),
                MidiPortRef::new("DIN Out A"),
            ],
            selected_input: Some(0),
            selected_output: Some(0),
        }
    }

    pub fn with_preserved_selection(&self, previous: &Self) -> Self {
        Self {
            selected_input: preserve_selection(&self.inputs, previous.selected_input_port()),
            selected_output: preserve_selection(&self.outputs, previous.selected_output_port()),
            inputs: self.inputs.clone(),
            outputs: self.outputs.clone(),
        }
    }

    pub fn input(&self, index: usize) -> Option<&MidiPortRef> {
        self.inputs.get(index)
    }

    pub fn output(&self, index: usize) -> Option<&MidiPortRef> {
        self.outputs.get(index)
    }

    pub fn selected_input_port(&self) -> Option<&MidiPortRef> {
        self.selected_input.and_then(|index| self.input(index))
    }

    pub fn selected_output_port(&self) -> Option<&MidiPortRef> {
        self.selected_output.and_then(|index| self.output(index))
    }

    pub fn set_selected_input(&mut self, index: usize) {
        if self.inputs.is_empty() {
            self.selected_input = None;
        } else {
            self.selected_input = Some(index.min(self.inputs.len() - 1));
        }
    }

    pub fn set_selected_output(&mut self, index: usize) {
        if self.outputs.is_empty() {
            self.selected_output = None;
        } else {
            self.selected_output = Some(index.min(self.outputs.len() - 1));
        }
    }
}

fn preserve_selection(ports: &[MidiPortRef], selected: Option<&MidiPortRef>) -> Option<usize> {
    let Some(selected) = selected else {
        return (!ports.is_empty()).then_some(0);
    };

    ports
        .iter()
        .position(|port| port == selected)
        .or_else(|| (!ports.is_empty()).then_some(0))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MidiEventPriority {
    Panic,
    LiveImmediate,
    NoteOff,
    Playback,
    DelayedFx,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum MidiOutputPayload {
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct QueuedMidiEvent {
    due_at: Instant,
    priority: MidiEventPriority,
    sequence: u64,
    payload: MidiOutputPayload,
}

impl Ord for QueuedMidiEvent {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (self.due_at, self.priority, self.sequence).cmp(&(
            other.due_at,
            other.priority,
            other.sequence,
        ))
    }
}

impl PartialOrd for QueuedMidiEvent {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg_attr(test, allow(dead_code))]
pub struct MidiOutputRuntime {
    sender: Sender<MidiSchedulerCommand>,
    sequence: AtomicU64,
    #[cfg(test)]
    sent_commands: Arc<Mutex<Vec<String>>>,
    #[cfg(test)]
    sent_messages: Arc<Mutex<Vec<(String, u8, u8, Option<u8>)>>>,
}

#[cfg_attr(test, allow(dead_code))]
pub struct MidiInputRuntime {
    app_name: &'static str,
    sender: Sender<MidiInputEvent>,
    receiver: Receiver<MidiInputEvent>,
    connections: HashMap<String, MidiInputConnection<()>>,
    #[cfg(test)]
    requested_ports: Vec<String>,
}

#[cfg_attr(test, allow(dead_code))]
enum MidiSchedulerCommand {
    Enqueue(QueuedMidiEvent),
    Prewarm(MidiPortRef),
    Shutdown,
}

#[cfg(not(test))]
struct MidiOutputWorker {
    app_name: &'static str,
    connections: HashMap<String, MidiOutputConnection>,
}

impl Default for MidiOutputRuntime {
    fn default() -> Self {
        let (sender, receiver) = mpsc::channel();
        #[cfg(test)]
        let _ = &receiver;
        #[cfg(test)]
        let sent_commands = Arc::new(Mutex::new(Vec::new()));
        #[cfg(test)]
        let sent_messages = Arc::new(Mutex::new(Vec::new()));
        #[cfg(not(test))]
        thread::Builder::new()
            .name("trekr-midi-output".to_string())
            .spawn(move || {
                let mut worker = MidiOutputWorker::default();
                worker.run(receiver);
            })
            .expect("midi output worker should start");

        Self {
            sender,
            sequence: AtomicU64::new(0),
            #[cfg(test)]
            sent_commands,
            #[cfg(test)]
            sent_messages,
        }
    }
}

impl Default for MidiInputRuntime {
    fn default() -> Self {
        let (sender, receiver) = mpsc::channel();
        Self {
            app_name: "trekr-midi-input",
            sender,
            receiver,
            connections: HashMap::new(),
            #[cfg(test)]
            requested_ports: Vec::new(),
        }
    }
}

#[cfg(not(test))]
impl Default for MidiOutputWorker {
    fn default() -> Self {
        Self {
            app_name: "trekr-midi-output",
            connections: HashMap::new(),
        }
    }
}

impl MidiOutputRuntime {
    pub fn send_note_on(
        &mut self,
        port: &MidiPortRef,
        channel: u8,
        pitch: u8,
        velocity: u8,
    ) -> Result<(), String> {
        self.schedule_note_on_at(
            port,
            channel,
            pitch,
            velocity,
            Instant::now(),
            MidiEventPriority::LiveImmediate,
        )
    }

    pub fn send_note_off(
        &mut self,
        port: &MidiPortRef,
        channel: u8,
        pitch: u8,
    ) -> Result<(), String> {
        self.schedule_note_off_at(
            port,
            channel,
            pitch,
            Instant::now(),
            MidiEventPriority::NoteOff,
        )
    }

    pub fn send_all_notes_off(&mut self, port: &MidiPortRef, channel: u8) -> Result<(), String> {
        let event = self.build_event(
            Instant::now(),
            MidiEventPriority::Panic,
            MidiOutputPayload::AllNotesOff {
                port: port.clone(),
                channel,
            },
        );
        self.record_command_for_test(&event);
        #[cfg(test)]
        {
            self.record_sent_payload(&event.payload);
            return Ok(());
        }
        #[cfg(not(test))]
        {
            self.sender
                .send(MidiSchedulerCommand::Enqueue(event))
                .map_err(|error| error.to_string())
        }
    }

    pub fn schedule_note_on_at(
        &mut self,
        port: &MidiPortRef,
        channel: u8,
        pitch: u8,
        velocity: u8,
        due_at: Instant,
        priority: MidiEventPriority,
    ) -> Result<(), String> {
        let event = self.build_event(
            due_at,
            priority,
            MidiOutputPayload::NoteOn {
                port: port.clone(),
                channel,
                pitch,
                velocity,
            },
        );
        self.record_command_for_test(&event);
        #[cfg(test)]
        {
            self.record_sent_payload(&event.payload);
            return Ok(());
        }
        #[cfg(not(test))]
        {
            self.sender
                .send(MidiSchedulerCommand::Enqueue(event))
                .map_err(|error| error.to_string())
        }
    }

    pub fn schedule_note_off_at(
        &mut self,
        port: &MidiPortRef,
        channel: u8,
        pitch: u8,
        due_at: Instant,
        priority: MidiEventPriority,
    ) -> Result<(), String> {
        let event = self.build_event(
            due_at,
            priority,
            MidiOutputPayload::NoteOff {
                port: port.clone(),
                channel,
                pitch,
            },
        );
        self.record_command_for_test(&event);
        #[cfg(test)]
        {
            self.record_sent_payload(&event.payload);
            return Ok(());
        }
        #[cfg(not(test))]
        {
            self.sender
                .send(MidiSchedulerCommand::Enqueue(event))
                .map_err(|error| error.to_string())
        }
    }

    pub fn prewarm_port(&mut self, port: &MidiPortRef) -> Result<(), String> {
        #[cfg(test)]
        {
            self.record_prewarm_for_test(port);
            return Ok(());
        }
        #[cfg(not(test))]
        {
            self.sender
                .send(MidiSchedulerCommand::Prewarm(port.clone()))
                .map_err(|error| error.to_string())
        }
    }

    fn build_event(
        &self,
        due_at: Instant,
        priority: MidiEventPriority,
        payload: MidiOutputPayload,
    ) -> QueuedMidiEvent {
        QueuedMidiEvent {
            due_at,
            priority,
            sequence: self.sequence.fetch_add(1, Ordering::Relaxed),
            payload,
        }
    }

    #[cfg(test)]
    fn record_command_for_test(&self, event: &QueuedMidiEvent) {
        let label = match &event.payload {
            MidiOutputPayload::NoteOn {
                port,
                channel,
                pitch,
                velocity,
            } => format!(
                "note_on:{}:{}:{}:{}:{:?}:{:?}",
                port.name, channel, pitch, velocity, event.priority, event.due_at
            ),
            MidiOutputPayload::NoteOff {
                port,
                channel,
                pitch,
            } => format!(
                "note_off:{}:{}:{}:{:?}:{:?}",
                port.name, channel, pitch, event.priority, event.due_at
            ),
            MidiOutputPayload::AllNotesOff { port, channel } => format!(
                "all_notes_off:{}:{}:{:?}:{:?}",
                port.name, channel, event.priority, event.due_at
            ),
        };
        self.sent_commands
            .lock()
            .expect("test midi output log should lock")
            .push(label);
    }

    #[cfg(not(test))]
    fn record_command_for_test(&self, _event: &QueuedMidiEvent) {}

    #[cfg(test)]
    fn record_sent_payload(&self, payload: &MidiOutputPayload) {
        let mut sent = self
            .sent_messages
            .lock()
            .expect("test midi output log should lock");
        match payload {
            MidiOutputPayload::NoteOn {
                port,
                channel,
                pitch,
                velocity,
            } => sent.push((port.name.clone(), *channel, *pitch, Some(*velocity))),
            MidiOutputPayload::NoteOff {
                port,
                channel,
                pitch,
            } => sent.push((port.name.clone(), *channel, *pitch, None)),
            MidiOutputPayload::AllNotesOff { port, channel } => {
                sent.push((port.name.clone(), *channel, 123, None))
            }
        }
    }

    #[cfg(test)]
    fn record_prewarm_for_test(&self, port: &MidiPortRef) {
        self.sent_commands
            .lock()
            .expect("test midi output log should lock")
            .push(format!("prewarm:{}", port.name));
    }

    #[cfg(test)]
    pub fn sent_all_notes_off_count(&self) -> usize {
        self.sent_messages
            .lock()
            .expect("test midi output log should lock")
            .iter()
            .filter(|(_, _, pitch, velocity)| *pitch == 123 && velocity.is_none())
            .count()
    }

    #[cfg(test)]
    pub fn sent_messages(&self) -> Vec<(String, u8, u8, Option<u8>)> {
        self.sent_messages
            .lock()
            .map(|messages| messages.clone())
            .unwrap_or_default()
    }
}

impl Drop for MidiOutputRuntime {
    fn drop(&mut self) {
        #[cfg(not(test))]
        {
            let _ = self.sender.send(MidiSchedulerCommand::Shutdown);
        }
    }
}

impl MidiInputRuntime {
    pub fn sync_ports(&mut self, ports: &[MidiPortRef]) {
        let wanted: Vec<String> = ports.iter().map(|port| port.name.clone()).collect();
        self.sync_ports_internal(&wanted, ports);
    }

    #[cfg(test)]
    fn sync_ports_internal(&mut self, wanted: &[String], _ports: &[MidiPortRef]) {
        self.requested_ports = wanted.to_vec();
        self.connections.clear();
    }

    #[cfg(not(test))]
    fn sync_ports_internal(&mut self, wanted: &[String], ports: &[MidiPortRef]) {
        self.connections
            .retain(|name, _| wanted.iter().any(|wanted_name| wanted_name == name));

        for port in ports {
            if self.connections.contains_key(&port.name) {
                continue;
            }

            if let Ok(connection) =
                connect_input_by_name(self.app_name, &port.name, self.sender.clone())
            {
                self.connections.insert(port.name.clone(), connection);
            }
        }
    }

    pub fn drain_events(&self) -> Vec<MidiInputEvent> {
        let mut events = Vec::new();
        while let Ok(event) = self.receiver.try_recv() {
            events.push(event);
        }
        events
    }

    #[cfg(test)]
    pub fn connected_port_names(&self) -> Vec<String> {
        let mut names = self.connections.keys().cloned().collect::<Vec<_>>();
        names.sort();
        names
    }

    #[cfg(test)]
    pub fn requested_port_names(&self) -> Vec<String> {
        let mut names = self.requested_ports.clone();
        names.sort();
        names
    }
}

#[cfg(not(test))]
impl MidiOutputWorker {
    fn run(&mut self, receiver: Receiver<MidiSchedulerCommand>) {
        let mut heap: BinaryHeap<Reverse<QueuedMidiEvent>> = BinaryHeap::new();
        loop {
            self.send_due_events(&mut heap, Instant::now());
            let next_due = heap.peek().map(|entry| entry.0.due_at);
            let command = match next_due {
                Some(due_at) => {
                    let timeout = due_at.saturating_duration_since(Instant::now());
                    match receiver.recv_timeout(timeout) {
                        Ok(command) => Some(command),
                        Err(RecvTimeoutError::Timeout) => None,
                        Err(RecvTimeoutError::Disconnected) => return,
                    }
                }
                None => match receiver.recv() {
                    Ok(command) => Some(command),
                    Err(_) => return,
                },
            };

            match command {
                Some(MidiSchedulerCommand::Enqueue(event)) => {
                    if matches!(event.payload, MidiOutputPayload::AllNotesOff { .. }) {
                        heap.clear();
                    }
                    heap.push(Reverse(event));
                }
                Some(MidiSchedulerCommand::Prewarm(port)) => {
                    let _ = self.connection_for(&port);
                }
                Some(MidiSchedulerCommand::Shutdown) => return,
                None => self.send_due_events(&mut heap, Instant::now()),
            }
        }
    }

    fn send_due_events(&mut self, heap: &mut BinaryHeap<Reverse<QueuedMidiEvent>>, now: Instant) {
        while heap.peek().is_some_and(|entry| entry.0.due_at <= now) {
            let Some(Reverse(event)) = heap.pop() else {
                break;
            };
            let _ = self.handle_event(event, now);
        }
    }

    fn handle_event(&mut self, event: QueuedMidiEvent, now: Instant) -> Result<(), String> {
        let scheduled_lag = now.saturating_duration_since(event.due_at);
        match event.payload {
            MidiOutputPayload::NoteOn {
                port,
                channel,
                pitch,
                velocity,
            } => {
                log_midi_output(format!(
                    "send due={:?} lag_ms={} priority={:?} kind=on port={} channel={} pitch={} velocity={}",
                    event.due_at,
                    scheduled_lag.as_millis(),
                    event.priority,
                    port.name,
                    channel,
                    pitch,
                    velocity
                ));
                self.send_message(&port, [status_byte(0x90, channel), pitch, velocity])
            }
            MidiOutputPayload::NoteOff {
                port,
                channel,
                pitch,
            } => {
                log_midi_output(format!(
                    "send due={:?} lag_ms={} priority={:?} kind=off port={} channel={} pitch={}",
                    event.due_at,
                    scheduled_lag.as_millis(),
                    event.priority,
                    port.name,
                    channel,
                    pitch
                ));
                self.send_message(&port, [status_byte(0x80, channel), pitch, 0])
            }
            MidiOutputPayload::AllNotesOff { port, channel } => {
                log_midi_output(format!(
                    "send due={:?} lag_ms={} priority={:?} kind=all_notes_off port={} channel={}",
                    event.due_at,
                    scheduled_lag.as_millis(),
                    event.priority,
                    port.name,
                    channel
                ));
                self.send_message(&port, [status_byte(0xB0, channel), 123, 0])
            }
        }
    }

    fn send_message(&mut self, port: &MidiPortRef, message: [u8; 3]) -> Result<(), String> {
        let connection = self.connection_for(port)?;
        let result = connection.send(&message).map_err(|error| error.to_string());
        if result.is_err() {
            self.connections.remove(&port.name);
        }
        result
    }

    fn connection_for(&mut self, port: &MidiPortRef) -> Result<&mut MidiOutputConnection, String> {
        if !self.connections.contains_key(&port.name) {
            let connection = connect_output_by_name(self.app_name, &port.name)?;
            self.connections.insert(port.name.clone(), connection);
        }

        self.connections
            .get_mut(&port.name)
            .ok_or_else(|| format!("missing output connection for {}", port.name))
    }
}

#[cfg(not(test))]
fn connect_output_by_name(
    app_name: &str,
    target_name: &str,
) -> Result<MidiOutputConnection, String> {
    let midi_out = MidiOutput::new(app_name).map_err(|error| error.to_string())?;
    let port = midi_out
        .ports()
        .into_iter()
        .find(|port| midi_out.port_name(port).ok().as_deref() == Some(target_name))
        .ok_or_else(|| format!("MIDI output port '{}' not found", target_name))?;

    midi_out
        .connect(&port, app_name)
        .map_err(|error| error.to_string())
}

#[cfg(not(test))]
fn log_midi_output(message: String) {
    if std::env::var("TREKR_MIDI_OUTPUT_LOG")
        .ok()
        .is_none_or(|value| value == "0")
    {
        return;
    }
    let line = format!("[midiout] {message}");
    eprintln!("{line}");
    append_env_log("TREKR_MIDI_OUTPUT_LOG_PATH", "trekr-midi-output.log", &line);
}

#[cfg(not(test))]
fn append_env_log(path_var: &str, default_name: &str, line: &str) {
    let path = std::env::var(path_var).unwrap_or_else(|_| default_name.to_string());
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
        let _ = writeln!(file, "{line}");
    }
}

#[cfg_attr(test, allow(dead_code))]
fn connect_input_by_name(
    app_name: &str,
    target_name: &str,
    sender: Sender<MidiInputEvent>,
) -> Result<MidiInputConnection<()>, String> {
    let mut midi_in = MidiInput::new(app_name).map_err(|error| error.to_string())?;
    midi_in.ignore(Ignore::None);
    let port = midi_in
        .ports()
        .into_iter()
        .find(|port| midi_in.port_name(port).ok().as_deref() == Some(target_name))
        .ok_or_else(|| format!("MIDI input port '{}' not found", target_name))?;
    let port_name = target_name.to_string();

    midi_in
        .connect(
            &port,
            app_name,
            move |_timestamp, message, _state| {
                if let Some(event) = parse_input_event(&port_name, message) {
                    let _ = sender.send(event);
                }
            },
            (),
        )
        .map_err(|error| error.to_string())
}

fn parse_input_event(port_name: &str, message: &[u8]) -> Option<MidiInputEvent> {
    let status = *message.first()?;
    let channel = (status & 0x0F) + 1;
    let pitch = *message.get(1)?;
    let value = *message.get(2).unwrap_or(&0);

    let message = match status & 0xF0 {
        0x80 => MidiInputMessage::NoteOff { pitch },
        0x90 if value == 0 => MidiInputMessage::NoteOff { pitch },
        0x90 => MidiInputMessage::NoteOn {
            pitch,
            velocity: value,
        },
        0xB0 => MidiInputMessage::ControlChange {
            controller: pitch,
            value,
        },
        _ => return None,
    };

    Some(MidiInputEvent {
        port: MidiPortRef::new(port_name),
        channel,
        message,
    })
}

fn status_byte(base: u8, channel: u8) -> u8 {
    base | channel.saturating_sub(1).min(15)
}

#[cfg(test)]
fn ordered_payloads(mut events: Vec<QueuedMidiEvent>) -> Vec<&'static str> {
    events.sort_by(|left, right| {
        (left.due_at, left.priority, left.sequence).cmp(&(
            right.due_at,
            right.priority,
            right.sequence,
        ))
    });
    events
        .into_iter()
        .map(|event| match event.payload {
            MidiOutputPayload::NoteOn { .. } => "on",
            MidiOutputPayload::NoteOff { .. } => "off",
            MidiOutputPayload::AllNotesOff { .. } => "panic",
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{
        MidiDeviceCatalog, MidiEventPriority, MidiInputMessage, MidiOutputPayload, MidiPortRef,
        QueuedMidiEvent, ordered_payloads, parse_input_event, preserve_selection, status_byte,
    };
    #[cfg(test)]
    use std::time::Duration;
    use std::time::Instant;

    #[test]
    fn status_byte_uses_one_based_channel_numbers() {
        assert_eq!(status_byte(0x90, 1), 0x90);
        assert_eq!(status_byte(0x90, 16), 0x9F);
    }

    #[test]
    fn preserve_selection_falls_back_to_first_port() {
        let ports = vec![MidiPortRef::new("A"), MidiPortRef::new("B")];
        let selected = MidiPortRef::new("Missing");

        assert_eq!(preserve_selection(&ports, Some(&selected)), Some(0));
        assert_eq!(preserve_selection(&ports, None), Some(0));
    }

    #[test]
    fn catalog_selection_clamps_to_available_ports() {
        let mut catalog = MidiDeviceCatalog {
            inputs: vec![MidiPortRef::new("In 1"), MidiPortRef::new("In 2")],
            outputs: vec![MidiPortRef::new("Out 1"), MidiPortRef::new("Out 2")],
            selected_input: Some(0),
            selected_output: Some(0),
        };
        catalog.set_selected_input(99);
        catalog.set_selected_output(99);

        assert_eq!(catalog.selected_input.unwrap(), 1);
        assert_eq!(catalog.selected_output.unwrap(), 1);
    }

    #[test]
    fn parse_input_event_handles_note_on_and_off() {
        let note_on = parse_input_event("In A", &[0x90, 64, 100]).unwrap();
        let note_off = parse_input_event("In A", &[0x90, 64, 0]).unwrap();
        let cc = parse_input_event("In A", &[0xB0, 21, 127]).unwrap();

        assert_eq!(
            note_on.message,
            MidiInputMessage::NoteOn {
                pitch: 64,
                velocity: 100
            }
        );
        assert_eq!(note_off.message, MidiInputMessage::NoteOff { pitch: 64 });
        assert_eq!(
            cc.message,
            MidiInputMessage::ControlChange {
                controller: 21,
                value: 127,
            }
        );
    }

    #[test]
    fn test_scan_uses_deterministic_demo_catalog() {
        let catalog = MidiDeviceCatalog::scan();

        assert_eq!(catalog, MidiDeviceCatalog::demo());
    }

    #[test]
    fn immediate_event_preempts_future_delayed_event() {
        let now = Instant::now();
        let events = vec![
            QueuedMidiEvent {
                due_at: now + Duration::from_millis(100),
                priority: MidiEventPriority::Playback,
                sequence: 0,
                payload: MidiOutputPayload::NoteOn {
                    port: MidiPortRef::new("Out"),
                    channel: 1,
                    pitch: 60,
                    velocity: 100,
                },
            },
            QueuedMidiEvent {
                due_at: now,
                priority: MidiEventPriority::LiveImmediate,
                sequence: 1,
                payload: MidiOutputPayload::NoteOn {
                    port: MidiPortRef::new("Out"),
                    channel: 1,
                    pitch: 61,
                    velocity: 100,
                },
            },
        ];

        assert_eq!(ordered_payloads(events), vec!["on", "on"]);
    }

    #[test]
    fn note_off_sorts_before_note_on_at_same_due_time() {
        let now = Instant::now();
        let events = vec![
            QueuedMidiEvent {
                due_at: now,
                priority: MidiEventPriority::Playback,
                sequence: 1,
                payload: MidiOutputPayload::NoteOn {
                    port: MidiPortRef::new("Out"),
                    channel: 1,
                    pitch: 60,
                    velocity: 100,
                },
            },
            QueuedMidiEvent {
                due_at: now,
                priority: MidiEventPriority::NoteOff,
                sequence: 0,
                payload: MidiOutputPayload::NoteOff {
                    port: MidiPortRef::new("Out"),
                    channel: 1,
                    pitch: 60,
                },
            },
        ];

        assert_eq!(ordered_payloads(events), vec!["off", "on"]);
    }

    #[test]
    fn panic_sorts_before_other_same_due_time_events() {
        let now = Instant::now();
        let events = vec![
            QueuedMidiEvent {
                due_at: now,
                priority: MidiEventPriority::DelayedFx,
                sequence: 2,
                payload: MidiOutputPayload::NoteOn {
                    port: MidiPortRef::new("Out"),
                    channel: 1,
                    pitch: 67,
                    velocity: 100,
                },
            },
            QueuedMidiEvent {
                due_at: now,
                priority: MidiEventPriority::Panic,
                sequence: 1,
                payload: MidiOutputPayload::AllNotesOff {
                    port: MidiPortRef::new("Out"),
                    channel: 1,
                },
            },
        ];

        assert_eq!(ordered_payloads(events), vec!["panic", "on"]);
    }

    #[test]
    fn sequence_preserves_deterministic_order_for_equal_due_time_and_priority() {
        let now = Instant::now();
        let events = vec![
            QueuedMidiEvent {
                due_at: now,
                priority: MidiEventPriority::Playback,
                sequence: 0,
                payload: MidiOutputPayload::NoteOn {
                    port: MidiPortRef::new("Out"),
                    channel: 1,
                    pitch: 60,
                    velocity: 100,
                },
            },
            QueuedMidiEvent {
                due_at: now,
                priority: MidiEventPriority::Playback,
                sequence: 1,
                payload: MidiOutputPayload::NoteOn {
                    port: MidiPortRef::new("Out"),
                    channel: 1,
                    pitch: 61,
                    velocity: 100,
                },
            },
        ];

        let ordered = events
            .into_iter()
            .map(|event| (event.due_at, event.priority, event.sequence))
            .collect::<Vec<_>>();
        assert_eq!(ordered[0].2, 0);
        assert_eq!(ordered[1].2, 1);
    }
}
