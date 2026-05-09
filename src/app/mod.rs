use crate::actions::{
    ActionSource, AppAction, KeyboardBindings, action_label, built_in_keyboard_binding_labels,
};
use crate::app_ui::branding;
use crate::engine::EngineConfig;
use crate::link::{LinkRuntime, LinkSnapshot};
use crate::mapping::{
    MappingEntry, MappingSourceKind, cycle_mapping_scope_value, cycle_mapping_source_device_label,
    cycle_mapping_source_kind, cycle_mapping_source_label, cycle_mapping_target_label,
    default_mapping_source_device, default_scope_label, default_source_label, demo_mappings,
    mapping_entry_key_actions, mapping_entry_targets_action, mapping_entry_to_actions,
    mapping_scope_valid_for_target,
};
use crate::midi_fx::{
    LiveMidiFxEvent, LiveMidiFxState, MIDI_FX_SLOT_COUNT, MidiFx, MidiFxChainKind, MidiFxSlot,
    cycle_existing_fx_kind, cycle_fx_kind, fx_slot_label, note_name,
    playback_timing_lookback_ticks, process_live_chain_event, process_live_chain_tick,
    reset_live_fx_timing, transform_notes,
};
use crate::midi_io::{
    MidiDeviceCatalog, MidiInputEvent, MidiInputMessage, MidiInputRuntime, MidiOutputRuntime,
    MidiPortRef,
};
use crate::page_widgets::{handle_page_pointer, page_discoverability_targets, render_page};
use crate::pages::{
    AppPage, AppPageState, MappingField, MappingPageMode, MidiIoListFocus, RoutingField,
};
use crate::project::{
    ClipAlignApplyMode, ClipAlignDestination, ClipAlignSettings, ClipAlignSourceEndMode,
    ClipAlignSourceStartMode, MidiNote, Project, RecordingView, STORED_LOOP_SLOT_COUNT, Track,
};
use crate::routing::{MidiChannelFilter, TrackPortSelection};
use crate::state::PersistedAppState;
use crate::theme::{Theme, ThemePreset};
use crate::timeline_fx::{TimelineContext, TimelineFxField};
use crate::ui::{LayoutMode, TimelineFlow};
use crate::ui_density::{UiDensityPreset, UiMetrics, ui_metrics};
use crate::undo::UndoHistory;
use image::RgbaImage;
use sdl3::pixels::{Color, PixelFormat};
use sdl3::rect::Rect;
use sdl3::render::{Canvas, RenderTarget};
use sdl3::surface::SurfaceRef;
use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};

mod capture;
mod direct_mapping_ui;
mod discoverability_ui;
mod input;
mod support;

mod mapping;

mod midi_io_page;
mod note_runtime;
mod routing_ui;
mod shell;
mod stored_loops;
mod timeline;
mod types;

use capture::{capture_specs, readback_color_at, readback_rect_rgba, seed_capture_demo_track};
use discoverability_ui::track_indicator_target;
pub(super) use input::rect_contains;
use mapping::input as mapping_input;
use mapping::lookup as mapping_lookup;
use mapping::ui as mapping_ui;
use mapping_input::{midi_learn_label, midi_mapping_matches_event};
use mapping_lookup::mapping_target_lookup_input;
use mapping_ui::{direct_mapping_key_label, mapping_target_label_for_action};
use note_runtime::{scheduled_note_occurrences, ticks_per_second_for_tempo};
use shell::scaling::{
    active_draw_size, effective_ui_scale, logical_viewport_size, should_interpolate_window_scale,
};
pub(super) use shell::ui::transport_strip_height;
use stored_loops::{
    clear_stored_loop_slot_index, recall_stored_loop_slot_index, store_stored_loop_slot_index,
    stored_loop_slot_color, stored_loop_slot_recall_action,
};
use support::io_helpers::{clamp_index, resolve_port_by_name};
use support::labels::{
    action_source_label, badge_kind_prefix, compact_badge_text, compact_scope_label,
    input_channel_label, launch_quantize_label, mapping_badge_palette, mapping_field_index,
    mapping_source_label, mapping_source_sort_key, on_off, output_channel_label, quantize_label,
};
use support::midi_runtime::{MidiRuntime, MidiRuntimeStateSync, MidiRuntimeUiSnapshot};
use support::ui_helpers::{centered_text_rect, contrasting_text_color};
use timeline::layout::{
    displayed_track_fx_band_height, timeline_subcolumn_content_rect, timeline_subcolumn_label_rect,
};
pub(crate) use types::DiscoverabilityTarget;
use types::{
    ActionDiscoverabilitySummary, ActiveMappingTargetLookup, AppOverlay, ClipAlignField,
    ClipAlignSession, DirectMappingMode, DirectMappingOrigin, DirectMappingState,
    DirectMappingTarget, LastActionStatus, MappingBadge, MappingTargetLookupLayout,
    MappingTargetLookupState, OverlayState, RecordingLaneLayout, RecordingLaneWindow, StatusState,
    TimelineFxRowLayout, TimelineFxRowRef, TimelineTrackLayout,
};
pub use types::{RunOptions, UiCaptureOptions, UiScalingMode, VideoMode};

const MIDI_REFRESH_INTERVAL: Duration = Duration::from_millis(1_000);
const MIDI_RUNTIME_APP_DIAG_INTERVAL: Duration = Duration::from_secs(1);

/// App is the top-level composition root for the first vertical slice.
pub struct App {
    project: Project,
    engine_config: EngineConfig,
    layout_mode: LayoutMode,
    timeline_flow: TimelineFlow,
    keyboard_bindings: KeyboardBindings,
    page_state: AppPageState,
    midi_devices: MidiDeviceCatalog,
    midi_input: MidiInputRuntime,
    midi_output: MidiOutputRuntime,
    midi_runtime: MidiRuntime,
    link: LinkRuntime,
    mappings: Vec<MappingEntry>,
    overlay_state: OverlayState,
    status_state: StatusState,
    direct_mapping_state: DirectMappingState,
    target_lookup_state: MappingTargetLookupState,
    clip_align_defaults: ClipAlignSettings,
    clip_align_session: Option<ClipAlignSession>,
    viewport_size: (u32, u32),
    ui_scale_override: Option<f32>,
    ui_scaling_mode: UiScalingMode,
    theme_preset: ThemePreset,
    ui_density_preset: UiDensityPreset,
    transport_ticks: u64,
    playhead_ticks: u64,
    live_fx_ticks: u64,
    link_snapshot: LinkSnapshot,
    note_additive_select_held: bool,
    focused_track_view: bool,
    last_tempo_tap_at: Option<Instant>,
    startup_started_at: Instant,
    last_midi_refresh_at: Instant,
    preferred_default_input_name: Option<String>,
    preferred_default_output_name: Option<String>,
    input_fx_live_states: Vec<LiveMidiFxState>,
    output_fx_live_states: Vec<LiveMidiFxState>,
    undo_history: UndoHistory,
    midi_runtime_dirty: bool,
    last_runtime_snapshot: MidiRuntimeUiSnapshot,
    midi_runtime_diag_enabled: bool,
    last_midi_runtime_diag_at: Instant,
    midi_runtime_sync_count: u64,
    midi_runtime_sync_skipped_count: u64,
    midi_runtime_sync_total_ns: u64,
    midi_runtime_sync_max_ns: u64,
}

impl App {
    pub fn new() -> Self {
        Self::new_demo()
    }

    pub fn new_demo() -> Self {
        let mut app = Self::with_project(Project::demo(), demo_mappings(), AppPageState::default());
        app.seed_demo_routing();
        app
    }

    pub fn new_empty() -> Self {
        let mut app =
            Self::with_project(Project::empty(), demo_mappings(), AppPageState::default());
        app.seed_demo_routing();
        for track in &mut app.project.tracks {
            track.clear_content();
        }
        app
    }

    pub fn from_persisted_state(state: PersistedAppState) -> Self {
        let mut app = Self::with_project(state.project, state.mappings, state.page_state);
        app.timeline_flow = state.timeline_flow;
        app.transport_ticks = state.transport_ticks;
        app.playhead_ticks = state.playhead_ticks;
        app.live_fx_ticks = state.transport_ticks;
        app.sync_midi_inputs();
        app
    }

    pub fn persisted_state(&self) -> PersistedAppState {
        PersistedAppState {
            project: self.project.clone(),
            page_state: self.page_state,
            timeline_flow: self.timeline_flow,
            mappings: self.mappings.clone(),
            transport_ticks: self.transport_ticks,
            playhead_ticks: self.playhead_ticks,
        }
    }

    pub(crate) fn theme(&self) -> &'static Theme {
        crate::theme::theme(self.theme_preset)
    }

    pub(crate) fn ui_metrics(&self) -> &'static UiMetrics {
        ui_metrics(self.ui_density_preset)
    }

    fn with_project(
        project: Project,
        mappings: Vec<MappingEntry>,
        page_state: AppPageState,
    ) -> Self {
        let scanned_devices = MidiDeviceCatalog::scan();
        let preferred_default_input_name = scanned_devices
            .selected_input_port()
            .map(|port| port.name.clone());
        let preferred_default_output_name = scanned_devices
            .selected_output_port()
            .map(|port| port.name.clone());
        let mut link = LinkRuntime::new(f64::from(project.transport.tempo_bpm));
        link.set_enabled(project.transport.link_enabled);
        link.set_start_stop_sync(project.transport.link_start_stop_sync);
        let link_snapshot = link.refresh();
        let track_count = project.tracks.len();
        let midi_output = MidiOutputRuntime::default();
        let midi_runtime = MidiRuntime::new(midi_output.clone());
        let mut midi_input = MidiInputRuntime::default();
        midi_input.set_fanout_sender(Some(midi_runtime.input_sender()));
        Self {
            project,
            engine_config: EngineConfig::default(),
            layout_mode: LayoutMode::FixedFit,
            timeline_flow: TimelineFlow::DownwardColumns,
            keyboard_bindings: KeyboardBindings,
            page_state,
            midi_devices: scanned_devices,
            midi_input,
            midi_output,
            midi_runtime,
            link,
            mappings,
            overlay_state: OverlayState::default(),
            status_state: StatusState::default(),
            direct_mapping_state: DirectMappingState::default(),
            target_lookup_state: MappingTargetLookupState::default(),
            clip_align_defaults: ClipAlignSettings::default(),
            clip_align_session: None,
            viewport_size: (1280, 720),
            ui_scale_override: None,
            ui_scaling_mode: UiScalingMode::Auto,
            theme_preset: ThemePreset::from_env(),
            ui_density_preset: UiDensityPreset::from_env(),
            transport_ticks: 0,
            playhead_ticks: 0,
            live_fx_ticks: 0,
            link_snapshot,
            note_additive_select_held: false,
            focused_track_view: false,
            last_tempo_tap_at: None,
            startup_started_at: Instant::now(),
            last_midi_refresh_at: Instant::now() - MIDI_REFRESH_INTERVAL,
            preferred_default_input_name,
            preferred_default_output_name,
            input_fx_live_states: vec![LiveMidiFxState::default(); track_count],
            output_fx_live_states: vec![LiveMidiFxState::default(); track_count],
            undo_history: UndoHistory::default(),
            midi_runtime_dirty: true,
            last_runtime_snapshot: MidiRuntimeUiSnapshot::default(),
            midi_runtime_diag_enabled: std::env::var("TREKR_MIDI_RUNTIME_LOG")
                .ok()
                .is_some_and(|value| value != "0"),
            last_midi_runtime_diag_at: Instant::now(),
            midi_runtime_sync_count: 0,
            midi_runtime_sync_skipped_count: 0,
            midi_runtime_sync_total_ns: 0,
            midi_runtime_sync_max_ns: 0,
        }
    }

    pub fn set_ui_scale_override(&mut self, scale: Option<f32>) {
        self.ui_scale_override = scale.filter(|value| *value >= 1.0);
    }

    pub fn set_ui_scaling_mode(&mut self, mode: UiScalingMode) {
        self.ui_scaling_mode = mode;
    }

    pub fn set_theme_preset(&mut self, preset: ThemePreset) {
        self.theme_preset = preset;
    }

    pub fn set_ui_density_preset(&mut self, preset: UiDensityPreset) {
        self.ui_density_preset = preset;
    }

    pub fn bootstrap_summary(&self) -> String {
        format!(
            "trekr bootstrap: project='{}', tracks={}, active_track={}, page={}, layout={:?}, theme={}, density={}, sample_rate={}, song_ticks={}, playing={}, loop_enabled={}, midi_inputs={}, midi_outputs={}",
            self.project.name,
            self.project.tracks.len(),
            self.project.active_track_index + 1,
            self.page_state.current_page.label(),
            self.layout_mode,
            self.theme_preset.label(),
            self.ui_density_preset.label(),
            self.engine_config.sample_rate_hz,
            self.project.full_song_range().length_ticks,
            self.project.transport.playing,
            self.project.transport.loop_enabled,
            self.midi_devices.inputs.len(),
            self.midi_devices.outputs.len(),
        )
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.run_with_options(RunOptions::default())
    }

    pub fn run_with_options(
        &mut self,
        options: RunOptions,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.startup_started_at = Instant::now();

        if options.video_mode == VideoMode::KmsDrmConsole {
            // Force SDL onto the DRM/KMS backend for minimal Linux console targets.
            sdl3::hint::set_with_priority(
                "SDL_VIDEO_DRIVER",
                "kmsdrm",
                &sdl3::hint::Hint::Override,
            );
            sdl3::hint::set_with_priority(
                "SDL_KMSDRM_REQUIRE_DRM_MASTER",
                "1",
                &sdl3::hint::Hint::Override,
            );
            sdl3::hint::set_video_minimize_on_focus_loss(false);
        }

        let sdl_context = sdl3::init()?;
        let video = sdl_context.video()?;
        println!("trekr video driver: {}", video.current_video_driver());

        let (window_width, window_height) = initial_window_size(&video, options.video_mode);
        let mut window_builder = video.window("trekr", window_width, window_height);
        match options.video_mode {
            VideoMode::Windowed => {
                window_builder
                    .position_centered()
                    .resizable()
                    .high_pixel_density();
            }
            VideoMode::Fullscreen | VideoMode::KmsDrmConsole => {
                window_builder
                    .fullscreen()
                    .borderless()
                    .high_pixel_density();
            }
        }
        let window = window_builder.build().map_err(|err| err.to_string())?;
        if options.video_mode != VideoMode::Windowed {
            let _ = window.sync();
        }
        if options.video_mode == VideoMode::KmsDrmConsole {
            let present_mode = std::env::var("TREKR_KMSDRM_PRESENT_MODE")
                .unwrap_or_else(|_| "renderer".to_owned());
            if present_mode.eq_ignore_ascii_case("surface") {
                return self.run_kmsdrm_surface_console(sdl_context, window);
            }
            return self.run_kmsdrm_renderer_console(sdl_context, window);
        }

        let mut canvas = window.into_canvas();
        self.configure_window_canvas(&mut canvas)?;
        let mut event_pump = sdl_context.event_pump()?;
        let started_at = Instant::now();
        let mut last_frame_at = started_at;
        let auto_exit_after = std::env::var("TREKR_EXIT_AFTER_MS")
            .ok()
            .and_then(|value| value.parse::<u64>().ok())
            .map(Duration::from_millis);

        'running: loop {
            for event in event_pump.poll_iter() {
                let pointer_event = event.get_converted_coords(&canvas).unwrap_or(event.clone());
                if let Some(control) = self.handle_pointer_event(&pointer_event) {
                    if control == AppControl::Quit {
                        break 'running;
                    }
                    continue;
                }

                if let Some(control) = self.handle_keyboard_event(&event) {
                    if control == AppControl::Quit {
                        break 'running;
                    }
                }
            }

            if auto_exit_after.is_some_and(|limit| started_at.elapsed() >= limit) {
                break 'running;
            }

            self.poll_midi_input();
            let now = Instant::now();
            self.maybe_refresh_midi_devices(now);
            self.advance_playhead(now.saturating_duration_since(last_frame_at));
            last_frame_at = now;
            self.configure_window_canvas(&mut canvas)?;

            self.update_window_title(canvas.window_mut())?;
            self.draw_window(&mut canvas)?;
            if options.video_mode != VideoMode::Windowed {
                let _ = canvas.window_mut().sync();
            }
            std::thread::sleep(Duration::from_millis(16));
        }

        Ok(())
    }

    fn run_kmsdrm_renderer_console(
        &mut self,
        sdl_context: sdl3::Sdl,
        window: sdl3::video::Window,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut canvas = window.into_canvas();
        self.configure_window_canvas(&mut canvas)?;
        let mut event_pump = sdl_context.event_pump()?;
        let started_at = Instant::now();
        let mut last_frame_at = started_at;
        let auto_exit_after = std::env::var("TREKR_EXIT_AFTER_MS")
            .ok()
            .and_then(|value| value.parse::<u64>().ok())
            .map(Duration::from_millis);

        'running: loop {
            for event in event_pump.poll_iter() {
                let pointer_event = event.get_converted_coords(&canvas).unwrap_or(event.clone());
                if let Some(control) = self.handle_pointer_event(&pointer_event) {
                    if control == AppControl::Quit {
                        break 'running;
                    }
                    continue;
                }

                if let Some(control) = self.handle_keyboard_event(&event) {
                    if control == AppControl::Quit {
                        break 'running;
                    }
                }
            }

            if auto_exit_after.is_some_and(|limit| started_at.elapsed() >= limit) {
                break 'running;
            }

            self.poll_midi_input();
            let now = Instant::now();
            self.maybe_refresh_midi_devices(now);
            self.advance_playhead(now.saturating_duration_since(last_frame_at));
            last_frame_at = now;
            self.configure_window_canvas(&mut canvas)?;

            self.update_window_title(canvas.window_mut())?;
            self.draw_window(&mut canvas)?;
            let _ = canvas.window_mut().sync();
            std::thread::sleep(Duration::from_millis(16));
        }

        Ok(())
    }
    fn run_kmsdrm_surface_console(
        &mut self,
        sdl_context: sdl3::Sdl,
        mut window: sdl3::video::Window,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut event_pump = sdl_context.event_pump()?;
        let started_at = Instant::now();
        let mut last_frame_at = started_at;
        let auto_exit_after = std::env::var("TREKR_EXIT_AFTER_MS")
            .ok()
            .and_then(|value| value.parse::<u64>().ok())
            .map(Duration::from_millis);
        let show_test_pattern = std::env::var("TREKR_KMSDRM_TEST_PATTERN")
            .ok()
            .is_some_and(|value| value != "0");

        'running: loop {
            for event in event_pump.poll_iter() {
                if let Some(control) = self.handle_pointer_event(&event) {
                    if control == AppControl::Quit {
                        break 'running;
                    }
                    continue;
                }

                if let Some(control) = self.handle_keyboard_event(&event) {
                    if control == AppControl::Quit {
                        break 'running;
                    }
                }
            }

            if auto_exit_after.is_some_and(|limit| started_at.elapsed() >= limit) {
                break 'running;
            }

            self.poll_midi_input();
            let now = Instant::now();
            self.maybe_refresh_midi_devices(now);
            self.advance_playhead(now.saturating_duration_since(last_frame_at));
            last_frame_at = now;
            self.viewport_size = window.size_in_pixels();

            self.update_window_title(&mut window)?;

            let mut window_surface = window.surface(&event_pump)?;
            if show_test_pattern {
                self.draw_kmsdrm_test_pattern(&mut window_surface)?;
            } else {
                let frame = self.draw_frame_surface(window.window_pixel_format())?;
                frame.blit_scaled(
                    None,
                    &mut window_surface,
                    None,
                    sdl3::sys::surface::SDL_SCALEMODE_LINEAR,
                )?;
            }
            window_surface.finish()?;
            let _ = window.sync();

            std::thread::sleep(Duration::from_millis(16));
        }

        Ok(())
    }

    pub fn capture_ui_pages(
        &mut self,
        options: UiCaptureOptions,
    ) -> Result<(), Box<dyn std::error::Error>> {
        fs::create_dir_all(&options.output_dir)?;

        let _sdl_context = sdl3::init()?;
        self.viewport_size = (1280, 720);
        // Keep renderer-owned screenshots deterministic by bypassing startup-only pulses.
        self.startup_started_at = Instant::now() - Duration::from_secs(10);
        self.seed_capture_demo_midi_devices();

        for spec in capture_specs() {
            self.page_state.current_page = spec.page;
            self.overlay_state.active = spec.overlay;
            self.focused_track_view = spec.focused_track_view;
            self.clip_align_session = None;
            if spec.open_clip_align && spec.page == AppPage::Timeline {
                self.open_selected_recording_clip_align();
            }
            let surface = sdl3::surface::Surface::new(1280, 720, PixelFormat::RGBA32)?;
            let mut canvas = surface.into_canvas()?;
            canvas.set_scale(1.0, 1.0)?;
            self.draw(&mut canvas)?;
            let output_path = options.output_dir.join(spec.filename);
            self.capture_surface_to_png(canvas.surface(), &output_path)?;
        }

        self.overlay_state.active = None;

        Ok(())
    }

    fn seed_capture_demo_midi_devices(&mut self) {
        self.midi_devices = MidiDeviceCatalog::demo();
        self.page_state.midi_io.selected_input_index =
            self.midi_devices.selected_input.unwrap_or(0);
        self.page_state.midi_io.selected_output_index =
            self.midi_devices.selected_output.unwrap_or(0);
        self.preferred_default_input_name = self
            .midi_devices
            .selected_input_port()
            .map(|port| port.name.clone());
        self.preferred_default_output_name = self
            .midi_devices
            .selected_output_port()
            .map(|port| port.name.clone());
    }

    pub fn seed_capture_demo_timeline_overlaps(&mut self) {
        for (track_index, track) in self.project.tracks.iter_mut().enumerate() {
            seed_capture_demo_track(track, track_index);
        }
        self.project.active_track_index = 0;
        self.transport_ticks = 0;
        self.playhead_ticks = 0;
    }

    fn draw_kmsdrm_test_pattern(
        &self,
        surface: &mut sdl3::video::WindowSurfaceRef<'_>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let width = self.viewport_size.0.max(1);
        let height = self.viewport_size.1.max(1);
        let stripe_width = (width / 3).max(1);
        surface.fill_rect(None, Color::RGB(12, 12, 12))?;
        surface.fill_rect(
            Rect::new(0, 0, stripe_width, height),
            Color::RGB(220, 32, 32),
        )?;
        surface.fill_rect(
            Rect::new(stripe_width as i32, 0, stripe_width, height),
            Color::RGB(32, 220, 32),
        )?;
        surface.fill_rect(
            Rect::new(
                (stripe_width * 2) as i32,
                0,
                width - stripe_width * 2,
                height,
            ),
            Color::RGB(32, 64, 220),
        )?;
        Ok(())
    }

    fn capture_surface_to_png(
        &self,
        surface: &SurfaceRef,
        path: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
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

        let image = RgbaImage::from_raw(width, height, pixels)
            .ok_or_else(|| "failed to convert renderer pixels to image".to_owned())?;
        image.save(path)?;
        Ok(())
    }

    fn update_window_title(
        &self,
        window: &mut sdl3::video::Window,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let active = self
            .project
            .active_track()
            .expect("demo project always has tracks");
        let title = match self.page_state.current_page {
            AppPage::Timeline => format!(
                "trekr | Page:{} (Tab/F1-F4) | T{} {} | Tick:{} | Space Play:{} | R Rec:{} | Shift+R Mode:{} | F6 Link:{} Shift+F6 Sync:{} | F7 Discover:{} | F8 Direct:{} | Peers:{} | C Clear Track | Shift+C Clear All | [ ] TrackLoop:{}-{} | , . Nudge | - = Resize | / \\ Half/Double | Shift+[ ] SongLoop:{}-{} | G:{} L:{} A:{} M:{} S:{} I:{}",
                self.page_state.current_page.label(),
                self.project.active_track_index + 1,
                active.name,
                self.playhead_ticks,
                on_off(self.project.transport.playing),
                on_off(self.project.transport.recording),
                self.project.transport.record_mode.label(),
                on_off(self.project.transport.link_enabled),
                on_off(self.project.transport.link_start_stop_sync),
                on_off(self.overlay_state.active == Some(AppOverlay::Discoverability)),
                on_off(self.direct_mapping_state.mode != DirectMappingMode::Inactive),
                self.link_snapshot.peers,
                active.loop_region.start_ticks,
                active.loop_region.end_ticks(),
                self.project.loop_region.start_ticks,
                self.project.loop_region.end_ticks(),
                on_off(self.project.transport.loop_enabled),
                on_off(active.state.loop_enabled),
                on_off(active.state.armed),
                on_off(active.state.muted),
                on_off(active.state.soloed),
                on_off(active.state.passthrough),
            ),
            AppPage::Mappings => {
                let selected = &self.mappings[self.page_state.selected_mapping_index];
                format!(
                    "trekr | Page:{} (Tab/F1-F4) | Mode:{} | F5 Overlay:{} | F7 Discover:{} | F8 Direct:{} | W Toggle Mode | N New | Del/Bsp Remove | Shift+Left/Right Field:{} | Learn:{} | Up/Down Select | Source:{} {} | Device:{} | Target:{} | Scope:{} | Enabled:{}",
                    self.page_state.current_page.label(),
                    self.page_state.mapping_mode.label(),
                    on_off(self.overlay_state.active == Some(AppOverlay::MappingsQuickView)),
                    on_off(self.overlay_state.active == Some(AppOverlay::Discoverability)),
                    on_off(self.direct_mapping_state.mode != DirectMappingMode::Inactive),
                    self.page_state.selected_mapping_field.label(),
                    on_off(self.page_state.mapping_midi_learn_armed),
                    mapping_source_label(selected.source_kind),
                    selected.source_label,
                    selected.source_device_label,
                    selected.target_label,
                    selected.scope_label,
                    on_off(selected.enabled),
                )
            }
            AppPage::MidiIo => {
                let focus = match self.page_state.midi_io.focus {
                    MidiIoListFocus::Inputs => "Inputs",
                    MidiIoListFocus::Outputs => "Outputs",
                };
                let selected = match self.page_state.midi_io.focus {
                    MidiIoListFocus::Inputs => self
                        .midi_devices
                        .input(self.page_state.midi_io.selected_input_index)
                        .map(|port| port.name.as_str())
                        .unwrap_or("none"),
                    MidiIoListFocus::Outputs => self
                        .midi_devices
                        .output(self.page_state.midi_io.selected_output_index)
                        .map(|port| port.name.as_str())
                        .unwrap_or("none"),
                };
                format!(
                    "trekr | Page:{} (Tab/F1-F4) | Focus:{} | F8 Direct:{} | Up/Down Select | Q/E Switch List | Enter Set Default | Selected:{} | Default In:{} | Default Out:{}",
                    self.page_state.current_page.label(),
                    focus,
                    on_off(self.direct_mapping_state.mode != DirectMappingMode::Inactive),
                    selected,
                    self.midi_devices
                        .selected_input_port()
                        .map(|port| port.name.as_str())
                        .unwrap_or("none"),
                    self.midi_devices
                        .selected_output_port()
                        .map(|port| port.name.as_str())
                        .unwrap_or("none"),
                )
            }
            AppPage::Routing => format!(
                "trekr | Page:{} (Tab/F1-F4) | T{} {} | F8 Direct:{} | Up/Down Field | Q/E Adjust | Enter Toggle | Field:{} | In:{} {} | Out:{} {} | Thru:{} | RecFX:{} | MonFX:{} | IFX:{} | OFX:{}",
                self.page_state.current_page.label(),
                self.project.active_track_index + 1,
                active.name,
                on_off(self.direct_mapping_state.mode != DirectMappingMode::Inactive),
                self.page_state.selected_routing_field.label(),
                self.routing_input_selection_label(&active.routing.input_port),
                input_channel_label(active.routing.input_channel),
                self.routing_output_selection_label(&active.routing.output_port),
                output_channel_label(active.routing.output_channel),
                on_off(active.state.passthrough),
                active.midi_fx.record_input_fx_mode.label(),
                on_off(active.midi_fx.monitor_input_fx),
                fx_slot_label(
                    active
                        .midi_fx
                        .input_fx
                        .get(self.page_state.selected_input_fx_slot)
                        .and_then(|slot| slot.as_ref())
                ),
                fx_slot_label(
                    active
                        .midi_fx
                        .output_fx
                        .get(self.page_state.selected_output_fx_slot)
                        .and_then(|slot| slot.as_ref())
                ),
            ),
        };
        window.set_title(&title)?;
        Ok(())
    }

    fn apply_action_inner(&mut self, action: AppAction) -> AppControl {
        match action {
            AppAction::Undo
            | AppAction::Redo
            | AppAction::UndoTimeline
            | AppAction::RedoTimeline
            | AppAction::UndoMappings
            | AppAction::RedoMappings
            | AppAction::UndoUi
            | AppAction::RedoUi => {
                unreachable!("undo actions are handled before apply_action_inner")
            }
            AppAction::Quit => AppControl::Quit,
            AppAction::ShowPage(page) => {
                self.clear_mapping_target_lookup();
                self.page_state.current_page = page;
                if page != AppPage::Timeline {
                    self.close_clip_align();
                }
                self.sync_midi_inputs();
                AppControl::Continue
            }
            AppAction::ShowNextPage => {
                self.clear_mapping_target_lookup();
                self.page_state.current_page = self.page_state.current_page.next();
                if self.page_state.current_page != AppPage::Timeline {
                    self.close_clip_align();
                }
                self.sync_midi_inputs();
                AppControl::Continue
            }
            AppAction::ShowPreviousPage => {
                self.clear_mapping_target_lookup();
                self.page_state.current_page = self.page_state.current_page.previous();
                if self.page_state.current_page != AppPage::Timeline {
                    self.close_clip_align();
                }
                self.sync_midi_inputs();
                AppControl::Continue
            }
            AppAction::SelectPreviousPageItem => {
                self.select_previous_page_item();
                AppControl::Continue
            }
            AppAction::SelectNextPageItem => {
                self.select_next_page_item();
                AppControl::Continue
            }
            AppAction::AdjustPageItemBackward => {
                self.adjust_page_item(-1);
                AppControl::Continue
            }
            AppAction::AdjustPageItemForward => {
                self.adjust_page_item(1);
                AppControl::Continue
            }
            AppAction::ActivatePageItem => {
                self.activate_page_item();
                AppControl::Continue
            }
            AppAction::ReverseActivatePageItem => {
                self.reverse_activate_page_item();
                AppControl::Continue
            }
            AppAction::CancelCurrentMode => {
                if self.target_lookup_state.active.is_some() {
                    self.cancel_mapping_target_lookup();
                } else if self.clip_align_session.is_some() {
                    self.close_clip_align();
                } else if self.direct_mapping_state.mode != DirectMappingMode::Inactive {
                    self.cancel_direct_mapping("Canceled direct mapping.");
                }
                AppControl::Continue
            }
            AppAction::ToggleMappingsOverlay => {
                self.overlay_state.active =
                    if self.overlay_state.active == Some(AppOverlay::MappingsQuickView) {
                        None
                    } else {
                        Some(AppOverlay::MappingsQuickView)
                    };
                self.sync_midi_inputs();
                AppControl::Continue
            }
            AppAction::ToggleDiscoverabilityOverlay => {
                self.overlay_state.active =
                    if self.overlay_state.active == Some(AppOverlay::Discoverability) {
                        None
                    } else {
                        Some(AppOverlay::Discoverability)
                    };
                AppControl::Continue
            }
            AppAction::ToggleDirectMappingMode => {
                self.toggle_direct_mapping_mode();
                AppControl::Continue
            }
            AppAction::ToggleMappingsWriteMode => {
                self.clear_mapping_target_lookup();
                self.page_state.mapping_mode = self.page_state.mapping_mode.toggle();
                self.page_state.mapping_midi_learn_armed = false;
                if self.page_state.mapping_mode == MappingPageMode::Overview {
                    self.page_state.selected_mapping_field = MappingField::SourceValue;
                } else {
                    self.normalize_selected_mapping_field();
                }
                self.sync_midi_inputs();
                AppControl::Continue
            }
            AppAction::AddMappingRow => {
                self.add_mapping_row();
                AppControl::Continue
            }
            AppAction::RemoveSelectedMapping => {
                self.remove_selected_mapping();
                AppControl::Continue
            }
            AppAction::SelectPreviousPageField => {
                self.select_previous_page_field();
                AppControl::Continue
            }
            AppAction::SelectNextPageField => {
                self.select_next_page_field();
                AppControl::Continue
            }
            AppAction::ToggleLinkEnabled => {
                self.project.transport.link_enabled = !self.project.transport.link_enabled;
                self.link.set_enabled(self.project.transport.link_enabled);
                self.link_snapshot = self.link.refresh();
                AppControl::Continue
            }
            AppAction::ToggleLinkStartStopSync => {
                self.project.transport.link_start_stop_sync =
                    !self.project.transport.link_start_stop_sync;
                self.link
                    .set_start_stop_sync(self.project.transport.link_start_stop_sync);
                self.link_snapshot = self.link.refresh();
                AppControl::Continue
            }
            AppAction::TogglePlayback => {
                let was_playing = self.project.transport.playing;
                if self.project.transport.playing && self.project.transport.recording {
                    self.finish_recording();
                }
                self.project.transport.playing = !self.project.transport.playing;
                if self.project.transport.link_enabled {
                    self.link.commit_playing(
                        self.project.transport.playing,
                        self.transport_ticks as f64 / f64::from(self.project.transport.ppqn.max(1)),
                    );
                    self.link_snapshot = self.link.refresh();
                }
                if !self.project.transport.playing {
                    self.live_fx_ticks = self.transport_ticks;
                    self.reset_live_fx_timing(self.live_fx_ticks);
                    self.silence_all_tracks();
                } else if !was_playing {
                    self.live_fx_ticks = self.transport_ticks;
                    self.reset_live_fx_timing(self.transport_ticks);
                }
                AppControl::Continue
            }
            AppAction::ToggleRecording => {
                if self.project.transport.recording {
                    self.finish_recording();
                } else {
                    self.begin_recording();
                }
                AppControl::Continue
            }
            AppAction::StartRecording => {
                if !self.project.transport.recording {
                    self.begin_recording();
                }
                AppControl::Continue
            }
            AppAction::StopRecording => {
                if self.project.transport.recording {
                    self.finish_recording();
                }
                AppControl::Continue
            }
            AppAction::CycleRecordMode => {
                self.project.transport.record_mode = self.project.transport.record_mode.next();
                AppControl::Continue
            }
            AppAction::ToggleLoopRecordingExtension => {
                self.project.transport.loop_recording_extends_clip =
                    !self.project.transport.loop_recording_extends_clip;
                AppControl::Continue
            }
            AppAction::DecreaseTempo => {
                self.set_transport_tempo(
                    self.project.transport.tempo_bpm.saturating_sub(1).max(20),
                );
                AppControl::Continue
            }
            AppAction::IncreaseTempo => {
                self.set_transport_tempo(
                    self.project.transport.tempo_bpm.saturating_add(1).min(400),
                );
                AppControl::Continue
            }
            AppAction::HalfTempo => {
                self.set_transport_tempo((self.project.transport.tempo_bpm / 2).max(20));
                AppControl::Continue
            }
            AppAction::DoubleTempo => {
                self.set_transport_tempo(
                    self.project.transport.tempo_bpm.saturating_mul(2).min(400),
                );
                AppControl::Continue
            }
            AppAction::TapTempo => {
                self.tap_transport_tempo();
                AppControl::Continue
            }
            AppAction::ToggleGlobalLoop => {
                self.project.transport.loop_enabled = !self.project.transport.loop_enabled;
                self.silence_tracks_for_loop_change();
                AppControl::Continue
            }
            AppAction::CycleGlobalHarmonyRoot => {
                self.project.global_harmony.root = (self.project.global_harmony.root + 1) % 12;
                AppControl::Continue
            }
            AppAction::ResetGlobalLoop => {
                self.project.loop_region = self.project.full_song_range();
                self.project.transport.loop_enabled = true;
                self.playhead_ticks = self.playhead_ticks.clamp(
                    self.project.loop_region.start_ticks,
                    self.project.loop_region.end_ticks(),
                );
                self.silence_tracks_for_loop_change();
                AppControl::Continue
            }
            AppAction::ClearCurrentTrackContent => {
                if let Some(track) = self.project.active_track_mut() {
                    track.clear_content();
                }
                AppControl::Continue
            }
            AppAction::ClearAllTrackContent => {
                self.project.clear_all_track_content();
                AppControl::Continue
            }
            AppAction::ToggleCurrentTrackLoop => {
                if let Some(track) = self.project.active_track_mut() {
                    track.state.loop_enabled = !track.state.loop_enabled;
                }
                self.silence_tracks_for_loop_change();
                AppControl::Continue
            }
            AppAction::ToggleStoredLoopRecallQuantize => {
                self.project.transport.stored_loop_recall_quantized =
                    !self.project.transport.stored_loop_recall_quantized;
                AppControl::Continue
            }
            AppAction::CycleStoredLoopLaunchQuantize => {
                self.project.transport.stored_loop_launch_quantize =
                    self.project.transport.stored_loop_launch_quantize.next();
                AppControl::Continue
            }
            AppAction::RecallStoredLoopSlot1
            | AppAction::RecallStoredLoopSlot2
            | AppAction::RecallStoredLoopSlot3
            | AppAction::RecallStoredLoopSlot4
            | AppAction::RecallStoredLoopSlot5
            | AppAction::RecallStoredLoopSlot6
            | AppAction::RecallStoredLoopSlot7
            | AppAction::RecallStoredLoopSlot8 => {
                let slot_index =
                    recall_stored_loop_slot_index(action).expect("stored loop recall checked");
                let launch_quantize = self.project.transport.stored_loop_launch_quantize;
                let quantized = self.project.transport.stored_loop_recall_quantized
                    && self.project.transport.playing
                    && launch_quantize != crate::transport::LaunchQuantizeMode::Off;
                if let Some(track) = self.project.active_track_mut() {
                    if track.active_take.is_some() {
                        return AppControl::Continue;
                    }
                    if track.stored_loop_slot(slot_index).is_some() {
                        track.state.loop_enabled = true;
                    }
                    if quantized {
                        track.queue_stored_loop_recall(
                            slot_index,
                            launch_quantize,
                            self.transport_ticks,
                        );
                    } else {
                        track.recall_stored_loop_slot(slot_index);
                    }
                }
                AppControl::Continue
            }
            AppAction::StoreCurrentLoopToSlot1
            | AppAction::StoreCurrentLoopToSlot2
            | AppAction::StoreCurrentLoopToSlot3
            | AppAction::StoreCurrentLoopToSlot4
            | AppAction::StoreCurrentLoopToSlot5
            | AppAction::StoreCurrentLoopToSlot6
            | AppAction::StoreCurrentLoopToSlot7
            | AppAction::StoreCurrentLoopToSlot8 => {
                let slot_index =
                    store_stored_loop_slot_index(action).expect("stored loop store checked");
                if let Some(track) = self.project.active_track_mut() {
                    track.store_current_loop_to_slot(slot_index);
                }
                AppControl::Continue
            }
            AppAction::ClearStoredLoopSlot1
            | AppAction::ClearStoredLoopSlot2
            | AppAction::ClearStoredLoopSlot3
            | AppAction::ClearStoredLoopSlot4
            | AppAction::ClearStoredLoopSlot5
            | AppAction::ClearStoredLoopSlot6
            | AppAction::ClearStoredLoopSlot7
            | AppAction::ClearStoredLoopSlot8 => {
                let slot_index =
                    clear_stored_loop_slot_index(action).expect("stored loop clear checked");
                if let Some(track) = self.project.active_track_mut() {
                    track.clear_stored_loop_slot(slot_index);
                }
                AppControl::Continue
            }
            AppAction::SetCurrentTrackLoopStart => {
                let edit_ticks = self.current_edit_ticks();
                if let Some(track) = self.project.active_track_mut() {
                    track.loop_region.set_start_preserving_end(edit_ticks);
                    track.sync_active_stored_loop_slot();
                }
                self.silence_tracks_for_loop_change();
                AppControl::Continue
            }
            AppAction::SetCurrentTrackLoopEnd => {
                let edit_ticks = self.current_edit_ticks();
                if let Some(track) = self.project.active_track_mut() {
                    track.loop_region.set_end(edit_ticks);
                    track.sync_active_stored_loop_slot();
                }
                self.silence_tracks_for_loop_change();
                AppControl::Continue
            }
            AppAction::SetGlobalLoopStart => {
                let edit_ticks = self.current_edit_ticks();
                self.project
                    .loop_region
                    .set_start_preserving_end(edit_ticks);
                self.silence_tracks_for_loop_change();
                AppControl::Continue
            }
            AppAction::SetGlobalLoopEnd => {
                let edit_ticks = self.current_edit_ticks();
                self.project.loop_region.set_end(edit_ticks);
                self.silence_tracks_for_loop_change();
                AppControl::Continue
            }
            AppAction::NudgeCurrentTrackLoopBackward => {
                let delta = -(self.nudge_step_ticks() as i64);
                if let Some(track) = self.project.active_track_mut() {
                    track.loop_region.shift_by(delta);
                    track.sync_active_stored_loop_slot();
                }
                self.silence_tracks_for_loop_change();
                AppControl::Continue
            }
            AppAction::NudgeCurrentTrackLoopForward => {
                let delta = self.nudge_step_ticks() as i64;
                if let Some(track) = self.project.active_track_mut() {
                    track.loop_region.shift_by(delta);
                    track.sync_active_stored_loop_slot();
                }
                self.silence_tracks_for_loop_change();
                AppControl::Continue
            }
            AppAction::NudgeGlobalLoopBackward => {
                let delta = -(self.nudge_step_ticks() as i64);
                self.project.loop_region.shift_by(delta);
                self.silence_tracks_for_loop_change();
                AppControl::Continue
            }
            AppAction::NudgeGlobalLoopForward => {
                let delta = self.nudge_step_ticks() as i64;
                self.project.loop_region.shift_by(delta);
                self.silence_tracks_for_loop_change();
                AppControl::Continue
            }
            AppAction::ShortenCurrentTrackLoop => {
                let step = self.nudge_step_ticks();
                if let Some(track) = self.project.active_track_mut() {
                    track.loop_region.shorten_by(step);
                    track.sync_active_stored_loop_slot();
                }
                self.silence_tracks_for_loop_change();
                AppControl::Continue
            }
            AppAction::ExtendCurrentTrackLoop => {
                let step = self.nudge_step_ticks();
                if let Some(track) = self.project.active_track_mut() {
                    track.loop_region.extend_by(step);
                    track.sync_active_stored_loop_slot();
                }
                self.silence_tracks_for_loop_change();
                AppControl::Continue
            }
            AppAction::HalfCurrentTrackLoop => {
                if let Some(track) = self.project.active_track_mut() {
                    track.loop_region.half_length();
                    track.sync_active_stored_loop_slot();
                }
                self.silence_tracks_for_loop_change();
                AppControl::Continue
            }
            AppAction::DoubleCurrentTrackLoop => {
                if let Some(track) = self.project.active_track_mut() {
                    track.loop_region.double_length();
                    track.sync_active_stored_loop_slot();
                }
                self.silence_tracks_for_loop_change();
                AppControl::Continue
            }
            AppAction::ShortenGlobalLoop => {
                let step = self.nudge_step_ticks();
                self.project.loop_region.shorten_by(step);
                self.silence_tracks_for_loop_change();
                AppControl::Continue
            }
            AppAction::ExtendGlobalLoop => {
                let step = self.nudge_step_ticks();
                self.project.loop_region.extend_by(step);
                self.silence_tracks_for_loop_change();
                AppControl::Continue
            }
            AppAction::HalfGlobalLoop => {
                self.project.loop_region.half_length();
                self.silence_tracks_for_loop_change();
                AppControl::Continue
            }
            AppAction::DoubleGlobalLoop => {
                self.project.loop_region.double_length();
                self.silence_tracks_for_loop_change();
                AppControl::Continue
            }
            AppAction::ToggleCurrentTrackArm => {
                if let Some(track) = self.project.active_track_mut() {
                    track.state.armed = !track.state.armed;
                }
                AppControl::Continue
            }
            AppAction::ToggleCurrentTrackMute => {
                if let Some(track) = self.project.active_track_mut() {
                    track.state.muted = !track.state.muted;
                    if track.state.muted {
                        self.silence_all_tracks();
                    }
                }
                AppControl::Continue
            }
            AppAction::ToggleCurrentTrackSolo => {
                if let Some(track) = self.project.active_track_mut() {
                    track.state.soloed = !track.state.soloed;
                }
                AppControl::Continue
            }
            AppAction::ToggleCurrentTrackPassthrough => {
                if let Some(track) = self.project.active_track_mut() {
                    track.state.passthrough = !track.state.passthrough;
                }
                AppControl::Continue
            }
            AppAction::ToggleCurrentTrackRecordingView => {
                if let Some(track) = self.project.active_track_mut() {
                    track.toggle_recording_view();
                }
                self.sync_active_track_recording_clip_scroll();
                AppControl::Continue
            }
            AppAction::SelectRecordingClip(clip_id) => {
                if let Some(track) = self.project.active_track_mut() {
                    track.select_recording_clip(clip_id);
                }
                self.sync_active_track_recording_clip_scroll();
                AppControl::Continue
            }
            AppAction::SelectPreviousRecordingClip => {
                if let Some(track) = self.project.active_track_mut() {
                    track.select_previous_recording_clip();
                }
                self.sync_active_track_recording_clip_scroll();
                AppControl::Continue
            }
            AppAction::SelectNextRecordingClip => {
                if let Some(track) = self.project.active_track_mut() {
                    track.select_next_recording_clip();
                }
                self.sync_active_track_recording_clip_scroll();
                AppControl::Continue
            }
            AppAction::ToggleSelectedRecordingClipMute => {
                if self.page_state.current_page == AppPage::Timeline
                    && self
                        .page_state
                        .selected_timeline_context
                        .chain_kind()
                        .is_some()
                {
                    self.toggle_selected_timeline_fx_enabled();
                } else if let Some(track) = self.project.active_track_mut() {
                    track.toggle_selected_recording_clip_mute();
                }
                AppControl::Continue
            }
            AppAction::DeletePageItem => {
                match self.page_state.current_page {
                    AppPage::Timeline
                        if self
                            .page_state
                            .selected_timeline_context
                            .chain_kind()
                            .is_some() =>
                    {
                        self.delete_selected_timeline_fx();
                    }
                    AppPage::Mappings => {
                        self.remove_selected_mapping();
                    }
                    _ => {}
                }
                AppControl::Continue
            }
            AppAction::DeleteSelectedRecordingClip => {
                if let Some(track) = self.project.active_track_mut() {
                    track.delete_selected_recording_clip();
                }
                self.sync_active_track_recording_clip_scroll();
                AppControl::Continue
            }
            AppAction::OpenSelectedRecordingClipAlign => {
                self.open_selected_recording_clip_align();
                AppControl::Continue
            }
            AppAction::CloseRecordingClipAlign => {
                self.close_clip_align();
                AppControl::Continue
            }
            AppAction::ApplyRecordingClipAlign => {
                self.apply_clip_align();
                AppControl::Continue
            }
            AppAction::SelectPreviousClipAlignField => {
                if let Some(session) = self.clip_align_session.as_mut() {
                    session.selected_field = session.selected_field.previous();
                }
                AppControl::Continue
            }
            AppAction::SelectNextClipAlignField => {
                if let Some(session) = self.clip_align_session.as_mut() {
                    session.selected_field = session.selected_field.next();
                }
                AppControl::Continue
            }
            AppAction::AdjustClipAlignFieldBackward => {
                self.adjust_clip_align_field(-1);
                AppControl::Continue
            }
            AppAction::AdjustClipAlignFieldForward => {
                self.adjust_clip_align_field(1);
                AppControl::Continue
            }
            AppAction::ToggleSelectedTimelineFx => {
                self.toggle_selected_timeline_fx_enabled();
                AppControl::Continue
            }
            AppAction::CycleSelectedTimelineFxKind => {
                self.adjust_selected_timeline_fx_kind(1);
                AppControl::Continue
            }
            AppAction::AdjustSelectedTimelineFxPrimary => {
                self.adjust_selected_timeline_fx_parameter(0, 1);
                AppControl::Continue
            }
            AppAction::AdjustSelectedTimelineFxSecondary => {
                self.adjust_selected_timeline_fx_parameter(1, 1);
                AppControl::Continue
            }
            AppAction::ScrollSelectedTimelineFxWindow => {
                self.scroll_selected_timeline_fx_parameter_window(1);
                AppControl::Continue
            }
            AppAction::MoveSelectedTimelineFxUp => {
                self.move_selected_timeline_fx(-1);
                AppControl::Continue
            }
            AppAction::MoveSelectedTimelineFxDown => {
                self.move_selected_timeline_fx(1);
                AppControl::Continue
            }
            AppAction::AddSelectedTimelineFx => {
                self.add_selected_timeline_fx();
                AppControl::Continue
            }
            AppAction::DeleteSelectedTimelineFx => {
                self.delete_selected_timeline_fx();
                AppControl::Continue
            }
            AppAction::ToggleFocusedTrackView => {
                self.focused_track_view = !self.focused_track_view;
                self.sync_active_track_recording_clip_scroll();
                AppControl::Continue
            }
            AppAction::SelectNextTrack => {
                self.project.select_next_track();
                self.sync_active_track_recording_clip_scroll();
                AppControl::Continue
            }
            AppAction::SelectPreviousTrack => {
                self.project.select_previous_track();
                self.sync_active_track_recording_clip_scroll();
                AppControl::Continue
            }
            AppAction::SelectTrack(index) => {
                self.project.select_track(index);
                self.sync_active_track_recording_clip_scroll();
                AppControl::Continue
            }
            AppAction::SelectNotesAtPlayhead => {
                let playhead_ticks = self.active_track_note_playhead_ticks();
                let additive = self.note_additive_select_held;
                if let Some(track) = self.project.active_track_mut() {
                    track.select_notes_at_playhead(playhead_ticks, additive);
                }
                AppControl::Continue
            }
            AppAction::SelectNotesAtPlayheadAdd => {
                let playhead_ticks = self.active_track_note_playhead_ticks();
                if let Some(track) = self.project.active_track_mut() {
                    track.select_notes_at_playhead(playhead_ticks, true);
                }
                AppControl::Continue
            }
            AppAction::DeselectTrackNotes => {
                if let Some(track) = self.project.active_track_mut() {
                    track.clear_note_selection();
                }
                AppControl::Continue
            }
            AppAction::SelectNextNote => {
                let playhead_ticks = self.active_track_note_playhead_ticks();
                let additive = self.note_additive_select_held;
                if let Some(track) = self.project.active_track_mut() {
                    track.select_next_note(playhead_ticks, additive);
                }
                AppControl::Continue
            }
            AppAction::SelectPreviousNote => {
                let playhead_ticks = self.active_track_note_playhead_ticks();
                let additive = self.note_additive_select_held;
                if let Some(track) = self.project.active_track_mut() {
                    track.select_previous_note(playhead_ticks, additive);
                }
                AppControl::Continue
            }
            AppAction::FocusFirstSelectedNote => {
                if let Some(track) = self.project.active_track_mut() {
                    track.focus_first_selected_note();
                }
                AppControl::Continue
            }
            AppAction::FocusLastSelectedNote => {
                if let Some(track) = self.project.active_track_mut() {
                    track.focus_last_selected_note();
                }
                AppControl::Continue
            }
            AppAction::ExtendNoteSelectionForward => {
                let playhead_ticks = self.active_track_note_playhead_ticks();
                if let Some(track) = self.project.active_track_mut() {
                    track.extend_note_selection_forward(playhead_ticks);
                }
                AppControl::Continue
            }
            AppAction::ExtendNoteSelectionBackward => {
                let playhead_ticks = self.active_track_note_playhead_ticks();
                if let Some(track) = self.project.active_track_mut() {
                    track.extend_note_selection_backward(playhead_ticks);
                }
                AppControl::Continue
            }
            AppAction::ExtendNoteSelectionBoth => {
                let playhead_ticks = self.active_track_note_playhead_ticks();
                if let Some(track) = self.project.active_track_mut() {
                    track.extend_note_selection_both(playhead_ticks);
                }
                AppControl::Continue
            }
            AppAction::ContractNoteSelection => {
                if let Some(track) = self.project.active_track_mut() {
                    track.contract_note_selection();
                }
                AppControl::Continue
            }
            AppAction::NudgeSelectedNotesEarlier => {
                let delta = -(self.note_time_nudge_step_ticks() as i64);
                if let Some(track) = self.project.active_track_mut() {
                    track.nudge_selected_notes_time(delta);
                }
                AppControl::Continue
            }
            AppAction::NudgeSelectedNotesLater => {
                let delta = self.note_time_nudge_step_ticks() as i64;
                if let Some(track) = self.project.active_track_mut() {
                    track.nudge_selected_notes_time(delta);
                }
                AppControl::Continue
            }
            AppAction::NudgeSelectedNotesUp => {
                if let Some(track) = self.project.active_track_mut() {
                    track.nudge_selected_notes_pitch(1);
                }
                AppControl::Continue
            }
            AppAction::NudgeSelectedNotesDown => {
                if let Some(track) = self.project.active_track_mut() {
                    track.nudge_selected_notes_pitch(-1);
                }
                AppControl::Continue
            }
            AppAction::BeginNoteAdditiveSelectionHold => {
                self.note_additive_select_held = true;
                AppControl::Continue
            }
            AppAction::EndNoteAdditiveSelectionHold => {
                self.note_additive_select_held = false;
                AppControl::Continue
            }
            AppAction::SetTimelineFlow(flow) => {
                self.timeline_flow = flow;
                AppControl::Continue
            }
        }
    }

    fn advance_playhead(&mut self, delta: Duration) {
        if self.midi_runtime.is_enabled() {
            self.advance_runtime_clock(delta);
            return;
        }

        if self.project.transport.link_enabled {
            self.advance_linked_playhead(delta);
            return;
        }

        if !self.project.transport.playing {
            self.advance_stopped_live_fx(delta, None);
            return;
        }

        let previous_ticks = self.transport_ticks;
        let ticks_per_second = self.project.transport.ticks_per_second();
        let advanced_ticks =
            (delta.as_nanos() as u128 * u128::from(ticks_per_second)) / 1_000_000_000_u128;
        self.transport_ticks = self.transport_ticks.saturating_add(advanced_ticks as u64);
        self.playhead_ticks = self.song_playhead_for_transport(self.transport_ticks);

        self.process_queued_stored_loop_recalls(previous_ticks, self.transport_ticks);
        self.dispatch_midi_notes(previous_ticks, advanced_ticks as u64);
        self.dispatch_live_arp_events(previous_ticks, self.transport_ticks);
        self.live_fx_ticks = self.transport_ticks;
    }

    fn advance_linked_playhead(&mut self, delta: Duration) {
        self.link_snapshot = self.link.refresh();
        self.project.transport.tempo_bpm =
            self.link_snapshot.tempo_bpm.round().clamp(20.0, 400.0) as u16;
        if self.project.transport.link_start_stop_sync {
            self.project.transport.playing = self.link_snapshot.is_playing;
        }
        if !self.project.transport.playing {
            self.advance_stopped_live_fx(delta, Some(self.link_snapshot.tempo_bpm));
            return;
        }

        let previous_ticks = self.transport_ticks;
        let linked_ticks = (self.link_snapshot.beat.max(0.0)
            * f64::from(self.project.transport.ppqn.max(1)))
        .round() as u64;
        self.transport_ticks = linked_ticks;
        self.playhead_ticks = self.song_playhead_for_transport(self.transport_ticks);

        if linked_ticks < previous_ticks {
            self.silence_all_tracks();
            return;
        }
        self.process_queued_stored_loop_recalls(previous_ticks, linked_ticks);
        self.dispatch_midi_notes(previous_ticks, linked_ticks.saturating_sub(previous_ticks));
        self.dispatch_live_arp_events(previous_ticks, linked_ticks);
        self.live_fx_ticks = self.transport_ticks;
    }

    fn advance_stopped_live_fx(&mut self, delta: Duration, tempo_override_bpm: Option<f64>) {
        let ticks_per_second = ticks_per_second_for_tempo(
            tempo_override_bpm.unwrap_or(f64::from(self.project.transport.tempo_bpm)),
            self.project.transport.ppqn,
        );
        if ticks_per_second == 0 {
            return;
        }
        let advanced_ticks =
            (delta.as_nanos() as u128 * u128::from(ticks_per_second)) / 1_000_000_000_u128;
        if advanced_ticks == 0 {
            return;
        }
        let previous_ticks = self.live_fx_ticks;
        self.live_fx_ticks = self.live_fx_ticks.saturating_add(advanced_ticks as u64);
        self.dispatch_live_arp_events(previous_ticks, self.live_fx_ticks);
    }

    fn advance_runtime_clock(&mut self, _delta: Duration) {
        if self.project.transport.link_enabled {
            let previous_tempo = self.project.transport.tempo_bpm;
            let previous_playing = self.project.transport.playing;
            self.link_snapshot = self.link.refresh();
            self.project.transport.tempo_bpm =
                self.link_snapshot.tempo_bpm.round().clamp(20.0, 400.0) as u16;
            if self.project.transport.link_start_stop_sync {
                self.project.transport.playing = self.link_snapshot.is_playing;
            }
            let linked_ticks = (self.link_snapshot.beat.max(0.0)
                * f64::from(self.project.transport.ppqn.max(1)))
            .round() as u64;
            self.transport_ticks = linked_ticks;
            self.playhead_ticks = self.song_playhead_for_transport(linked_ticks);
            self.live_fx_ticks = linked_ticks;
            if previous_tempo != self.project.transport.tempo_bpm
                || previous_playing != self.project.transport.playing
            {
                self.mark_midi_runtime_dirty();
            }
        }
        self.sync_midi_runtime_state_if_needed();
        self.update_timing_from_runtime();
        self.live_fx_ticks = self.transport_ticks.max(self.live_fx_ticks);
    }

    fn set_transport_tempo(&mut self, bpm: u16) {
        let bpm = bpm.clamp(20, 400);
        self.project.transport.tempo_bpm = bpm;
        self.last_tempo_tap_at = None;
        self.link.commit_tempo(f64::from(bpm));
        self.link_snapshot = self.link.refresh();
    }

    fn tap_transport_tempo(&mut self) {
        let now = Instant::now();
        if let Some(previous_tap) = self.last_tempo_tap_at {
            let interval_ms = now.saturating_duration_since(previous_tap).as_millis() as u64;
            if (150..=3_000).contains(&interval_ms) {
                let bpm = (60_000 / interval_ms).clamp(20, 400) as u16;
                self.project.transport.tempo_bpm = bpm;
                self.link.commit_tempo(f64::from(bpm));
                self.link_snapshot = self.link.refresh();
            }
        }
        self.last_tempo_tap_at = Some(now);
    }

    fn process_queued_stored_loop_recalls(
        &mut self,
        previous_transport_ticks: u64,
        current_transport_ticks: u64,
    ) {
        if current_transport_ticks <= previous_transport_ticks {
            return;
        }
        let ppqn = self.project.transport.ppqn;
        for track in self.project.tracks.iter_mut() {
            if track.active_take.is_some() {
                continue;
            }
            let _ = track.resolve_queued_stored_loop_recall_if_due(
                previous_transport_ticks,
                current_transport_ticks,
                ppqn,
            );
        }
    }

    fn current_edit_ticks(&self) -> u64 {
        self.project
            .transport
            .quantize_to_nearest(self.playhead_ticks)
    }

    fn nudge_step_ticks(&self) -> u64 {
        self.project
            .transport
            .quantize_step_ticks()
            .unwrap_or(1)
            .max(1)
    }

    fn note_time_nudge_step_ticks(&self) -> u64 {
        self.project
            .transport
            .quantize_step_ticks()
            .unwrap_or((u64::from(self.project.transport.ppqn) / 8).max(1))
            .max(1)
    }

    fn active_track_note_playhead_ticks(&self) -> u64 {
        self.project
            .active_track()
            .map(|track| self.effective_track_playhead(track))
            .unwrap_or(self.playhead_ticks)
    }

    fn song_playhead_for_transport(&self, transport_ticks: u64) -> u64 {
        if !self.project.transport.loop_enabled || self.project.loop_region.length_ticks == 0 {
            return transport_ticks;
        }

        let loop_region = self.project.loop_region;
        let relative = transport_ticks.saturating_sub(loop_region.start_ticks);
        loop_region.start_ticks + (relative % loop_region.length_ticks.max(1))
    }

    fn effective_track_playhead(&self, track: &Track) -> u64 {
        if !track.state.loop_enabled || track.loop_region.length_ticks == 0 {
            return self.playhead_ticks;
        }

        track.loop_region.start_ticks + (self.transport_ticks % track.loop_region.length_ticks)
    }

    fn seed_demo_routing(&mut self) {
        let output_count = self.midi_devices.outputs.len().max(1);
        for (index, track) in self.project.tracks.iter_mut().enumerate() {
            track.routing.input_port = TrackPortSelection::Default;
            track.routing.input_channel = if index % 2 == 0 {
                MidiChannelFilter::Omni
            } else {
                MidiChannelFilter::Channel(((index % 16) + 1) as u8)
            };
            track.routing.output_port = self
                .midi_devices
                .outputs
                .get(index % output_count)
                .cloned()
                .map(TrackPortSelection::named)
                .unwrap_or_default();
            track.routing.output_channel = Some(((index % 16) + 1) as u8);
            if index == 0 {
                track.midi_fx.input_fx[0] = Some(MidiFxSlot {
                    enabled: true,
                    effect: MidiFx::Transpose { semitones: 12 },
                });
                track.midi_fx.output_fx[0] = Some(MidiFxSlot {
                    enabled: true,
                    effect: MidiFx::Velocity { percent: 120 },
                });
            } else if index == 1 {
                track.midi_fx.output_fx[0] = Some(MidiFxSlot {
                    enabled: true,
                    effect: MidiFx::Arp {
                        step_ticks: 240,
                        order: crate::midi_fx::ArpOrder::Up,
                        gate_percent: 100,
                    },
                });
            } else if index == 2 {
                track.midi_fx.output_fx[0] = Some(MidiFxSlot {
                    enabled: true,
                    effect: MidiFx::Delay { ticks: 240 },
                });
            }
        }
        self.sync_midi_inputs();
    }

    fn select_previous_timeline_context(&mut self) {
        self.page_state.selected_timeline_context =
            self.page_state.selected_timeline_context.previous();
        self.normalize_timeline_fx_selection();
    }

    fn select_next_timeline_context(&mut self) {
        self.page_state.selected_timeline_context =
            self.page_state.selected_timeline_context.next();
        self.normalize_timeline_fx_selection();
    }

    fn maybe_refresh_midi_devices(&mut self, now: Instant) {
        self.refresh_midi_devices(false, now);
    }

    fn refresh_midi_devices_now(&mut self) {
        self.refresh_midi_devices(true, Instant::now());
    }

    fn refresh_midi_devices(&mut self, force: bool, now: Instant) {
        if !force
            && now.saturating_duration_since(self.last_midi_refresh_at) < MIDI_REFRESH_INTERVAL
        {
            return;
        }
        self.last_midi_refresh_at = now;

        let previous_catalog = self.midi_devices.clone();
        let scanned = MidiDeviceCatalog::scan_live();
        let inputs = scanned.inputs;
        let outputs = scanned.outputs;
        let mut next = MidiDeviceCatalog {
            selected_input: resolve_port_by_name(
                &inputs,
                self.preferred_default_input_name.as_deref(),
            ),
            selected_output: resolve_port_by_name(
                &outputs,
                self.preferred_default_output_name.as_deref(),
            ),
            inputs,
            outputs,
        };

        if self.preferred_default_input_name.is_none() {
            self.preferred_default_input_name =
                next.selected_input_port().map(|port| port.name.clone());
        }
        if self.preferred_default_output_name.is_none() {
            self.preferred_default_output_name =
                next.selected_output_port().map(|port| port.name.clone());
        }

        next.selected_input =
            resolve_port_by_name(&next.inputs, self.preferred_default_input_name.as_deref());
        next.selected_output =
            resolve_port_by_name(&next.outputs, self.preferred_default_output_name.as_deref());

        if next == previous_catalog {
            return;
        }

        self.midi_devices = next;
        self.page_state.midi_io.selected_input_index = clamp_index(
            self.page_state.midi_io.selected_input_index,
            self.midi_devices.inputs.len(),
        );
        self.page_state.midi_io.selected_output_index = clamp_index(
            self.page_state.midi_io.selected_output_index,
            self.midi_devices.outputs.len(),
        );
        self.sync_midi_inputs();
        self.midi_runtime_dirty = true;
    }

    fn build_midi_runtime_state_sync(&self) -> MidiRuntimeStateSync {
        MidiRuntimeStateSync {
            project: self.project.clone(),
            transport_ticks: self.transport_ticks,
            playhead_ticks: self.playhead_ticks,
            live_fx_ticks: self.live_fx_ticks,
            default_input_port: self.default_input_port().cloned(),
            default_output_port: self.default_output_port().cloned(),
        }
    }

    fn sync_midi_runtime_state_if_needed(&mut self) {
        if !self.midi_runtime.is_enabled() || !self.midi_runtime_dirty {
            return;
        }
        let started_at = Instant::now();
        self.midi_runtime
            .sync_state(self.build_midi_runtime_state_sync());
        let elapsed_ns = started_at.elapsed().as_nanos().min(u128::from(u64::MAX)) as u64;
        self.midi_runtime_sync_count = self.midi_runtime_sync_count.saturating_add(1);
        self.midi_runtime_sync_total_ns =
            self.midi_runtime_sync_total_ns.saturating_add(elapsed_ns);
        self.midi_runtime_sync_max_ns = self.midi_runtime_sync_max_ns.max(elapsed_ns);
        self.midi_runtime_dirty = false;
    }

    fn mark_midi_runtime_dirty(&mut self) {
        self.midi_runtime_dirty = true;
    }

    fn update_timing_from_runtime(&mut self) {
        if !self.midi_runtime.is_enabled() {
            return;
        }
        let previous_transport_ticks = self.transport_ticks;
        let snapshot = self.midi_runtime.snapshot();
        self.apply_runtime_snapshot(snapshot);
        self.process_queued_stored_loop_recalls(previous_transport_ticks, self.transport_ticks);
        self.maybe_print_midi_runtime_app_summary();
    }

    fn maybe_print_midi_runtime_app_summary(&mut self) {
        if !self.midi_runtime_diag_enabled
            || self.last_midi_runtime_diag_at.elapsed() < MIDI_RUNTIME_APP_DIAG_INTERVAL
        {
            return;
        }
        self.last_midi_runtime_diag_at = Instant::now();
        let avg_sync_ms = if self.midi_runtime_sync_count == 0 {
            0.0
        } else {
            (self.midi_runtime_sync_total_ns / self.midi_runtime_sync_count) as f64 / 1_000_000.0
        };
        let max_sync_ms = self.midi_runtime_sync_max_ns as f64 / 1_000_000.0;
        eprintln!(
            "trekr midi app sync: syncs={} skipped_actions={} avg_sync_ms={:.3} max_sync_ms={:.3} recording={} playing={}",
            self.midi_runtime_sync_count,
            self.midi_runtime_sync_skipped_count,
            avg_sync_ms,
            max_sync_ms,
            self.project.transport.recording,
            self.project.transport.playing
        );
    }

    fn apply_runtime_snapshot(&mut self, snapshot: MidiRuntimeUiSnapshot) {
        self.transport_ticks = snapshot.transport_ticks;
        self.playhead_ticks = snapshot.playhead_ticks;
        self.live_fx_ticks = snapshot.live_fx_ticks;
        self.merge_runtime_recording_takes(&snapshot);
        self.last_runtime_snapshot = snapshot;
    }

    fn merge_runtime_recording_takes(&mut self, snapshot: &MidiRuntimeUiSnapshot) {
        for (track, active_take) in self
            .project
            .tracks
            .iter_mut()
            .zip(snapshot.recording_takes.iter())
        {
            track.active_take = active_take.clone();
        }
    }

    fn input_port_is_available(&self, name: &str) -> bool {
        self.midi_devices
            .inputs
            .iter()
            .any(|port| port.name == name)
    }

    fn output_port_is_available(&self, name: &str) -> bool {
        self.midi_devices
            .outputs
            .iter()
            .any(|port| port.name == name)
    }

    fn set_preferred_default_input_from_index(&mut self, index: usize) {
        let Some(port) = self.midi_devices.input(index) else {
            return;
        };
        self.preferred_default_input_name = Some(port.name.clone());
        self.midi_devices.set_selected_input(index);
        self.sync_midi_inputs();
        self.midi_runtime_dirty = true;
    }

    fn set_preferred_default_output_from_index(&mut self, index: usize) {
        let Some(port) = self.midi_devices.output(index) else {
            return;
        };
        self.preferred_default_output_name = Some(port.name.clone());
        self.midi_devices.set_selected_output(index);
        self.midi_runtime_dirty = true;
    }

    pub(super) fn default_input_port(&self) -> Option<&MidiPortRef> {
        self.midi_devices.selected_input_port()
    }

    pub(super) fn default_output_port(&self) -> Option<&MidiPortRef> {
        self.midi_devices.selected_output_port()
    }

    pub(super) fn resolved_input_port<'a>(
        &'a self,
        selection: &'a TrackPortSelection,
    ) -> Option<&'a MidiPortRef> {
        selection.resolve(self.default_input_port())
    }

    pub(super) fn routing_input_selection_label(&self, selection: &TrackPortSelection) -> String {
        self.routing_selection_label(selection, self.default_input_port(), |name| {
            self.input_port_is_available(name)
        })
    }

    pub(super) fn routing_output_selection_label(&self, selection: &TrackPortSelection) -> String {
        self.routing_selection_label(selection, self.default_output_port(), |name| {
            self.output_port_is_available(name)
        })
    }

    fn routing_selection_label(
        &self,
        selection: &TrackPortSelection,
        default_port: Option<&MidiPortRef>,
        is_available: impl Fn(&str) -> bool,
    ) -> String {
        match selection {
            TrackPortSelection::None => "None".to_string(),
            TrackPortSelection::Default => default_port
                .map(|port| {
                    if is_available(&port.name) {
                        format!("Default ({})", port.name)
                    } else {
                        format!("Default ({} offline)", port.name)
                    }
                })
                .unwrap_or_else(|| "Default (offline)".to_string()),
            TrackPortSelection::Port(port) => {
                if is_available(&port.name) {
                    port.name.clone()
                } else {
                    format!("{} (offline)", port.name)
                }
            }
        }
    }

    fn sync_midi_inputs(&mut self) {
        let mut ports = Vec::new();
        for track in &self.project.tracks {
            if let Some(port) = track
                .routing
                .input_port
                .cloned_resolved(self.default_input_port())
            {
                if !ports.iter().any(|existing: &MidiPortRef| existing == &port) {
                    ports.push(port);
                }
            }
        }
        if self.page_state.current_page == AppPage::Mappings
            || self.page_state.current_page == AppPage::MidiIo
            || self.overlay_state.active == Some(AppOverlay::MappingsQuickView)
            || self.page_state.mapping_midi_learn_armed
            || self.direct_mapping_state.mode != DirectMappingMode::Inactive
        {
            for port in &self.midi_devices.inputs {
                if !ports.iter().any(|existing: &MidiPortRef| existing == port) {
                    ports.push(port.clone());
                }
            }
        }
        self.midi_input.sync_ports(&ports);
        self.midi_runtime_dirty = true;
    }

    fn ensure_fx_live_state_len(&mut self) {
        let track_count = self.project.tracks.len();
        if self.input_fx_live_states.len() < track_count {
            self.input_fx_live_states
                .resize(track_count, LiveMidiFxState::default());
        }
        if self.output_fx_live_states.len() < track_count {
            self.output_fx_live_states
                .resize(track_count, LiveMidiFxState::default());
        }
    }

    fn track_emits_clone_source(&self, source_track_index: usize) -> bool {
        let Some(source) = self.project.tracks.get(source_track_index) else {
            return false;
        };
        if source.state.muted {
            return false;
        }
        let any_solo = self.project.tracks.iter().any(|track| track.state.soloed);
        !any_solo || source.state.soloed
    }

    fn process_track_clone_live_events(
        track: &Track,
        source_track_index: usize,
        source_events: &[LiveMidiFxEvent],
        state: &mut LiveMidiFxState,
        current_ticks: u64,
        global_quantize_root: u8,
    ) -> Vec<LiveMidiFxEvent> {
        let mut events = Vec::new();
        for slot in track
            .midi_fx
            .input_fx
            .iter()
            .flatten()
            .filter(|slot| slot.enabled)
        {
            match slot.effect {
                MidiFx::TrackClone { source_track } if source_track == source_track_index => {
                    events.extend(source_events.iter().cloned());
                }
                MidiFx::TrackClone { .. } => {}
                _ => {
                    let mut transformed = Vec::new();
                    for event in events {
                        transformed.extend(process_live_chain_event(
                            &[Some(slot.clone())],
                            state,
                            event,
                            current_ticks,
                            global_quantize_root,
                        ));
                    }
                    events = transformed;
                    if events.is_empty() {
                        break;
                    }
                }
            }
        }
        events
    }

    fn propagate_live_clone_events(
        &mut self,
        source_track_index: usize,
        source_events: &[LiveMidiFxEvent],
        emit_live_output: bool,
    ) {
        if source_events.is_empty() || !self.track_emits_clone_source(source_track_index) {
            return;
        }

        #[derive(Clone)]
        struct CloneTarget {
            target_index: usize,
            record_mode: crate::midi_fx::RecordInputFxMode,
            monitor_input_fx: bool,
            output_port: Option<MidiPortRef>,
            output_channel: Option<u8>,
            output_chain: Vec<Option<MidiFxSlot>>,
        }

        let mut targets = Vec::new();
        for (target_index, track) in self.project.tracks.iter().enumerate() {
            if target_index == source_track_index {
                continue;
            }
            let clone_matches = track
                .midi_fx
                .input_fx
                .iter()
                .flatten()
                .filter(|slot| slot.enabled)
                .filter(|slot| matches!(slot.effect, MidiFx::TrackClone { source_track } if source_track == source_track_index))
                .count();
            if clone_matches == 0 {
                continue;
            }
            let base = CloneTarget {
                target_index,
                record_mode: track.midi_fx.record_input_fx_mode,
                monitor_input_fx: track.midi_fx.monitor_input_fx,
                output_port: track
                    .routing
                    .output_port
                    .cloned_resolved(self.default_output_port()),
                output_channel: track.routing.output_channel,
                output_chain: track.midi_fx.output_fx.clone(),
            };
            for _ in 0..clone_matches {
                targets.push(base.clone());
            }
        }

        self.ensure_fx_live_state_len();
        for target in targets {
            let input_ticks = self
                .project
                .tracks
                .get(target.target_index)
                .map(|track| self.record_capture_ticks(track))
                .unwrap_or(self.playhead_ticks);

            let post_input_events = if let (Some(state), Some(track)) = (
                self.input_fx_live_states.get_mut(target.target_index),
                self.project.tracks.get(target.target_index),
            ) {
                Self::process_track_clone_live_events(
                    track,
                    source_track_index,
                    source_events,
                    state,
                    input_ticks,
                    self.project.global_harmony.root,
                )
            } else {
                Vec::new()
            };
            if post_input_events.is_empty() {
                continue;
            }

            if let Some(track) = self.project.tracks.get_mut(target.target_index) {
                if track.active_take.is_some()
                    && target.record_mode == crate::midi_fx::RecordInputFxMode::PostInputFx
                {
                    let mut recorded = false;
                    for record_event in &post_input_events {
                        match *record_event {
                            LiveMidiFxEvent::NoteOn { pitch, velocity } => {
                                track.record_note_on(pitch, velocity, input_ticks);
                                recorded = true;
                            }
                            LiveMidiFxEvent::NoteOff { pitch } => {
                                track.record_note_off(pitch, input_ticks);
                                recorded = true;
                            }
                        }
                    }
                    if recorded {
                        self.mark_midi_runtime_dirty();
                    }
                }
            }

            if emit_live_output && target.monitor_input_fx {
                self.send_live_monitor_events(
                    target.target_index,
                    &target.output_chain,
                    target.output_port.as_ref(),
                    target.output_channel,
                    post_input_events,
                    input_ticks,
                );
            }
        }
    }

    fn monitor_source_events(
        &mut self,
        track_index: usize,
        event: LiveMidiFxEvent,
        input_chain: &[Option<MidiFxSlot>],
        monitor_input_fx: bool,
        current_ticks: u64,
    ) -> (Vec<LiveMidiFxEvent>, Vec<LiveMidiFxEvent>) {
        self.ensure_fx_live_state_len();
        let processed = if let Some(state) = self.input_fx_live_states.get_mut(track_index) {
            process_live_chain_event(
                input_chain,
                state,
                event.clone(),
                current_ticks,
                self.project.global_harmony.root,
            )
        } else {
            vec![event.clone()]
        };
        let monitor_source = if monitor_input_fx {
            processed.clone()
        } else {
            vec![event]
        };
        (processed, monitor_source)
    }

    fn send_live_monitor_events(
        &mut self,
        track_index: usize,
        output_chain: &[Option<MidiFxSlot>],
        output_port: Option<&MidiPortRef>,
        output_channel: Option<u8>,
        events: Vec<LiveMidiFxEvent>,
        current_ticks: u64,
    ) {
        let (Some(port), Some(channel)) = (output_port, output_channel) else {
            return;
        };
        self.ensure_fx_live_state_len();
        for event in events {
            let processed_events =
                if let Some(state) = self.output_fx_live_states.get_mut(track_index) {
                    process_live_chain_event(
                        output_chain,
                        state,
                        event,
                        current_ticks,
                        self.project.global_harmony.root,
                    )
                } else {
                    Vec::new()
                };
            for processed in processed_events {
                match processed {
                    LiveMidiFxEvent::NoteOn { pitch, velocity } => {
                        let _ = self.midi_output.send_note_on(
                            port,
                            channel.clamp(1, 16),
                            pitch,
                            velocity,
                        );
                    }
                    LiveMidiFxEvent::NoteOff { pitch } => {
                        let _ = self
                            .midi_output
                            .send_note_off(port, channel.clamp(1, 16), pitch);
                    }
                }
            }
        }
    }

    fn playback_loop_range_for_track(&self, track: &Track) -> Option<crate::timeline::LoopRegion> {
        if track.state.loop_enabled {
            Some(track.loop_region)
        } else {
            self.project
                .transport
                .loop_enabled
                .then_some(self.project.loop_region)
        }
    }

    fn dispatch_live_arp_events(&mut self, previous_ticks: u64, current_ticks: u64) {
        if current_ticks <= previous_ticks {
            return;
        }
        self.ensure_fx_live_state_len();
        for track_index in 0..self.project.tracks.len() {
            let Some(track_view) = self.project.tracks.get(track_index) else {
                continue;
            };
            let input_chain = track_view.midi_fx.input_fx.clone();
            let output_chain = track_view.midi_fx.output_fx.clone();
            let record_mode = track_view.midi_fx.record_input_fx_mode;
            let monitor_input_fx = track_view.midi_fx.monitor_input_fx;
            let passthrough = track_view.state.passthrough;
            let output_port = track_view
                .routing
                .output_port
                .cloned_resolved(self.default_output_port());
            let output_channel = track_view.routing.output_channel;

            let input_events = if let Some(state) = self.input_fx_live_states.get_mut(track_index) {
                process_live_chain_tick(
                    &input_chain,
                    state,
                    previous_ticks,
                    current_ticks,
                    self.project.global_harmony.root,
                )
            } else {
                Vec::new()
            };
            if let Some(track) = self.project.tracks.get_mut(track_index) {
                if track.active_take.is_some()
                    && record_mode == crate::midi_fx::RecordInputFxMode::PostInputFx
                {
                    let mut recorded = false;
                    for (tick, event) in &input_events {
                        match *event {
                            LiveMidiFxEvent::NoteOn { pitch, velocity } => {
                                track.record_note_on(pitch, velocity, *tick);
                                recorded = true;
                            }
                            LiveMidiFxEvent::NoteOff { pitch } => {
                                track.record_note_off(pitch, *tick);
                                recorded = true;
                            }
                        }
                    }
                    if recorded {
                        self.mark_midi_runtime_dirty();
                    }
                }
            }
            if passthrough && monitor_input_fx {
                for (tick, event) in input_events {
                    self.send_live_monitor_events(
                        track_index,
                        &output_chain,
                        output_port.as_ref(),
                        output_channel,
                        vec![event],
                        tick,
                    );
                }
            }

            let output_events = if let Some(state) = self.output_fx_live_states.get_mut(track_index)
            {
                process_live_chain_tick(
                    &output_chain,
                    state,
                    previous_ticks,
                    current_ticks,
                    self.project.global_harmony.root,
                )
            } else {
                Vec::new()
            };
            if let (Some(port), Some(channel)) = (output_port.as_ref(), output_channel) {
                for (_, event) in output_events {
                    match event {
                        LiveMidiFxEvent::NoteOn { pitch, velocity } => {
                            let _ = self.midi_output.send_note_on(
                                port,
                                channel.clamp(1, 16),
                                pitch,
                                velocity,
                            );
                        }
                        LiveMidiFxEvent::NoteOff { pitch } => {
                            let _ =
                                self.midi_output
                                    .send_note_off(port, channel.clamp(1, 16), pitch);
                        }
                    }
                }
            }
        }
    }

    fn effective_track_clone_playback_notes_recursive(
        &self,
        track_index: usize,
        previous_ticks: u64,
        advanced_ticks: u64,
        visited: &mut [bool],
    ) -> Vec<MidiNote> {
        let Some(track) = self.project.tracks.get(track_index) else {
            return Vec::new();
        };
        let mut notes = Vec::new();
        for slot in track
            .midi_fx
            .input_fx
            .iter()
            .flatten()
            .filter(|slot| slot.enabled)
        {
            let MidiFx::TrackClone { source_track } = slot.effect else {
                notes = transform_notes(
                    &notes,
                    &[Some(slot.clone())],
                    self.project.global_harmony.root,
                );
                continue;
            };
            if source_track == track_index || !self.track_emits_clone_source(source_track) {
                continue;
            }
            notes.extend(self.effective_track_pre_output_playback_notes_recursive(
                source_track,
                previous_ticks,
                advanced_ticks,
                visited,
            ));
        }
        notes
    }

    fn effective_track_pre_output_playback_notes_recursive(
        &self,
        track_index: usize,
        previous_ticks: u64,
        advanced_ticks: u64,
        visited: &mut [bool],
    ) -> Vec<MidiNote> {
        let Some(track) = self.project.tracks.get(track_index) else {
            return Vec::new();
        };
        if visited.get(track_index).copied().unwrap_or(true) {
            return Vec::new();
        }
        visited[track_index] = true;
        let native_notes = scheduled_note_occurrences(
            track,
            &track.midi_notes,
            previous_ticks,
            advanced_ticks,
            self.playback_loop_range_for_track(track),
        );
        let cloned_notes = self.effective_track_clone_playback_notes_recursive(
            track_index,
            previous_ticks,
            advanced_ticks,
            visited,
        );
        let mut notes = native_notes;
        notes.extend(cloned_notes);
        visited[track_index] = false;
        notes.sort_by_key(|note| (note.start_ticks, note.pitch, note.length_ticks));
        notes
    }

    pub(crate) fn handle_mappings_pointer(
        &mut self,
        content_bounds: Rect,
        x: i32,
        y: i32,
        source: crate::actions::ActionSource,
    ) -> Option<AppControl> {
        if let Some(layout) = self.mapping_target_lookup_layout(content_bounds) {
            if rect_contains(layout.results_panel, x, y) {
                let relative_y = y - (layout.results_panel.y + 14);
                if relative_y >= 0 {
                    let row_index = (relative_y / 12) as usize;
                    let results = self.mapping_target_lookup_results();
                    let result_index = layout.start_index + row_index;
                    if let Some(label) =
                        results.get(result_index.min(results.len().saturating_sub(1)))
                    {
                        self.commit_mapping_target_lookup_label(label);
                    }
                }
                return Some(AppControl::Continue);
            }
            if !rect_contains(layout.target_cell, x, y) {
                self.cancel_mapping_target_lookup();
                return Some(AppControl::Continue);
            }
        }

        let layout = self.mappings_page_layout(content_bounds);
        let overview_badge = layout.overview_badge;
        let learn_badge = layout.learn_badge;
        let direct_badge = layout.direct_badge;
        if rect_contains(overview_badge, x, y) {
            return Some(self.apply_action_with_source(AppAction::ToggleMappingsWriteMode, source));
        }
        if rect_contains(learn_badge, x, y)
            && self.page_state.mapping_mode == MappingPageMode::Write
            && self.page_state.selected_mapping_field == MappingField::SourceValue
        {
            self.clear_mapping_target_lookup();
            return Some(self.apply_action_with_source(AppAction::ActivatePageItem, source));
        }
        if rect_contains(direct_badge, x, y) {
            return Some(self.apply_action_with_source(AppAction::ToggleDirectMappingMode, source));
        }

        let list_bounds = layout.list_bounds;
        let (visible_rows, start_index) = self.mappings_visible_row_range(&layout);

        for visible_index in 0..visible_rows {
            let index = start_index + visible_index;
            if index >= self.mappings.len() {
                break;
            }
            let row = Rect::new(
                list_bounds.x,
                list_bounds.y + visible_index as i32 * (layout.row_height + layout.row_gap),
                list_bounds.width(),
                layout.row_height as u32,
            );
            if !rect_contains(row, x, y) {
                continue;
            }

            self.page_state.selected_mapping_index = index;
            self.normalize_selected_mapping_field();
            self.page_state.mapping_midi_learn_armed = false;
            self.clear_mapping_target_lookup();

            if self.page_state.mapping_mode != MappingPageMode::Write {
                return Some(AppControl::Continue);
            }

            let cells = self.mapping_row_cells(row);
            for field in MappingField::ALL {
                let rect = cells[mapping_field_index(field)];
                if !rect_contains(rect, x, y) || !self.mapping_field_enabled(field) {
                    continue;
                }
                let same_field = self.page_state.selected_mapping_field == field;
                self.page_state.selected_mapping_field = field;
                if same_field {
                    self.activate_mapping_field();
                }
                return Some(AppControl::Continue);
            }

            return Some(AppControl::Continue);
        }

        None
    }

    fn apply_action_with_source(
        &mut self,
        action: AppAction,
        source: crate::actions::ActionSource,
    ) -> AppControl {
        self.status_state.hovered_target = None;
        self.direct_mapping_state.status_message = None;
        if !matches!(
            action,
            AppAction::Undo
                | AppAction::Redo
                | AppAction::UndoTimeline
                | AppAction::RedoTimeline
                | AppAction::UndoMappings
                | AppAction::RedoMappings
                | AppAction::UndoUi
                | AppAction::RedoUi
        ) {
            self.status_state.history_message = None;
        }
        self.status_state.last_action = Some(LastActionStatus { action, source });
        let control = self.apply_action(action);
        if action_affects_midi_runtime(action) {
            self.mark_midi_runtime_dirty();
        } else {
            self.midi_runtime_sync_skipped_count =
                self.midi_runtime_sync_skipped_count.saturating_add(1);
        }
        control
    }

    #[cfg(test)]
    pub(crate) fn wait_for_midi_runtime(&mut self) {
        self.midi_runtime.wait_until_idle();
        self.update_timing_from_runtime();
    }

    #[cfg(test)]
    pub(crate) fn inject_midi_input_event(&mut self, event: MidiInputEvent) {
        self.midi_runtime_dirty = true;
        self.sync_midi_runtime_state_if_needed();
        self.wait_for_midi_runtime();
        let _ = self.midi_runtime.input_sender().send(event.clone());
        self.handle_midi_input_event(event);
        self.wait_for_midi_runtime();
    }

    #[cfg(test)]
    pub(crate) fn force_sync_midi_runtime(&mut self) {
        self.midi_runtime_dirty = true;
        self.sync_midi_runtime_state_if_needed();
        self.wait_for_midi_runtime();
    }
}

fn initial_window_size(video: &sdl3::VideoSubsystem, video_mode: VideoMode) -> (u32, u32) {
    if video_mode != VideoMode::KmsDrmConsole {
        return (1280, 720);
    }

    if let Some(size) = parse_kmsdrm_size_override() {
        println!("trekr kmsdrm window size override: {}x{}", size.0, size.1);
        return size;
    }

    match video
        .get_primary_display()
        .and_then(|display| display.get_mode())
    {
        Ok(mode) if mode.w > 0 && mode.h > 0 => {
            let size = (mode.w as u32, mode.h as u32);
            println!("trekr kmsdrm display mode: {}x{}", size.0, size.1);
            size
        }
        Ok(mode) => {
            eprintln!(
                "trekr kmsdrm display mode was invalid ({}x{}); falling back to 1280x720",
                mode.w, mode.h
            );
            (1280, 720)
        }
        Err(err) => {
            eprintln!(
                "trekr could not query kmsdrm display mode ({err}); falling back to 1280x720"
            );
            (1280, 720)
        }
    }
}

fn action_affects_midi_runtime(action: AppAction) -> bool {
    !matches!(
        action,
        AppAction::Quit
            | AppAction::UndoUi
            | AppAction::RedoUi
            | AppAction::ShowPage(_)
            | AppAction::ShowNextPage
            | AppAction::ShowPreviousPage
            | AppAction::SelectPreviousPageItem
            | AppAction::SelectNextPageItem
            | AppAction::CancelCurrentMode
            | AppAction::ToggleMappingsOverlay
            | AppAction::ToggleDiscoverabilityOverlay
            | AppAction::ToggleDirectMappingMode
            | AppAction::ToggleMappingsWriteMode
            | AppAction::AddMappingRow
            | AppAction::RemoveSelectedMapping
            | AppAction::SelectPreviousPageField
            | AppAction::SelectNextPageField
            | AppAction::ToggleFocusedTrackView
            | AppAction::SelectNextTrack
            | AppAction::SelectPreviousTrack
            | AppAction::SelectTrack(_)
            | AppAction::SelectNotesAtPlayhead
            | AppAction::SelectNotesAtPlayheadAdd
            | AppAction::DeselectTrackNotes
            | AppAction::SelectNextNote
            | AppAction::SelectPreviousNote
            | AppAction::FocusFirstSelectedNote
            | AppAction::FocusLastSelectedNote
            | AppAction::ExtendNoteSelectionForward
            | AppAction::ExtendNoteSelectionBackward
            | AppAction::ExtendNoteSelectionBoth
            | AppAction::ContractNoteSelection
            | AppAction::BeginNoteAdditiveSelectionHold
            | AppAction::EndNoteAdditiveSelectionHold
    )
}

fn parse_kmsdrm_size_override() -> Option<(u32, u32)> {
    std::env::var("TREKR_KMSDRM_SIZE")
        .ok()
        .and_then(|value| parse_window_size(&value))
}

fn parse_window_size(value: &str) -> Option<(u32, u32)> {
    let (width, height) = value.trim().split_once(['x', 'X'])?;
    let width = width.trim().parse::<u32>().ok()?;
    let height = height.trim().parse::<u32>().ok()?;
    if width == 0 || height == 0 {
        return None;
    }
    Some((width, height))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AppControl {
    Continue,
    Quit,
}

#[cfg(test)]
mod tests {
    use super::{App, AppControl, LastActionStatus, parse_window_size};
    use crate::actions::{ActionSource, AppAction};
    use crate::mapping::{MappingEntry, MappingSourceKind, default_mapping_source_device};
    use crate::midi_io::{MidiInputEvent, MidiInputMessage, MidiPortRef};
    use crate::routing::TrackPortSelection;
    use crate::transport::{QuantizeMode, RecordMode};
    use crate::ui::TimelineFlow;

    #[test]
    fn parse_window_size_accepts_kmsdrm_override_format() {
        assert_eq!(parse_window_size("1024x600"), Some((1024, 600)));
        assert_eq!(parse_window_size(" 1024 X 600 "), Some((1024, 600)));
    }

    #[test]
    fn parse_window_size_rejects_invalid_kmsdrm_override_format() {
        assert_eq!(parse_window_size("1024"), None);
        assert_eq!(parse_window_size("1024x0"), None);
        assert_eq!(parse_window_size("0x600"), None);
        assert_eq!(parse_window_size("widextall"), None);
    }

    #[test]
    fn apply_action_sets_active_track_and_current_track_flags() {
        let mut app = App::new();
        assert_eq!(app.project.active_track_index, 0);

        let control = app.apply_action(AppAction::SelectTrack(2));
        app.apply_action(AppAction::ToggleCurrentTrackLoop);
        app.apply_action(AppAction::ToggleCurrentTrackArm);

        assert_eq!(control, AppControl::Continue);
        assert_eq!(app.project.active_track_index, 2);
        assert!(app.project.tracks[2].state.loop_enabled);
        assert!(app.project.tracks[2].state.armed);
    }

    #[test]
    fn apply_action_toggles_transport_flags() {
        let mut app = App::new();
        assert!(!app.project.transport.playing);
        assert!(app.project.transport.loop_enabled);

        app.apply_action(AppAction::TogglePlayback);
        app.apply_action(AppAction::ToggleGlobalLoop);

        assert!(app.project.transport.playing);
        assert!(!app.project.transport.loop_enabled);
    }

    #[test]
    fn note_actions_select_and_nudge_active_track_notes() {
        let mut app = App::new();
        app.project.select_track(0);
        app.playhead_ticks = 0;

        app.apply_action(AppAction::SelectNotesAtPlayhead);
        let selected = app.project.active_track().unwrap().selected_note_indices();
        assert!(!selected.is_empty());

        let before_start = app.project.active_track().unwrap().midi_notes[selected[0]].start_ticks;
        let before_pitch = app.project.active_track().unwrap().midi_notes[selected[0]].pitch;
        app.apply_action(AppAction::NudgeSelectedNotesLater);
        app.apply_action(AppAction::NudgeSelectedNotesUp);

        let active = app.project.active_track().unwrap();
        assert_eq!(
            active.midi_notes[selected[0]].start_ticks,
            before_start + app.note_time_nudge_step_ticks()
        );
        assert_eq!(active.midi_notes[selected[0]].pitch, before_pitch + 1);
    }

    #[test]
    fn note_additive_hold_mapping_uses_press_and_release() {
        let mut app = App::new();
        app.project.select_track(0);
        app.playhead_ticks = 0;
        app.mappings = vec![MappingEntry {
            source_kind: MappingSourceKind::Midi,
            source_device_label: "Any MIDI".to_string(),
            source_label: "Note C2".to_string(),
            target_label: "Select Notes At Playhead Add".to_string(),
            scope_label: "Active Track".to_string(),
            enabled: true,
        }];

        app.handle_midi_input_event(MidiInputEvent {
            port: MidiPortRef::new("Port A"),
            channel: 1,
            message: MidiInputMessage::NoteOn {
                pitch: 36,
                velocity: 127,
            },

            received_at: std::time::Instant::now(),
            backend_timestamp_micros: None,
            sequence: 0,
        });

        assert!(app.note_additive_select_held);
        assert!(app.project.active_track().unwrap().has_note_selection());

        app.handle_midi_input_event(MidiInputEvent {
            port: MidiPortRef::new("Port A"),
            channel: 1,
            message: MidiInputMessage::NoteOff { pitch: 36 },

            received_at: std::time::Instant::now(),
            backend_timestamp_micros: None,
            sequence: 0,
        });

        assert!(!app.note_additive_select_held);
    }

    #[test]
    fn note_time_nudge_defaults_to_editor_step_when_quantize_is_off() {
        let mut app = App::new();
        app.project.transport.quantize = QuantizeMode::Off;

        assert_eq!(app.note_time_nudge_step_ticks(), 120);
    }

    #[test]
    fn apply_action_sets_current_track_loop_bounds_from_playhead() {
        let mut app = App::new();
        app.playhead_ticks = 1_440;
        app.apply_action(AppAction::SetCurrentTrackLoopStart);
        app.playhead_ticks = 2_880;
        app.apply_action(AppAction::SetCurrentTrackLoopEnd);

        let active = app.project.active_track().unwrap();
        assert_eq!(active.loop_region.start_ticks, 1_440);
        assert_eq!(active.loop_region.end_ticks(), 2_880);
    }

    #[test]
    fn apply_action_sets_global_loop_bounds_from_playhead() {
        let mut app = App::new();
        app.playhead_ticks = 960;
        app.apply_action(AppAction::SetGlobalLoopStart);
        app.playhead_ticks = 3_840;
        app.apply_action(AppAction::SetGlobalLoopEnd);

        assert_eq!(app.project.loop_region.start_ticks, 960);
        assert_eq!(app.project.loop_region.end_ticks(), 3_840);
    }

    #[test]
    fn app_still_supports_absolute_flow_override() {
        let mut app = App::new();
        let control = app.apply_action(AppAction::SetTimelineFlow(TimelineFlow::AcrossRows));

        assert_eq!(control, AppControl::Continue);
        assert_eq!(app.timeline_flow, TimelineFlow::AcrossRows);
    }

    #[test]
    fn effective_track_playhead_wraps_inside_track_loop() {
        let mut app = App::new();
        let track = app.project.active_track_mut().unwrap();
        track.state.loop_enabled = true;
        track.loop_region.start_ticks = 960;
        track.loop_region.length_ticks = 960;
        app.transport_ticks = 2_400;
        app.playhead_ticks = 2_400;

        assert_eq!(
            app.effective_track_playhead(app.project.active_track().unwrap()),
            1_440
        );
    }

    #[test]
    fn effective_track_playhead_moves_even_before_loop_start() {
        let mut app = App::new();
        let track = app.project.active_track_mut().unwrap();
        track.state.loop_enabled = true;
        track.loop_region.start_ticks = 1_920;
        track.loop_region.length_ticks = 960;
        app.transport_ticks = 480;
        app.playhead_ticks = 480;

        assert_eq!(
            app.effective_track_playhead(app.project.active_track().unwrap()),
            2_400
        );
    }

    #[test]
    fn effective_track_playhead_uses_transport_phase_when_song_wraps() {
        let mut app = App::new();
        app.project.transport.loop_enabled = true;
        app.project.loop_region = crate::timeline::LoopRegion::new(0, 1_920);
        let track = app.project.active_track_mut().unwrap();
        track.state.loop_enabled = true;
        track.loop_region = crate::timeline::LoopRegion::new(6_720, 4_560);

        app.transport_ticks = 3_138_174;
        app.playhead_ticks = app.song_playhead_for_transport(app.transport_ticks);

        assert_eq!(
            app.effective_track_playhead(app.project.active_track().unwrap()),
            7_614
        );
        assert_eq!(app.playhead_ticks, 894);
    }

    #[test]
    fn effective_track_playhead_uses_song_playhead_when_track_loop_is_off() {
        let mut app = App::new();
        app.project.transport.loop_enabled = true;
        app.project.loop_region = crate::timeline::LoopRegion::new(0, 1_920);
        let track = app.project.active_track_mut().unwrap();
        track.state.loop_enabled = false;
        track.loop_region = crate::timeline::LoopRegion::new(6_720, 4_560);

        app.transport_ticks = 3_138_174;
        app.playhead_ticks = app.song_playhead_for_transport(app.transport_ticks);

        assert_eq!(
            app.effective_track_playhead(app.project.active_track().unwrap()),
            894
        );
    }

    #[test]
    fn nudge_actions_shift_current_track_loop_by_quantize_step() {
        let mut app = App::new();
        let start = app.project.active_track().unwrap().loop_region.start_ticks;

        app.apply_action(AppAction::NudgeCurrentTrackLoopForward);
        assert_eq!(
            app.project.active_track().unwrap().loop_region.start_ticks,
            start + app.nudge_step_ticks()
        );

        app.apply_action(AppAction::NudgeCurrentTrackLoopBackward);
        assert_eq!(
            app.project.active_track().unwrap().loop_region.start_ticks,
            start
        );
    }

    #[test]
    fn nudge_actions_shift_global_loop_by_quantize_step() {
        let mut app = App::new();
        let start = app.project.loop_region.start_ticks;

        app.apply_action(AppAction::NudgeGlobalLoopForward);
        assert_eq!(
            app.project.loop_region.start_ticks,
            start + app.nudge_step_ticks()
        );
    }

    #[test]
    fn resize_actions_change_current_track_loop_length() {
        let mut app = App::new();
        let base = app.project.active_track().unwrap().loop_region.length_ticks;

        app.apply_action(AppAction::ExtendCurrentTrackLoop);
        assert_eq!(
            app.project.active_track().unwrap().loop_region.length_ticks,
            base + app.nudge_step_ticks()
        );

        app.apply_action(AppAction::HalfCurrentTrackLoop);
        assert!(
            app.project.active_track().unwrap().loop_region.length_ticks
                <= base + app.nudge_step_ticks()
        );

        app.apply_action(AppAction::DoubleCurrentTrackLoop);
        assert!(
            app.project.active_track().unwrap().loop_region.length_ticks
                >= base + app.nudge_step_ticks()
        );
    }

    #[test]
    fn resize_actions_change_global_loop_length() {
        let mut app = App::new();
        let base = app.project.loop_region.length_ticks;

        app.apply_action(AppAction::ShortenGlobalLoop);
        assert_eq!(
            app.project.loop_region.length_ticks,
            base.saturating_sub(app.nudge_step_ticks()).max(1)
        );

        app.apply_action(AppAction::DoubleGlobalLoop);
        assert!(app.project.loop_region.length_ticks >= 2);
    }

    #[test]
    fn apply_action_with_source_updates_last_action_status() {
        let mut app = App::new();

        app.apply_action_with_source(AppAction::TogglePlayback, ActionSource::Keyboard);

        assert_eq!(
            app.status_state.last_action,
            Some(LastActionStatus {
                action: AppAction::TogglePlayback,
                source: ActionSource::Keyboard,
            })
        );
    }

    #[test]
    fn key_mappings_execute_before_built_in_keyboard_bindings() {
        let mut app = App::new();
        app.project.transport.playing = false;
        app.project.transport.recording = false;
        app.mappings = vec![MappingEntry {
            source_kind: MappingSourceKind::Key,
            source_device_label: default_mapping_source_device(),
            source_label: "Space".to_string(),
            target_label: "Record".to_string(),
            scope_label: "Armed/Active".to_string(),
            enabled: true,
        }];

        let control = app.handle_keyboard_event(&sdl3::event::Event::KeyDown {
            timestamp: 0,
            window_id: 0,
            keycode: Some(sdl3::keyboard::Keycode::Space),
            scancode: None,
            keymod: sdl3::keyboard::Mod::NOMOD,
            repeat: false,
            which: 0,
            raw: 0,
        });

        assert_eq!(control, Some(AppControl::Continue));
        assert!(app.project.transport.recording);
        assert!(app.project.transport.playing);
    }

    #[test]
    fn reset_global_loop_restores_full_song_range() {
        let mut app = App::new();
        app.project.loop_region.start_ticks = 1_920;
        app.project.loop_region.length_ticks = 1;
        app.playhead_ticks = 1_920;

        app.apply_action(AppAction::ResetGlobalLoop);

        assert_eq!(app.project.loop_region, app.project.full_song_range());
        assert!(app.project.transport.loop_enabled);
    }

    #[test]
    fn toggle_recording_creates_visible_take_content() {
        let mut app = App::new();
        let track = app.project.active_track_mut().unwrap();
        track.clear_content();
        track.routing.input_port = TrackPortSelection::named(MidiPortRef::new("Test Input"));
        app.transport_ticks = 0;
        app.playhead_ticks = 0;

        app.apply_action(AppAction::ToggleRecording);
        assert!(app.project.transport.recording);
        assert!(app.project.transport.playing);

        let input_port = app
            .project
            .active_track()
            .and_then(|track| track.routing.input_port.as_named_port().cloned())
            .expect("test track should have explicit input port");
        app.inject_midi_input_event(MidiInputEvent {
            port: input_port.clone(),
            channel: 1,
            message: MidiInputMessage::NoteOn {
                pitch: 64,
                velocity: 100,
            },

            received_at: std::time::Instant::now(),
            backend_timestamp_micros: None,
            sequence: 0,
        });

        app.transport_ticks = 1_920;
        app.playhead_ticks = 1_920;
        app.inject_midi_input_event(MidiInputEvent {
            port: input_port,
            channel: 1,
            message: MidiInputMessage::NoteOff { pitch: 64 },

            received_at: std::time::Instant::now(),
            backend_timestamp_micros: None,
            sequence: 0,
        });
        app.apply_action(AppAction::ToggleRecording);

        let active = app.project.active_track().unwrap();
        assert!(!app.project.transport.recording);
        assert!(active.active_take.is_none());
        assert!(!active.regions.is_empty());
        assert!(active.midi_notes.iter().any(|note| note.pitch == 64));
    }

    #[test]
    fn runtime_recording_snapshot_clears_pending_note_after_note_off() {
        let mut app = App::new();
        let track = app.project.active_track_mut().unwrap();
        track.clear_content();
        track.routing.input_port = TrackPortSelection::named(MidiPortRef::new("Test Input"));

        app.apply_action(AppAction::ToggleRecording);

        let input_port = app
            .project
            .active_track()
            .and_then(|track| track.routing.input_port.as_named_port().cloned())
            .expect("test track should have explicit input port");
        app.inject_midi_input_event(MidiInputEvent {
            port: input_port.clone(),
            channel: 1,
            message: MidiInputMessage::NoteOn {
                pitch: 64,
                velocity: 100,
            },
            received_at: std::time::Instant::now(),
            backend_timestamp_micros: None,
            sequence: 0,
        });
        assert_eq!(
            app.project
                .active_track()
                .unwrap()
                .active_take
                .as_ref()
                .map(|take| take.pending_notes.len()),
            Some(1)
        );

        app.transport_ticks = 960;
        app.playhead_ticks = 960;
        app.inject_midi_input_event(MidiInputEvent {
            port: input_port,
            channel: 1,
            message: MidiInputMessage::NoteOff { pitch: 64 },
            received_at: std::time::Instant::now(),
            backend_timestamp_micros: None,
            sequence: 0,
        });

        let take = app
            .project
            .active_track()
            .unwrap()
            .active_take
            .as_ref()
            .expect("active take should still exist while recording");
        assert!(take.pending_notes.is_empty());
        assert_eq!(take.recorded_notes.len(), 1);
        assert_eq!(take.recorded_notes[0].pitch, 64);
    }

    #[test]
    fn runtime_recording_snapshot_clears_active_take_after_stop() {
        let mut app = App::new();
        let track = app.project.active_track_mut().unwrap();
        track.clear_content();
        track.routing.input_port = TrackPortSelection::named(MidiPortRef::new("Test Input"));

        app.apply_action(AppAction::ToggleRecording);
        let input_port = app
            .project
            .active_track()
            .and_then(|track| track.routing.input_port.as_named_port().cloned())
            .expect("test track should have explicit input port");
        app.inject_midi_input_event(MidiInputEvent {
            port: input_port.clone(),
            channel: 1,
            message: MidiInputMessage::NoteOn {
                pitch: 64,
                velocity: 100,
            },
            received_at: std::time::Instant::now(),
            backend_timestamp_micros: None,
            sequence: 0,
        });
        app.transport_ticks = 960;
        app.playhead_ticks = 960;
        app.inject_midi_input_event(MidiInputEvent {
            port: input_port,
            channel: 1,
            message: MidiInputMessage::NoteOff { pitch: 64 },
            received_at: std::time::Instant::now(),
            backend_timestamp_micros: None,
            sequence: 0,
        });

        app.apply_action(AppAction::ToggleRecording);

        assert!(!app.project.transport.recording);
        assert!(app.project.active_track().unwrap().active_take.is_none());
    }

    #[test]
    fn cycle_record_mode_updates_transport() {
        let mut app = App::new();
        assert_eq!(app.project.transport.record_mode, RecordMode::Overdub);

        app.apply_action(AppAction::CycleRecordMode);
        assert_eq!(app.project.transport.record_mode, RecordMode::Replace);
    }

    #[test]
    fn clear_actions_remove_track_content() {
        let mut app = App::new();
        app.apply_action(AppAction::ClearCurrentTrackContent);
        assert!(app.project.active_track().unwrap().midi_notes.is_empty());
        assert!(app.project.active_track().unwrap().regions.is_empty());

        app.project.tracks[1]
            .regions
            .push(crate::timeline::Region::new(0, 480));
        app.apply_action(AppAction::ClearAllTrackContent);
        assert!(
            app.project
                .tracks
                .iter()
                .all(|track| track.midi_notes.is_empty())
        );
        assert!(
            app.project
                .tracks
                .iter()
                .all(|track| track.regions.is_empty())
        );
    }

    #[test]
    fn default_routes_follow_current_defaults_without_mutating_none_or_named_routes() {
        let mut app = App::new();
        app.midi_devices.inputs = vec![MidiPortRef::new("In A"), MidiPortRef::new("In B")];
        app.midi_devices.outputs = vec![MidiPortRef::new("Out A"), MidiPortRef::new("Out B")];
        app.set_preferred_default_input_from_index(0);
        app.set_preferred_default_output_from_index(0);

        app.project.tracks[0].routing.input_port = TrackPortSelection::Default;
        app.project.tracks[0].routing.output_port = TrackPortSelection::Default;
        app.project.tracks[1].routing.input_port =
            TrackPortSelection::named(MidiPortRef::new("In A"));
        app.project.tracks[1].routing.output_port =
            TrackPortSelection::named(MidiPortRef::new("Out A"));
        app.project.tracks[2].routing.input_port = TrackPortSelection::None;
        app.project.tracks[2].routing.output_port = TrackPortSelection::None;

        assert_eq!(
            app.resolved_input_port(&app.project.tracks[0].routing.input_port)
                .map(|port| port.name.as_str()),
            Some("In A")
        );
        assert_eq!(
            app.project.tracks[0]
                .routing
                .output_port
                .resolve(app.default_output_port())
                .map(|port| port.name.as_str()),
            Some("Out A")
        );

        app.set_preferred_default_input_from_index(1);
        app.set_preferred_default_output_from_index(1);

        assert_eq!(
            app.project.tracks[0].routing.input_port,
            TrackPortSelection::Default
        );
        assert_eq!(
            app.project.tracks[0].routing.output_port,
            TrackPortSelection::Default
        );
        assert_eq!(
            app.resolved_input_port(&app.project.tracks[0].routing.input_port)
                .map(|port| port.name.as_str()),
            Some("In B")
        );
        assert_eq!(
            app.project.tracks[0]
                .routing
                .output_port
                .resolve(app.default_output_port())
                .map(|port| port.name.as_str()),
            Some("Out B")
        );

        assert_eq!(
            app.project.tracks[1].routing.input_port,
            TrackPortSelection::named(MidiPortRef::new("In A"))
        );
        assert_eq!(
            app.project.tracks[1].routing.output_port,
            TrackPortSelection::named(MidiPortRef::new("Out A"))
        );
        assert_eq!(
            app.resolved_input_port(&app.project.tracks[1].routing.input_port)
                .map(|port| port.name.as_str()),
            Some("In A")
        );
        assert_eq!(
            app.project.tracks[1]
                .routing
                .output_port
                .resolve(app.default_output_port())
                .map(|port| port.name.as_str()),
            Some("Out A")
        );
        assert_eq!(
            app.resolved_input_port(&app.project.tracks[2].routing.input_port),
            None
        );
        assert_eq!(
            app.project.tracks[2]
                .routing
                .output_port
                .resolve(app.default_output_port()),
            None
        );
    }

    #[test]
    fn passthrough_does_not_treat_none_input_as_default_route() {
        let mut app = App::new();
        app.midi_devices.inputs = vec![MidiPortRef::new("In A")];
        app.midi_devices.outputs = vec![MidiPortRef::new("Out A")];
        app.set_preferred_default_input_from_index(0);
        app.set_preferred_default_output_from_index(0);

        let track = &mut app.project.tracks[0];
        track.routing.input_port = TrackPortSelection::None;
        track.routing.output_port = TrackPortSelection::Default;
        track.routing.output_channel = Some(1);
        track.state.passthrough = true;

        app.handle_midi_input_event(MidiInputEvent {
            port: MidiPortRef::new("In A"),
            channel: 1,
            message: MidiInputMessage::NoteOn {
                pitch: 64,
                velocity: 100,
            },

            received_at: std::time::Instant::now(),
            backend_timestamp_micros: None,
            sequence: 0,
        });

        assert!(app.midi_output.sent_messages().is_empty());
    }
}
