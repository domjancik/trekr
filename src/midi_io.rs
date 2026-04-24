use midir::{Ignore, MidiInput, MidiInputConnection, MidiOutput, MidiOutputConnection};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::mpsc::{self, Receiver, Sender};
#[cfg(test)]
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum MidiTransportProtocol {
    #[default]
    SystemMidi,
    RtpMidiNative,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetworkMidiEndpoint {
    pub key: String,
    pub host: String,
    pub control_port: u16,
    pub data_port: u16,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub service_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub host_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MidiPortRef {
    pub name: String,
    #[serde(default)]
    pub protocol: MidiTransportProtocol,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub network_endpoint: Option<NetworkMidiEndpoint>,
}

impl MidiPortRef {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            protocol: MidiTransportProtocol::SystemMidi,
            network_endpoint: None,
        }
    }

    pub fn rtp_midi(name: &str, endpoint: NetworkMidiEndpoint) -> Self {
        Self {
            name: name.to_string(),
            protocol: MidiTransportProtocol::RtpMidiNative,
            network_endpoint: Some(endpoint),
        }
    }

    pub fn network_key(&self) -> Option<String> {
        self.network_endpoint
            .as_ref()
            .map(|endpoint| endpoint.key.clone())
    }

    pub fn protocol_badge(&self) -> Option<&'static str> {
        match self.protocol {
            MidiTransportProtocol::SystemMidi => None,
            MidiTransportProtocol::RtpMidiNative => Some("RTP"),
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
    pub fn scan() -> Self {
        Self::scan_internal(true)
    }

    pub fn scan_live() -> Self {
        Self::scan_internal(false)
    }

    fn scan_internal(allow_demo_fallback: bool) -> Self {
        let inputs: Vec<MidiPortRef> = match MidiInput::new("trekr-midi-inputs") {
            Ok(midi_in) => midi_in
                .ports()
                .into_iter()
                .filter_map(|port| midi_in.port_name(&port).ok())
                .map(|name| MidiPortRef::new(&name))
                .collect(),
            Err(_) => Vec::new(),
        };
        let outputs: Vec<MidiPortRef> = match MidiOutput::new("trekr-midi-outputs") {
            Ok(midi_out) => midi_out
                .ports()
                .into_iter()
                .filter_map(|port| midi_out.port_name(&port).ok())
                .map(|name| MidiPortRef::new(&name))
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

pub struct MidiOutputRuntime {
    sender: Sender<MidiOutputCommand>,
    #[cfg(test)]
    sent_commands: Arc<Mutex<Vec<MidiOutputCommand>>>,
    #[cfg(test)]
    sent_messages: Arc<Mutex<Vec<(String, u8, u8, Option<u8>)>>>,
}

pub struct MidiInputRuntime {
    app_name: &'static str,
    sender: Sender<MidiInputEvent>,
    receiver: Receiver<MidiInputEvent>,
    connections: HashMap<String, MidiInputConnection<()>>,
    #[cfg(test)]
    requested_ports: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum MidiOutputCommand {
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
        &mut self,
        port: &MidiPortRef,
        channel: u8,
        pitch: u8,
        velocity: u8,
    ) -> Result<(), String> {
        let command = MidiOutputCommand::NoteOn {
            port: port.clone(),
            channel,
            pitch,
            velocity,
        };
        self.record_command_for_test(&command);
        #[cfg(test)]
        if let Ok(mut sent) = self.sent_messages.lock() {
            sent.push((port.name.clone(), channel, pitch, Some(velocity)));
        }
        self.sender.send(command).map_err(|error| error.to_string())
    }

    pub fn send_note_off(
        &mut self,
        port: &MidiPortRef,
        channel: u8,
        pitch: u8,
    ) -> Result<(), String> {
        let command = MidiOutputCommand::NoteOff {
            port: port.clone(),
            channel,
            pitch,
        };
        self.record_command_for_test(&command);
        #[cfg(test)]
        if let Ok(mut sent) = self.sent_messages.lock() {
            sent.push((port.name.clone(), channel, pitch, None));
        }
        self.sender.send(command).map_err(|error| error.to_string())
    }

    pub fn send_all_notes_off(&mut self, port: &MidiPortRef, channel: u8) -> Result<(), String> {
        let command = MidiOutputCommand::AllNotesOff {
            port: port.clone(),
            channel,
        };
        self.record_command_for_test(&command);
        #[cfg(test)]
        if let Ok(mut sent) = self.sent_messages.lock() {
            sent.push((port.name.clone(), channel, 123, None));
        }
        self.sender.send(command).map_err(|error| error.to_string())
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
    pub fn sync_ports(&mut self, ports: &[MidiPortRef]) {
        let wanted: Vec<String> = ports.iter().map(|port| port.name.clone()).collect();
        #[cfg(test)]
        {
            self.requested_ports = wanted.clone();
        }
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

impl MidiOutputWorker {
    fn handle(&mut self, command: MidiOutputCommand) -> Result<(), String> {
        match command {
            MidiOutputCommand::NoteOn {
                port,
                channel,
                pitch,
                velocity,
            } => self.send_message(&port, [status_byte(0x90, channel), pitch, velocity]),
            MidiOutputCommand::NoteOff {
                port,
                channel,
                pitch,
            } => self.send_message(&port, [status_byte(0x80, channel), pitch, 0]),
            MidiOutputCommand::AllNotesOff { port, channel } => {
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
mod tests {
    use super::{
        MidiDeviceCatalog, MidiInputMessage, MidiPortRef, parse_input_event, preserve_selection,
        status_byte,
    };

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
}
