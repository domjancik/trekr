use crate::actions::{ActionSource, AppAction, action_label};
use crate::app::{
    App, AppOverlay, CapturePadding, CaptureRect, DirectMappingMode, UiCaptureOptions,
};
use crate::pages::{AppPage, MappingField, MappingPageMode, MidiIoListFocus, RoutingField};
use crate::project::{MidiNote, RecordingView, STORED_LOOP_SLOT_COUNT, Track};
use crate::routing::MidiChannelFilter;
use image::RgbaImage;
use serde::{Deserialize, Serialize};
use sdl3::pixels::{Color, PixelFormat};
use sdl3::rect::Rect;
use sdl3::render::{Canvas, RenderTarget};
use sdl3::surface::SurfaceRef;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use super::TransportChipSpec;
use super::shell::layout::page_tabs_layout;
use super::shell::ui::transport_strip_height;
use super::types::CaptureSpec;

pub(super) fn chip_row_width(specs: &[TransportChipSpec]) -> u32 {
    if specs.is_empty() {
        return 0;
    }
    let chips = specs
        .iter()
        .map(|spec| crate::ui::text_width(&spec.label, 1) + 10)
        .sum::<u32>();
    let gaps = (specs.len().saturating_sub(1) as u32) * 6;
    chips.saturating_add(gaps)
}

pub(super) fn capture_specs() -> [CaptureSpec; 6] {
    [
        CaptureSpec {
            page: AppPage::Timeline,
            overlay: None,
            focused_track_view: false,
            filename: "timeline.png",
        },
        CaptureSpec {
            page: AppPage::Timeline,
            overlay: None,
            focused_track_view: true,
            filename: "timeline-focused.png",
        },
        CaptureSpec {
            page: AppPage::Mappings,
            overlay: None,
            focused_track_view: false,
            filename: "mappings.png",
        },
        CaptureSpec {
            page: AppPage::Mappings,
            overlay: Some(AppOverlay::MappingsQuickView),
            focused_track_view: false,
            filename: "mappings-overlay.png",
        },
        CaptureSpec {
            page: AppPage::MidiIo,
            overlay: None,
            focused_track_view: false,
            filename: "midi-io.png",
        },
        CaptureSpec {
            page: AppPage::Routing,
            overlay: None,
            focused_track_view: false,
            filename: "routing.png",
        },
    ]
}

#[derive(Debug, Clone)]
pub(super) struct RgbaReadback {
    logical_rect: Rect,
    output_rect: Rect,
    scale_x: f32,
    scale_y: f32,
    pitch: usize,
    pixels: Vec<u8>,
}

pub(super) fn readback_rect_rgba<T: RenderTarget>(
    canvas: &Canvas<T>,
    logical_rect: Rect,
    logical_viewport_size: (u32, u32),
) -> Option<RgbaReadback> {
    if logical_rect.width() == 0 || logical_rect.height() == 0 {
        return None;
    }
    let output_size = canvas.output_size().ok()?;
    let scale_x = if logical_viewport_size.0 > 0 {
        output_size.0 as f32 / logical_viewport_size.0 as f32
    } else {
        1.0
    };
    let scale_y = if logical_viewport_size.1 > 0 {
        output_size.1 as f32 / logical_viewport_size.1 as f32
    } else {
        1.0
    };
    let sx = scale_x.max(0.0001);
    let sy = scale_y.max(0.0001);
    let ox = (logical_rect.x as f32 * sx).floor() as i32;
    let oy = (logical_rect.y as f32 * sy).floor() as i32;
    let ow = (logical_rect.width() as f32 * sx).ceil().max(1.0) as u32;
    let oh = (logical_rect.height() as f32 * sy).ceil().max(1.0) as u32;
    let output_rect = Rect::new(ox, oy, ow, oh);
    let surface = canvas.read_pixels(output_rect).ok()?;
    let converted = surface.convert_format(PixelFormat::RGBA32).ok()?;
    let pitch = converted.pitch() as usize;
    let pixels = converted.with_lock(|src| src.to_vec());
    Some(RgbaReadback {
        logical_rect,
        output_rect,
        scale_x,
        scale_y,
        pitch,
        pixels,
    })
}

pub(super) fn readback_color_at(readback: &Option<RgbaReadback>, x: i32, y: i32) -> Option<Color> {
    let readback = readback.as_ref()?;
    if x < readback.logical_rect.x
        || y < readback.logical_rect.y
        || x >= readback.logical_rect.x + readback.logical_rect.width() as i32
        || y >= readback.logical_rect.y + readback.logical_rect.height() as i32
    {
        return None;
    }
    let local_logical_x = x - readback.logical_rect.x;
    let local_logical_y = y - readback.logical_rect.y;
    let local_x = (local_logical_x as f32 * readback.scale_x).floor() as usize;
    let local_y = (local_logical_y as f32 * readback.scale_y).floor() as usize;
    if local_x >= readback.output_rect.width() as usize
        || local_y >= readback.output_rect.height() as usize
    {
        return None;
    }
    let base = local_y
        .saturating_mul(readback.pitch)
        .saturating_add(local_x.saturating_mul(4));
    if base + 3 >= readback.pixels.len() {
        return None;
    }
    Some(Color::RGBA(
        readback.pixels[base],
        readback.pixels[base + 1],
        readback.pixels[base + 2],
        readback.pixels[base + 3],
    ))
}

pub(super) fn seed_capture_demo_track(track: &mut Track, track_index: usize) {
    let overlaps = [
        crate::timeline::LoopRegion::new(0, 3_840),
        crate::timeline::LoopRegion::new(480, 3_360),
        crate::timeline::LoopRegion::new(960, 2_880),
        crate::timeline::LoopRegion::new(1_440, 2_400),
        crate::timeline::LoopRegion::new(1_920, 1_920),
        crate::timeline::LoopRegion::new(2_400, 1_440),
        crate::timeline::LoopRegion::new(2_880, 960),
        crate::timeline::LoopRegion::new(3_360, 960),
    ];

    track.midi_notes = dense_capture_notes(track_index);
    track.loop_region = overlaps[0];
    track.state.loop_enabled = true;

    track.stored_loops = vec![None; STORED_LOOP_SLOT_COUNT];
    track.active_stored_loop_slot = None;
    for (slot_index, range) in overlaps.iter().copied().enumerate() {
        track.loop_region = range;
        track.store_current_loop_to_slot(slot_index);
    }
    track.recall_stored_loop_slot(2);
}

fn dense_capture_notes(track_index: usize) -> Vec<MidiNote> {
    let mut notes = Vec::with_capacity(80);
    let base_pitch = 42_u8.saturating_add((track_index as u8).saturating_mul(2));
    for step in 0..40_u64 {
        let start = step * 120;
        let primary_pitch = base_pitch.saturating_add((step % 12) as u8);
        let secondary_pitch = primary_pitch.saturating_add(7);
        let velocity = 72_u8.saturating_add(((step * 9) % 44) as u8);
        notes.push(MidiNote::new(primary_pitch, start, 180, velocity));
        if step % 2 == 0 {
            notes.push(MidiNote::new(
                secondary_pitch,
                start + 60,
                120,
                velocity.saturating_sub(10),
            ));
        }
    }
    notes
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CaptureScript {
    #[serde(default)]
    steps: Vec<CaptureScriptStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum CaptureScriptStep {
    ShowPage {
        page: String,
    },
    SendAction {
        action: String,
    },
    Click {
        x: Option<i32>,
        y: Option<i32>,
        named_target: Option<String>,
    },
    WaitFrames {
        frames: u32,
    },
    SetStateOverride {
        state_file: Option<String>,
        selected_track_index: Option<usize>,
        transport_ticks: Option<u64>,
        playhead_ticks: Option<u64>,
        focused_track_view: Option<bool>,
        overlay: Option<String>,
        mapping_mode: Option<String>,
        mapping_learn_armed: Option<bool>,
        selected_mapping_index: Option<usize>,
        selected_mapping_field: Option<String>,
        selected_routing_field: Option<String>,
        midi_focus: Option<String>,
        recording_view: Option<String>,
        selected_recording_clip_id: Option<u64>,
        selected_recording_clip_index: Option<usize>,
        active_stored_loop_slot: Option<usize>,
        queued_stored_loop_slot: Option<usize>,
        #[serde(default)]
        track_states: Vec<CaptureTrackStateOverride>,
        #[serde(default)]
        routing: Vec<CaptureTrackRoutingOverride>,
        direct_mapping_mode: Option<String>,
        direct_mapping_target_action: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CaptureTrackStateOverride {
    track_index: usize,
    #[serde(default)]
    armed: Option<bool>,
    #[serde(default)]
    muted: Option<bool>,
    #[serde(default)]
    soloed: Option<bool>,
    #[serde(default)]
    passthrough: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CaptureTrackRoutingOverride {
    track_index: usize,
    #[serde(default)]
    input_channel: Option<u8>,
    #[serde(default)]
    output_channel: Option<u8>,
    #[serde(default)]
    passthrough: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CaptureAnnotations {
    #[serde(default)]
    overlays: Vec<CaptureOverlay>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum CaptureOverlay {
    Box {
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        #[serde(default)]
        color: Option<[u8; 4]>,
        #[serde(default)]
        label: Option<String>,
    },
    Arrow {
        from: [u32; 2],
        to: [u32; 2],
        #[serde(default)]
        color: Option<[u8; 4]>,
        #[serde(default)]
        label: Option<String>,
    },
    RegionTint {
        region_id: String,
        #[serde(default)]
        color: Option<[u8; 4]>,
        #[serde(default)]
        label: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize)]
struct CaptureManifest {
    generated_at: String,
    output_dir: String,
    state_mode: String,
    capture_region: Option<String>,
    capture_region_name: Option<String>,
    capture_rect: Option<CaptureRect>,
    capture_padding: Option<CapturePadding>,
    script: Option<String>,
    sequence_script: Option<String>,
    annotations: Option<String>,
    app_commit_hash: Option<String>,
    command_args: Vec<String>,
    files: Vec<CaptureManifestFile>,
}

#[derive(Debug, Clone, Serialize)]
struct CaptureManifestFile {
    filename: String,
    path: String,
    width: u32,
    height: u32,
    page: String,
    overlay: Option<String>,
    focused_track_view: bool,
    sequence_index: Option<usize>,
    sequence_label: Option<String>,
    state_hash: String,
    command_args: Vec<String>,
    targets: Vec<CaptureManifestTarget>,
}

#[derive(Debug, Clone, Serialize)]
struct CaptureManifestTarget {
    id: String,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

#[derive(Debug, Clone)]
struct CaptureOutputSpec {
    filename: String,
    page: AppPage,
    overlay: Option<AppOverlay>,
    focused_track_view: bool,
    sequence_index: Option<usize>,
    sequence_label: Option<String>,
}

#[derive(Debug, Clone)]
struct AutomationTarget {
    id: String,
    page: AppPage,
    rect: Rect,
}

impl App {
    pub fn capture_ui_pages(
        &mut self,
        options: UiCaptureOptions,
    ) -> Result<(), Box<dyn std::error::Error>> {
        fs::create_dir_all(&options.output_dir)?;

        let _sdl_context = sdl3::init()?;
        self.viewport_size = (1280, 720);
        self.startup_started_at = std::time::Instant::now() - Duration::from_secs(10);
        let script = options
            .script_path
            .as_ref()
            .map(|path| read_json_file::<CaptureScript>(path))
            .transpose()?;
        let sequence_script = options
            .sequence_path
            .as_ref()
            .map(|path| read_json_file::<CaptureScript>(path))
            .transpose()?;
        let annotations = options
            .annotation_path
            .as_ref()
            .map(|path| read_json_file::<CaptureAnnotations>(path))
            .transpose()?;

        if let Some(script) = &script {
            self.apply_capture_script(script)?;
        }

        let outputs = if let Some(sequence) = &sequence_script {
            self.capture_sequence_outputs(sequence)?
        } else {
            capture_specs()
                .into_iter()
                .map(|spec| CaptureOutputSpec {
                    filename: spec.filename.to_owned(),
                    page: spec.page,
                    overlay: spec.overlay,
                    focused_track_view: spec.focused_track_view,
                    sequence_index: None,
                    sequence_label: None,
                })
                .collect::<Vec<_>>()
        };
        let mut outputs = outputs;
        if let Some(region_id) = options.capture_region.as_deref() {
            outputs.retain(|output| self.capture_region_rect(output.page, region_id).is_some());
            if outputs.is_empty() {
                return Err(format!(
                    "capture region `{region_id}` is not available for the selected capture outputs"
                )
                .into());
            }
        }

        let mut files = Vec::new();
        for output in outputs {
            self.page_state.current_page = output.page;
            self.overlay_state.active = output.overlay;
            self.focused_track_view = output.focused_track_view;

            let surface = sdl3::surface::Surface::new(1280, 720, PixelFormat::RGBA32)?;
            let mut canvas = surface.into_canvas()?;
            canvas.set_scale(1.0, 1.0)?;
            self.draw(&mut canvas)?;

            let mut image = surface_ref_to_rgba_image(canvas.surface())?;
            if let Some(annotations) = &annotations {
                self.apply_capture_annotations(&mut image, annotations, output.page)?;
            }
            let crop_rect = self.resolve_capture_crop_rect(
                &options,
                output.page,
                image.width(),
                image.height(),
            )?;
            if let Some(rect) = crop_rect {
                image = crop_rgba_image(&image, rect)?;
            }

            let output_path = options.output_dir.join(&output.filename);
            image.save(&output_path)?;
            let targets = self
                .automation_targets(
                    self.capture_content_bounds().map_err(|err| {
                        format!("failed to compute capture content bounds: {err}")
                    })?,
                )
                .into_iter()
                .filter(|target| {
                    (target.page == output.page || target.id.starts_with("tab."))
                        && target.rect.width() > 0
                        && target.rect.height() > 0
                })
                .map(|target| CaptureManifestTarget {
                    id: target.id,
                    x: target.rect.x.max(0) as u32,
                    y: target.rect.y.max(0) as u32,
                    width: target.rect.width(),
                    height: target.rect.height(),
                })
                .collect::<Vec<_>>();

            files.push(CaptureManifestFile {
                filename: output.filename.clone(),
                path: output_path.display().to_string(),
                width: image.width(),
                height: image.height(),
                page: output.page.label().to_owned(),
                overlay: output
                    .overlay
                    .map(capture_overlay_label)
                    .map(ToOwned::to_owned),
                focused_track_view: output.focused_track_view,
                sequence_index: output.sequence_index,
                sequence_label: output.sequence_label.clone(),
                state_hash: self.capture_state_hash(),
                command_args: std::env::args().collect(),
                targets,
            });
        }

        self.overlay_state.active = None;

        let manifest = CaptureManifest {
            generated_at: now_iso8601_string(),
            output_dir: options.output_dir.display().to_string(),
            state_mode: options.state_mode,
            capture_region_name: options
                .capture_region
                .as_deref()
                .and_then(capture_region_name)
                .map(ToOwned::to_owned),
            capture_region: options.capture_region,
            capture_rect: options.capture_rect,
            capture_padding: options.capture_padding,
            script: options.script_path.map(|path| path.display().to_string()),
            sequence_script: options.sequence_path.map(|path| path.display().to_string()),
            annotations: options.annotation_path.map(|path| path.display().to_string()),
            app_commit_hash: git_commit_hash(),
            command_args: std::env::args().collect(),
            files,
        };
        let manifest_path = options.output_dir.join("manifest.json");
        fs::write(&manifest_path, serde_json::to_vec_pretty(&manifest)?)?;

        Ok(())
    }

    fn apply_capture_script(
        &mut self,
        script: &CaptureScript,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for step in &script.steps {
            match step {
                CaptureScriptStep::ShowPage { page } => {
                    self.page_state.current_page = parse_app_page(page)?;
                }
                CaptureScriptStep::SendAction { action } => {
                    let action = parse_capture_action(action)?;
                    self.apply_action_with_source(action, ActionSource::Internal);
                }
                CaptureScriptStep::Click { x, y, named_target } => {
                    let (click_x, click_y) = match (x, y, named_target) {
                        (Some(x), Some(y), _) => (*x, *y),
                        (_, _, Some(target_id)) => self
                            .resolve_automation_target_center(target_id)
                            .ok_or_else(|| {
                                format!("unknown capture click target id: {target_id}")
                            })?,
                        _ => {
                            return Err(
                                "click step requires x/y or named_target in capture script".into(),
                            );
                        }
                    };
                    let _ = self.handle_pointer_down(click_x, click_y, ActionSource::Pointer);
                }
                CaptureScriptStep::WaitFrames { frames } => {
                    for _ in 0..*frames {
                        self.advance_playhead(Duration::from_millis(16));
                    }
                }
                CaptureScriptStep::SetStateOverride {
                    state_file,
                    selected_track_index,
                    transport_ticks,
                    playhead_ticks,
                    focused_track_view,
                    overlay,
                    mapping_mode,
                    mapping_learn_armed,
                    selected_mapping_index,
                    selected_mapping_field,
                    selected_routing_field,
                    midi_focus,
                    recording_view,
                    selected_recording_clip_id,
                    selected_recording_clip_index,
                    active_stored_loop_slot,
                    queued_stored_loop_slot,
                    track_states,
                    routing,
                    direct_mapping_mode,
                    direct_mapping_target_action,
                } => {
                    if let Some(state_file) = state_file {
                        self.load_capture_state_file(state_file)?;
                    }
                    if let Some(index) = selected_track_index {
                        if !self.project.tracks.is_empty() {
                            self.project.active_track_index =
                                (*index).min(self.project.tracks.len().saturating_sub(1));
                        }
                    }
                    if let Some(ticks) = transport_ticks {
                        self.transport_ticks = *ticks;
                    }
                    if let Some(ticks) = playhead_ticks {
                        self.playhead_ticks = *ticks;
                    }
                    if let Some(value) = focused_track_view {
                        self.focused_track_view = *value;
                    }
                    if let Some(overlay) = overlay {
                        self.overlay_state.active = parse_capture_overlay(overlay)?;
                    }
                    if let Some(mapping_mode) = mapping_mode {
                        self.page_state.mapping_mode = parse_mapping_mode(mapping_mode)?;
                    }
                    if let Some(armed) = mapping_learn_armed {
                        self.page_state.mapping_midi_learn_armed = *armed;
                    }
                    if let Some(index) = selected_mapping_index {
                        self.page_state.selected_mapping_index =
                            (*index).min(self.mappings.len().saturating_sub(1));
                    }
                    if let Some(field) = selected_mapping_field {
                        self.page_state.selected_mapping_field = parse_mapping_field(field)?;
                    }
                    if let Some(field) = selected_routing_field {
                        self.page_state.selected_routing_field = parse_routing_field(field)?;
                    }
                    if let Some(focus) = midi_focus {
                        self.page_state.midi_io.focus = parse_midi_focus(focus)?;
                    }
                    if let Some(view) = recording_view {
                        self.project.active_track_mut().unwrap().recording_view =
                            parse_recording_view(view)?;
                    }
                    if let Some(clip_id) = selected_recording_clip_id {
                        self.project
                            .active_track_mut()
                            .unwrap()
                            .selected_recording_clip_id = Some(*clip_id);
                    }
                    if let Some(clip_index) = selected_recording_clip_index {
                        if let Some(clip_id) = self
                            .project
                            .active_track()
                            .and_then(|track| track.recording_clips().get(*clip_index))
                            .map(|clip| clip.id)
                        {
                            self.project
                                .active_track_mut()
                                .unwrap()
                                .selected_recording_clip_id = Some(clip_id);
                        }
                    }
                    if let Some(slot) = active_stored_loop_slot {
                        self.set_active_stored_loop_slot(*slot);
                    }
                    if let Some(slot) = queued_stored_loop_slot {
                        self.queue_stored_loop_slot(*slot);
                    }
                    self.apply_capture_track_state_overrides(track_states);
                    self.apply_capture_routing_overrides(routing);
                    if let Some(mode) = direct_mapping_mode {
                        self.set_direct_mapping_mode(mode, direct_mapping_target_action.as_deref())?;
                    } else if let Some(action) = direct_mapping_target_action {
                        self.set_direct_mapping_mode("awaiting_input", Some(action))?;
                    }
                }
            }
        }
        Ok(())
    }

    fn capture_sequence_outputs(
        &mut self,
        script: &CaptureScript,
    ) -> Result<Vec<CaptureOutputSpec>, Box<dyn std::error::Error>> {
        let mut outputs = Vec::new();
        for (index, step) in script.steps.iter().enumerate() {
            match step {
                CaptureScriptStep::ShowPage { page } => {
                    self.page_state.current_page = parse_app_page(page)?;
                }
                CaptureScriptStep::SendAction { action } => {
                    let action = parse_capture_action(action)?;
                    self.apply_action_with_source(action, ActionSource::Internal);
                }
                CaptureScriptStep::Click { x, y, named_target } => {
                    let (click_x, click_y) = match (x, y, named_target) {
                        (Some(x), Some(y), _) => (*x, *y),
                        (_, _, Some(target_id)) => self
                            .resolve_automation_target_center(target_id)
                            .ok_or_else(|| {
                                format!("unknown capture click target id: {target_id}")
                            })?,
                        _ => {
                            return Err(
                                "click step requires x/y or named_target in sequence script".into(),
                            );
                        }
                    };
                    let _ = self.handle_pointer_down(click_x, click_y, ActionSource::Pointer);
                }
                CaptureScriptStep::WaitFrames { frames } => {
                    for _ in 0..*frames {
                        self.advance_playhead(Duration::from_millis(16));
                    }
                }
                CaptureScriptStep::SetStateOverride { .. } => {
                    self.apply_capture_script(&CaptureScript {
                        steps: vec![step.clone()],
                    })?;
                }
            }

            outputs.push(CaptureOutputSpec {
                filename: format!("sequence-{index:03}.png"),
                page: self.page_state.current_page,
                overlay: self.overlay_state.active,
                focused_track_view: self.focused_track_view,
                sequence_index: Some(index),
                sequence_label: Some(capture_script_step_label(step)),
            });
        }

        Ok(outputs)
    }

    fn load_capture_state_file(
        &mut self,
        state_file: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let state_path = PathBuf::from(state_file);
        let state = crate::state::load(&state_path)?;
        let mut app = App::from_persisted_state(state);
        app.viewport_size = self.viewport_size;
        app.ui_scale_override = self.ui_scale_override;
        app.startup_started_at = self.startup_started_at;
        *self = app;
        Ok(())
    }

    fn apply_capture_track_state_overrides(&mut self, overrides: &[CaptureTrackStateOverride]) {
        for override_entry in overrides {
            if let Some(track) = self.project.tracks.get_mut(override_entry.track_index) {
                if let Some(value) = override_entry.armed {
                    track.state.armed = value;
                }
                if let Some(value) = override_entry.muted {
                    track.state.muted = value;
                }
                if let Some(value) = override_entry.soloed {
                    track.state.soloed = value;
                }
                if let Some(value) = override_entry.passthrough {
                    track.state.passthrough = value;
                }
            }
        }
    }

    fn apply_capture_routing_overrides(&mut self, overrides: &[CaptureTrackRoutingOverride]) {
        for override_entry in overrides {
            if let Some(track) = self.project.tracks.get_mut(override_entry.track_index) {
                if let Some(value) = override_entry.input_channel {
                    track.routing.input_channel = if value == 0 {
                        MidiChannelFilter::Omni
                    } else {
                        MidiChannelFilter::Channel(value.clamp(1, 16))
                    };
                }
                if let Some(value) = override_entry.output_channel {
                    track.routing.output_channel = if value == 0 {
                        None
                    } else {
                        Some(value.clamp(1, 16))
                    };
                }
                if let Some(value) = override_entry.passthrough {
                    track.state.passthrough = value;
                }
            }
        }
    }

    fn set_active_stored_loop_slot(&mut self, slot: usize) {
        if let Some(track) = self.project.active_track_mut() {
            let index = slot.min(STORED_LOOP_SLOT_COUNT.saturating_sub(1));
            if track.stored_loop_slot(index).is_none() {
                let _ = track.store_current_loop_to_slot(index);
            }
            let _ = track.recall_stored_loop_slot(index);
        }
    }

    fn queue_stored_loop_slot(&mut self, slot: usize) {
        let launch_quantize = self.project.transport.stored_loop_launch_quantize;
        if let Some(track) = self.project.active_track_mut() {
            let index = slot.min(STORED_LOOP_SLOT_COUNT.saturating_sub(1));
            if track.stored_loop_slot(index).is_none() {
                let _ = track.store_current_loop_to_slot(index);
            }
            let _ = track.queue_stored_loop_recall(index, launch_quantize, self.transport_ticks);
        }
    }

    fn set_direct_mapping_mode(
        &mut self,
        mode: &str,
        target_action: Option<&str>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match mode.trim().to_ascii_lowercase().as_str() {
            "inactive" => self.direct_mapping_state.mode = DirectMappingMode::Inactive,
            "targeting" => self.direct_mapping_state.mode = DirectMappingMode::Targeting,
            "awaiting_input" => {
                let action = target_action.ok_or_else(|| {
                    "direct_mapping_target_action is required for awaiting_input".to_owned()
                })?;
                let action = parse_capture_action(action)?;
                let content_bounds = self.capture_content_bounds().map_err(|err| {
                    format!("cannot resolve direct mapping target without content bounds: {err}")
                })?;
                let target = self
                    .direct_mapping_targets(content_bounds)
                    .into_iter()
                    .find(|target| target.action == action)
                    .ok_or_else(|| {
                        format!(
                            "no direct-mapping target found for action `{}` on page {}",
                            action_label(action),
                            self.page_state.current_page.label()
                        )
                    })?;
                self.direct_mapping_state.mode = DirectMappingMode::AwaitingInput(target);
            }
            other => {
                return Err(format!(
                    "unknown direct_mapping_mode: {other} (expected inactive|targeting|awaiting_input)"
                )
                .into());
            }
        }
        Ok(())
    }

    fn resolve_capture_crop_rect(
        &self,
        options: &UiCaptureOptions,
        page: AppPage,
        image_width: u32,
        image_height: u32,
    ) -> Result<Option<CaptureRect>, Box<dyn std::error::Error>> {
        let base_rect = if let Some(rect) = options.capture_rect {
            Some(rect)
        } else if let Some(region_id) = options.capture_region.as_deref() {
            let rect = self.capture_region_rect(page, region_id).ok_or_else(|| {
                format!(
                    "capture region `{region_id}` is not available on page {}",
                    page.label()
                )
            })?;
            Some(rect)
        } else {
            None
        };
        let Some(base_rect) = base_rect else {
            return Ok(None);
        };
        if let Some(padding) = options.capture_padding {
            return Ok(Some(
                apply_capture_padding(base_rect, padding, image_width, image_height).ok_or_else(
                    || "capture padding collapsed crop rect outside source image bounds".to_owned(),
                )?,
            ));
        }
        Ok(Some(base_rect))
    }

    fn capture_region_rect(&self, page: AppPage, region_id: &str) -> Option<CaptureRect> {
        let content_bounds = self.capture_content_bounds().ok()?;
        let normalized = region_id.replace('-', "_");
        match normalized.as_str() {
            "timeline_transport" if page == AppPage::Timeline => {
                let (_, body_bounds) = crate::ui::split_top_strip(content_bounds, 28, 6).ok()?;
                let (transport_bounds, _) =
                    crate::ui::split_top_strip(body_bounds, transport_strip_height(), 8).ok()?;
                capture_rect_from_rect(transport_bounds)
            }
            "timeline_recwrap_quantize_strip" if page == AppPage::Timeline => {
                let (_, body_bounds) = crate::ui::split_top_strip(content_bounds, 28, 6).ok()?;
                let (transport_bounds, _) =
                    crate::ui::split_top_strip(body_bounds, transport_strip_height(), 8).ok()?;
                capture_rect_from_rect(Rect::new(
                    transport_bounds.x + 4,
                    transport_bounds.y + 16,
                    transport_bounds.width().saturating_sub(8),
                    14,
                ))
            }
            "timeline_link_status_strip" if page == AppPage::Timeline => {
                let (_, body_bounds) = crate::ui::split_top_strip(content_bounds, 28, 6).ok()?;
                let (transport_bounds, _) =
                    crate::ui::split_top_strip(body_bounds, transport_strip_height(), 8).ok()?;
                let right_panel_width = self.transport_right_panel_width(transport_bounds);
                capture_rect_from_rect(Rect::new(
                    transport_bounds.x + transport_bounds.width() as i32
                        - right_panel_width as i32
                        - 8,
                    transport_bounds.y + 2,
                    right_panel_width.saturating_add(4),
                    transport_bounds.height().saturating_sub(4),
                ))
            }
            "timeline_track_header_active" if page == AppPage::Timeline => {
                let (_, body_bounds) = crate::ui::split_top_strip(content_bounds, 28, 6).ok()?;
                let (_, timeline_bounds) =
                    crate::ui::split_top_strip(body_bounds, transport_strip_height(), 8).ok()?;
                let (_, full_bounds, _) = self
                    .visible_track_columns(timeline_bounds)
                    .into_iter()
                    .find(|(index, _, _)| *index == self.project.active_track_index)?;
                let header = crate::ui::track_header_rect(full_bounds, self.timeline_flow);
                capture_rect_from_rect(header)
            }
            "transport_left" if page == AppPage::Timeline => {
                let (_, body_bounds) = crate::ui::split_top_strip(content_bounds, 28, 6).ok()?;
                let (transport_bounds, _) =
                    crate::ui::split_top_strip(body_bounds, transport_strip_height(), 8).ok()?;
                let right_panel_width = self.transport_right_panel_width(transport_bounds);
                let right_panel_x =
                    transport_bounds.x + transport_bounds.width() as i32 - right_panel_width as i32 - 6;
                capture_rect_from_rect(Rect::new(
                    transport_bounds.x + 2,
                    transport_bounds.y + 2,
                    right_panel_x.saturating_sub(transport_bounds.x + 6) as u32,
                    transport_bounds.height().saturating_sub(4),
                ))
            }
            "transport_right" if page == AppPage::Timeline => {
                let (_, body_bounds) = crate::ui::split_top_strip(content_bounds, 28, 6).ok()?;
                let (transport_bounds, _) =
                    crate::ui::split_top_strip(body_bounds, transport_strip_height(), 8).ok()?;
                let right_panel_width = self.transport_right_panel_width(transport_bounds);
                capture_rect_from_rect(Rect::new(
                    transport_bounds.x + transport_bounds.width() as i32 - right_panel_width as i32 - 6,
                    transport_bounds.y + 3,
                    right_panel_width,
                    transport_bounds.height().saturating_sub(6),
                ))
            }
            "status_strip" if page == AppPage::Timeline => {
                let (_, body_bounds) = crate::ui::split_top_strip(content_bounds, 28, 6).ok()?;
                let (_, timeline_bounds) =
                    crate::ui::split_top_strip(body_bounds, transport_strip_height(), 8).ok()?;
                let (_, full_bounds, detail_bounds) = self
                    .visible_track_columns(timeline_bounds)
                    .into_iter()
                    .find(|(index, _, _)| *index == self.project.active_track_index)?;
                capture_rect_from_rect(crate::ui::track_status_rect(
                    crate::ui::union_rect(full_bounds, detail_bounds),
                    self.timeline_flow,
                ))
            }
            "timeline_header_controls" if page == AppPage::Timeline => {
                let (header_bounds, _) = crate::ui::split_top_strip(content_bounds, 28, 6).ok()?;
                capture_rect_from_rect(crate::ui::union_rect(
                    self.focused_track_view_button_rect(header_bounds),
                    self.global_loop_reset_button_rect(header_bounds),
                ))
            }
            "mappings_selected_row" if page == AppPage::Mappings => self
                .mappings_selected_row_rect(content_bounds)
                .and_then(capture_rect_from_rect),
            "mappings_bank_panel" if page == AppPage::Mappings => {
                let panel = Rect::new(
                    content_bounds.x + content_bounds.width() as i32 - 320,
                    content_bounds.y + 44,
                    312,
                    content_bounds.height().saturating_sub(52),
                );
                capture_rect_from_rect(panel)
            }
            "timeline_stacked_clip_controls" if page == AppPage::Timeline => {
                let (_, body_bounds) = crate::ui::split_top_strip(content_bounds, 28, 6).ok()?;
                let (_, timeline_bounds) =
                    crate::ui::split_top_strip(body_bounds, transport_strip_height(), 8).ok()?;
                let (_, _, detail_bounds) = self
                    .visible_track_columns(timeline_bounds)
                    .into_iter()
                    .find(|(index, _, _)| *index == self.project.active_track_index)?;
                let detail_label_rect = crate::ui::track_label_rect(detail_bounds, self.timeline_flow);
                let (mute_rect, delete_rect) = self.recording_clip_control_rects(detail_label_rect);
                let union = crate::ui::union_rect(mute_rect, delete_rect);
                capture_rect_from_rect(Rect::new(
                    union.x.saturating_sub(6),
                    union.y.saturating_sub(4),
                    union.width().saturating_add(12),
                    union.height().saturating_add(8),
                ))
            }
            "timeline_direct_mapping_target" if page == AppPage::Timeline => {
                match self.direct_mapping_state.mode {
                    DirectMappingMode::AwaitingInput(target) => capture_rect_from_rect(target.hit_rect),
                    _ => None,
                }
            }
            "mappings_direct_mapping_target" if page == AppPage::Mappings => {
                match self.direct_mapping_state.mode {
                    DirectMappingMode::AwaitingInput(target) => capture_rect_from_rect(target.hit_rect),
                    _ => None,
                }
            }
            "routing_active_row" if page == AppPage::Routing => {
                let inner = crate::ui::inset_rect(content_bounds, 12, 32).ok()?;
                let (_, body) = crate::ui::split_top_strip(inner, 48, 10).ok()?;
                let rows = crate::ui::stacked_rows(body, RoutingField::ALL.len(), 10);
                let index = RoutingField::ALL
                    .iter()
                    .position(|field| *field == self.page_state.selected_routing_field)?;
                capture_rect_from_rect(rows[index])
            }
            "routing_passthrough_block" if page == AppPage::Routing => {
                let inner = crate::ui::inset_rect(content_bounds, 12, 32).ok()?;
                let (header, body) = crate::ui::split_top_strip(inner, 48, 10).ok()?;
                let rows = crate::ui::stacked_rows(body, RoutingField::ALL.len(), 10);
                let passthrough_row = rows[RoutingField::ALL
                    .iter()
                    .position(|field| *field == RoutingField::Passthrough)?];
                capture_rect_from_rect(crate::ui::union_rect(
                    Rect::new(header.x + 106, header.y + 8, 92, header.height().saturating_sub(16)),
                    passthrough_row,
                ))
            }
            "routing_direct_mapping_target" if page == AppPage::Routing => {
                match self.direct_mapping_state.mode {
                    DirectMappingMode::AwaitingInput(target) => capture_rect_from_rect(target.hit_rect),
                    _ => None,
                }
            }
            "fx_row" if page == AppPage::Routing => {
                let inner = crate::ui::inset_rect(content_bounds, 12, 32).ok()?;
                let (_, body) = crate::ui::split_top_strip(inner, 48, 10).ok()?;
                let rows = crate::ui::stacked_rows(body, RoutingField::ALL.len(), 10);
                let fx_fields = [
                    RoutingField::RecordInputFx,
                    RoutingField::MonitorInputFx,
                    RoutingField::InputFxSlot,
                    RoutingField::InputFxKind,
                    RoutingField::InputFxEnabled,
                    RoutingField::InputFxParam1,
                    RoutingField::InputFxParam2,
                    RoutingField::InputFxMore,
                    RoutingField::OutputFxSlot,
                    RoutingField::OutputFxKind,
                    RoutingField::OutputFxEnabled,
                    RoutingField::OutputFxParam1,
                    RoutingField::OutputFxParam2,
                    RoutingField::OutputFxMore,
                ];
                let field = if fx_fields.contains(&self.page_state.selected_routing_field) {
                    self.page_state.selected_routing_field
                } else {
                    RoutingField::InputFxKind
                };
                let index = RoutingField::ALL.iter().position(|candidate| *candidate == field)?;
                capture_rect_from_rect(rows[index])
            }
            "routing_channel_fanout_rows" if page == AppPage::Routing => {
                let inner = crate::ui::inset_rect(content_bounds, 12, 32).ok()?;
                let (_, body) = crate::ui::split_top_strip(inner, 48, 10).ok()?;
                capture_rect_from_rect(body)
            }
            "midi_io_selected_list" if page == AppPage::MidiIo => {
                let (_, lists_bounds) = crate::ui::split_top_strip(content_bounds, 28, 10).ok()?;
                let columns = crate::ui::equal_columns(lists_bounds, 2, 14);
                let target = if self.page_state.midi_io.focus == MidiIoListFocus::Inputs {
                    columns[0]
                } else {
                    columns[1]
                };
                capture_rect_from_rect(target)
            }
            "midi_io_inputs_list" if page == AppPage::MidiIo => {
                let (_, lists_bounds) = crate::ui::split_top_strip(content_bounds, 28, 10).ok()?;
                let columns = crate::ui::equal_columns(lists_bounds, 2, 14);
                capture_rect_from_rect(columns[0])
            }
            "midi_io_outputs_list" if page == AppPage::MidiIo => {
                let (_, lists_bounds) = crate::ui::split_top_strip(content_bounds, 28, 10).ok()?;
                let columns = crate::ui::equal_columns(lists_bounds, 2, 14);
                capture_rect_from_rect(columns[1])
            }
            _ => None,
        }
    }

    fn apply_capture_annotations(
        &self,
        image: &mut RgbaImage,
        annotations: &CaptureAnnotations,
        page: AppPage,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for overlay in &annotations.overlays {
            match overlay {
                CaptureOverlay::Box { x, y, width, height, color, label } => {
                    let color = color.unwrap_or([248, 232, 146, 255]);
                    draw_rect_outline(image, *x, *y, *width, *height, color);
                    if let Some(label) = label {
                        draw_label_tag(image, *x, *y, label, color);
                    }
                }
                CaptureOverlay::Arrow { from, to, color, label } => {
                    let color = color.unwrap_or([252, 188, 128, 255]);
                    draw_line(image, from[0], from[1], to[0], to[1], color);
                    if let Some(label) = label {
                        draw_label_tag(image, to[0], to[1], label, color);
                    }
                }
                CaptureOverlay::RegionTint { region_id, color, label } => {
                    let tint = color.unwrap_or([92, 182, 232, 120]);
                    let Some(region) = self.capture_region_rect(page, region_id) else {
                        continue;
                    };
                    fill_rect_tint(image, region, tint);
                    if let Some(label) = label {
                        draw_label_tag(image, region.x, region.y, label, tint);
                    }
                }
            }
        }
        Ok(())
    }

    fn capture_content_bounds(&self) -> Result<Rect, String> {
        let surface = crate::ui::surface_rect(self.viewport_size.0, self.viewport_size.1);
        let inset = crate::ui::inset_rect(surface, 24, 24)?;
        let (_, page_area_bounds) = crate::ui::split_top_strip(inset, 28, 12)?;
        let footer_height = 22_u32;
        let footer_gap = 8_i32;
        Ok(Rect::new(
            page_area_bounds.x,
            page_area_bounds.y,
            page_area_bounds.width(),
            page_area_bounds
                .height()
                .saturating_sub(footer_height)
                .saturating_sub(footer_gap as u32),
        ))
    }

    fn mappings_selected_row_rect(&self, content_bounds: Rect) -> Option<Rect> {
        self.mappings_visible_row_rects(content_bounds)
            .into_iter()
            .find_map(|(index, rect)| {
                (index == self.page_state.selected_mapping_index).then_some(rect)
            })
    }

    fn mappings_visible_row_rects(&self, content_bounds: Rect) -> Vec<(usize, Rect)> {
        let list_bounds = Rect::new(
            content_bounds.x + 8,
            content_bounds.y + 44,
            content_bounds.width().saturating_sub(16),
            content_bounds.height().saturating_sub(68),
        );
        let row_gap = 3_i32;
        let row_height = 18_i32;
        let stride = row_height + row_gap;
        let visible_rows = ((list_bounds.height() as i32 + row_gap) / stride).max(1) as usize;
        let selected_index = self
            .page_state
            .selected_mapping_index
            .min(self.mappings.len().saturating_sub(1));
        let start_index = if self.mappings.len() <= visible_rows {
            0
        } else {
            selected_index
                .saturating_sub(visible_rows / 2)
                .min(self.mappings.len() - visible_rows)
        };
        let mut rows = Vec::new();
        for visible_index in 0..visible_rows {
            let index = start_index + visible_index;
            if index >= self.mappings.len() {
                break;
            }
            rows.push((
                index,
                Rect::new(
                    list_bounds.x,
                    list_bounds.y + visible_index as i32 * stride,
                    list_bounds.width(),
                    row_height as u32,
                ),
            ));
        }
        rows
    }

    fn resolve_automation_target_center(&self, target_id: &str) -> Option<(i32, i32)> {
        let content_bounds = self.capture_content_bounds().ok()?;
        self.automation_targets(content_bounds)
            .into_iter()
            .find(|target| {
                target.id == target_id && target.rect.width() > 0 && target.rect.height() > 0
            })
            .map(|target| {
                (
                    target.rect.x + (target.rect.width() as i32 / 2),
                    target.rect.y + (target.rect.height() as i32 / 2),
                )
            })
    }

    fn automation_targets(&self, content_bounds: Rect) -> Vec<AutomationTarget> {
        let mut targets = Vec::new();
        let surface = crate::ui::surface_rect(self.viewport_size.0, self.viewport_size.1);
        let inset = match crate::ui::inset_rect(surface, 24, 24) {
            Ok(value) => value,
            Err(_) => return targets,
        };
        let (tabs_bounds, _) = match crate::ui::split_top_strip(inset, 28, 12) {
            Ok(value) => value,
            Err(_) => return targets,
        };
        let (_, tabs_only) = page_tabs_layout(tabs_bounds);
        let rects = crate::ui::equal_columns(tabs_only, AppPage::ALL.len(), 10);
        for (index, page) in AppPage::ALL.iter().copied().enumerate() {
            targets.push(AutomationTarget {
                id: format!("tab.{}", capture_page_id(page)),
                page,
                rect: rects[index],
            });
        }

        match self.page_state.current_page {
            AppPage::Timeline => {
                if let Ok((header, body)) = crate::ui::split_top_strip(content_bounds, 28, 6) {
                    if let Ok((transport, _)) =
                        crate::ui::split_top_strip(body, transport_strip_height(), 8)
                    {
                        for (rect, action) in self.transport_chip_actions(transport) {
                            targets.push(AutomationTarget {
                                id: format!("timeline.transport.{}", capture_action_id(action)),
                                page: AppPage::Timeline,
                                rect,
                            });
                        }
                    }
                    if let Ok((_, timeline_bounds)) =
                        crate::ui::split_top_strip(body, transport_strip_height(), 8)
                    {
                        let columns = self.visible_track_columns(timeline_bounds);
                        for (index, full_bounds, detail_bounds) in &columns {
                            targets.push(AutomationTarget {
                                id: format!("timeline.track.{}.header", index + 1),
                                page: AppPage::Timeline,
                                rect: crate::ui::track_header_rect(*full_bounds, self.timeline_flow),
                            });
                            targets.push(AutomationTarget {
                                id: format!("timeline.track.{}.status", index + 1),
                                page: AppPage::Timeline,
                                rect: crate::ui::track_status_rect(
                                    crate::ui::union_rect(*full_bounds, *detail_bounds),
                                    self.timeline_flow,
                                ),
                            });
                            let detail_label =
                                crate::ui::track_label_rect(*detail_bounds, self.timeline_flow);
                            for (slot_index, slot_rect) in self.stored_loop_slot_rects(detail_label) {
                                targets.push(AutomationTarget {
                                    id: format!(
                                        "timeline.track.{}.stored-loop-slot.{}",
                                        index + 1,
                                        slot_index + 1
                                    ),
                                    page: AppPage::Timeline,
                                    rect: slot_rect,
                                });
                            }
                        }
                        if let Some((_, full_bounds, detail_bounds)) = columns
                            .into_iter()
                            .find(|(index, _, _)| *index == self.project.active_track_index)
                        {
                            targets.push(AutomationTarget {
                                id: "timeline.active-track.header".to_owned(),
                                page: AppPage::Timeline,
                                rect: crate::ui::track_header_rect(full_bounds, self.timeline_flow),
                            });
                            let detail_label =
                                crate::ui::track_label_rect(detail_bounds, self.timeline_flow);
                            let (mute_rect, delete_rect) =
                                self.recording_clip_control_rects(detail_label);
                            targets.push(AutomationTarget {
                                id: "timeline.active-track.clip-mute".to_owned(),
                                page: AppPage::Timeline,
                                rect: mute_rect,
                            });
                            targets.push(AutomationTarget {
                                id: "timeline.active-track.clip-delete".to_owned(),
                                page: AppPage::Timeline,
                                rect: delete_rect,
                            });
                            let (scroll_prev, scroll_next) =
                                self.recording_view_scroll_control_rects(detail_label);
                            targets.push(AutomationTarget {
                                id: "timeline.active-track.clip-scroll-prev".to_owned(),
                                page: AppPage::Timeline,
                                rect: scroll_prev,
                            });
                            targets.push(AutomationTarget {
                                id: "timeline.active-track.clip-scroll-next".to_owned(),
                                page: AppPage::Timeline,
                                rect: scroll_next,
                            });
                            if let DirectMappingMode::AwaitingInput(target) =
                                self.direct_mapping_state.mode
                            {
                                targets.push(AutomationTarget {
                                    id: "timeline.direct-mapping.awaiting-target".to_owned(),
                                    page: AppPage::Timeline,
                                    rect: target.hit_rect,
                                });
                            }
                        }
                    }
                    targets.push(AutomationTarget {
                        id: "timeline.focused-track-toggle".to_owned(),
                        page: AppPage::Timeline,
                        rect: self.focused_track_view_button_rect(header),
                    });
                    targets.push(AutomationTarget {
                        id: "timeline.reset-global-loop".to_owned(),
                        page: AppPage::Timeline,
                        rect: self.global_loop_reset_button_rect(header),
                    });
                }
            }
            AppPage::Mappings => {
                let overview_badge = Rect::new(content_bounds.x + 200, content_bounds.y + 8, 188, 16);
                let learn_badge = Rect::new(content_bounds.x + 392, content_bounds.y + 8, 136, 16);
                let direct_badge = Rect::new(content_bounds.x + 532, content_bounds.y + 8, 154, 16);
                targets.push(AutomationTarget {
                    id: "mappings.mode-toggle".to_owned(),
                    page: AppPage::Mappings,
                    rect: overview_badge,
                });
                targets.push(AutomationTarget {
                    id: "mappings.learn-toggle".to_owned(),
                    page: AppPage::Mappings,
                    rect: learn_badge,
                });
                targets.push(AutomationTarget {
                    id: "mappings.direct-toggle".to_owned(),
                    page: AppPage::Mappings,
                    rect: direct_badge,
                });
                targets.push(AutomationTarget {
                    id: "mappings.bank-panel".to_owned(),
                    page: AppPage::Mappings,
                    rect: Rect::new(
                        content_bounds.x + content_bounds.width() as i32 - 320,
                        content_bounds.y + 44,
                        312,
                        content_bounds.height().saturating_sub(52),
                    ),
                });
                for (index, rect) in self.mappings_visible_row_rects(content_bounds) {
                    targets.push(AutomationTarget {
                        id: format!("mappings.row.{}", index + 1),
                        page: AppPage::Mappings,
                        rect,
                    });
                }
                if let Some(row) = self.mappings_selected_row_rect(content_bounds) {
                    targets.push(AutomationTarget {
                        id: "mappings.selected-row".to_owned(),
                        page: AppPage::Mappings,
                        rect: row,
                    });
                    let cells = self.mapping_row_cells(row);
                    for field in MappingField::ALL {
                        let rect = cells[super::support::labels::mapping_field_index(field)];
                        targets.push(AutomationTarget {
                            id: format!("mappings.selected-field.{}", capture_mapping_field_id(field)),
                            page: AppPage::Mappings,
                            rect,
                        });
                    }
                }
                if let DirectMappingMode::AwaitingInput(target) = self.direct_mapping_state.mode {
                    targets.push(AutomationTarget {
                        id: "mappings.direct-mapping.awaiting-target".to_owned(),
                        page: AppPage::Mappings,
                        rect: target.hit_rect,
                    });
                }
            }
            AppPage::MidiIo => {
                if let Ok((_, lists_bounds)) = crate::ui::split_top_strip(content_bounds, 28, 10) {
                    let columns = crate::ui::equal_columns(lists_bounds, 2, 14);
                    let input = columns[0];
                    let output = columns[1];
                    let input_header = Rect::new(input.x, input.y, input.width(), 22);
                    let output_header = Rect::new(output.x, output.y, output.width(), 22);
                    targets.push(AutomationTarget { id: "midi-io.inputs".to_owned(), page: AppPage::MidiIo, rect: input });
                    targets.push(AutomationTarget { id: "midi-io.outputs".to_owned(), page: AppPage::MidiIo, rect: output });
                    targets.push(AutomationTarget { id: "midi-io.inputs.header".to_owned(), page: AppPage::MidiIo, rect: input_header });
                    targets.push(AutomationTarget { id: "midi-io.outputs.header".to_owned(), page: AppPage::MidiIo, rect: output_header });
                    let input_list = Rect::new(input.x, input_header.y + input_header.height() as i32 + 6, input.width(), input.height().saturating_sub(input_header.height().saturating_add(28)));
                    let output_list = Rect::new(output.x, output_header.y + output_header.height() as i32 + 6, output.width(), output.height().saturating_sub(output_header.height().saturating_add(28)));
                    if let Ok(inset) = crate::ui::inset_rect(input_list, 10, 10) {
                        let rows = crate::ui::stacked_rows(inset, self.midi_devices.inputs.len().max(1), 8);
                        for (index, row) in rows.iter().enumerate().take(self.midi_devices.inputs.len()) {
                            targets.push(AutomationTarget { id: format!("midi-io.inputs.row.{}", index + 1), page: AppPage::MidiIo, rect: *row });
                        }
                        if let Some(rect) = rows.get(self.page_state.midi_io.selected_input_index.min(self.midi_devices.inputs.len().saturating_sub(1))).copied() {
                            targets.push(AutomationTarget { id: "midi-io.inputs.selected".to_owned(), page: AppPage::MidiIo, rect });
                        }
                    }
                    if let Ok(inset) = crate::ui::inset_rect(output_list, 10, 10) {
                        let rows = crate::ui::stacked_rows(inset, self.midi_devices.outputs.len().max(1), 8);
                        for (index, row) in rows.iter().enumerate().take(self.midi_devices.outputs.len()) {
                            targets.push(AutomationTarget { id: format!("midi-io.outputs.row.{}", index + 1), page: AppPage::MidiIo, rect: *row });
                        }
                        if let Some(rect) = rows.get(self.page_state.midi_io.selected_output_index.min(self.midi_devices.outputs.len().saturating_sub(1))).copied() {
                            targets.push(AutomationTarget { id: "midi-io.outputs.selected".to_owned(), page: AppPage::MidiIo, rect });
                        }
                    }
                }
            }
            AppPage::Routing => {
                if let Ok(inner) = crate::ui::inset_rect(content_bounds, 12, 32) {
                    if let Ok((header, body)) = crate::ui::split_top_strip(inner, 48, 10) {
                        targets.push(AutomationTarget {
                            id: "routing.meta.active-track".to_owned(),
                            page: AppPage::Routing,
                            rect: Rect::new(header.x + 8, header.y + 8, 90, header.height().saturating_sub(16)),
                        });
                        targets.push(AutomationTarget {
                            id: "routing.meta.passthrough".to_owned(),
                            page: AppPage::Routing,
                            rect: Rect::new(header.x + 106, header.y + 8, 92, header.height().saturating_sub(16)),
                        });
                        let rows = crate::ui::stacked_rows(body, RoutingField::ALL.len(), 10);
                        for (index, field) in RoutingField::ALL.iter().copied().enumerate() {
                            let row = rows[index];
                            targets.push(AutomationTarget { id: format!("routing.row.{}", capture_routing_field_id(field)), page: AppPage::Routing, rect: row });
                            let value = Rect::new(row.x + 156, row.y + 8, row.width().saturating_sub(220), row.height().saturating_sub(16));
                            targets.push(AutomationTarget { id: format!("routing.row.{}.value", capture_routing_field_id(field)), page: AppPage::Routing, rect: value });
                            let affordance = Rect::new(row.x + row.width() as i32 - 72, row.y + 8, 62, row.height().saturating_sub(16));
                            targets.push(AutomationTarget { id: format!("routing.row.{}.affordance", capture_routing_field_id(field)), page: AppPage::Routing, rect: affordance });
                        }
                        if let DirectMappingMode::AwaitingInput(target) = self.direct_mapping_state.mode {
                            targets.push(AutomationTarget {
                                id: "routing.direct-mapping.awaiting-target".to_owned(),
                                page: AppPage::Routing,
                                rect: target.hit_rect,
                            });
                        }
                    }
                }
            }
        }

        targets
    }

    fn capture_state_hash(&self) -> String {
        let state = self.persisted_state();
        match serde_json::to_vec(&state) {
            Ok(bytes) => simple_hash_hex(&bytes),
            Err(_) => "unavailable".to_owned(),
        }
    }
}

fn capture_overlay_label(overlay: AppOverlay) -> &'static str {
    match overlay {
        AppOverlay::MappingsQuickView => "mappings_quick_view",
        AppOverlay::Discoverability => "discoverability",
    }
}

fn parse_capture_overlay(value: &str) -> Result<Option<AppOverlay>, Box<dyn std::error::Error>> {
    match value.trim().to_ascii_lowercase().as_str() {
        "none" => Ok(None),
        "mappings_quick_view" | "mappings-overlay" => Ok(Some(AppOverlay::MappingsQuickView)),
        "discoverability" | "discoverability-overlay" => Ok(Some(AppOverlay::Discoverability)),
        _ => Err(format!("unknown capture overlay: {value}").into()),
    }
}

fn parse_app_page(value: &str) -> Result<AppPage, Box<dyn std::error::Error>> {
    match value.trim().to_ascii_lowercase().as_str() {
        "timeline" => Ok(AppPage::Timeline),
        "mappings" => Ok(AppPage::Mappings),
        "midi-io" | "midi_io" => Ok(AppPage::MidiIo),
        "routing" => Ok(AppPage::Routing),
        _ => Err(format!("unknown page: {value}").into()),
    }
}

fn capture_page_id(page: AppPage) -> &'static str {
    match page {
        AppPage::Timeline => "timeline",
        AppPage::Mappings => "mappings",
        AppPage::MidiIo => "midi-io",
        AppPage::Routing => "routing",
    }
}

fn capture_region_name(region_id: &str) -> Option<&'static str> {
    let normalized = region_id.replace('-', "_");
    match normalized.as_str() {
        "timeline_transport" => Some("Timeline transport strip"),
        "timeline_recwrap_quantize_strip" => Some("RecWrap and quantize row"),
        "timeline_link_status_strip" => Some("Link, sync, and peer status area"),
        "timeline_track_header_active" => Some("Active track header"),
        "transport_left" => Some("Transport left controls"),
        "transport_right" => Some("Transport right status panel"),
        "status_strip" => Some("Active track status strip"),
        "timeline_header_controls" => Some("Timeline header controls"),
        "timeline_stacked_clip_controls" => Some("Stacked clip mute and delete controls"),
        "timeline_direct_mapping_target" => Some("Timeline direct-mapping target highlight"),
        "mappings_selected_row" => Some("Selected mapping row"),
        "mappings_bank_panel" => Some("Mappings bank panel"),
        "mappings_direct_mapping_target" => Some("Mappings direct-mapping target highlight"),
        "routing_active_row" => Some("Selected routing row"),
        "routing_passthrough_block" => Some("Routing passthrough block"),
        "routing_channel_fanout_rows" => Some("Routing channel fan-out rows"),
        "routing_direct_mapping_target" => Some("Routing direct-mapping target highlight"),
        "fx_row" => Some("FX row hotspot"),
        "midi_io_selected_list" => Some("Selected MIDI I/O list"),
        "midi_io_inputs_list" => Some("MIDI input list"),
        "midi_io_outputs_list" => Some("MIDI output list"),
        _ => None,
    }
}

fn parse_capture_action(value: &str) -> Result<AppAction, Box<dyn std::error::Error>> {
    let normalized = value.trim();
    if let Some(action) = parse_select_track_action(normalized) {
        return Ok(action);
    }
    if let Some(action) = parse_stored_loop_action(normalized) {
        return Ok(action);
    }
    match normalized {
        "show_page_timeline" | "ShowPageTimeline" => Ok(AppAction::ShowPage(AppPage::Timeline)),
        "show_page_mappings" | "ShowPageMappings" => Ok(AppAction::ShowPage(AppPage::Mappings)),
        "show_page_midi_io" | "ShowPageMidiIo" => Ok(AppAction::ShowPage(AppPage::MidiIo)),
        "show_page_routing" | "ShowPageRouting" => Ok(AppAction::ShowPage(AppPage::Routing)),
        "show_next_page" | "ShowNextPage" => Ok(AppAction::ShowNextPage),
        "show_previous_page" | "ShowPreviousPage" => Ok(AppAction::ShowPreviousPage),
        "toggle_playback" | "TogglePlayback" => Ok(AppAction::TogglePlayback),
        "toggle_recording" | "ToggleRecording" => Ok(AppAction::ToggleRecording),
        "cycle_record_mode" | "CycleRecordMode" => Ok(AppAction::CycleRecordMode),
        "toggle_record_wrap" | "ToggleLoopRecordingExtension" => {
            Ok(AppAction::ToggleLoopRecordingExtension)
        }
        "toggle_mappings_overlay" | "ToggleMappingsOverlay" => Ok(AppAction::ToggleMappingsOverlay),
        "toggle_discoverability_overlay" | "ToggleDiscoverabilityOverlay" => {
            Ok(AppAction::ToggleDiscoverabilityOverlay)
        }
        "toggle_direct_mapping_mode" | "ToggleDirectMappingMode" => {
            Ok(AppAction::ToggleDirectMappingMode)
        }
        "toggle_mappings_write_mode" | "ToggleMappingsWriteMode" => {
            Ok(AppAction::ToggleMappingsWriteMode)
        }
        "add_mapping_row" | "AddMappingRow" => Ok(AppAction::AddMappingRow),
        "remove_selected_mapping" | "RemoveSelectedMapping" => Ok(AppAction::RemoveSelectedMapping),
        "activate_page_item" | "ActivatePageItem" => Ok(AppAction::ActivatePageItem),
        "select_next_page_item" | "SelectNextPageItem" => Ok(AppAction::SelectNextPageItem),
        "select_previous_page_item" | "SelectPreviousPageItem" => {
            Ok(AppAction::SelectPreviousPageItem)
        }
        "adjust_page_item_forward" | "AdjustPageItemForward" => Ok(AppAction::AdjustPageItemForward),
        "adjust_page_item_backward" | "AdjustPageItemBackward" => Ok(AppAction::AdjustPageItemBackward),
        "toggle_current_track_arm" | "ToggleCurrentTrackArm" => Ok(AppAction::ToggleCurrentTrackArm),
        "toggle_current_track_mute" | "ToggleCurrentTrackMute" => Ok(AppAction::ToggleCurrentTrackMute),
        "toggle_current_track_solo" | "ToggleCurrentTrackSolo" => Ok(AppAction::ToggleCurrentTrackSolo),
        "toggle_current_track_recording_view" | "ToggleCurrentTrackRecordingView" => Ok(AppAction::ToggleCurrentTrackRecordingView),
        "select_previous_recording_clip" | "SelectPreviousRecordingClip" => Ok(AppAction::SelectPreviousRecordingClip),
        "select_next_recording_clip" | "SelectNextRecordingClip" => Ok(AppAction::SelectNextRecordingClip),
        "toggle_selected_recording_clip_mute" | "ToggleSelectedRecordingClipMute" => Ok(AppAction::ToggleSelectedRecordingClipMute),
        "delete_selected_recording_clip" | "DeleteSelectedRecordingClip" => Ok(AppAction::DeleteSelectedRecordingClip),
        "select_next_track" | "SelectNextTrack" => Ok(AppAction::SelectNextTrack),
        "select_previous_track" | "SelectPreviousTrack" => Ok(AppAction::SelectPreviousTrack),
        "toggle_focused_track_view" | "ToggleFocusedTrackView" => Ok(AppAction::ToggleFocusedTrackView),
        "toggle_link" | "ToggleLinkEnabled" => Ok(AppAction::ToggleLinkEnabled),
        "toggle_link_sync" | "ToggleLinkStartStopSync" => Ok(AppAction::ToggleLinkStartStopSync),
        "toggle_global_loop" | "ToggleGlobalLoop" => Ok(AppAction::ToggleGlobalLoop),
        "reset_global_loop" | "ResetGlobalLoop" => Ok(AppAction::ResetGlobalLoop),
        "clear_current_track_content" | "ClearCurrentTrackContent" => Ok(AppAction::ClearCurrentTrackContent),
        "clear_all_track_content" | "ClearAllTrackContent" => Ok(AppAction::ClearAllTrackContent),
        "set_current_track_loop_start" | "SetCurrentTrackLoopStart" => Ok(AppAction::SetCurrentTrackLoopStart),
        "set_current_track_loop_end" | "SetCurrentTrackLoopEnd" => Ok(AppAction::SetCurrentTrackLoopEnd),
        "set_global_loop_start" | "SetGlobalLoopStart" => Ok(AppAction::SetGlobalLoopStart),
        "set_global_loop_end" | "SetGlobalLoopEnd" => Ok(AppAction::SetGlobalLoopEnd),
        "nudge_current_track_loop_backward" | "NudgeCurrentTrackLoopBackward" => Ok(AppAction::NudgeCurrentTrackLoopBackward),
        "nudge_current_track_loop_forward" | "NudgeCurrentTrackLoopForward" => Ok(AppAction::NudgeCurrentTrackLoopForward),
        "nudge_global_loop_backward" | "NudgeGlobalLoopBackward" => Ok(AppAction::NudgeGlobalLoopBackward),
        "nudge_global_loop_forward" | "NudgeGlobalLoopForward" => Ok(AppAction::NudgeGlobalLoopForward),
        "shorten_current_track_loop" | "ShortenCurrentTrackLoop" => Ok(AppAction::ShortenCurrentTrackLoop),
        "extend_current_track_loop" | "ExtendCurrentTrackLoop" => Ok(AppAction::ExtendCurrentTrackLoop),
        "half_current_track_loop" | "HalfCurrentTrackLoop" => Ok(AppAction::HalfCurrentTrackLoop),
        "double_current_track_loop" | "DoubleCurrentTrackLoop" => Ok(AppAction::DoubleCurrentTrackLoop),
        "shorten_global_loop" | "ShortenGlobalLoop" => Ok(AppAction::ShortenGlobalLoop),
        "extend_global_loop" | "ExtendGlobalLoop" => Ok(AppAction::ExtendGlobalLoop),
        "half_global_loop" | "HalfGlobalLoop" => Ok(AppAction::HalfGlobalLoop),
        "double_global_loop" | "DoubleGlobalLoop" => Ok(AppAction::DoubleGlobalLoop),
        "toggle_stored_loop_recall_quantize" | "ToggleStoredLoopRecallQuantize" => Ok(AppAction::ToggleStoredLoopRecallQuantize),
        "cycle_stored_loop_launch_quantize" | "CycleStoredLoopLaunchQuantize" => Ok(AppAction::CycleStoredLoopLaunchQuantize),
        "toggle_current_track_loop" | "ToggleCurrentTrackLoop" => Ok(AppAction::ToggleCurrentTrackLoop),
        "toggle_current_track_passthrough" | "ToggleCurrentTrackPassthrough" => Ok(AppAction::ToggleCurrentTrackPassthrough),
        other => Err(format!("unknown capture action: {other}").into()),
    }
}

fn parse_select_track_action(value: &str) -> Option<AppAction> {
    let lowercase = value.to_ascii_lowercase();
    if let Some(raw) = lowercase.strip_prefix("select_track_") {
        let index = raw.parse::<usize>().ok()?.saturating_sub(1);
        return Some(AppAction::SelectTrack(index));
    }
    if let Some(raw) = lowercase.strip_prefix("select_track:") {
        let index = raw.parse::<usize>().ok()?.saturating_sub(1);
        return Some(AppAction::SelectTrack(index));
    }
    None
}

fn parse_stored_loop_action(value: &str) -> Option<AppAction> {
    let lowercase = value.to_ascii_lowercase();
    if let Some(raw) = lowercase.strip_prefix("recall_stored_loop_slot_") {
        let slot = raw.parse::<usize>().ok()?.saturating_sub(1);
        return super::stored_loops::stored_loop_slot_recall_action(slot);
    }
    if let Some(raw) = lowercase.strip_prefix("store_current_loop_to_slot_") {
        let slot = raw.parse::<usize>().ok()?.saturating_sub(1);
        return match slot {
            0 => Some(AppAction::StoreCurrentLoopToSlot1),
            1 => Some(AppAction::StoreCurrentLoopToSlot2),
            2 => Some(AppAction::StoreCurrentLoopToSlot3),
            3 => Some(AppAction::StoreCurrentLoopToSlot4),
            4 => Some(AppAction::StoreCurrentLoopToSlot5),
            5 => Some(AppAction::StoreCurrentLoopToSlot6),
            6 => Some(AppAction::StoreCurrentLoopToSlot7),
            7 => Some(AppAction::StoreCurrentLoopToSlot8),
            _ => None,
        };
    }
    if let Some(raw) = lowercase.strip_prefix("clear_stored_loop_slot_") {
        let slot = raw.parse::<usize>().ok()?.saturating_sub(1);
        return match slot {
            0 => Some(AppAction::ClearStoredLoopSlot1),
            1 => Some(AppAction::ClearStoredLoopSlot2),
            2 => Some(AppAction::ClearStoredLoopSlot3),
            3 => Some(AppAction::ClearStoredLoopSlot4),
            4 => Some(AppAction::ClearStoredLoopSlot5),
            5 => Some(AppAction::ClearStoredLoopSlot6),
            6 => Some(AppAction::ClearStoredLoopSlot7),
            7 => Some(AppAction::ClearStoredLoopSlot8),
            _ => None,
        };
    }
    None
}

fn parse_mapping_mode(value: &str) -> Result<MappingPageMode, Box<dyn std::error::Error>> {
    match value.trim().to_ascii_lowercase().as_str() {
        "read_only" | "read-only" | "overview" => Ok(MappingPageMode::Overview),
        "write" => Ok(MappingPageMode::Write),
        _ => Err(format!("unknown mapping mode: {value}").into()),
    }
}

fn parse_mapping_field(value: &str) -> Result<MappingField, Box<dyn std::error::Error>> {
    match value.trim().to_ascii_lowercase().as_str() {
        "source_kind" => Ok(MappingField::SourceKind),
        "source_value" => Ok(MappingField::SourceValue),
        "source_device" => Ok(MappingField::SourceDevice),
        "target" => Ok(MappingField::Target),
        "scope" => Ok(MappingField::Scope),
        "enabled" => Ok(MappingField::Enabled),
        _ => Err(format!("unknown mapping field: {value}").into()),
    }
}

fn parse_routing_field(value: &str) -> Result<RoutingField, Box<dyn std::error::Error>> {
    match value.trim().to_ascii_lowercase().as_str() {
        "input_device" => Ok(RoutingField::InputDevice),
        "input_channel" => Ok(RoutingField::InputChannel),
        "output_device" => Ok(RoutingField::OutputDevice),
        "output_channel" => Ok(RoutingField::OutputChannel),
        "passthrough" => Ok(RoutingField::Passthrough),
        _ => Err(format!("unknown routing field: {value}").into()),
    }
}

fn parse_midi_focus(value: &str) -> Result<MidiIoListFocus, Box<dyn std::error::Error>> {
    match value.trim().to_ascii_lowercase().as_str() {
        "inputs" => Ok(MidiIoListFocus::Inputs),
        "outputs" => Ok(MidiIoListFocus::Outputs),
        _ => Err(format!("unknown midi focus value: {value}").into()),
    }
}

fn parse_recording_view(value: &str) -> Result<RecordingView, Box<dyn std::error::Error>> {
    match value.trim().to_ascii_lowercase().as_str() {
        "overlay" => Ok(RecordingView::Overlay),
        "stacked" => Ok(RecordingView::Stacked),
        _ => Err(format!("unknown recording_view value: {value}").into()),
    }
}

fn capture_script_step_label(step: &CaptureScriptStep) -> String {
    match step {
        CaptureScriptStep::ShowPage { page } => format!("show_page_{page}"),
        CaptureScriptStep::SendAction { action } => format!("action_{action}"),
        CaptureScriptStep::Click { named_target, .. } => {
            named_target.clone().unwrap_or_else(|| "click".to_owned())
        }
        CaptureScriptStep::WaitFrames { frames } => format!("wait_{frames}_frames"),
        CaptureScriptStep::SetStateOverride { .. } => "state_override".to_owned(),
    }
}

fn capture_action_id(action: AppAction) -> &'static str {
    match action {
        AppAction::TogglePlayback => "toggle_playback",
        AppAction::ToggleRecording => "toggle_recording",
        AppAction::CycleRecordMode => "cycle_record_mode",
        AppAction::ToggleLoopRecordingExtension => "toggle_record_wrap",
        AppAction::ToggleGlobalLoop => "toggle_global_loop",
        AppAction::ResetGlobalLoop => "reset_global_loop",
        AppAction::ToggleCurrentTrackLoop => "toggle_track_loop",
        AppAction::ToggleLinkEnabled => "toggle_link",
        AppAction::ToggleLinkStartStopSync => "toggle_link_sync",
        AppAction::ToggleStoredLoopRecallQuantize => "toggle_launch_quantize",
        AppAction::CycleStoredLoopLaunchQuantize => "cycle_launch_quantize",
        AppAction::ToggleCurrentTrackArm => "toggle_track_arm",
        AppAction::ToggleCurrentTrackMute => "toggle_track_mute",
        AppAction::ToggleCurrentTrackSolo => "toggle_track_solo",
        AppAction::ToggleCurrentTrackPassthrough => "toggle_track_passthrough",
        AppAction::SelectNextTrack => "select_next_track",
        AppAction::SelectPreviousTrack => "select_previous_track",
        AppAction::SelectNextRecordingClip => "select_next_clip",
        AppAction::SelectPreviousRecordingClip => "select_previous_clip",
        AppAction::ToggleSelectedRecordingClipMute => "toggle_selected_clip_mute",
        AppAction::DeleteSelectedRecordingClip => "delete_selected_clip",
        _ => "action",
    }
}

fn capture_mapping_field_id(field: MappingField) -> &'static str {
    match field {
        MappingField::SourceKind => "source_kind",
        MappingField::SourceValue => "source_value",
        MappingField::SourceDevice => "source_device",
        MappingField::Target => "target",
        MappingField::Scope => "scope",
        MappingField::Enabled => "enabled",
    }
}

fn capture_routing_field_id(field: RoutingField) -> &'static str {
    match field {
        RoutingField::InputDevice => "input_device",
        RoutingField::InputChannel => "input_channel",
        RoutingField::OutputDevice => "output_device",
        RoutingField::OutputChannel => "output_channel",
        RoutingField::Passthrough => "passthrough",
        RoutingField::RecordInputFx => "record_input_fx",
        RoutingField::MonitorInputFx => "monitor_input_fx",
        RoutingField::InputFxSlot => "input_fx_slot",
        RoutingField::InputFxKind => "input_fx_kind",
        RoutingField::InputFxEnabled => "input_fx_enabled",
        RoutingField::InputFxParam1 => "input_fx_param_1",
        RoutingField::InputFxParam2 => "input_fx_param_2",
        RoutingField::InputFxMore => "input_fx_more",
        RoutingField::OutputFxSlot => "output_fx_slot",
        RoutingField::OutputFxKind => "output_fx_kind",
        RoutingField::OutputFxEnabled => "output_fx_enabled",
        RoutingField::OutputFxParam1 => "output_fx_param_1",
        RoutingField::OutputFxParam2 => "output_fx_param_2",
        RoutingField::OutputFxMore => "output_fx_more",
    }
}

fn read_json_file<T: for<'de> Deserialize<'de>>(
    path: &Path,
) -> Result<T, Box<dyn std::error::Error>> {
    let bytes = fs::read(path)?;
    Ok(serde_json::from_slice(&bytes)?)
}

fn surface_ref_to_rgba_image(
    surface: &SurfaceRef,
) -> Result<RgbaImage, Box<dyn std::error::Error>> {
    let surface = surface.convert_format(PixelFormat::RGBA32)?;
    let width = surface.width();
    let height = surface.height();
    let pitch = surface.pitch() as usize;
    let row_len = width as usize * 4;
    let mut pixels = vec![0_u8; row_len * height as usize];

    surface.with_lock(|src| {
        for row in 0..height as usize {
            let src_start = row * pitch;
            let dst_start = row * row_len;
            pixels[dst_start..dst_start + row_len]
                .copy_from_slice(&src[src_start..src_start + row_len]);
        }
    });

    RgbaImage::from_raw(width, height, pixels)
        .ok_or_else(|| "failed to convert renderer pixels to image".into())
}

fn crop_rgba_image(
    image: &RgbaImage,
    rect: CaptureRect,
) -> Result<RgbaImage, Box<dyn std::error::Error>> {
    let width = image.width();
    let height = image.height();
    if rect.width == 0 || rect.height == 0 {
        return Err("capture crop width/height must be greater than zero".into());
    }
    if rect.x >= width || rect.y >= height {
        return Err("capture crop starts outside source image bounds".into());
    }
    let crop_width = rect.width.min(width - rect.x);
    let crop_height = rect.height.min(height - rect.y);
    Ok(image::imageops::crop_imm(image, rect.x, rect.y, crop_width, crop_height).to_image())
}

fn apply_capture_padding(
    rect: CaptureRect,
    padding: CapturePadding,
    image_width: u32,
    image_height: u32,
) -> Option<CaptureRect> {
    let left = rect.x as i64 - padding.left as i64;
    let top = rect.y as i64 - padding.top as i64;
    let right = rect.x as i64 + rect.width as i64 + padding.right as i64;
    let bottom = rect.y as i64 + rect.height as i64 + padding.bottom as i64;
    let clamped_left = left.clamp(0, image_width as i64);
    let clamped_top = top.clamp(0, image_height as i64);
    let clamped_right = right.clamp(0, image_width as i64);
    let clamped_bottom = bottom.clamp(0, image_height as i64);
    if clamped_right <= clamped_left || clamped_bottom <= clamped_top {
        return None;
    }
    Some(CaptureRect {
        x: clamped_left as u32,
        y: clamped_top as u32,
        width: (clamped_right - clamped_left) as u32,
        height: (clamped_bottom - clamped_top) as u32,
    })
}

fn capture_rect_from_rect(rect: Rect) -> Option<CaptureRect> {
    if rect.width() == 0 || rect.height() == 0 || rect.x < 0 || rect.y < 0 {
        return None;
    }
    Some(CaptureRect {
        x: rect.x as u32,
        y: rect.y as u32,
        width: rect.width(),
        height: rect.height(),
    })
}

fn now_iso8601_string() -> String {
    use std::time::SystemTime;
    match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(duration) => format!("{}", duration.as_secs()),
        Err(_) => "0".to_owned(),
    }
}

fn git_commit_hash() -> Option<String> {
    let output = Command::new("git").args(["rev-parse", "HEAD"]).output().ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8(output.stdout)
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

fn simple_hash_hex(bytes: &[u8]) -> String {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in bytes {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}

fn draw_rect_outline(
    image: &mut RgbaImage,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    color: [u8; 4],
) {
    if width == 0 || height == 0 {
        return;
    }
    for dx in 0..width {
        put_pixel_checked(image, x + dx, y, color);
        put_pixel_checked(image, x + dx, y + height.saturating_sub(1), color);
    }
    for dy in 0..height {
        put_pixel_checked(image, x, y + dy, color);
        put_pixel_checked(image, x + width.saturating_sub(1), y + dy, color);
    }
}

fn draw_line(
    image: &mut RgbaImage,
    from_x: u32,
    from_y: u32,
    to_x: u32,
    to_y: u32,
    color: [u8; 4],
) {
    let mut x0 = from_x as i32;
    let mut y0 = from_y as i32;
    let x1 = to_x as i32;
    let y1 = to_y as i32;
    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    loop {
        if x0 >= 0 && y0 >= 0 {
            put_pixel_checked(image, x0 as u32, y0 as u32, color);
        }
        if x0 == x1 && y0 == y1 {
            break;
        }
        let e2 = err * 2;
        if e2 >= dy {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            err += dx;
            y0 += sy;
        }
    }
}

fn fill_rect_tint(image: &mut RgbaImage, rect: CaptureRect, tint: [u8; 4]) {
    for y in rect.y..rect.y.saturating_add(rect.height).min(image.height()) {
        for x in rect.x..rect.x.saturating_add(rect.width).min(image.width()) {
            let pixel = image.get_pixel_mut(x, y);
            let alpha = tint[3] as u16;
            for channel in 0..3 {
                let src = pixel[channel] as u16;
                let over = tint[channel] as u16;
                pixel[channel] = (((src * (255 - alpha)) + (over * alpha)) / 255) as u8;
            }
        }
    }
}

fn draw_label_tag(image: &mut RgbaImage, x: u32, y: u32, label: &str, color: [u8; 4]) {
    let width = (label.len() as u32 * 5).clamp(20, 180);
    let height = 10;
    for py in y..y.saturating_add(height).min(image.height()) {
        for px in x..x.saturating_add(width).min(image.width()) {
            put_pixel_checked(image, px, py, [color[0], color[1], color[2], 220]);
        }
    }
}

fn put_pixel_checked(image: &mut RgbaImage, x: u32, y: u32, color: [u8; 4]) {
    if x >= image.width() || y >= image.height() {
        return;
    }
    let pixel = image.get_pixel_mut(x, y);
    pixel[0] = color[0];
    pixel[1] = color[1];
    pixel[2] = color[2];
    pixel[3] = color[3];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capture_region_name_supports_review_hotspot_aliases() {
        assert_eq!(capture_region_name("transport-left"), Some("Transport left controls"));
        assert_eq!(
            capture_region_name("timeline-header-controls"),
            Some("Timeline header controls")
        );
        assert_eq!(capture_region_name("fx-row"), Some("FX row hotspot"));
    }

    #[test]
    fn review_hotspot_regions_resolve_for_timeline_and_routing_pages() {
        let mut app = App::new_demo();
        app.viewport_size = (1280, 720);
        app.page_state.current_page = AppPage::Timeline;
        assert!(app.capture_region_rect(AppPage::Timeline, "transport-left").is_some());
        assert!(app.capture_region_rect(AppPage::Timeline, "transport-right").is_some());
        assert!(app.capture_region_rect(AppPage::Timeline, "status-strip").is_some());
        assert!(app.capture_region_rect(AppPage::Timeline, "timeline-header-controls").is_some());

        app.page_state.current_page = AppPage::Routing;
        app.page_state.selected_routing_field = RoutingField::InputFxKind;
        assert!(app.capture_region_rect(AppPage::Routing, "fx-row").is_some());
    }
}
