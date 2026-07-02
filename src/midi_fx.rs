use crate::project::MidiNote;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

pub const MIDI_FX_SLOT_COUNT: usize = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum RecordInputFxMode {
    #[default]
    DryInput,
    PostInputFx,
}

impl RecordInputFxMode {
    pub fn toggle(self) -> Self {
        match self {
            Self::DryInput => Self::PostInputFx,
            Self::PostInputFx => Self::DryInput,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::DryInput => "Dry",
            Self::PostInputFx => "Post FX",
        }
    }
}

impl Hash for RecordInputFxMode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (*self as u8).hash(state);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrackMidiFx {
    #[serde(default)]
    pub record_input_fx_mode: RecordInputFxMode,
    #[serde(default = "default_monitor_input_fx")]
    pub monitor_input_fx: bool,
    #[serde(default = "default_fx_slots")]
    pub input_fx: Vec<Option<MidiFxSlot>>,
    #[serde(default = "default_fx_slots")]
    pub output_fx: Vec<Option<MidiFxSlot>>,
    #[serde(default)]
    pub timeline_ui: TimelineFxUiState,
}

fn default_monitor_input_fx() -> bool {
    true
}

fn default_fx_slots() -> Vec<Option<MidiFxSlot>> {
    vec![None; MIDI_FX_SLOT_COUNT]
}

impl Default for TrackMidiFx {
    fn default() -> Self {
        Self {
            record_input_fx_mode: RecordInputFxMode::default(),
            monitor_input_fx: default_monitor_input_fx(),
            input_fx: default_fx_slots(),
            output_fx: default_fx_slots(),
            timeline_ui: TimelineFxUiState::default(),
        }
    }
}

impl TrackMidiFx {
    pub(crate) fn clone_for_runtime(&self) -> Self {
        Self {
            record_input_fx_mode: self.record_input_fx_mode,
            monitor_input_fx: self.monitor_input_fx,
            input_fx: self.input_fx.clone(),
            output_fx: self.output_fx.clone(),
            timeline_ui: TimelineFxUiState::default(),
        }
    }

    pub(crate) fn midi_runtime_signature(&self, hasher: &mut impl Hasher) {
        self.record_input_fx_mode.hash(hasher);
        self.monitor_input_fx.hash(hasher);
        hash_midi_fx_chain(&self.input_fx, hasher);
        hash_midi_fx_chain(&self.output_fx, hasher);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimelineFxUiState {
    #[serde(default)]
    pub input_selected_row: usize,
    #[serde(default)]
    pub output_selected_row: usize,
    #[serde(default = "default_fx_param_windows")]
    pub input_param_windows: Vec<usize>,
    #[serde(default = "default_fx_param_windows")]
    pub output_param_windows: Vec<usize>,
    #[serde(default)]
    pub input_row_window: usize,
    #[serde(default)]
    pub output_row_window: usize,
}

fn default_fx_param_windows() -> Vec<usize> {
    vec![0; MIDI_FX_SLOT_COUNT]
}

impl Default for TimelineFxUiState {
    fn default() -> Self {
        Self {
            input_selected_row: 0,
            output_selected_row: 0,
            input_param_windows: default_fx_param_windows(),
            output_param_windows: default_fx_param_windows(),
            input_row_window: 0,
            output_row_window: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MidiFxChainKind {
    Input,
    Output,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MidiFxSlot {
    #[serde(default = "default_fx_enabled")]
    pub enabled: bool,
    pub effect: MidiFx,
}

fn default_fx_enabled() -> bool {
    true
}

impl Default for MidiFxSlot {
    fn default() -> Self {
        Self {
            enabled: true,
            effect: MidiFx::Transpose { semitones: 0 },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MidiFxKind {
    Arp,
    NoteFilter,
    Transpose,
    Velocity,
    Duration,
    ScaleQuantize,
    ChordQuantize,
    #[serde(alias = "TimeShift")]
    Delay,
    TrackClone,
}

impl MidiFxKind {
    pub const ALL: [Self; 9] = [
        Self::Arp,
        Self::NoteFilter,
        Self::Transpose,
        Self::Velocity,
        Self::Duration,
        Self::ScaleQuantize,
        Self::ChordQuantize,
        Self::Delay,
        Self::TrackClone,
    ];

    pub const ALL_WITH_NONE: [Option<Self>; 10] = [
        None,
        Some(Self::Arp),
        Some(Self::NoteFilter),
        Some(Self::Transpose),
        Some(Self::Velocity),
        Some(Self::Duration),
        Some(Self::ScaleQuantize),
        Some(Self::ChordQuantize),
        Some(Self::Delay),
        Some(Self::TrackClone),
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Arp => "Arp",
            Self::NoteFilter => "Filter",
            Self::Transpose => "Transpose",
            Self::Velocity => "Velocity",
            Self::Duration => "Duration",
            Self::ScaleQuantize => "Scale",
            Self::ChordQuantize => "Chord",
            Self::Delay => "Delay",
            Self::TrackClone => "Clone",
        }
    }

    pub fn short_label(self) -> &'static str {
        match self {
            Self::Arp => "ARP",
            Self::NoteFilter => "FLT",
            Self::Transpose => "TRN",
            Self::Velocity => "VEL",
            Self::Duration => "DUR",
            Self::ScaleQuantize => "SCL",
            Self::ChordQuantize => "CHD",
            Self::Delay => "DLY",
            Self::TrackClone => "CLN",
        }
    }

    pub fn compact_label(self) -> &'static str {
        match self {
            Self::Arp => "AR",
            Self::NoteFilter => "FL",
            Self::Transpose => "TR",
            Self::Velocity => "VE",
            Self::Duration => "DU",
            Self::ScaleQuantize => "SC",
            Self::ChordQuantize => "CH",
            Self::Delay => "DL",
            Self::TrackClone => "CL",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MidiFx {
    Arp {
        step_ticks: u64,
        order: ArpOrder,
        gate_percent: u8,
    },
    NoteFilter {
        low: u8,
        high: u8,
        enabled_notes: Vec<u8>,
    },
    Transpose {
        semitones: i8,
    },
    Velocity {
        percent: u16,
    },
    Duration {
        ticks: u64,
    },
    ScaleQuantize {
        root: u8,
        #[serde(default)]
        target: QuantizeTarget,
    },
    ChordQuantize {
        root: u8,
        #[serde(default)]
        target: QuantizeTarget,
    },
    #[serde(alias = "TimeShift")]
    Delay {
        ticks: u64,
    },
    TrackClone {
        source_track: usize,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ArpOrder {
    #[default]
    Up,
    Down,
    UpDown,
    AsPlayed,
}

impl Hash for ArpOrder {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (*self as u8).hash(state);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum QuantizeTarget {
    #[default]
    Local,
    Global,
}

impl Hash for QuantizeTarget {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (*self as u8).hash(state);
    }
}

impl QuantizeTarget {
    pub fn label(self) -> &'static str {
        match self {
            Self::Local => "Loc",
            Self::Global => "Gbl",
        }
    }

    pub fn cycle(self, delta: i32) -> Self {
        const ALL: [QuantizeTarget; 2] = [QuantizeTarget::Local, QuantizeTarget::Global];
        let index = ALL
            .iter()
            .position(|candidate| *candidate == self)
            .unwrap_or(0);
        ALL[(index as i32 + delta).rem_euclid(ALL.len() as i32) as usize]
    }
}

impl ArpOrder {
    pub fn label(self) -> &'static str {
        match self {
            Self::Up => "Up",
            Self::Down => "Down",
            Self::UpDown => "UpDn",
            Self::AsPlayed => "Play",
        }
    }

    pub fn cycle(self, delta: i32) -> Self {
        const ALL: [ArpOrder; 4] = [
            ArpOrder::Up,
            ArpOrder::Down,
            ArpOrder::UpDown,
            ArpOrder::AsPlayed,
        ];
        let index = ALL
            .iter()
            .position(|candidate| *candidate == self)
            .unwrap_or(0);
        ALL[(index as i32 + delta).rem_euclid(ALL.len() as i32) as usize]
    }
}

fn hash_midi_fx_chain(chain: &[Option<MidiFxSlot>], hasher: &mut impl Hasher) {
    chain.len().hash(hasher);
    for slot in chain {
        match slot {
            Some(slot) => {
                true.hash(hasher);
                slot.enabled.hash(hasher);
                hash_midi_fx(&slot.effect, hasher);
            }
            None => false.hash(hasher),
        }
    }
}

fn hash_midi_fx(effect: &MidiFx, hasher: &mut impl Hasher) {
    match effect {
        MidiFx::Arp {
            step_ticks,
            order,
            gate_percent,
        } => {
            0_u8.hash(hasher);
            step_ticks.hash(hasher);
            order.hash(hasher);
            gate_percent.hash(hasher);
        }
        MidiFx::NoteFilter {
            low,
            high,
            enabled_notes,
        } => {
            1_u8.hash(hasher);
            low.hash(hasher);
            high.hash(hasher);
            enabled_notes.hash(hasher);
        }
        MidiFx::Transpose { semitones } => {
            2_u8.hash(hasher);
            semitones.hash(hasher);
        }
        MidiFx::Velocity { percent } => {
            3_u8.hash(hasher);
            percent.hash(hasher);
        }
        MidiFx::Duration { ticks } => {
            4_u8.hash(hasher);
            ticks.hash(hasher);
        }
        MidiFx::ScaleQuantize { root, target } => {
            5_u8.hash(hasher);
            root.hash(hasher);
            target.hash(hasher);
        }
        MidiFx::ChordQuantize { root, target } => {
            6_u8.hash(hasher);
            root.hash(hasher);
            target.hash(hasher);
        }
        MidiFx::Delay { ticks } => {
            7_u8.hash(hasher);
            ticks.hash(hasher);
        }
        MidiFx::TrackClone { source_track } => {
            8_u8.hash(hasher);
            source_track.hash(hasher);
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MidiFxInlineParam {
    pub label: &'static str,
    pub value: String,
}

impl MidiFx {
    pub fn default_for_kind(kind: MidiFxKind) -> Self {
        match kind {
            MidiFxKind::Arp => Self::Arp {
                step_ticks: 240,
                order: ArpOrder::Up,
                gate_percent: 100,
            },
            MidiFxKind::NoteFilter => Self::NoteFilter {
                low: 0,
                high: 127,
                enabled_notes: Vec::new(),
            },
            MidiFxKind::Transpose => Self::Transpose { semitones: 0 },
            MidiFxKind::Velocity => Self::Velocity { percent: 100 },
            MidiFxKind::Duration => Self::Duration { ticks: 0 },
            MidiFxKind::ScaleQuantize => Self::ScaleQuantize {
                root: 0,
                target: QuantizeTarget::Local,
            },
            MidiFxKind::ChordQuantize => Self::ChordQuantize {
                root: 0,
                target: QuantizeTarget::Local,
            },
            MidiFxKind::Delay => Self::Delay { ticks: 0 },
            MidiFxKind::TrackClone => Self::TrackClone { source_track: 0 },
        }
    }

    pub fn kind(&self) -> MidiFxKind {
        match self {
            Self::Arp { .. } => MidiFxKind::Arp,
            Self::NoteFilter { .. } => MidiFxKind::NoteFilter,
            Self::Transpose { .. } => MidiFxKind::Transpose,
            Self::Velocity { .. } => MidiFxKind::Velocity,
            Self::Duration { .. } => MidiFxKind::Duration,
            Self::ScaleQuantize { .. } => MidiFxKind::ScaleQuantize,
            Self::ChordQuantize { .. } => MidiFxKind::ChordQuantize,
            Self::Delay { .. } => MidiFxKind::Delay,
            Self::TrackClone { .. } => MidiFxKind::TrackClone,
        }
    }

    pub fn value_label(&self) -> String {
        match self {
            Self::Arp {
                step_ticks, order, ..
            } => format!("{} {}", arp_rate_label(*step_ticks), order.label()),
            Self::NoteFilter {
                low,
                high,
                enabled_notes,
            } => {
                if enabled_notes.is_empty() {
                    format!("{low}-{high}")
                } else {
                    format!("{low}-{high}/{}", enabled_notes.len())
                }
            }
            Self::Transpose { semitones } => format!("{:+}", semitones),
            Self::Velocity { percent } => format!("{percent}%"),
            Self::Duration { ticks } => duration_rate_label(*ticks).to_string(),
            Self::ScaleQuantize { root, target } => {
                format!("{} {}", note_name(*root), target.label())
            }
            Self::ChordQuantize { root, target } => {
                format!("{} {}", note_name(*root), target.label())
            }
            Self::Delay { ticks } => delay_rate_label(*ticks).to_string(),
            Self::TrackClone { source_track } => format!("T{}", source_track + 1),
        }
    }

    pub fn summary(&self) -> String {
        format!("{} {}", self.kind().short_label(), self.value_label())
    }

    pub fn inline_parameters(&self) -> Vec<MidiFxInlineParam> {
        match self {
            Self::Arp {
                step_ticks,
                order,
                gate_percent,
            } => vec![
                MidiFxInlineParam {
                    label: "Rate",
                    value: arp_rate_label(*step_ticks).to_string(),
                },
                MidiFxInlineParam {
                    label: "Ord",
                    value: order.label().to_string(),
                },
                MidiFxInlineParam {
                    label: "Gate",
                    value: format!("{gate_percent}%"),
                },
            ],
            Self::NoteFilter {
                low,
                high,
                enabled_notes,
            } => vec![
                MidiFxInlineParam {
                    label: "Low",
                    value: low.to_string(),
                },
                MidiFxInlineParam {
                    label: "High",
                    value: high.to_string(),
                },
                MidiFxInlineParam {
                    label: "List",
                    value: if enabled_notes.is_empty() {
                        "All".to_string()
                    } else {
                        enabled_notes.len().to_string()
                    },
                },
            ],
            Self::Transpose { semitones } => vec![MidiFxInlineParam {
                label: "Semi",
                value: format!("{:+}", semitones),
            }],
            Self::Velocity { percent } => vec![MidiFxInlineParam {
                label: "Vel",
                value: format!("{percent}%"),
            }],
            Self::Duration { ticks } => vec![MidiFxInlineParam {
                label: "Dur",
                value: duration_rate_label(*ticks).to_string(),
            }],
            Self::ScaleQuantize { root, target } => vec![
                MidiFxInlineParam {
                    label: "Root",
                    value: note_name(*root).to_string(),
                },
                MidiFxInlineParam {
                    label: "Tgt",
                    value: target.label().to_string(),
                },
            ],
            Self::ChordQuantize { root, target } => vec![
                MidiFxInlineParam {
                    label: "Root",
                    value: note_name(*root).to_string(),
                },
                MidiFxInlineParam {
                    label: "Tgt",
                    value: target.label().to_string(),
                },
            ],
            Self::Delay { ticks } => vec![MidiFxInlineParam {
                label: "Dly",
                value: delay_rate_label(*ticks).to_string(),
            }],
            Self::TrackClone { source_track } => vec![MidiFxInlineParam {
                label: "Src",
                value: format!("T{}", source_track + 1),
            }],
        }
    }

    pub fn adjust_inline_parameter(
        &mut self,
        param_index: usize,
        delta: i32,
        track_count: usize,
        ppqn: u16,
    ) {
        match self {
            Self::Transpose { .. }
            | Self::Velocity { .. }
            | Self::Duration { .. }
            | Self::Delay { .. }
            | Self::TrackClone { .. } => self.adjust_value(delta, track_count, ppqn),
            Self::ScaleQuantize { root, target } | Self::ChordQuantize { root, target } => {
                if param_index == 0 {
                    *root = ((*root as i32 + delta).rem_euclid(12)) as u8;
                } else {
                    *target = target.cycle(delta);
                }
            }
            Self::Arp {
                step_ticks,
                order,
                gate_percent,
            } => match param_index {
                0 => {
                    let steps = arp_step_choices(ppqn);
                    *step_ticks = cycle_u64_choice(*step_ticks, &steps, delta);
                }
                1 => *order = order.cycle(delta),
                _ => *gate_percent = (*gate_percent as i32 + delta * 10).clamp(10, 100) as u8,
            },
            Self::NoteFilter {
                low,
                high,
                enabled_notes,
            } => match param_index {
                0 => *low = (*low as i32 + delta).clamp(0, i32::from(*high)) as u8,
                1 => *high = (*high as i32 + delta).clamp(i32::from(*low), 127) as u8,
                _ => adjust_enabled_note_list(enabled_notes, *low, *high, delta),
            },
        }
    }

    pub fn adjust_value(&mut self, delta: i32, track_count: usize, ppqn: u16) {
        match self {
            Self::Arp { step_ticks, .. } => {
                let steps = arp_step_choices(ppqn);
                *step_ticks = cycle_u64_choice(*step_ticks, &steps, delta);
            }
            Self::NoteFilter {
                low,
                high,
                enabled_notes,
            } => {
                if enabled_notes.is_empty() {
                    if delta < 0 {
                        *low = low.saturating_sub(1);
                    } else {
                        *high = high.saturating_add(1).min(127);
                    }
                    if *low > *high {
                        *low = *high;
                    }
                } else {
                    adjust_enabled_note_list(enabled_notes, *low, *high, delta);
                }
            }
            Self::Transpose { semitones } => {
                *semitones = (*semitones as i32 + delta).clamp(-24, 24) as i8;
            }
            Self::Velocity { percent } => {
                *percent = (*percent as i32 + delta * 10).clamp(0, 300) as u16;
            }
            Self::Duration { ticks } => {
                let steps = duration_step_choices(ppqn);
                *ticks = step_u64_choice(*ticks, &steps, delta);
            }
            Self::ScaleQuantize { root, .. } | Self::ChordQuantize { root, .. } => {
                *root = ((*root as i32 + delta).rem_euclid(12)) as u8;
            }
            Self::Delay { ticks } => {
                let steps = delay_step_choices(ppqn);
                *ticks = step_u64_choice(*ticks, &steps, delta);
            }
            Self::TrackClone { source_track } => {
                let count = track_count.max(1) as i32;
                *source_track = ((*source_track as i32 + delta).rem_euclid(count)) as usize;
            }
        }
    }
}

fn adjust_enabled_note_list(enabled_notes: &mut Vec<u8>, low: u8, high: u8, delta: i32) {
    if delta == 0 || low > high {
        return;
    }

    if enabled_notes.is_empty() {
        if delta > 0 {
            return;
        }
        enabled_notes.extend(low..=high);
    }

    if delta < 0 {
        if enabled_notes.len() > 1 {
            enabled_notes.pop();
        }
        return;
    }

    let current = enabled_notes.last().copied().unwrap_or(low);
    if current < high {
        enabled_notes.push(current + 1);
    }
    enabled_notes.sort_unstable();
    enabled_notes.dedup();
}

fn cycle_u64_choice(current: u64, options: &[u64], delta: i32) -> u64 {
    let current_index = options
        .iter()
        .position(|candidate| *candidate == current)
        .unwrap_or(0);
    let next_index = (current_index as i32 + delta).rem_euclid(options.len() as i32) as usize;
    options[next_index]
}

fn step_u64_choice(current: u64, options: &[u64], delta: i32) -> u64 {
    let current_index = options
        .iter()
        .position(|candidate| *candidate == current)
        .unwrap_or(0);
    let next_index =
        (current_index as i32 + delta).clamp(0, options.len().saturating_sub(1) as i32) as usize;
    options[next_index]
}

pub fn note_name(root: u8) -> &'static str {
    match root % 12 {
        0 => "C",
        1 => "C#",
        2 => "D",
        3 => "D#",
        4 => "E",
        5 => "F",
        6 => "F#",
        7 => "G",
        8 => "G#",
        9 => "A",
        10 => "A#",
        _ => "B",
    }
}

fn arp_step_choices(ppqn: u16) -> Vec<u64> {
    let ppqn = u64::from(ppqn.max(1));
    vec![
        (ppqn / 16).max(1),
        (ppqn / 8).max(1),
        (ppqn / 4).max(1),
        (ppqn / 2).max(1),
        ppqn,
        ppqn.saturating_mul(2),
        ppqn.saturating_mul(4),
    ]
}

fn delay_step_choices(ppqn: u16) -> Vec<u64> {
    let mut steps = vec![0];
    steps.extend(arp_step_choices(ppqn));
    steps
}

fn duration_step_choices(ppqn: u16) -> Vec<u64> {
    let mut steps = vec![0];
    steps.extend(arp_step_choices(ppqn));
    steps
}

pub fn arp_rate_label(step_ticks: u64) -> &'static str {
    match step_ticks {
        60 => "1/64",
        120 => "1/32",
        240 => "1/16",
        480 => "1/8",
        960 => "1/4",
        1920 => "1/2",
        3840 => "1 Bar",
        _ => "?",
    }
}

pub fn delay_rate_label(step_ticks: u64) -> &'static str {
    if step_ticks == 0 {
        "Off"
    } else {
        arp_rate_label(step_ticks)
    }
}

pub fn duration_rate_label(step_ticks: u64) -> &'static str {
    if step_ticks == 0 {
        "Off"
    } else {
        arp_rate_label(step_ticks)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LiveMidiFxEvent {
    NoteOn { pitch: u8, velocity: u8 },
    NoteOff { pitch: u8 },
}

#[derive(Debug, Clone, Default)]
pub struct LiveMidiFxState {
    note_pitch_map: HashMap<u8, Vec<u8>>,
    scheduled_events: Vec<ScheduledLiveMidiFxEvent>,
    duration_controlled_notes: HashMap<u8, usize>,
    arp_held_notes: Vec<ArpHeldNote>,
    arp_sequence_counter: u64,
    arp_next_step_tick: Option<u64>,
    arp_pending_note_off_tick: Option<u64>,
    arp_active_note: Option<u8>,
    arp_cycle_index: usize,
    arp_direction_forward: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ArpHeldNote {
    pitch: u8,
    velocity: u8,
    sequence: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ScheduledLiveMidiFxEvent {
    tick: u64,
    event: PendingLiveMidiFxEvent,
    next_slot_index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PendingLiveMidiFxEvent {
    event: LiveMidiFxEvent,
    suppress_original_note_off_pitch: Option<u8>,
}

pub fn cycle_fx_kind(current: Option<&MidiFxSlot>, delta: i32) -> Option<MidiFxSlot> {
    let current_kind = current.map(|slot| slot.effect.kind());
    let current_index = MidiFxKind::ALL_WITH_NONE
        .iter()
        .position(|candidate| *candidate == current_kind)
        .unwrap_or(0);
    let next_index =
        (current_index as i32 + delta).rem_euclid(MidiFxKind::ALL_WITH_NONE.len() as i32) as usize;
    MidiFxKind::ALL_WITH_NONE[next_index].map(|kind| MidiFxSlot {
        enabled: true,
        effect: MidiFx::default_for_kind(kind),
    })
}

pub fn cycle_existing_fx_kind(current: &MidiFxSlot, delta: i32) -> MidiFxSlot {
    let current_index = MidiFxKind::ALL
        .iter()
        .position(|candidate| *candidate == current.effect.kind())
        .unwrap_or(0);
    let next_index =
        (current_index as i32 + delta).rem_euclid(MidiFxKind::ALL.len() as i32) as usize;
    MidiFxSlot {
        enabled: current.enabled,
        effect: MidiFx::default_for_kind(MidiFxKind::ALL[next_index]),
    }
}

pub fn fx_slot_label(slot: Option<&MidiFxSlot>) -> String {
    match slot {
        Some(slot) if slot.enabled => slot.effect.summary(),
        Some(slot) => format!("{} Off", slot.effect.kind().short_label()),
        None => "None".to_string(),
    }
}

pub fn process_live_event(
    chain: &[Option<MidiFxSlot>],
    state: &mut LiveMidiFxState,
    event: LiveMidiFxEvent,
    global_quantize_root: u8,
) -> Vec<LiveMidiFxEvent> {
    process_live_chain_event(chain, state, event, 0, global_quantize_root)
}

pub fn process_live_chain_event(
    chain: &[Option<MidiFxSlot>],
    state: &mut LiveMidiFxState,
    event: LiveMidiFxEvent,
    current_ticks: u64,
    global_quantize_root: u8,
) -> Vec<LiveMidiFxEvent> {
    process_live_events_from_index(
        chain,
        state,
        vec![PendingLiveMidiFxEvent {
            event,
            suppress_original_note_off_pitch: None,
        }],
        0,
        current_ticks,
        global_quantize_root,
    )
    .into_iter()
    .map(|pending| pending.event)
    .collect()
}

pub fn process_live_chain_tick(
    chain: &[Option<MidiFxSlot>],
    state: &mut LiveMidiFxState,
    _previous_ticks: u64,
    current_ticks: u64,
    global_quantize_root: u8,
) -> Vec<(u64, LiveMidiFxEvent)> {
    let mut output = Vec::new();
    loop {
        let next_tick = match (
            next_scheduled_event_tick(state),
            next_live_arp_tick(chain, state),
        ) {
            (Some(delay_tick), Some(arp_tick)) => Some(delay_tick.min(arp_tick)),
            (Some(delay_tick), None) => Some(delay_tick),
            (None, Some(arp_tick)) => Some(arp_tick),
            (None, None) => None,
        };
        let Some(next_tick) = next_tick else {
            break;
        };
        if next_tick > current_ticks {
            break;
        }
        for scheduled in take_scheduled_events_at_tick(state, next_tick) {
            let immediate = process_live_events_from_index(
                chain,
                state,
                vec![scheduled.event],
                scheduled.next_slot_index,
                next_tick,
                global_quantize_root,
            );
            for pending in immediate {
                if let (LiveMidiFxEvent::NoteOff { .. }, Some(pitch)) =
                    (&pending.event, pending.suppress_original_note_off_pitch)
                {
                    decrement_duration_controlled_note(state, pitch);
                }
                output.push((next_tick, pending.event));
            }
        }

        if next_live_arp_tick(chain, state) == Some(next_tick) {
            let arp_index = first_live_arp_index(chain).unwrap_or(0);
            let generated = collect_live_arp_events_at_tick(chain, state, next_tick);
            let processed = process_live_events_from_index(
                chain,
                state,
                generated
                    .into_iter()
                    .map(|event| PendingLiveMidiFxEvent {
                        event,
                        suppress_original_note_off_pitch: None,
                    })
                    .collect(),
                arp_index + 1,
                next_tick,
                global_quantize_root,
            );
            output.extend(
                processed
                    .into_iter()
                    .map(|pending| (next_tick, pending.event)),
            );
        }
    }
    output
}

pub fn reset_live_fx_timing(state: &mut LiveMidiFxState, current_ticks: u64) {
    state.scheduled_events.clear();
    state.arp_pending_note_off_tick = None;
    state.arp_active_note = None;
    state.arp_cycle_index = 0;
    state.arp_direction_forward = true;
    state.arp_next_step_tick = (!state.arp_held_notes.is_empty()).then_some(current_ticks);
}

fn first_live_arp_index(chain: &[Option<MidiFxSlot>]) -> Option<usize> {
    chain.iter().position(|slot| {
        slot.as_ref()
            .is_some_and(|slot| slot.enabled && matches!(slot.effect, MidiFx::Arp { .. }))
    })
}

fn process_live_events_from_index(
    chain: &[Option<MidiFxSlot>],
    state: &mut LiveMidiFxState,
    mut events: Vec<PendingLiveMidiFxEvent>,
    start_index: usize,
    current_ticks: u64,
    global_quantize_root: u8,
) -> Vec<PendingLiveMidiFxEvent> {
    for (slot_index, slot) in chain.iter().enumerate().skip(start_index) {
        let Some(slot) = slot.as_ref().filter(|slot| slot.enabled) else {
            continue;
        };
        if events.is_empty() {
            break;
        }
        match &slot.effect {
            MidiFx::Delay { ticks } if *ticks > 0 => {
                for event in events.drain(..) {
                    state.scheduled_events.push(ScheduledLiveMidiFxEvent {
                        tick: current_ticks.saturating_add(*ticks),
                        event,
                        next_slot_index: slot_index + 1,
                    });
                }
            }
            MidiFx::Duration { ticks } if *ticks > 0 => {
                let mut immediate = Vec::new();
                for pending in events.drain(..) {
                    match pending.event {
                        LiveMidiFxEvent::NoteOn { pitch, velocity } => {
                            *state.duration_controlled_notes.entry(pitch).or_default() += 1;
                            state.scheduled_events.push(ScheduledLiveMidiFxEvent {
                                tick: current_ticks.saturating_add(*ticks),
                                event: PendingLiveMidiFxEvent {
                                    event: LiveMidiFxEvent::NoteOff { pitch },
                                    suppress_original_note_off_pitch: Some(pitch),
                                },
                                next_slot_index: slot_index + 1,
                            });
                            immediate.push(PendingLiveMidiFxEvent {
                                event: LiveMidiFxEvent::NoteOn { pitch, velocity },
                                suppress_original_note_off_pitch: None,
                            });
                        }
                        LiveMidiFxEvent::NoteOff { pitch } => {
                            if state
                                .duration_controlled_notes
                                .get(&pitch)
                                .copied()
                                .unwrap_or(0)
                                == 0
                            {
                                immediate.push(PendingLiveMidiFxEvent {
                                    event: LiveMidiFxEvent::NoteOff { pitch },
                                    suppress_original_note_off_pitch: None,
                                });
                            }
                        }
                    }
                }
                events = immediate;
            }
            MidiFx::Arp { .. } => {
                let mut immediate = Vec::new();
                for pending in events.drain(..) {
                    if let Some(return_event) =
                        update_live_arp_held_notes(state, pending.event, current_ticks)
                    {
                        immediate.push(PendingLiveMidiFxEvent {
                            event: return_event,
                            suppress_original_note_off_pitch: pending
                                .suppress_original_note_off_pitch,
                        });
                    }
                }
                events = immediate;
            }
            _ => events = apply_live_fx(slot, state, events, global_quantize_root),
        }
    }
    events
}

fn next_scheduled_event_tick(state: &LiveMidiFxState) -> Option<u64> {
    state.scheduled_events.iter().map(|event| event.tick).min()
}

fn take_scheduled_events_at_tick(
    state: &mut LiveMidiFxState,
    tick: u64,
) -> Vec<ScheduledLiveMidiFxEvent> {
    let mut ready = Vec::new();
    let mut pending = Vec::with_capacity(state.scheduled_events.len());
    for event in state.scheduled_events.drain(..) {
        if event.tick == tick {
            ready.push(event);
        } else {
            pending.push(event);
        }
    }
    state.scheduled_events = pending;
    ready
}

fn next_live_arp_tick(chain: &[Option<MidiFxSlot>], state: &LiveMidiFxState) -> Option<u64> {
    let Some(_arp_index) = first_live_arp_index(chain) else {
        return None;
    };
    let next_off = state.arp_pending_note_off_tick;
    let next_on = state
        .arp_next_step_tick
        .filter(|_| !state.arp_held_notes.is_empty());
    match (next_off, next_on) {
        (Some(off), Some(on)) => Some(off.min(on)),
        (Some(off), None) => Some(off),
        (None, Some(on)) => Some(on),
        (None, None) => None,
    }
}

fn collect_live_arp_events_at_tick(
    chain: &[Option<MidiFxSlot>],
    state: &mut LiveMidiFxState,
    tick: u64,
) -> Vec<LiveMidiFxEvent> {
    let Some(arp_index) = first_live_arp_index(chain) else {
        return Vec::new();
    };
    let Some(MidiFx::Arp {
        step_ticks,
        order,
        gate_percent,
    }) = chain
        .get(arp_index)
        .and_then(|slot| slot.as_ref())
        .filter(|slot| slot.enabled)
        .map(|slot| &slot.effect)
    else {
        return Vec::new();
    };
    if *step_ticks == 0 {
        return Vec::new();
    }
    let gate_ticks =
        ((u128::from(*step_ticks) * u128::from(*gate_percent)) / 100).max(1_u128) as u64;
    let mut scheduled = Vec::new();
    loop {
        if state.arp_pending_note_off_tick == Some(tick) {
            if let Some(pitch) = state.arp_active_note.take() {
                scheduled.push(LiveMidiFxEvent::NoteOff { pitch });
            }
            state.arp_pending_note_off_tick = None;
            continue;
        }
        if state.arp_next_step_tick == Some(tick) {
            if let Some(pitch) = state.arp_active_note.take() {
                scheduled.push(LiveMidiFxEvent::NoteOff { pitch });
                state.arp_pending_note_off_tick = None;
            }
            if let Some((pitch, velocity)) = next_live_arp_note(state, *order) {
                scheduled.push(LiveMidiFxEvent::NoteOn { pitch, velocity });
                state.arp_active_note = Some(pitch);
                state.arp_pending_note_off_tick = Some(tick.saturating_add(gate_ticks));
                state.arp_next_step_tick = Some(tick.saturating_add(*step_ticks));
            } else {
                state.arp_next_step_tick = None;
            }
            continue;
        }
        break;
    }
    scheduled
}

fn update_live_arp_held_notes(
    state: &mut LiveMidiFxState,
    event: LiveMidiFxEvent,
    current_ticks: u64,
) -> Option<LiveMidiFxEvent> {
    match event {
        LiveMidiFxEvent::NoteOn { pitch, velocity } => {
            state.arp_sequence_counter = state.arp_sequence_counter.saturating_add(1);
            state.arp_held_notes.push(ArpHeldNote {
                pitch,
                velocity,
                sequence: state.arp_sequence_counter,
            });
            state
                .arp_held_notes
                .sort_by_key(|note| (note.pitch, note.sequence));
            if state.arp_next_step_tick.is_none() {
                state.arp_next_step_tick = Some(current_ticks);
            }
            None
        }
        LiveMidiFxEvent::NoteOff { pitch } => {
            if let Some(index) = state
                .arp_held_notes
                .iter()
                .position(|note| note.pitch == pitch)
            {
                state.arp_held_notes.remove(index);
            }
            if state.arp_held_notes.is_empty() {
                state.arp_cycle_index = 0;
                state.arp_direction_forward = true;
                state.arp_next_step_tick = None;
                state.arp_pending_note_off_tick = None;
                return state
                    .arp_active_note
                    .take()
                    .map(|pitch| LiveMidiFxEvent::NoteOff { pitch });
            }
            None
        }
    }
}

fn next_live_arp_note(state: &mut LiveMidiFxState, order: ArpOrder) -> Option<(u8, u8)> {
    if state.arp_held_notes.is_empty() {
        return None;
    }
    let mut notes = state.arp_held_notes.clone();
    match order {
        ArpOrder::Up | ArpOrder::UpDown => notes.sort_by_key(|note| (note.pitch, note.sequence)),
        ArpOrder::Down => notes.sort_by_key(|note| (std::cmp::Reverse(note.pitch), note.sequence)),
        ArpOrder::AsPlayed => notes.sort_by_key(|note| note.sequence),
    }
    let index = match order {
        ArpOrder::Up | ArpOrder::Down | ArpOrder::AsPlayed => {
            let index = state.arp_cycle_index % notes.len();
            state.arp_cycle_index = state.arp_cycle_index.saturating_add(1) % notes.len().max(1);
            index
        }
        ArpOrder::UpDown => {
            if notes.len() <= 1 {
                0
            } else {
                let index = state.arp_cycle_index.min(notes.len() - 1);
                if state.arp_direction_forward {
                    if state.arp_cycle_index + 1 >= notes.len() {
                        state.arp_direction_forward = false;
                        state.arp_cycle_index = notes.len().saturating_sub(2);
                    } else {
                        state.arp_cycle_index += 1;
                    }
                } else if state.arp_cycle_index == 0 {
                    state.arp_direction_forward = true;
                    state.arp_cycle_index = 1.min(notes.len() - 1);
                } else {
                    state.arp_cycle_index -= 1;
                }
                index
            }
        }
    };
    notes.get(index).map(|note| (note.pitch, note.velocity))
}

fn apply_live_fx(
    slot: &MidiFxSlot,
    state: &mut LiveMidiFxState,
    events: Vec<PendingLiveMidiFxEvent>,
    global_quantize_root: u8,
) -> Vec<PendingLiveMidiFxEvent> {
    let mut transformed = Vec::new();
    for pending in events {
        let suppress_original_note_off_pitch = pending.suppress_original_note_off_pitch;
        match (&slot.effect, pending.event) {
            (MidiFx::TrackClone { .. }, event)
            | (MidiFx::Arp { .. }, event)
            | (MidiFx::Duration { .. }, event)
            | (MidiFx::Delay { .. }, event) => transformed.push(PendingLiveMidiFxEvent {
                event,
                suppress_original_note_off_pitch,
            }),
            (
                MidiFx::NoteFilter {
                    low,
                    high,
                    enabled_notes,
                },
                LiveMidiFxEvent::NoteOn { pitch, velocity },
            ) => {
                if pitch < *low || pitch > *high {
                    continue;
                }
                if !enabled_notes.is_empty() && !enabled_notes.contains(&pitch) {
                    continue;
                }
                transformed.push(PendingLiveMidiFxEvent {
                    event: LiveMidiFxEvent::NoteOn { pitch, velocity },
                    suppress_original_note_off_pitch,
                });
            }
            (
                MidiFx::NoteFilter {
                    low,
                    high,
                    enabled_notes,
                },
                LiveMidiFxEvent::NoteOff { pitch },
            ) => {
                if pitch < *low || pitch > *high {
                    continue;
                }
                if !enabled_notes.is_empty() && !enabled_notes.contains(&pitch) {
                    continue;
                }
                transformed.push(PendingLiveMidiFxEvent {
                    event: LiveMidiFxEvent::NoteOff { pitch },
                    suppress_original_note_off_pitch,
                });
            }
            (MidiFx::Transpose { semitones }, LiveMidiFxEvent::NoteOn { pitch, velocity }) => {
                let transformed_pitch = transpose_pitch(pitch, *semitones);
                state
                    .note_pitch_map
                    .entry(pitch)
                    .or_default()
                    .push(transformed_pitch);
                transformed.push(PendingLiveMidiFxEvent {
                    event: LiveMidiFxEvent::NoteOn {
                        pitch: transformed_pitch,
                        velocity,
                    },
                    suppress_original_note_off_pitch,
                });
            }
            (MidiFx::Transpose { .. }, LiveMidiFxEvent::NoteOff { pitch }) => {
                if let Some(mapped) = state
                    .note_pitch_map
                    .get_mut(&pitch)
                    .and_then(|pitches| pitches.pop())
                {
                    transformed.push(PendingLiveMidiFxEvent {
                        event: LiveMidiFxEvent::NoteOff { pitch: mapped },
                        suppress_original_note_off_pitch,
                    });
                } else {
                    transformed.push(PendingLiveMidiFxEvent {
                        event: LiveMidiFxEvent::NoteOff { pitch },
                        suppress_original_note_off_pitch,
                    });
                }
            }
            (MidiFx::Velocity { percent }, LiveMidiFxEvent::NoteOn { pitch, velocity }) => {
                transformed.push(PendingLiveMidiFxEvent {
                    event: LiveMidiFxEvent::NoteOn {
                        pitch,
                        velocity: scale_percent(velocity, *percent),
                    },
                    suppress_original_note_off_pitch,
                });
            }
            (MidiFx::Velocity { .. }, LiveMidiFxEvent::NoteOff { pitch }) => {
                transformed.push(PendingLiveMidiFxEvent {
                    event: LiveMidiFxEvent::NoteOff { pitch },
                    suppress_original_note_off_pitch,
                });
            }
            (
                MidiFx::ScaleQuantize { root, target },
                LiveMidiFxEvent::NoteOn { pitch, velocity },
            ) => {
                let active_root = quantize_root(*root, *target, global_quantize_root);
                let quantized = quantize_to_scale(pitch, active_root);
                state
                    .note_pitch_map
                    .entry(pitch)
                    .or_default()
                    .push(quantized);
                transformed.push(PendingLiveMidiFxEvent {
                    event: LiveMidiFxEvent::NoteOn {
                        pitch: quantized,
                        velocity,
                    },
                    suppress_original_note_off_pitch,
                });
            }
            (MidiFx::ScaleQuantize { .. }, LiveMidiFxEvent::NoteOff { pitch }) => {
                if let Some(mapped) = state
                    .note_pitch_map
                    .get_mut(&pitch)
                    .and_then(|pitches| pitches.pop())
                {
                    transformed.push(PendingLiveMidiFxEvent {
                        event: LiveMidiFxEvent::NoteOff { pitch: mapped },
                        suppress_original_note_off_pitch,
                    });
                } else {
                    transformed.push(PendingLiveMidiFxEvent {
                        event: LiveMidiFxEvent::NoteOff { pitch },
                        suppress_original_note_off_pitch,
                    });
                }
            }
            (
                MidiFx::ChordQuantize { root, target },
                LiveMidiFxEvent::NoteOn { pitch, velocity },
            ) => {
                let active_root = quantize_root(*root, *target, global_quantize_root);
                let quantized = quantize_to_chord(pitch, active_root);
                state
                    .note_pitch_map
                    .entry(pitch)
                    .or_default()
                    .push(quantized);
                transformed.push(PendingLiveMidiFxEvent {
                    event: LiveMidiFxEvent::NoteOn {
                        pitch: quantized,
                        velocity,
                    },
                    suppress_original_note_off_pitch,
                });
            }
            (MidiFx::ChordQuantize { .. }, LiveMidiFxEvent::NoteOff { pitch }) => {
                if let Some(mapped) = state
                    .note_pitch_map
                    .get_mut(&pitch)
                    .and_then(|pitches| pitches.pop())
                {
                    transformed.push(PendingLiveMidiFxEvent {
                        event: LiveMidiFxEvent::NoteOff { pitch: mapped },
                        suppress_original_note_off_pitch,
                    });
                } else {
                    transformed.push(PendingLiveMidiFxEvent {
                        event: LiveMidiFxEvent::NoteOff { pitch },
                        suppress_original_note_off_pitch,
                    });
                }
            }
        }
    }
    transformed
}

fn decrement_duration_controlled_note(state: &mut LiveMidiFxState, pitch: u8) {
    let Some(count) = state.duration_controlled_notes.get_mut(&pitch) else {
        return;
    };
    if *count <= 1 {
        state.duration_controlled_notes.remove(&pitch);
    } else {
        *count -= 1;
    }
}

pub fn transform_notes(
    notes: &[MidiNote],
    chain: &[Option<MidiFxSlot>],
    global_quantize_root: u8,
) -> Vec<MidiNote> {
    let mut transformed = notes.to_vec();
    for slot in chain.iter().flatten().filter(|slot| slot.enabled) {
        transformed = apply_note_fx(slot, &transformed, global_quantize_root);
    }
    transformed
}

pub fn playback_timing_lookback_ticks(chain: &[Option<MidiFxSlot>]) -> u64 {
    let mut delay_ticks = 0_u64;
    let mut duration_ticks = None;
    for slot in chain.iter().flatten().filter(|slot| slot.enabled) {
        match slot.effect {
            MidiFx::Delay { ticks } => {
                delay_ticks = delay_ticks.saturating_add(ticks);
            }
            MidiFx::Duration { ticks } if ticks > 0 => {
                duration_ticks = Some(ticks);
            }
            _ => {}
        }
    }
    delay_ticks.saturating_add(duration_ticks.unwrap_or(0))
}

fn apply_note_fx(slot: &MidiFxSlot, notes: &[MidiNote], global_quantize_root: u8) -> Vec<MidiNote> {
    match &slot.effect {
        MidiFx::TrackClone { .. } => notes.to_vec(),
        MidiFx::Arp {
            step_ticks,
            order,
            gate_percent,
        } => apply_arp(notes, *step_ticks, *order, *gate_percent),
        MidiFx::NoteFilter {
            low,
            high,
            enabled_notes,
        } => notes
            .iter()
            .copied()
            .filter(|note| {
                note.pitch >= *low
                    && note.pitch <= *high
                    && (enabled_notes.is_empty() || enabled_notes.contains(&note.pitch))
            })
            .collect(),
        MidiFx::Transpose { semitones } => notes
            .iter()
            .copied()
            .map(|mut note| {
                note.pitch = transpose_pitch(note.pitch, *semitones);
                note
            })
            .collect(),
        MidiFx::Velocity { percent } => notes
            .iter()
            .copied()
            .map(|mut note| {
                note.velocity = scale_percent(note.velocity, *percent);
                note
            })
            .collect(),
        MidiFx::Duration { ticks } => {
            if *ticks == 0 {
                notes.to_vec()
            } else {
                notes
                    .iter()
                    .copied()
                    .map(|mut note| {
                        note.length_ticks = *ticks;
                        note
                    })
                    .collect()
            }
        }
        MidiFx::ScaleQuantize { root, target } => notes
            .iter()
            .copied()
            .map(|mut note| {
                let active_root = quantize_root(*root, *target, global_quantize_root);
                note.pitch = quantize_to_scale(note.pitch, active_root);
                note
            })
            .collect(),
        MidiFx::ChordQuantize { root, target } => notes
            .iter()
            .copied()
            .map(|mut note| {
                let active_root = quantize_root(*root, *target, global_quantize_root);
                note.pitch = quantize_to_chord(note.pitch, active_root);
                note
            })
            .collect(),
        MidiFx::Delay { ticks } => notes
            .iter()
            .copied()
            .map(|mut note| {
                note.start_ticks = note.start_ticks.saturating_add(*ticks);
                note
            })
            .collect(),
    }
}

fn apply_arp(
    notes: &[MidiNote],
    step_ticks: u64,
    order: ArpOrder,
    gate_percent: u8,
) -> Vec<MidiNote> {
    if step_ticks == 0 {
        return notes.to_vec();
    }

    let mut groups: HashMap<u64, Vec<MidiNote>> = HashMap::new();
    for note in notes {
        groups.entry(note.start_ticks).or_default().push(*note);
    }

    let mut transformed = Vec::new();
    for mut group in groups.into_values() {
        sort_arp_group(&mut group, order);
        if group.len() <= 1 {
            transformed.extend(group);
            continue;
        }
        let group_start = group.iter().map(|note| note.start_ticks).min().unwrap_or(0);
        let group_end = group
            .iter()
            .map(|note| note.end_ticks())
            .max()
            .unwrap_or(group_start);
        let gate_ticks = ((u128::from(step_ticks) * u128::from(gate_percent.clamp(10, 100))) / 100)
            .max(1) as u64;
        let sequence = arp_group_sequence(group.len(), order);
        let mut step_index = 0_usize;
        let mut current = group_start;
        while current < group_end {
            let note = &group[sequence[step_index % sequence.len()]];
            transformed.push(MidiNote {
                pitch: note.pitch,
                start_ticks: current,
                length_ticks: gate_ticks.min(group_end.saturating_sub(current)).max(1),
                velocity: note.velocity,
                recording_clip_id: note.recording_clip_id,
            });
            current = current.saturating_add(step_ticks);
            step_index += 1;
        }
    }
    transformed.sort_by_key(|note| (note.start_ticks, note.pitch));
    transformed
}

fn sort_arp_group(group: &mut [MidiNote], order: ArpOrder) {
    match order {
        ArpOrder::Up | ArpOrder::UpDown => group.sort_by_key(|note| note.pitch),
        ArpOrder::Down => group.sort_by_key(|note| std::cmp::Reverse(note.pitch)),
        ArpOrder::AsPlayed => {}
    }
}

fn arp_group_sequence(len: usize, order: ArpOrder) -> Vec<usize> {
    if len <= 1 {
        return vec![0];
    }
    match order {
        ArpOrder::Up | ArpOrder::AsPlayed => (0..len).collect(),
        ArpOrder::Down => (0..len).rev().collect(),
        ArpOrder::UpDown => {
            let mut sequence: Vec<usize> = (0..len).collect();
            sequence.extend((1..len.saturating_sub(1)).rev());
            sequence
        }
    }
}

fn transpose_pitch(pitch: u8, semitones: i8) -> u8 {
    (pitch as i16 + i16::from(semitones)).clamp(0, 127) as u8
}

fn scale_percent(value: u8, percent: u16) -> u8 {
    ((u16::from(value) * percent) / 100).clamp(0, 127) as u8
}

fn quantize_to_scale(pitch: u8, root: u8) -> u8 {
    quantize_to_allowed_steps(pitch, root, &[0, 2, 4, 5, 7, 9, 11])
}

fn quantize_to_chord(pitch: u8, root: u8) -> u8 {
    quantize_to_allowed_steps(pitch, root, &[0, 4, 7, 11])
}

fn quantize_root(local_root: u8, target: QuantizeTarget, global_root: u8) -> u8 {
    match target {
        QuantizeTarget::Local => local_root,
        QuantizeTarget::Global => global_root,
    }
}

fn quantize_to_allowed_steps(pitch: u8, root: u8, steps: &[u8]) -> u8 {
    let octave = pitch / 12;
    let pitch_class = pitch % 12;
    let relative = (12 + i16::from(pitch_class) - i16::from(root % 12)) % 12;
    let nearest = steps
        .iter()
        .copied()
        .min_by_key(|step| ((i16::from(*step) - relative).abs(), *step))
        .unwrap_or(0);
    octave
        .saturating_mul(12)
        .saturating_add((root % 12 + nearest) % 12)
}

#[cfg(test)]
mod tests {
    use super::{
        ArpOrder, LiveMidiFxEvent, LiveMidiFxState, MidiFx, MidiFxSlot, QuantizeTarget,
        arp_rate_label, cycle_existing_fx_kind, cycle_fx_kind, delay_rate_label,
        duration_rate_label, playback_timing_lookback_ticks, process_live_chain_event,
        process_live_chain_tick, process_live_event, transform_notes,
    };
    use crate::project::MidiNote;

    #[test]
    fn cycle_kind_creates_first_effect() {
        let slot = cycle_fx_kind(None, 1).expect("slot");
        assert!(matches!(slot.effect, MidiFx::Arp { .. }));
    }

    #[test]
    fn cycle_existing_kind_skips_none() {
        let slot = MidiFxSlot {
            enabled: false,
            effect: MidiFx::TrackClone { source_track: 0 },
        };
        let next = cycle_existing_fx_kind(&slot, 1);
        assert!(matches!(next.effect, MidiFx::Arp { .. }));
        assert!(!next.enabled);
    }

    #[test]
    fn transpose_live_event_remaps_note_off() {
        let mut state = LiveMidiFxState::default();
        let chain = [Some(MidiFxSlot {
            enabled: true,
            effect: MidiFx::Transpose { semitones: 12 },
        })];

        let on = process_live_event(
            &chain,
            &mut state,
            LiveMidiFxEvent::NoteOn {
                pitch: 60,
                velocity: 100,
            },
            0,
        );
        let off = process_live_event(
            &chain,
            &mut state,
            LiveMidiFxEvent::NoteOff { pitch: 60 },
            0,
        );

        assert_eq!(
            on,
            vec![LiveMidiFxEvent::NoteOn {
                pitch: 72,
                velocity: 100
            }]
        );
        assert_eq!(off, vec![LiveMidiFxEvent::NoteOff { pitch: 72 }]);
    }

    #[test]
    fn transform_notes_applies_duration_and_delay() {
        let notes = [MidiNote::new(60, 100, 120, 100)];
        let chain = [
            Some(MidiFxSlot {
                enabled: true,
                effect: MidiFx::Duration { ticks: 240 },
            }),
            Some(MidiFxSlot {
                enabled: true,
                effect: MidiFx::Delay { ticks: 60 },
            }),
        ];

        let transformed = transform_notes(&notes, &chain, 0);
        assert_eq!(transformed[0].start_ticks, 160);
        assert_eq!(transformed[0].length_ticks, 240);
    }

    #[test]
    fn arp_labels_use_musical_notation() {
        assert_eq!(arp_rate_label(240), "1/16");
        assert_eq!(
            MidiFx::Arp {
                step_ticks: 480,
                order: ArpOrder::Down,
                gate_percent: 100,
            }
            .inline_parameters()[0]
                .value,
            "1/8"
        );
        assert_eq!(delay_rate_label(0), "Off");
        assert_eq!(delay_rate_label(240), "1/16");
        assert_eq!(duration_rate_label(0), "Off");
        assert_eq!(duration_rate_label(240), "1/16");
    }

    #[test]
    fn delay_parameter_never_wraps_negative() {
        let mut effect = MidiFx::Delay { ticks: 0 };
        effect.adjust_value(-1, 0, 960);
        assert_eq!(effect.value_label(), "Off");
        effect.adjust_value(1, 0, 960);
        assert_eq!(effect.value_label(), "1/64");
    }

    #[test]
    fn transform_notes_arp_repeats_held_group_over_duration() {
        let notes = [
            MidiNote::new(60, 0, 960, 100),
            MidiNote::new(64, 0, 960, 100),
        ];
        let chain = [Some(MidiFxSlot {
            enabled: true,
            effect: MidiFx::Arp {
                step_ticks: 240,
                order: ArpOrder::Up,
                gate_percent: 100,
            },
        })];
        let transformed = transform_notes(&notes, &chain, 0);
        assert!(
            transformed
                .iter()
                .any(|note| note.start_ticks == 0 && note.pitch == 60)
        );
        assert!(
            transformed
                .iter()
                .any(|note| note.start_ticks == 240 && note.pitch == 64)
        );
        assert!(
            transformed
                .iter()
                .any(|note| note.start_ticks == 480 && note.pitch == 60)
        );
        assert!(
            transformed
                .iter()
                .any(|note| note.start_ticks == 720 && note.pitch == 64)
        );
    }

    #[test]
    fn live_chain_arp_emits_steps_for_held_notes() {
        let chain = [Some(MidiFxSlot {
            enabled: true,
            effect: MidiFx::Arp {
                step_ticks: 240,
                order: ArpOrder::Up,
                gate_percent: 100,
            },
        })];
        let mut state = LiveMidiFxState::default();

        let immediate = process_live_chain_event(
            &chain,
            &mut state,
            LiveMidiFxEvent::NoteOn {
                pitch: 60,
                velocity: 100,
            },
            0,
            0,
        );
        assert!(immediate.is_empty());
        let immediate = process_live_chain_event(
            &chain,
            &mut state,
            LiveMidiFxEvent::NoteOn {
                pitch: 64,
                velocity: 100,
            },
            0,
            0,
        );
        assert!(immediate.is_empty());

        let scheduled = process_live_chain_tick(&chain, &mut state, 0, 500, 0);
        assert!(scheduled.contains(&(
            0,
            LiveMidiFxEvent::NoteOn {
                pitch: 60,
                velocity: 100
            }
        )));
        assert!(scheduled.contains(&(240, LiveMidiFxEvent::NoteOff { pitch: 60 })));
        assert!(scheduled.contains(&(
            240,
            LiveMidiFxEvent::NoteOn {
                pitch: 64,
                velocity: 100
            }
        )));
    }

    #[test]
    fn note_filter_list_parameter_decrements_monotonically() {
        let mut effect = MidiFx::NoteFilter {
            low: 60,
            high: 64,
            enabled_notes: Vec::new(),
        };

        effect.adjust_inline_parameter(2, -1, 0, 960);
        assert_eq!(effect.inline_parameters()[2].value, "4");

        effect.adjust_inline_parameter(2, -1, 0, 960);
        assert_eq!(effect.inline_parameters()[2].value, "3");

        effect.adjust_inline_parameter(2, -1, 0, 960);
        assert_eq!(effect.inline_parameters()[2].value, "2");
    }

    #[test]
    fn live_chain_delay_schedules_note_later() {
        let chain = [Some(MidiFxSlot {
            enabled: true,
            effect: MidiFx::Delay { ticks: 240 },
        })];
        let mut state = LiveMidiFxState::default();

        let immediate = process_live_chain_event(
            &chain,
            &mut state,
            LiveMidiFxEvent::NoteOn {
                pitch: 60,
                velocity: 100,
            },
            0,
            0,
        );
        assert!(immediate.is_empty());

        let scheduled = process_live_chain_tick(&chain, &mut state, 0, 300, 0);
        assert_eq!(
            scheduled,
            vec![(
                240,
                LiveMidiFxEvent::NoteOn {
                    pitch: 60,
                    velocity: 100
                }
            )]
        );
    }

    #[test]
    fn live_chain_delay_emits_note_on_at_exact_boundary_before_later_note_off() {
        let chain = [Some(MidiFxSlot {
            enabled: true,
            effect: MidiFx::Delay { ticks: 240 },
        })];
        let mut state = LiveMidiFxState::default();

        assert!(
            process_live_chain_event(
                &chain,
                &mut state,
                LiveMidiFxEvent::NoteOn {
                    pitch: 60,
                    velocity: 100,
                },
                0,
                0,
            )
            .is_empty()
        );
        assert!(
            process_live_chain_event(
                &chain,
                &mut state,
                LiveMidiFxEvent::NoteOff { pitch: 60 },
                120,
                0,
            )
            .is_empty()
        );

        assert_eq!(
            process_live_chain_tick(&chain, &mut state, 0, 240, 0),
            vec![(
                240,
                LiveMidiFxEvent::NoteOn {
                    pitch: 60,
                    velocity: 100
                }
            )]
        );
        assert_eq!(
            process_live_chain_tick(&chain, &mut state, 240, 360, 0),
            vec![(360, LiveMidiFxEvent::NoteOff { pitch: 60 })]
        );
    }

    #[test]
    fn live_chain_duration_schedules_absolute_note_off() {
        let chain = [Some(MidiFxSlot {
            enabled: true,
            effect: MidiFx::Duration { ticks: 240 },
        })];
        let mut state = LiveMidiFxState::default();

        let immediate = process_live_chain_event(
            &chain,
            &mut state,
            LiveMidiFxEvent::NoteOn {
                pitch: 60,
                velocity: 100,
            },
            0,
            0,
        );
        assert_eq!(
            immediate,
            vec![LiveMidiFxEvent::NoteOn {
                pitch: 60,
                velocity: 100
            }]
        );

        let suppressed = process_live_chain_event(
            &chain,
            &mut state,
            LiveMidiFxEvent::NoteOff { pitch: 60 },
            120,
            0,
        );
        assert!(suppressed.is_empty());

        let scheduled = process_live_chain_tick(&chain, &mut state, 0, 241, 0);
        assert_eq!(
            scheduled,
            vec![(240, LiveMidiFxEvent::NoteOff { pitch: 60 })]
        );
    }

    #[test]
    fn live_chain_duration_emits_note_off_at_exact_boundary() {
        let chain = [Some(MidiFxSlot {
            enabled: true,
            effect: MidiFx::Duration { ticks: 240 },
        })];
        let mut state = LiveMidiFxState::default();

        assert_eq!(
            process_live_chain_event(
                &chain,
                &mut state,
                LiveMidiFxEvent::NoteOn {
                    pitch: 60,
                    velocity: 100,
                },
                0,
                0,
            ),
            vec![LiveMidiFxEvent::NoteOn {
                pitch: 60,
                velocity: 100
            }]
        );

        assert_eq!(
            process_live_chain_tick(&chain, &mut state, 0, 240, 0),
            vec![(240, LiveMidiFxEvent::NoteOff { pitch: 60 })]
        );
    }

    #[test]
    fn playback_timing_lookback_covers_delay_and_absolute_duration() {
        let chain = [
            Some(MidiFxSlot {
                enabled: true,
                effect: MidiFx::Delay { ticks: 60 },
            }),
            Some(MidiFxSlot {
                enabled: true,
                effect: MidiFx::Duration { ticks: 240 },
            }),
        ];

        assert_eq!(playback_timing_lookback_ticks(&chain), 300);
    }

    #[test]
    fn quantize_target_global_uses_global_root_in_note_transform() {
        let notes = [MidiNote::new(60, 0, 120, 100)];
        let chain = [Some(MidiFxSlot {
            enabled: true,
            effect: MidiFx::ScaleQuantize {
                root: 0,
                target: QuantizeTarget::Global,
            },
        })];
        let transformed = transform_notes(&notes, &chain, 2);
        assert_eq!(transformed[0].pitch, 71);
    }
}
