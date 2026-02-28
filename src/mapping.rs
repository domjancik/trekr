#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MappingTarget {
    TransportPlay,
    TransportRecord,
    TrackArm,
    TrackMute,
    LoopSet,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MidiMapping {
    pub target: MappingTarget,
    pub track_index: Option<usize>,
}
