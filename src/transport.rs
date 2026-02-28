use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuantizeMode {
    Off,
    Pulse,
    Sixteenth,
    Eighth,
    Quarter,
    Bar,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecordMode {
    Overdub,
    Replace,
}

impl RecordMode {
    pub fn next(self) -> Self {
        match self {
            Self::Overdub => Self::Replace,
            Self::Replace => Self::Overdub,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Overdub => "Overdub",
            Self::Replace => "Replace",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Transport {
    pub tempo_bpm: u16,
    pub ppqn: u16,
    pub quantize: QuantizeMode,
    pub record_mode: RecordMode,
    pub loop_enabled: bool,
    pub playing: bool,
    pub recording: bool,
}

impl Default for Transport {
    fn default() -> Self {
        Self {
            tempo_bpm: 120,
            ppqn: 960,
            quantize: QuantizeMode::Sixteenth,
            record_mode: RecordMode::Overdub,
            loop_enabled: true,
            playing: false,
            recording: false,
        }
    }
}

impl Transport {
    pub fn ticks_per_second(self) -> u64 {
        (u64::from(self.tempo_bpm) * u64::from(self.ppqn)) / 60
    }

    pub fn quantize_step_ticks(self) -> Option<u64> {
        match self.quantize {
            QuantizeMode::Off => None,
            QuantizeMode::Pulse => Some(1),
            QuantizeMode::Sixteenth => Some((self.ppqn / 4) as u64),
            QuantizeMode::Eighth => Some((self.ppqn / 2) as u64),
            QuantizeMode::Quarter => Some(self.ppqn as u64),
            QuantizeMode::Bar => Some((self.ppqn * 4) as u64),
        }
    }

    /// Hold-to-record commits on release to the nearest quantize boundary.
    pub fn quantize_to_nearest(self, ticks: u64) -> u64 {
        let Some(step) = self.quantize_step_ticks() else {
            return ticks;
        };

        let lower = (ticks / step) * step;
        let upper = lower + step;

        if ticks - lower < upper - ticks {
            lower
        } else {
            upper
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{QuantizeMode, RecordMode, Transport};

    #[test]
    fn quantize_to_nearest_prefers_nearest_boundary() {
        let transport = Transport {
            quantize: QuantizeMode::Sixteenth,
            ..Transport::default()
        };

        assert_eq!(transport.quantize_to_nearest(200), 240);
        assert_eq!(transport.quantize_to_nearest(260), 240);
    }

    #[test]
    fn quantize_off_returns_original_ticks() {
        let transport = Transport {
            quantize: QuantizeMode::Off,
            ..Transport::default()
        };

        assert_eq!(transport.quantize_to_nearest(257), 257);
    }

    #[test]
    fn ticks_per_second_uses_tempo_and_ppqn() {
        let transport = Transport::default();
        assert_eq!(transport.ticks_per_second(), 1_920);
    }

    #[test]
    fn record_mode_cycles() {
        assert_eq!(RecordMode::Overdub.next(), RecordMode::Replace);
        assert_eq!(RecordMode::Replace.label(), "Replace");
    }
}
