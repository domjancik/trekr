use crate::midi_fx::MidiFxChainKind;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TimelineContext {
    InputFx,
    #[default]
    TrackTimeline,
    OutputFx,
}

impl TimelineContext {
    pub const ALL: [Self; 3] = [Self::InputFx, Self::TrackTimeline, Self::OutputFx];

    pub fn previous(self) -> Self {
        cycle_enum(Self::ALL, self, -1)
    }

    pub fn next(self) -> Self {
        cycle_enum(Self::ALL, self, 1)
    }

    pub fn chain_kind(self) -> Option<MidiFxChainKind> {
        match self {
            Self::InputFx => Some(MidiFxChainKind::Input),
            Self::TrackTimeline => None,
            Self::OutputFx => Some(MidiFxChainKind::Output),
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::InputFx => "Input FX",
            Self::TrackTimeline => "Timeline",
            Self::OutputFx => "Output FX",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TimelineFxField {
    Enabled,
    Kind,
    #[default]
    ParamPrimary,
    ParamSecondary,
    Scroll,
    Move,
}

impl TimelineFxField {
    pub const ALL: [Self; 6] = [
        Self::Enabled,
        Self::Kind,
        Self::ParamPrimary,
        Self::ParamSecondary,
        Self::Scroll,
        Self::Move,
    ];

    pub fn previous(self) -> Self {
        cycle_enum(Self::ALL, self, -1)
    }

    pub fn next(self) -> Self {
        cycle_enum(Self::ALL, self, 1)
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Enabled => "On",
            Self::Kind => "Kind",
            Self::ParamPrimary => "P1",
            Self::ParamSecondary => "P2",
            Self::Scroll => "More",
            Self::Move => "Move",
        }
    }
}

fn cycle_enum<T: Copy + Eq, const N: usize>(all: [T; N], current: T, delta: isize) -> T {
    let index = all.iter().position(|item| *item == current).unwrap_or(0) as isize;
    let len = N as isize;
    let next_index = (index + delta).rem_euclid(len) as usize;
    all[next_index]
}
