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
    search_mapping_targets, MappingEntry, MappingSourceKind,
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
use sdl3::render::{Canvas, FRect, RenderTarget};
use sdl3::surface::SurfaceRef;
use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};

#[path = "app/capture.rs"]
mod capture;
#[path = "app/io_pages.rs"]
mod io_pages;
#[path = "app/labels.rs"]
mod labels;
#[path = "app/mapping_ui.rs"]
mod mapping_ui;
#[path = "app/timeline_ui.rs"]
mod timeline_ui;
#[path = "app/types.rs"]
mod types;

use capture::{
    capture_specs, chip_row_width, readback_color_at, readback_rect_rgba, seed_capture_demo_track,
};
use labels::{
    action_source_label, badge_kind_prefix, compact_badge_text, compact_scope_label,
    launch_quantize_label, mapping_badge_palette, mapping_field_index, mapping_source_label,
    mapping_source_sort_key, quantize_label,
};
use mapping_ui::{
    direct_mapping_key_label, mapping_target_label_for_action, mapping_target_lookup_input,
    track_indicator_target,
};
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

    fn draw_frame_surface(
        &self,
        pixel_format: PixelFormat,
    ) -> Result<sdl3::surface::Surface<'static>, Box<dyn std::error::Error>> {
        let width = self.viewport_size.0.max(1);
        let height = self.viewport_size.1.max(1);
        let surface = sdl3::surface::Surface::new(width, height, pixel_format)?;
        let mut canvas = surface.into_canvas()?;
        self.draw(&mut canvas)?;
        Ok(canvas.into_surface())
    }

    fn draw<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.draw_scene(canvas)?;
        canvas.present();
        Ok(())
    }

    fn page_frame_layout(
        &self,
        inset: Rect,
    ) -> Result<(Rect, Rect, Rect), Box<dyn std::error::Error>> {
        let (tabs_bounds, page_area_bounds) = crate::ui::split_top_strip(inset, 28, 12)?;
        let footer_height = 22_u32;
        let footer_gap = 8_i32;
        let footer_bounds = Rect::new(
            page_area_bounds.x,
            page_area_bounds.y + page_area_bounds.height() as i32 - footer_height as i32,
            page_area_bounds.width(),
            footer_height,
        );
        let content_bounds = Rect::new(
            page_area_bounds.x,
            page_area_bounds.y,
            page_area_bounds.width(),
            page_area_bounds
                .height()
                .saturating_sub(footer_height)
                .saturating_sub(footer_gap as u32),
        );
        Ok((tabs_bounds, content_bounds, footer_bounds))
    }

    fn draw_scene<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (width, height) = active_draw_size(canvas.output_size()?, self.viewport_size);
        let surface = crate::ui::surface_rect(width, height);
        let inset = crate::ui::inset_rect(surface, 24, 24)?;
        let (tabs_bounds, content_bounds, footer_bounds) = self.page_frame_layout(inset)?;

        canvas.set_draw_color(Color::RGB(18, 24, 38));
        canvas.clear();

        canvas.set_draw_color(Color::RGB(28, 34, 50));
        canvas.fill_rect(surface)?;
        canvas.set_draw_color(Color::RGB(88, 96, 120));
        canvas.draw_rect(surface)?;

        if preferred_branding_width(tabs_bounds.width()) == 0 {
            self.draw_frame_brand_fallback(canvas, surface)?;
        }
        self.draw_page_tabs(canvas, tabs_bounds)?;

        render_page(self.page_state.current_page, self, canvas, content_bounds)?;

        self.draw_direct_mapping_targets(canvas, tabs_bounds, content_bounds)?;
        self.draw_overlay(canvas, inset)?;
        self.draw_footer(canvas, footer_bounds)?;
        Ok(())
    }

    fn draw_frame_brand_fallback<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        surface: Rect,
    ) -> Result<(), Box<dyn std::error::Error>> {
        branding::draw_frame_brand_fallback(canvas, surface)
    }

    fn configure_window_canvas(
        &mut self,
        canvas: &mut Canvas<sdl3::video::Window>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let scale = effective_ui_scale(canvas.window().display_scale(), self.ui_scale_override);
        let output_size = canvas.output_size()?;
        self.viewport_size = logical_viewport_size(output_size, scale);
        canvas.set_scale(scale, scale)?;
        Ok(())
    }

    fn draw_window(
        &self,
        canvas: &mut Canvas<sdl3::video::Window>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (scale_x, scale_y) = canvas.scale();
        if !should_interpolate_window_scale(self.ui_scaling_mode, scale_x, scale_y) {
            return self.draw(canvas);
        }

        let output_size = canvas.output_size()?;
        let logical_size = self.viewport_size;
        let texture_creator = canvas.texture_creator();
        let mut frame = texture_creator.create_texture_target(
            Some(texture_creator.default_pixel_format()),
            logical_size.0.max(1),
            logical_size.1.max(1),
        )?;
        frame.set_scale_mode(sdl3::render::ScaleMode::Linear);

        let mut draw_result: Result<(), Box<dyn std::error::Error>> = Ok(());
        canvas.with_texture_canvas(&mut frame, |texture_canvas| {
            draw_result = (|| -> Result<(), Box<dyn std::error::Error>> {
                texture_canvas.set_scale(1.0, 1.0)?;
                self.draw_scene(texture_canvas)
            })();
        })?;
        draw_result?;

        canvas.set_scale(1.0, 1.0)?;
        canvas.set_draw_color(Color::RGB(18, 24, 38));
        canvas.clear();
        canvas.copy(
            &frame,
            None,
            FRect::new(
                0.0,
                0.0,
                output_size.0.max(1) as f32,
                output_size.1.max(1) as f32,
            ),
        )?;
        canvas.present();
        canvas.set_scale(scale_x, scale_y)?;
        Ok(())
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

    fn draw_overlay<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        bounds: Rect,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match self.overlay_state.active {
            Some(AppOverlay::MappingsQuickView) => self.draw_mappings_overlay(canvas, bounds),
            Some(AppOverlay::Discoverability) | None => Ok(()),
        }
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

    fn draw_page_tabs<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        bounds: Rect,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (branding_bounds, tabs_bounds) = page_tabs_layout(bounds);
        self.draw_branding(canvas, branding_bounds)?;

        let tabs = crate::ui::equal_columns(tabs_bounds, AppPage::ALL.len(), 10);
        for (index, page) in AppPage::ALL.iter().copied().enumerate() {
            let tab = tabs[index];
            let active = page == self.page_state.current_page;
            canvas.set_draw_color(if active {
                Color::RGB(72, 96, 142)
            } else {
                Color::RGB(34, 44, 64)
            });
            canvas.fill_rect(tab)?;
            canvas.set_draw_color(if active {
                Color::RGB(248, 236, 162)
            } else {
                Color::RGB(92, 100, 120)
            });
            canvas.draw_rect(tab)?;

            let accent = Rect::new(tab.x + 6, tab.y + 6, 18, tab.height().saturating_sub(12));
            let color = match page {
                AppPage::Timeline => Color::RGB(84, 144, 220),
                AppPage::Mappings => Color::RGB(212, 168, 84),
                AppPage::MidiIo => Color::RGB(96, 200, 164),
                AppPage::Routing => Color::RGB(224, 112, 112),
            };
            canvas.set_draw_color(color);
            canvas.fill_rect(accent)?;
            crate::ui::draw_text_fitted(
                canvas,
                page.label(),
                Rect::new(tab.x + 30, tab.y + 8, tab.width().saturating_sub(36), 8),
                1,
                if active {
                    Color::RGB(248, 244, 212)
                } else {
                    Color::RGB(188, 194, 206)
                },
            )?;
        }

        Ok(())
    }

    fn draw_branding<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        bounds: Rect,
    ) -> Result<(), Box<dyn std::error::Error>> {
        branding::draw_branding(canvas, bounds, self.startup_started_at.elapsed())
    }

    fn draw_track_subcolumn<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        bounds: Rect,
        accent: Color,
        view_start_ticks: u64,
        range_ticks: u64,
        playhead_ticks: u64,
        is_active: bool,
        detail: bool,
        track: &Track,
    ) -> Result<(), Box<dyn std::error::Error>> {
        canvas.set_draw_color(if track.state.muted {
            Color::RGB(16, 18, 24)
        } else {
            Color::RGB(20, 27, 40)
        });
        canvas.fill_rect(bounds)?;
        canvas.set_draw_color(if track.state.soloed {
            Color::RGB(124, 214, 132)
        } else if is_active {
            Color::RGB(240, 222, 116)
        } else {
            Color::RGB(88, 96, 120)
        });
        canvas.draw_rect(bounds)?;
        if track.state.passthrough {
            canvas.set_draw_color(Color::RGB(74, 210, 214));
            canvas.fill_rect(Rect::new(
                bounds.x + 1,
                bounds.y + 1,
                2,
                bounds.height().saturating_sub(2),
            ))?;
        }

        let label_rect = timeline_subcolumn_label_rect(bounds, self.timeline_flow);
        let content_rect = timeline_subcolumn_content_rect(bounds, self.timeline_flow);
        canvas.set_draw_color(accent);
        canvas.fill_rect(label_rect)?;

        if !detail && track.state.loop_enabled {
            let loop_highlight = crate::ui::range_highlight_rect(
                content_rect,
                self.timeline_flow,
                view_start_ticks,
                range_ticks.max(1),
                track.loop_region,
            );
            canvas.set_draw_color(if is_active {
                Color::RGB(88, 72, 24)
            } else {
                Color::RGB(54, 48, 28)
            });
            canvas.fill_rect(loop_highlight)?;
        }

        for guide in crate::ui::timeline_guides(content_rect, self.timeline_flow) {
            canvas.set_draw_color(Color::RGB(52, 62, 84));
            canvas.fill_rect(guide)?;
        }
        let top_row_y = label_rect.y + 3;
        let bottom_row_y = label_rect.y + label_rect.height() as i32 - 10;
        let clip_controls = if !detail && track.selected_recording_clip().is_some() {
            Some(self.recording_clip_control_rects(label_rect))
        } else {
            None
        };
        let name_right = clip_controls
            .map(|(mute_rect, _)| mute_rect.x - 4)
            .unwrap_or(label_rect.x + label_rect.width() as i32 - 4);
        let label_left = if detail {
            let slot_rects = self.stored_loop_slot_rects(label_rect);
            let active_slot = track.active_stored_loop_slot();
            let queued_slot = track.queued_stored_loop_slot();
            for (slot_index, slot_rect) in &slot_rects {
                let filled = track.stored_loop_slot(*slot_index).is_some();
                let active = active_slot == Some(*slot_index);
                let queued = queued_slot == Some(*slot_index);
                canvas.set_draw_color(if active {
                    Color::RGB(238, 186, 112)
                } else if queued {
                    Color::RGB(104, 146, 172)
                } else if filled {
                    Color::RGB(132, 118, 98)
                } else {
                    Color::RGB(72, 70, 68)
                });
                canvas.fill_rect(*slot_rect)?;
                canvas.set_draw_color(if active {
                    Color::RGB(252, 228, 164)
                } else if queued {
                    Color::RGB(176, 222, 246)
                } else if filled {
                    Color::RGB(184, 168, 138)
                } else {
                    Color::RGB(122, 120, 116)
                });
                canvas.draw_rect(*slot_rect)?;
                crate::ui::draw_text_fitted(
                    canvas,
                    &(slot_index + 1).to_string(),
                    Rect::new(
                        slot_rect.x + 1,
                        slot_rect.y + 1,
                        slot_rect.width().saturating_sub(2),
                        slot_rect.height().saturating_sub(2),
                    ),
                    1,
                    if active {
                        Color::RGB(26, 20, 16)
                    } else if queued {
                        Color::RGB(16, 26, 34)
                    } else if filled {
                        Color::RGB(38, 34, 28)
                    } else {
                        Color::RGB(180, 178, 172)
                    },
                )?;
            }
            if STORED_LOOP_SLOT_COUNT > slot_rects.len() {
                let overflow = format!("+{}", STORED_LOOP_SLOT_COUNT - slot_rects.len());
                if let Some((_, last_slot_rect)) = slot_rects.last() {
                    crate::ui::draw_text_fitted(
                        canvas,
                        &overflow,
                        Rect::new(
                            last_slot_rect.x + last_slot_rect.width() as i32 + 3,
                            last_slot_rect.y + 1,
                            14,
                            7,
                        ),
                        1,
                        Color::RGB(210, 194, 160),
                    )?;
                }
            }
            slot_rects
                .last()
                .map(|(_, rect)| rect.x + rect.width() as i32 + 5)
                .unwrap_or(label_rect.x + 4)
        } else {
            let passthrough_button = self.track_passthrough_button_rect(label_rect);
            canvas.set_draw_color(if track.state.passthrough {
                Color::RGB(74, 210, 214)
            } else {
                Color::RGB(44, 70, 94)
            });
            canvas.fill_rect(passthrough_button)?;
            canvas.set_draw_color(if track.state.passthrough {
                Color::RGB(210, 246, 248)
            } else {
                Color::RGB(144, 170, 194)
            });
            canvas.draw_rect(passthrough_button)?;
            crate::ui::draw_text_fitted(
                canvas,
                "THRU",
                Rect::new(
                    passthrough_button.x + 2,
                    passthrough_button.y + 1,
                    passthrough_button.width().saturating_sub(4),
                    passthrough_button.height().saturating_sub(2),
                ),
                1,
                if track.state.passthrough {
                    Color::RGB(10, 28, 34)
                } else {
                    Color::RGB(230, 236, 240)
                },
            )?;
            passthrough_button.x + passthrough_button.width() as i32 + 4
        };
        crate::ui::draw_text_fitted(
            canvas,
            &track.name,
            Rect::new(
                label_left,
                top_row_y,
                (name_right - label_left).max(0) as u32,
                8,
            ),
            1,
            Color::RGB(244, 244, 236),
        )?;

        let role_badge = if detail {
            crate::ui::detail_badge_rect(label_rect)
        } else {
            Rect::new(
                label_rect.x + 4,
                bottom_row_y,
                label_rect.width().saturating_sub(8).min(28),
                8,
            )
        };
        canvas.set_draw_color(if detail {
            if track.state.loop_enabled && self.project.transport.loop_enabled {
                Color::RGB(252, 192, 104)
            } else {
                Color::RGB(88, 82, 76)
            }
        } else {
            Color::RGB(38, 58, 90)
        });
        canvas.fill_rect(role_badge)?;
        canvas.set_draw_color(if detail {
            Color::RGB(238, 214, 172)
        } else {
            Color::RGB(188, 204, 226)
        });
        canvas.draw_rect(role_badge)?;
        crate::ui::draw_text_fitted(
            canvas,
            if detail { "LOOP" } else { "SONG" },
            Rect::new(
                role_badge.x + 2,
                role_badge.y + 1,
                role_badge.width().saturating_sub(4),
                role_badge.height().saturating_sub(2),
            ),
            1,
            if detail {
                Color::RGB(28, 22, 18)
            } else {
                Color::RGB(244, 244, 236)
            },
        )?;
        if !detail {
            self.draw_recording_view_controls(
                canvas,
                label_rect,
                content_rect,
                track,
                clip_controls,
            )?;
        }

        let note_range = crate::timeline::LoopRegion::new(view_start_ticks, range_ticks.max(1));
        let selected_note_indices = track.selected_note_indices();
        let focused_note_index = track.focused_note_index();
        let anchor_note_index = track.anchor_note_index();
        let preview_region = track.preview_region(
            self.project.transport,
            self.record_capture_ticks(track),
            self.record_context(track),
        );
        let preview_notes = track.preview_notes(
            self.project.transport,
            self.record_capture_ticks(track),
            self.record_context(track),
        );
        self.draw_track_recording_content(
            canvas,
            content_rect,
            track,
            note_range,
            is_active,
            detail,
            selected_note_indices.as_slice(),
            focused_note_index,
            anchor_note_index,
            preview_region,
            preview_notes.as_slice(),
        )?;

        if track.recording_view != RecordingView::Stacked {
            if let Some(preview_region) = preview_region {
                if preview_region.intersects(note_range) {
                    for region in crate::ui::region_rects(
                        content_rect,
                        &[preview_region],
                        note_range,
                        self.timeline_flow,
                    ) {
                        if detail {
                            canvas.set_draw_color(Color::RGBA(214, 72, 72, 124));
                            canvas.fill_rect(region.rect)?;
                        }
                        canvas.set_draw_color(Color::RGB(248, 122, 122));
                        canvas.draw_rect(region.rect)?;
                    }
                }
            }

            for note in crate::ui::note_rects(
                content_rect,
                preview_notes.as_slice(),
                note_range,
                self.timeline_flow,
            ) {
                canvas.set_draw_color(Color::RGBA(238, 108, 108, 176));
                canvas.fill_rect(note.rect)?;
                canvas.set_draw_color(Color::RGB(255, 176, 176));
                canvas.draw_rect(note.rect)?;
            }
        }

        self.draw_track_loop_markers(canvas, content_rect, note_range, track)?;

        let playhead = crate::ui::playhead_rect_in_range(
            content_rect,
            self.timeline_flow,
            view_start_ticks,
            range_ticks.max(1),
            playhead_ticks,
        )?;
        if !detail && track.recording_view == RecordingView::Stacked && is_active {
            canvas.set_draw_color(if self.project.transport.playing {
                Color::RGB(248, 240, 132)
            } else {
                Color::RGB(140, 150, 162)
            });
            canvas.fill_rect(playhead)?;
            self.draw_recording_clip_scrollbar(canvas, content_rect, track)?;
        } else {
            canvas.set_draw_color(if self.project.transport.playing {
                Color::RGB(248, 240, 132)
            } else {
                Color::RGB(140, 150, 162)
            });
            canvas.fill_rect(playhead)?;
        }
        for tick in crate::ui::timeline_ruler_ticks(content_rect, self.timeline_flow) {
            canvas.set_draw_color(Color::RGB(166, 178, 198));
            canvas.fill_rect(tick)?;
        }

        Ok(())
    }

    fn draw_recording_view_controls<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        label_rect: Rect,
        _content_rect: Rect,
        track: &Track,
        clip_controls: Option<(Rect, Rect)>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if track.recording_view == RecordingView::Stacked {
            let can_scroll_left = self.can_select_previous_recording_clip(track);
            let can_scroll_right = self.can_select_next_recording_clip(track);
            let (left_rect, right_rect) = self.recording_view_scroll_control_rects(label_rect);
            canvas.set_draw_color(if can_scroll_left {
                Color::RGB(74, 82, 98)
            } else {
                Color::RGB(48, 54, 68)
            });
            canvas.fill_rect(left_rect)?;
            canvas.set_draw_color(if can_scroll_left {
                Color::RGB(202, 212, 224)
            } else {
                Color::RGB(112, 118, 130)
            });
            canvas.draw_rect(left_rect)?;
            crate::ui::draw_text_fitted(
                canvas,
                "<",
                Rect::new(
                    left_rect.x + 6,
                    left_rect.y + 1,
                    left_rect.width().saturating_sub(12),
                    8,
                ),
                1,
                if can_scroll_left {
                    Color::RGB(244, 244, 236)
                } else {
                    Color::RGB(144, 150, 160)
                },
            )?;
            canvas.set_draw_color(if can_scroll_right {
                Color::RGB(74, 82, 98)
            } else {
                Color::RGB(48, 54, 68)
            });
            canvas.fill_rect(right_rect)?;
            canvas.set_draw_color(if can_scroll_right {
                Color::RGB(202, 212, 224)
            } else {
                Color::RGB(112, 118, 130)
            });
            canvas.draw_rect(right_rect)?;
            crate::ui::draw_text_fitted(
                canvas,
                ">",
                Rect::new(
                    right_rect.x + 6,
                    right_rect.y + 1,
                    right_rect.width().saturating_sub(12),
                    8,
                ),
                1,
                if can_scroll_right {
                    Color::RGB(244, 244, 236)
                } else {
                    Color::RGB(144, 150, 160)
                },
            )?;
        }
        let view_rect = self.recording_view_chip_rect(label_rect);
        canvas.set_draw_color(match track.recording_view {
            RecordingView::Overlay => Color::RGB(50, 84, 126),
            RecordingView::Stacked => Color::RGB(124, 98, 48),
        });
        canvas.fill_rect(view_rect)?;
        canvas.set_draw_color(Color::RGB(232, 228, 208));
        canvas.draw_rect(view_rect)?;
        crate::ui::draw_text_fitted(
            canvas,
            match track.recording_view {
                RecordingView::Overlay => "OVR",
                RecordingView::Stacked => "STK",
            },
            Rect::new(
                view_rect.x + 3,
                view_rect.y + 1,
                view_rect.width().saturating_sub(6),
                view_rect.height().saturating_sub(2),
            ),
            1,
            Color::RGB(248, 244, 236),
        )?;

        if let (Some(selected_clip), Some((mute_rect, delete_rect))) =
            (track.selected_recording_clip(), clip_controls)
        {
            canvas.set_draw_color(if selected_clip.muted {
                Color::RGB(120, 118, 112)
            } else {
                Color::RGB(84, 122, 92)
            });
            canvas.fill_rect(mute_rect)?;
            canvas.set_draw_color(Color::RGB(228, 232, 216));
            canvas.draw_rect(mute_rect)?;
            crate::ui::draw_text_fitted(
                canvas,
                if selected_clip.muted { "ON" } else { "M" },
                Rect::new(
                    mute_rect.x + 2,
                    mute_rect.y + 1,
                    mute_rect.width().saturating_sub(4),
                    mute_rect.height().saturating_sub(2),
                ),
                1,
                Color::RGB(246, 244, 236),
            )?;

            canvas.set_draw_color(Color::RGB(132, 74, 70));
            canvas.fill_rect(delete_rect)?;
            canvas.set_draw_color(Color::RGB(240, 220, 210));
            canvas.draw_rect(delete_rect)?;
            crate::ui::draw_text_fitted(
                canvas,
                "X",
                Rect::new(
                    delete_rect.x + 2,
                    delete_rect.y + 1,
                    delete_rect.width().saturating_sub(4),
                    delete_rect.height().saturating_sub(2),
                ),
                1,
                Color::RGB(250, 242, 236),
            )?;
        }

        Ok(())
    }

    fn draw_track_recording_content<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        content_rect: Rect,
        track: &Track,
        note_range: crate::timeline::LoopRegion,
        is_active: bool,
        detail: bool,
        selected_note_indices: &[usize],
        focused_note_index: Option<usize>,
        anchor_note_index: Option<usize>,
        preview_region: Option<crate::timeline::Region>,
        preview_notes: &[crate::project::MidiNote],
    ) -> Result<(), Box<dyn std::error::Error>> {
        if track.recording_view == RecordingView::Stacked
            && (!track.recording_clips().is_empty() || preview_region.is_some())
        {
            let unowned_regions: Vec<_> = track
                .regions
                .iter()
                .copied()
                .filter(|region| region.recording_clip_id.is_none())
                .collect();
            let unowned_notes = indexed_notes(track, None);
            self.draw_region_entries(
                canvas,
                content_rect,
                &unowned_regions,
                note_range,
                track,
                is_active,
                track.state.muted,
            )?;
            self.draw_note_entries(
                canvas,
                content_rect,
                &unowned_notes,
                note_range,
                track,
                detail,
                track.state.muted,
                selected_note_indices,
                focused_note_index,
                anchor_note_index,
            )?;

            for lane in self.recording_lane_layouts(content_rect, track) {
                canvas.set_draw_color(if lane.preview {
                    Color::RGB(54, 32, 36)
                } else if lane.selected {
                    Color::RGB(46, 62, 94)
                } else {
                    Color::RGB(26, 34, 48)
                });
                canvas.fill_rect(lane.rect)?;
                canvas.set_draw_color(if lane.preview {
                    Color::RGB(248, 122, 122)
                } else if lane.selected {
                    Color::RGB(248, 226, 134)
                } else {
                    Color::RGB(76, 92, 118)
                });
                canvas.draw_rect(lane.rect)?;

                if lane.preview {
                    if let Some(preview_region) =
                        preview_region.filter(|region| region.intersects(note_range))
                    {
                        for region in crate::ui::region_rects(
                            lane.rect,
                            &[preview_region],
                            note_range,
                            self.timeline_flow,
                        ) {
                            if detail {
                                canvas.set_draw_color(Color::RGBA(214, 72, 72, 124));
                                canvas.fill_rect(region.rect)?;
                            }
                            canvas.set_draw_color(Color::RGB(248, 122, 122));
                            canvas.draw_rect(region.rect)?;
                        }
                    }

                    for note in crate::ui::note_rects(
                        lane.rect,
                        preview_notes,
                        note_range,
                        self.timeline_flow,
                    ) {
                        canvas.set_draw_color(Color::RGBA(238, 108, 108, 176));
                        canvas.fill_rect(note.rect)?;
                        canvas.set_draw_color(Color::RGB(255, 176, 176));
                        canvas.draw_rect(note.rect)?;
                    }
                } else if let Some(clip_id) = lane.clip_id {
                    let lane_muted = track.state.muted || lane.muted;
                    let lane_regions: Vec<_> = track
                        .regions
                        .iter()
                        .copied()
                        .filter(|region| region.recording_clip_id == Some(clip_id))
                        .collect();
                    let lane_notes = indexed_notes(track, Some(clip_id));
                    self.draw_region_entries(
                        canvas,
                        lane.rect,
                        &lane_regions,
                        note_range,
                        track,
                        is_active,
                        lane_muted,
                    )?;
                    self.draw_note_entries(
                        canvas,
                        lane.rect,
                        &lane_notes,
                        note_range,
                        track,
                        detail,
                        lane_muted,
                        selected_note_indices,
                        focused_note_index,
                        anchor_note_index,
                    )?;
                }
            }

            return Ok(());
        }

        self.draw_region_entries(
            canvas,
            content_rect,
            &track.regions,
            note_range,
            track,
            is_active,
            track.state.muted,
        )?;
        self.draw_note_entries(
            canvas,
            content_rect,
            &indexed_all_notes(track),
            note_range,
            track,
            detail,
            track.state.muted,
            selected_note_indices,
            focused_note_index,
            anchor_note_index,
        )?;
        Ok(())
    }

    fn draw_region_entries<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        lane_rect: Rect,
        regions: &[crate::timeline::Region],
        note_range: crate::timeline::LoopRegion,
        track: &Track,
        is_active: bool,
        muted_override: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for source_region in regions.iter().copied() {
            let region_muted =
                muted_override || track.recording_clip_is_muted(source_region.recording_clip_id);
            let Some(region) = crate::ui::region_rects(
                lane_rect,
                &[source_region],
                note_range,
                self.timeline_flow,
            )
            .into_iter()
            .next() else {
                continue;
            };
            canvas.set_draw_color(if region.clipped {
                Color::RGB(108, 88, 56)
            } else if region_muted {
                Color::RGB(42, 46, 56)
            } else {
                Color::RGB(44, 54, 76)
            });
            canvas.fill_rect(region.rect)?;
            canvas.set_draw_color(if is_active {
                Color::RGB(212, 196, 122)
            } else {
                Color::RGB(96, 106, 126)
            });
            canvas.draw_rect(region.rect)?;
        }

        Ok(())
    }

    fn draw_note_entries<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        lane_rect: Rect,
        note_entries: &[(usize, crate::project::MidiNote)],
        note_range: crate::timeline::LoopRegion,
        track: &Track,
        detail: bool,
        muted_override: bool,
        selected_note_indices: &[usize],
        focused_note_index: Option<usize>,
        anchor_note_index: Option<usize>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let notes: Vec<_> = note_entries.iter().map(|(_, note)| *note).collect();
        for note in crate::ui::note_rects(lane_rect, &notes, note_range, self.timeline_flow) {
            let absolute_index = note_entries[note.source_index].0;
            let note_muted = muted_override
                || track
                    .recording_clip_is_muted(note_entries[note.source_index].1.recording_clip_id);
            let selected = selected_note_indices.contains(&absolute_index);
            let focused = focused_note_index == Some(absolute_index);
            let anchored = anchor_note_index == Some(absolute_index);
            canvas.set_draw_color(if selected && detail {
                Color::RGB(112, 174, 228)
            } else if selected {
                Color::RGB(88, 136, 194)
            } else if note_muted {
                Color::RGB(92, 100, 112)
            } else if note.clipped {
                Color::RGB(244, 204, 132)
            } else {
                Color::RGB(210, 222, 236)
            });
            canvas.fill_rect(note.rect)?;
            canvas.set_draw_color(if focused {
                Color::RGB(252, 246, 158)
            } else if anchored {
                Color::RGB(180, 226, 176)
            } else if selected {
                Color::RGB(224, 238, 248)
            } else if note_muted {
                Color::RGB(128, 134, 144)
            } else {
                Color::RGB(245, 247, 250)
            });
            canvas.draw_rect(note.rect)?;
            if focused {
                let inner = Rect::new(
                    note.rect.x + 1,
                    note.rect.y + 1,
                    note.rect.width().saturating_sub(2).max(1),
                    note.rect.height().saturating_sub(2).max(1),
                );
                canvas.set_draw_color(Color::RGB(252, 208, 88));
                canvas.draw_rect(inner)?;
            }
        }

        Ok(())
    }

    fn recording_lane_layouts(
        &self,
        content_rect: Rect,
        track: &Track,
    ) -> Vec<RecordingLaneLayout> {
        let gap = 2;
        let window = self.recording_lane_window(track, self.recording_lane_capacity(content_rect));
        let visible_clips = &track.recording_clips()[window.committed_start..window.committed_end];
        let lane_count = window.visible_committed + usize::from(window.show_preview);
        let lane_rects = match self.timeline_flow {
            TimelineFlow::DownwardColumns => {
                crate::ui::equal_columns(content_rect, lane_count, gap)
            }
            TimelineFlow::AcrossRows => crate::ui::stacked_rows(content_rect, lane_count, gap),
        };

        let mut layouts: Vec<_> = visible_clips
            .iter()
            .zip(lane_rects.iter().copied())
            .map(|(clip, rect)| RecordingLaneLayout {
                clip_id: Some(clip.id),
                rect,
                selected: track.selected_recording_clip_id == Some(clip.id),
                muted: clip.muted,
                preview: false,
            })
            .collect();

        if window.show_preview {
            if let Some(rect) = lane_rects.get(window.visible_committed).copied() {
                layouts.push(RecordingLaneLayout {
                    clip_id: None,
                    rect,
                    selected: false,
                    muted: false,
                    preview: true,
                });
            }
        }

        layouts
    }

    fn recording_lane_hit_clip(
        &self,
        content_rect: Rect,
        track: &Track,
        x: i32,
        y: i32,
    ) -> Option<u64> {
        self.recording_lane_layouts(content_rect, track)
            .into_iter()
            .find_map(|lane| {
                rect_contains(lane.rect, x, y)
                    .then_some(lane.clip_id)
                    .flatten()
            })
    }

    fn recording_lane_capacity(&self, content_rect: Rect) -> usize {
        match self.timeline_flow {
            TimelineFlow::DownwardColumns => {
                let min_lane_width = 15_i32;
                let gap = 2_i32;
                (((content_rect.width() as i32 + gap) / (min_lane_width + gap)).max(1)) as usize
            }
            TimelineFlow::AcrossRows => {
                let min_lane_height = 26_i32;
                let gap = 2_i32;
                (((content_rect.height() as i32 + gap) / (min_lane_height + gap)).max(1)) as usize
            }
        }
    }

    fn recording_view_chip_rect(&self, label_rect: Rect) -> Rect {
        let top_y = label_rect.y + label_rect.height() as i32 - 10;
        let right = label_rect.x + label_rect.width() as i32 - 4;
        Rect::new(right - 26, top_y, 26, 8)
    }

    fn track_passthrough_button_rect(&self, label_rect: Rect) -> Rect {
        Rect::new(
            label_rect.x + 4,
            label_rect.y + 3,
            label_rect.width().saturating_sub(8).min(30),
            8,
        )
    }

    fn stored_loop_visible_slot_count(&self, label_rect: Rect) -> usize {
        let slot_w = 8_i32;
        let gap = 2_i32;
        let side_padding = 8_i32;
        let min_name_space = 24_i32;
        let available = label_rect.width() as i32 - side_padding - min_name_space;
        if available < slot_w {
            return 0;
        }
        (((available + gap) / (slot_w + gap)).max(0) as usize).min(STORED_LOOP_SLOT_COUNT)
    }

    fn stored_loop_slot_rects(&self, label_rect: Rect) -> Vec<(usize, Rect)> {
        let visible_slots = self
            .stored_loop_visible_slot_count(label_rect)
            .min(STORED_LOOP_SLOT_COUNT);
        let slot_w = 8_u32;
        let slot_h = 7_u32;
        let gap = 2_i32;
        let mut rects = Vec::with_capacity(visible_slots);
        for slot_index in 0..visible_slots {
            rects.push((
                slot_index,
                Rect::new(
                    label_rect.x + 4 + slot_index as i32 * (slot_w as i32 + gap),
                    label_rect.y + 2,
                    slot_w,
                    slot_h,
                ),
            ));
        }
        rects
    }

    fn track_column_body_bounds(&self, full_bounds: Rect, detail_bounds: Rect) -> (Rect, Rect) {
        let pair_bounds = crate::ui::union_rect(full_bounds, detail_bounds);
        let status_rect = crate::ui::track_status_rect(pair_bounds, self.timeline_flow);
        let (top_band_height, bottom_band_height) = self.timeline_fx_band_heights();
        let top_gap = 4_i32;
        let bottom_gap = 4_i32;
        let top_reserve = (status_rect.y + status_rect.height() as i32 + top_gap + top_band_height
            - pair_bounds.y)
            .max(0);
        let bottom_reserve = (bottom_gap + bottom_band_height).max(0);
        let new_height = full_bounds
            .height()
            .saturating_sub(top_reserve as u32)
            .saturating_sub(bottom_reserve as u32);
        let full = Rect::new(
            full_bounds.x,
            full_bounds.y + top_reserve,
            full_bounds.width(),
            new_height,
        );
        let detail = Rect::new(
            detail_bounds.x,
            detail_bounds.y + top_reserve,
            detail_bounds.width(),
            new_height,
        );
        (full, detail)
    }

    fn timeline_fx_band_heights(&self) -> (i32, i32) {
        let input = self
            .project
            .tracks
            .iter()
            .map(|track| displayed_track_fx_band_height(&track.midi_fx.input_fx))
            .max()
            .unwrap_or(displayed_track_fx_band_height(&[]));
        let output = self
            .project
            .tracks
            .iter()
            .map(|track| displayed_track_fx_band_height(&track.midi_fx.output_fx))
            .max()
            .unwrap_or(displayed_track_fx_band_height(&[]));
        (input, output)
    }

    fn track_fx_band_rects(
        &self,
        full_bounds: Rect,
        detail_bounds: Rect,
        _track: &Track,
    ) -> (Rect, Rect) {
        let pair_bounds = crate::ui::union_rect(full_bounds, detail_bounds);
        let status_rect = crate::ui::track_status_rect(pair_bounds, self.timeline_flow);
        let (body_full_bounds, body_detail_bounds) =
            self.track_column_body_bounds(full_bounds, detail_bounds);
        let body_pair_bounds = crate::ui::union_rect(body_full_bounds, body_detail_bounds);
        let (top_band_height, bottom_band_height) = self.timeline_fx_band_heights();
        let top = Rect::new(
            pair_bounds.x + 4,
            status_rect.y + status_rect.height() as i32 + 4,
            pair_bounds.width().saturating_sub(8),
            top_band_height as u32,
        );
        let bottom = Rect::new(
            pair_bounds.x + 4,
            body_pair_bounds.y + body_pair_bounds.height() as i32 + 4,
            pair_bounds.width().saturating_sub(8),
            bottom_band_height as u32,
        );
        (top, bottom)
    }

    fn draw_track_fx_bands<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        layout: TimelineTrackLayout,
        track: &Track,
        is_active: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for (context, rect) in [
            (TimelineContext::InputFx, layout.input_fx_rect),
            (TimelineContext::OutputFx, layout.output_fx_rect),
        ] {
            let chain_kind = context.chain_kind().expect("fx context");
            let chain = self.fx_chain(track, chain_kind);
            let active_slots: Vec<(usize, &MidiFxSlot)> = chain
                .iter()
                .enumerate()
                .filter_map(|(index, slot)| slot.as_ref().map(|slot| (index, slot)))
                .collect();
            let displayed_rows =
                self.displayed_timeline_fx_slot_indices_for_track(track, chain_kind);
            let enabled = active_slots.iter().any(|(_, slot)| slot.enabled);
            let fill = if context == TimelineContext::InputFx {
                if enabled {
                    Color::RGB(78, 128, 198)
                } else if is_active {
                    Color::RGB(56, 70, 94)
                } else {
                    Color::RGB(46, 56, 74)
                }
            } else if enabled {
                Color::RGB(172, 108, 156)
            } else if is_active {
                Color::RGB(84, 68, 94)
            } else {
                Color::RGB(64, 58, 76)
            };
            let border = if enabled {
                Color::RGB(236, 238, 228)
            } else if is_active {
                Color::RGB(176, 184, 198)
            } else {
                Color::RGB(120, 126, 140)
            };
            canvas.set_draw_color(fill);
            canvas.fill_rect(rect)?;
            canvas.set_draw_color(border);
            canvas.draw_rect(rect)?;

            let selected_row = if is_active && self.page_state.selected_timeline_context == context
            {
                self.selected_timeline_fx_row(chain_kind)
            } else {
                usize::MAX
            };
            let layouts =
                self.timeline_fx_row_layouts(rect, &displayed_rows, chain, Some(selected_row));
            for (line_index, (display_row, layout)) in
                displayed_rows.iter().zip(layouts.iter()).enumerate()
            {
                let selected = line_index == selected_row;
                if let Some(slot_index) = display_row {
                    let slot = chain[*slot_index].as_ref().expect("timeline slot");
                    let text_color = if slot.enabled {
                        Color::RGB(248, 244, 236)
                    } else {
                        Color::RGB(198, 202, 210)
                    };
                    self.draw_timeline_fx_row(
                        canvas,
                        context,
                        *slot_index,
                        slot,
                        *layout,
                        selected,
                        text_color,
                    )?;
                } else {
                    self.draw_timeline_fx_add_row(canvas, context, *layout, selected)?;
                }
            }
        }
        Ok(())
    }

    fn timeline_fx_row_layouts(
        &self,
        band_rect: Rect,
        displayed_rows: &[Option<usize>],
        chain: &[Option<MidiFxSlot>],
        _selected_row: Option<usize>,
    ) -> Vec<TimelineFxRowLayout> {
        fn empty_row_rect(row: Rect) -> Rect {
            Rect::new(-10_000, row.y, 1, 1)
        }

        fn take_right(row: Rect, right: &mut i32, width: i32, gap: i32) -> Rect {
            if width <= 0 || *right - width < row.x {
                return empty_row_rect(row);
            }
            let rect = Rect::new(*right - width, row.y, width as u32, row.height());
            *right = rect.x - gap;
            rect
        }

        let row_count = displayed_rows.len().max(1);
        let line_height = 8_i32;
        let line_gap = 2_i32;
        let top_padding = 2_i32;
        let row_y = band_rect.y + top_padding;
        let row_width = band_rect.width().saturating_sub(4);
        let rows: Vec<Rect> = (0..row_count)
            .map(|row_index| {
                Rect::new(
                    band_rect.x + 2,
                    row_y + row_index as i32 * (line_height + line_gap),
                    row_width,
                    line_height as u32,
                )
            })
            .collect();
        rows.into_iter()
            .enumerate()
            .map(|(row_index, row)| {
                let gap = 1;
                let available = row.width() as i32;
                let enabled_width = available.clamp(10, 14);
                let delete_width = available.clamp(5, 6);
                let param_min_width = if available >= 72 { 18 } else { 12 };
                let move_width = if available >= 132 { 6 } else { 0 };
                let (kind_width, visible_param_count, total_param_count) = displayed_rows
                    .get(row_index)
                    .and_then(|slot_index| slot_index.and_then(|index| chain.get(index)))
                    .and_then(|slot| slot.as_ref())
                    .map(|slot| {
                        (
                            timeline_fx_kind_target_width(slot, available as u32) as i32,
                            slot.effect.inline_parameters().len().min(2),
                            slot.effect.inline_parameters().len(),
                        )
                    })
                    .unwrap_or((12, 0, 0));

                let enabled = Rect::new(row.x, row.y, enabled_width as u32, row.height());
                let kind_x = enabled.x + enabled.width() as i32 + gap;
                let kind = Rect::new(kind_x, row.y, kind_width.max(0) as u32, row.height());
                let params_x = kind.x + kind.width() as i32 + gap;
                let mut move_down_width = move_width;
                let mut move_up_width = move_width;
                let mut overflow_width = if total_param_count > 2 {
                    if available >= 72 {
                        10
                    } else {
                        8
                    }
                } else {
                    0
                };
                let mut show_secondary = visible_param_count >= 2;
                loop {
                    let right_fixed_width = delete_width
                        + move_down_width
                        + move_up_width
                        + overflow_width
                        + gap // kind -> params
                        + gap; // params -> delete
                    let right_fixed_gaps = i32::from(move_down_width > 0)
                        + i32::from(move_up_width > 0)
                        + i32::from(overflow_width > 0);
                    let params_total_width = available
                        - enabled_width
                        - kind_width
                        - right_fixed_width
                        - right_fixed_gaps * gap
                        - gap; // enabled -> kind
                    let required_param_width = if show_secondary {
                        param_min_width * 2 + gap
                    } else {
                        param_min_width
                    };
                    if params_total_width >= required_param_width {
                        let mut right = row.x + row.width() as i32;
                        let delete = take_right(row, &mut right, delete_width, gap);
                        let move_down = take_right(row, &mut right, move_down_width, gap);
                        let move_up = take_right(row, &mut right, move_up_width, gap);
                        let overflow = take_right(row, &mut right, overflow_width, gap);
                        let param_right = delete.x - gap;
                        let available_param_width = (param_right - params_x).max(0);
                        let (param_primary, param_secondary) = if show_secondary {
                            let primary_width = (available_param_width - gap) / 2;
                            let secondary_width = available_param_width - gap - primary_width;
                            let primary = Rect::new(
                                params_x,
                                row.y,
                                primary_width.max(0) as u32,
                                row.height(),
                            );
                            let secondary_x = primary.x + primary.width() as i32 + gap;
                            let secondary = Rect::new(
                                secondary_x,
                                row.y,
                                secondary_width.max(0) as u32,
                                row.height(),
                            );
                            (primary, secondary)
                        } else {
                            (
                                Rect::new(
                                    params_x,
                                    row.y,
                                    available_param_width.max(0) as u32,
                                    row.height(),
                                ),
                                empty_row_rect(row),
                            )
                        };
                        return TimelineFxRowLayout {
                            row,
                            enabled,
                            kind,
                            param_primary,
                            param_secondary,
                            overflow,
                            move_up,
                            move_down,
                            delete,
                        };
                    }

                    if move_down_width > 0 {
                        move_down_width = 0;
                    } else if move_up_width > 0 {
                        move_up_width = 0;
                    } else if overflow_width > 0 {
                        overflow_width = 0;
                    } else if show_secondary {
                        show_secondary = false;
                    } else {
                        let mut right = row.x + row.width() as i32;
                        let delete = take_right(row, &mut right, delete_width, gap);
                        let move_down = empty_row_rect(row);
                        let move_up = empty_row_rect(row);
                        let overflow = empty_row_rect(row);
                        let param_right = delete.x - gap;
                        let available_param_width = (param_right - params_x).max(0);
                        let param_primary = Rect::new(
                            params_x,
                            row.y,
                            available_param_width.max(0) as u32,
                            row.height(),
                        );
                        return TimelineFxRowLayout {
                            row,
                            enabled,
                            kind,
                            param_primary,
                            param_secondary: empty_row_rect(row),
                            overflow,
                            move_up,
                            move_down,
                            delete,
                        };
                    }
                }
            })
            .collect()
    }

    fn draw_track_loop_markers<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        content_rect: Rect,
        note_range: crate::timeline::LoopRegion,
        track: &Track,
    ) -> Result<(), Box<dyn std::error::Error>> {
        #[derive(Clone)]
        struct LoopMarker {
            range: crate::timeline::LoopRegion,
            label: String,
            color: Color,
            emphasized: bool,
            queued: bool,
        }
        #[derive(Clone, Copy)]
        struct MarkerSpan {
            color: Color,
            start: i32,
            end: i32,
        }

        let active_slot = track.active_stored_loop_slot();
        let queued_slot = track.queued_stored_loop_slot();
        let mut markers = Vec::new();
        for slot_index in 0..STORED_LOOP_SLOT_COUNT {
            let Some(stored_loop) = track.stored_loop_slot(slot_index) else {
                continue;
            };
            markers.push(LoopMarker {
                range: stored_loop.as_loop_region(),
                label: (slot_index + 1).to_string(),
                color: stored_loop_slot_color(slot_index),
                emphasized: active_slot == Some(slot_index),
                queued: queued_slot == Some(slot_index),
            });
        }

        if active_slot.is_none() {
            markers.push(LoopMarker {
                range: track.loop_region,
                label: "L".to_string(),
                color: if track.state.loop_enabled {
                    Color::RGB(242, 190, 112)
                } else {
                    Color::RGB(128, 122, 112)
                },
                emphasized: true,
                queued: false,
            });
        }

        let mut spans = Vec::new();
        for marker in markers.iter() {
            if !loop_regions_intersect(marker.range, note_range) {
                continue;
            }

            let span_rect = crate::ui::range_highlight_rect(
                content_rect,
                self.timeline_flow,
                note_range.start_ticks,
                note_range.length_ticks,
                marker.range,
            );
            let (start, end) = match self.timeline_flow {
                TimelineFlow::DownwardColumns => (
                    span_rect.y,
                    span_rect.y + span_rect.height().max(1) as i32 - 1,
                ),
                TimelineFlow::AcrossRows => (
                    span_rect.x,
                    span_rect.x + span_rect.width().max(1) as i32 - 1,
                ),
            };
            spans.push(MarkerSpan {
                color: marker.color,
                start,
                end,
            });
        }

        if spans.is_empty() {
            return Ok(());
        }

        let side_thickness = 4_i32;
        let primary_tick = Color::RGB(252, 238, 194);
        let queued_tick = Color::RGB(184, 226, 248);
        let secondary_tick = Color::RGB(218, 224, 232);
        let side_major = side_thickness.max(1) as u32;
        let content_bg = if track.state.muted {
            Color::RGB(16, 18, 24)
        } else {
            Color::RGB(20, 27, 40)
        };

        match self.timeline_flow {
            TimelineFlow::DownwardColumns => {
                let x = content_rect.x + 1;
                let usable_width = (content_rect.x + content_rect.width() as i32 - x).max(1);
                let band_width = side_major.min(usable_width as u32);
                let start_y = content_rect.y;
                let end_y = content_rect.y + content_rect.height() as i32 - 2;
                if end_y < start_y {
                    return Ok(());
                }
                let mut placed_label_rects = Vec::new();
                let label_spacing = 9_i32;

                for y in start_y..=end_y {
                    let colors = spans
                        .iter()
                        .filter(|span| y >= span.start && y <= span.end)
                        .map(|span| span.color)
                        .collect::<Vec<_>>();
                    if colors.is_empty() {
                        continue;
                    }
                    if let Some(color) = interlaced_color_at(&colors, (y - start_y).max(0) as usize)
                    {
                        canvas.set_draw_color(color);
                        canvas.fill_rect(Rect::new(x, y, band_width, 1))?;
                    }
                }

                for marker in markers.iter() {
                    if !loop_regions_intersect(marker.range, note_range) {
                        continue;
                    }
                    let span_rect = crate::ui::range_highlight_rect(
                        content_rect,
                        self.timeline_flow,
                        note_range.start_ticks,
                        note_range.length_ticks,
                        marker.range,
                    );
                    let line_h = span_rect.height().max(1);
                    let marker_start_y = span_rect.y.clamp(start_y, end_y);
                    let end_marker_y = (span_rect.y + line_h as i32 - 1).clamp(start_y, end_y);
                    if marker_start_y > end_marker_y {
                        continue;
                    }
                    canvas.set_draw_color(if marker.emphasized {
                        primary_tick
                    } else if marker.queued {
                        queued_tick
                    } else {
                        secondary_tick
                    });
                    canvas.fill_rect(Rect::new(x, marker_start_y, band_width.min(4), 1))?;
                    canvas.fill_rect(Rect::new(x, end_marker_y, band_width.min(4), 1))?;

                    let marker_mid_y = marker_start_y + (end_marker_y - marker_start_y) / 2;
                    let label_y = (marker_mid_y - 3).clamp(
                        content_rect.y,
                        content_rect.y + content_rect.height() as i32 - 7,
                    );
                    let mut label_rect = Rect::new(x + band_width as i32 + 3, label_y, 8, 7);
                    for offset_step in 0..8 {
                        let candidate = Rect::new(
                            x + band_width as i32 + 3 + offset_step * label_spacing,
                            label_y,
                            8,
                            7,
                        );
                        if !placed_label_rects
                            .iter()
                            .any(|existing| rects_overlap(*existing, candidate))
                        {
                            label_rect = candidate;
                            break;
                        }
                    }
                    placed_label_rects.push(label_rect);
                    let label_readback = readback_rect_rgba(canvas, label_rect, self.viewport_size);
                    crate::ui::draw_text_fitted_inverted(
                        canvas,
                        marker.label.as_str(),
                        label_rect,
                        1,
                        |px, py| readback_color_at(&label_readback, px, py).unwrap_or(content_bg),
                    )?;
                    if marker.emphasized {
                        draw_loop_label_underline(
                            canvas,
                            marker.label.as_str(),
                            label_rect,
                            content_rect,
                            self.viewport_size,
                            content_bg,
                        )?;
                    } else if marker.queued {
                        canvas.set_draw_color(queued_tick);
                        canvas.fill_rect(Rect::new(
                            label_rect.x,
                            (label_rect.y + label_rect.height() as i32 + 1)
                                .min(content_rect.y + content_rect.height() as i32 - 1),
                            label_rect.width().min(4),
                            1,
                        ))?;
                    }
                }
            }
            TimelineFlow::AcrossRows => {
                let y = content_rect.y + 1;
                let usable_height = (content_rect.y + content_rect.height() as i32 - y).max(1);
                let band_height = side_major.min(usable_height as u32);
                let start_x = content_rect.x;
                let end_x = content_rect.x + content_rect.width() as i32 - 1;
                let mut placed_label_rects = Vec::new();
                let label_spacing = 9_i32;

                for x in start_x..=end_x {
                    let colors = spans
                        .iter()
                        .filter(|span| x >= span.start && x <= span.end)
                        .map(|span| span.color)
                        .collect::<Vec<_>>();
                    if colors.is_empty() {
                        continue;
                    }
                    for pixel in 0..band_height as usize {
                        if let Some(color) = interlaced_color_at(&colors, pixel) {
                            canvas.set_draw_color(color);
                            canvas.fill_rect(Rect::new(x, y + pixel as i32, 1, 1))?;
                        }
                    }
                }

                for marker in markers.iter() {
                    if !loop_regions_intersect(marker.range, note_range) {
                        continue;
                    }
                    let span_rect = crate::ui::range_highlight_rect(
                        content_rect,
                        self.timeline_flow,
                        note_range.start_ticks,
                        note_range.length_ticks,
                        marker.range,
                    );
                    let line_w = span_rect.width().max(1);
                    let end_marker_x = span_rect.x + line_w as i32 - 1;
                    canvas.set_draw_color(if marker.emphasized {
                        primary_tick
                    } else if marker.queued {
                        queued_tick
                    } else {
                        secondary_tick
                    });
                    canvas.fill_rect(Rect::new(span_rect.x, y, 1, band_height.min(4)))?;
                    canvas.fill_rect(Rect::new(end_marker_x, y, 1, band_height.min(4)))?;

                    let label_x = (span_rect.x + line_w as i32 / 2 - 3).clamp(
                        content_rect.x,
                        content_rect.x + content_rect.width() as i32 - 7,
                    );
                    let mut label_rect = Rect::new(label_x, y + band_height as i32 + 3, 7, 6);
                    for offset_step in 0..8 {
                        let candidate =
                            Rect::new(label_x + offset_step * label_spacing, label_rect.y, 7, 6);
                        if !placed_label_rects
                            .iter()
                            .any(|existing| rects_overlap(*existing, candidate))
                        {
                            label_rect = candidate;
                            break;
                        }
                    }
                    placed_label_rects.push(label_rect);
                    let label_readback = readback_rect_rgba(canvas, label_rect, self.viewport_size);
                    crate::ui::draw_text_fitted_inverted(
                        canvas,
                        marker.label.as_str(),
                        label_rect,
                        1,
                        |px, py| readback_color_at(&label_readback, px, py).unwrap_or(content_bg),
                    )?;
                    if marker.emphasized {
                        draw_loop_label_underline(
                            canvas,
                            marker.label.as_str(),
                            label_rect,
                            content_rect,
                            self.viewport_size,
                            content_bg,
                        )?;
                    } else if marker.queued {
                        canvas.set_draw_color(queued_tick);
                        canvas.fill_rect(Rect::new(
                            label_rect.x,
                            (label_rect.y + label_rect.height() as i32 + 1)
                                .min(content_rect.y + content_rect.height() as i32 - 1),
                            label_rect.width().min(4),
                            1,
                        ))?;
                    }
                }
            }
        }

        Ok(())
    }

    fn recording_view_scroll_control_rects(&self, label_rect: Rect) -> (Rect, Rect) {
        let top_y = label_rect.y + label_rect.height() as i32 - 10;
        let view_rect = self.recording_view_chip_rect(label_rect);
        let right_rect = Rect::new(view_rect.x - 16, top_y, 12, 8);
        let left_rect = Rect::new(right_rect.x - 14, top_y, 12, 8);
        (left_rect, right_rect)
    }

    fn selected_recording_clip_index(&self, track: &Track) -> Option<usize> {
        track.selected_recording_clip_id.and_then(|selected_id| {
            track
                .recording_clips()
                .iter()
                .position(|clip| clip.id == selected_id)
        })
    }

    fn can_select_previous_recording_clip(&self, track: &Track) -> bool {
        self.selected_recording_clip_index(track)
            .map(|index| index > 0)
            .unwrap_or(false)
    }

    fn can_select_next_recording_clip(&self, track: &Track) -> bool {
        self.selected_recording_clip_index(track)
            .map(|index| index + 1 < track.recording_clips().len())
            .unwrap_or(false)
    }

    fn sync_active_track_recording_clip_scroll(&mut self) {
        let Some(full_bounds) = self.active_track_full_bounds() else {
            return;
        };
        let content_rect = crate::ui::track_content_rect(full_bounds, self.timeline_flow);
        let total_capacity = self.recording_lane_capacity(content_rect).max(1);
        let Some(track) = self.project.active_track_mut() else {
            return;
        };
        let total_lanes = track.recording_clips.len() + usize::from(track.active_take.is_some());
        if total_lanes == 0 {
            track.recording_clip_scroll = 0;
            return;
        }

        let visible_lanes = total_capacity.min(total_lanes);
        let max_start = total_lanes.saturating_sub(visible_lanes);
        track.recording_clip_scroll = track.recording_clip_scroll.min(max_start);
        if track.active_take.is_some() {
            track.recording_clip_scroll = track.recording_clip_scroll.max(max_start);
        }
        let Some(selected_id) = track.selected_recording_clip_id else {
            return;
        };
        if track.active_take.is_some() {
            return;
        }
        let Some(selected_index) = track
            .recording_clips
            .iter()
            .position(|clip| clip.id == selected_id)
        else {
            return;
        };
        if selected_index < track.recording_clip_scroll {
            track.recording_clip_scroll = selected_index;
        } else if selected_index >= track.recording_clip_scroll + visible_lanes {
            track.recording_clip_scroll = selected_index + 1 - visible_lanes;
        }
    }

    fn recording_clip_scroll_control_hit(
        &self,
        label_rect: Rect,
        track: &Track,
        x: i32,
        y: i32,
    ) -> Option<AppAction> {
        if track.recording_view != RecordingView::Stacked {
            return None;
        }
        let (left_rect, right_rect) = self.recording_view_scroll_control_rects(label_rect);
        if rect_contains(left_rect, x, y) && self.can_select_previous_recording_clip(track) {
            return Some(AppAction::SelectPreviousRecordingClip);
        }
        if rect_contains(right_rect, x, y) && self.can_select_next_recording_clip(track) {
            return Some(AppAction::SelectNextRecordingClip);
        }
        None
    }

    fn recording_clip_control_rects(&self, label_rect: Rect) -> (Rect, Rect) {
        let top_y = label_rect.y + 3;
        let right = label_rect.x + label_rect.width() as i32 - 4;
        (
            Rect::new(right - 28, top_y, 12, 8),
            Rect::new(right - 12, top_y, 12, 8),
        )
    }

    fn recording_clip_scrollbar_rects(
        &self,
        content_rect: Rect,
        track: &Track,
    ) -> Option<(Rect, Rect)> {
        if track.recording_view != RecordingView::Stacked {
            return None;
        }

        let total_lanes = track.recording_clips.len() + usize::from(track.active_take.is_some());
        if total_lanes == 0 {
            return None;
        }

        let window = self.recording_lane_window(track, self.recording_lane_capacity(content_rect));
        let visible_lanes = window.visible_total.clamp(1, total_lanes);
        let start = window.start;
        let rail = Rect::new(
            content_rect.x + 4,
            content_rect.y,
            content_rect.width().saturating_sub(8),
            2,
        );
        if rail.width() == 0 {
            return None;
        }

        let thumb_width = ((rail.width() as usize * visible_lanes) / total_lanes)
            .max(6)
            .min(rail.width() as usize) as u32;
        let max_offset = rail.width().saturating_sub(thumb_width) as i32;
        let max_start = total_lanes.saturating_sub(visible_lanes);
        let thumb_x = if max_start == 0 {
            rail.x
        } else {
            rail.x + (max_offset as i64 * start as i64 / max_start as i64) as i32
        };
        let thumb = Rect::new(thumb_x, rail.y, thumb_width, 2);
        Some((rail, thumb))
    }

    fn draw_recording_clip_scrollbar<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        content_rect: Rect,
        track: &Track,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let Some((rail, thumb)) = self.recording_clip_scrollbar_rects(content_rect, track) else {
            return Ok(());
        };
        canvas.set_draw_color(Color::RGB(92, 100, 120));
        canvas.fill_rect(rail)?;
        canvas.set_draw_color(Color::RGB(244, 214, 118));
        canvas.fill_rect(thumb)?;
        Ok(())
    }

    fn recording_lane_window(&self, track: &Track, total_capacity: usize) -> RecordingLaneWindow {
        let total_capacity = total_capacity.max(1);
        let committed_len = track.recording_clips().len();
        let preview_index = track.active_take.as_ref().map(|_| committed_len);
        let total_lanes = committed_len + usize::from(preview_index.is_some());
        if total_lanes == 0 {
            return RecordingLaneWindow {
                start: 0,
                visible_total: 0,
                committed_start: 0,
                committed_end: 0,
                visible_committed: 0,
                show_preview: false,
            };
        }

        let visible_total = total_capacity.min(total_lanes);
        let max_start = total_lanes.saturating_sub(visible_total);
        let mut start = track.recording_clip_scroll.min(max_start);
        if let Some(preview_index) = preview_index {
            start = start.max(preview_index + 1 - visible_total);
        }
        let end = start + visible_total;
        let committed_start = start.min(committed_len);
        let committed_end = end.min(committed_len);
        let show_preview = preview_index.is_some_and(|preview_index| preview_index >= start);

        RecordingLaneWindow {
            start,
            visible_total,
            committed_start,
            committed_end,
            visible_committed: committed_end.saturating_sub(committed_start),
            show_preview,
        }
    }

    fn draw_transport_chip<T: RenderTarget>(
        canvas: &mut Canvas<T>,
        chip: Rect,
        spec: &TransportChipSpec,
    ) -> Result<(), Box<dyn std::error::Error>> {
        canvas.set_draw_color(spec.fill);
        canvas.fill_rect(chip)?;
        crate::ui::draw_text_fitted(
            canvas,
            &spec.label,
            Rect::new(chip.x + 5, chip.y + 2, chip.width().saturating_sub(10), 8),
            1,
            Color::RGB(244, 244, 236),
        )?;
        Ok(())
    }

    fn draw_footer<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        bounds: Rect,
    ) -> Result<(), Box<dyn std::error::Error>> {
        canvas.set_draw_color(Color::RGB(20, 26, 38));
        canvas.fill_rect(bounds)?;
        canvas.set_draw_color(Color::RGB(88, 96, 120));
        canvas.draw_rect(bounds)?;

        let overlay_chips = [
            (
                "F5 Mappings",
                self.overlay_state.active == Some(AppOverlay::MappingsQuickView),
                Color::RGB(156, 122, 68),
            ),
            (
                "F7 Discover",
                self.overlay_state.active == Some(AppOverlay::Discoverability),
                Color::RGB(72, 136, 166),
            ),
            (
                "F8 Direct",
                self.direct_mapping_state.mode != DirectMappingMode::Inactive,
                Color::RGB(188, 82, 82),
            ),
        ];
        let mut right_edge = bounds.x + bounds.width() as i32 - 6;
        for (label, active, color) in overlay_chips.into_iter().rev() {
            let width = crate::ui::text_width(label, 1) + 10;
            let chip = Rect::new(
                right_edge - width as i32,
                bounds.y + 5,
                width,
                bounds.height().saturating_sub(10),
            );
            canvas.set_draw_color(if active {
                color
            } else {
                Color::RGB(56, 66, 84)
            });
            canvas.fill_rect(chip)?;
            crate::ui::draw_text_fitted(
                canvas,
                label,
                Rect::new(chip.x + 5, chip.y + 2, chip.width().saturating_sub(10), 8),
                1,
                if active {
                    Color::RGB(248, 244, 214)
                } else {
                    Color::RGB(180, 190, 204)
                },
            )?;
            right_edge = chip.x - 6;
        }

        if let Some((title, detail, badges)) = self.direct_mapping_footer_content() {
            let label_width = crate::ui::text_width(&title, 1) + 4;
            let label_rect = Rect::new(bounds.x + 8, bounds.y + 7, label_width, 8);
            crate::ui::draw_text_fitted(canvas, &title, label_rect, 1, Color::RGB(248, 228, 208))?;
            let detail_left = label_rect.x + label_rect.width() as i32 + 8;
            let detail_width = (right_edge - detail_left).max(0) as u32;
            if !badges.is_empty() {
                self.draw_mapping_badges(
                    canvas,
                    Rect::new(
                        detail_left,
                        bounds.y + 3,
                        detail_width,
                        bounds.height().saturating_sub(6),
                    ),
                    &badges,
                    badges.len(),
                    4,
                    10,
                )?;
            } else {
                crate::ui::draw_text_fitted(
                    canvas,
                    &detail,
                    Rect::new(detail_left, bounds.y + 7, detail_width, 8),
                    1,
                    Color::RGB(214, 200, 188),
                )?;
            }
        } else if let Some(target) = self.status_state.hovered_target {
            let summary = self.summarize_discoverability_target(target);
            let label_width = crate::ui::text_width(&summary.title, 1) + 4;
            let label_rect = Rect::new(bounds.x + 8, bounds.y + 7, label_width, 8);
            crate::ui::draw_text_fitted(
                canvas,
                &summary.title,
                label_rect,
                1,
                Color::RGB(244, 244, 236),
            )?;
            let badges_left = label_rect.x + label_rect.width() as i32 + 8;
            let badges_width = (right_edge - badges_left).max(0) as u32;
            if summary.badges.is_empty() {
                crate::ui::draw_text_fitted(
                    canvas,
                    "No mappings",
                    Rect::new(badges_left, bounds.y + 7, badges_width, 8),
                    1,
                    Color::RGB(168, 178, 194),
                )?;
            } else {
                self.draw_mapping_badges(
                    canvas,
                    Rect::new(
                        badges_left,
                        bounds.y + 3,
                        badges_width,
                        bounds.height().saturating_sub(6),
                    ),
                    &summary.badges,
                    summary.total_bindings,
                    4,
                    10,
                )?;
            }
        } else if let Some((title, detail)) = self.timeline_fx_footer_content() {
            let label_width = crate::ui::text_width(&title, 1) + 4;
            let label_rect = Rect::new(bounds.x + 8, bounds.y + 7, label_width, 8);
            crate::ui::draw_text_fitted(canvas, &title, label_rect, 1, Color::RGB(244, 232, 146))?;
            crate::ui::draw_text_fitted(
                canvas,
                &detail,
                Rect::new(
                    label_rect.x + label_rect.width() as i32 + 8,
                    bounds.y + 7,
                    (right_edge - label_rect.x - label_rect.width() as i32 - 12).max(0) as u32,
                    8,
                ),
                1,
                Color::RGB(188, 198, 212),
            )?;
        } else {
            let last_action = self
                .status_state
                .last_action
                .map(|status| {
                    format!(
                        "Last Action: {} via {}",
                        action_label(status.action),
                        action_source_label(status.source)
                    )
                })
                .unwrap_or_else(|| "Last Action: Ready".to_string());
            crate::ui::draw_text_fitted(
                canvas,
                &last_action,
                Rect::new(
                    bounds.x + 8,
                    bounds.y + 7,
                    (right_edge - bounds.x - 12).max(0) as u32,
                    8,
                ),
                1,
                Color::RGB(188, 198, 212),
            )?;
        }

        Ok(())
    }

    fn timeline_fx_footer_content(&self) -> Option<(String, String)> {
        if self.page_state.current_page != AppPage::Timeline {
            return None;
        }
        let context = self.page_state.selected_timeline_context;
        let chain_kind = context.chain_kind()?;
        let track = self.project.active_track()?;
        if let Some(slot) = self.selected_timeline_fx_slot(track, chain_kind) {
            Some((
                format!(
                    "{} {}",
                    context.label(),
                    self.page_state.selected_timeline_fx_field.label()
                ),
                format!(
                    "Shift+Left/Right ctx  Up/Down row  Enter field  Q/E edit  Delete remove  {}",
                    slot.effect.kind().label()
                ),
            ))
        } else {
            Some((
                format!("{} Add", context.label()),
                "Shift+Left/Right ctx  Up/Down row  Q/E or click add row".to_string(),
            ))
        }
    }

    fn mapping_row_cells(&self, row: Rect) -> [Rect; 6] {
        let type_rect = Rect::new(row.x + 4, row.y + 3, 46, row.height().saturating_sub(6));
        let source_rect = Rect::new(
            type_rect.x + type_rect.width() as i32 + 6,
            row.y + 3,
            92,
            row.height().saturating_sub(6),
        );
        let device_rect = Rect::new(
            source_rect.x + source_rect.width() as i32 + 6,
            row.y + 3,
            98,
            row.height().saturating_sub(6),
        );
        let enabled_rect = Rect::new(
            row.x + row.width() as i32 - 34,
            row.y + 3,
            28,
            row.height().saturating_sub(6),
        );
        let scope_rect = Rect::new(
            enabled_rect.x - 80,
            row.y + 3,
            72,
            row.height().saturating_sub(6),
        );
        let target_rect = Rect::new(
            device_rect.x + device_rect.width() as i32 + 6,
            row.y + 3,
            (scope_rect.x - (device_rect.x + device_rect.width() as i32 + 12)).max(48) as u32,
            row.height().saturating_sub(6),
        );
        [
            type_rect,
            source_rect,
            device_rect,
            target_rect,
            scope_rect,
            enabled_rect,
        ]
    }

    pub(crate) fn draw_mappings_page<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        content_bounds: Rect,
    ) -> Result<(), Box<dyn std::error::Error>> {
        canvas.set_draw_color(Color::RGB(22, 28, 42));
        canvas.fill_rect(content_bounds)?;
        canvas.set_draw_color(Color::RGB(88, 96, 120));
        canvas.draw_rect(content_bounds)?;
        crate::ui::draw_text_fitted(
            canvas,
            "Mappings",
            Rect::new(content_bounds.x + 8, content_bounds.y + 8, 180, 14),
            2,
            Color::RGB(244, 232, 146),
        )?;
        let overview_badge = Rect::new(content_bounds.x + 200, content_bounds.y + 8, 188, 16);
        canvas.set_draw_color(if self.page_state.mapping_mode == MappingPageMode::Write {
            Color::RGB(74, 96, 138)
        } else {
            Color::RGB(50, 62, 88)
        });
        canvas.fill_rect(overview_badge)?;
        canvas.set_draw_color(Color::RGB(244, 232, 146));
        canvas.draw_rect(overview_badge)?;
        crate::ui::draw_text_fitted(
            canvas,
            &format!("Tap Mode: {}", self.page_state.mapping_mode.label()),
            Rect::new(content_bounds.x + 208, content_bounds.y + 12, 170, 8),
            1,
            Color::RGB(236, 242, 248),
        )?;
        let learn_badge = Rect::new(content_bounds.x + 392, content_bounds.y + 8, 136, 16);
        canvas.set_draw_color(if self.page_state.mapping_midi_learn_armed {
            Color::RGB(146, 62, 62)
        } else {
            Color::RGB(44, 56, 78)
        });
        canvas.fill_rect(learn_badge)?;
        canvas.set_draw_color(
            if self.page_state.selected_mapping_field == MappingField::SourceValue
                && self.page_state.mapping_mode == MappingPageMode::Write
            {
                Color::RGB(252, 232, 146)
            } else {
                Color::RGB(96, 108, 132)
            },
        );
        canvas.draw_rect(learn_badge)?;
        crate::ui::draw_text_fitted(
            canvas,
            if self.page_state.mapping_midi_learn_armed {
                "Tap Learn: waiting"
            } else {
                "Tap Learn: idle"
            },
            Rect::new(learn_badge.x + 8, learn_badge.y + 4, 120, 8),
            1,
            Color::RGB(236, 240, 246),
        )?;
        let direct_badge = Rect::new(content_bounds.x + 532, content_bounds.y + 8, 154, 16);
        canvas.set_draw_color(
            if self.direct_mapping_state.mode == DirectMappingMode::Inactive {
                Color::RGB(54, 62, 82)
            } else {
                Color::RGB(140, 74, 74)
            },
        );
        canvas.fill_rect(direct_badge)?;
        canvas.set_draw_color(
            if self.direct_mapping_state.mode == DirectMappingMode::Inactive {
                Color::RGB(108, 118, 138)
            } else {
                Color::RGB(252, 214, 194)
            },
        );
        canvas.draw_rect(direct_badge)?;
        crate::ui::draw_text_fitted(
            canvas,
            if self.direct_mapping_state.mode == DirectMappingMode::Inactive {
                "Tap Direct Map"
            } else {
                "Tap Direct: armed"
            },
            Rect::new(direct_badge.x + 8, direct_badge.y + 4, 138, 8),
            1,
            Color::RGB(242, 238, 234),
        )?;
        crate::ui::draw_text_fitted(
            canvas,
            &format!(
                "Rows {} / {}",
                self.page_state
                    .selected_mapping_index
                    .saturating_add(1)
                    .min(self.mappings.len()),
                self.mappings.len()
            ),
            Rect::new(
                content_bounds.x + content_bounds.width() as i32 - 100,
                content_bounds.y + 12,
                92,
                8,
            ),
            1,
            Color::RGB(154, 166, 182),
        )?;

        let footer_bounds = Rect::new(
            content_bounds.x + 8,
            content_bounds.y + content_bounds.height() as i32 - 20,
            content_bounds.width().saturating_sub(16),
            12,
        );
        let list_bounds = Rect::new(
            content_bounds.x + 8,
            content_bounds.y + 44,
            content_bounds.width().saturating_sub(16),
            content_bounds.height().saturating_sub(68),
        );
        let header_row = Rect::new(
            list_bounds.x,
            content_bounds.y + 30,
            list_bounds.width(),
            10,
        );
        let header_cells = self.mapping_row_cells(Rect::new(
            header_row.x,
            header_row.y,
            header_row.width(),
            18,
        ));
        for (index, field) in MappingField::ALL.iter().enumerate() {
            crate::ui::draw_text_fitted(
                canvas,
                field.label(),
                Rect::new(
                    header_cells[index].x,
                    header_row.y,
                    header_cells[index].width(),
                    8,
                ),
                1,
                Color::RGB(154, 166, 182),
            )?;
        }
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
            let entry = &self.mappings[index];
            let selected = index == self.page_state.selected_mapping_index;
            canvas.set_draw_color(if selected {
                Color::RGB(52, 64, 92)
            } else {
                Color::RGB(30, 36, 52)
            });
            canvas.fill_rect(row)?;
            canvas.set_draw_color(if selected {
                Color::RGB(244, 232, 146)
            } else {
                Color::RGB(78, 88, 110)
            });
            canvas.draw_rect(row)?;

            let cells = self.mapping_row_cells(row);
            let source_rect = Rect::new(cells[0].x, cells[0].y, 14, cells[0].height());
            let source_color = match entry.source_kind {
                MappingSourceKind::Key => Color::RGB(98, 148, 232),
                MappingSourceKind::Midi => Color::RGB(96, 202, 146),
                MappingSourceKind::Osc => Color::RGB(220, 154, 88),
            };
            canvas.set_draw_color(source_color);
            canvas.fill_rect(source_rect)?;

            let enabled_rect = Rect::new(cells[5].x + 6, cells[5].y, 14, cells[5].height());
            canvas.set_draw_color(if entry.enabled {
                Color::RGB(132, 220, 120)
            } else {
                Color::RGB(92, 96, 102)
            });
            canvas.fill_rect(enabled_rect)?;

            let kind_rect = cells[0];
            let device_rect = cells[1];
            let trigger_rect = cells[2];
            let target_rect = cells[3];
            let scope_rect = cells[4];
            canvas.set_draw_color(if selected {
                Color::RGB(66, 80, 112)
            } else {
                Color::RGB(42, 50, 70)
            });
            canvas.fill_rect(kind_rect)?;
            canvas.fill_rect(trigger_rect)?;
            canvas.fill_rect(device_rect)?;
            canvas.set_draw_color(if entry.enabled {
                Color::RGB(182, 194, 212)
            } else {
                Color::RGB(104, 112, 124)
            });
            canvas.fill_rect(target_rect)?;
            canvas.set_draw_color(Color::RGB(66, 74, 88));
            canvas.fill_rect(scope_rect)?;
            canvas.fill_rect(cells[5])?;
            if selected && self.page_state.mapping_mode == MappingPageMode::Write {
                let field_rect = cells[mapping_field_index(self.page_state.selected_mapping_field)];
                canvas.set_draw_color(
                    if self.page_state.mapping_midi_learn_armed
                        && self.page_state.selected_mapping_field == MappingField::SourceValue
                    {
                        Color::RGB(120, 42, 42)
                    } else {
                        Color::RGB(92, 98, 64)
                    },
                );
                canvas.fill_rect(field_rect)?;
            }
            crate::ui::draw_text_fitted(
                canvas,
                mapping_source_label(entry.source_kind),
                Rect::new(
                    kind_rect.x + 18,
                    row.y + 5,
                    kind_rect.width().saturating_sub(22),
                    8,
                ),
                1,
                Color::RGB(244, 244, 236),
            )?;
            crate::ui::draw_text_fitted(
                canvas,
                &entry.source_label,
                Rect::new(
                    trigger_rect.x + 4,
                    row.y + 5,
                    trigger_rect.width().saturating_sub(8),
                    8,
                ),
                1,
                Color::RGB(244, 244, 236),
            )?;
            let mapping_device_label = if entry.source_kind == MappingSourceKind::Midi {
                if entry.source_device_label != default_mapping_source_device()
                    && !self.input_port_is_available(&entry.source_device_label)
                {
                    format!("{} (offline)", entry.source_device_label)
                } else {
                    entry.source_device_label.clone()
                }
            } else {
                "--".to_string()
            };
            crate::ui::draw_text_fitted(
                canvas,
                &mapping_device_label,
                Rect::new(
                    device_rect.x + 4,
                    row.y + 5,
                    device_rect.width().saturating_sub(8),
                    8,
                ),
                1,
                if entry.source_kind == MappingSourceKind::Midi {
                    Color::RGB(226, 234, 244)
                } else {
                    Color::RGB(124, 132, 146)
                },
            )?;
            crate::ui::draw_text_fitted(
                canvas,
                &if selected
                    && self.page_state.mapping_mode == MappingPageMode::Write
                    && self.page_state.selected_mapping_field == MappingField::Target
                    && self.target_lookup_state.active.is_some()
                {
                    self.target_lookup_state
                        .active
                        .as_ref()
                        .map(|lookup| {
                            if lookup.query.is_empty() {
                                "Search target…".to_string()
                            } else {
                                format!("Search: {}", lookup.query)
                            }
                        })
                        .unwrap_or_else(|| entry.target_label.clone())
                } else {
                    entry.target_label.clone()
                },
                Rect::new(
                    target_rect.x + 4,
                    row.y + 5,
                    target_rect.width().saturating_sub(8),
                    8,
                ),
                1,
                Color::RGB(24, 28, 36),
            )?;
            crate::ui::draw_text_fitted(
                canvas,
                compact_scope_label(&entry.scope_label),
                Rect::new(
                    scope_rect.x + 4,
                    row.y + 5,
                    scope_rect.width().saturating_sub(8),
                    8,
                ),
                1,
                Color::RGB(236, 238, 242),
            )?;
            crate::ui::draw_text_fitted(
                canvas,
                if entry.enabled { "On" } else { "Off" },
                Rect::new(
                    cells[5].x + 2,
                    row.y + 5,
                    cells[5].width().saturating_sub(4),
                    8,
                ),
                1,
                Color::RGB(236, 238, 242),
            )?;

            if selected && self.page_state.mapping_mode == MappingPageMode::Write {
                let field_rect = cells[mapping_field_index(self.page_state.selected_mapping_field)];
                canvas.set_draw_color(
                    if self.page_state.mapping_midi_learn_armed
                        && self.page_state.selected_mapping_field == MappingField::SourceValue
                    {
                        Color::RGB(252, 126, 126)
                    } else {
                        Color::RGB(252, 232, 146)
                    },
                );
                canvas.draw_rect(field_rect)?;
                let tap_tag = Rect::new(row.x + row.width() as i32 - 68, row.y + 3, 34, 12);
                canvas.set_draw_color(Color::RGB(86, 98, 124));
                canvas.fill_rect(tap_tag)?;
                crate::ui::draw_text_fitted(
                    canvas,
                    "Tap",
                    Rect::new(
                        tap_tag.x + 6,
                        tap_tag.y + 2,
                        tap_tag.width().saturating_sub(12),
                        8,
                    ),
                    1,
                    Color::RGB(244, 244, 236),
                )?;
            }
        }

        self.draw_mapping_target_lookup(canvas, content_bounds)?;

        canvas.set_draw_color(Color::RGB(26, 32, 46));
        canvas.fill_rect(footer_bounds)?;
        let footer_tokens = [
            ("Tap row", Color::RGB(62, 78, 106)),
            ("Tap field", Color::RGB(74, 88, 118)),
            ("Tap again act", Color::RGB(82, 100, 136)),
            ("W Write", Color::RGB(96, 82, 52)),
            ("F8 Direct", Color::RGB(128, 78, 78)),
            ("N New", Color::RGB(66, 96, 84)),
            ("Del/Bsp Remove", Color::RGB(110, 74, 74)),
        ];
        let mut footer_x = footer_bounds.x + 6;
        for (label, fill) in footer_tokens {
            let token = Rect::new(
                footer_x,
                footer_bounds.y + 1,
                crate::ui::text_width(label, 1) + 12,
                footer_bounds.height().saturating_sub(2),
            );
            canvas.set_draw_color(fill);
            canvas.fill_rect(token)?;
            crate::ui::draw_text_fitted(
                canvas,
                label,
                Rect::new(
                    token.x + 6,
                    token.y + 2,
                    token.width().saturating_sub(12),
                    8,
                ),
                1,
                Color::RGB(244, 244, 236),
            )?;
            footer_x += token.width() as i32 + 6;
        }
        crate::ui::draw_text_fitted(
            canvas,
            if self.target_lookup_state.active.is_some() {
                "Type filter  Up/Down Select  Enter Commit  Esc Cancel  Tab stays in lookup"
            } else {
                "Shift+Left/Right Field  Q/E Adjust  Enter Learn/Toggle"
            },
            Rect::new(
                footer_x + 6,
                footer_bounds.y + 2,
                footer_bounds
                    .width()
                    .saturating_sub((footer_x - footer_bounds.x) as u32)
                    .saturating_sub(12),
                8,
            ),
            1,
            Color::RGB(154, 166, 182),
        )?;

        Ok(())
    }

    fn draw_mappings_overlay<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        bounds: Rect,
    ) -> Result<(), Box<dyn std::error::Error>> {
        canvas.set_draw_color(Color::RGBA(10, 14, 24, 220));
        canvas.fill_rect(bounds)?;

        let panel = Rect::new(
            bounds.x + 84,
            bounds.y + 44,
            bounds.width() - 168,
            bounds.height() - 88,
        );
        canvas.set_draw_color(Color::RGB(24, 30, 44));
        canvas.fill_rect(panel)?;
        canvas.set_draw_color(Color::RGB(244, 232, 146));
        canvas.draw_rect(panel)?;
        let title_bounds = Rect::new(panel.x + 12, panel.y + 12, 220, 14);
        crate::ui::draw_text_fitted(
            canvas,
            "Mappings Overlay",
            title_bounds,
            2,
            Color::RGB(244, 232, 146),
        )?;
        crate::ui::draw_text_fitted(
            canvas,
            "F5 Close",
            Rect::new(panel.x + 12, panel.y + 32, 58, 8),
            1,
            Color::RGB(188, 198, 212),
        )?;
        crate::ui::draw_text_fitted(
            canvas,
            "W Write",
            Rect::new(panel.x + 80, panel.y + 32, 52, 8),
            1,
            Color::RGB(188, 198, 212),
        )?;
        crate::ui::draw_text_fitted(
            canvas,
            "Trigger",
            Rect::new(panel.x + 12, panel.y + 46, 56, 8),
            1,
            Color::RGB(150, 162, 180),
        )?;
        crate::ui::draw_text_fitted(
            canvas,
            "Action",
            Rect::new(panel.x + 146, panel.y + 46, 48, 8),
            1,
            Color::RGB(150, 162, 180),
        )?;
        crate::ui::draw_text_fitted(
            canvas,
            "Scope",
            Rect::new(panel.x + panel.width() as i32 - 126, panel.y + 46, 44, 8),
            1,
            Color::RGB(150, 162, 180),
        )?;

        let list_bounds = crate::ui::inset_rect(panel, 12, 66)?;
        let row_height = 18_i32;
        let row_gap = 3_i32;
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
            let entry = &self.mappings[index];
            let selected = index == self.page_state.selected_mapping_index;
            canvas.set_draw_color(if selected {
                Color::RGB(58, 72, 102)
            } else {
                Color::RGB(34, 42, 60)
            });
            canvas.fill_rect(row)?;
            canvas.set_draw_color(if selected {
                Color::RGB(244, 232, 146)
            } else {
                Color::RGB(82, 92, 114)
            });
            canvas.draw_rect(row)?;

            crate::ui::draw_text_fitted(
                canvas,
                &entry.source_label,
                Rect::new(row.x + 8, row.y + 5, 126, 8),
                1,
                Color::RGB(244, 244, 236),
            )?;
            crate::ui::draw_text_fitted(
                canvas,
                &entry.target_label,
                Rect::new(row.x + 146, row.y + 5, 210, 8),
                1,
                Color::RGB(208, 220, 236),
            )?;
            crate::ui::draw_text_fitted(
                canvas,
                compact_scope_label(&entry.scope_label),
                Rect::new(row.x + row.width() as i32 - 126, row.y + 5, 90, 8),
                1,
                Color::RGB(182, 192, 210),
            )?;
        }

        crate::ui::draw_text_fitted(
            canvas,
            &format!(
                "Rows {}-{} / {}",
                start_index.saturating_add(1),
                (start_index + visible_rows).min(self.mappings.len()),
                self.mappings.len()
            ),
            Rect::new(panel.x + panel.width() as i32 - 116, panel.y + 34, 104, 8),
            1,
            Color::RGB(160, 170, 184),
        )?;

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

    fn toggle_direct_mapping_mode(&mut self) {
        self.clear_mapping_target_lookup();
        if self.direct_mapping_state.mode == DirectMappingMode::Inactive {
            self.direct_mapping_state.mode = DirectMappingMode::Targeting;
            self.direct_mapping_state.origin = if self.page_state.current_page == AppPage::Mappings
            {
                DirectMappingOrigin::MappingsPage
            } else {
                DirectMappingOrigin::InPlace
            };
            self.direct_mapping_state.status_message = None;
            self.page_state.mapping_midi_learn_armed = false;
            if self.overlay_state.active == Some(AppOverlay::MappingsQuickView) {
                self.overlay_state.active = None;
            }
        } else {
            self.cancel_direct_mapping("Canceled direct mapping.");
        }
        self.sync_midi_inputs();
    }

    fn cancel_direct_mapping(&mut self, message: &str) {
        self.clear_mapping_target_lookup();
        self.direct_mapping_state.mode = DirectMappingMode::Inactive;
        self.direct_mapping_state.origin = DirectMappingOrigin::InPlace;
        self.direct_mapping_state.status_message = Some(message.to_string());
        self.sync_midi_inputs();
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
        timeline_fx_overflow_label(param_count, window_start)
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

    fn capture_direct_mapping_input(&mut self, event: &MidiInputEvent) -> bool {
        let DirectMappingMode::AwaitingInput(target) = self.direct_mapping_state.mode else {
            return false;
        };

        match event.message {
            MidiInputMessage::NoteOn { .. } | MidiInputMessage::ControlChange { .. } => {}
            MidiInputMessage::NoteOff { .. } => return false,
        }

        self.commit_direct_mapping_source(
            MappingSourceKind::Midi,
            target,
            &event.port.name,
            &midi_learn_label(event),
        );
        true
    }

    fn commit_direct_mapping_source(
        &mut self,
        source_kind: MappingSourceKind,
        target: DirectMappingTarget,
        source_device_label: &str,
        source_label: &str,
    ) {
        let target_index = self.find_unique_direct_mapping_target_row(
            source_kind,
            target.target_label,
            target.scope_label,
        );
        let source_index =
            self.find_direct_mapping_source_row(source_kind, source_device_label, source_label);

        let index = if let Some(index) = source_index {
            if let Some(target_index) = target_index.filter(|target_index| *target_index != index) {
                if let Some(entry) = self.mappings.get_mut(target_index) {
                    entry.enabled = false;
                }
            }
            index
        } else if let Some(index) = target_index {
            index
        } else {
            let entry = MappingEntry {
                source_kind,
                source_device_label: source_device_label.to_string(),
                source_label: source_label.to_string(),
                target_label: target.target_label.to_string(),
                scope_label: target.scope_label.to_string(),
                enabled: true,
            };
            self.mappings.push(entry);
            self.mappings.len() - 1
        };

        let same_target = self.mappings.get(index).is_some_and(|entry| {
            entry.target_label == target.target_label && entry.scope_label == target.scope_label
        });
        if let Some(entry) = self.mappings.get_mut(index) {
            entry.source_kind = source_kind;
            entry.source_device_label = source_device_label.to_string();
            entry.source_label = source_label.to_string();
            entry.target_label = target.target_label.to_string();
            entry.scope_label = target.scope_label.to_string();
            entry.enabled = true;
        }
        self.page_state.selected_mapping_index = index;
        if self.direct_mapping_state.origin == DirectMappingOrigin::MappingsPage {
            self.page_state.current_page = AppPage::Mappings;
        }
        let message = if same_target {
            format!(
                "Updated {} ({}) to {}. Select another control to continue, or press Esc to finish.",
                target.target_label, target.scope_label, source_label
            )
        } else {
            format!(
                "Mapped {} ({}) to {}. Select another control to continue, or press Esc to finish.",
                target.target_label, target.scope_label, source_label
            )
        };
        self.direct_mapping_state.mode = DirectMappingMode::Targeting;
        self.direct_mapping_state.status_message = Some(message);
        self.sync_midi_inputs();
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

    fn handle_pointer_event(&mut self, event: &sdl3::event::Event) -> Option<AppControl> {
        if let Some((x, y)) = pointer_hover_position(event, self.viewport_size) {
            self.status_state.hovered_target =
                if self.direct_mapping_state.mode == DirectMappingMode::Inactive {
                    self.discoverability_target_at(x, y)
                } else {
                    None
                };
            if self.status_state.hovered_target.is_some() {
                self.direct_mapping_state.status_message = None;
            }
            return Some(AppControl::Continue);
        }

        let (x, y, source) = pointer_down_position(event, self.viewport_size)?;
        self.handle_pointer_down(x, y, source)
    }

    fn handle_keyboard_event(&mut self, event: &sdl3::event::Event) -> Option<AppControl> {
        if self.target_lookup_state.active.is_some() {
            match event {
                sdl3::event::Event::KeyDown {
                    keycode: Some(sdl3::keyboard::Keycode::Escape),
                    repeat: false,
                    ..
                } => {
                    return Some(self.apply_action_with_source(
                        AppAction::CancelCurrentMode,
                        crate::actions::ActionSource::Keyboard,
                    ));
                }
                sdl3::event::Event::KeyDown {
                    keycode: Some(sdl3::keyboard::Keycode::Backspace),
                    repeat: false,
                    ..
                } => {
                    self.backspace_mapping_target_lookup();
                    return Some(AppControl::Continue);
                }
                sdl3::event::Event::KeyDown {
                    keycode: Some(sdl3::keyboard::Keycode::Tab),
                    repeat: false,
                    ..
                } => {
                    return Some(AppControl::Continue);
                }
                _ => {
                    if let Some(input) = mapping_target_lookup_input(event) {
                        self.append_mapping_target_lookup_text(&input);
                        return Some(AppControl::Continue);
                    }
                }
            }
        }

        if matches!(
            event,
            sdl3::event::Event::KeyDown {
                keycode: Some(sdl3::keyboard::Keycode::Escape),
                repeat: false,
                ..
            }
        ) && self.direct_mapping_state.mode != DirectMappingMode::Inactive
        {
            return Some(self.apply_action_with_source(
                AppAction::CancelCurrentMode,
                crate::actions::ActionSource::Keyboard,
            ));
        }

        if let Some(source_label) = direct_mapping_key_label(event) {
            if self.direct_mapping_state.mode != DirectMappingMode::Inactive {
                if let DirectMappingMode::AwaitingInput(target) = self.direct_mapping_state.mode {
                    self.commit_direct_mapping_source(
                        MappingSourceKind::Key,
                        target,
                        &default_mapping_source_device(),
                        &source_label,
                    );
                }
                return Some(AppControl::Continue);
            }

            let mapping_actions = self.resolve_key_mapping_actions(&source_label);
            if !mapping_actions.is_empty() {
                for action in mapping_actions {
                    let control = self.apply_action_with_source(action, ActionSource::Keyboard);
                    if control == AppControl::Quit {
                        return Some(control);
                    }
                }
                return Some(AppControl::Continue);
            }
        }

        self.keyboard_bindings.resolve(event).map(|action_event| {
            self.apply_action_with_source(action_event.action, action_event.source)
        })
    }

    fn handle_pointer_down(
        &mut self,
        x: i32,
        y: i32,
        source: crate::actions::ActionSource,
    ) -> Option<AppControl> {
        let surface = crate::ui::surface_rect(self.viewport_size.0, self.viewport_size.1);
        let inset = crate::ui::inset_rect(surface, 24, 24).ok()?;
        let (tabs_bounds, content_bounds, _) = self.page_frame_layout(inset).ok()?;

        if let Some(control) =
            self.handle_direct_mapping_pointer_down(tabs_bounds, content_bounds, x, y, source)
        {
            return Some(control);
        }

        if let Some(page) = self.hit_page_tab(tabs_bounds, x, y) {
            return Some(self.apply_action_with_source(AppAction::ShowPage(page), source));
        }

        handle_page_pointer(
            self.page_state.current_page,
            self,
            content_bounds,
            x,
            y,
            source,
        )
    }

    fn handle_direct_mapping_pointer_down(
        &mut self,
        tabs_bounds: Rect,
        content_bounds: Rect,
        x: i32,
        y: i32,
        source: crate::actions::ActionSource,
    ) -> Option<AppControl> {
        if self.direct_mapping_state.mode == DirectMappingMode::Inactive {
            return None;
        }

        if self.page_state.current_page == AppPage::Mappings {
            let direct_badge = Rect::new(content_bounds.x + 532, content_bounds.y + 8, 154, 16);
            if rect_contains(direct_badge, x, y) {
                return Some(
                    self.apply_action_with_source(AppAction::ToggleDirectMappingMode, source),
                );
            }
        }

        if let Some(page) = self.hit_page_tab(tabs_bounds, x, y) {
            return Some(self.apply_action_with_source(AppAction::ShowPage(page), source));
        }

        if let Some(target) = self.direct_mapping_target_at(content_bounds, x, y) {
            self.direct_mapping_state.mode = DirectMappingMode::AwaitingInput(target);
            self.direct_mapping_state.status_message = None;
            self.sync_midi_inputs();
            return Some(AppControl::Continue);
        }

        Some(AppControl::Continue)
    }

    fn direct_mapping_target_at(
        &self,
        content_bounds: Rect,
        x: i32,
        y: i32,
    ) -> Option<DirectMappingTarget> {
        self.direct_mapping_targets(content_bounds)
            .into_iter()
            .find(|target| rect_contains(target.hit_rect, x, y))
    }

    fn direct_mapping_targets(&self, content_bounds: Rect) -> Vec<DirectMappingTarget> {
        let raw_targets =
            page_discoverability_targets(self.page_state.current_page, self, content_bounds);

        raw_targets
            .into_iter()
            .filter_map(|(rect, target)| {
                mapping_target_label_for_action(target.action).map(|target_label| {
                    DirectMappingTarget {
                        action: target.action,
                        target_label,
                        scope_label: target
                            .allowed_mapping_scopes
                            .first()
                            .copied()
                            .unwrap_or("Global"),
                        display_scope: target.display_scope,
                        hit_rect: rect,
                    }
                })
            })
            .collect()
    }

    fn direct_mapping_tab_targets(&self, _tabs_bounds: Rect) -> Vec<DirectMappingTarget> {
        Vec::new()
    }

    fn find_unique_direct_mapping_target_row(
        &self,
        source_kind: MappingSourceKind,
        target_label: &str,
        scope_label: &str,
    ) -> Option<usize> {
        let mut matches = self
            .mappings
            .iter()
            .enumerate()
            .filter(|(_, entry)| {
                entry.target_label == target_label
                    && entry.scope_label == scope_label
                    && entry.source_kind == source_kind
            })
            .map(|(index, _)| index);
        let first = matches.next()?;
        matches.next().is_none().then_some(first)
    }

    fn find_direct_mapping_source_row(
        &self,
        source_kind: MappingSourceKind,
        device_label: &str,
        source_label: &str,
    ) -> Option<usize> {
        self.mappings.iter().position(|entry| {
            entry.source_kind == source_kind
                && entry.source_device_label == device_label
                && entry.source_label == source_label
        })
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

    fn hit_page_tab(&self, bounds: Rect, x: i32, y: i32) -> Option<AppPage> {
        let (_, tabs_bounds) = page_tabs_layout(bounds);
        let tabs = crate::ui::equal_columns(tabs_bounds, AppPage::ALL.len(), 10);
        AppPage::ALL
            .iter()
            .copied()
            .zip(tabs)
            .find_map(|(page, rect)| rect_contains(rect, x, y).then_some(page))
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

    fn visible_track_columns(&self, timeline_bounds: Rect) -> Vec<(usize, Rect, Rect)> {
        if self.project.tracks.is_empty() {
            return Vec::new();
        }

        if self.focused_track_view {
            return crate::ui::track_column_pairs(timeline_bounds, 1)
                .into_iter()
                .next()
                .map(|(full_bounds, detail_bounds)| {
                    vec![(self.project.active_track_index, full_bounds, detail_bounds)]
                })
                .unwrap_or_default();
        }

        crate::ui::track_column_pairs(timeline_bounds, self.project.tracks.len())
            .into_iter()
            .enumerate()
            .map(|(index, (full_bounds, detail_bounds))| (index, full_bounds, detail_bounds))
            .collect()
    }

    fn timeline_track_layout(
        &self,
        track_index: usize,
        full_bounds: Rect,
        detail_bounds: Rect,
    ) -> TimelineTrackLayout {
        let pair_bounds = crate::ui::union_rect(full_bounds, detail_bounds);
        let status_rect = crate::ui::track_status_rect(pair_bounds, self.timeline_flow);
        let (body_full_bounds, body_detail_bounds) =
            self.track_column_body_bounds(full_bounds, detail_bounds);
        let full_label_rect = timeline_subcolumn_label_rect(body_full_bounds, self.timeline_flow);
        let detail_label_rect =
            timeline_subcolumn_label_rect(body_detail_bounds, self.timeline_flow);
        let full_content_rect =
            timeline_subcolumn_content_rect(body_full_bounds, self.timeline_flow);
        let detail_content_rect =
            timeline_subcolumn_content_rect(body_detail_bounds, self.timeline_flow);
        let (input_fx_rect, output_fx_rect) = self.track_fx_band_rects(
            full_bounds,
            detail_bounds,
            &self.project.tracks[track_index],
        );
        TimelineTrackLayout {
            track_index,
            full_bounds,
            detail_bounds,
            pair_bounds,
            status_rect,
            body_full_bounds,
            body_detail_bounds,
            full_label_rect,
            detail_label_rect,
            full_content_rect,
            detail_content_rect,
            input_fx_rect,
            output_fx_rect,
        }
    }

    fn active_track_full_bounds(&self) -> Option<Rect> {
        let surface = crate::ui::surface_rect(self.viewport_size.0, self.viewport_size.1);
        let inset = crate::ui::inset_rect(surface, 24, 24).ok()?;
        let (_, content_bounds) = crate::ui::split_top_strip(inset, 28, 12).ok()?;
        let (_, body_bounds) = crate::ui::split_top_strip(content_bounds, 28, 6).ok()?;
        let (_, timeline_bounds) =
            crate::ui::split_top_strip(body_bounds, transport_strip_height(), 8).ok()?;
        self.visible_track_columns(timeline_bounds)
            .into_iter()
            .find(|(index, _, _)| *index == self.project.active_track_index)
            .map(|(_, full_bounds, _)| full_bounds)
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

fn rect_contains(rect: Rect, x: i32, y: i32) -> bool {
    x >= rect.x
        && x < rect.x + rect.width() as i32
        && y >= rect.y
        && y < rect.y + rect.height() as i32
}

fn pointer_down_position(
    event: &sdl3::event::Event,
    viewport_size: (u32, u32),
) -> Option<(i32, i32, crate::actions::ActionSource)> {
    match event {
        sdl3::event::Event::MouseButtonDown { x, y, .. } => {
            Some((*x as i32, *y as i32, crate::actions::ActionSource::Pointer))
        }
        sdl3::event::Event::FingerDown { x, y, .. } => Some((
            (*x * viewport_size.0 as f32) as i32,
            (*y * viewport_size.1 as f32) as i32,
            crate::actions::ActionSource::Touch,
        )),
        _ => None,
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

fn pointer_hover_position(
    event: &sdl3::event::Event,
    viewport_size: (u32, u32),
) -> Option<(i32, i32)> {
    match event {
        sdl3::event::Event::MouseMotion { x, y, .. } => Some((*x as i32, *y as i32)),
        sdl3::event::Event::FingerMotion { x, y, .. } => Some((
            (*x * viewport_size.0 as f32) as i32,
            (*y * viewport_size.1 as f32) as i32,
        )),
        _ => None,
    }
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

fn timeline_fx_enabled_chip_label(slot: &MidiFxSlot, show_kind_title: bool) -> &'static str {
    if show_kind_title {
        ""
    } else {
        slot.effect.kind().compact_label()
    }
}

fn timeline_fx_kind_display(slot: &MidiFxSlot, width: u32) -> &'static str {
    if width >= 20 {
        slot.effect.kind().short_label()
    } else {
        slot.effect.kind().compact_label()
    }
}

fn timeline_fx_kind_target_width(slot: &MidiFxSlot, available: u32) -> u32 {
    let label = if available >= 72 {
        slot.effect.kind().short_label()
    } else {
        slot.effect.kind().compact_label()
    };
    let glyph_width = 5_u32;
    let padding = 8_u32;
    (label.len() as u32 * glyph_width + padding).clamp(20, 28)
}

fn timeline_param_compact_label(label: &str) -> &str {
    match label {
        "Rate" => "Rt",
        "Gate" => "Gt",
        "Low" => "Lo",
        "High" => "Hi",
        "List" => "Ls",
        "Semi" => "Sm",
        "Vel" => "Vl",
        "Len" => "Ln",
        "Root" => "Rt",
        "Tgt" => "Tg",
        "Dly" => "Dl",
        "Src" => "Sc",
        other => other,
    }
}

fn timeline_fx_overflow_label(param_count: usize, window_start: usize) -> String {
    if param_count <= 2 {
        "--".to_string()
    } else {
        let window_count = param_count.saturating_sub(1).max(1);
        format!("{}/{}", window_start + 1, window_count)
    }
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
        routing_field_short_label, ticks_per_second_for_tempo, timeline_fx_overflow_label,
        transport_strip_height, App, AppControl, AppOverlay, DirectMappingMode,
        DirectMappingOrigin, DirectMappingTarget, DiscoverabilityTarget, LastActionStatus,
    };
    use crate::actions::{ActionSource, AppAction};
    use crate::mapping::{default_mapping_source_device, MappingEntry, MappingSourceKind};
    use crate::midi_fx::{MidiFx, MidiFxChainKind, MidiFxSlot, MIDI_FX_SLOT_COUNT};
    use crate::midi_io::{MidiInputEvent, MidiInputMessage, MidiPortRef};
    use crate::pages::{AppPage, MappingField, MappingPageMode, MidiIoListFocus, RoutingField};
    use crate::project::{
        MidiNote, RecordContext, RecordingView, Track, TrackKind, STORED_LOOP_SLOT_COUNT,
    };
    use crate::routing::MidiChannelFilter;
    use crate::timeline::RecordingTake;
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
    fn recording_clip_actions_update_active_track_clip_state() {
        let mut app = App::new();
        let transport = app.project.transport;
        {
            let track = app.project.active_track_mut().unwrap();
            track.clear_content();
            track.commit_take(transport, RecordingTake::new(0).release(480), None);
            track.commit_take(transport, RecordingTake::new(960).release(1_440), None);
        }

        app.apply_action(AppAction::ToggleCurrentTrackRecordingView);
        assert_eq!(
            app.project.active_track().unwrap().recording_view,
            RecordingView::Stacked
        );

        app.apply_action(AppAction::SelectPreviousRecordingClip);
        let selected_before_delete = app
            .project
            .active_track()
            .unwrap()
            .selected_recording_clip_id
            .expect("selected clip");
        app.apply_action(AppAction::ToggleSelectedRecordingClipMute);
        assert!(
            app.project
                .active_track()
                .unwrap()
                .selected_recording_clip()
                .unwrap()
                .muted
        );

        app.apply_action(AppAction::DeleteSelectedRecordingClip);
        let active = app.project.active_track().unwrap();
        assert_eq!(active.recording_clips.len(), 1);
        assert_ne!(active.recording_clips[0].id, selected_before_delete);
    }

    #[test]
    fn focused_track_view_limits_timeline_to_active_track() {
        let mut app = App::new();
        let timeline_bounds = Rect::new(0, 0, 1000, 420);

        assert_eq!(
            app.visible_track_columns(timeline_bounds).len(),
            app.project.tracks.len()
        );

        app.apply_action(AppAction::SelectTrack(2));
        app.apply_action(AppAction::ToggleFocusedTrackView);

        let visible = app.visible_track_columns(timeline_bounds);
        assert_eq!(visible.len(), 1);
        assert_eq!(visible[0].0, 2);
    }

    #[test]
    fn stacked_all_track_layout_shows_at_least_three_recording_lanes() {
        let app = App::new();
        let timeline_bounds = Rect::new(0, 0, 1000, 420);
        let (_, full_bounds, _) = app.visible_track_columns(timeline_bounds)[0];
        let content_rect = crate::ui::track_content_rect(full_bounds, app.timeline_flow);

        assert!(app.recording_lane_capacity(content_rect) >= 3);
    }

    #[test]
    fn stacked_view_shows_preview_lane_while_recording() {
        let mut app = App::new();
        let transport = app.project.transport;
        {
            let track = app.project.active_track_mut().unwrap();
            track.clear_content();
            track.recording_view = RecordingView::Stacked;
            track.commit_take(transport, RecordingTake::new(0).release(480), None);
        }

        app.transport_ticks = 960;
        app.playhead_ticks = 960;
        app.apply_action(AppAction::ToggleRecording);

        let timeline_bounds = Rect::new(0, 0, 1000, 420);
        let (_, full_bounds, _) = app.visible_track_columns(timeline_bounds)[0];
        let content_rect = crate::ui::track_content_rect(full_bounds, app.timeline_flow);
        let layouts = app.recording_lane_layouts(content_rect, app.project.active_track().unwrap());

        assert_eq!(layouts.len(), 2);
        assert!(layouts.iter().any(|lane| lane.preview));
    }

    #[test]
    fn stacked_view_preview_lane_shifts_visible_window_as_committed() {
        let mut app = App::new();
        let transport = app.project.transport;
        let trailing_clip_ids = {
            let track = app.project.active_track_mut().unwrap();
            track.clear_content();
            track.recording_view = RecordingView::Stacked;
            for index in 0..5 {
                let start = index * 480;
                track.commit_take(
                    transport,
                    RecordingTake::new(start).release(start + 240),
                    None,
                );
            }
            let trailing = vec![track.recording_clips[3].id, track.recording_clips[4].id];
            track.recording_clip_scroll = 2;
            track.active_take = Some(RecordingTake::new(2400));
            trailing
        };

        let content_rect = Rect::new(0, 0, 49, 200);
        assert_eq!(app.recording_lane_capacity(content_rect), 3);
        let layouts = app.recording_lane_layouts(content_rect, app.project.active_track().unwrap());

        assert_eq!(layouts.len(), 3);
        assert_eq!(layouts[0].clip_id, Some(trailing_clip_ids[0]));
        assert_eq!(layouts[1].clip_id, Some(trailing_clip_ids[1]));
        assert!(layouts[2].preview);
    }

    #[test]
    fn stacked_scrollbar_thumb_tracks_clip_window_position() {
        let mut app = App::new();
        let transport = app.project.transport;
        {
            let track = app.project.active_track_mut().unwrap();
            track.clear_content();
            track.recording_view = RecordingView::Stacked;
            for index in 0..5 {
                let start = index * 480;
                track.commit_take(
                    transport,
                    RecordingTake::new(start).release(start + 240),
                    None,
                );
            }
            track.recording_clip_scroll = 0;
        }

        let timeline_bounds = Rect::new(0, 0, 1000, 420);
        let (_, full_bounds, _) = app.visible_track_columns(timeline_bounds)[0];
        let content_rect = crate::ui::track_content_rect(full_bounds, app.timeline_flow);
        let (_, thumb_before) = app
            .recording_clip_scrollbar_rects(content_rect, app.project.active_track().unwrap())
            .expect("scrollbar");

        app.project
            .active_track_mut()
            .unwrap()
            .recording_clip_scroll = 2;
        let (_, thumb_after) = app
            .recording_clip_scrollbar_rects(content_rect, app.project.active_track().unwrap())
            .expect("scrollbar");

        assert!(thumb_after.x > thumb_before.x);
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
    fn transport_chip_specs_include_visible_loop_recording_wrap_status() {
        let mut app = App::new();
        let labels = app
            .transport_bottom_chip_specs()
            .into_iter()
            .map(|chip| chip.label)
            .collect::<Vec<_>>();
        assert!(labels.iter().any(|label| label == "Wrap Extend"));
        assert!(labels.iter().any(|label| label == "Harmony C"));

        app.apply_action(AppAction::ToggleLoopRecordingExtension);
        let labels = app
            .transport_bottom_chip_specs()
            .into_iter()
            .map(|chip| chip.label)
            .collect::<Vec<_>>();
        assert!(labels.iter().any(|label| label == "Wrap Clamp"));
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
    fn mappings_target_lookup_uses_canonical_page_actions_while_open() {
        let mut app = App::new();
        app.apply_action(AppAction::ShowPage(AppPage::Mappings));
        app.apply_action(AppAction::ToggleMappingsWriteMode);
        app.page_state.selected_mapping_field = MappingField::Target;
        app.apply_action(AppAction::ActivatePageItem);

        assert!(app.target_lookup_state.active.is_some());
        assert_eq!(
            app.target_lookup_state
                .active
                .as_ref()
                .map(|lookup| lookup.highlighted_index),
            Some(0)
        );

        app.apply_action(AppAction::SelectNextPageItem);
        assert_eq!(
            app.target_lookup_state
                .active
                .as_ref()
                .map(|lookup| lookup.highlighted_index),
            Some(1)
        );

        app.apply_action(AppAction::AdjustPageItemForward);
        assert_eq!(
            app.target_lookup_state
                .active
                .as_ref()
                .map(|lookup| lookup.highlighted_index),
            Some(2)
        );

        let expected = app.mapping_target_lookup_highlighted_label();
        app.apply_action(AppAction::ActivatePageItem);

        assert_eq!(app.mappings[0].target_label.as_str(), expected.unwrap());
        assert!(app.target_lookup_state.active.is_none());
    }

    #[test]
    fn mappings_target_lookup_next_and_previous_clamp_and_scroll_instead_of_wrapping() {
        let mut app = App::new();
        app.apply_action(AppAction::ShowPage(AppPage::Mappings));
        app.apply_action(AppAction::ToggleMappingsWriteMode);
        app.page_state.selected_mapping_field = MappingField::Target;
        app.apply_action(AppAction::ActivatePageItem);

        let result_len = app.mapping_target_lookup_results().len();
        assert!(result_len > 6);

        for _ in 0..(result_len + 3) {
            app.apply_action(AppAction::SelectNextPageItem);
        }
        assert_eq!(
            app.target_lookup_state
                .active
                .as_ref()
                .map(|lookup| lookup.highlighted_index),
            Some(result_len - 1)
        );

        let content_bounds = Rect::new(0, 0, 960, 540);
        let layout = app
            .mapping_target_lookup_layout(content_bounds)
            .expect("lookup layout");
        assert_eq!(layout.visible_count, 6);
        assert_eq!(layout.start_index, result_len - layout.visible_count);

        app.apply_action(AppAction::SelectNextPageItem);
        assert_eq!(
            app.target_lookup_state
                .active
                .as_ref()
                .map(|lookup| lookup.highlighted_index),
            Some(result_len - 1)
        );

        for _ in 0..(result_len + 3) {
            app.apply_action(AppAction::SelectPreviousPageItem);
        }
        assert_eq!(
            app.target_lookup_state
                .active
                .as_ref()
                .map(|lookup| lookup.highlighted_index),
            Some(0)
        );
        let layout = app
            .mapping_target_lookup_layout(content_bounds)
            .expect("lookup layout");
        assert_eq!(layout.start_index, 0);
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
    fn mappings_overlay_toggles_on_and_off() {
        let mut app = App::new();
        assert!(app.overlay_state.active.is_none());

        app.apply_action(AppAction::ToggleMappingsOverlay);
        assert_eq!(
            app.overlay_state.active,
            Some(AppOverlay::MappingsQuickView)
        );

        app.apply_action(AppAction::ToggleMappingsOverlay);
        assert!(app.overlay_state.active.is_none());
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
    fn discoverability_summary_hides_disabled_and_absolute_track_mappings() {
        let mut app = App::new();
        app.mappings = vec![
            MappingEntry {
                source_kind: MappingSourceKind::Midi,
                source_device_label: "Any MIDI".to_string(),
                source_label: "CC20".to_string(),
                target_label: "Track Arm".to_string(),
                scope_label: "Active Track".to_string(),
                enabled: true,
            },
            MappingEntry {
                source_kind: MappingSourceKind::Midi,
                source_device_label: "Any MIDI".to_string(),
                source_label: "CC21".to_string(),
                target_label: "Track Arm".to_string(),
                scope_label: "Track 3".to_string(),
                enabled: true,
            },
            MappingEntry {
                source_kind: MappingSourceKind::Osc,
                source_device_label: default_mapping_source_device(),
                source_label: "/track/active/arm".to_string(),
                target_label: "Track Arm".to_string(),
                scope_label: "Active Track".to_string(),
                enabled: false,
            },
        ];

        let summary = app.summarize_discoverability_target(DiscoverabilityTarget {
            action: AppAction::ToggleCurrentTrackArm,
            display_scope: Some("Active Track"),
            allowed_mapping_scopes: &["Active Track"],
            overlay_slot: None,
        });

        assert!(summary.badges.iter().any(|badge| badge.text == "A"));
        assert!(summary.badges.iter().any(|badge| badge.text == "CC20"));
        assert!(!summary.badges.iter().any(|badge| badge.text == "CC21"));
        assert!(!summary
            .badges
            .iter()
            .any(|badge| badge.text == "/track/active/arm"));
    }

    #[test]
    fn summarize_discoverability_target_includes_note_edit_shortcuts() {
        let app = App::new();

        let summary = app.summarize_discoverability_target(DiscoverabilityTarget {
            action: AppAction::SelectNotesAtPlayhead,
            display_scope: Some("Active Track"),
            allowed_mapping_scopes: &["Active Track"],
            overlay_slot: None,
        });

        assert!(summary.badges.iter().any(|badge| badge.text == "T"));
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
    fn direct_mapping_shortcut_toggles_targeting_mode() {
        let mut app = App::new();

        app.apply_action(AppAction::ToggleDirectMappingMode);
        assert_eq!(app.direct_mapping_state.mode, DirectMappingMode::Targeting);

        app.apply_action(AppAction::ToggleDirectMappingMode);
        assert_eq!(app.direct_mapping_state.mode, DirectMappingMode::Inactive);
    }

    #[test]
    fn direct_mapping_input_creates_new_mapping_for_target() {
        let mut app = App::new();
        app.mappings.clear();
        app.direct_mapping_state.mode = DirectMappingMode::AwaitingInput(DirectMappingTarget {
            action: AppAction::TogglePlayback,
            target_label: "Play/Stop",
            scope_label: "Global",
            display_scope: Some("Global"),
            hit_rect: Rect::new(0, 0, 10, 10),
        });

        app.handle_midi_input_event(MidiInputEvent {
            port: MidiPortRef::new("In A"),
            channel: 1,
            message: MidiInputMessage::ControlChange {
                controller: 24,
                value: 127,
            },
        });

        assert_eq!(app.mappings.len(), 1);
        assert_eq!(app.mappings[0].target_label, "Play/Stop");
        assert_eq!(app.mappings[0].scope_label, "Global");
        assert_eq!(app.mappings[0].source_device_label, "In A");
        assert_eq!(app.mappings[0].source_label, "CC24 Ch1");
        assert!(app.mappings[0].enabled);
        assert_eq!(app.page_state.current_page, AppPage::Timeline);
        assert_eq!(app.direct_mapping_state.mode, DirectMappingMode::Targeting);
    }

    #[test]
    fn direct_mapping_from_mappings_page_returns_to_mappings() {
        let mut app = App::new();
        app.mappings.clear();
        app.page_state.current_page = AppPage::Mappings;
        app.direct_mapping_state.origin = DirectMappingOrigin::MappingsPage;
        app.direct_mapping_state.mode = DirectMappingMode::AwaitingInput(DirectMappingTarget {
            action: AppAction::TogglePlayback,
            target_label: "Play/Stop",
            scope_label: "Global",
            display_scope: Some("Global"),
            hit_rect: Rect::new(0, 0, 10, 10),
        });

        app.handle_midi_input_event(MidiInputEvent {
            port: MidiPortRef::new("In A"),
            channel: 1,
            message: MidiInputMessage::ControlChange {
                controller: 24,
                value: 127,
            },
        });

        assert_eq!(app.page_state.current_page, AppPage::Mappings);
        assert_eq!(app.direct_mapping_state.mode, DirectMappingMode::Targeting);
    }

    #[test]
    fn direct_mapping_reuses_unique_target_row() {
        let mut app = App::new();
        app.mappings = vec![MappingEntry {
            source_kind: MappingSourceKind::Midi,
            source_device_label: "Old Port".to_string(),
            source_label: "CC20 Ch1".to_string(),
            target_label: "Track Arm".to_string(),
            scope_label: "Active Track".to_string(),
            enabled: true,
        }];
        app.direct_mapping_state.mode = DirectMappingMode::AwaitingInput(DirectMappingTarget {
            action: AppAction::ToggleCurrentTrackArm,
            target_label: "Track Arm",
            scope_label: "Active Track",
            display_scope: Some("Active Track"),
            hit_rect: Rect::new(0, 0, 10, 10),
        });

        app.handle_midi_input_event(MidiInputEvent {
            port: MidiPortRef::new("New Port"),
            channel: 2,
            message: MidiInputMessage::ControlChange {
                controller: 21,
                value: 127,
            },
        });

        assert_eq!(app.mappings.len(), 1);
        assert_eq!(app.mappings[0].source_device_label, "New Port");
        assert_eq!(app.mappings[0].source_label, "CC21 Ch2");
        assert_eq!(app.mappings[0].target_label, "Track Arm");
        assert_eq!(app.mappings[0].scope_label, "Active Track");
        assert!(app.mappings[0].enabled);
    }

    #[test]
    fn direct_mapping_moves_existing_source_and_disables_old_target_row() {
        let mut app = App::new();
        app.mappings = vec![
            MappingEntry {
                source_kind: MappingSourceKind::Midi,
                source_device_label: "Port A".to_string(),
                source_label: "CC20 Ch1".to_string(),
                target_label: "Play/Stop".to_string(),
                scope_label: "Global".to_string(),
                enabled: true,
            },
            MappingEntry {
                source_kind: MappingSourceKind::Midi,
                source_device_label: "Port B".to_string(),
                source_label: "CC21 Ch1".to_string(),
                target_label: "Track Arm".to_string(),
                scope_label: "Active Track".to_string(),
                enabled: true,
            },
        ];
        app.direct_mapping_state.mode = DirectMappingMode::AwaitingInput(DirectMappingTarget {
            action: AppAction::ToggleCurrentTrackArm,
            target_label: "Track Arm",
            scope_label: "Active Track",
            display_scope: Some("Active Track"),
            hit_rect: Rect::new(0, 0, 10, 10),
        });

        app.handle_midi_input_event(MidiInputEvent {
            port: MidiPortRef::new("Port A"),
            channel: 1,
            message: MidiInputMessage::ControlChange {
                controller: 20,
                value: 127,
            },
        });

        assert_eq!(app.mappings.len(), 2);
        assert_eq!(app.mappings[0].target_label, "Track Arm");
        assert_eq!(app.mappings[0].scope_label, "Active Track");
        assert!(app.mappings[0].enabled);
        assert!(!app.mappings[1].enabled);
    }

    #[test]
    fn direct_mapping_cancel_message_yields_to_hover_summary() {
        let mut app = App::new();
        app.cancel_direct_mapping("Canceled direct mapping.");
        app.status_state.hovered_target = Some(DiscoverabilityTarget {
            action: AppAction::TogglePlayback,
            display_scope: Some("Global"),
            allowed_mapping_scopes: &["Global"],
            overlay_slot: None,
        });

        assert!(app.direct_mapping_footer_content().is_none());
    }

    #[test]
    fn direct_mapping_keyboard_capture_supports_modifiers() {
        let mut app = App::new();
        app.mappings.clear();
        app.direct_mapping_state.mode = DirectMappingMode::AwaitingInput(DirectMappingTarget {
            action: AppAction::TogglePlayback,
            target_label: "Play/Stop",
            scope_label: "Global",
            display_scope: Some("Global"),
            hit_rect: Rect::new(0, 0, 10, 10),
        });

        let control = app.handle_keyboard_event(&sdl3::event::Event::KeyDown {
            timestamp: 0,
            window_id: 0,
            keycode: Some(sdl3::keyboard::Keycode::R),
            scancode: None,
            keymod: sdl3::keyboard::Mod::LCTRLMOD | sdl3::keyboard::Mod::LSHIFTMOD,
            repeat: false,
            which: 0,
            raw: 0,
        });

        assert_eq!(control, Some(AppControl::Continue));
        assert_eq!(app.mappings.len(), 1);
        assert_eq!(app.mappings[0].source_kind, MappingSourceKind::Key);
        assert_eq!(app.mappings[0].source_label, "Ctrl+Shift+R");
        assert_eq!(app.mappings[0].target_label, "Play/Stop");
        assert_eq!(app.direct_mapping_state.mode, DirectMappingMode::Targeting);
    }

    #[test]
    fn direct_mapping_keyboard_path_reserves_escape_and_f8_for_cancel() {
        let mut app = App::new();
        app.direct_mapping_state.mode = DirectMappingMode::AwaitingInput(DirectMappingTarget {
            action: AppAction::TogglePlayback,
            target_label: "Play/Stop",
            scope_label: "Global",
            display_scope: Some("Global"),
            hit_rect: Rect::new(0, 0, 10, 10),
        });

        let escape = app.handle_keyboard_event(&sdl3::event::Event::KeyDown {
            timestamp: 0,
            window_id: 0,
            keycode: Some(sdl3::keyboard::Keycode::Escape),
            scancode: None,
            keymod: sdl3::keyboard::Mod::NOMOD,
            repeat: false,
            which: 0,
            raw: 0,
        });
        assert_eq!(escape, Some(AppControl::Continue));
        assert!(
            app.mappings.is_empty()
                || app
                    .mappings
                    .iter()
                    .all(|entry| entry.target_label != "Play/Stop"
                        || entry.source_kind != MappingSourceKind::Key
                        || entry.source_label != "Escape")
        );
        assert_eq!(app.direct_mapping_state.mode, DirectMappingMode::Inactive);

        app.direct_mapping_state.mode = DirectMappingMode::AwaitingInput(DirectMappingTarget {
            action: AppAction::TogglePlayback,
            target_label: "Play/Stop",
            scope_label: "Global",
            display_scope: Some("Global"),
            hit_rect: Rect::new(0, 0, 10, 10),
        });
        let f8 = app.handle_keyboard_event(&sdl3::event::Event::KeyDown {
            timestamp: 0,
            window_id: 0,
            keycode: Some(sdl3::keyboard::Keycode::F8),
            scancode: None,
            keymod: sdl3::keyboard::Mod::NOMOD,
            repeat: false,
            which: 0,
            raw: 0,
        });
        assert_eq!(f8, Some(AppControl::Continue));
        assert_eq!(app.direct_mapping_state.mode, DirectMappingMode::Inactive);
    }

    #[test]
    fn direct_mapping_pointer_can_retarget_while_awaiting_input() {
        let mut app = App::new();
        let surface = crate::ui::surface_rect(app.viewport_size.0, app.viewport_size.1);
        let inset = crate::ui::inset_rect(surface, 24, 24).expect("surface inset");
        let (tabs_bounds, page_area_bounds) =
            crate::ui::split_top_strip(inset, 28, 12).expect("page split");
        let content_bounds = Rect::new(
            page_area_bounds.x(),
            page_area_bounds.y(),
            page_area_bounds.width(),
            page_area_bounds.height().saturating_sub(30),
        );
        app.direct_mapping_state.mode = DirectMappingMode::AwaitingInput(DirectMappingTarget {
            action: AppAction::TogglePlayback,
            target_label: "Play/Stop",
            scope_label: "Global",
            display_scope: Some("Global"),
            hit_rect: Rect::new(0, 0, 10, 10),
        });

        let record_target = app
            .direct_mapping_targets(content_bounds)
            .into_iter()
            .find(|target| target.target_label == "Record" && target.scope_label == "Armed/Active")
            .expect("record target");
        let point_x = record_target.hit_rect.x() + (record_target.hit_rect.width() / 2) as i32;
        let point_y = record_target.hit_rect.y() + (record_target.hit_rect.height() / 2) as i32;

        let control = app.handle_direct_mapping_pointer_down(
            tabs_bounds,
            content_bounds,
            point_x,
            point_y,
            ActionSource::Pointer,
        );

        assert_eq!(control, Some(AppControl::Continue));
        assert_eq!(
            app.direct_mapping_state.mode,
            DirectMappingMode::AwaitingInput(record_target)
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
    fn midi_io_page_can_switch_focus_and_commit_default_ports() {
        let mut app = App::new();
        app.midi_devices.inputs = vec![MidiPortRef::new("In A"), MidiPortRef::new("In B")];
        app.midi_devices.outputs = vec![MidiPortRef::new("Out A"), MidiPortRef::new("Out B")];
        app.apply_action(AppAction::ShowPage(AppPage::MidiIo));
        app.apply_action(AppAction::SelectNextPageItem);
        app.apply_action(AppAction::ActivatePageItem);
        assert_eq!(
            app.midi_devices.selected_input,
            Some(app.page_state.midi_io.selected_input_index)
        );

        app.apply_action(AppAction::AdjustPageItemForward);
        assert_eq!(app.page_state.midi_io.focus, MidiIoListFocus::Outputs);
    }

    #[test]
    fn routing_page_adjusts_active_track_routing() {
        let mut app = App::new();
        app.apply_action(AppAction::ShowPage(AppPage::Routing));
        app.page_state.selected_routing_field = RoutingField::OutputChannel;

        let before = app.project.active_track().unwrap().routing.output_channel;
        app.apply_action(AppAction::AdjustPageItemForward);

        assert_ne!(
            app.project.active_track().unwrap().routing.output_channel,
            before
        );
    }

    #[test]
    fn routing_fx_panels_use_two_column_grid_for_six_fields() {
        let app = App::new();
        let body = Rect::new(0, 0, 900, 520);
        let (_, input_panel, _, output_panel) = app.routing_panel_rects(body);
        let rects = app.routing_field_rects(body);

        let input_slot = rects
            .iter()
            .find(|(field, _)| *field == RoutingField::InputFxSlot)
            .map(|(_, rect)| *rect)
            .unwrap();
        let input_kind = rects
            .iter()
            .find(|(field, _)| *field == RoutingField::InputFxKind)
            .map(|(_, rect)| *rect)
            .unwrap();
        let input_on = rects
            .iter()
            .find(|(field, _)| *field == RoutingField::InputFxEnabled)
            .map(|(_, rect)| *rect)
            .unwrap();
        let input_p1 = rects
            .iter()
            .find(|(field, _)| *field == RoutingField::InputFxParam1)
            .map(|(_, rect)| *rect)
            .unwrap();
        let input_p2 = rects
            .iter()
            .find(|(field, _)| *field == RoutingField::InputFxParam2)
            .map(|(_, rect)| *rect)
            .unwrap();
        let input_more = rects
            .iter()
            .find(|(field, _)| *field == RoutingField::InputFxMore)
            .map(|(_, rect)| *rect)
            .unwrap();

        assert_eq!(input_slot.y, input_kind.y);
        assert_eq!(input_on.y, input_p1.y);
        assert_eq!(input_p2.y, input_more.y);
        assert!(input_slot.x < input_kind.x);
        assert!(input_on.x < input_p1.x);
        assert!(input_p2.x < input_more.x);
        for rect in [
            input_slot, input_kind, input_on, input_p1, input_p2, input_more,
        ] {
            assert!(input_panel.contains_point((rect.x, rect.y)));
            assert!(input_panel.contains_point((
                rect.x + rect.width() as i32 - 1,
                rect.y + rect.height() as i32 - 1
            )));
        }

        let output_slot = rects
            .iter()
            .find(|(field, _)| *field == RoutingField::OutputFxSlot)
            .map(|(_, rect)| *rect)
            .unwrap();
        let output_kind = rects
            .iter()
            .find(|(field, _)| *field == RoutingField::OutputFxKind)
            .map(|(_, rect)| *rect)
            .unwrap();
        let output_on = rects
            .iter()
            .find(|(field, _)| *field == RoutingField::OutputFxEnabled)
            .map(|(_, rect)| *rect)
            .unwrap();
        let output_p1 = rects
            .iter()
            .find(|(field, _)| *field == RoutingField::OutputFxParam1)
            .map(|(_, rect)| *rect)
            .unwrap();
        let output_p2 = rects
            .iter()
            .find(|(field, _)| *field == RoutingField::OutputFxParam2)
            .map(|(_, rect)| *rect)
            .unwrap();
        let output_more = rects
            .iter()
            .find(|(field, _)| *field == RoutingField::OutputFxMore)
            .map(|(_, rect)| *rect)
            .unwrap();

        assert_eq!(output_slot.y, output_kind.y);
        assert_eq!(output_on.y, output_p1.y);
        assert_eq!(output_p2.y, output_more.y);
        assert!(output_slot.x < output_kind.x);
        assert!(output_on.x < output_p1.x);
        assert!(output_p2.x < output_more.x);
        for rect in [
            output_slot,
            output_kind,
            output_on,
            output_p1,
            output_p2,
            output_more,
        ] {
            assert!(output_panel.contains_point((rect.x, rect.y)));
            assert!(output_panel.contains_point((
                rect.x + rect.width() as i32 - 1,
                rect.y + rect.height() as i32 - 1
            )));
        }
    }

    #[test]
    fn routing_field_short_labels_match_compact_fx_grid() {
        assert_eq!(routing_field_short_label(RoutingField::InputFxSlot), "Slot");
        assert_eq!(routing_field_short_label(RoutingField::InputFxKind), "Kind");
        assert_eq!(
            routing_field_short_label(RoutingField::InputFxEnabled),
            "On"
        );
        assert_eq!(routing_field_short_label(RoutingField::InputFxParam1), "P1");
        assert_eq!(routing_field_short_label(RoutingField::InputFxParam2), "P2");
        assert_eq!(routing_field_short_label(RoutingField::InputFxMore), "More");
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
    fn timeline_body_label_controls_do_not_overlap_input_fx_band() {
        let app = App::new();
        let content_bounds = Rect::new(40, 40, 1200, 620);
        let (_, body_bounds) =
            crate::ui::split_top_strip(content_bounds, 28, 6).expect("timeline content");
        let (_, timeline_bounds) =
            crate::ui::split_top_strip(body_bounds, transport_strip_height(), 8)
                .expect("timeline body");
        let columns = crate::ui::track_column_pairs(timeline_bounds, app.project.tracks.len());
        let (full_bounds, detail_bounds) = columns[0];
        let (body_full_bounds, body_detail_bounds) =
            app.track_column_body_bounds(full_bounds, detail_bounds);
        let full_label_rect =
            super::timeline_subcolumn_label_rect(body_full_bounds, app.timeline_flow);
        let detail_label_rect =
            super::timeline_subcolumn_label_rect(body_detail_bounds, app.timeline_flow);
        let (input_fx_rect, _) =
            app.track_fx_band_rects(full_bounds, detail_bounds, &app.project.tracks[0]);
        let view_rect = app.recording_view_chip_rect(full_label_rect);
        let thru_rect = app.track_passthrough_button_rect(full_label_rect);
        let detail_badge = crate::ui::detail_badge_rect(detail_label_rect);
        let stored_slot = app.stored_loop_slot_rects(detail_label_rect)[0].1;
        let intersects = |a: Rect, b: Rect| {
            a.x < b.x + b.width() as i32
                && a.x + a.width() as i32 > b.x
                && a.y < b.y + b.height() as i32
                && a.y + a.height() as i32 > b.y
        };

        assert!(!intersects(input_fx_rect, view_rect));
        assert!(!intersects(input_fx_rect, thru_rect));
        assert!(!intersects(input_fx_rect, detail_badge));
        assert!(!intersects(input_fx_rect, stored_slot));
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
    fn timeline_track_fx_row_click_selects_output_fx_context() {
        let mut app = App::new();
        let content_bounds = Rect::new(40, 40, 1200, 620);
        let (_, body_bounds) =
            crate::ui::split_top_strip(content_bounds, 28, 6).expect("timeline content");
        let (_, timeline_bounds) =
            crate::ui::split_top_strip(body_bounds, transport_strip_height(), 8)
                .expect("timeline body");
        let columns = crate::ui::track_column_pairs(timeline_bounds, app.project.tracks.len());
        let (full_bounds, detail_bounds) = columns[0];
        let (_, output_band) =
            app.track_fx_band_rects(full_bounds, detail_bounds, &app.project.tracks[0]);
        let displayed = app.displayed_timeline_fx_slot_indices(MidiFxChainKind::Output);
        let row = app.timeline_fx_row_layouts(
            output_band,
            &displayed,
            &app.project.tracks[0].midi_fx.output_fx,
            None,
        )[0]
        .row;

        let control = app.handle_timeline_pointer(
            content_bounds,
            row.x + 2,
            row.y + row.height() as i32 / 2,
            ActionSource::Pointer,
        );

        assert_eq!(control, Some(AppControl::Continue));
        assert_eq!(
            app.page_state.selected_timeline_context,
            TimelineContext::OutputFx
        );
    }

    #[test]
    fn timeline_resized_content_rects_do_not_overlap_input_fx_band() {
        let app = App::new();
        let content_bounds = Rect::new(40, 40, 1200, 620);
        let (_, body_bounds) =
            crate::ui::split_top_strip(content_bounds, 28, 6).expect("timeline content");
        let (_, timeline_bounds) =
            crate::ui::split_top_strip(body_bounds, transport_strip_height(), 8)
                .expect("timeline body");
        let columns = crate::ui::track_column_pairs(timeline_bounds, app.project.tracks.len());
        let (full_bounds, detail_bounds) = columns[0];
        let (body_full_bounds, body_detail_bounds) =
            app.track_column_body_bounds(full_bounds, detail_bounds);
        let (input_band, _) =
            app.track_fx_band_rects(full_bounds, detail_bounds, &app.project.tracks[0]);
        let full_content = crate::ui::track_content_rect(body_full_bounds, app.timeline_flow);
        let detail_content = crate::ui::track_content_rect(body_detail_bounds, app.timeline_flow);
        let intersects = |a: Rect, b: Rect| {
            a.x < b.x + b.width() as i32
                && a.x + a.width() as i32 > b.x
                && a.y < b.y + b.height() as i32
                && a.y + a.height() as i32 > b.y
        };

        assert!(!intersects(input_band, full_content));
        assert!(!intersects(input_band, detail_content));
    }

    #[test]
    fn timeline_fx_adjust_and_move_actions_update_selected_output_row() {
        let mut app = App::new();
        app.page_state.current_page = AppPage::Timeline;
        app.page_state.selected_timeline_context = TimelineContext::OutputFx;
        app.page_state.selected_timeline_fx_field = TimelineFxField::Kind;

        let before_kind = app
            .selected_timeline_fx_slot(app.project.active_track().unwrap(), MidiFxChainKind::Output)
            .unwrap()
            .effect
            .kind();
        app.adjust_page_item(1);
        let after_kind = app
            .selected_timeline_fx_slot(app.project.active_track().unwrap(), MidiFxChainKind::Output)
            .unwrap()
            .effect
            .kind();
        assert_ne!(before_kind, after_kind);

        app.page_state.selected_timeline_fx_field = TimelineFxField::Move;
        let before_row = app.selected_timeline_fx_row(MidiFxChainKind::Output);
        app.adjust_page_item(1);
        let after_row = app.selected_timeline_fx_row(MidiFxChainKind::Output);
        assert!(after_row >= before_row);
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
    fn timeline_unselected_fx_row_prioritizes_kind_and_primary_value_width() {
        let mut app = App::new();
        app.project.tracks[0].midi_fx.output_fx[0] = Some(MidiFxSlot {
            enabled: true,
            effect: MidiFx::Arp {
                step_ticks: 240,
                order: crate::midi_fx::ArpOrder::Up,
                gate_percent: 100,
            },
        });
        let content_bounds = Rect::new(40, 40, 1200, 620);
        let (_, body_bounds) =
            crate::ui::split_top_strip(content_bounds, 28, 6).expect("timeline content");
        let (_, timeline_bounds) =
            crate::ui::split_top_strip(body_bounds, transport_strip_height(), 8)
                .expect("timeline body");
        let columns = crate::ui::track_column_pairs(timeline_bounds, app.project.tracks.len());
        let (full_bounds, detail_bounds) = columns[0];
        let (_, output_band) =
            app.track_fx_band_rects(full_bounds, detail_bounds, &app.project.tracks[0]);
        let displayed = app.displayed_timeline_fx_slot_indices(MidiFxChainKind::Output);
        let layout = app.timeline_fx_row_layouts(
            output_band,
            &displayed,
            &app.project.tracks[0].midi_fx.output_fx,
            None,
        )[0];

        assert!(layout.param_primary.width() > 0);
        assert!(layout.kind.width() < layout.row.width());
        assert!(layout.param_secondary.width() > 0);
        assert!(layout.delete.width() > 0);
    }

    #[test]
    fn timeline_fx_row_layout_drops_low_priority_controls_when_narrow() {
        let mut app = App::new();
        app.project.tracks[0].midi_fx.output_fx[0] = Some(MidiFxSlot {
            enabled: true,
            effect: MidiFx::Arp {
                step_ticks: 240,
                order: crate::midi_fx::ArpOrder::Up,
                gate_percent: 100,
            },
        });
        let displayed = vec![Some(0)];
        let layout = app.timeline_fx_row_layouts(
            Rect::new(10, 10, 56, 14),
            &displayed,
            &app.project.tracks[0].midi_fx.output_fx,
            None,
        )[0];
        let row_right = layout.row.x + layout.row.width() as i32;
        for rect in [
            layout.enabled,
            layout.kind,
            layout.param_primary,
            layout.param_secondary,
            layout.overflow,
            layout.move_up,
            layout.move_down,
            layout.delete,
        ] {
            if rect.x >= layout.row.x {
                assert!(rect.x + rect.width() as i32 <= row_right);
            }
        }
        assert!(layout.kind.width() > 0);
        assert!(layout.param_primary.width() > 0);
        assert!(layout.delete.width() > 0);
        assert!(layout.param_secondary.x < layout.row.x);
        assert!(layout.move_up.x < layout.row.x);
        assert!(layout.move_down.x < layout.row.x);
    }

    #[test]
    fn timeline_selected_fx_row_uses_same_compact_layout() {
        let mut app = App::new();
        app.project.tracks[0].midi_fx.output_fx[0] = Some(MidiFxSlot {
            enabled: true,
            effect: MidiFx::Arp {
                step_ticks: 240,
                order: crate::midi_fx::ArpOrder::Up,
                gate_percent: 100,
            },
        });
        let content_bounds = Rect::new(40, 40, 1200, 620);
        let (_, body_bounds) =
            crate::ui::split_top_strip(content_bounds, 28, 6).expect("timeline content");
        let (_, timeline_bounds) =
            crate::ui::split_top_strip(body_bounds, transport_strip_height(), 8)
                .expect("timeline body");
        let columns = crate::ui::track_column_pairs(timeline_bounds, app.project.tracks.len());
        let (full_bounds, detail_bounds) = columns[0];
        let (_, output_band) =
            app.track_fx_band_rects(full_bounds, detail_bounds, &app.project.tracks[0]);
        let displayed = app.displayed_timeline_fx_slot_indices(MidiFxChainKind::Output);
        let unselected_layout = app.timeline_fx_row_layouts(
            output_band,
            &displayed,
            &app.project.tracks[0].midi_fx.output_fx,
            None,
        )[0];
        let layout = app.timeline_fx_row_layouts(
            output_band,
            &displayed,
            &app.project.tracks[0].midi_fx.output_fx,
            Some(0),
        )[0];

        assert!(layout.param_secondary.width() > 0);
        assert!(layout.move_up.width() > 0);
        assert!(layout.move_down.width() > 0);
        assert!(layout.delete.width() > 0);
        assert_eq!(layout.kind.width(), unselected_layout.kind.width());
        assert_eq!(
            layout.param_secondary.width(),
            unselected_layout.param_secondary.width()
        );
        assert_eq!(layout.delete.width(), unselected_layout.delete.width());
    }

    #[test]
    fn timeline_fx_row_places_secondary_parameter_before_overflow() {
        let mut app = App::new();
        app.project.tracks[0].midi_fx.output_fx[0] = Some(MidiFxSlot {
            enabled: true,
            effect: MidiFx::Arp {
                step_ticks: 240,
                order: crate::midi_fx::ArpOrder::Up,
                gate_percent: 100,
            },
        });
        let displayed = vec![Some(0)];
        let layout = app.timeline_fx_row_layouts(
            Rect::new(10, 10, 120, 14),
            &displayed,
            &app.project.tracks[0].midi_fx.output_fx,
            Some(0),
        )[0];

        assert!(layout.param_secondary.width() > 0);
        assert!(layout.overflow.width() > 0);
        assert!(layout.param_secondary.x < layout.overflow.x);
    }

    #[test]
    fn overflow_label_uses_window_position() {
        assert_eq!(timeline_fx_overflow_label(2, 0), "--");
        assert_eq!(timeline_fx_overflow_label(3, 0), "1/2");
        assert_eq!(timeline_fx_overflow_label(3, 1), "2/2");
    }

    #[test]
    fn timeline_fx_kind_display_uses_short_labels_at_compact_widths() {
        let slot = MidiFxSlot {
            enabled: true,
            effect: MidiFx::Arp {
                step_ticks: 240,
                order: crate::midi_fx::ArpOrder::Up,
                gate_percent: 100,
            },
        };

        assert_eq!(super::timeline_fx_kind_display(&slot, 19), "AR");
        assert_eq!(super::timeline_fx_kind_display(&slot, 20), "ARP");
    }

    #[test]
    fn timeline_fx_row_splits_width_evenly_between_two_visible_params() {
        let mut app = App::new();
        app.project.tracks[0].midi_fx.output_fx[0] = Some(MidiFxSlot {
            enabled: true,
            effect: MidiFx::Arp {
                step_ticks: 240,
                order: crate::midi_fx::ArpOrder::Up,
                gate_percent: 100,
            },
        });
        let displayed = vec![Some(0)];
        let layout = app.timeline_fx_row_layouts(
            Rect::new(10, 10, 120, 14),
            &displayed,
            &app.project.tracks[0].midi_fx.output_fx,
            Some(0),
        )[0];

        assert!(layout.param_primary.width() > 0);
        assert!(layout.param_secondary.width() > 0);
        assert!(
            (layout.param_primary.width() as i32 - layout.param_secondary.width() as i32).abs()
                <= 1
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
    fn timeline_fx_enabled_click_toggles_effect_without_changing_kind() {
        let mut app = App::new();
        app.project.active_track_mut().unwrap().midi_fx.output_fx =
            vec![Some(MidiFxSlot::default()), None, None, None];
        app.page_state.current_page = AppPage::Timeline;
        app.page_state.selected_timeline_context = TimelineContext::OutputFx;
        app.set_selected_timeline_fx_row(MidiFxChainKind::Output, 0);
        let before_enabled = app.project.tracks[0].midi_fx.output_fx[0]
            .as_ref()
            .unwrap()
            .enabled;
        let before_kind = app.project.tracks[0].midi_fx.output_fx[0]
            .as_ref()
            .unwrap()
            .effect
            .kind();
        let content_bounds = Rect::new(40, 40, 1200, 620);
        let (_, body_bounds) =
            crate::ui::split_top_strip(content_bounds, 28, 6).expect("timeline content");
        let (_, timeline_bounds) =
            crate::ui::split_top_strip(body_bounds, transport_strip_height(), 8)
                .expect("timeline body");
        let columns = crate::ui::track_column_pairs(timeline_bounds, app.project.tracks.len());
        let (full_bounds, detail_bounds) = columns[0];
        let (_, output_band) =
            app.track_fx_band_rects(full_bounds, detail_bounds, &app.project.tracks[0]);
        let displayed = app.displayed_timeline_fx_slot_indices(MidiFxChainKind::Output);
        let layout = app.timeline_fx_row_layouts(
            output_band,
            &displayed,
            &app.project.tracks[0].midi_fx.output_fx,
            Some(0),
        )[0];

        let control = app.handle_timeline_pointer(
            content_bounds,
            layout.enabled.x + layout.enabled.width() as i32 / 2,
            layout.enabled.y + layout.enabled.height() as i32 / 2,
            ActionSource::Pointer,
        );

        assert_eq!(control, Some(AppControl::Continue));
        assert_eq!(
            app.page_state.selected_timeline_fx_field,
            TimelineFxField::Enabled
        );
        let after_slot = app.project.tracks[0].midi_fx.output_fx[0].as_ref().unwrap();
        assert_ne!(after_slot.enabled, before_enabled);
        assert_eq!(after_slot.effect.kind(), before_kind);
    }

    #[test]
    fn timeline_fx_enabled_chip_hides_label_when_kind_title_is_visible() {
        let slot = MidiFxSlot::default();
        assert_eq!(super::timeline_fx_enabled_chip_label(&slot, true), "");
    }

    #[test]
    fn timeline_fx_enabled_chip_uses_two_letter_code_when_kind_title_is_hidden() {
        let slot = MidiFxSlot::default();
        assert_eq!(super::timeline_fx_enabled_chip_label(&slot, false), "TR");
    }

    #[test]
    fn timeline_fx_enabled_and_kind_rects_are_disjoint() {
        let app = App::new();
        let displayed = app.displayed_timeline_fx_slot_indices(MidiFxChainKind::Output);
        let layout = app.timeline_fx_row_layouts(
            Rect::new(10, 10, 120, 14),
            &displayed,
            &app.project.tracks[0].midi_fx.output_fx,
            Some(0),
        )[0];
        assert!(layout.enabled.x + layout.enabled.width() as i32 <= layout.kind.x);
    }

    #[test]
    fn timeline_fx_delete_chip_click_removes_effect() {
        let mut app = App::new();
        app.project.active_track_mut().unwrap().midi_fx.output_fx =
            vec![Some(MidiFxSlot::default()), None, None, None];
        app.page_state.selected_timeline_context = TimelineContext::OutputFx;
        app.set_selected_timeline_fx_row(MidiFxChainKind::Output, 0);
        let content_bounds = Rect::new(40, 40, 1200, 620);
        let (_, body_bounds) =
            crate::ui::split_top_strip(content_bounds, 28, 6).expect("timeline content");
        let (_, timeline_bounds) =
            crate::ui::split_top_strip(body_bounds, transport_strip_height(), 8)
                .expect("timeline body");
        let columns = crate::ui::track_column_pairs(timeline_bounds, app.project.tracks.len());
        let (full_bounds, detail_bounds) = columns[0];
        let (_, output_band) =
            app.track_fx_band_rects(full_bounds, detail_bounds, &app.project.tracks[0]);
        let displayed = app.displayed_timeline_fx_slot_indices(MidiFxChainKind::Output);
        let layout = app.timeline_fx_row_layouts(
            output_band,
            &displayed,
            &app.project.tracks[0].midi_fx.output_fx,
            Some(0),
        )[0];
        let before = app
            .active_timeline_fx_slot_indices(MidiFxChainKind::Output)
            .len();

        let control = app.handle_timeline_pointer(
            content_bounds,
            layout.delete.x + layout.delete.width() as i32 / 2,
            layout.delete.y + layout.delete.height() as i32 / 2,
            ActionSource::Pointer,
        );

        assert_eq!(control, Some(AppControl::Continue));
        let after = app
            .active_timeline_fx_slot_indices(MidiFxChainKind::Output)
            .len();
        assert_eq!(after, before - 1);
    }

    #[test]
    fn timeline_add_row_click_inserts_effect_on_first_click() {
        let mut app = App::new();
        app.project.active_track_mut().unwrap().midi_fx.output_fx =
            vec![Some(MidiFxSlot::default()), None, None, None];
        let content_bounds = Rect::new(40, 40, 1200, 620);
        let (_, body_bounds) =
            crate::ui::split_top_strip(content_bounds, 28, 6).expect("timeline content");
        let (_, timeline_bounds) =
            crate::ui::split_top_strip(body_bounds, transport_strip_height(), 8)
                .expect("timeline body");
        let columns = crate::ui::track_column_pairs(timeline_bounds, app.project.tracks.len());
        let (full_bounds, detail_bounds) = columns[0];
        let (_, output_band) =
            app.track_fx_band_rects(full_bounds, detail_bounds, &app.project.tracks[0]);
        let displayed = app.displayed_timeline_fx_slot_indices(MidiFxChainKind::Output);
        let layouts = app.timeline_fx_row_layouts(
            output_band,
            &displayed,
            &app.project.tracks[0].midi_fx.output_fx,
            None,
        );
        let add_row = layouts.last().expect("add row").row;
        let before = app
            .active_timeline_fx_slot_indices(MidiFxChainKind::Output)
            .len();

        let control = app.handle_timeline_pointer(
            content_bounds,
            add_row.x + 4,
            add_row.y + add_row.height() as i32 / 2,
            ActionSource::Pointer,
        );

        assert_eq!(control, Some(AppControl::Continue));
        let after = app
            .active_timeline_fx_slot_indices(MidiFxChainKind::Output)
            .len();
        assert_eq!(after, before + 1);
    }

    #[test]
    fn timeline_fx_hover_targets_kind_action_not_routing() {
        let app = App::new();
        let mut app = app;
        app.project.active_track_mut().unwrap().midi_fx.output_fx =
            vec![Some(MidiFxSlot::default()), None, None, None];
        let content_bounds = Rect::new(40, 40, 1200, 620);
        let (_, body_bounds) =
            crate::ui::split_top_strip(content_bounds, 28, 6).expect("timeline content");
        let (_, timeline_bounds) =
            crate::ui::split_top_strip(body_bounds, transport_strip_height(), 8)
                .expect("timeline body");
        let columns = crate::ui::track_column_pairs(timeline_bounds, app.project.tracks.len());
        let (full_bounds, detail_bounds) = columns[0];
        let (_, output_band) =
            app.track_fx_band_rects(full_bounds, detail_bounds, &app.project.tracks[0]);
        let displayed = app.displayed_timeline_fx_slot_indices(MidiFxChainKind::Output);
        let layout = app.timeline_fx_row_layouts(
            output_band,
            &displayed,
            &app.project.tracks[0].midi_fx.output_fx,
            None,
        )[0];

        let target = app
            .timeline_discoverability_targets(content_bounds)
            .into_iter()
            .find_map(|(rect, target)| {
                super::rect_contains(
                    rect,
                    layout.kind.x + layout.kind.width() as i32 / 2,
                    layout.kind.y + layout.kind.height() as i32 / 2,
                )
                .then_some(target)
            })
            .expect("discoverability target");

        assert_eq!(target.action, AppAction::CycleSelectedTimelineFxKind);
    }

    #[test]
    fn output_fx_lower_empty_band_space_does_not_hit_row() {
        let mut app = App::new();
        app.project.tracks[0].midi_fx.output_fx =
            vec![Some(MidiFxSlot::default()), None, None, None];
        app.project.tracks[1].midi_fx.output_fx = vec![
            Some(MidiFxSlot::default()),
            Some(MidiFxSlot::default()),
            Some(MidiFxSlot::default()),
            None,
        ];
        let content_bounds = Rect::new(40, 40, 1200, 620);
        let (_, body_bounds) =
            crate::ui::split_top_strip(content_bounds, 28, 6).expect("timeline content");
        let (_, timeline_bounds) =
            crate::ui::split_top_strip(body_bounds, transport_strip_height(), 8)
                .expect("timeline body");
        let layout = app.visible_timeline_track_layouts(timeline_bounds)[0];
        let displayed = app.displayed_timeline_fx_slot_indices(MidiFxChainKind::Output);
        let row = app.timeline_fx_row_layouts(
            layout.output_fx_rect,
            &displayed,
            &app.project.tracks[0].midi_fx.output_fx,
            Some(0),
        )[0]
        .row;
        let x = row.x + row.width() as i32 / 2;
        let y = layout.output_fx_rect.y + layout.output_fx_rect.height() as i32 - 2;

        assert!(y > row.y + row.height() as i32);
        assert!(app
            .timeline_fx_hit(
                TimelineContext::OutputFx,
                layout.output_fx_rect,
                &app.project.tracks[0],
                x,
                y,
            )
            .is_none());
    }

    #[test]
    fn canonical_timeline_layout_keeps_output_fx_band_disjoint_from_body_content() {
        let app = App::new();
        let content_bounds = Rect::new(40, 40, 1200, 620);
        let (_, body_bounds) =
            crate::ui::split_top_strip(content_bounds, 28, 6).expect("timeline content");
        let (_, timeline_bounds) =
            crate::ui::split_top_strip(body_bounds, transport_strip_height(), 8)
                .expect("timeline body");
        let layout = app.visible_timeline_track_layouts(timeline_bounds)[0];
        let intersects = |a: Rect, b: Rect| {
            a.x < b.x + b.width() as i32
                && a.x + a.width() as i32 > b.x
                && a.y < b.y + b.height() as i32
                && a.y + a.height() as i32 > b.y
        };

        assert!(!intersects(layout.output_fx_rect, layout.full_content_rect));
        assert!(!intersects(
            layout.output_fx_rect,
            layout.detail_content_rect
        ));
    }

    #[test]
    fn canonical_output_fx_row_point_does_not_land_in_body_content() {
        let app = App::new();
        let content_bounds = Rect::new(40, 40, 1200, 620);
        let (_, body_bounds) =
            crate::ui::split_top_strip(content_bounds, 28, 6).expect("timeline content");
        let (_, timeline_bounds) =
            crate::ui::split_top_strip(body_bounds, transport_strip_height(), 8)
                .expect("timeline body");
        let layout = app.visible_timeline_track_layouts(timeline_bounds)[0];
        let displayed = app.displayed_timeline_fx_slot_indices(MidiFxChainKind::Output);
        let row = app.timeline_fx_row_layouts(
            layout.output_fx_rect,
            &displayed,
            &app.project.tracks[0].midi_fx.output_fx,
            Some(0),
        )[0]
        .row;
        let x = row.x + row.width() as i32 / 2;
        let y = row.y + row.height() as i32 / 2;

        assert!(!super::rect_contains(layout.full_content_rect, x, y));
        assert!(!super::rect_contains(layout.detail_content_rect, x, y));
    }

    #[test]
    fn output_fx_band_starts_below_track_body_with_fixed_gap() {
        let app = App::new();
        let content_bounds = Rect::new(40, 40, 1200, 620);
        let (_, body_bounds) =
            crate::ui::split_top_strip(content_bounds, 28, 6).expect("timeline content");
        let (_, timeline_bounds) =
            crate::ui::split_top_strip(body_bounds, transport_strip_height(), 8)
                .expect("timeline body");
        let columns = crate::ui::track_column_pairs(timeline_bounds, app.project.tracks.len());
        let (full_bounds, detail_bounds) = columns[0];
        let (body_full_bounds, body_detail_bounds) =
            app.track_column_body_bounds(full_bounds, detail_bounds);
        let body_pair = crate::ui::union_rect(body_full_bounds, body_detail_bounds);
        let (_, output_rect) =
            app.track_fx_band_rects(full_bounds, detail_bounds, &app.project.tracks[0]);

        assert_eq!(output_rect.y, body_pair.y + body_pair.height() as i32 + 4);
    }

    #[test]
    fn timeline_context_indicator_sits_left_of_selected_context_with_gap() {
        let mut app = App::new();
        app.page_state.selected_timeline_context = TimelineContext::TrackTimeline;
        let content_bounds = Rect::new(40, 40, 1200, 620);
        let (_, body_bounds) =
            crate::ui::split_top_strip(content_bounds, 28, 6).expect("timeline content");
        let (_, timeline_bounds) =
            crate::ui::split_top_strip(body_bounds, transport_strip_height(), 8)
                .expect("timeline body");
        let columns = crate::ui::track_column_pairs(timeline_bounds, app.project.tracks.len());
        let (full_bounds, detail_bounds) = columns[0];
        let track = &app.project.tracks[0];
        let layout = app.timeline_track_layout(0, full_bounds, detail_bounds);
        let context_rect = layout.fx_rect(app.page_state.selected_timeline_context);
        let indicator = app
            .timeline_context_indicator_rect(full_bounds, detail_bounds, track)
            .expect("indicator rect");

        assert_eq!(indicator.width(), 1);
        assert_eq!(indicator.height(), context_rect.height());
        assert_eq!(indicator.y, context_rect.y);
        assert_eq!(indicator.x + indicator.width() as i32 + 1, context_rect.x);
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
    fn pointer_position_uses_render_coordinates_for_mouse() {
        let event = sdl3::event::Event::MouseButtonDown {
            timestamp: 0,
            window_id: 1,
            which: 0,
            mouse_btn: sdl3::mouse::MouseButton::Left,
            clicks: 1,
            x: 512.5,
            y: 288.25,
        };

        assert_eq!(
            super::pointer_down_position(&event, (1280, 720)),
            Some((512, 288, crate::actions::ActionSource::Pointer))
        );
    }

    #[test]
    fn page_frame_layout_matches_draw_content_height_contract() {
        let app = App::new();
        let surface = crate::ui::surface_rect(1280, 720);
        let inset = crate::ui::inset_rect(surface, 24, 24).expect("inset");
        let (_, content_bounds, footer_bounds) = app.page_frame_layout(inset).expect("layout");
        let (_, page_area_bounds) = crate::ui::split_top_strip(inset, 28, 12).expect("page split");

        assert_eq!(content_bounds.y, page_area_bounds.y);
        assert_eq!(
            footer_bounds.y + footer_bounds.height() as i32,
            page_area_bounds.y + page_area_bounds.height() as i32
        );
        assert_eq!(content_bounds.height() + 22 + 8, page_area_bounds.height());
    }

    #[test]
    fn pointer_position_uses_converted_render_coordinates_for_touch() {
        let event = sdl3::event::Event::FingerDown {
            timestamp: 0,
            touch_id: 1,
            finger_id: 1,
            x: 0.5,
            y: 0.5,
            dx: 0.0,
            dy: 0.0,
            pressure: 1.0,
        };

        assert_eq!(
            super::pointer_down_position(&event, (1280, 720)),
            Some((640, 360, crate::actions::ActionSource::Touch))
        );
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
