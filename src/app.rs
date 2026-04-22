use crate::actions::{
    action_label, built_in_keyboard_binding_labels, ActionSource, AppAction, KeyboardBindings,
};
use crate::app_ui::branding;
use crate::engine::EngineConfig;
use crate::link::{LinkRuntime, LinkSnapshot};
use crate::mapping::{
    cycle_mapping_scope_value, cycle_mapping_source_device_label, cycle_mapping_source_kind,
    cycle_mapping_source_label, cycle_mapping_target_label, default_mapping_source_device,
    default_scope_label, default_source_label, demo_mappings, mapping_entry_key_actions,
    mapping_entry_targets_action, mapping_entry_to_actions, mapping_scope_valid_for_target,
    MappingEntry, MappingSourceKind,
};
use crate::midi_fx::{
    cycle_existing_fx_kind, cycle_fx_kind, fx_slot_label, note_name,
    playback_timing_lookback_ticks, process_live_chain_event, process_live_chain_tick,
    reset_live_fx_timing, transform_notes, LiveMidiFxEvent, LiveMidiFxState, MidiFx,
    MidiFxChainKind, MidiFxInlineParam, MidiFxSlot, MIDI_FX_SLOT_COUNT,
};
use crate::midi_io::{
    MidiDeviceCatalog, MidiInputEvent, MidiInputMessage, MidiInputRuntime, MidiOutputRuntime,
    MidiPortRef,
};
use crate::page_widgets::{handle_page_pointer, page_discoverability_targets, render_page};
use crate::pages::{
    AppPage, AppPageState, MappingField, MappingPageMode, MidiIoListFocus, RoutingField,
};
use crate::project::{MidiNote, Project, RecordingView, Track, STORED_LOOP_SLOT_COUNT};
use crate::routing::MidiChannelFilter;
use crate::state::PersistedAppState;
use crate::timeline_fx::{TimelineContext, TimelineFxField};
use crate::ui::{LayoutMode, TimelineFlow};
use image::RgbaImage;
use sdl3::pixels::{Color, PixelFormat};
use sdl3::rect::Rect;
use sdl3::render::{Canvas, RenderTarget};
use sdl3::surface::SurfaceRef;
use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};

#[path = "app/capture.rs"]
mod capture;
#[path = "app/direct_mapping_ui.rs"]
mod direct_mapping_ui;
#[path = "app/discoverability_ui.rs"]
mod discoverability_ui;
#[path = "app/input.rs"]
mod input;
#[path = "app/labels.rs"]
mod labels;
#[path = "app/mapping_ui.rs"]
mod mapping_ui;
#[path = "app/mapping_lookup.rs"]
mod mapping_lookup;
#[path = "app/midi_io_page.rs"]
mod midi_io_page;
#[path = "app/routing_ui.rs"]
mod routing_ui;
#[path = "app/shell_ui.rs"]
mod shell_ui;
#[path = "app/timeline_fx_ui.rs"]
mod timeline_fx_ui;
#[path = "app/timeline_layout.rs"]
mod timeline_layout;
#[path = "app/timeline_page.rs"]
mod timeline_page;
#[path = "app/timeline_recording.rs"]
mod timeline_recording;
#[path = "app/timeline_ui.rs"]
mod timeline_ui;
#[path = "app/types.rs"]
mod types;

use capture::{
    capture_specs, chip_row_width, readback_color_at, readback_rect_rgba, seed_capture_demo_track,
};
pub(super) use input::rect_contains;
use labels::{
    action_source_label, badge_kind_prefix, compact_badge_text, compact_scope_label,
    launch_quantize_label, mapping_badge_palette, mapping_field_index, mapping_source_label,
    mapping_source_sort_key, quantize_label,
};
use mapping_lookup::mapping_target_lookup_input;
use discoverability_ui::track_indicator_target;
use mapping_ui::{direct_mapping_key_label, mapping_target_label_for_action};
pub(crate) use types::DiscoverabilityTarget;
use types::{
    ActionDiscoverabilitySummary, ActiveMappingTargetLookup, AppOverlay, DirectMappingMode,
    DirectMappingOrigin, DirectMappingState, DirectMappingTarget, LastActionStatus, MappingBadge,
    MappingTargetLookupLayout, MappingTargetLookupState, OverlayState, RecordingLaneLayout,
    RecordingLaneWindow, StatusState, TimelineFxRowLayout, TimelineFxRowRef, TimelineTrackLayout,
};
pub use types::{RunOptions, UiCaptureOptions, UiScalingMode, VideoMode};

const MIDI_REFRESH_INTERVAL: Duration = Duration::from_millis(1_000);

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
    link: LinkRuntime,
    mappings: Vec<MappingEntry>,
    overlay_state: OverlayState,
    status_state: StatusState,
    direct_mapping_state: DirectMappingState,
    target_lookup_state: MappingTargetLookupState,
    viewport_size: (u32, u32),
    ui_scale_override: Option<f32>,
    ui_scaling_mode: UiScalingMode,
    transport_ticks: u64,
    playhead_ticks: u64,
    live_fx_ticks: u64,
    link_snapshot: LinkSnapshot,
    note_additive_select_held: bool,
    focused_track_view: bool,
    startup_started_at: Instant,
    last_midi_refresh_at: Instant,
    preferred_default_input_name: Option<String>,
    preferred_default_output_name: Option<String>,
    input_fx_live_states: Vec<LiveMidiFxState>,
    output_fx_live_states: Vec<LiveMidiFxState>,
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
        Self {
            project,
            engine_config: EngineConfig::default(),
            layout_mode: LayoutMode::FixedFit,
            timeline_flow: TimelineFlow::DownwardColumns,
            keyboard_bindings: KeyboardBindings,
            page_state,
            midi_devices: scanned_devices,
            midi_input: MidiInputRuntime::default(),
            midi_output: MidiOutputRuntime::default(),
            link,
            mappings,
            overlay_state: OverlayState::default(),
            status_state: StatusState::default(),
            direct_mapping_state: DirectMappingState::default(),
            target_lookup_state: MappingTargetLookupState::default(),
            viewport_size: (1280, 720),
            ui_scale_override: None,
            ui_scaling_mode: UiScalingMode::Auto,
            transport_ticks: 0,
            playhead_ticks: 0,
            live_fx_ticks: 0,
            link_snapshot,
            note_additive_select_held: false,
            focused_track_view: false,
            startup_started_at: Instant::now(),
            last_midi_refresh_at: Instant::now() - MIDI_REFRESH_INTERVAL,
            preferred_default_input_name,
            preferred_default_output_name,
            input_fx_live_states: vec![LiveMidiFxState::default(); track_count],
            output_fx_live_states: vec![LiveMidiFxState::default(); track_count],
        }
    }

    pub fn set_ui_scale_override(&mut self, scale: Option<f32>) {
        self.ui_scale_override = scale.filter(|value| *value >= 1.0);
    }

    pub fn set_ui_scaling_mode(&mut self, mode: UiScalingMode) {
        self.ui_scaling_mode = mode;
    }

    pub fn bootstrap_summary(&self) -> String {
        format!(
            "trekr bootstrap: project='{}', tracks={}, active_track={}, page={}, layout={:?}, sample_rate={}, song_ticks={}, playing={}, loop_enabled={}, midi_inputs={}, midi_outputs={}",
            self.project.name,
            self.project.tracks.len(),
            self.project.active_track_index + 1,
            self.page_state.current_page.label(),
            self.layout_mode,
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

        let mut window_builder = video.window("trekr", 1280, 720);
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

        for spec in capture_specs() {
            self.page_state.current_page = spec.page;
            self.overlay_state.active = spec.overlay;
            self.focused_track_view = spec.focused_track_view;
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
                port_name(active.routing.input_port.as_ref()),
                input_channel_label(active.routing.input_channel),
                port_name(active.routing.output_port.as_ref()),
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

    fn apply_action(&mut self, action: AppAction) -> AppControl {
        match action {
            AppAction::Quit => AppControl::Quit,
            AppAction::ShowPage(page) => {
                self.clear_mapping_target_lookup();
                self.page_state.current_page = page;
                self.sync_midi_inputs();
                AppControl::Continue
            }
            AppAction::ShowNextPage => {
                self.clear_mapping_target_lookup();
                self.page_state.current_page = self.page_state.current_page.next();
                self.sync_midi_inputs();
                AppControl::Continue
            }
            AppAction::ShowPreviousPage => {
                self.clear_mapping_target_lookup();
                self.page_state.current_page = self.page_state.current_page.previous();
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

    fn record_head_ticks(&self, track: &Track) -> u64 {
        if track.state.loop_enabled {
            self.effective_track_playhead(track)
        } else {
            self.playhead_ticks
        }
    }

    fn record_capture_ticks(&self, track: &Track) -> u64 {
        if self.record_context(track).is_some() {
            self.transport_ticks
        } else {
            self.record_head_ticks(track)
        }
    }

    fn live_input_event_ticks(&self, track: &Track) -> u64 {
        if self.project.transport.playing {
            self.record_capture_ticks(track)
        } else {
            self.live_fx_ticks
        }
    }

    fn record_context(&self, track: &Track) -> Option<crate::project::RecordContext> {
        if track.state.loop_enabled {
            Some(crate::project::RecordContext {
                range: track.loop_region,
                wrap_basis_ticks: 0,
                extend_clip_on_wrap: self.project.transport.loop_recording_extends_clip,
            })
        } else if self.project.transport.loop_enabled {
            Some(crate::project::RecordContext {
                range: self.project.loop_region,
                wrap_basis_ticks: self.project.loop_region.start_ticks,
                extend_clip_on_wrap: self.project.transport.loop_recording_extends_clip,
            })
        } else {
            None
        }
    }

    fn detail_loop_range(&self, track: &Track) -> crate::timeline::LoopRegion {
        self.record_context(track)
            .map(|context| context.range)
            .unwrap_or(track.loop_region)
    }

    fn begin_recording(&mut self) {
        let target_indices = self.record_target_indices();
        if target_indices.is_empty() {
            return;
        }

        for index in target_indices {
            let pressed_at = self
                .project
                .tracks
                .get(index)
                .map(|track| self.record_capture_ticks(track))
                .unwrap_or(self.playhead_ticks);
            if let Some(track) = self.project.tracks.get_mut(index) {
                track.clear_queued_stored_loop_recall();
                track.begin_recording(pressed_at);
            }
        }
        self.project.transport.recording = true;
        self.project.transport.playing = true;
    }

    fn finish_recording(&mut self) {
        let transport = self.project.transport;
        let track_count = self.project.tracks.len();

        for index in 0..track_count {
            let release_ticks = self
                .project
                .tracks
                .get(index)
                .map(|track| self.record_capture_ticks(track))
                .unwrap_or(self.playhead_ticks);
            let record_context = self
                .project
                .tracks
                .get(index)
                .and_then(|track| self.record_context(track));
            if let Some(track) = self.project.tracks.get_mut(index) {
                if track.active_take.is_some() {
                    track.finish_recording(transport, release_ticks, record_context);
                }
            }
        }

        self.project.transport.recording = false;
        self.sync_active_track_recording_clip_scroll();
    }

    fn record_target_indices(&self) -> Vec<usize> {
        let armed: Vec<usize> = self
            .project
            .tracks
            .iter()
            .enumerate()
            .filter_map(|(index, track)| track.state.armed.then_some(index))
            .collect();
        if armed.is_empty() {
            vec![self.project.active_track_index]
        } else {
            armed
        }
    }

    fn seed_demo_routing(&mut self) {
        let input_default = self.midi_devices.selected_input_port().cloned();
        let output_count = self.midi_devices.outputs.len().max(1);
        for (index, track) in self.project.tracks.iter_mut().enumerate() {
            track.routing.input_port = input_default.clone();
            track.routing.input_channel = if index % 2 == 0 {
                MidiChannelFilter::Omni
            } else {
                MidiChannelFilter::Channel(((index % 16) + 1) as u8)
            };
            track.routing.output_port =
                self.midi_devices.outputs.get(index % output_count).cloned();
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

    fn select_previous_page_item(&mut self) {
        if self.mapping_target_lookup_is_active() {
            self.move_mapping_target_lookup_highlight(-1);
            return;
        }
        self.clear_mapping_target_lookup();
        match self.page_state.current_page {
            AppPage::Timeline => {
                if self.page_state.selected_timeline_context == TimelineContext::TrackTimeline {
                    self.project.select_previous_track();
                } else {
                    self.select_timeline_fx_row(-1);
                }
            }
            AppPage::Mappings => {
                if !self.mappings.is_empty() {
                    let count = self.mappings.len();
                    self.page_state.selected_mapping_index =
                        (self.page_state.selected_mapping_index + count - 1) % count;
                    self.normalize_selected_mapping_field();
                    self.page_state.mapping_midi_learn_armed = false;
                }
            }
            AppPage::MidiIo => match self.page_state.midi_io.focus {
                MidiIoListFocus::Inputs => {
                    let count = self.midi_devices.inputs.len().max(1);
                    self.page_state.midi_io.selected_input_index =
                        (self.page_state.midi_io.selected_input_index + count - 1) % count;
                }
                MidiIoListFocus::Outputs => {
                    let count = self.midi_devices.outputs.len().max(1);
                    self.page_state.midi_io.selected_output_index =
                        (self.page_state.midi_io.selected_output_index + count - 1) % count;
                }
            },
            AppPage::Routing => {
                self.page_state.selected_routing_field =
                    self.page_state.selected_routing_field.previous();
            }
        }
    }

    fn select_next_page_item(&mut self) {
        if self.mapping_target_lookup_is_active() {
            self.move_mapping_target_lookup_highlight(1);
            return;
        }
        self.clear_mapping_target_lookup();
        match self.page_state.current_page {
            AppPage::Timeline => {
                if self.page_state.selected_timeline_context == TimelineContext::TrackTimeline {
                    self.project.select_next_track();
                } else {
                    self.select_timeline_fx_row(1);
                }
            }
            AppPage::Mappings => {
                if !self.mappings.is_empty() {
                    self.page_state.selected_mapping_index =
                        (self.page_state.selected_mapping_index + 1) % self.mappings.len();
                    self.normalize_selected_mapping_field();
                    self.page_state.mapping_midi_learn_armed = false;
                }
            }
            AppPage::MidiIo => match self.page_state.midi_io.focus {
                MidiIoListFocus::Inputs => {
                    let count = self.midi_devices.inputs.len().max(1);
                    self.page_state.midi_io.selected_input_index =
                        (self.page_state.midi_io.selected_input_index + 1) % count;
                }
                MidiIoListFocus::Outputs => {
                    let count = self.midi_devices.outputs.len().max(1);
                    self.page_state.midi_io.selected_output_index =
                        (self.page_state.midi_io.selected_output_index + 1) % count;
                }
            },
            AppPage::Routing => {
                self.page_state.selected_routing_field =
                    self.page_state.selected_routing_field.next();
            }
        }
    }

    fn select_previous_page_field(&mut self) {
        match self.page_state.current_page {
            AppPage::Timeline => self.select_previous_timeline_context(),
            AppPage::Mappings if self.page_state.mapping_mode == MappingPageMode::Write => {
                self.clear_mapping_target_lookup();
                self.page_state.selected_mapping_field =
                    self.previous_enabled_mapping_field(self.page_state.selected_mapping_field);
                self.page_state.mapping_midi_learn_armed = false;
            }
            _ => {}
        }
    }

    fn select_next_page_field(&mut self) {
        match self.page_state.current_page {
            AppPage::Timeline => self.select_next_timeline_context(),
            AppPage::Mappings if self.page_state.mapping_mode == MappingPageMode::Write => {
                self.clear_mapping_target_lookup();
                self.page_state.selected_mapping_field =
                    self.next_enabled_mapping_field(self.page_state.selected_mapping_field);
                self.page_state.mapping_midi_learn_armed = false;
            }
            _ => {}
        }
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

    fn select_timeline_fx_row(&mut self, delta: i32) {
        let Some(chain_kind) = self.page_state.selected_timeline_context.chain_kind() else {
            return;
        };
        let displayed_rows = self.displayed_timeline_fx_slot_indices(chain_kind);
        if displayed_rows.is_empty() {
            return;
        }
        let current = self.selected_timeline_fx_row(chain_kind) as i32;
        let next = (current + delta).rem_euclid(displayed_rows.len() as i32) as usize;
        self.set_selected_timeline_fx_row(chain_kind, next);
    }

    fn selected_timeline_fx_row(&self, chain_kind: MidiFxChainKind) -> usize {
        let Some(track) = self.project.active_track() else {
            return 0;
        };
        let stored = match chain_kind {
            MidiFxChainKind::Input => track.midi_fx.timeline_ui.input_selected_row,
            MidiFxChainKind::Output => track.midi_fx.timeline_ui.output_selected_row,
        };
        let len = self.displayed_timeline_fx_slot_indices(chain_kind).len();
        if len == 0 {
            0
        } else {
            stored.min(len - 1)
        }
    }

    fn set_selected_timeline_fx_row(&mut self, chain_kind: MidiFxChainKind, row_index: usize) {
        let len = self.displayed_timeline_fx_slot_indices(chain_kind).len();
        let clamped = if len == 0 { 0 } else { row_index.min(len - 1) };
        if let Some(track) = self.project.active_track_mut() {
            match chain_kind {
                MidiFxChainKind::Input => track.midi_fx.timeline_ui.input_selected_row = clamped,
                MidiFxChainKind::Output => track.midi_fx.timeline_ui.output_selected_row = clamped,
            }
        }
    }

    fn active_timeline_fx_slot_indices(&self, chain_kind: MidiFxChainKind) -> Vec<usize> {
        let Some(track) = self.project.active_track() else {
            return Vec::new();
        };
        self.active_timeline_fx_slot_indices_for_track(track, chain_kind)
    }

    fn active_timeline_fx_slot_indices_for_track(
        &self,
        track: &Track,
        chain_kind: MidiFxChainKind,
    ) -> Vec<usize> {
        self.fx_chain(track, chain_kind)
            .iter()
            .enumerate()
            .filter_map(|(index, slot)| slot.as_ref().map(|_| index))
            .collect()
    }

    fn displayed_timeline_fx_slot_indices(
        &self,
        chain_kind: MidiFxChainKind,
    ) -> Vec<Option<usize>> {
        let Some(track) = self.project.active_track() else {
            return Vec::new();
        };
        self.displayed_timeline_fx_slot_indices_for_track(track, chain_kind)
    }

    fn displayed_timeline_fx_slot_indices_for_track(
        &self,
        track: &Track,
        chain_kind: MidiFxChainKind,
    ) -> Vec<Option<usize>> {
        let mut rows = self
            .active_timeline_fx_slot_indices_for_track(track, chain_kind)
            .into_iter()
            .map(Some)
            .collect::<Vec<_>>();
        if self
            .first_empty_timeline_fx_slot_index_for_track(track, chain_kind)
            .is_some()
        {
            rows.push(None);
        }
        rows
    }

    fn first_empty_timeline_fx_slot_index_for_track(
        &self,
        track: &Track,
        chain_kind: MidiFxChainKind,
    ) -> Option<usize> {
        self.fx_chain(track, chain_kind)
            .iter()
            .enumerate()
            .find_map(|(index, slot)| slot.is_none().then_some(index))
    }

    fn selected_timeline_fx_slot_index(&self, chain_kind: MidiFxChainKind) -> Option<usize> {
        self.displayed_timeline_fx_slot_indices(chain_kind)
            .get(self.selected_timeline_fx_row(chain_kind))
            .copied()
            .flatten()
    }

    fn selected_timeline_fx_active_row_index(&self, chain_kind: MidiFxChainKind) -> Option<usize> {
        let selected_slot = self.selected_timeline_fx_slot_index(chain_kind)?;
        self.active_timeline_fx_slot_indices(chain_kind)
            .iter()
            .position(|slot_index| *slot_index == selected_slot)
    }

    fn set_selected_timeline_fx_slot_index(
        &mut self,
        chain_kind: MidiFxChainKind,
        slot_index: usize,
    ) {
        let row_index = self
            .displayed_timeline_fx_slot_indices(chain_kind)
            .iter()
            .position(|candidate| *candidate == Some(slot_index))
            .unwrap_or(0);
        self.set_selected_timeline_fx_row(chain_kind, row_index);
    }

    fn selected_timeline_fx_slot<'a>(
        &self,
        track: &'a Track,
        chain_kind: MidiFxChainKind,
    ) -> Option<&'a MidiFxSlot> {
        self.selected_timeline_fx_slot_index(chain_kind)
            .and_then(|slot_index| self.fx_chain(track, chain_kind).get(slot_index))
            .and_then(|slot| slot.as_ref())
    }

    fn selected_timeline_fx_param_window(&self, chain_kind: MidiFxChainKind) -> usize {
        let Some(track) = self.project.active_track() else {
            return 0;
        };
        let Some(slot_index) = self.selected_timeline_fx_slot_index(chain_kind) else {
            return 0;
        };
        let windows = match chain_kind {
            MidiFxChainKind::Input => &track.midi_fx.timeline_ui.input_param_windows,
            MidiFxChainKind::Output => &track.midi_fx.timeline_ui.output_param_windows,
        };
        windows.get(slot_index).copied().unwrap_or(0)
    }

    fn timeline_fx_param_window_for_slot(
        &self,
        context: TimelineContext,
        slot_index: usize,
    ) -> usize {
        let Some(track) = self.project.active_track() else {
            return 0;
        };
        let windows = match context.chain_kind() {
            Some(MidiFxChainKind::Input) => &track.midi_fx.timeline_ui.input_param_windows,
            Some(MidiFxChainKind::Output) => &track.midi_fx.timeline_ui.output_param_windows,
            None => return 0,
        };
        windows.get(slot_index).copied().unwrap_or(0)
    }

    fn set_selected_timeline_fx_param_window(&mut self, chain_kind: MidiFxChainKind, start: usize) {
        let Some(slot_index) = self.selected_timeline_fx_slot_index(chain_kind) else {
            return;
        };
        if let Some(track) = self.project.active_track_mut() {
            let windows = match chain_kind {
                MidiFxChainKind::Input => &mut track.midi_fx.timeline_ui.input_param_windows,
                MidiFxChainKind::Output => &mut track.midi_fx.timeline_ui.output_param_windows,
            };
            if let Some(window) = windows.get_mut(slot_index) {
                *window = start;
            }
        }
    }

    fn normalize_timeline_fx_selection(&mut self) {
        if let Some(chain_kind) = self.page_state.selected_timeline_context.chain_kind() {
            let displayed = self.displayed_timeline_fx_slot_indices(chain_kind);
            if displayed.is_empty() {
                self.set_selected_timeline_fx_row(chain_kind, 0);
                return;
            }
            self.set_selected_timeline_fx_row(
                chain_kind,
                self.selected_timeline_fx_row(chain_kind),
            );
        }
    }

    fn adjust_timeline_context(&mut self, delta: i32) {
        let Some(chain_kind) = self.page_state.selected_timeline_context.chain_kind() else {
            return;
        };
        if self.selected_timeline_fx_slot_index(chain_kind).is_none() {
            self.adjust_selected_timeline_fx_kind(delta);
        } else {
            match self.page_state.selected_timeline_fx_field {
                TimelineFxField::Enabled => self.toggle_selected_timeline_fx_enabled(),
                TimelineFxField::Kind => self.adjust_selected_timeline_fx_kind(delta),
                TimelineFxField::ParamPrimary => {
                    self.adjust_selected_timeline_fx_parameter(0, delta)
                }
                TimelineFxField::ParamSecondary => {
                    self.adjust_selected_timeline_fx_parameter(1, delta)
                }
                TimelineFxField::Scroll => self.scroll_selected_timeline_fx_parameter_window(delta),
                TimelineFxField::Move => self.move_selected_timeline_fx(delta),
            }
        }
        self.normalize_timeline_fx_selection();
        if self.page_state.selected_timeline_context.chain_kind() != Some(chain_kind) {
            self.page_state.selected_timeline_context = match chain_kind {
                MidiFxChainKind::Input => TimelineContext::InputFx,
                MidiFxChainKind::Output => TimelineContext::OutputFx,
            };
        }
    }

    fn activate_timeline_context_item(&mut self) {
        let Some(chain_kind) = self.page_state.selected_timeline_context.chain_kind() else {
            return;
        };
        if self.selected_timeline_fx_slot_index(chain_kind).is_none() {
            self.add_selected_timeline_fx();
            return;
        }
        self.page_state.selected_timeline_fx_field =
            self.page_state.selected_timeline_fx_field.next();
    }

    fn reverse_activate_timeline_context_item(&mut self) {
        let Some(chain_kind) = self.page_state.selected_timeline_context.chain_kind() else {
            return;
        };
        if self.selected_timeline_fx_slot_index(chain_kind).is_none() {
            self.activate_timeline_context_item();
            return;
        }
        self.page_state.selected_timeline_fx_field =
            self.page_state.selected_timeline_fx_field.previous();
    }

    fn toggle_selected_timeline_fx_enabled(&mut self) {
        let Some(chain_kind) = self.page_state.selected_timeline_context.chain_kind() else {
            return;
        };
        let Some(slot_index) = self.selected_timeline_fx_slot_index(chain_kind) else {
            return;
        };
        let mut changed = false;
        if let Some(track) = self.project.active_track_mut() {
            let chain = match chain_kind {
                MidiFxChainKind::Input => &mut track.midi_fx.input_fx,
                MidiFxChainKind::Output => &mut track.midi_fx.output_fx,
            };
            if let Some(Some(slot)) = chain.get_mut(slot_index) {
                slot.enabled = !slot.enabled;
                changed = true;
            }
        }
        if changed {
            self.handle_timeline_fx_configuration_changed();
        }
    }

    fn adjust_selected_timeline_fx_kind(&mut self, delta: i32) {
        let Some(chain_kind) = self.page_state.selected_timeline_context.chain_kind() else {
            return;
        };
        let selected_slot_index = self.selected_timeline_fx_slot_index(chain_kind);
        let mut changed = false;
        if let Some(track) = self.project.active_track_mut() {
            let chain = match chain_kind {
                MidiFxChainKind::Input => &mut track.midi_fx.input_fx,
                MidiFxChainKind::Output => &mut track.midi_fx.output_fx,
            };
            if let Some(slot_index) = selected_slot_index {
                if let Some(Some(slot)) = chain.get_mut(slot_index) {
                    *slot = cycle_existing_fx_kind(slot, delta);
                    changed = true;
                }
            } else if let Some(empty_slot) = chain.iter().position(|slot| slot.is_none()) {
                chain[empty_slot] = cycle_fx_kind(None, delta);
                self.set_selected_timeline_fx_slot_index(chain_kind, empty_slot);
                changed = true;
            }
        }
        self.normalize_timeline_fx_selection();
        if changed {
            self.handle_timeline_fx_configuration_changed();
        }
    }

    fn add_selected_timeline_fx(&mut self) {
        let Some(chain_kind) = self.page_state.selected_timeline_context.chain_kind() else {
            return;
        };
        let selected_slot_index = self.selected_timeline_fx_slot_index(chain_kind);
        if selected_slot_index.is_some() {
            return;
        }
        self.adjust_selected_timeline_fx_kind(1);
    }

    fn delete_selected_timeline_fx(&mut self) {
        let Some(chain_kind) = self.page_state.selected_timeline_context.chain_kind() else {
            return;
        };
        let Some(slot_index) = self.selected_timeline_fx_slot_index(chain_kind) else {
            return;
        };
        let mut changed = false;
        if let Some(track) = self.project.active_track_mut() {
            let (chain, windows) = match chain_kind {
                MidiFxChainKind::Input => (
                    &mut track.midi_fx.input_fx,
                    &mut track.midi_fx.timeline_ui.input_param_windows,
                ),
                MidiFxChainKind::Output => (
                    &mut track.midi_fx.output_fx,
                    &mut track.midi_fx.timeline_ui.output_param_windows,
                ),
            };
            chain[slot_index] = None;
            if let Some(window) = windows.get_mut(slot_index) {
                *window = 0;
            }
            changed = true;
        }
        self.normalize_timeline_fx_selection();
        if changed {
            self.handle_timeline_fx_configuration_changed();
        }
    }

    fn adjust_selected_timeline_fx_parameter(&mut self, visible_offset: usize, delta: i32) {
        let Some(chain_kind) = self.page_state.selected_timeline_context.chain_kind() else {
            return;
        };
        let Some(slot_index) = self.selected_timeline_fx_slot_index(chain_kind) else {
            return;
        };
        let track_count = self.project.tracks.len();
        let ppqn = self.project.transport.ppqn;
        let window_start = self.selected_timeline_fx_param_window(chain_kind);
        let parameter_index = window_start + visible_offset;
        let mut changed = false;
        if let Some(track) = self.project.active_track_mut() {
            let chain = match chain_kind {
                MidiFxChainKind::Input => &mut track.midi_fx.input_fx,
                MidiFxChainKind::Output => &mut track.midi_fx.output_fx,
            };
            let Some(Some(slot)) = chain.get_mut(slot_index) else {
                return;
            };
            slot.effect
                .adjust_inline_parameter(parameter_index, delta, track_count, ppqn);
            changed = true;
        }
        if changed {
            self.handle_timeline_fx_configuration_changed();
        }
    }

    fn scroll_selected_timeline_fx_parameter_window(&mut self, delta: i32) {
        let Some(chain_kind) = self.page_state.selected_timeline_context.chain_kind() else {
            return;
        };
        let Some(track) = self.project.active_track() else {
            return;
        };
        let Some(slot) = self.selected_timeline_fx_slot(track, chain_kind) else {
            return;
        };
        let param_count = slot.effect.inline_parameters().len();
        let max_start = param_count.saturating_sub(2);
        let current = self.selected_timeline_fx_param_window(chain_kind);
        let next = (current as i32 + delta).clamp(0, max_start as i32) as usize;
        self.set_selected_timeline_fx_param_window(chain_kind, next);
    }

    fn move_selected_timeline_fx(&mut self, delta: i32) {
        let Some(chain_kind) = self.page_state.selected_timeline_context.chain_kind() else {
            return;
        };
        let active_slots = self.active_timeline_fx_slot_indices(chain_kind);
        if active_slots.len() < 2 {
            return;
        }
        let Some(row_index) = self.selected_timeline_fx_active_row_index(chain_kind) else {
            return;
        };
        let target_row = if delta < 0 {
            row_index.saturating_sub(1)
        } else {
            (row_index + 1).min(active_slots.len() - 1)
        };
        if row_index == target_row {
            return;
        }
        let source_slot = active_slots[row_index];
        let target_slot = active_slots[target_row];
        let mut changed = false;
        if let Some(track) = self.project.active_track_mut() {
            let (chain, windows) = match chain_kind {
                MidiFxChainKind::Input => (
                    &mut track.midi_fx.input_fx,
                    &mut track.midi_fx.timeline_ui.input_param_windows,
                ),
                MidiFxChainKind::Output => (
                    &mut track.midi_fx.output_fx,
                    &mut track.midi_fx.timeline_ui.output_param_windows,
                ),
            };
            chain.swap(source_slot, target_slot);
            windows.swap(source_slot, target_slot);
            changed = true;
        }
        self.set_selected_timeline_fx_row(chain_kind, target_row);
        if changed {
            self.handle_timeline_fx_configuration_changed();
        }
    }

    fn adjust_page_item(&mut self, delta: i32) {
        if self.mapping_target_lookup_is_active() {
            self.move_mapping_target_lookup_highlight(delta);
            return;
        }
        match self.page_state.current_page {
            AppPage::Timeline => self.adjust_timeline_context(delta),
            AppPage::Mappings => {
                if self.page_state.mapping_mode == MappingPageMode::Write
                    && !self.mappings.is_empty()
                {
                    self.clear_mapping_target_lookup();
                    self.adjust_mapping_field(delta);
                }
            }
            AppPage::MidiIo => {
                self.page_state.midi_io.focus = self.page_state.midi_io.focus.toggle();
            }
            AppPage::Routing => self.adjust_routing_field(delta),
        }
    }

    fn activate_page_item(&mut self) {
        if self.mapping_target_lookup_is_active() {
            self.commit_mapping_target_lookup();
            return;
        }
        match self.page_state.current_page {
            AppPage::Timeline => self.activate_timeline_context_item(),
            AppPage::Mappings => {
                if self.page_state.mapping_mode == MappingPageMode::Write
                    && !self.mappings.is_empty()
                {
                    self.activate_mapping_field();
                }
            }
            AppPage::MidiIo => match self.page_state.midi_io.focus {
                MidiIoListFocus::Inputs => {
                    self.set_preferred_default_input_from_index(
                        self.page_state.midi_io.selected_input_index,
                    );
                }
                MidiIoListFocus::Outputs => self.set_preferred_default_output_from_index(
                    self.page_state.midi_io.selected_output_index,
                ),
            },
            AppPage::Routing => match self.page_state.selected_routing_field {
                RoutingField::Passthrough
                | RoutingField::RecordInputFx
                | RoutingField::MonitorInputFx
                | RoutingField::InputFxEnabled
                | RoutingField::OutputFxEnabled => self.adjust_routing_field(1),
                _ => {}
            },
        }
    }

    fn adjust_mapping_field(&mut self, delta: i32) {
        let index = self.page_state.selected_mapping_index;
        let field = self.page_state.selected_mapping_field;
        let track_count = self.project.tracks.len();
        let mapping_device_names = self
            .midi_devices
            .inputs
            .iter()
            .map(|port| port.name.clone())
            .collect::<Vec<_>>();
        let Some(entry) = self.mappings.get_mut(index) else {
            return;
        };

        self.page_state.mapping_midi_learn_armed = false;
        match field {
            MappingField::SourceKind => {
                entry.source_kind = cycle_mapping_source_kind(entry.source_kind, delta);
                if entry.source_kind != MappingSourceKind::Midi {
                    entry.source_device_label = default_mapping_source_device();
                }
                entry.source_label = default_source_label(entry.source_kind).to_string();
                self.normalize_selected_mapping_field();
            }
            MappingField::SourceDevice => {
                if entry.source_kind == MappingSourceKind::Midi {
                    entry.source_device_label = cycle_mapping_source_device_label(
                        &entry.source_device_label,
                        &mapping_device_names,
                        delta,
                    );
                }
            }
            MappingField::SourceValue => {
                entry.source_label =
                    cycle_mapping_source_label(entry.source_kind, &entry.source_label, delta)
                        .to_string();
            }
            MappingField::Target => {
                entry.target_label =
                    cycle_mapping_target_label(&entry.target_label, delta).to_string();
                if !mapping_scope_valid_for_target(
                    &entry.target_label,
                    &entry.scope_label,
                    track_count,
                ) {
                    entry.scope_label = default_scope_label(&entry.target_label, track_count);
                }
            }
            MappingField::Scope => {
                entry.scope_label = cycle_mapping_scope_value(
                    &entry.scope_label,
                    delta,
                    &entry.target_label,
                    track_count,
                );
            }
            MappingField::Enabled => {
                entry.enabled = delta > 0;
            }
        }
    }

    fn reverse_activate_page_item(&mut self) {
        match self.page_state.current_page {
            AppPage::Timeline => self.reverse_activate_timeline_context_item(),
            _ => self.activate_page_item(),
        }
    }

    fn activate_mapping_field(&mut self) {
        let index = self.page_state.selected_mapping_index;
        let field = self.page_state.selected_mapping_field;

        if field == MappingField::Target {
            self.open_mapping_target_lookup();
            return;
        }

        let track_count = self.project.tracks.len();
        let Some(entry) = self.mappings.get_mut(index) else {
            return;
        };

        match field {
            MappingField::SourceKind => {
                entry.source_kind = cycle_mapping_source_kind(entry.source_kind, 1);
                if entry.source_kind != MappingSourceKind::Midi {
                    entry.source_device_label = default_mapping_source_device();
                }
                entry.source_label = default_source_label(entry.source_kind).to_string();
                self.page_state.mapping_midi_learn_armed = false;
                self.normalize_selected_mapping_field();
            }
            MappingField::SourceDevice => {
                if entry.source_kind == MappingSourceKind::Midi {
                    let mapping_device_names = self
                        .midi_devices
                        .inputs
                        .iter()
                        .map(|port| port.name.clone())
                        .collect::<Vec<_>>();
                    entry.source_device_label = cycle_mapping_source_device_label(
                        &entry.source_device_label,
                        &mapping_device_names,
                        1,
                    );
                }
                self.page_state.mapping_midi_learn_armed = false;
            }
            MappingField::SourceValue => {
                if entry.source_kind == MappingSourceKind::Midi {
                    self.page_state.mapping_midi_learn_armed =
                        !self.page_state.mapping_midi_learn_armed;
                    self.sync_midi_inputs();
                } else {
                    entry.source_label =
                        cycle_mapping_source_label(entry.source_kind, &entry.source_label, 1)
                            .to_string();
                }
            }
            MappingField::Target => {}
            MappingField::Scope => {
                entry.scope_label = cycle_mapping_scope_value(
                    &entry.scope_label,
                    1,
                    &entry.target_label,
                    track_count,
                );
                self.page_state.mapping_midi_learn_armed = false;
            }
            MappingField::Enabled => {
                entry.enabled = !entry.enabled;
                self.page_state.mapping_midi_learn_armed = false;
            }
        }
    }

    fn add_mapping_row(&mut self) {
        if self.page_state.current_page != AppPage::Mappings
            || self.page_state.mapping_mode != MappingPageMode::Write
        {
            return;
        }

        self.clear_mapping_target_lookup();
        let insert_index = self
            .page_state
            .selected_mapping_index
            .min(self.mappings.len());
        let mut entry = self
            .mappings
            .get(insert_index)
            .cloned()
            .unwrap_or_else(MappingEntry::default_new);
        entry.enabled = false;
        entry.scope_label = default_scope_label(&entry.target_label, self.project.tracks.len());
        self.mappings
            .insert(insert_index + usize::from(!self.mappings.is_empty()), entry);
        self.page_state.selected_mapping_index =
            (insert_index + usize::from(!self.mappings.is_empty())).min(self.mappings.len() - 1);
        self.normalize_selected_mapping_field();
        self.page_state.mapping_midi_learn_armed = false;
    }

    fn remove_selected_mapping(&mut self) {
        if self.page_state.current_page != AppPage::Mappings
            || self.page_state.mapping_mode != MappingPageMode::Write
            || self.mappings.is_empty()
        {
            return;
        }

        self.clear_mapping_target_lookup();
        self.mappings.remove(self.page_state.selected_mapping_index);
        if self.mappings.is_empty() {
            self.mappings.push(MappingEntry::default_new());
        }
        self.page_state.selected_mapping_index = self
            .page_state
            .selected_mapping_index
            .min(self.mappings.len().saturating_sub(1));
        self.normalize_selected_mapping_field();
        self.page_state.mapping_midi_learn_armed = false;
    }

    fn next_enabled_mapping_field(&self, start: MappingField) -> MappingField {
        let mut field = start;
        for _ in 0..MappingField::ALL.len() {
            field = field.next();
            if self.mapping_field_enabled(field) {
                return field;
            }
        }
        start
    }

    fn previous_enabled_mapping_field(&self, start: MappingField) -> MappingField {
        let mut field = start;
        for _ in 0..MappingField::ALL.len() {
            field = field.previous();
            if self.mapping_field_enabled(field) {
                return field;
            }
        }
        start
    }

    fn normalize_selected_mapping_field(&mut self) {
        if !self.mapping_field_enabled(self.page_state.selected_mapping_field) {
            self.page_state.selected_mapping_field =
                self.next_enabled_mapping_field(self.page_state.selected_mapping_field);
        }
    }

    fn mapping_field_enabled(&self, field: MappingField) -> bool {
        let Some(entry) = self.mappings.get(self.page_state.selected_mapping_index) else {
            return field != MappingField::SourceDevice;
        };
        !matches!(field, MappingField::SourceDevice) || entry.source_kind == MappingSourceKind::Midi
    }

    fn selected_fx_slot_index(&self, chain_kind: MidiFxChainKind) -> usize {
        match chain_kind {
            MidiFxChainKind::Input => self
                .page_state
                .selected_input_fx_slot
                .min(MIDI_FX_SLOT_COUNT - 1),
            MidiFxChainKind::Output => self
                .page_state
                .selected_output_fx_slot
                .min(MIDI_FX_SLOT_COUNT - 1),
        }
    }

    fn set_selected_fx_slot_index(&mut self, chain_kind: MidiFxChainKind, index: usize) {
        let clamped = index.min(MIDI_FX_SLOT_COUNT - 1);
        match chain_kind {
            MidiFxChainKind::Input => self.page_state.selected_input_fx_slot = clamped,
            MidiFxChainKind::Output => self.page_state.selected_output_fx_slot = clamped,
        }
    }

    fn selected_fx_param_window(&self, chain_kind: MidiFxChainKind) -> usize {
        let Some(track) = self.project.active_track() else {
            return 0;
        };
        let slot_index = self.selected_fx_slot_index(chain_kind);
        let windows = match chain_kind {
            MidiFxChainKind::Input => &track.midi_fx.timeline_ui.input_param_windows,
            MidiFxChainKind::Output => &track.midi_fx.timeline_ui.output_param_windows,
        };
        windows.get(slot_index).copied().unwrap_or(0)
    }

    fn set_selected_fx_param_window(&mut self, chain_kind: MidiFxChainKind, start: usize) {
        let slot_index = self.selected_fx_slot_index(chain_kind);
        if let Some(track) = self.project.active_track_mut() {
            let windows = match chain_kind {
                MidiFxChainKind::Input => &mut track.midi_fx.timeline_ui.input_param_windows,
                MidiFxChainKind::Output => &mut track.midi_fx.timeline_ui.output_param_windows,
            };
            if let Some(window) = windows.get_mut(slot_index) {
                *window = start;
            }
        }
    }

    fn fx_chain<'a>(
        &self,
        track: &'a Track,
        chain_kind: MidiFxChainKind,
    ) -> &'a [Option<MidiFxSlot>] {
        match chain_kind {
            MidiFxChainKind::Input => &track.midi_fx.input_fx,
            MidiFxChainKind::Output => &track.midi_fx.output_fx,
        }
    }

    fn selected_fx_slot<'a>(
        &self,
        track: &'a Track,
        chain_kind: MidiFxChainKind,
    ) -> Option<&'a MidiFxSlot> {
        self.fx_chain(track, chain_kind)
            .get(self.selected_fx_slot_index(chain_kind))
            .and_then(|slot| slot.as_ref())
    }

    fn selected_fx_visible_params(
        &self,
        track: &Track,
        chain_kind: MidiFxChainKind,
    ) -> (
        Option<MidiFxInlineParam>,
        Option<MidiFxInlineParam>,
        usize,
        usize,
    ) {
        let Some(slot) = self.selected_fx_slot(track, chain_kind) else {
            return (None, None, 0, 0);
        };
        let params = slot.effect.inline_parameters();
        let window_start = self
            .selected_fx_param_window(chain_kind)
            .min(params.len().saturating_sub(1));
        (
            params.get(window_start).cloned(),
            params.get(window_start + 1).cloned(),
            params.len(),
            window_start,
        )
    }

    fn selected_fx_overflow_label(&self, track: &Track, chain_kind: MidiFxChainKind) -> String {
        let (_, _, param_count, window_start) = self.selected_fx_visible_params(track, chain_kind);
        timeline_fx_ui::timeline_fx_overflow_label(param_count, window_start)
    }

    fn adjust_fx_slot_index(&mut self, chain_kind: MidiFxChainKind, delta: i32) {
        let current = self.selected_fx_slot_index(chain_kind) as i32;
        let next = (current + delta).rem_euclid(MIDI_FX_SLOT_COUNT as i32) as usize;
        self.set_selected_fx_slot_index(chain_kind, next);
    }

    fn adjust_fx_kind(&mut self, chain_kind: MidiFxChainKind, delta: i32) {
        let track_count = self.project.tracks.len();
        let slot_index = self.selected_fx_slot_index(chain_kind);
        let Some(track) = self.project.active_track_mut() else {
            return;
        };
        let chain = match chain_kind {
            MidiFxChainKind::Input => &mut track.midi_fx.input_fx,
            MidiFxChainKind::Output => &mut track.midi_fx.output_fx,
        };
        if slot_index >= chain.len() {
            return;
        }
        let current = chain[slot_index].as_ref();
        chain[slot_index] = cycle_fx_kind(current, delta);
        if let Some(slot) = chain[slot_index].as_mut() {
            if let MidiFx::TrackClone { source_track } = &mut slot.effect {
                let max_source = track_count.saturating_sub(1);
                *source_track = (*source_track).min(max_source);
            }
        }
        self.set_selected_fx_param_window(chain_kind, 0);
    }

    fn toggle_fx_enabled(&mut self, chain_kind: MidiFxChainKind) {
        let slot_index = self.selected_fx_slot_index(chain_kind);
        let Some(track) = self.project.active_track_mut() else {
            return;
        };
        let chain = match chain_kind {
            MidiFxChainKind::Input => &mut track.midi_fx.input_fx,
            MidiFxChainKind::Output => &mut track.midi_fx.output_fx,
        };
        if let Some(Some(slot)) = chain.get_mut(slot_index) {
            slot.enabled = !slot.enabled;
        }
    }

    fn adjust_fx_parameter(
        &mut self,
        chain_kind: MidiFxChainKind,
        visible_offset: usize,
        delta: i32,
    ) {
        let slot_index = self.selected_fx_slot_index(chain_kind);
        let track_count = self.project.tracks.len();
        let ppqn = self.project.transport.ppqn;
        let active_track_index = self.project.active_track_index;
        let parameter_index = self.selected_fx_param_window(chain_kind) + visible_offset;
        let source_muted: Vec<bool> = self
            .project
            .tracks
            .iter()
            .map(|track| track.state.muted)
            .collect();
        let Some(track) = self.project.active_track_mut() else {
            return;
        };
        let chain = match chain_kind {
            MidiFxChainKind::Input => &mut track.midi_fx.input_fx,
            MidiFxChainKind::Output => &mut track.midi_fx.output_fx,
        };
        let Some(Some(slot)) = chain.get_mut(slot_index) else {
            return;
        };
        slot.effect
            .adjust_inline_parameter(parameter_index, delta, track_count, ppqn);
        if let MidiFx::TrackClone { source_track } = &mut slot.effect {
            if *source_track == active_track_index && track_count > 1 {
                *source_track = if delta >= 0 { 1 } else { track_count - 1 };
            }
            if source_muted.get(*source_track).copied().unwrap_or(false) && track_count > 1 {
                let step = if delta >= 0 { 1 } else { track_count - 1 };
                *source_track = (*source_track + step) % track_count;
            }
        }
    }

    fn adjust_routing_field(&mut self, delta: i32) {
        let current_input = self.midi_devices.selected_input_port().cloned();
        let current_output = self.midi_devices.selected_output_port().cloned();
        match self.page_state.selected_routing_field {
            RoutingField::InputDevice => {
                if let Some(track) = self.project.active_track_mut() {
                    track.routing.input_port = cycle_optional_port(
                        track.routing.input_port.as_ref(),
                        &self.midi_devices.inputs,
                        delta,
                    );
                }
                self.sync_midi_inputs();
            }
            RoutingField::InputChannel => {
                if let Some(track) = self.project.active_track_mut() {
                    track.routing.input_channel =
                        cycle_input_channel(track.routing.input_channel, delta);
                }
            }
            RoutingField::OutputDevice => {
                if let Some(track) = self.project.active_track_mut() {
                    track.routing.output_port = cycle_optional_port(
                        track.routing.output_port.as_ref(),
                        &self.midi_devices.outputs,
                        delta,
                    );
                }
            }
            RoutingField::OutputChannel => {
                if let Some(track) = self.project.active_track_mut() {
                    track.routing.output_channel =
                        cycle_output_channel(track.routing.output_channel, delta);
                }
            }
            RoutingField::Passthrough => {
                if let Some(track) = self.project.active_track_mut() {
                    track.state.passthrough = !track.state.passthrough;
                    if track.routing.input_port.is_none() {
                        track.routing.input_port = current_input;
                    }
                    if track.routing.output_port.is_none() {
                        track.routing.output_port = current_output;
                    }
                }
                self.sync_midi_inputs();
            }
            RoutingField::RecordInputFx => {
                if let Some(track) = self.project.active_track_mut() {
                    track.midi_fx.record_input_fx_mode =
                        track.midi_fx.record_input_fx_mode.toggle();
                }
            }
            RoutingField::MonitorInputFx => {
                if let Some(track) = self.project.active_track_mut() {
                    track.midi_fx.monitor_input_fx = !track.midi_fx.monitor_input_fx;
                }
            }
            RoutingField::InputFxSlot => self.adjust_fx_slot_index(MidiFxChainKind::Input, delta),
            RoutingField::InputFxKind => self.adjust_fx_kind(MidiFxChainKind::Input, delta),
            RoutingField::InputFxEnabled => self.toggle_fx_enabled(MidiFxChainKind::Input),
            RoutingField::InputFxParam1 => {
                self.adjust_fx_parameter(MidiFxChainKind::Input, 0, delta)
            }
            RoutingField::InputFxParam2 => {
                self.adjust_fx_parameter(MidiFxChainKind::Input, 1, delta)
            }
            RoutingField::InputFxMore => {
                self.scroll_fx_parameter_window(MidiFxChainKind::Input, delta)
            }
            RoutingField::OutputFxSlot => self.adjust_fx_slot_index(MidiFxChainKind::Output, delta),
            RoutingField::OutputFxKind => self.adjust_fx_kind(MidiFxChainKind::Output, delta),
            RoutingField::OutputFxEnabled => self.toggle_fx_enabled(MidiFxChainKind::Output),
            RoutingField::OutputFxParam1 => {
                self.adjust_fx_parameter(MidiFxChainKind::Output, 0, delta)
            }
            RoutingField::OutputFxParam2 => {
                self.adjust_fx_parameter(MidiFxChainKind::Output, 1, delta)
            }
            RoutingField::OutputFxMore => {
                self.scroll_fx_parameter_window(MidiFxChainKind::Output, delta)
            }
        }
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
    }

    fn set_preferred_default_output_from_index(&mut self, index: usize) {
        let Some(port) = self.midi_devices.output(index) else {
            return;
        };
        self.preferred_default_output_name = Some(port.name.clone());
        self.midi_devices.set_selected_output(index);
    }

    fn sync_midi_inputs(&mut self) {
        let mut ports = Vec::new();
        for track in &self.project.tracks {
            if let Some(port) = track.routing.input_port.clone() {
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
                output_port: track.routing.output_port.clone(),
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
                    for record_event in &post_input_events {
                        match *record_event {
                            LiveMidiFxEvent::NoteOn { pitch, velocity } => {
                                track.record_note_on(pitch, velocity, input_ticks);
                            }
                            LiveMidiFxEvent::NoteOff { pitch } => {
                                track.record_note_off(pitch, input_ticks);
                            }
                        }
                    }
                }
            }

            if target.monitor_input_fx {
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
            let output_port = track_view.routing.output_port.clone();
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
                    for (tick, event) in &input_events {
                        match *event {
                            LiveMidiFxEvent::NoteOn { pitch, velocity } => {
                                track.record_note_on(pitch, velocity, *tick);
                            }
                            LiveMidiFxEvent::NoteOff { pitch } => {
                                track.record_note_off(pitch, *tick);
                            }
                        }
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

    fn scroll_fx_parameter_window(&mut self, chain_kind: MidiFxChainKind, delta: i32) {
        let Some(track) = self.project.active_track() else {
            return;
        };
        let Some(slot) = self.selected_fx_slot(track, chain_kind) else {
            return;
        };
        let param_count = slot.effect.inline_parameters().len();
        let max_start = param_count.saturating_sub(2);
        let current = self.selected_fx_param_window(chain_kind);
        let next = (current as i32 + delta).clamp(0, max_start as i32) as usize;
        self.set_selected_fx_param_window(chain_kind, next);
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

    #[cfg(test)]
    fn effective_track_output_notes(&self, track_index: usize) -> Vec<MidiNote> {
        let Some(track) = self.project.tracks.get(track_index) else {
            return Vec::new();
        };
        let mut visited = vec![false; self.project.tracks.len()];
        let notes = self.effective_track_pre_output_playback_notes_recursive(
            track_index,
            self.transport_ticks,
            self.project.transport.ppqn as u64,
            &mut visited,
        );
        transform_notes(
            &notes,
            &track.midi_fx.output_fx,
            self.project.global_harmony.root,
        )
    }

    fn poll_midi_input(&mut self) {
        let events = self.midi_input.drain_events();
        for event in events {
            self.handle_midi_input_event(event);
        }
    }

    fn handle_midi_input_event(&mut self, event: MidiInputEvent) {
        if self.capture_direct_mapping_input(&event) {
            return;
        }

        if self.capture_mapping_midi_learn(&event) {
            return;
        }

        let mapping_actions = self.resolve_midi_mapping_actions(&event);
        for action in mapping_actions {
            let _ = self.apply_action_with_source(action, ActionSource::Midi);
        }

        let matching_tracks: Vec<usize> = self
            .project
            .tracks
            .iter()
            .enumerate()
            .filter(|(_, track)| {
                track.routing.input_port.as_ref() == Some(&event.port)
                    && match track.routing.input_channel {
                        MidiChannelFilter::Omni => true,
                        MidiChannelFilter::Channel(channel) => channel == event.channel,
                    }
            })
            .map(|(index, _)| index)
            .collect();

        for index in matching_tracks {
            let input_ticks = self
                .project
                .tracks
                .get(index)
                .map(|track| self.live_input_event_ticks(track))
                .unwrap_or(self.playhead_ticks);

            let Some(track_view) = self.project.tracks.get(index) else {
                continue;
            };

            let (
                record_mode,
                monitor_input_fx,
                passthrough,
                output_port,
                output_channel,
                input_chain,
                output_chain,
            ) = (
                track_view.midi_fx.record_input_fx_mode,
                track_view.midi_fx.monitor_input_fx,
                track_view.state.passthrough,
                track_view.routing.output_port.clone(),
                track_view.routing.output_channel,
                track_view.midi_fx.input_fx.clone(),
                track_view.midi_fx.output_fx.clone(),
            );

            match event.message {
                MidiInputMessage::NoteOn { pitch, velocity } => {
                    let raw_event = LiveMidiFxEvent::NoteOn { pitch, velocity };
                    let (post_input_events, monitor_source_events) = self.monitor_source_events(
                        index,
                        raw_event,
                        &input_chain,
                        monitor_input_fx,
                        input_ticks,
                    );
                    if let Some(track) = self.project.tracks.get_mut(index) {
                        if track.active_take.is_some() {
                            let record_events =
                                if record_mode == crate::midi_fx::RecordInputFxMode::PostInputFx {
                                    post_input_events.clone()
                                } else {
                                    vec![LiveMidiFxEvent::NoteOn { pitch, velocity }]
                                };
                            for record_event in record_events {
                                if let LiveMidiFxEvent::NoteOn { pitch, velocity } = record_event {
                                    track.record_note_on(pitch, velocity, input_ticks);
                                }
                            }
                        }
                    }
                    self.propagate_live_clone_events(index, &post_input_events);
                    if passthrough {
                        self.send_live_monitor_events(
                            index,
                            &output_chain,
                            output_port.as_ref(),
                            output_channel,
                            monitor_source_events,
                            input_ticks,
                        );
                    }
                }
                MidiInputMessage::NoteOff { pitch } => {
                    let raw_event = LiveMidiFxEvent::NoteOff { pitch };
                    let (post_input_events, monitor_source_events) = self.monitor_source_events(
                        index,
                        raw_event,
                        &input_chain,
                        monitor_input_fx,
                        input_ticks,
                    );
                    if let Some(track) = self.project.tracks.get_mut(index) {
                        if track.active_take.is_some() {
                            let record_events =
                                if record_mode == crate::midi_fx::RecordInputFxMode::PostInputFx {
                                    post_input_events.clone()
                                } else {
                                    vec![LiveMidiFxEvent::NoteOff { pitch }]
                                };
                            for record_event in record_events {
                                if let LiveMidiFxEvent::NoteOff { pitch } = record_event {
                                    track.record_note_off(pitch, input_ticks);
                                }
                            }
                        }
                    }
                    self.propagate_live_clone_events(index, &post_input_events);
                    if passthrough {
                        self.send_live_monitor_events(
                            index,
                            &output_chain,
                            output_port.as_ref(),
                            output_channel,
                            monitor_source_events,
                            input_ticks,
                        );
                    }
                }
                MidiInputMessage::ControlChange { .. } => {}
            }
        }
    }

    fn capture_mapping_midi_learn(&mut self, event: &MidiInputEvent) -> bool {
        if self.page_state.current_page != AppPage::Mappings
            || self.page_state.mapping_mode != MappingPageMode::Write
            || !self.page_state.mapping_midi_learn_armed
        {
            return false;
        }

        let index = self.page_state.selected_mapping_index;
        let Some(entry) = self.mappings.get_mut(index) else {
            return false;
        };

        entry.source_kind = MappingSourceKind::Midi;
        entry.source_device_label = event.port.name.clone();
        entry.source_label = midi_learn_label(event);
        entry.enabled = true;
        self.page_state.mapping_midi_learn_armed = false;
        true
    }

    fn resolve_midi_mapping_actions(&self, event: &MidiInputEvent) -> Vec<AppAction> {
        self.mappings
            .iter()
            .filter(|entry| midi_mapping_matches_event(entry, event))
            .flat_map(|entry| mapping_entry_to_actions(entry, event))
            .collect()
    }

    fn resolve_key_mapping_actions(&self, source_label: &str) -> Vec<AppAction> {
        self.mappings
            .iter()
            .filter(|entry| {
                entry.enabled
                    && entry.source_kind == MappingSourceKind::Key
                    && entry.source_label == source_label
            })
            .flat_map(mapping_entry_key_actions)
            .collect()
    }

    fn dispatch_midi_notes(&mut self, previous_ticks: u64, advanced_ticks: u64) {
        if advanced_ticks == 0 {
            return;
        }

        let track_events: Vec<(
            Option<MidiPortRef>,
            u8,
            Vec<(u64, bool, u8, u8)>,
            Vec<(u64, bool, u8, u8)>,
        )> = self
            .project
            .tracks
            .iter()
            .enumerate()
            .map(|(track_index, track)| {
                let channel = track.routing.output_channel.unwrap_or(1).clamp(1, 16);
                let port = track.routing.output_port.clone();
                let output_lookback = playback_timing_lookback_ticks(&track.midi_fx.output_fx);
                let lookback_padding =
                    output_lookback.saturating_add(u64::from(output_lookback > 0));
                let source_previous_ticks = previous_ticks.saturating_sub(lookback_padding);
                let source_advanced_ticks = advanced_ticks.saturating_add(lookback_padding);
                let mut visited = vec![false; self.project.tracks.len()];
                let mut pre_output_notes = self
                    .effective_track_pre_output_playback_notes_recursive(
                        track_index,
                        source_previous_ticks,
                        source_advanced_ticks,
                        &mut visited,
                    );
                let preview_notes = track.playback_preview_notes(
                    self.project.transport,
                    self.transport_ticks,
                    self.record_context(track),
                );
                let preview_occurrences = scheduled_note_occurrences(
                    track,
                    &preview_notes,
                    source_previous_ticks,
                    source_advanced_ticks,
                    self.playback_loop_range_for_track(track),
                );
                pre_output_notes.extend(preview_occurrences);
                let clone_notes = self.effective_track_clone_playback_notes_recursive(
                    track_index,
                    previous_ticks,
                    advanced_ticks,
                    &mut visited,
                );
                let transformed_notes = transform_notes(
                    &pre_output_notes,
                    &track.midi_fx.output_fx,
                    self.project.global_harmony.root,
                );
                let events = occurrence_note_events(
                    track,
                    &transformed_notes,
                    previous_ticks,
                    advanced_ticks,
                );
                let record_events = if track.active_take.is_some()
                    && track.midi_fx.record_input_fx_mode
                        == crate::midi_fx::RecordInputFxMode::PostInputFx
                {
                    occurrence_note_events_unmuted(&clone_notes, previous_ticks, advanced_ticks)
                } else {
                    Vec::new()
                };
                (port, channel, events, record_events)
            })
            .collect();

        for (track_index, (port, channel, events, record_events)) in
            track_events.into_iter().enumerate()
        {
            if let Some(track) = self.project.tracks.get_mut(track_index) {
                for (event_ticks, note_on, pitch, velocity) in &record_events {
                    if *note_on {
                        track.record_note_on(*pitch, *velocity, *event_ticks);
                    } else {
                        track.record_note_off(*pitch, *event_ticks);
                    }
                }
            }

            let Some(port) = port else {
                continue;
            };

            let mut refresh_needed = false;
            for (_, note_on, pitch, velocity) in events {
                let result = if note_on {
                    self.midi_output
                        .send_note_on(&port, channel, pitch, velocity)
                } else {
                    self.midi_output.send_note_off(&port, channel, pitch)
                };
                if result.is_err() {
                    refresh_needed = true;
                }
            }
            if refresh_needed {
                self.refresh_midi_devices_now();
            }
        }
    }

    fn silence_all_tracks(&mut self) {
        let ports_and_channels: Vec<(MidiPortRef, u8)> = self
            .project
            .tracks
            .iter()
            .filter_map(|track| {
                track
                    .routing
                    .output_port
                    .clone()
                    .zip(track.routing.output_channel)
            })
            .collect();

        for (port, channel) in ports_and_channels {
            if self.midi_output.send_all_notes_off(&port, channel).is_err() {
                self.refresh_midi_devices_now();
            }
        }
    }

    fn silence_tracks_for_loop_change(&mut self) {
        self.silence_all_tracks();
    }

    fn handle_timeline_fx_configuration_changed(&mut self) {
        self.silence_all_tracks();
        let current_ticks = if self.project.transport.playing {
            self.transport_ticks
        } else {
            self.live_fx_ticks
        };
        self.reset_live_fx_timing(current_ticks);
    }

    fn reset_live_fx_timing(&mut self, current_ticks: u64) {
        for state in &mut self.input_fx_live_states {
            reset_live_fx_timing(state, current_ticks);
        }
        for state in &mut self.output_fx_live_states {
            reset_live_fx_timing(state, current_ticks);
        }
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

        let overview_badge = Rect::new(content_bounds.x + 200, content_bounds.y + 8, 188, 16);
        let learn_badge = Rect::new(content_bounds.x + 392, content_bounds.y + 8, 136, 16);
        let direct_badge = Rect::new(content_bounds.x + 532, content_bounds.y + 8, 154, 16);
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

        for visible_index in 0..visible_rows {
            let index = start_index + visible_index;
            if index >= self.mappings.len() {
                break;
            }
            let row = Rect::new(
                list_bounds.x,
                list_bounds.y + visible_index as i32 * stride,
                list_bounds.width(),
                row_height as u32,
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

    fn discoverability_target_at(&self, x: i32, y: i32) -> Option<DiscoverabilityTarget> {
        if self.overlay_state.active == Some(AppOverlay::MappingsQuickView) {
            return None;
        }
        let surface = crate::ui::surface_rect(self.viewport_size.0, self.viewport_size.1);
        let inset = crate::ui::inset_rect(surface, 24, 24).ok()?;
        let (_, content_bounds, _) = self.page_frame_layout(inset).ok()?;

        let targets =
            page_discoverability_targets(self.page_state.current_page, self, content_bounds);

        targets
            .into_iter()
            .find_map(|(rect, target)| rect_contains(rect, x, y).then_some(target))
    }

    fn track_discoverability_targets(
        &self,
        layout: TimelineTrackLayout,
        track: &Track,
    ) -> Vec<(Rect, DiscoverabilityTarget)> {
        let mut targets = Vec::new();
        let status_rect = layout.status_rect;
        let label_rect = layout.full_label_rect;
        let detail_label_rect = layout.detail_label_rect;
        if track.recording_view == RecordingView::Stacked {
            let (left_rect, right_rect) = self.recording_view_scroll_control_rects(label_rect);
            targets.push((
                left_rect,
                DiscoverabilityTarget {
                    action: AppAction::SelectPreviousRecordingClip,
                    display_scope: Some("Active Track"),
                    allowed_mapping_scopes: &["Active Track"],
                    overlay_slot: None,
                },
            ));
            targets.push((
                right_rect,
                DiscoverabilityTarget {
                    action: AppAction::SelectNextRecordingClip,
                    display_scope: Some("Active Track"),
                    allowed_mapping_scopes: &["Active Track"],
                    overlay_slot: None,
                },
            ));
        }
        targets.push((
            self.recording_view_chip_rect(label_rect),
            DiscoverabilityTarget {
                action: AppAction::ToggleCurrentTrackRecordingView,
                display_scope: Some("Active Track"),
                allowed_mapping_scopes: &["Active Track"],
                overlay_slot: None,
            },
        ));
        if track.selected_recording_clip().is_some() {
            let (mute_rect, delete_rect) = self.recording_clip_control_rects(label_rect);
            targets.push((
                mute_rect,
                DiscoverabilityTarget {
                    action: AppAction::ToggleSelectedRecordingClipMute,
                    display_scope: Some("Active Track"),
                    allowed_mapping_scopes: &["Active Track"],
                    overlay_slot: None,
                },
            ));
            targets.push((
                delete_rect,
                DiscoverabilityTarget {
                    action: AppAction::DeleteSelectedRecordingClip,
                    display_scope: Some("Active Track"),
                    allowed_mapping_scopes: &["Active Track"],
                    overlay_slot: None,
                },
            ));
        }
        for content_rect in [layout.full_content_rect, layout.detail_content_rect] {
            for lane in self.recording_lane_layouts(content_rect, track) {
                if let Some(clip_id) = lane.clip_id {
                    targets.push((
                        lane.rect,
                        DiscoverabilityTarget {
                            action: AppAction::SelectRecordingClip(clip_id),
                            display_scope: Some("Active Track"),
                            allowed_mapping_scopes: &["Active Track"],
                            overlay_slot: None,
                        },
                    ));
                }
            }
        }
        for indicator in crate::ui::track_indicators(status_rect) {
            if let Some(target) = track_indicator_target(indicator.kind, Some(indicator.rect)) {
                targets.push((
                    Rect::new(
                        indicator.rect.x - 2,
                        indicator.rect.y - 2,
                        indicator.rect.width().saturating_add(4),
                        indicator.rect.height().saturating_add(4),
                    ),
                    target,
                ));
            }
        }

        targets.push((
            self.track_passthrough_button_rect(label_rect),
            DiscoverabilityTarget {
                action: AppAction::ToggleCurrentTrackPassthrough,
                display_scope: Some("Active Track"),
                allowed_mapping_scopes: &["Active Track"],
                overlay_slot: None,
            },
        ));
        targets.extend(self.timeline_fx_discoverability_targets_for_track(
            track,
            TimelineContext::OutputFx,
            layout.output_fx_rect,
        ));
        targets.extend(self.timeline_fx_discoverability_targets_for_track(
            track,
            TimelineContext::InputFx,
            layout.input_fx_rect,
        ));
        for (slot_index, slot_rect) in self.stored_loop_slot_rects(detail_label_rect) {
            if let Some(action) = stored_loop_slot_recall_action(slot_index) {
                targets.push((
                    slot_rect,
                    DiscoverabilityTarget {
                        action,
                        display_scope: Some("Active Track"),
                        allowed_mapping_scopes: &["Active Track"],
                        overlay_slot: Some(slot_rect),
                    },
                ));
            }
        }
        targets.push((
            crate::ui::detail_badge_rect(detail_label_rect),
            DiscoverabilityTarget {
                action: AppAction::ToggleCurrentTrackLoop,
                display_scope: Some("Active Track"),
                allowed_mapping_scopes: &["Active Track"],
                overlay_slot: None,
            },
        ));

        targets
    }

    fn timeline_fx_discoverability_targets_for_track(
        &self,
        track: &Track,
        context: TimelineContext,
        band_rect: Rect,
    ) -> Vec<(Rect, DiscoverabilityTarget)> {
        let Some(chain_kind) = context.chain_kind() else {
            return Vec::new();
        };
        let displayed_rows = self.displayed_timeline_fx_slot_indices_for_track(track, chain_kind);
        let selected_row = (self
            .project
            .active_track()
            .is_some_and(|active| std::ptr::eq(active, track))
            && self.page_state.selected_timeline_context == context)
            .then(|| self.selected_timeline_fx_row(chain_kind));
        let chain = self.fx_chain(track, chain_kind);
        let layouts = self.timeline_fx_row_layouts(band_rect, &displayed_rows, chain, selected_row);
        let rows = displayed_rows;
        let mut targets = Vec::new();
        for (row, layout) in rows.iter().zip(layouts.into_iter()) {
            if row.is_none() {
                targets.push((
                    layout.row,
                    DiscoverabilityTarget {
                        action: AppAction::AddSelectedTimelineFx,
                        display_scope: Some("Active Track"),
                        allowed_mapping_scopes: &["Active Track"],
                        overlay_slot: None,
                    },
                ));
                continue;
            }
            targets.push((
                layout.enabled,
                DiscoverabilityTarget {
                    action: AppAction::ToggleSelectedTimelineFx,
                    display_scope: Some("Active Track"),
                    allowed_mapping_scopes: &["Active Track"],
                    overlay_slot: None,
                },
            ));
            targets.push((
                layout.kind,
                DiscoverabilityTarget {
                    action: AppAction::CycleSelectedTimelineFxKind,
                    display_scope: Some("Active Track"),
                    allowed_mapping_scopes: &["Active Track"],
                    overlay_slot: None,
                },
            ));
            targets.push((
                layout.param_primary,
                DiscoverabilityTarget {
                    action: AppAction::AdjustSelectedTimelineFxPrimary,
                    display_scope: Some("Active Track"),
                    allowed_mapping_scopes: &["Active Track"],
                    overlay_slot: None,
                },
            ));
            targets.push((
                layout.param_secondary,
                DiscoverabilityTarget {
                    action: AppAction::AdjustSelectedTimelineFxSecondary,
                    display_scope: Some("Active Track"),
                    allowed_mapping_scopes: &["Active Track"],
                    overlay_slot: None,
                },
            ));
            targets.push((
                layout.overflow,
                DiscoverabilityTarget {
                    action: AppAction::ScrollSelectedTimelineFxWindow,
                    display_scope: Some("Active Track"),
                    allowed_mapping_scopes: &["Active Track"],
                    overlay_slot: None,
                },
            ));
            targets.push((
                layout.move_up,
                DiscoverabilityTarget {
                    action: AppAction::MoveSelectedTimelineFxUp,
                    display_scope: Some("Active Track"),
                    allowed_mapping_scopes: &["Active Track"],
                    overlay_slot: None,
                },
            ));
            targets.push((
                layout.move_down,
                DiscoverabilityTarget {
                    action: AppAction::MoveSelectedTimelineFxDown,
                    display_scope: Some("Active Track"),
                    allowed_mapping_scopes: &["Active Track"],
                    overlay_slot: None,
                },
            ));
            targets.push((
                layout.delete,
                DiscoverabilityTarget {
                    action: AppAction::DeleteSelectedTimelineFx,
                    display_scope: Some("Active Track"),
                    allowed_mapping_scopes: &["Active Track"],
                    overlay_slot: None,
                },
            ));
        }
        targets
    }

    pub(crate) fn routing_discoverability_targets(
        &self,
        content_bounds: Rect,
    ) -> Vec<(Rect, DiscoverabilityTarget)> {
        let mut targets = Vec::new();
        let inner = crate::ui::inset_rect(content_bounds, 12, 32).expect("routing inner");
        let (header, body) = crate::ui::split_top_strip(inner, 48, 10).expect("routing layout");
        targets.push((
            Rect::new(
                header.x + 106,
                header.y + 8,
                92,
                header.height().saturating_sub(16),
            ),
            DiscoverabilityTarget {
                action: AppAction::ToggleCurrentTrackPassthrough,
                display_scope: Some("Active Track"),
                allowed_mapping_scopes: &["Active Track"],
                overlay_slot: None,
            },
        ));

        for (field, row) in self.routing_field_rects(body) {
            if field != RoutingField::Passthrough {
                continue;
            }
            let control_height = row.height().saturating_sub(20).max(10);
            let control_y = row.y + row.height() as i32 - control_height as i32 - 6;
            let value = Rect::new(
                row.x + 8,
                control_y,
                row.width().saturating_sub(64),
                control_height,
            );
            targets.push((
                value,
                DiscoverabilityTarget {
                    action: AppAction::ToggleCurrentTrackPassthrough,
                    display_scope: Some("Active Track"),
                    allowed_mapping_scopes: &["Active Track"],
                    overlay_slot: None,
                },
            ));
        }

        targets
    }

    fn apply_action_with_source(
        &mut self,
        action: AppAction,
        source: crate::actions::ActionSource,
    ) -> AppControl {
        self.status_state.hovered_target = None;
        self.direct_mapping_state.status_message = None;
        self.status_state.last_action = Some(LastActionStatus { action, source });
        self.apply_action(action)
    }

    fn transport_top_chip_specs(&self) -> Vec<TransportChipSpec> {
        vec![
            TransportChipSpec {
                label: format!("Play {}", on_off(self.project.transport.playing)),
                action: Some(AppAction::TogglePlayback),
                fill: if self.project.transport.playing {
                    Color::RGB(96, 162, 122)
                } else {
                    Color::RGB(74, 84, 102)
                },
            },
            TransportChipSpec {
                label: format!("Record {}", on_off(self.project.transport.recording)),
                action: Some(AppAction::ToggleRecording),
                fill: if self.project.transport.recording {
                    Color::RGB(180, 76, 76)
                } else {
                    Color::RGB(88, 78, 82)
                },
            },
            TransportChipSpec {
                label: format!("Mode {}", self.project.transport.record_mode.label()),
                action: Some(AppAction::CycleRecordMode),
                fill: Color::RGB(76, 94, 136),
            },
        ]
    }

    fn transport_bottom_chip_specs(&self) -> Vec<TransportChipSpec> {
        vec![
            TransportChipSpec {
                label: format!(
                    "Wrap {}",
                    if self.project.transport.loop_recording_extends_clip {
                        "Extend"
                    } else {
                        "Clamp"
                    }
                ),
                action: Some(AppAction::ToggleLoopRecordingExtension),
                fill: if self.project.transport.loop_recording_extends_clip {
                    Color::RGB(126, 106, 60)
                } else {
                    Color::RGB(96, 82, 70)
                },
            },
            TransportChipSpec {
                label: format!("Song Loop {}", on_off(self.project.transport.loop_enabled)),
                action: Some(AppAction::ToggleGlobalLoop),
                fill: Color::RGB(116, 96, 54),
            },
            TransportChipSpec {
                label: format!("Tempo {}", self.project.transport.tempo_bpm),
                action: None,
                fill: Color::RGB(70, 100, 120),
            },
            TransportChipSpec {
                label: format!("Harmony {}", note_name(self.project.global_harmony.root)),
                action: Some(AppAction::CycleGlobalHarmonyRoot),
                fill: Color::RGB(88, 82, 124),
            },
            TransportChipSpec {
                label: format!("NoteAdd {}", on_off(self.note_additive_select_held)),
                action: None,
                fill: if self.note_additive_select_held {
                    Color::RGB(88, 130, 176)
                } else {
                    Color::RGB(62, 76, 94)
                },
            },
        ]
    }

    fn transport_link_chip_specs(&self) -> Vec<TransportChipSpec> {
        vec![
            TransportChipSpec {
                label: format!("Link {}", on_off(self.project.transport.link_enabled)),
                action: Some(AppAction::ToggleLinkEnabled),
                fill: if self.project.transport.link_enabled {
                    Color::RGB(74, 122, 144)
                } else {
                    Color::RGB(68, 76, 92)
                },
            },
            TransportChipSpec {
                label: format!(
                    "Start/Stop {}",
                    on_off(self.project.transport.link_start_stop_sync)
                ),
                action: Some(AppAction::ToggleLinkStartStopSync),
                fill: Color::RGB(82, 98, 130),
            },
        ]
    }

    fn transport_status_chip_specs(&self) -> Vec<TransportChipSpec> {
        vec![
            TransportChipSpec {
                label: format!(
                    "LaunchQ {}",
                    on_off(self.project.transport.stored_loop_recall_quantized)
                ),
                action: Some(AppAction::ToggleStoredLoopRecallQuantize),
                fill: if self.project.transport.stored_loop_recall_quantized {
                    Color::RGB(102, 124, 86)
                } else {
                    Color::RGB(72, 88, 110)
                },
            },
            TransportChipSpec {
                label: format!(
                    "Launch {}",
                    launch_quantize_label(self.project.transport.stored_loop_launch_quantize)
                ),
                action: Some(AppAction::CycleStoredLoopLaunchQuantize),
                fill: Color::RGB(78, 96, 122),
            },
            TransportChipSpec {
                label: format!("Quant {}", quantize_label(self.project.transport.quantize)),
                action: None,
                fill: Color::RGB(70, 86, 108),
            },
            TransportChipSpec {
                label: format!("Peers {}", self.link_snapshot.peers),
                action: None,
                fill: Color::RGB(66, 80, 102),
            },
        ]
    }

    fn transport_right_panel_width(&self, bounds: Rect) -> u32 {
        let top_row = chip_row_width(&self.transport_link_chip_specs())
            .saturating_add(96)
            .saturating_add(12);
        let bottom_row = chip_row_width(&self.transport_status_chip_specs()).saturating_add(12);
        let desired = top_row.max(bottom_row).max(236);
        let max_allowed = bounds.width().saturating_sub(220).max(236);
        desired.min(max_allowed)
    }
}

fn logical_viewport_size(output_size: (u32, u32), display_scale: f32) -> (u32, u32) {
    let scale = display_scale.max(1.0);
    let logical_width = (output_size.0 as f32 / scale).round().max(1.0) as u32;
    let logical_height = (output_size.1 as f32 / scale).round().max(1.0) as u32;
    (logical_width, logical_height)
}

fn active_draw_size(canvas_output_size: (u32, u32), viewport_size: (u32, u32)) -> (u32, u32) {
    if viewport_size.0 > 0 && viewport_size.1 > 0 {
        viewport_size
    } else {
        canvas_output_size
    }
}

fn effective_ui_scale(display_scale: f32, override_scale: Option<f32>) -> f32 {
    override_scale.unwrap_or(display_scale).max(1.0)
}

fn should_interpolate_window_scale(mode: UiScalingMode, scale_x: f32, scale_y: f32) -> bool {
    match mode {
        UiScalingMode::Auto => {
            has_fractional_scale_component(scale_x) || has_fractional_scale_component(scale_y)
        }
        UiScalingMode::Nearest => false,
        UiScalingMode::Linear => true,
    }
}

fn has_fractional_scale_component(scale: f32) -> bool {
    (scale - scale.round()).abs() > 0.001
}

fn scheduled_note_occurrences(
    track: &Track,
    notes: &[MidiNote],
    previous_ticks: u64,
    advanced_ticks: u64,
    loop_range: Option<crate::timeline::LoopRegion>,
) -> Vec<MidiNote> {
    if advanced_ticks == 0 || track.state.muted {
        return Vec::new();
    }

    let segments = loop_range
        .map(|range| ranged_segments(previous_ticks, advanced_ticks, range))
        .unwrap_or_else(|| {
            vec![(
                previous_ticks,
                previous_ticks.saturating_add(advanced_ticks),
            )]
        });

    let mut occurrences = Vec::new();
    let mut transport_cursor = previous_ticks;
    for (segment_start, segment_end) in segments {
        for note in notes {
            if track.recording_clip_is_muted(note.recording_clip_id) {
                continue;
            }
            let overlap_start = note.start_ticks.max(segment_start);
            let overlap_end = note.end_ticks().min(segment_end);
            if overlap_start >= overlap_end {
                continue;
            }
            occurrences.push(MidiNote {
                pitch: note.pitch,
                start_ticks: if note.start_ticks >= segment_start {
                    transport_cursor.saturating_add(note.start_ticks.saturating_sub(segment_start))
                } else {
                    transport_cursor.saturating_sub(segment_start.saturating_sub(note.start_ticks))
                },
                length_ticks: note.length_ticks,
                velocity: note.velocity,
                recording_clip_id: note.recording_clip_id,
            });
        }
        transport_cursor =
            transport_cursor.saturating_add(segment_end.saturating_sub(segment_start));
    }

    occurrences.sort_by_key(|note| (note.start_ticks, note.pitch, note.length_ticks));
    occurrences
}

fn occurrence_note_events(
    track: &Track,
    notes: &[MidiNote],
    previous_ticks: u64,
    advanced_ticks: u64,
) -> Vec<(u64, bool, u8, u8)> {
    if track.state.muted {
        Vec::new()
    } else {
        occurrence_note_events_unmuted(notes, previous_ticks, advanced_ticks)
    }
}

fn occurrence_note_events_unmuted(
    notes: &[MidiNote],
    previous_ticks: u64,
    advanced_ticks: u64,
) -> Vec<(u64, bool, u8, u8)> {
    let end_ticks = previous_ticks.saturating_add(advanced_ticks);
    let mut events = Vec::with_capacity(notes.len().saturating_mul(2));
    for note in notes {
        if note.start_ticks >= previous_ticks && note.start_ticks < end_ticks {
            events.push((note.start_ticks, true, note.pitch, note.velocity));
        }
        let note_end = note.end_ticks();
        if note_end >= previous_ticks && note_end < end_ticks {
            events.push((note_end, false, note.pitch, note.velocity));
        }
    }
    events.sort_by_key(|event| (event.0, event.1));
    events
}

fn ranged_segments(
    previous_ticks: u64,
    advanced_ticks: u64,
    range: crate::timeline::LoopRegion,
) -> Vec<(u64, u64)> {
    if range.length_ticks == 0 || advanced_ticks == 0 {
        return Vec::new();
    }

    let mut segments = Vec::new();
    let mut remaining = advanced_ticks;
    let mut cursor = range.start_ticks + (previous_ticks % range.length_ticks);
    let end = range.end_ticks();

    while remaining > 0 {
        let next_boundary = end.min(cursor.saturating_add(remaining));
        segments.push((cursor, next_boundary));
        let consumed = next_boundary.saturating_sub(cursor);
        if consumed >= remaining {
            break;
        }

        remaining = remaining.saturating_sub(consumed);
        cursor = range.start_ticks;
    }

    segments
}

fn indexed_notes(
    track: &Track,
    recording_clip_id: Option<u64>,
) -> Vec<(usize, crate::project::MidiNote)> {
    track
        .midi_notes
        .iter()
        .copied()
        .enumerate()
        .filter(|(_, note)| match recording_clip_id {
            Some(clip_id) => note.recording_clip_id == Some(clip_id),
            None => note.recording_clip_id.is_none(),
        })
        .collect()
}

fn indexed_all_notes(track: &Track) -> Vec<(usize, crate::project::MidiNote)> {
    track.midi_notes.iter().copied().enumerate().collect()
}

fn cycle_optional_port(
    current: Option<&MidiPortRef>,
    ports: &[MidiPortRef],
    delta: i32,
) -> Option<MidiPortRef> {
    if ports.is_empty() {
        return None;
    }

    let option_count = ports.len() as i32 + 1;
    let current_index = current
        .and_then(|port| ports.iter().position(|candidate| candidate == port))
        .map(|index| index as i32 + 1)
        .unwrap_or(0);
    let next_index = (current_index + delta).rem_euclid(option_count);
    if next_index == 0 {
        None
    } else {
        ports.get((next_index - 1) as usize).cloned()
    }
}

fn cycle_input_channel(current: MidiChannelFilter, delta: i32) -> MidiChannelFilter {
    let current_index = match current {
        MidiChannelFilter::Omni => 0,
        MidiChannelFilter::Channel(channel) => i32::from(channel.clamp(1, 16)),
    };
    let next_index = (current_index + delta).rem_euclid(17);
    if next_index == 0 {
        MidiChannelFilter::Omni
    } else {
        MidiChannelFilter::Channel(next_index as u8)
    }
}

fn cycle_output_channel(current: Option<u8>, delta: i32) -> Option<u8> {
    let current_index = current
        .map(|value| i32::from(value.clamp(1, 16)))
        .unwrap_or(0);
    let next_index = (current_index + delta).rem_euclid(17);
    if next_index == 0 {
        None
    } else {
        Some(next_index as u8)
    }
}

fn transport_strip_height() -> u32 {
    34
}

fn recall_stored_loop_slot_index(action: AppAction) -> Option<usize> {
    match action {
        AppAction::RecallStoredLoopSlot1 => Some(0),
        AppAction::RecallStoredLoopSlot2 => Some(1),
        AppAction::RecallStoredLoopSlot3 => Some(2),
        AppAction::RecallStoredLoopSlot4 => Some(3),
        AppAction::RecallStoredLoopSlot5 => Some(4),
        AppAction::RecallStoredLoopSlot6 => Some(5),
        AppAction::RecallStoredLoopSlot7 => Some(6),
        AppAction::RecallStoredLoopSlot8 => Some(7),
        _ => None,
    }
}

fn store_stored_loop_slot_index(action: AppAction) -> Option<usize> {
    match action {
        AppAction::StoreCurrentLoopToSlot1 => Some(0),
        AppAction::StoreCurrentLoopToSlot2 => Some(1),
        AppAction::StoreCurrentLoopToSlot3 => Some(2),
        AppAction::StoreCurrentLoopToSlot4 => Some(3),
        AppAction::StoreCurrentLoopToSlot5 => Some(4),
        AppAction::StoreCurrentLoopToSlot6 => Some(5),
        AppAction::StoreCurrentLoopToSlot7 => Some(6),
        AppAction::StoreCurrentLoopToSlot8 => Some(7),
        _ => None,
    }
}

fn clear_stored_loop_slot_index(action: AppAction) -> Option<usize> {
    match action {
        AppAction::ClearStoredLoopSlot1 => Some(0),
        AppAction::ClearStoredLoopSlot2 => Some(1),
        AppAction::ClearStoredLoopSlot3 => Some(2),
        AppAction::ClearStoredLoopSlot4 => Some(3),
        AppAction::ClearStoredLoopSlot5 => Some(4),
        AppAction::ClearStoredLoopSlot6 => Some(5),
        AppAction::ClearStoredLoopSlot7 => Some(6),
        AppAction::ClearStoredLoopSlot8 => Some(7),
        _ => None,
    }
}

fn stored_loop_slot_recall_action(slot_index: usize) -> Option<AppAction> {
    match slot_index {
        0 => Some(AppAction::RecallStoredLoopSlot1),
        1 => Some(AppAction::RecallStoredLoopSlot2),
        2 => Some(AppAction::RecallStoredLoopSlot3),
        3 => Some(AppAction::RecallStoredLoopSlot4),
        4 => Some(AppAction::RecallStoredLoopSlot5),
        5 => Some(AppAction::RecallStoredLoopSlot6),
        6 => Some(AppAction::RecallStoredLoopSlot7),
        7 => Some(AppAction::RecallStoredLoopSlot8),
        _ => None,
    }
}

fn stored_loop_slot_color(slot_index: usize) -> Color {
    match slot_index % STORED_LOOP_SLOT_COUNT {
        0 => Color::RGB(214, 124, 118),
        1 => Color::RGB(214, 176, 98),
        2 => Color::RGB(184, 206, 108),
        3 => Color::RGB(114, 198, 174),
        4 => Color::RGB(114, 168, 214),
        5 => Color::RGB(144, 138, 214),
        6 => Color::RGB(204, 132, 206),
        _ => Color::RGB(210, 144, 164),
    }
}

fn loop_regions_intersect(a: crate::timeline::LoopRegion, b: crate::timeline::LoopRegion) -> bool {
    a.start_ticks < b.end_ticks() && a.end_ticks() > b.start_ticks
}

fn interlaced_color_at(colors: &[Color], pixel_index: usize) -> Option<Color> {
    (!colors.is_empty()).then_some(colors[pixel_index % colors.len()])
}

fn rects_overlap(a: Rect, b: Rect) -> bool {
    a.x < b.x + b.width() as i32
        && a.x + a.width() as i32 > b.x
        && a.y < b.y + b.height() as i32
        && a.y + a.height() as i32 > b.y
}

fn draw_loop_label_underline<T: RenderTarget>(
    canvas: &mut Canvas<T>,
    label: &str,
    label_rect: Rect,
    content_rect: Rect,
    viewport_size: (u32, u32),
    fallback_bg: Color,
) -> Result<(), String> {
    let underline_width = crate::ui::text_width(label, 1)
        .min(label_rect.width())
        .max(1);
    let underline_x =
        (label_rect.x + ((label_rect.width() as i32 - underline_width as i32) / 2).max(0) - 1)
            .max(content_rect.x);
    let underline_y = (label_rect.y + label_rect.height() as i32 + 1)
        .min(content_rect.y + content_rect.height() as i32 - 1);
    let underline_rect = Rect::new(underline_x, underline_y, underline_width, 1);
    let underline_readback = readback_rect_rgba(canvas, underline_rect, viewport_size);
    for offset in 0..underline_width as i32 {
        let px = underline_x + offset;
        let bg = readback_color_at(&underline_readback, px, underline_y).unwrap_or(fallback_bg);
        canvas.set_draw_color(Color::RGB(255 - bg.r, 255 - bg.g, 255 - bg.b));
        canvas
            .fill_rect(Rect::new(px, underline_y, 1, 1))
            .map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn displayed_track_fx_band_height(chain: &[Option<MidiFxSlot>]) -> i32 {
    let line_height = 8_i32;
    let line_gap = 2_i32;
    let vertical_padding = 4_i32;
    let active = chain.iter().flatten().count();
    let show_add = active < chain.len().max(MIDI_FX_SLOT_COUNT);
    let line_count = (active + usize::from(show_add)).max(1) as i32;
    vertical_padding + line_count * line_height + (line_count - 1) * line_gap
}

fn timeline_subcolumn_label_rect(lane: Rect, flow: TimelineFlow) -> Rect {
    match flow {
        TimelineFlow::DownwardColumns => Rect::new(lane.x, lane.y, lane.width(), 24),
        TimelineFlow::AcrossRows => Rect::new(lane.x, lane.y, 56, lane.height().saturating_sub(14)),
    }
}

fn timeline_subcolumn_content_rect(lane: Rect, flow: TimelineFlow) -> Rect {
    match flow {
        TimelineFlow::DownwardColumns => Rect::new(
            lane.x,
            lane.y + 24,
            lane.width(),
            lane.height().saturating_sub(24),
        ),
        TimelineFlow::AcrossRows => Rect::new(
            lane.x + 56,
            lane.y,
            lane.width().saturating_sub(56),
            lane.height(),
        ),
    }
}

fn centered_text_rect(rect: Rect) -> Rect {
    Rect::new(
        rect.x,
        rect.y + ((rect.height() as i32 - 8) / 2).max(0),
        rect.width(),
        8,
    )
}

fn contrasting_text_color(fill: Color) -> Color {
    let brightness = u32::from(fill.r) * 299 + u32::from(fill.g) * 587 + u32::from(fill.b) * 114;
    if brightness / 1000 < 140 {
        Color::RGB(244, 244, 236)
    } else {
        Color::RGB(24, 28, 36)
    }
}

fn routing_field_short_label(field: RoutingField) -> &'static str {
    match field {
        RoutingField::InputDevice => "Input Device",
        RoutingField::InputChannel => "Input Chan",
        RoutingField::OutputDevice => "Output Device",
        RoutingField::OutputChannel => "Output Chan",
        RoutingField::Passthrough => "Thru",
        RoutingField::RecordInputFx => "Rec FX",
        RoutingField::MonitorInputFx => "Mon FX",
        RoutingField::InputFxSlot | RoutingField::OutputFxSlot => "Slot",
        RoutingField::InputFxKind | RoutingField::OutputFxKind => "Kind",
        RoutingField::InputFxEnabled | RoutingField::OutputFxEnabled => "On",
        RoutingField::InputFxParam1 | RoutingField::OutputFxParam1 => "P1",
        RoutingField::InputFxParam2 | RoutingField::OutputFxParam2 => "P2",
        RoutingField::InputFxMore | RoutingField::OutputFxMore => "More",
    }
}

fn visible_param_label(param: Option<&MidiFxInlineParam>, fallback: &'static str) -> String {
    param
        .map(|param| param.label.to_string())
        .unwrap_or_else(|| fallback.to_string())
}

fn ticks_per_second_for_tempo(tempo_bpm: f64, ppqn: u16) -> u64 {
    let clamped_bpm = tempo_bpm.clamp(20.0, 400.0);
    ((clamped_bpm * f64::from(ppqn.max(1))) / 60.0).round() as u64
}

fn midi_learn_label(event: &MidiInputEvent) -> String {
    match event.message {
        MidiInputMessage::NoteOn { pitch, .. } | MidiInputMessage::NoteOff { pitch } => {
            format!("Note {} Ch{}", midi_note_name(pitch), event.channel)
        }
        MidiInputMessage::ControlChange { controller, .. } => {
            format!("CC{} Ch{}", controller, event.channel)
        }
    }
}

fn midi_note_name(pitch: u8) -> String {
    const NAMES: [&str; 12] = [
        "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
    ];
    let name = NAMES[(pitch % 12) as usize];
    let octave = (pitch / 12) as i16 - 1;
    format!("{name}{octave}")
}

fn midi_note_label(pitch: u8) -> String {
    format!("Note {}", midi_note_name(pitch))
}

fn midi_mapping_matches_event(entry: &MappingEntry, event: &MidiInputEvent) -> bool {
    if !entry.enabled || entry.source_kind != MappingSourceKind::Midi {
        return false;
    }

    if entry.source_device_label != default_mapping_source_device()
        && entry.source_device_label != event.port.name
    {
        return false;
    }

    match event.message {
        MidiInputMessage::NoteOn { pitch, .. } | MidiInputMessage::NoteOff { pitch } => {
            if matches!(event.message, MidiInputMessage::NoteOff { .. })
                && !midi_mapping_target_supports_release(entry.target_label.as_str())
            {
                return false;
            }
            entry.source_label == midi_note_label(pitch)
                || entry.source_label == format!("{} Ch{}", midi_note_label(pitch), event.channel)
        }
        MidiInputMessage::ControlChange { controller, value } => {
            if value == 0 && !midi_mapping_target_supports_release(entry.target_label.as_str()) {
                return false;
            }
            entry.source_label == format!("CC{controller}")
                || entry.source_label == format!("CC{controller} Ch{}", event.channel)
        }
    }
}

fn midi_mapping_target_supports_release(target_label: &str) -> bool {
    matches!(target_label, "Record Hold" | "Select Notes At Playhead Add")
}

fn port_name(port: Option<&MidiPortRef>) -> &str {
    port.map(|value| value.name.as_str()).unwrap_or("none")
}

fn resolve_port_by_name(ports: &[MidiPortRef], preferred_name: Option<&str>) -> Option<usize> {
    let preferred_name = preferred_name?;
    ports.iter().position(|port| port.name == preferred_name)
}

fn clamp_index(index: usize, len: usize) -> usize {
    if len == 0 {
        0
    } else {
        index.min(len - 1)
    }
}

fn input_channel_label(channel: MidiChannelFilter) -> String {
    match channel {
        MidiChannelFilter::Omni => "all".to_string(),
        MidiChannelFilter::Channel(value) => value.to_string(),
    }
}

fn output_channel_label(channel: Option<u8>) -> String {
    channel
        .map(|value| value.to_string())
        .unwrap_or_else(|| "none".to_string())
}

fn page_tabs_layout(bounds: Rect) -> (Rect, Rect) {
    let branding_width = preferred_branding_width(bounds.width());
    if branding_width == 0 {
        return (Rect::new(bounds.x, bounds.y, 0, bounds.height()), bounds);
    }

    let gap = 14_i32;
    let tabs_width = bounds
        .width()
        .saturating_sub(branding_width)
        .saturating_sub(gap as u32);
    (
        Rect::new(bounds.x, bounds.y, branding_width, bounds.height()),
        Rect::new(
            bounds.x + branding_width as i32 + gap,
            bounds.y,
            tabs_width,
            bounds.height(),
        ),
    )
}

fn preferred_branding_width(bounds_width: u32) -> u32 {
    let desired = 220_u32;
    let minimum_tabs_width = 360_u32;
    if bounds_width <= desired + minimum_tabs_width {
        0
    } else {
        desired
    }
}

fn on_off(value: bool) -> &'static str {
    if value {
        "on"
    } else {
        "off"
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AppControl {
    Continue,
    Quit,
}

struct TransportChipSpec {
    label: String,
    action: Option<AppAction>,
    fill: Color,
}

#[cfg(test)]
mod tests {
    use super::{
        cycle_input_channel, cycle_optional_port, cycle_output_channel, mapping_field_index,
        ticks_per_second_for_tempo, transport_strip_height, App, AppControl, AppOverlay,
        LastActionStatus,
    };
    use crate::actions::{ActionSource, AppAction};
    use crate::mapping::{default_mapping_source_device, MappingEntry, MappingSourceKind};
    use crate::midi_fx::{MidiFx, MidiFxChainKind, MidiFxSlot, MIDI_FX_SLOT_COUNT};
    use crate::midi_io::{MidiInputEvent, MidiInputMessage, MidiPortRef};
    use crate::pages::{AppPage, MappingField, MappingPageMode};
    use crate::project::{MidiNote, RecordContext, Track, TrackKind, STORED_LOOP_SLOT_COUNT};
    use crate::routing::MidiChannelFilter;
    use crate::timeline_fx::{TimelineContext, TimelineFxField};
    use crate::transport::{QuantizeMode, RecordMode};
    use crate::ui::TimelineFlow;
    use sdl3::pixels::Color;
    use sdl3::rect::Rect;
    use std::time::Duration;

    fn region_span(region: crate::timeline::Region) -> (u64, u64) {
        (region.start_ticks, region.length_ticks)
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
    fn logical_viewport_size_respects_display_scale() {
        assert_eq!(super::logical_viewport_size((2560, 1440), 2.0), (1280, 720));
        assert_eq!(super::logical_viewport_size((1920, 1080), 1.5), (1280, 720));
    }

    #[test]
    fn active_draw_size_prefers_logical_viewport_over_output_pixels() {
        assert_eq!(
            super::active_draw_size((2560, 1440), (1280, 720)),
            (1280, 720)
        );
        assert_eq!(super::active_draw_size((1280, 720), (0, 0)), (1280, 720));
    }

    #[test]
    fn ui_scale_override_wins_over_display_scale() {
        assert_eq!(super::effective_ui_scale(1.5, Some(2.0)), 2.0);
        assert_eq!(super::effective_ui_scale(1.5, None), 1.5);
        assert_eq!(super::effective_ui_scale(0.5, None), 1.0);
    }

    #[test]
    fn auto_window_scale_interpolation_only_enables_for_non_integer_values() {
        assert!(super::should_interpolate_window_scale(
            super::UiScalingMode::Auto,
            1.5,
            1.0
        ));
        assert!(super::should_interpolate_window_scale(
            super::UiScalingMode::Auto,
            1.0,
            1.25
        ));
        assert!(!super::should_interpolate_window_scale(
            super::UiScalingMode::Auto,
            1.0,
            2.0
        ));
        assert!(!super::should_interpolate_window_scale(
            super::UiScalingMode::Auto,
            2.0004,
            1.0
        ));
    }

    #[test]
    fn explicit_window_scale_modes_override_auto_behavior() {
        assert!(!super::should_interpolate_window_scale(
            super::UiScalingMode::Nearest,
            1.5,
            1.5
        ));
        assert!(super::should_interpolate_window_scale(
            super::UiScalingMode::Linear,
            1.0,
            1.0
        ));
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
    fn cycle_global_harmony_root_updates_transport_chip_label() {
        let mut app = App::new();
        app.apply_action(AppAction::CycleGlobalHarmonyRoot);
        let labels = app
            .transport_bottom_chip_specs()
            .into_iter()
            .map(|chip| chip.label)
            .collect::<Vec<_>>();
        assert!(labels.iter().any(|label| label == "Harmony C#"));
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
        });

        assert!(app.note_additive_select_held);
        assert!(app.project.active_track().unwrap().has_note_selection());

        app.handle_midi_input_event(MidiInputEvent {
            port: MidiPortRef::new("Port A"),
            channel: 1,
            message: MidiInputMessage::NoteOff { pitch: 36 },
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
    fn page_actions_cycle_between_views() {
        let mut app = App::new();

        app.apply_action(AppAction::ShowNextPage);
        assert_eq!(app.page_state.current_page, AppPage::Mappings);

        app.apply_action(AppAction::ShowPreviousPage);
        assert_eq!(app.page_state.current_page, AppPage::Timeline);
    }

    #[test]
    fn mappings_page_is_read_only() {
        let mut app = App::new();
        app.apply_action(AppAction::ShowPage(AppPage::Mappings));
        let before = app.mappings[0].enabled;

        app.apply_action(AppAction::ActivatePageItem);
        assert_eq!(app.mappings[0].enabled, before);
    }

    #[test]
    fn mappings_page_write_mode_can_edit_enabled_state() {
        let mut app = App::new();
        app.apply_action(AppAction::ShowPage(AppPage::Mappings));
        let before = app.mappings[0].enabled;

        app.apply_action(AppAction::ToggleMappingsWriteMode);
        app.page_state.selected_mapping_field = MappingField::Enabled;
        app.apply_action(AppAction::ActivatePageItem);

        assert_ne!(app.mappings[0].enabled, before);
    }

    #[test]
    fn mappings_page_write_mode_cycles_selected_field() {
        let mut app = App::new();
        app.apply_action(AppAction::ShowPage(AppPage::Mappings));
        app.apply_action(AppAction::ToggleMappingsWriteMode);
        assert_eq!(app.page_state.mapping_mode, MappingPageMode::Write);
        assert_eq!(
            app.page_state.selected_mapping_field,
            MappingField::SourceValue
        );

        app.apply_action(AppAction::SelectNextPageField);
        assert_eq!(app.page_state.selected_mapping_field, MappingField::Target);
    }

    #[test]
    fn mappings_page_write_mode_can_add_and_remove_rows() {
        let mut app = App::new();
        app.apply_action(AppAction::ShowPage(AppPage::Mappings));
        app.apply_action(AppAction::ToggleMappingsWriteMode);
        let original_len = app.mappings.len();
        let selected_index = app.page_state.selected_mapping_index;

        app.apply_action(AppAction::AddMappingRow);

        assert_eq!(app.mappings.len(), original_len + 1);
        assert_eq!(app.page_state.selected_mapping_index, selected_index + 1);
        assert!(!app.mappings[app.page_state.selected_mapping_index].enabled);

        app.apply_action(AppAction::RemoveSelectedMapping);

        assert_eq!(app.mappings.len(), original_len);
        assert!(app.page_state.selected_mapping_index < app.mappings.len());
    }

    #[test]
    fn mappings_target_lookup_opens_and_commits_filtered_result() {
        let mut app = App::new();
        app.apply_action(AppAction::ShowPage(AppPage::Mappings));
        app.apply_action(AppAction::ToggleMappingsWriteMode);
        app.page_state.selected_mapping_field = MappingField::Target;
        app.mappings[0].target_label = "Play/Stop".to_string();
        app.mappings[0].scope_label = "Global".to_string();

        app.apply_action(AppAction::ActivatePageItem);
        assert!(app.target_lookup_state.active.is_some());

        let _ = app.handle_keyboard_event(&sdl3::event::Event::KeyDown {
            timestamp: 0,
            window_id: 0,
            which: 0,
            scancode: None,
            keycode: Some(sdl3::keyboard::Keycode::A),
            keymod: sdl3::keyboard::Mod::NOMOD,
            repeat: false,
            raw: 0,
        });
        let _ = app.handle_keyboard_event(&sdl3::event::Event::KeyDown {
            timestamp: 0,
            window_id: 0,
            which: 0,
            scancode: None,
            keycode: Some(sdl3::keyboard::Keycode::R),
            keymod: sdl3::keyboard::Mod::NOMOD,
            repeat: false,
            raw: 0,
        });
        let _ = app.handle_keyboard_event(&sdl3::event::Event::KeyDown {
            timestamp: 0,
            window_id: 0,
            which: 0,
            scancode: None,
            keycode: Some(sdl3::keyboard::Keycode::M),
            keymod: sdl3::keyboard::Mod::NOMOD,
            repeat: false,
            raw: 0,
        });
        let _ = app.handle_keyboard_event(&sdl3::event::Event::KeyDown {
            timestamp: 0,
            window_id: 0,
            which: 0,
            scancode: None,
            keycode: Some(sdl3::keyboard::Keycode::Return),
            keymod: sdl3::keyboard::Mod::NOMOD,
            repeat: false,
            raw: 0,
        });

        assert_eq!(app.mappings[0].target_label, "Track Arm");
        assert_eq!(app.mappings[0].scope_label, "Active Track");
        assert!(app.target_lookup_state.active.is_none());
    }

    #[test]
    fn mappings_target_lookup_resets_invalid_scope_and_escape_cancels() {
        let mut app = App::new();
        app.apply_action(AppAction::ShowPage(AppPage::Mappings));
        app.apply_action(AppAction::ToggleMappingsWriteMode);
        app.page_state.selected_mapping_field = MappingField::Target;
        app.mappings[0].target_label = "Track Arm".to_string();
        app.mappings[0].scope_label = "Track 3".to_string();

        app.apply_action(AppAction::ActivatePageItem);
        let _ = app.handle_keyboard_event(&sdl3::event::Event::KeyDown {
            timestamp: 0,
            window_id: 0,
            which: 0,
            scancode: None,
            keycode: Some(sdl3::keyboard::Keycode::P),
            keymod: sdl3::keyboard::Mod::NOMOD,
            repeat: false,
            raw: 0,
        });
        let _ = app.handle_keyboard_event(&sdl3::event::Event::KeyDown {
            timestamp: 0,
            window_id: 0,
            which: 0,
            scancode: None,
            keycode: Some(sdl3::keyboard::Keycode::L),
            keymod: sdl3::keyboard::Mod::NOMOD,
            repeat: false,
            raw: 0,
        });
        let _ = app.handle_keyboard_event(&sdl3::event::Event::KeyDown {
            timestamp: 0,
            window_id: 0,
            which: 0,
            scancode: None,
            keycode: Some(sdl3::keyboard::Keycode::A),
            keymod: sdl3::keyboard::Mod::NOMOD,
            repeat: false,
            raw: 0,
        });
        let _ = app.handle_keyboard_event(&sdl3::event::Event::KeyDown {
            timestamp: 0,
            window_id: 0,
            which: 0,
            scancode: None,
            keycode: Some(sdl3::keyboard::Keycode::Y),
            keymod: sdl3::keyboard::Mod::NOMOD,
            repeat: false,
            raw: 0,
        });
        let _ = app.handle_keyboard_event(&sdl3::event::Event::KeyDown {
            timestamp: 0,
            window_id: 0,
            which: 0,
            scancode: None,
            keycode: Some(sdl3::keyboard::Keycode::Return),
            keymod: sdl3::keyboard::Mod::NOMOD,
            repeat: false,
            raw: 0,
        });

        assert_eq!(app.mappings[0].target_label, "Play/Stop");
        assert_eq!(app.mappings[0].scope_label, "Global");

        app.mappings[0].target_label = "Track Arm".to_string();
        app.mappings[0].scope_label = "Track 3".to_string();
        app.apply_action(AppAction::ActivatePageItem);
        let _ = app.handle_keyboard_event(&sdl3::event::Event::KeyDown {
            timestamp: 0,
            window_id: 0,
            which: 0,
            scancode: None,
            keycode: Some(sdl3::keyboard::Keycode::Escape),
            keymod: sdl3::keyboard::Mod::NOMOD,
            repeat: false,
            raw: 0,
        });

        assert_eq!(app.mappings[0].target_label, "Track Arm");
        assert_eq!(app.mappings[0].scope_label, "Track 3");
        assert!(app.target_lookup_state.active.is_none());
        assert_eq!(
            app.status_state
                .last_action
                .as_ref()
                .map(|status| status.action),
            Some(AppAction::CancelCurrentMode)
        );
    }

    #[test]
    fn mappings_page_scope_cycles_into_absolute_track_targets() {
        let mut app = App::new();
        app.apply_action(AppAction::ShowPage(AppPage::Mappings));
        app.apply_action(AppAction::ToggleMappingsWriteMode);
        app.page_state.selected_mapping_index = 0;
        app.page_state.selected_mapping_field = MappingField::Target;

        app.mappings[0].target_label = "Track Arm".to_string();
        app.mappings[0].scope_label = "Active Track".to_string();
        app.apply_action(AppAction::SelectNextPageField);
        app.apply_action(AppAction::AdjustPageItemForward);
        assert_eq!(app.mappings[0].scope_label, "Track 1");

        app.apply_action(AppAction::AdjustPageItemBackward);
        assert_eq!(app.mappings[0].scope_label, "Active Track");
    }

    #[test]
    fn mappings_page_skips_device_field_for_non_midi_rows() {
        let mut app = App::new();
        app.apply_action(AppAction::ShowPage(AppPage::Mappings));
        app.apply_action(AppAction::ToggleMappingsWriteMode);
        app.mappings[0].source_kind = MappingSourceKind::Key;
        app.page_state.selected_mapping_field = MappingField::SourceKind;

        app.apply_action(AppAction::SelectNextPageField);

        assert_eq!(
            app.page_state.selected_mapping_field,
            MappingField::SourceValue
        );
    }

    #[test]
    fn switching_away_from_midi_disables_device_field() {
        let mut app = App::new();
        app.apply_action(AppAction::ShowPage(AppPage::Mappings));
        app.apply_action(AppAction::ToggleMappingsWriteMode);
        app.mappings[0].source_kind = MappingSourceKind::Midi;
        app.mappings[0].source_device_label = "Port A".to_string();
        app.page_state.selected_mapping_field = MappingField::SourceDevice;

        app.page_state.selected_mapping_field = MappingField::SourceKind;
        app.apply_action(AppAction::ActivatePageItem);

        assert_ne!(app.mappings[0].source_kind, MappingSourceKind::Midi);
        assert_eq!(
            app.mappings[0].source_device_label,
            default_mapping_source_device()
        );
        assert_ne!(
            app.page_state.selected_mapping_field,
            MappingField::SourceDevice
        );
    }

    #[test]
    fn mapping_row_cells_match_field_order_for_device_and_source() {
        let app = App::new();
        let cells = app.mapping_row_cells(Rect::new(0, 0, 400, 18));

        assert!(
            cells[mapping_field_index(MappingField::SourceDevice)].x
                < cells[mapping_field_index(MappingField::SourceValue)].x
        );
    }

    #[test]
    fn midi_learn_updates_selected_mapping_source() {
        let mut app = App::new();
        app.apply_action(AppAction::ShowPage(AppPage::Mappings));
        app.apply_action(AppAction::ToggleMappingsWriteMode);
        app.page_state.selected_mapping_field = MappingField::SourceValue;
        app.mappings[0].source_kind = MappingSourceKind::Midi;
        app.apply_action(AppAction::ActivatePageItem);
        assert!(app.page_state.mapping_midi_learn_armed);

        app.handle_midi_input_event(MidiInputEvent {
            port: MidiPortRef::new("In A"),
            channel: 3,
            message: MidiInputMessage::ControlChange {
                controller: 24,
                value: 127,
            },
        });

        assert_eq!(app.mappings[0].source_label, "CC24 Ch3");
        assert_eq!(app.mappings[0].source_device_label, "In A");
        assert!(!app.page_state.mapping_midi_learn_armed);
    }

    #[test]
    fn mappings_page_syncs_all_inputs_for_midi_learn() {
        let mut app = App::new();
        app.midi_devices.inputs = vec![MidiPortRef::new("In A"), MidiPortRef::new("In B")];
        for track in &mut app.project.tracks {
            track.routing.input_port = None;
        }

        app.apply_action(AppAction::ShowPage(AppPage::Mappings));
        app.apply_action(AppAction::ToggleMappingsWriteMode);
        app.mappings[0].source_kind = MappingSourceKind::Midi;
        app.page_state.selected_mapping_field = MappingField::SourceValue;
        app.apply_action(AppAction::ActivatePageItem);

        let connected = app.midi_input.requested_port_names();
        assert!(app.page_state.mapping_midi_learn_armed);
        assert_eq!(connected, vec!["In A".to_string(), "In B".to_string()]);
    }

    #[test]
    fn midi_mapping_triggers_action_for_matching_device() {
        let mut app = App::new();
        app.project.select_track(1);
        app.project.tracks[1].state.armed = false;
        app.mappings = vec![MappingEntry {
            source_kind: MappingSourceKind::Midi,
            source_device_label: "Port A".to_string(),
            source_label: "CC20".to_string(),
            target_label: "Track Arm".to_string(),
            scope_label: "Active Track".to_string(),
            enabled: true,
        }];

        app.handle_midi_input_event(MidiInputEvent {
            port: MidiPortRef::new("Port A"),
            channel: 1,
            message: MidiInputMessage::ControlChange {
                controller: 20,
                value: 127,
            },
        });

        assert!(app.project.tracks[1].state.armed);
    }

    #[test]
    fn midi_mapping_ignores_non_matching_device() {
        let mut app = App::new();
        app.project.select_track(1);
        app.project.tracks[1].state.armed = false;
        app.mappings = vec![MappingEntry {
            source_kind: MappingSourceKind::Midi,
            source_device_label: "Port A".to_string(),
            source_label: "CC20".to_string(),
            target_label: "Track Arm".to_string(),
            scope_label: "Active Track".to_string(),
            enabled: true,
        }];

        app.handle_midi_input_event(MidiInputEvent {
            port: MidiPortRef::new("Port B"),
            channel: 1,
            message: MidiInputMessage::ControlChange {
                controller: 20,
                value: 127,
            },
        });

        assert!(!app.project.tracks[1].state.armed);
    }

    #[test]
    fn midi_mapping_can_target_absolute_track_scope() {
        let mut app = App::new();
        app.project.select_track(0);
        app.project.tracks[2].state.armed = false;
        app.mappings = vec![MappingEntry {
            source_kind: MappingSourceKind::Midi,
            source_device_label: "Any MIDI".to_string(),
            source_label: "CC20".to_string(),
            target_label: "Track Arm".to_string(),
            scope_label: "Track 3".to_string(),
            enabled: true,
        }];

        app.handle_midi_input_event(MidiInputEvent {
            port: MidiPortRef::new("Port A"),
            channel: 1,
            message: MidiInputMessage::ControlChange {
                controller: 20,
                value: 127,
            },
        });

        assert_eq!(app.project.active_track_index, 2);
        assert!(app.project.tracks[2].state.armed);
    }

    #[test]
    fn discoverability_overlay_toggles_separately_from_quick_overlay() {
        let mut app = App::new();

        app.apply_action(AppAction::ToggleDiscoverabilityOverlay);
        assert_eq!(app.overlay_state.active, Some(AppOverlay::Discoverability));

        app.apply_action(AppAction::ToggleMappingsOverlay);
        assert_eq!(
            app.overlay_state.active,
            Some(AppOverlay::MappingsQuickView)
        );
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
    fn cycle_helpers_wrap_through_expected_ranges() {
        let app = App::new();
        assert_eq!(
            cycle_optional_port(None, &app.midi_devices.outputs, 1)
                .unwrap()
                .name,
            app.midi_devices.outputs[0].name
        );
        assert_eq!(
            cycle_input_channel(MidiChannelFilter::Omni, 1),
            MidiChannelFilter::Channel(1)
        );
        assert_eq!(cycle_output_channel(None, -1), Some(16));
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
    fn changing_global_loop_sends_all_notes_off() {
        let mut app = App::new();
        let baseline = app.midi_output.sent_all_notes_off_count();

        app.apply_action(AppAction::SetGlobalLoopStart);

        assert_eq!(
            app.midi_output.sent_all_notes_off_count(),
            baseline + app.project.tracks.len()
        );
    }

    #[test]
    fn changing_track_loop_sends_all_notes_off() {
        let mut app = App::new();
        let baseline = app.midi_output.sent_all_notes_off_count();

        app.apply_action(AppAction::NudgeCurrentTrackLoopForward);

        assert_eq!(
            app.midi_output.sent_all_notes_off_count(),
            baseline + app.project.tracks.len()
        );
    }

    #[test]
    fn toggle_recording_creates_visible_take_content() {
        let mut app = App::new();
        let track = app.project.active_track_mut().unwrap();
        track.clear_content();
        track.routing.input_port = Some(MidiPortRef::new("Test Input"));
        app.transport_ticks = 0;
        app.playhead_ticks = 0;

        app.apply_action(AppAction::ToggleRecording);
        assert!(app.project.transport.recording);
        assert!(app.project.transport.playing);

        let input_port = app
            .project
            .active_track()
            .and_then(|track| track.routing.input_port.clone())
            .expect("test track should have explicit input port");
        app.handle_midi_input_event(MidiInputEvent {
            port: input_port.clone(),
            channel: 1,
            message: MidiInputMessage::NoteOn {
                pitch: 64,
                velocity: 100,
            },
        });

        app.transport_ticks = 1_920;
        app.playhead_ticks = 1_920;
        app.handle_midi_input_event(MidiInputEvent {
            port: input_port,
            channel: 1,
            message: MidiInputMessage::NoteOff { pitch: 64 },
        });
        app.apply_action(AppAction::ToggleRecording);

        let active = app.project.active_track().unwrap();
        assert!(!app.project.transport.recording);
        assert!(active.active_take.is_none());
        assert!(!active.regions.is_empty());
        assert!(active.midi_notes.iter().any(|note| note.pitch == 64));
    }

    #[test]
    fn track_clone_passthrough_sends_live_output_to_target_track() {
        let mut app = App::new();
        app.project.clear_all_track_content();
        app.project.tracks[0].routing.input_port = Some(MidiPortRef::new("Test Input"));
        app.project.tracks[0].midi_fx.input_fx = vec![None; MIDI_FX_SLOT_COUNT];
        app.project.tracks[1].state.passthrough = true;
        app.project.tracks[1].midi_fx.monitor_input_fx = true;
        app.project.tracks[1].routing.output_port = Some(MidiPortRef::new("Out B"));
        app.project.tracks[1].routing.output_channel = Some(2);
        app.project.tracks[1].midi_fx.input_fx[0] = Some(MidiFxSlot {
            enabled: true,
            effect: MidiFx::TrackClone { source_track: 0 },
        });
        app.project.tracks[1].midi_fx.input_fx[1] = Some(MidiFxSlot {
            enabled: true,
            effect: MidiFx::Transpose { semitones: 12 },
        });
        app.project.tracks[1].midi_fx.output_fx = vec![None; MIDI_FX_SLOT_COUNT];

        let input_port = app.project.tracks[0].routing.input_port.clone().unwrap();
        app.handle_midi_input_event(MidiInputEvent {
            port: input_port.clone(),
            channel: 1,
            message: MidiInputMessage::NoteOn {
                pitch: 60,
                velocity: 100,
            },
        });
        app.handle_midi_input_event(MidiInputEvent {
            port: input_port,
            channel: 1,
            message: MidiInputMessage::NoteOff { pitch: 60 },
        });

        let sent = app.midi_output.sent_messages();
        assert!(sent
            .iter()
            .any(|(port, channel, pitch, velocity)| port == "Out B"
                && *channel == 2
                && *pitch == 72
                && velocity.is_some()));
        assert!(sent
            .iter()
            .any(|(port, channel, pitch, velocity)| port == "Out B"
                && *channel == 2
                && *pitch == 72
                && velocity.is_none()));
    }

    #[test]
    fn track_clone_monitor_fx_sends_live_output_without_passthrough() {
        let mut app = App::new();
        app.project.clear_all_track_content();
        app.project.tracks[0].routing.input_port = Some(MidiPortRef::new("Test Input"));
        app.project.tracks[0].midi_fx.input_fx = vec![None; MIDI_FX_SLOT_COUNT];
        app.project.tracks[1].state.passthrough = false;
        app.project.tracks[1].midi_fx.monitor_input_fx = true;
        app.project.tracks[1].routing.output_port = Some(MidiPortRef::new("Out B"));
        app.project.tracks[1].routing.output_channel = Some(2);
        app.project.tracks[1].routing.input_port = Some(MidiPortRef::new("Test Input"));
        app.project.tracks[1].routing.input_channel = MidiChannelFilter::Channel(1);
        app.project.tracks[1].midi_fx.input_fx[0] = Some(MidiFxSlot {
            enabled: true,
            effect: MidiFx::TrackClone { source_track: 0 },
        });
        app.project.tracks[1].midi_fx.input_fx[1] = Some(MidiFxSlot {
            enabled: true,
            effect: MidiFx::Transpose { semitones: 12 },
        });
        app.project.tracks[1].midi_fx.output_fx = vec![None; MIDI_FX_SLOT_COUNT];

        let input_port = app.project.tracks[0].routing.input_port.clone().unwrap();
        app.handle_midi_input_event(MidiInputEvent {
            port: input_port.clone(),
            channel: 1,
            message: MidiInputMessage::NoteOn {
                pitch: 60,
                velocity: 100,
            },
        });
        app.handle_midi_input_event(MidiInputEvent {
            port: input_port,
            channel: 1,
            message: MidiInputMessage::NoteOff { pitch: 60 },
        });

        let sent = app.midi_output.sent_messages();
        assert!(sent
            .iter()
            .any(|(port, channel, pitch, velocity)| port == "Out B"
                && *channel == 2
                && *pitch == 72
                && velocity.is_some()));
        assert!(!sent
            .iter()
            .any(|(port, channel, pitch, velocity)| port == "Out B"
                && *channel == 2
                && *pitch == 60
                && velocity.is_some()));
    }

    #[test]
    fn track_clone_uses_recorded_source_midi_for_playback_stream() {
        let mut app = App::new();
        app.project.clear_all_track_content();
        app.project.tracks[0]
            .midi_notes
            .push(MidiNote::new(48, 0, 480, 100));
        app.project.tracks[1].midi_fx.input_fx = vec![None; MIDI_FX_SLOT_COUNT];
        app.project.tracks[1].midi_fx.input_fx[0] = Some(MidiFxSlot {
            enabled: true,
            effect: MidiFx::TrackClone { source_track: 0 },
        });
        app.project.tracks[1].midi_fx.input_fx[1] = Some(MidiFxSlot {
            enabled: true,
            effect: MidiFx::Transpose { semitones: 12 },
        });
        app.project.tracks[1].midi_fx.output_fx = vec![None; MIDI_FX_SLOT_COUNT];

        let notes = app.effective_track_output_notes(1);
        assert!(notes.iter().any(|note| note.pitch == 60));
    }

    #[test]
    fn track_clone_records_into_post_input_fx_take() {
        let mut app = App::new();
        app.project.clear_all_track_content();
        app.project.select_track(1);
        app.project.tracks[0].routing.input_port = Some(MidiPortRef::new("Test Input"));
        app.project.tracks[0].midi_fx.input_fx = vec![None; MIDI_FX_SLOT_COUNT];
        app.project.tracks[1].state.armed = true;
        app.project.tracks[1].midi_fx.record_input_fx_mode =
            crate::midi_fx::RecordInputFxMode::PostInputFx;
        app.project.tracks[1].midi_fx.input_fx[0] = Some(MidiFxSlot {
            enabled: true,
            effect: MidiFx::TrackClone { source_track: 0 },
        });
        app.project.tracks[1].midi_fx.input_fx[1] = Some(MidiFxSlot {
            enabled: true,
            effect: MidiFx::Transpose { semitones: 12 },
        });
        app.transport_ticks = 0;
        app.playhead_ticks = 0;

        app.apply_action(AppAction::ToggleRecording);
        assert!(app.project.tracks[1].active_take.is_some());
        let input_port = app.project.tracks[0].routing.input_port.clone().unwrap();
        app.handle_midi_input_event(MidiInputEvent {
            port: input_port.clone(),
            channel: 1,
            message: MidiInputMessage::NoteOn {
                pitch: 60,
                velocity: 100,
            },
        });
        app.transport_ticks = 960;
        app.playhead_ticks = 960;
        app.handle_midi_input_event(MidiInputEvent {
            port: input_port,
            channel: 1,
            message: MidiInputMessage::NoteOff { pitch: 60 },
        });
        app.apply_action(AppAction::ToggleRecording);

        let target = &app.project.tracks[1];
        assert!(target.midi_notes.iter().any(|note| note.pitch == 72));
    }

    #[test]
    fn track_clone_records_playback_source_midi_into_post_input_fx_take() {
        let mut app = App::new();
        app.project.clear_all_track_content();
        app.project.select_track(1);
        app.project.tracks[0]
            .midi_notes
            .push(MidiNote::new(60, 0, 480, 100));
        app.project.tracks[1].state.armed = true;
        app.project.tracks[1].midi_fx.record_input_fx_mode =
            crate::midi_fx::RecordInputFxMode::PostInputFx;
        app.project.tracks[1].midi_fx.input_fx[0] = Some(MidiFxSlot {
            enabled: true,
            effect: MidiFx::TrackClone { source_track: 0 },
        });
        app.project.tracks[1].midi_fx.input_fx[1] = Some(MidiFxSlot {
            enabled: true,
            effect: MidiFx::Transpose { semitones: 12 },
        });
        app.transport_ticks = 0;
        app.playhead_ticks = 0;

        app.apply_action(AppAction::ToggleRecording);
        assert!(app.project.tracks[1].active_take.is_some());
        app.dispatch_midi_notes(0, 960);
        app.transport_ticks = 960;
        app.playhead_ticks = 960;
        app.apply_action(AppAction::ToggleRecording);

        let target = &app.project.tracks[1];
        assert!(target.midi_notes.iter().any(|note| note.pitch == 72));
    }

    #[test]
    fn track_clone_follows_source_track_loop_phase() {
        let mut app = App::new();
        app.project.clear_all_track_content();
        app.project.transport.loop_enabled = false;
        app.project.tracks[0].state.loop_enabled = true;
        app.project.tracks[0].loop_region = crate::timeline::LoopRegion::new(960, 960);
        app.project.tracks[0]
            .midi_notes
            .push(MidiNote::new(60, 960, 480, 100));
        app.project.tracks[1].routing.output_port = Some(MidiPortRef::new("Out B"));
        app.project.tracks[1].routing.output_channel = Some(2);
        app.project.tracks[1].midi_fx.input_fx[0] = Some(MidiFxSlot {
            enabled: true,
            effect: MidiFx::TrackClone { source_track: 0 },
        });

        app.dispatch_midi_notes(1_920, 960);

        let sent = app.midi_output.sent_messages();
        assert!(sent
            .iter()
            .any(|(port, channel, pitch, velocity)| port == "Out B"
                && *channel == 2
                && *pitch == 60
                && velocity.is_some()));
    }

    #[test]
    fn live_input_arp_passthrough_emits_timed_notes() {
        let mut app = App::new();
        app.project.clear_all_track_content();
        app.project.tracks[0].routing.input_port = Some(MidiPortRef::new("Test Input"));
        app.project.tracks[0].state.passthrough = true;
        app.project.tracks[0].routing.output_port = Some(MidiPortRef::new("Out A"));
        app.project.tracks[0].routing.output_channel = Some(1);
        app.project.tracks[0].midi_fx.input_fx[0] = Some(MidiFxSlot {
            enabled: true,
            effect: MidiFx::Arp {
                step_ticks: 240,
                order: crate::midi_fx::ArpOrder::Up,
                gate_percent: 100,
            },
        });

        let input_port = app.project.tracks[0].routing.input_port.clone().unwrap();
        app.handle_midi_input_event(MidiInputEvent {
            port: input_port.clone(),
            channel: 1,
            message: MidiInputMessage::NoteOn {
                pitch: 60,
                velocity: 100,
            },
        });
        app.handle_midi_input_event(MidiInputEvent {
            port: input_port,
            channel: 1,
            message: MidiInputMessage::NoteOn {
                pitch: 64,
                velocity: 100,
            },
        });

        app.dispatch_live_arp_events(0, 480);

        let sent = app.midi_output.sent_messages();
        assert!(sent.iter().any(|(port, channel, pitch, velocity)| {
            port == "Out A" && *channel == 1 && *pitch == 60 && velocity.is_some()
        }));
        assert!(sent.iter().any(|(port, channel, pitch, velocity)| {
            port == "Out A" && *channel == 1 && *pitch == 64 && velocity.is_some()
        }));
    }

    #[test]
    fn playback_output_delay_emits_note_off_after_source_note_window() {
        let mut app = App::new();
        app.project.clear_all_track_content();
        app.project.tracks[0].routing.output_port = Some(MidiPortRef::new("Out A"));
        app.project.tracks[0].routing.output_channel = Some(1);
        app.project.tracks[0]
            .midi_notes
            .push(MidiNote::new(60, 0, 240, 100));
        app.project.tracks[0].midi_fx.output_fx = vec![None; MIDI_FX_SLOT_COUNT];
        app.project.tracks[0].midi_fx.output_fx[0] = Some(MidiFxSlot {
            enabled: true,
            effect: MidiFx::Delay { ticks: 60 },
        });

        app.dispatch_midi_notes(0, 60);
        assert!(app.midi_output.sent_messages().is_empty());

        app.dispatch_midi_notes(60, 60);
        assert_eq!(
            app.midi_output.sent_messages(),
            vec![("Out A".to_string(), 1, 60, Some(100))]
        );

        app.dispatch_midi_notes(120, 60);
        app.dispatch_midi_notes(180, 60);
        app.dispatch_midi_notes(240, 60);
        assert_eq!(
            app.midi_output.sent_messages(),
            vec![("Out A".to_string(), 1, 60, Some(100))]
        );

        app.dispatch_midi_notes(300, 60);
        assert_eq!(
            app.midi_output.sent_messages(),
            vec![
                ("Out A".to_string(), 1, 60, Some(100)),
                ("Out A".to_string(), 1, 60, None),
            ]
        );
    }

    #[test]
    fn playback_output_delay_releases_notes_for_repeated_pattern_across_small_windows() {
        let mut app = App::new();
        app.project.clear_all_track_content();
        app.project.tracks[0].routing.output_port = Some(MidiPortRef::new("Out A"));
        app.project.tracks[0].routing.output_channel = Some(1);
        app.project.tracks[0].midi_notes = vec![
            MidiNote::new(60, 0, 240, 100),
            MidiNote::new(64, 480, 240, 96),
        ];
        app.project.tracks[0].midi_fx.output_fx = vec![None; MIDI_FX_SLOT_COUNT];
        app.project.tracks[0].midi_fx.output_fx[0] = Some(MidiFxSlot {
            enabled: true,
            effect: MidiFx::Delay { ticks: 60 },
        });

        for start in [
            0_u64, 60, 120, 180, 240, 300, 360, 420, 480, 540, 600, 660, 720, 780,
        ] {
            app.dispatch_midi_notes(start, 60);
        }

        let sent = app.midi_output.sent_messages();
        for (pitch, velocity) in [(60, Some(100)), (64, Some(96))] {
            assert_eq!(
                sent.iter()
                    .filter(|(port, channel, event_pitch, event_velocity)| {
                        port == "Out A"
                            && *channel == 1
                            && *event_pitch == pitch
                            && *event_velocity == velocity
                    })
                    .count(),
                1
            );
            assert_eq!(
                sent.iter()
                    .filter(|(port, channel, event_pitch, event_velocity)| {
                        port == "Out A"
                            && *channel == 1
                            && *event_pitch == pitch
                            && event_velocity.is_none()
                    })
                    .count(),
                1
            );
        }
    }

    #[test]
    fn playback_output_duration_releases_note_after_extended_length_window() {
        let mut app = App::new();
        app.project.clear_all_track_content();
        app.project.tracks[0].routing.output_port = Some(MidiPortRef::new("Out A"));
        app.project.tracks[0].routing.output_channel = Some(1);
        app.project.tracks[0]
            .midi_notes
            .push(MidiNote::new(60, 0, 60, 100));
        app.project.tracks[0].midi_fx.output_fx = vec![None; MIDI_FX_SLOT_COUNT];
        app.project.tracks[0].midi_fx.output_fx[0] = Some(MidiFxSlot {
            enabled: true,
            effect: MidiFx::Duration { ticks: 240 },
        });

        app.dispatch_midi_notes(0, 60);
        assert_eq!(
            app.midi_output.sent_messages(),
            vec![("Out A".to_string(), 1, 60, Some(100))]
        );

        app.dispatch_midi_notes(60, 60);
        app.dispatch_midi_notes(120, 60);
        app.dispatch_midi_notes(180, 60);
        assert_eq!(
            app.midi_output.sent_messages(),
            vec![("Out A".to_string(), 1, 60, Some(100))]
        );

        app.dispatch_midi_notes(240, 60);
        assert_eq!(
            app.midi_output.sent_messages(),
            vec![
                ("Out A".to_string(), 1, 60, Some(100)),
                ("Out A".to_string(), 1, 60, None),
            ]
        );
    }

    #[test]
    fn shortening_duration_mid_playback_sends_all_notes_off_and_resets_fx_timing() {
        let mut app = App::new();
        app.project.clear_all_track_content();
        app.project.transport.playing = true;
        app.transport_ticks = 0;
        app.playhead_ticks = 0;
        app.project.select_track(0);
        app.project.tracks[0].routing.output_port = Some(MidiPortRef::new("Out A"));
        app.project.tracks[0].routing.output_channel = Some(1);
        app.project.tracks[0]
            .midi_notes
            .push(MidiNote::new(60, 0, 60, 100));
        app.project.tracks[0].midi_fx.output_fx = vec![None; MIDI_FX_SLOT_COUNT];
        app.project.tracks[0].midi_fx.output_fx[0] = Some(MidiFxSlot {
            enabled: true,
            effect: MidiFx::Duration { ticks: 240 },
        });
        app.page_state.current_page = AppPage::Timeline;
        app.page_state.selected_timeline_context = TimelineContext::OutputFx;
        app.page_state.selected_timeline_fx_field = TimelineFxField::ParamPrimary;
        app.set_selected_timeline_fx_slot_index(MidiFxChainKind::Output, 0);

        app.dispatch_midi_notes(0, 60);
        assert_eq!(
            app.midi_output.sent_messages(),
            vec![("Out A".to_string(), 1, 60, Some(100))]
        );
        let baseline_all_notes_off = app.midi_output.sent_all_notes_off_count();

        app.adjust_selected_timeline_fx_parameter(0, -1);

        assert!(app.midi_output.sent_all_notes_off_count() > baseline_all_notes_off);
        assert!(app
            .midi_output
            .sent_messages()
            .contains(&("Out A".to_string(), 1, 123, None)));
    }

    #[test]
    fn stopped_live_input_delay_uses_live_fx_clock_for_note_on_and_note_off() {
        let mut app = App::new();
        app.project.clear_all_track_content();
        app.project.transport.playing = false;
        app.transport_ticks = 0;
        app.playhead_ticks = 0;
        app.live_fx_ticks = 0;
        app.project.tracks[0].routing.input_port = Some(MidiPortRef::new("Test Input"));
        app.project.tracks[0].state.passthrough = true;
        app.project.tracks[0].routing.output_port = Some(MidiPortRef::new("Out A"));
        app.project.tracks[0].routing.output_channel = Some(1);
        app.project.tracks[0].midi_fx.input_fx = vec![None; MIDI_FX_SLOT_COUNT];
        app.project.tracks[0].midi_fx.output_fx = vec![None; MIDI_FX_SLOT_COUNT];
        app.project.tracks[0].midi_fx.input_fx[0] = Some(MidiFxSlot {
            enabled: true,
            effect: MidiFx::Delay { ticks: 240 },
        });

        app.advance_stopped_live_fx(Duration::from_millis(1_000), None);
        assert!(app.midi_output.sent_messages().is_empty());

        let input_port = app.project.tracks[0].routing.input_port.clone().unwrap();
        app.handle_midi_input_event(MidiInputEvent {
            port: input_port.clone(),
            channel: 1,
            message: MidiInputMessage::NoteOn {
                pitch: 60,
                velocity: 100,
            },
        });
        assert!(app.midi_output.sent_messages().is_empty());

        let note_on_tick = app.live_fx_ticks;
        app.dispatch_live_arp_events(note_on_tick, note_on_tick + 239);
        app.live_fx_ticks = note_on_tick + 239;
        assert!(app.midi_output.sent_messages().is_empty());

        app.dispatch_live_arp_events(app.live_fx_ticks, note_on_tick + 240);
        app.live_fx_ticks = note_on_tick + 240;
        assert_eq!(
            app.midi_output.sent_messages(),
            vec![("Out A".to_string(), 1, 60, Some(100))]
        );

        app.dispatch_live_arp_events(app.live_fx_ticks, note_on_tick + 720);
        app.live_fx_ticks = note_on_tick + 720;
        app.handle_midi_input_event(MidiInputEvent {
            port: input_port,
            channel: 1,
            message: MidiInputMessage::NoteOff { pitch: 60 },
        });
        assert_eq!(
            app.midi_output.sent_messages(),
            vec![("Out A".to_string(), 1, 60, Some(100))]
        );

        let note_off_tick = app.live_fx_ticks;
        app.dispatch_live_arp_events(note_off_tick, note_off_tick + 239);
        app.live_fx_ticks = note_off_tick + 239;
        assert_eq!(
            app.midi_output.sent_messages(),
            vec![("Out A".to_string(), 1, 60, Some(100))]
        );

        app.dispatch_live_arp_events(app.live_fx_ticks, note_off_tick + 240);
        app.live_fx_ticks = note_off_tick + 240;
        assert_eq!(
            app.midi_output.sent_messages(),
            vec![
                ("Out A".to_string(), 1, 60, Some(100)),
                ("Out A".to_string(), 1, 60, None),
            ]
        );
    }

    #[test]
    fn stopped_live_input_duration_uses_live_fx_clock_when_note_starts_late() {
        let mut app = App::new();
        app.project.clear_all_track_content();
        app.project.transport.playing = false;
        app.transport_ticks = 0;
        app.playhead_ticks = 0;
        app.live_fx_ticks = 0;
        app.project.tracks[0].routing.input_port = Some(MidiPortRef::new("Test Input"));
        app.project.tracks[0].state.passthrough = true;
        app.project.tracks[0].routing.output_port = Some(MidiPortRef::new("Out A"));
        app.project.tracks[0].routing.output_channel = Some(1);
        app.project.tracks[0].midi_fx.input_fx = vec![None; MIDI_FX_SLOT_COUNT];
        app.project.tracks[0].midi_fx.output_fx = vec![None; MIDI_FX_SLOT_COUNT];
        app.project.tracks[0].midi_fx.input_fx[0] = Some(MidiFxSlot {
            enabled: true,
            effect: MidiFx::Duration { ticks: 240 },
        });

        app.advance_stopped_live_fx(Duration::from_millis(1_000), None);

        let input_port = app.project.tracks[0].routing.input_port.clone().unwrap();
        app.handle_midi_input_event(MidiInputEvent {
            port: input_port,
            channel: 1,
            message: MidiInputMessage::NoteOn {
                pitch: 60,
                velocity: 100,
            },
        });
        assert_eq!(
            app.midi_output.sent_messages(),
            vec![("Out A".to_string(), 1, 60, Some(100))]
        );

        let note_on_tick = app.live_fx_ticks;
        app.dispatch_live_arp_events(note_on_tick, note_on_tick + 239);
        app.live_fx_ticks = note_on_tick + 239;
        assert_eq!(
            app.midi_output.sent_messages(),
            vec![("Out A".to_string(), 1, 60, Some(100))]
        );

        app.dispatch_live_arp_events(app.live_fx_ticks, note_on_tick + 240);
        app.live_fx_ticks = note_on_tick + 240;
        assert_eq!(
            app.midi_output.sent_messages(),
            vec![
                ("Out A".to_string(), 1, 60, Some(100)),
                ("Out A".to_string(), 1, 60, None),
            ]
        );
    }

    #[test]
    fn muting_track_sends_all_notes_off() {
        let mut app = App::new();
        app.project.clear_all_track_content();
        app.project.select_track(0);
        app.project.tracks[0].routing.output_port = Some(MidiPortRef::new("Out A"));
        app.project.tracks[0].routing.output_channel = Some(1);
        app.project.tracks[0]
            .midi_notes
            .push(MidiNote::new(60, 0, 1_920, 100));

        app.dispatch_midi_notes(0, 960);
        app.apply_action(AppAction::ToggleCurrentTrackMute);

        let sent = app.midi_output.sent_messages();
        assert!(sent.iter().any(|(port, channel, pitch, velocity)| {
            port == "Out A" && *channel == 1 && *pitch == 60 && velocity.is_some()
        }));
        assert!(sent.iter().any(|(port, channel, pitch, velocity)| {
            port == "Out A" && *channel == 1 && *pitch == 123 && velocity.is_none()
        }));
    }

    #[test]
    fn cycle_record_mode_updates_transport() {
        let mut app = App::new();
        assert_eq!(app.project.transport.record_mode, RecordMode::Overdub);

        app.apply_action(AppAction::CycleRecordMode);
        assert_eq!(app.project.transport.record_mode, RecordMode::Replace);
    }

    #[test]
    fn recording_targets_armed_tracks_before_active_track() {
        let mut app = App::new();
        app.project.clear_all_track_content();
        app.project.select_track(0);
        app.project.tracks[2].state.armed = true;

        app.apply_action(AppAction::ToggleRecording);
        assert!(app.project.tracks[2].active_take.is_some());
        assert!(app.project.tracks[0].active_take.is_none());

        app.transport_ticks = 960;
        app.playhead_ticks = 960;
        app.apply_action(AppAction::ToggleRecording);

        assert!(!app.project.tracks[2].regions.is_empty());
        assert!(app.project.tracks[0].regions.is_empty());
    }

    #[test]
    fn stopping_playback_commits_active_recording() {
        let mut app = App::new();
        app.project.active_track_mut().unwrap().clear_content();

        app.apply_action(AppAction::ToggleRecording);
        app.transport_ticks = 960;
        app.playhead_ticks = 960;
        app.apply_action(AppAction::TogglePlayback);

        assert!(!app.project.transport.recording);
        assert!(!app.project.transport.playing);
        assert!(!app.project.active_track().unwrap().regions.is_empty());
    }

    #[test]
    fn looped_track_recording_commits_inside_track_loop() {
        let mut app = App::new();
        let track = app.project.active_track_mut().unwrap();
        track.clear_content();
        track.state.loop_enabled = true;
        track.loop_region = crate::timeline::LoopRegion::new(960, 960);
        app.project.transport.quantize = crate::transport::QuantizeMode::Off;
        app.project.transport.loop_enabled = false;
        app.project.transport.loop_recording_extends_clip = false;
        app.transport_ticks = 1_680;
        app.playhead_ticks = 1_680;

        app.apply_action(AppAction::ToggleRecording);
        app.transport_ticks = 2_160;
        app.playhead_ticks = 1_200;
        app.apply_action(AppAction::ToggleRecording);

        let regions = &app.project.active_track().unwrap().regions;
        assert_eq!(regions.len(), 1);
        assert_eq!(region_span(regions[0]), (1_680, 240));
    }

    #[test]
    fn looped_track_recording_can_extend_clip_after_wrap() {
        let mut app = App::new();
        let track = app.project.active_track_mut().unwrap();
        track.clear_content();
        track.state.loop_enabled = true;
        track.loop_region = crate::timeline::LoopRegion::new(960, 960);
        app.project.transport.quantize = crate::transport::QuantizeMode::Off;
        app.project.transport.loop_enabled = false;
        app.project.transport.loop_recording_extends_clip = true;
        app.transport_ticks = 1_680;
        app.playhead_ticks = 1_680;

        app.apply_action(AppAction::ToggleRecording);
        app.transport_ticks = 2_160;
        app.playhead_ticks = 1_200;
        app.apply_action(AppAction::ToggleRecording);

        let regions = &app.project.active_track().unwrap().regions;
        assert_eq!(regions.len(), 1);
        assert_eq!(region_span(regions[0]), (960, 960));
    }

    #[test]
    fn looped_track_recording_preview_rebases_to_loop_start_after_wrap() {
        let mut app = App::new();
        let track = app.project.active_track_mut().unwrap();
        track.clear_content();
        track.state.loop_enabled = true;
        track.loop_region = crate::timeline::LoopRegion::new(960, 960);
        app.project.transport.quantize = crate::transport::QuantizeMode::Off;
        app.project.transport.loop_enabled = false;
        app.project.transport.loop_recording_extends_clip = true;
        app.transport_ticks = 1_680;
        app.playhead_ticks = 1_680;

        app.apply_action(AppAction::ToggleRecording);
        app.transport_ticks = 2_160;
        app.playhead_ticks = 1_200;

        let active_track = app.project.active_track().unwrap();
        let preview = active_track.preview_region(
            app.project.transport,
            app.record_capture_ticks(active_track),
            app.record_context(active_track),
        );

        assert_eq!(preview.map(region_span), Some((960, 960)));
    }

    #[test]
    fn detail_loop_range_uses_global_loop_when_track_loop_is_disabled() {
        let mut app = App::new();
        app.project.loop_region = crate::timeline::LoopRegion::new(960, 960);
        app.project.transport.loop_enabled = true;
        let track = app.project.active_track_mut().unwrap();
        track.state.loop_enabled = false;
        track.loop_region = crate::timeline::LoopRegion::new(0, 3_840);

        let detail_range = app.detail_loop_range(app.project.active_track().unwrap());

        assert_eq!(detail_range, crate::timeline::LoopRegion::new(960, 960));
    }

    #[test]
    fn record_context_prefers_global_loop_over_track_loop_when_both_are_enabled() {
        let mut app = App::new();
        app.project.loop_region = crate::timeline::LoopRegion::new(960, 960);
        app.project.transport.loop_enabled = true;
        let track = app.project.active_track_mut().unwrap();
        track.state.loop_enabled = true;
        track.loop_region = crate::timeline::LoopRegion::new(0, 3_840);

        let record_context = app
            .record_context(app.project.active_track().unwrap())
            .unwrap();

        assert_eq!(
            record_context.range,
            crate::timeline::LoopRegion::new(0, 3_840)
        );
        assert_eq!(record_context.wrap_basis_ticks, 0);
    }

    #[test]
    fn record_context_uses_transport_phase_for_track_loops_with_offset_start() {
        let mut app = App::new();
        app.project.transport.loop_enabled = false;
        let track = app.project.active_track_mut().unwrap();
        track.state.loop_enabled = true;
        track.loop_region = crate::timeline::LoopRegion::new(240, 960);

        let record_context = app
            .record_context(app.project.active_track().unwrap())
            .unwrap();

        assert_eq!(
            record_context.range,
            crate::timeline::LoopRegion::new(240, 960)
        );
        assert_eq!(record_context.wrap_basis_ticks, 0);
    }

    #[test]
    fn track_loop_recording_with_offset_start_commits_at_audible_playhead() {
        let mut app = App::new();
        app.project.transport.quantize = crate::transport::QuantizeMode::Off;
        app.project.transport.loop_enabled = false;
        let track = app.project.active_track_mut().unwrap();
        track.clear_content();
        track.state.loop_enabled = true;
        track.loop_region = crate::timeline::LoopRegion::new(240, 960);

        app.transport_ticks = 1_140;
        app.playhead_ticks = app.effective_track_playhead(app.project.active_track().unwrap());
        app.apply_action(AppAction::ToggleRecording);
        app.transport_ticks = 1_260;
        app.playhead_ticks = app.effective_track_playhead(app.project.active_track().unwrap());
        app.apply_action(AppAction::ToggleRecording);

        let committed = app
            .project
            .active_track()
            .unwrap()
            .regions
            .last()
            .copied()
            .unwrap();
        assert_eq!(region_span(committed), (420, 120));
    }

    #[test]
    fn looped_track_preview_clamps_to_loop_end_when_extension_is_off() {
        let mut app = App::new();
        let track = app.project.active_track_mut().unwrap();
        track.clear_content();
        track.state.loop_enabled = true;
        track.loop_region = crate::timeline::LoopRegion::new(2_880, 1_920);
        app.project.transport.quantize = crate::transport::QuantizeMode::Off;
        app.project.transport.loop_enabled = false;
        app.project.transport.loop_recording_extends_clip = false;
        app.transport_ticks = 5_600;
        app.playhead_ticks = 5_600;

        app.apply_action(AppAction::ToggleRecording);
        app.transport_ticks = 5_900;
        app.playhead_ticks = 2_980;

        let active_track = app.project.active_track().unwrap();
        let preview = active_track.preview_region(
            app.project.transport,
            app.record_capture_ticks(active_track),
            app.record_context(active_track),
        );

        assert_eq!(preview, Some(crate::timeline::Region::new(4_640, 160)));
    }

    #[test]
    fn scheduled_note_events_include_recorded_take_notes_before_commit() {
        let transport = crate::transport::Transport {
            quantize: QuantizeMode::Off,
            ..crate::transport::Transport::default()
        };
        let mut track = Track::new_empty("Track 1", TrackKind::Midi);
        track.state.loop_enabled = true;
        track.loop_region = crate::timeline::LoopRegion::new(960, 960);
        track.begin_recording(1_680);
        track.record_note_on(64, 100, 1_700);
        track.record_note_off(64, 1_760);

        let preview_notes = track.playback_preview_notes(
            transport,
            2_670,
            Some(RecordContext {
                range: track.loop_region,
                wrap_basis_ticks: 0,
                extend_clip_on_wrap: true,
            }),
        );
        let preview_occurrences = super::scheduled_note_occurrences(
            &track,
            preview_notes.as_slice(),
            2_650,
            20,
            Some(track.loop_region),
        );
        let events =
            super::occurrence_note_events(&track, preview_occurrences.as_slice(), 2_650, 20);

        assert!(events.iter().any(|event| *event == (2_660, true, 64, 100)));
    }

    #[test]
    fn scheduled_note_events_ignore_pending_take_notes_until_note_off() {
        let transport = crate::transport::Transport {
            quantize: QuantizeMode::Off,
            ..crate::transport::Transport::default()
        };
        let mut track = Track::new_empty("Track 1", TrackKind::Midi);
        track.state.loop_enabled = true;
        track.loop_region = crate::timeline::LoopRegion::new(960, 960);
        track.begin_recording(1_680);
        track.record_note_on(64, 100, 1_700);

        let preview_notes = track.playback_preview_notes(
            transport,
            2_670,
            Some(RecordContext {
                range: track.loop_region,
                wrap_basis_ticks: 0,
                extend_clip_on_wrap: true,
            }),
        );
        let preview_occurrences = super::scheduled_note_occurrences(
            &track,
            preview_notes.as_slice(),
            2_650,
            20,
            Some(track.loop_region),
        );
        let events =
            super::occurrence_note_events(&track, preview_occurrences.as_slice(), 2_650, 20);

        assert!(events.is_empty());
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
        assert!(app
            .project
            .tracks
            .iter()
            .all(|track| track.midi_notes.is_empty()));
        assert!(app
            .project
            .tracks
            .iter()
            .all(|track| track.regions.is_empty()));
    }

    #[test]
    fn timeline_track_arm_indicator_is_clickable() {
        let mut app = App::new();
        let content_bounds = Rect::new(40, 40, 1200, 620);
        let (_, body_bounds) =
            crate::ui::split_top_strip(content_bounds, 28, 6).expect("timeline content");
        let (_, timeline_bounds) =
            crate::ui::split_top_strip(body_bounds, transport_strip_height(), 8)
                .expect("timeline body");
        let columns = crate::ui::track_column_pairs(timeline_bounds, app.project.tracks.len());
        let (full_bounds, detail_bounds) = columns[1];
        let status_rect = crate::ui::track_status_rect(
            crate::ui::union_rect(full_bounds, detail_bounds),
            app.timeline_flow,
        );
        let arm_rect = crate::ui::track_indicators(status_rect)[0].rect;

        let control = app.handle_timeline_pointer(
            content_bounds,
            arm_rect.x + arm_rect.width() as i32 / 2,
            arm_rect.y + arm_rect.height() as i32 / 2,
            ActionSource::Pointer,
        );

        assert_eq!(control, Some(AppControl::Continue));
        assert_eq!(app.project.active_track_index, 1);
        assert!(app.project.tracks[1].state.armed);
    }

    #[test]
    fn timeline_track_record_indicator_starts_recording_for_clicked_track() {
        let mut app = App::new();
        let content_bounds = Rect::new(40, 40, 1200, 620);
        let (_, body_bounds) =
            crate::ui::split_top_strip(content_bounds, 28, 6).expect("timeline content");
        let (_, timeline_bounds) =
            crate::ui::split_top_strip(body_bounds, transport_strip_height(), 8)
                .expect("timeline body");
        let columns = crate::ui::track_column_pairs(timeline_bounds, app.project.tracks.len());
        let (full_bounds, detail_bounds) = columns[2];
        let status_rect = crate::ui::track_status_rect(
            crate::ui::union_rect(full_bounds, detail_bounds),
            app.timeline_flow,
        );
        let record_rect = crate::ui::track_indicators(status_rect)[1].rect;

        let control = app.handle_timeline_pointer(
            content_bounds,
            record_rect.x + record_rect.width() as i32 / 2,
            record_rect.y + record_rect.height() as i32 / 2,
            ActionSource::Pointer,
        );

        assert_eq!(control, Some(AppControl::Continue));
        assert_eq!(app.project.active_track_index, 2);
        assert!(app.project.transport.recording);
        assert!(app.project.transport.playing);
    }

    #[test]
    fn stored_loop_actions_store_and_recall_active_track_slot() {
        let mut app = App::new();
        let track = app.project.active_track_mut().unwrap();
        track.loop_region = crate::timeline::LoopRegion::new(1_920, 960);

        app.apply_action(AppAction::StoreCurrentLoopToSlot2);
        let track = app.project.active_track_mut().unwrap();
        track.loop_region = crate::timeline::LoopRegion::new(0, 4_800);
        track.state.loop_enabled = false;

        app.apply_action(AppAction::RecallStoredLoopSlot2);

        let track = app.project.active_track().unwrap();
        assert_eq!(
            track.loop_region,
            crate::timeline::LoopRegion::new(1_920, 960)
        );
        assert!(track.state.loop_enabled);
        assert_eq!(track.active_stored_loop_slot(), Some(1));
    }

    #[test]
    fn timeline_stored_loop_slot_is_clickable_for_recall() {
        let mut app = App::new();
        app.project.active_track_index = 1;
        {
            let track = &mut app.project.tracks[1];
            track.loop_region = crate::timeline::LoopRegion::new(2_880, 960);
            track.store_current_loop_to_slot(0);
            track.loop_region = crate::timeline::LoopRegion::new(0, 3_840);
        }

        let content_bounds = Rect::new(40, 40, 1200, 620);
        let (_, body_bounds) =
            crate::ui::split_top_strip(content_bounds, 28, 6).expect("timeline content");
        let (_, timeline_bounds) =
            crate::ui::split_top_strip(body_bounds, transport_strip_height(), 8)
                .expect("timeline body");
        let columns = crate::ui::track_column_pairs(timeline_bounds, app.project.tracks.len());
        let (full_bounds, detail_bounds) = columns[1];
        let (_, body_detail_bounds) = app.track_column_body_bounds(full_bounds, detail_bounds);
        let detail_label_rect =
            super::timeline_subcolumn_label_rect(body_detail_bounds, app.timeline_flow);
        let (_, slot_rect) = app.stored_loop_slot_rects(detail_label_rect)[0];

        let control = app.handle_timeline_pointer(
            content_bounds,
            slot_rect.x + slot_rect.width() as i32 / 2,
            slot_rect.y + slot_rect.height() as i32 / 2,
            ActionSource::Pointer,
        );

        assert_eq!(control, Some(AppControl::Continue));
        assert_eq!(app.project.active_track_index, 1);
        assert_eq!(
            app.project.tracks[1].loop_region,
            crate::timeline::LoopRegion::new(2_880, 960)
        );
    }

    #[test]
    fn timeline_thru_hit_rect_matches_rendered_subcolumn_header() {
        let app = App::new();
        let content_bounds = Rect::new(40, 40, 1200, 620);
        let (_, body_bounds) =
            crate::ui::split_top_strip(content_bounds, 28, 6).expect("timeline content");
        let (_, timeline_bounds) =
            crate::ui::split_top_strip(body_bounds, transport_strip_height(), 8)
                .expect("timeline body");
        let columns = crate::ui::track_column_pairs(timeline_bounds, app.project.tracks.len());
        let (full_bounds, detail_bounds) = columns[0];
        let (body_full_bounds, _) = app.track_column_body_bounds(full_bounds, detail_bounds);
        let label_rect = super::timeline_subcolumn_label_rect(body_full_bounds, app.timeline_flow);
        let content_rect =
            super::timeline_subcolumn_content_rect(body_full_bounds, app.timeline_flow);
        let thru_rect = app.track_passthrough_button_rect(label_rect);
        let intersects = |a: Rect, b: Rect| {
            a.x < b.x + b.width() as i32
                && a.x + a.width() as i32 > b.x
                && a.y < b.y + b.height() as i32
                && a.y + a.height() as i32 > b.y
        };

        assert!(super::rect_contains(
            label_rect,
            thru_rect.x + thru_rect.width() as i32 / 2,
            thru_rect.y + thru_rect.height() as i32 / 2,
        ));
        assert!(!intersects(thru_rect, content_rect));
    }

    #[test]
    fn click_below_thru_does_not_toggle_passthrough() {
        let mut app = App::new();
        let content_bounds = Rect::new(40, 40, 1200, 620);
        let (_, body_bounds) =
            crate::ui::split_top_strip(content_bounds, 28, 6).expect("timeline content");
        let (_, timeline_bounds) =
            crate::ui::split_top_strip(body_bounds, transport_strip_height(), 8)
                .expect("timeline body");
        let columns = crate::ui::track_column_pairs(timeline_bounds, app.project.tracks.len());
        let (full_bounds, detail_bounds) = columns[0];
        let (body_full_bounds, _) = app.track_column_body_bounds(full_bounds, detail_bounds);
        let label_rect = super::timeline_subcolumn_label_rect(body_full_bounds, app.timeline_flow);
        let thru_rect = app.track_passthrough_button_rect(label_rect);
        let before = app.project.tracks[0].state.passthrough;
        let below_y = thru_rect.y + thru_rect.height() as i32 + 2;

        let control = app.handle_timeline_pointer(
            content_bounds,
            thru_rect.x + thru_rect.width() as i32 / 2,
            below_y,
            ActionSource::Pointer,
        );

        assert!(matches!(control, None | Some(AppControl::Continue)));
        assert_eq!(app.project.tracks[0].state.passthrough, before);
    }

    #[test]
    fn reverse_activate_page_item_moves_timeline_fx_field_backward() {
        let mut app = App::new();
        app.page_state.current_page = AppPage::Timeline;
        app.page_state.selected_timeline_context = TimelineContext::OutputFx;
        app.page_state.selected_timeline_fx_field = TimelineFxField::ParamSecondary;

        app.reverse_activate_page_item();

        assert_eq!(
            app.page_state.selected_timeline_fx_field,
            TimelineFxField::ParamPrimary
        );
    }

    #[test]
    fn stopped_live_arp_ticks_without_advancing_playhead() {
        let mut app = App::new();
        app.project.clear_all_track_content();
        app.project.tracks[0].routing.input_port = Some(MidiPortRef::new("In A"));
        app.project.tracks[0].routing.output_port = Some(MidiPortRef::new("Out A"));
        app.project.tracks[0].routing.output_channel = Some(1);
        app.project.tracks[0].state.passthrough = true;
        app.project.tracks[0].midi_fx.monitor_input_fx = true;
        app.project.tracks[0].midi_fx.input_fx = vec![None; MIDI_FX_SLOT_COUNT];
        app.project.tracks[0].midi_fx.input_fx[0] = Some(MidiFxSlot {
            enabled: true,
            effect: MidiFx::Arp {
                step_ticks: 240,
                order: crate::midi_fx::ArpOrder::Up,
                gate_percent: 100,
            },
        });

        let input_port = app.project.tracks[0].routing.input_port.clone().unwrap();
        app.handle_midi_input_event(MidiInputEvent {
            port: input_port.clone(),
            channel: 1,
            message: MidiInputMessage::NoteOn {
                pitch: 60,
                velocity: 100,
            },
        });
        app.handle_midi_input_event(MidiInputEvent {
            port: input_port,
            channel: 1,
            message: MidiInputMessage::NoteOn {
                pitch: 64,
                velocity: 100,
            },
        });

        app.advance_playhead(Duration::from_millis(250));

        assert_eq!(app.transport_ticks, 0);
        assert_eq!(app.playhead_ticks, 0);
        assert!(app.live_fx_ticks > 0);
        let sent = app.midi_output.sent_messages();
        assert!(sent.iter().any(|(port, channel, pitch, velocity)| {
            port == "Out A" && *channel == 1 && *pitch == 60 && velocity.is_some()
        }));
    }

    #[test]
    fn live_fx_ticks_per_second_can_follow_external_tempo() {
        assert_eq!(ticks_per_second_for_tempo(120.0, 960), 1_920);
        assert_eq!(ticks_per_second_for_tempo(90.0, 960), 1_440);
    }

    #[test]
    fn timeline_kind_adjust_keeps_existing_row_visible() {
        let mut app = App::new();
        app.page_state.current_page = AppPage::Timeline;
        app.page_state.selected_timeline_context = TimelineContext::OutputFx;
        app.page_state.selected_timeline_fx_field = TimelineFxField::Kind;

        for _ in 0..16 {
            app.adjust_page_item(-1);
            assert!(app
                .selected_timeline_fx_slot(
                    app.project.active_track().unwrap(),
                    MidiFxChainKind::Output
                )
                .is_some());
        }
    }

    #[test]
    fn shift_m_action_toggles_selected_timeline_fx_when_fx_context_is_active() {
        let mut app = App::new();
        app.page_state.current_page = AppPage::Timeline;
        app.page_state.selected_timeline_context = TimelineContext::OutputFx;

        let before = app
            .selected_timeline_fx_slot(app.project.active_track().unwrap(), MidiFxChainKind::Output)
            .unwrap()
            .enabled;
        app.apply_action(AppAction::ToggleSelectedRecordingClipMute);
        let after = app
            .selected_timeline_fx_slot(app.project.active_track().unwrap(), MidiFxChainKind::Output)
            .unwrap()
            .enabled;
        assert_ne!(before, after);
    }

    #[test]
    fn timeline_add_row_adjust_inserts_even_when_non_kind_field_was_selected() {
        let mut app = App::new();
        app.page_state.current_page = AppPage::Timeline;
        app.page_state.selected_timeline_context = TimelineContext::OutputFx;
        let existing = app
            .active_timeline_fx_slot_indices(MidiFxChainKind::Output)
            .len();
        let add_row = app
            .displayed_timeline_fx_slot_indices(MidiFxChainKind::Output)
            .len()
            - 1;
        app.set_selected_timeline_fx_row(MidiFxChainKind::Output, add_row);
        app.page_state.selected_timeline_fx_field = TimelineFxField::Move;

        app.adjust_page_item(1);

        let after = app
            .active_timeline_fx_slot_indices(MidiFxChainKind::Output)
            .len();
        assert_eq!(after, existing + 1);
    }

    #[test]
    fn timeline_add_row_activate_inserts_new_fx() {
        let mut app = App::new();
        app.page_state.current_page = AppPage::Timeline;
        app.page_state.selected_timeline_context = TimelineContext::OutputFx;
        let existing = app
            .active_timeline_fx_slot_indices(MidiFxChainKind::Output)
            .len();
        let add_row = app
            .displayed_timeline_fx_slot_indices(MidiFxChainKind::Output)
            .len()
            - 1;
        app.set_selected_timeline_fx_row(MidiFxChainKind::Output, add_row);
        app.page_state.selected_timeline_fx_field = TimelineFxField::ParamSecondary;

        app.activate_page_item();

        let after = app
            .active_timeline_fx_slot_indices(MidiFxChainKind::Output)
            .len();
        assert_eq!(after, existing + 1);
    }

    #[test]
    fn timeline_add_row_kind_adjust_inserts_new_fx_into_empty_slot() {
        let mut app = App::new();
        app.page_state.current_page = AppPage::Timeline;
        app.page_state.selected_timeline_context = TimelineContext::OutputFx;
        let existing = app
            .active_timeline_fx_slot_indices(MidiFxChainKind::Output)
            .len();
        let add_row = app
            .displayed_timeline_fx_slot_indices(MidiFxChainKind::Output)
            .len()
            - 1;
        app.set_selected_timeline_fx_row(MidiFxChainKind::Output, add_row);
        app.page_state.selected_timeline_fx_field = TimelineFxField::Kind;

        app.adjust_page_item(1);

        let after = app
            .active_timeline_fx_slot_indices(MidiFxChainKind::Output)
            .len();
        assert_eq!(after, existing + 1);
    }

    #[test]
    fn timeline_add_row_selects_newly_inserted_fx_row() {
        let mut app = App::new();
        app.project.active_track_mut().unwrap().midi_fx.output_fx = vec![
            Some(MidiFxSlot::default()),
            Some(MidiFxSlot::default()),
            None,
            None,
        ];
        app.page_state.current_page = AppPage::Timeline;
        app.page_state.selected_timeline_context = TimelineContext::OutputFx;
        let add_row = app
            .displayed_timeline_fx_slot_indices(MidiFxChainKind::Output)
            .len()
            - 1;
        app.set_selected_timeline_fx_row(MidiFxChainKind::Output, add_row);
        app.page_state.selected_timeline_fx_field = TimelineFxField::Kind;

        app.adjust_page_item(1);

        assert_eq!(
            app.selected_timeline_fx_slot_index(MidiFxChainKind::Output),
            Some(2)
        );
        assert_eq!(
            app.selected_timeline_fx_active_row_index(MidiFxChainKind::Output),
            Some(2)
        );
    }

    #[test]
    fn timeline_move_after_insert_from_add_row_does_not_panic() {
        let mut app = App::new();
        app.project.active_track_mut().unwrap().midi_fx.output_fx = vec![
            Some(MidiFxSlot::default()),
            Some(MidiFxSlot::default()),
            None,
            None,
        ];
        app.page_state.current_page = AppPage::Timeline;
        app.page_state.selected_timeline_context = TimelineContext::OutputFx;
        let add_row = app
            .displayed_timeline_fx_slot_indices(MidiFxChainKind::Output)
            .len()
            - 1;
        app.set_selected_timeline_fx_row(MidiFxChainKind::Output, add_row);
        app.page_state.selected_timeline_fx_field = TimelineFxField::Kind;
        app.adjust_page_item(1);

        app.page_state.selected_timeline_fx_field = TimelineFxField::Move;
        app.adjust_page_item(1);

        assert!(app
            .selected_timeline_fx_slot(app.project.active_track().unwrap(), MidiFxChainKind::Output)
            .is_some());
    }

    #[test]
    fn manual_loop_edit_unlinks_active_stored_loop_slot() {
        let mut app = App::new();
        let track = app.project.active_track_mut().unwrap();
        track.loop_region = crate::timeline::LoopRegion::new(1_920, 960);
        assert!(track.store_current_loop_to_slot(2));
        assert_eq!(track.active_stored_loop_slot(), Some(2));

        app.transport_ticks = 0;
        app.apply_action(AppAction::SetCurrentTrackLoopStart);

        assert_eq!(
            app.project
                .active_track()
                .unwrap()
                .active_stored_loop_slot(),
            None
        );
    }

    #[test]
    fn quantized_stored_loop_recall_queues_and_resolves_at_boundary() {
        let mut app = App::new();
        app.project.transport.stored_loop_recall_quantized = true;
        app.project.transport.stored_loop_launch_quantize =
            crate::transport::LaunchQuantizeMode::Quarter;
        app.project.transport.playing = true;
        app.transport_ticks = 1_000;
        app.playhead_ticks = 1_000;

        {
            let track = app.project.active_track_mut().unwrap();
            track.loop_region = crate::timeline::LoopRegion::new(0, 960);
            assert!(track.store_current_loop_to_slot(0));
            track.loop_region = crate::timeline::LoopRegion::new(1_920, 960);
            assert!(track.store_current_loop_to_slot(1));
            track.loop_region = crate::timeline::LoopRegion::new(0, 960);
        }

        app.apply_action(AppAction::RecallStoredLoopSlot2);

        let track = app.project.active_track().unwrap();
        assert_eq!(track.active_stored_loop_slot(), None);
        assert_eq!(track.queued_stored_loop_slot(), Some(1));
        assert_eq!(track.loop_region, crate::timeline::LoopRegion::new(0, 960));

        app.process_queued_stored_loop_recalls(1_000, 1_920);
        let track = app.project.active_track().unwrap();
        assert_eq!(track.active_stored_loop_slot(), Some(1));
        assert_eq!(track.queued_stored_loop_slot(), None);
    }

    #[test]
    fn stored_loop_recall_is_blocked_on_recording_track() {
        let mut app = App::new();
        app.project.transport.stored_loop_recall_quantized = true;
        app.project.transport.stored_loop_launch_quantize =
            crate::transport::LaunchQuantizeMode::Bar;
        app.project.transport.playing = true;

        let track = app.project.active_track_mut().unwrap();
        track.loop_region = crate::timeline::LoopRegion::new(960, 960);
        assert!(track.store_current_loop_to_slot(0));
        track.loop_region = crate::timeline::LoopRegion::new(0, 960);
        track.begin_recording(0);

        app.apply_action(AppAction::RecallStoredLoopSlot1);

        let track = app.project.active_track().unwrap();
        assert_eq!(track.loop_region, crate::timeline::LoopRegion::new(0, 960));
        assert_eq!(track.queued_stored_loop_slot(), None);
        assert_eq!(track.active_stored_loop_slot(), None);
    }

    #[test]
    fn stored_loop_recall_enables_track_loop_before_queueing() {
        let mut app = App::new();
        app.project.transport.stored_loop_recall_quantized = true;
        app.project.transport.stored_loop_launch_quantize =
            crate::transport::LaunchQuantizeMode::LoopEnd;
        app.project.transport.playing = true;
        app.transport_ticks = 1_000;
        app.playhead_ticks = 1_000;

        let track = app.project.active_track_mut().unwrap();
        track.state.loop_enabled = false;
        track.loop_region = crate::timeline::LoopRegion::new(0, 960);
        assert!(track.store_current_loop_to_slot(0));

        app.apply_action(AppAction::RecallStoredLoopSlot1);

        let track = app.project.active_track().unwrap();
        assert!(track.state.loop_enabled);
        assert_eq!(track.queued_stored_loop_slot(), Some(0));
    }

    #[test]
    fn stored_loop_slot_rects_expand_to_fit_available_label_width() {
        let app = App::new();
        let wide = Rect::new(0, 0, 120, 14);
        let narrow = Rect::new(0, 0, 44, 14);

        assert_eq!(
            app.stored_loop_slot_rects(wide).len(),
            STORED_LOOP_SLOT_COUNT
        );
        assert!(app.stored_loop_slot_rects(narrow).len() < STORED_LOOP_SLOT_COUNT);
    }

    #[test]
    fn interlaced_color_pattern_cycles_proportionally() {
        let b = Color::RGB(0, 0, 255);
        let r = Color::RGB(255, 0, 0);
        let g = Color::RGB(0, 255, 0);

        let two = [b, r];
        assert_eq!(
            (0..4)
                .filter_map(|pixel| super::interlaced_color_at(&two, pixel))
                .collect::<Vec<_>>(),
            vec![b, r, b, r]
        );

        let three = [r, b, g];
        assert_eq!(
            (0..6)
                .filter_map(|pixel| super::interlaced_color_at(&three, pixel))
                .collect::<Vec<_>>(),
            vec![r, b, g, r, b, g]
        );
    }
}
