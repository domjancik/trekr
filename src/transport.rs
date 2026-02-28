#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuantizeMode {
    Off,
    Pulse,
    Sixteenth,
    Eighth,
    Quarter,
    Bar,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Transport {
    pub tempo_bpm: u16,
    pub ppqn: u16,
    pub quantize: QuantizeMode,
    pub playing: bool,
    pub recording: bool,
}

impl Default for Transport {
    fn default() -> Self {
        Self {
            tempo_bpm: 120,
            ppqn: 960,
            quantize: QuantizeMode::Sixteenth,
            playing: false,
            recording: false,
        }
    }
}
