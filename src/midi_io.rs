use midir::{MidiInput, MidiOutput, MidiOutputConnection};
use std::collections::HashMap;
use std::sync::mpsc::{self, Sender};
use std::thread;

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MidiDeviceCatalog {
    pub inputs: Vec<MidiPortRef>,
    pub outputs: Vec<MidiPortRef>,
    pub selected_input: Option<usize>,
    pub selected_output: Option<usize>,
}

impl MidiDeviceCatalog {
    pub fn scan() -> Self {
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

        if inputs.is_empty() && outputs.is_empty() {
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
}

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
        thread::Builder::new()
            .name("trekr-midi-output".to_string())
            .spawn(move || {
                let mut worker = MidiOutputWorker::default();
                while let Ok(command) = receiver.recv() {
                    let _ = worker.handle(command);
                }
            })
            .expect("midi output worker should start");

        Self { sender }
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
        self.sender
            .send(MidiOutputCommand::NoteOn {
                port: port.clone(),
                channel,
                pitch,
                velocity,
            })
            .map_err(|error| error.to_string())
    }

    pub fn send_note_off(
        &mut self,
        port: &MidiPortRef,
        channel: u8,
        pitch: u8,
    ) -> Result<(), String> {
        self.sender
            .send(MidiOutputCommand::NoteOff {
                port: port.clone(),
                channel,
                pitch,
            })
            .map_err(|error| error.to_string())
    }

    pub fn send_all_notes_off(&mut self, port: &MidiPortRef, channel: u8) -> Result<(), String> {
        self.sender
            .send(MidiOutputCommand::AllNotesOff {
                port: port.clone(),
                channel,
            })
            .map_err(|error| error.to_string())
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

fn status_byte(base: u8, channel: u8) -> u8 {
    base | channel.saturating_sub(1).min(15)
}

#[cfg(test)]
mod tests {
    use super::{MidiDeviceCatalog, MidiPortRef, preserve_selection, status_byte};

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
}
