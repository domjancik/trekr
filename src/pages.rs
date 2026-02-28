#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppPage {
    Timeline,
    Mappings,
    MidiIo,
    Routing,
}

impl AppPage {
    pub const ALL: [Self; 4] = [
        Self::Timeline,
        Self::Mappings,
        Self::MidiIo,
        Self::Routing,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Timeline => "Timeline",
            Self::Mappings => "Mappings",
            Self::MidiIo => "MIDI I/O",
            Self::Routing => "Routing",
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::Timeline => Self::Mappings,
            Self::Mappings => Self::MidiIo,
            Self::MidiIo => Self::Routing,
            Self::Routing => Self::Timeline,
        }
    }

    pub fn previous(self) -> Self {
        match self {
            Self::Timeline => Self::Routing,
            Self::Mappings => Self::Timeline,
            Self::MidiIo => Self::Mappings,
            Self::Routing => Self::MidiIo,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MappingPageMode {
    Overview,
    Write,
}

impl MappingPageMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::Overview => "Read Only",
            Self::Write => "Write",
        }
    }

    pub fn toggle(self) -> Self {
        match self {
            Self::Overview => Self::Write,
            Self::Write => Self::Overview,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MidiIoListFocus {
    Inputs,
    Outputs,
}

impl MidiIoListFocus {
    pub fn toggle(self) -> Self {
        match self {
            Self::Inputs => Self::Outputs,
            Self::Outputs => Self::Inputs,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MidiIoPageState {
    pub focus: MidiIoListFocus,
    pub selected_input_index: usize,
    pub selected_output_index: usize,
}

impl Default for MidiIoPageState {
    fn default() -> Self {
        Self {
            focus: MidiIoListFocus::Inputs,
            selected_input_index: 0,
            selected_output_index: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoutingField {
    InputDevice,
    InputChannel,
    OutputDevice,
    OutputChannel,
    Passthrough,
}

impl RoutingField {
    pub const ALL: [Self; 5] = [
        Self::InputDevice,
        Self::InputChannel,
        Self::OutputDevice,
        Self::OutputChannel,
        Self::Passthrough,
    ];

    pub fn next(self) -> Self {
        match self {
            Self::InputDevice => Self::InputChannel,
            Self::InputChannel => Self::OutputDevice,
            Self::OutputDevice => Self::OutputChannel,
            Self::OutputChannel => Self::Passthrough,
            Self::Passthrough => Self::InputDevice,
        }
    }

    pub fn previous(self) -> Self {
        match self {
            Self::InputDevice => Self::Passthrough,
            Self::InputChannel => Self::InputDevice,
            Self::OutputDevice => Self::InputChannel,
            Self::OutputChannel => Self::OutputDevice,
            Self::Passthrough => Self::OutputChannel,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::InputDevice => "Input Device",
            Self::InputChannel => "Input Channel",
            Self::OutputDevice => "Output Device",
            Self::OutputChannel => "Output Channel",
            Self::Passthrough => "Passthrough",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AppPageState {
    pub current_page: AppPage,
    pub midi_io: MidiIoPageState,
    pub selected_mapping_index: usize,
    pub mapping_mode: MappingPageMode,
    pub selected_routing_field: RoutingField,
}

impl Default for AppPageState {
    fn default() -> Self {
        Self {
            current_page: AppPage::Timeline,
            midi_io: MidiIoPageState::default(),
            selected_mapping_index: 0,
            mapping_mode: MappingPageMode::Overview,
            selected_routing_field: RoutingField::InputDevice,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AppPage, MappingPageMode, MidiIoListFocus, RoutingField};

    #[test]
    fn app_pages_cycle_in_expected_order() {
        assert_eq!(AppPage::Timeline.next(), AppPage::Mappings);
        assert_eq!(AppPage::Timeline.previous(), AppPage::Routing);
    }

    #[test]
    fn midi_io_focus_toggles_between_lists() {
        assert_eq!(MidiIoListFocus::Inputs.toggle(), MidiIoListFocus::Outputs);
    }

    #[test]
    fn routing_fields_cycle() {
        assert_eq!(RoutingField::InputDevice.previous(), RoutingField::Passthrough);
        assert_eq!(RoutingField::Passthrough.next(), RoutingField::InputDevice);
    }

    #[test]
    fn mapping_page_mode_toggles() {
        assert_eq!(MappingPageMode::Overview.toggle(), MappingPageMode::Write);
        assert_eq!(MappingPageMode::Write.label(), "Write");
    }
}
