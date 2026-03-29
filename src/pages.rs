use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AppPage {
    Timeline,
    Mappings,
    MidiIo,
    Routing,
}

impl AppPage {
    pub const ALL: [Self; 4] = [Self::Timeline, Self::Mappings, Self::MidiIo, Self::Routing];

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MappingPageMode {
    Overview,
    Write,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MappingField {
    SourceKind,
    SourceDevice,
    SourceValue,
    Target,
    Scope,
    Enabled,
}

impl MappingField {
    pub const ALL: [Self; 6] = [
        Self::SourceKind,
        Self::SourceDevice,
        Self::SourceValue,
        Self::Target,
        Self::Scope,
        Self::Enabled,
    ];

    pub fn next(self) -> Self {
        match self {
            Self::SourceKind => Self::SourceDevice,
            Self::SourceDevice => Self::SourceValue,
            Self::SourceValue => Self::Target,
            Self::Target => Self::Scope,
            Self::Scope => Self::Enabled,
            Self::Enabled => Self::SourceKind,
        }
    }

    pub fn previous(self) -> Self {
        match self {
            Self::SourceKind => Self::Enabled,
            Self::SourceDevice => Self::SourceKind,
            Self::SourceValue => Self::SourceDevice,
            Self::Target => Self::SourceValue,
            Self::Scope => Self::Target,
            Self::Enabled => Self::Scope,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::SourceKind => "Type",
            Self::SourceDevice => "Device",
            Self::SourceValue => "Source",
            Self::Target => "Target",
            Self::Scope => "Scope",
            Self::Enabled => "On",
        }
    }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoutingField {
    InputDevice,
    InputChannel,
    OutputDevice,
    OutputChannel,
    Passthrough,
    RecordInputFx,
    MonitorInputFx,
    InputFxSlot,
    InputFxKind,
    InputFxEnabled,
    InputFxValue,
    OutputFxSlot,
    OutputFxKind,
    OutputFxEnabled,
    OutputFxValue,
}

impl RoutingField {
    pub const ALL: [Self; 15] = [
        Self::InputDevice,
        Self::InputChannel,
        Self::OutputDevice,
        Self::OutputChannel,
        Self::Passthrough,
        Self::RecordInputFx,
        Self::MonitorInputFx,
        Self::InputFxSlot,
        Self::InputFxKind,
        Self::InputFxEnabled,
        Self::InputFxValue,
        Self::OutputFxSlot,
        Self::OutputFxKind,
        Self::OutputFxEnabled,
        Self::OutputFxValue,
    ];

    pub fn next(self) -> Self {
        match self {
            Self::InputDevice => Self::InputChannel,
            Self::InputChannel => Self::OutputDevice,
            Self::OutputDevice => Self::OutputChannel,
            Self::OutputChannel => Self::Passthrough,
            Self::Passthrough => Self::RecordInputFx,
            Self::RecordInputFx => Self::MonitorInputFx,
            Self::MonitorInputFx => Self::InputFxSlot,
            Self::InputFxSlot => Self::InputFxKind,
            Self::InputFxKind => Self::InputFxEnabled,
            Self::InputFxEnabled => Self::InputFxValue,
            Self::InputFxValue => Self::OutputFxSlot,
            Self::OutputFxSlot => Self::OutputFxKind,
            Self::OutputFxKind => Self::OutputFxEnabled,
            Self::OutputFxEnabled => Self::OutputFxValue,
            Self::OutputFxValue => Self::InputDevice,
        }
    }

    pub fn previous(self) -> Self {
        match self {
            Self::InputDevice => Self::Passthrough,
            Self::InputChannel => Self::InputDevice,
            Self::OutputDevice => Self::InputChannel,
            Self::OutputChannel => Self::OutputDevice,
            Self::Passthrough => Self::OutputChannel,
            Self::RecordInputFx => Self::Passthrough,
            Self::MonitorInputFx => Self::RecordInputFx,
            Self::InputFxSlot => Self::MonitorInputFx,
            Self::InputFxKind => Self::InputFxSlot,
            Self::InputFxEnabled => Self::InputFxKind,
            Self::InputFxValue => Self::InputFxEnabled,
            Self::OutputFxSlot => Self::InputFxValue,
            Self::OutputFxKind => Self::OutputFxSlot,
            Self::OutputFxEnabled => Self::OutputFxKind,
            Self::OutputFxValue => Self::OutputFxEnabled,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::InputDevice => "Input Device",
            Self::InputChannel => "Input Channel",
            Self::OutputDevice => "Output Device",
            Self::OutputChannel => "Output Channel",
            Self::Passthrough => "Passthrough",
            Self::RecordInputFx => "Rec FX",
            Self::MonitorInputFx => "Mon FX",
            Self::InputFxSlot => "InFX Slot",
            Self::InputFxKind => "InFX Kind",
            Self::InputFxEnabled => "InFX On",
            Self::InputFxValue => "InFX Val",
            Self::OutputFxSlot => "OutFX Slot",
            Self::OutputFxKind => "OutFX Kind",
            Self::OutputFxEnabled => "OutFX On",
            Self::OutputFxValue => "OutFX Val",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct AppPageState {
    pub current_page: AppPage,
    pub midi_io: MidiIoPageState,
    pub selected_mapping_index: usize,
    pub mapping_mode: MappingPageMode,
    pub selected_mapping_field: MappingField,
    pub mapping_midi_learn_armed: bool,
    pub selected_routing_field: RoutingField,
    pub selected_input_fx_slot: usize,
    pub selected_output_fx_slot: usize,
}

impl Default for AppPageState {
    fn default() -> Self {
        Self {
            current_page: AppPage::Timeline,
            midi_io: MidiIoPageState::default(),
            selected_mapping_index: 0,
            mapping_mode: MappingPageMode::Overview,
            selected_mapping_field: MappingField::SourceValue,
            mapping_midi_learn_armed: false,
            selected_routing_field: RoutingField::InputDevice,
            selected_input_fx_slot: 0,
            selected_output_fx_slot: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AppPage, MappingField, MappingPageMode, MidiIoListFocus, RoutingField};

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
        assert_eq!(
            RoutingField::InputDevice.previous(),
            RoutingField::OutputFxValue
        );
        assert_eq!(
            RoutingField::Passthrough.next(),
            RoutingField::RecordInputFx
        );
        assert_eq!(
            RoutingField::OutputFxValue.next(),
            RoutingField::InputDevice
        );
    }

    #[test]
    fn mapping_page_mode_toggles() {
        assert_eq!(MappingPageMode::Overview.toggle(), MappingPageMode::Write);
        assert_eq!(MappingPageMode::Write.label(), "Write");
    }

    #[test]
    fn mapping_fields_cycle() {
        assert_eq!(MappingField::SourceKind.previous(), MappingField::Enabled);
        assert_eq!(MappingField::Enabled.next(), MappingField::SourceKind);
    }
}
