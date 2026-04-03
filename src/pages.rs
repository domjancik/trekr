use crate::timeline_fx::{TimelineContext, TimelineFxField};
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
        cycle_enum(Self::ALL, self, 1)
    }

    pub fn previous(self) -> Self {
        cycle_enum(Self::ALL, self, -1)
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
        cycle_enum(Self::ALL, self, 1)
    }

    pub fn previous(self) -> Self {
        cycle_enum(Self::ALL, self, -1)
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
    InputFxParam1,
    InputFxParam2,
    InputFxMore,
    OutputFxSlot,
    OutputFxKind,
    OutputFxEnabled,
    OutputFxParam1,
    OutputFxParam2,
    OutputFxMore,
}

impl RoutingField {
    pub const ALL: [Self; 19] = [
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
        Self::InputFxParam1,
        Self::InputFxParam2,
        Self::InputFxMore,
        Self::OutputFxSlot,
        Self::OutputFxKind,
        Self::OutputFxEnabled,
        Self::OutputFxParam1,
        Self::OutputFxParam2,
        Self::OutputFxMore,
    ];

    pub fn next(self) -> Self {
        cycle_enum(Self::ALL, self, 1)
    }

    pub fn previous(self) -> Self {
        cycle_enum(Self::ALL, self, -1)
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::InputDevice => "Input Dev",
            Self::InputChannel => "Input Ch",
            Self::OutputDevice => "Output Dev",
            Self::OutputChannel => "Output Ch",
            Self::Passthrough => "Passthrough",
            Self::RecordInputFx => "Rec FX",
            Self::MonitorInputFx => "Mon FX",
            Self::InputFxSlot => "Input Slot",
            Self::InputFxKind => "Input Kind",
            Self::InputFxEnabled => "Input On",
            Self::InputFxParam1 => "Input P1",
            Self::InputFxParam2 => "Input P2",
            Self::InputFxMore => "Input More",
            Self::OutputFxSlot => "Output Slot",
            Self::OutputFxKind => "Output Kind",
            Self::OutputFxEnabled => "Output On",
            Self::OutputFxParam1 => "Output P1",
            Self::OutputFxParam2 => "Output P2",
            Self::OutputFxMore => "Output More",
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
    pub selected_timeline_context: TimelineContext,
    pub selected_timeline_fx_field: TimelineFxField,
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
            selected_timeline_context: TimelineContext::default(),
            selected_timeline_fx_field: TimelineFxField::default(),
        }
    }
}

fn cycle_enum<T: Copy + Eq, const N: usize>(all: [T; N], current: T, delta: isize) -> T {
    let index = all.iter().position(|item| *item == current).unwrap_or(0) as isize;
    let len = N as isize;
    let next_index = (index + delta).rem_euclid(len) as usize;
    all[next_index]
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
            RoutingField::OutputFxMore
        );
        assert_eq!(
            RoutingField::Passthrough.next(),
            RoutingField::RecordInputFx
        );
        assert_eq!(RoutingField::OutputFxMore.next(), RoutingField::InputDevice);
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
