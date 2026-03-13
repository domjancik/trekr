use crate::actions::{ActionSource, AppAction};
use crate::mapping::MappingSourceKind;
use crate::pages::AppPage;
use crate::timeline_fx::TimelineContext;
use serde::{Deserialize, Serialize};
use sdl3::rect::Rect;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AppOverlay {
    MappingsQuickView,
    Discoverability,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) struct OverlayState {
    pub active: Option<AppOverlay>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct StatusState {
    pub hovered_target: Option<DiscoverabilityTarget>,
    pub last_action: Option<LastActionStatus>,
    pub history_message: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TimelineFxRowRef {
    pub context: TimelineContext,
    pub row_index: usize,
    pub slot_index: Option<usize>,
    pub layout: TimelineFxRowLayout,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TimelineFxRowLayout {
    pub row: Rect,
    pub enabled: Rect,
    pub kind: Rect,
    pub param_primary: Rect,
    pub param_secondary: Rect,
    pub overflow: Rect,
    pub move_up: Rect,
    pub move_down: Rect,
    pub delete: Rect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TimelineTrackLayout {
    pub track_index: usize,
    pub full_bounds: Rect,
    pub detail_bounds: Rect,
    pub pair_bounds: Rect,
    pub status_rect: Rect,
    pub body_full_bounds: Rect,
    pub body_detail_bounds: Rect,
    pub full_label_rect: Rect,
    pub detail_label_rect: Rect,
    pub full_content_rect: Rect,
    pub detail_content_rect: Rect,
    pub input_fx_rect: Rect,
    pub output_fx_rect: Rect,
}

impl TimelineTrackLayout {
    pub fn fx_rect(self, context: TimelineContext) -> Rect {
        match context {
            TimelineContext::InputFx => self.input_fx_rect,
            TimelineContext::OutputFx => self.output_fx_rect,
            TimelineContext::TrackTimeline => {
                crate::ui::union_rect(self.body_full_bounds, self.body_detail_bounds)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct DirectMappingState {
    pub mode: DirectMappingMode,
    pub origin: DirectMappingOrigin,
    pub status_message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct MappingTargetLookupState {
    pub active: Option<ActiveMappingTargetLookup>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ActiveMappingTargetLookup {
    pub original_target_label: String,
    pub original_scope_label: String,
    pub query: String,
    pub highlighted_index: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct MappingTargetLookupLayout {
    pub target_cell: Rect,
    pub results_panel: Rect,
    pub start_index: usize,
    pub visible_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum DirectMappingMode {
    #[default]
    Inactive,
    Targeting,
    AwaitingInput(DirectMappingTarget),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum DirectMappingOrigin {
    #[default]
    InPlace,
    MappingsPage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DirectMappingTarget {
    pub action: AppAction,
    pub target_label: &'static str,
    pub scope_label: &'static str,
    pub display_scope: Option<&'static str>,
    pub hit_rect: Rect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LastActionStatus {
    pub action: AppAction,
    pub source: ActionSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DiscoverabilityTarget {
    pub action: AppAction,
    pub display_scope: Option<&'static str>,
    pub allowed_mapping_scopes: &'static [&'static str],
    pub overlay_slot: Option<Rect>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ActionDiscoverabilitySummary {
    pub title: String,
    pub badges: Vec<MappingBadge>,
    pub total_bindings: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MappingBadge {
    pub text: String,
    pub source_kind: MappingSourceKind,
    pub built_in: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RecordingLaneLayout {
    pub clip_id: Option<u64>,
    pub rect: Rect,
    pub selected: bool,
    pub muted: bool,
    pub preview: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RecordingLaneWindow {
    pub start: usize,
    pub visible_total: usize,
    pub committed_start: usize,
    pub committed_end: usize,
    pub visible_committed: usize,
    pub show_preview: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiCaptureOptions {
    pub output_dir: PathBuf,
    pub state_mode: String,
    pub script_path: Option<PathBuf>,
    pub capture_region: Option<String>,
    pub capture_rect: Option<CaptureRect>,
    pub capture_padding: Option<CapturePadding>,
    pub annotation_path: Option<PathBuf>,
    pub sequence_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VideoMode {
    #[default]
    Windowed,
    Fullscreen,
    KmsDrmConsole,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UiScalingMode {
    #[default]
    Auto,
    Nearest,
    Linear,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RunOptions {
    pub video_mode: VideoMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CaptureRect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapturePadding {
    pub left: u32,
    pub top: u32,
    pub right: u32,
    pub bottom: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CaptureSpec {
    pub page: AppPage,
    pub overlay: Option<AppOverlay>,
    pub focused_track_view: bool,
    pub filename: &'static str,
}
