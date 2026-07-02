use crate::thread_priority::promote_current_thread_for_midi;
use midir::{Ignore, MidiInput, MidiInputConnection, MidiOutput, MidiOutputConnection};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
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
    pub received_at: Instant,
    pub backend_timestamp_micros: Option<u64>,
    pub sequence: u64,
}

impl MidiInputEvent {
    pub fn received_at(&self) -> Instant {
        self.received_at
    }
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

#[cfg_attr(test, allow(dead_code))]
#[derive(Clone)]
pub struct MidiOutputRuntime {
    sender: Sender<MidiOutputCommand>,
    #[cfg(test)]
    sent_commands: Arc<Mutex<Vec<MidiOutputCommand>>>,
    #[cfg(test)]
    sent_messages: Arc<Mutex<Vec<(String, u8, u8, Option<u8>)>>>,
}

#[cfg_attr(test, allow(dead_code))]
pub struct MidiInputRuntime {
    app_name: &'static str,
    sender: Sender<MidiInputEvent>,
    receiver: Receiver<MidiInputEvent>,
    connections: HashMap<String, MidiInputConnection<()>>,
    fanout_sender: Arc<Mutex<Option<Sender<MidiInputEvent>>>>,
    sequence: Arc<AtomicU64>,
    #[cfg(test)]
    requested_ports: Vec<String>,
}

#[derive(Debug, Clone)]
enum MidiOutputCommand {
    NoteOn {
        port: MidiPortRef,
        channel: u8,
        pitch: u8,
        velocity: u8,
        meta: Option<MidiOutputCommandMeta>,
    },
    NoteOff {
        port: MidiPortRef,
        channel: u8,
        pitch: u8,
        meta: Option<MidiOutputCommandMeta>,
    },
    AllNotesOff {
        port: MidiPortRef,
        channel: u8,
        meta: Option<MidiOutputCommandMeta>,
    },
    Prewarm {
        port: MidiPortRef,
    },
}

struct MidiOutputWorker {
    app_name: &'static str,
    connections: HashMap<String, MidiOutputConnection>,
}

impl Default for MidiOutputRuntime {
    fn default() -> Self {
        let (sender, receiver) = mpsc::channel();
        #[cfg(test)]
        let sent_commands = Arc::new(Mutex::new(Vec::new()));
        #[cfg(test)]
        let sent_messages = Arc::new(Mutex::new(Vec::new()));
        thread::Builder::new()
            .name("trekr-midi-output".to_string())
            .spawn(move || {
                let diag_enabled = std::env::var("TREKR_MIDI_RUNTIME_LOG")
                    .ok()
                    .is_some_and(|value| value != "0");
                if let Err(error) = promote_current_thread_for_midi("midi output") {
                    if diag_enabled {
                        eprintln!("trekr midi runtime: thread_priority midi_output={error}");
                    }
                }
                let mut worker = MidiOutputWorker::default();
                while let Ok(command) = receiver.recv() {
                    let _ = worker.handle(command);
                }
            })
            .expect("midi output worker should start");

        Self {
            sender,
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
            fanout_sender: Arc::new(Mutex::new(None)),
            sequence: Arc::new(AtomicU64::new(1)),
            #[cfg(test)]
            requested_ports: Vec::new(),
        }
    }
}

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
        &self,
        port: &MidiPortRef,
        channel: u8,
        pitch: u8,
        velocity: u8,
    ) -> Result<(), String> {
        self.send_note_on_with_meta(port, channel, pitch, velocity, None)
    }

    pub fn send_note_on_with_meta(
        &self,
        port: &MidiPortRef,
        channel: u8,
        pitch: u8,
        velocity: u8,
        meta: Option<MidiOutputCommandMeta>,
    ) -> Result<(), String> {
        let command = MidiOutputCommand::NoteOn {
            port: port.clone(),
            channel,
            pitch,
            velocity,
            meta,
        };
        self.send_note_on_internal(port, channel, pitch, velocity, command)
    }

    #[cfg(not(test))]
    fn send_note_on_internal(
        &self,
        _port: &MidiPortRef,
        _channel: u8,
        _pitch: u8,
        _velocity: u8,
        command: MidiOutputCommand,
    ) -> Result<(), String> {
        self.record_command_for_test(&command);
        self.sender.send(command).map_err(|error| error.to_string())
    }

    #[cfg(test)]
    fn send_note_on_internal(
        &self,
        port: &MidiPortRef,
        channel: u8,
        pitch: u8,
        velocity: u8,
        command: MidiOutputCommand,
    ) -> Result<(), String> {
        self.record_command_for_test(&command);
        if let Ok(mut sent) = self.sent_messages.lock() {
            sent.push((port.name.clone(), channel, pitch, Some(velocity)));
        }
        Ok(())
    }

    pub fn send_note_off(&self, port: &MidiPortRef, channel: u8, pitch: u8) -> Result<(), String> {
        self.send_note_off_with_meta(port, channel, pitch, None)
    }

    pub fn send_note_off_with_meta(
        &self,
        port: &MidiPortRef,
        channel: u8,
        pitch: u8,
        meta: Option<MidiOutputCommandMeta>,
    ) -> Result<(), String> {
        let command = MidiOutputCommand::NoteOff {
            port: port.clone(),
            channel,
            pitch,
            meta,
        };
        self.send_note_off_internal(port, channel, pitch, command)
    }

    #[cfg(not(test))]
    fn send_note_off_internal(
        &self,
        _port: &MidiPortRef,
        _channel: u8,
        _pitch: u8,
        command: MidiOutputCommand,
    ) -> Result<(), String> {
        self.record_command_for_test(&command);
        self.sender.send(command).map_err(|error| error.to_string())
    }

    #[cfg(test)]
    fn send_note_off_internal(
        &self,
        port: &MidiPortRef,
        channel: u8,
        pitch: u8,
        command: MidiOutputCommand,
    ) -> Result<(), String> {
        self.record_command_for_test(&command);
        if let Ok(mut sent) = self.sent_messages.lock() {
            sent.push((port.name.clone(), channel, pitch, None));
        }
        Ok(())
    }

    pub fn send_all_notes_off(&self, port: &MidiPortRef, channel: u8) -> Result<(), String> {
        self.send_all_notes_off_with_meta(port, channel, None)
    }

    pub fn send_all_notes_off_with_meta(
        &self,
        port: &MidiPortRef,
        channel: u8,
        meta: Option<MidiOutputCommandMeta>,
    ) -> Result<(), String> {
        let command = MidiOutputCommand::AllNotesOff {
            port: port.clone(),
            channel,
            meta,
        };
        self.send_all_notes_off_internal(port, channel, command)
    }

    #[cfg(not(test))]
    fn send_all_notes_off_internal(
        &self,
        _port: &MidiPortRef,
        _channel: u8,
        command: MidiOutputCommand,
    ) -> Result<(), String> {
        self.record_command_for_test(&command);
        self.sender.send(command).map_err(|error| error.to_string())
    }

    #[cfg(test)]
    fn send_all_notes_off_internal(
        &self,
        port: &MidiPortRef,
        channel: u8,
        command: MidiOutputCommand,
    ) -> Result<(), String> {
        self.record_command_for_test(&command);
        if let Ok(mut sent) = self.sent_messages.lock() {
            sent.push((port.name.clone(), channel, 123, None));
        }
        Ok(())
    }

    #[cfg(test)]
    fn record_command_for_test(&self, command: &MidiOutputCommand) {
        self.sent_commands
            .lock()
            .expect("test midi output log should lock")
            .push(command.clone());
    }

    #[cfg(not(test))]
    fn record_command_for_test(&self, _command: &MidiOutputCommand) {}

    #[cfg(test)]
    pub fn sent_all_notes_off_count(&self) -> usize {
        self.sent_commands
            .lock()
            .expect("test midi output log should lock")
            .iter()
            .filter(|command| matches!(command, MidiOutputCommand::AllNotesOff { .. }))
            .count()
    }

    pub fn prewarm(&self, port: &MidiPortRef) -> Result<(), String> {
        let command = MidiOutputCommand::Prewarm { port: port.clone() };
        self.record_command_for_test(&command);
        self.sender.send(command).map_err(|error| error.to_string())
    }
}

impl MidiOutputRuntime {
    #[cfg(test)]
    pub fn sent_messages(&self) -> Vec<(String, u8, u8, Option<u8>)> {
        self.sent_messages
            .lock()
            .map(|messages| messages.clone())
            .unwrap_or_default()
    }
}

impl MidiInputRuntime {
    pub fn set_fanout_sender(&mut self, sender: Option<Sender<MidiInputEvent>>) {
        if let Ok(mut target) = self.fanout_sender.lock() {
            *target = sender;
        }
    }

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

            if let Ok(connection) = connect_input_by_name(
                self.app_name,
                &port.name,
                self.sender.clone(),
                self.fanout_sender.clone(),
                self.sequence.clone(),
            ) {
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

impl MidiOutputWorker {
    fn handle(&mut self, command: MidiOutputCommand) -> Result<(), String> {
        match command {
            MidiOutputCommand::NoteOn {
                port,
                channel,
                pitch,
                velocity,
                meta,
            } => self.send_message(
                &port,
                channel,
                pitch,
                Some(velocity),
                [status_byte(0x90, channel), pitch, velocity],
                meta,
            ),
            MidiOutputCommand::NoteOff {
                port,
                channel,
                pitch,
                meta,
            } => self.send_message(
                &port,
                channel,
                pitch,
                None,
                [status_byte(0x80, channel), pitch, 0],
                meta,
            ),
            MidiOutputCommand::AllNotesOff {
                port,
                channel,
                meta,
            } => self.send_message(
                &port,
                channel,
                123,
                None,
                [status_byte(0xB0, channel), 123, 0],
                meta,
            ),
            MidiOutputCommand::Prewarm { port } => self.connection_for(&port).map(|_| ()),
        }
    }

    fn send_message(
        &mut self,
        port: &MidiPortRef,
        channel: u8,
        pitch: u8,
        velocity: Option<u8>,
        message: [u8; 3],
        meta: Option<MidiOutputCommandMeta>,
    ) -> Result<(), String> {
        let connection = self.connection_for(port)?;
        let dequeued_at = Instant::now();
        let result = connection.send(&message).map_err(|error| error.to_string());
        let sent_at = Instant::now();
        if result.is_ok() {
            if let Some(meta) = meta {
                if let Some(sender) = meta.completion_sender {
                    let _ = sender.send(MidiOutputObservedEvent {
                        origin: meta.origin,
                        sequence: meta.sequence,
                        port: port.clone(),
                        channel,
                        pitch,
                        velocity,
                        callback_received_at: meta.callback_received_at,
                        due_at: meta.due_at,
                        enqueued_at: meta.enqueued_at,
                        dequeued_at,
                        sent_at,
                    });
                }
            }
        }
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

#[cfg_attr(test, allow(dead_code))]
fn connect_input_by_name(
    app_name: &str,
    target_name: &str,
    sender: Sender<MidiInputEvent>,
    fanout_sender: Arc<Mutex<Option<Sender<MidiInputEvent>>>>,
    sequence: Arc<AtomicU64>,
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
            move |timestamp, message, _state| {
                if let Some(event) = parse_input_event(
                    &port_name,
                    message,
                    Instant::now(),
                    Some(timestamp),
                    &sequence,
                ) {
                    let _ = sender.send(event.clone());
                    if let Ok(target) = fanout_sender.lock() {
                        if let Some(ref fanout) = *target {
                            let _ = fanout.send(event);
                        }
                    }
                }
            },
            (),
        )
        .map_err(|error| error.to_string())
}

fn parse_input_event(
    port_name: &str,
    message: &[u8],
    _received_at: Instant,
    _backend_timestamp_micros: Option<u64>,
    _sequence: &Arc<AtomicU64>,
) -> Option<MidiInputEvent> {
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
        received_at: _received_at,
        backend_timestamp_micros: _backend_timestamp_micros,
        sequence: _sequence.fetch_add(1, Ordering::Relaxed),
    })
}

fn status_byte(base: u8, channel: u8) -> u8 {
    base | channel.saturating_sub(1).min(15)
}

#[cfg(test)]
mod tests {
    use super::{
        MidiDeviceCatalog, MidiInputMessage, MidiPortRef, parse_input_event, preserve_selection,
        status_byte,
    };
    use std::sync::Arc;
    use std::sync::atomic::AtomicU64;
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
        let sequence = Arc::new(AtomicU64::new(1));
        let note_on =
            parse_input_event("In A", &[0x90, 64, 100], Instant::now(), None, &sequence).unwrap();
        let note_off =
            parse_input_event("In A", &[0x90, 64, 0], Instant::now(), None, &sequence).unwrap();
        let cc =
            parse_input_event("In A", &[0xB0, 21, 127], Instant::now(), None, &sequence).unwrap();

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
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MidiOutputOrigin {
    Direct,
    LiveImmediate,
    LiveScheduled,
    Playback,
    Panic,
}

#[derive(Debug, Clone)]
pub struct MidiOutputCommandMeta {
    pub origin: MidiOutputOrigin,
    pub sequence: u64,
    pub callback_received_at: Option<Instant>,
    pub due_at: Option<Instant>,
    pub enqueued_at: Instant,
    pub completion_sender: Option<Sender<MidiOutputObservedEvent>>,
}

#[derive(Debug, Clone)]
pub struct MidiOutputObservedEvent {
    pub origin: MidiOutputOrigin,
    pub sequence: u64,
    pub port: MidiPortRef,
    pub channel: u8,
    pub pitch: u8,
    pub velocity: Option<u8>,
    pub callback_received_at: Option<Instant>,
    pub due_at: Option<Instant>,
    pub enqueued_at: Instant,
    pub dequeued_at: Instant,
    pub sent_at: Instant,
}
