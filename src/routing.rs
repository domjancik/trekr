use crate::midi_io::MidiPortRef;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct TrackRouting {
    pub input_port: Option<MidiPortRef>,
    pub output_port: Option<MidiPortRef>,
    pub input_channel: MidiChannelFilter,
    pub output_channel: Option<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum MidiChannelFilter {
    #[default]
    Omni,
    Channel(u8),
}
