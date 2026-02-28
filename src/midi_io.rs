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

#[cfg(test)]
mod tests {
    use super::MidiDeviceCatalog;

    #[test]
    fn demo_catalog_exposes_default_ports() {
        let catalog = MidiDeviceCatalog::demo();

        assert_eq!(catalog.selected_input_port().unwrap().name, "Keystep 37");
        assert_eq!(catalog.selected_output_port().unwrap().name, "Digitone");
    }

    #[test]
    fn catalog_selection_clamps_to_available_ports() {
        let mut catalog = MidiDeviceCatalog::demo();
        catalog.set_selected_input(99);
        catalog.set_selected_output(99);

        assert_eq!(catalog.selected_input.unwrap(), 2);
        assert_eq!(catalog.selected_output.unwrap(), 2);
    }
}
