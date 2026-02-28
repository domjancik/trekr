/// Engine configuration is kept simple for the first compileable scaffold.
#[derive(Debug, Clone)]
pub struct EngineConfig {
    pub sample_rate_hz: u32,
    pub max_buffer_frames: u32,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            sample_rate_hz: 48_000,
            max_buffer_frames: 512,
        }
    }
}
