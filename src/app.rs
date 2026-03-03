use crate::actions::{
    ActionSource, AppAction, KeyboardBindings, action_label, built_in_keyboard_binding_labels,
};
use crate::engine::EngineConfig;
use crate::link::{LinkRuntime, LinkSnapshot};
use crate::mapping::{
    MappingEntry, MappingSourceKind, cycle_mapping_scope_value, cycle_mapping_source_device_label,
    cycle_mapping_source_kind, cycle_mapping_source_label, cycle_mapping_target_label,
    default_mapping_source_device, default_scope_label, default_source_label, demo_mappings,
    mapping_entry_key_actions, mapping_entry_targets_action, mapping_entry_to_actions,
};
use crate::midi_io::{
    MidiDeviceCatalog, MidiInputEvent, MidiInputMessage, MidiInputRuntime, MidiOutputRuntime,
    MidiPortRef,
};
use crate::pages::{
    AppPage, AppPageState, MappingField, MappingPageMode, MidiIoListFocus, RoutingField,
};
use crate::project::{Project, Track};
use crate::routing::MidiChannelFilter;
use crate::state::PersistedAppState;
use crate::ui::{LayoutMode, TimelineFlow};
use image::RgbaImage;
use sdl3::pixels::Color;
use sdl3::pixels::PixelFormat;
use sdl3::rect::Rect;
use sdl3::render::{Canvas, RenderTarget};
use sdl3::surface::SurfaceRef;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

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
    viewport_size: (u32, u32),
    ui_scale_override: Option<f32>,
    transport_ticks: u64,
    playhead_ticks: u64,
    link_snapshot: LinkSnapshot,
    note_additive_select_held: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AppOverlay {
    MappingsQuickView,
    Discoverability,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct OverlayState {
    active: Option<AppOverlay>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct StatusState {
    hovered_target: Option<DiscoverabilityTarget>,
    last_action: Option<LastActionStatus>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct DirectMappingState {
    mode: DirectMappingMode,
    origin: DirectMappingOrigin,
    status_message: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum DirectMappingMode {
    #[default]
    Inactive,
    Targeting,
    AwaitingInput(DirectMappingTarget),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum DirectMappingOrigin {
    #[default]
    InPlace,
    MappingsPage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DirectMappingTarget {
    action: AppAction,
    target_label: &'static str,
    scope_label: &'static str,
    display_scope: Option<&'static str>,
    hit_rect: Rect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct LastActionStatus {
    action: AppAction,
    source: ActionSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DiscoverabilityTarget {
    action: AppAction,
    display_scope: Option<&'static str>,
    allowed_mapping_scopes: &'static [&'static str],
    overlay_slot: Option<Rect>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ActionDiscoverabilitySummary {
    title: String,
    badges: Vec<MappingBadge>,
    total_bindings: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MappingBadge {
    text: String,
    source_kind: MappingSourceKind,
    built_in: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiCaptureOptions {
    pub output_dir: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VideoMode {
    #[default]
    Windowed,
    Fullscreen,
    KmsDrmConsole,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RunOptions {
    pub video_mode: VideoMode,
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
        let mut link = LinkRuntime::new(f64::from(project.transport.tempo_bpm));
        link.set_enabled(project.transport.link_enabled);
        link.set_start_stop_sync(project.transport.link_start_stop_sync);
        let link_snapshot = link.refresh();
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
            viewport_size: (1280, 720),
            ui_scale_override: None,
            transport_ticks: 0,
            playhead_ticks: 0,
            link_snapshot,
            note_additive_select_held: false,
        }
    }

    pub fn set_ui_scale_override(&mut self, scale: Option<f32>) {
        self.ui_scale_override = scale.filter(|value| *value >= 1.0);
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
            self.advance_playhead(now.saturating_duration_since(last_frame_at));
            last_frame_at = now;
            self.configure_window_canvas(&mut canvas)?;

            self.update_window_title(canvas.window_mut())?;
            self.draw(&mut canvas)?;
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
            self.advance_playhead(now.saturating_duration_since(last_frame_at));
            last_frame_at = now;
            self.configure_window_canvas(&mut canvas)?;

            self.update_window_title(canvas.window_mut())?;
            self.draw(&mut canvas)?;
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

        for spec in capture_specs() {
            self.page_state.current_page = spec.page;
            self.overlay_state.active = spec.overlay;
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
        let (width, height) = active_draw_size(canvas.output_size()?, self.viewport_size);
        let surface = crate::ui::surface_rect(width, height);
        let inset = crate::ui::inset_rect(surface, 24, 24)?;
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

        canvas.set_draw_color(Color::RGB(18, 24, 38));
        canvas.clear();

        canvas.set_draw_color(Color::RGB(28, 34, 50));
        canvas.fill_rect(surface)?;
        canvas.set_draw_color(Color::RGB(88, 96, 120));
        canvas.draw_rect(surface)?;

        self.draw_page_tabs(canvas, tabs_bounds)?;

        match self.page_state.current_page {
            AppPage::Timeline => self.draw_timeline_page(canvas, content_bounds)?,
            AppPage::Mappings => self.draw_mappings_page(canvas, content_bounds)?,
            AppPage::MidiIo => self.draw_midi_io_page(canvas, content_bounds)?,
            AppPage::Routing => self.draw_routing_page(canvas, content_bounds)?,
        }

        self.draw_direct_mapping_targets(canvas, tabs_bounds, content_bounds)?;
        self.draw_overlay(canvas, inset)?;
        self.draw_footer(canvas, footer_bounds)?;

        canvas.present();
        Ok(())
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
        let tabs = crate::ui::equal_columns(bounds, AppPage::ALL.len(), 10);
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

    fn draw_timeline_page<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        content_bounds: Rect,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (header_bounds, body_bounds) = crate::ui::split_top_strip(content_bounds, 28, 6)?;
        let (transport_bounds, timeline_bounds) = crate::ui::split_top_strip(body_bounds, 24, 8)?;
        let reset_button = self.global_loop_reset_button_rect(header_bounds);
        canvas.set_draw_color(Color::RGB(34, 44, 64));
        canvas.fill_rect(header_bounds)?;
        canvas.set_draw_color(Color::RGB(88, 96, 120));
        canvas.draw_rect(header_bounds)?;
        crate::ui::draw_text_fitted(
            canvas,
            "Timeline",
            Rect::new(header_bounds.x + 8, header_bounds.y + 8, 84, 8),
            1,
            Color::RGB(192, 206, 222),
        )?;
        crate::ui::draw_text_fitted(
            canvas,
            "Vertical",
            Rect::new(header_bounds.x + 96, header_bounds.y + 8, 54, 8),
            1,
            Color::RGB(212, 220, 230),
        )?;
        crate::ui::draw_text_fitted(
            canvas,
            "Song columns + loop detail",
            Rect::new(header_bounds.x + 212, header_bounds.y + 8, 180, 8),
            1,
            Color::RGB(190, 198, 210),
        )?;
        canvas.set_draw_color(Color::RGB(122, 84, 52));
        canvas.fill_rect(reset_button)?;
        canvas.set_draw_color(Color::RGB(244, 232, 146));
        canvas.draw_rect(reset_button)?;
        crate::ui::draw_text_fitted(
            canvas,
            "Reset Song Loop",
            Rect::new(
                reset_button.x + 8,
                reset_button.y + 8,
                reset_button.width().saturating_sub(16),
                8,
            ),
            1,
            Color::RGB(248, 244, 212),
        )?;
        self.draw_transport_strip(canvas, transport_bounds)?;

        let columns = crate::ui::track_column_pairs(timeline_bounds, self.project.tracks.len());

        for (index, track) in self.project.tracks.iter().enumerate() {
            if let Some((full_bounds, detail_bounds)) = columns.get(index).copied() {
                let is_active = index == self.project.active_track_index;
                self.draw_track_column(canvas, full_bounds, detail_bounds, track, is_active)?;
            }
        }

        if self.overlay_state.active == Some(AppOverlay::Discoverability) {
            self.draw_timeline_discoverability_overlay(canvas, content_bounds)?;
        }

        Ok(())
    }

    fn draw_track_column<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        full_bounds: Rect,
        detail_bounds: Rect,
        track: &Track,
        is_active: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let detail_range = self.detail_loop_range(track);
        let full_accent = if track.state.armed {
            Color::RGB(148, 54, 54)
        } else if is_active {
            Color::RGB(42, 90, 168)
        } else {
            Color::RGB(36, 58, 92)
        };
        let detail_accent = if detail_range != track.loop_region {
            Color::RGB(170, 120, 44)
        } else if track.state.loop_enabled && self.project.transport.loop_enabled {
            Color::RGB(178, 104, 34)
        } else if is_active {
            Color::RGB(124, 82, 46)
        } else {
            Color::RGB(74, 54, 40)
        };

        self.draw_track_subcolumn(
            canvas,
            full_bounds,
            full_accent,
            0,
            self.project.full_song_range().length_ticks,
            self.effective_track_playhead(track),
            is_active,
            false,
            track,
        )?;
        self.draw_track_subcolumn(
            canvas,
            detail_bounds,
            detail_accent,
            detail_range.start_ticks,
            detail_range.length_ticks,
            self.effective_track_playhead(track),
            is_active,
            true,
            track,
        )?;
        self.draw_track_status_strip(canvas, full_bounds, detail_bounds, track, is_active)?;

        Ok(())
    }

    fn draw_track_status_strip<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        full_bounds: Rect,
        detail_bounds: Rect,
        track: &Track,
        is_active: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let pair_bounds = crate::ui::union_rect(full_bounds, detail_bounds);
        let status_rect = crate::ui::track_status_rect(pair_bounds, self.timeline_flow);
        canvas.set_draw_color(Color::RGB(26, 34, 52));
        canvas.fill_rect(status_rect)?;
        canvas.set_draw_color(if is_active {
            Color::RGB(98, 110, 136)
        } else {
            Color::RGB(68, 78, 98)
        });
        canvas.draw_rect(status_rect)?;

        for indicator in crate::ui::track_indicators(status_rect) {
            let (enabled, fill, border, label) = match indicator.kind {
                crate::ui::TrackIndicatorKind::Armed => (
                    track.state.armed,
                    Color::RGB(188, 72, 72),
                    Color::RGB(238, 138, 138),
                    if indicator.rect.width() >= 24 {
                        "ARM"
                    } else {
                        "A"
                    },
                ),
                crate::ui::TrackIndicatorKind::Recording => (
                    track.active_take.is_some(),
                    Color::RGB(214, 64, 64),
                    Color::RGB(248, 132, 132),
                    if indicator.rect.width() >= 24 {
                        "REC"
                    } else {
                        "R"
                    },
                ),
                crate::ui::TrackIndicatorKind::Muted => (
                    track.state.muted,
                    Color::RGB(114, 120, 132),
                    Color::RGB(180, 186, 198),
                    if indicator.rect.width() >= 24 {
                        "MUTE"
                    } else {
                        "M"
                    },
                ),
                crate::ui::TrackIndicatorKind::Solo => (
                    track.state.soloed,
                    Color::RGB(82, 162, 92),
                    Color::RGB(144, 224, 154),
                    if indicator.rect.width() >= 24 {
                        "SOLO"
                    } else {
                        "S"
                    },
                ),
            };
            canvas.set_draw_color(if enabled {
                fill
            } else if is_active {
                Color::RGB(44, 52, 68)
            } else {
                Color::RGB(34, 42, 56)
            });
            canvas.fill_rect(indicator.rect)?;
            canvas.set_draw_color(if enabled {
                border
            } else {
                Color::RGB(76, 86, 104)
            });
            canvas.draw_rect(indicator.rect)?;
            crate::ui::draw_text_fitted(
                canvas,
                label,
                Rect::new(
                    indicator.rect.x + 3,
                    indicator.rect.y + 1,
                    indicator.rect.width().saturating_sub(6),
                    indicator.rect.height().saturating_sub(2),
                ),
                1,
                if enabled {
                    Color::RGB(248, 244, 236)
                } else {
                    Color::RGB(160, 170, 186)
                },
            )?;
        }

        Ok(())
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

        let label_rect = crate::ui::track_label_rect(bounds, self.timeline_flow);
        let content_rect = crate::ui::track_content_rect(bounds, self.timeline_flow);
        canvas.set_draw_color(accent);
        canvas.fill_rect(label_rect)?;
        if track.state.passthrough {
            let rail = crate::ui::passthrough_rail_rect(bounds);
            canvas.set_draw_color(Color::RGB(74, 210, 214));
            canvas.fill_rect(rail)?;
        }

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
        for tick in crate::ui::timeline_ruler_ticks(content_rect, self.timeline_flow) {
            canvas.set_draw_color(Color::RGB(166, 178, 198));
            canvas.fill_rect(tick)?;
        }

        if detail {
            let loop_tag = crate::ui::detail_badge_rect(label_rect);
            canvas.set_draw_color(
                if track.state.loop_enabled && self.project.transport.loop_enabled {
                    Color::RGB(252, 192, 104)
                } else {
                    Color::RGB(88, 82, 76)
                },
            );
            canvas.fill_rect(loop_tag)?;
        }

        let role_badge = Rect::new(
            label_rect.x + 4,
            label_rect.y + 4,
            label_rect.width().saturating_sub(8).min(28),
            8,
        );
        canvas.set_draw_color(if detail {
            Color::RGB(94, 68, 44)
        } else {
            Color::RGB(38, 58, 90)
        });
        canvas.fill_rect(role_badge)?;
        crate::ui::draw_text_fitted(
            canvas,
            if detail { "Loop" } else { "Song" },
            Rect::new(
                role_badge.x + 3,
                role_badge.y + 1,
                role_badge.width().saturating_sub(6),
                8,
            ),
            1,
            Color::RGB(244, 244, 236),
        )?;

        let label_left = label_rect.x + 4;
        crate::ui::draw_text_fitted(
            canvas,
            if detail { "Loop" } else { &track.name },
            Rect::new(
                label_left,
                label_rect.y + 14,
                (label_rect.width() as i32 - (label_left - label_rect.x) - 4).max(0) as u32,
                8,
            ),
            1,
            Color::RGB(244, 244, 236),
        )?;

        let note_range = crate::timeline::LoopRegion::new(view_start_ticks, range_ticks.max(1));
        let selected_note_indices = track.selected_note_indices();
        let focused_note_index = track.focused_note_index();
        let anchor_note_index = track.anchor_note_index();
        for region in
            crate::ui::region_rects(content_rect, &track.regions, note_range, self.timeline_flow)
        {
            canvas.set_draw_color(if region.clipped {
                Color::RGB(108, 88, 56)
            } else if track.state.muted {
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

        if let Some(preview_region) = track.preview_region(
            self.project.transport,
            self.record_capture_ticks(track),
            self.record_context(track),
        ) {
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
            &track.preview_notes(
                self.project.transport,
                self.record_capture_ticks(track),
                self.record_context(track),
            ),
            note_range,
            self.timeline_flow,
        ) {
            canvas.set_draw_color(Color::RGBA(238, 108, 108, 176));
            canvas.fill_rect(note.rect)?;
            canvas.set_draw_color(Color::RGB(255, 176, 176));
            canvas.draw_rect(note.rect)?;
        }

        for note in crate::ui::note_rects(
            content_rect,
            &track.midi_notes,
            note_range,
            self.timeline_flow,
        ) {
            let selected = selected_note_indices.contains(&note.source_index);
            let focused = focused_note_index == Some(note.source_index);
            let anchored = anchor_note_index == Some(note.source_index);
            canvas.set_draw_color(if selected && detail {
                Color::RGB(112, 174, 228)
            } else if selected {
                Color::RGB(88, 136, 194)
            } else if track.state.muted {
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
            } else if track.state.muted {
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

        let playhead = crate::ui::playhead_rect_in_range(
            content_rect,
            self.timeline_flow,
            view_start_ticks,
            range_ticks.max(1),
            playhead_ticks,
        )?;
        canvas.set_draw_color(if self.project.transport.playing {
            Color::RGB(248, 240, 132)
        } else {
            Color::RGB(140, 150, 162)
        });
        canvas.fill_rect(playhead)?;

        Ok(())
    }

    fn draw_transport_strip<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        bounds: Rect,
    ) -> Result<(), Box<dyn std::error::Error>> {
        canvas.set_draw_color(Color::RGB(28, 36, 52));
        canvas.fill_rect(bounds)?;
        canvas.set_draw_color(Color::RGB(88, 96, 120));
        canvas.draw_rect(bounds)?;

        let transport_chips = self.transport_chip_specs();
        let mut cursor_x = bounds.x + 6;
        for chip_spec in transport_chips {
            let label = chip_spec.label.as_str();
            let width = crate::ui::text_width(&label, 1) + 12;
            let chip = Rect::new(
                cursor_x,
                bounds.y + 4,
                width,
                bounds.height().saturating_sub(8),
            );
            canvas.set_draw_color(chip_spec.fill);
            canvas.fill_rect(chip)?;
            crate::ui::draw_text_fitted(
                canvas,
                label,
                Rect::new(chip.x + 6, chip.y + 4, chip.width().saturating_sub(12), 8),
                1,
                Color::RGB(244, 244, 236),
            )?;
            cursor_x += chip.width() as i32 + 6;
            if cursor_x >= bounds.x + bounds.width() as i32 - 240 {
                break;
            }
        }

        let divider = Rect::new(
            cursor_x + 4,
            bounds.y + 4,
            1,
            bounds.height().saturating_sub(8),
        );
        canvas.set_draw_color(Color::RGB(86, 96, 114));
        canvas.fill_rect(divider)?;
        cursor_x = divider.x + 8;

        crate::ui::draw_text_fitted(
            canvas,
            "Sync",
            Rect::new(cursor_x, bounds.y + 8, 22, 8),
            1,
            Color::RGB(170, 180, 196),
        )?;
        cursor_x += 28;

        let right_badges = [
            (
                format!("Q {}", quantize_label(self.project.transport.quantize)),
                Color::RGB(72, 88, 110),
            ),
            (
                format!("Link {}", on_off(self.project.transport.link_enabled)),
                if self.project.transport.link_enabled {
                    Color::RGB(74, 122, 144)
                } else {
                    Color::RGB(68, 76, 92)
                },
            ),
            (
                format!(
                    "Sync {}",
                    on_off(self.project.transport.link_start_stop_sync)
                ),
                Color::RGB(82, 98, 130),
            ),
            (
                format!("Peers {}", self.link_snapshot.peers),
                Color::RGB(66, 80, 102),
            ),
        ];
        for (label, fill) in right_badges {
            let width = crate::ui::text_width(&label, 1) + 12;
            let chip = Rect::new(
                cursor_x,
                bounds.y + 4,
                width,
                bounds.height().saturating_sub(8),
            );
            canvas.set_draw_color(fill);
            canvas.fill_rect(chip)?;
            crate::ui::draw_text_fitted(
                canvas,
                &label,
                Rect::new(chip.x + 6, chip.y + 4, chip.width().saturating_sub(12), 8),
                1,
                Color::RGB(244, 244, 236),
            )?;
            cursor_x += chip.width() as i32 + 6;
        }

        let hint = if self.project.transport.link_enabled {
            "F6 Link  Shift+F6 Sync"
        } else {
            "F6 Link"
        };
        let hint_width = crate::ui::text_width(hint, 1) + 6;
        crate::ui::draw_text_fitted(
            canvas,
            hint,
            Rect::new(
                bounds.x + bounds.width() as i32 - hint_width as i32 - 8,
                bounds.y + 8,
                hint_width,
                8,
            ),
            1,
            Color::RGB(166, 176, 192),
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

    fn direct_mapping_footer_content(&self) -> Option<(String, String, Vec<MappingBadge>)> {
        match self.direct_mapping_state.mode {
            DirectMappingMode::Inactive => {
                if self.status_state.hovered_target.is_some() {
                    None
                } else {
                    self.direct_mapping_state
                        .status_message
                        .as_ref()
                        .map(|message| ("Direct Map".to_string(), message.clone(), Vec::new()))
                }
            }
            DirectMappingMode::Targeting => Some((
                "Direct Map".to_string(),
                "Select a highlighted control, then move a MIDI control. Esc cancels.".to_string(),
                Vec::new(),
            )),
            DirectMappingMode::AwaitingInput(target) => {
                let title = match target.display_scope {
                    Some(scope) => format!("Direct Map: {} ({scope})", target.target_label),
                    None => format!("Direct Map: {}", target.target_label),
                };
                Some((
                    title,
                    "Move a MIDI note or CC now. Esc cancels.".to_string(),
                    self.summarize_direct_mapping_target(target).badges,
                ))
            }
        }
    }

    fn summarize_direct_mapping_target(
        &self,
        target: DirectMappingTarget,
    ) -> ActionDiscoverabilitySummary {
        let mut summary = self.summarize_discoverability_target(DiscoverabilityTarget {
            action: target.action,
            display_scope: target.display_scope,
            allowed_mapping_scopes: &[],
            overlay_slot: None,
        });
        summary.badges.retain(|badge| {
            badge.built_in || self.direct_mapping_badge_matches_scope(badge, target)
        });
        summary.total_bindings = summary.badges.len();
        summary
    }

    fn direct_mapping_badge_matches_scope(
        &self,
        badge: &MappingBadge,
        target: DirectMappingTarget,
    ) -> bool {
        self.mappings.iter().any(|entry| {
            !badge.built_in
                && entry.enabled
                && entry.scope_label == target.scope_label
                && entry.source_kind == badge.source_kind
                && entry.source_label == badge.text
                && mapping_entry_targets_action(entry, target.action)
        })
    }

    fn summarize_discoverability_target(
        &self,
        target: DiscoverabilityTarget,
    ) -> ActionDiscoverabilitySummary {
        let mut badges = built_in_keyboard_binding_labels(target.action)
            .iter()
            .map(|label| MappingBadge {
                text: (*label).to_string(),
                source_kind: MappingSourceKind::Key,
                built_in: true,
            })
            .collect::<Vec<_>>();

        badges.extend(self.mappings.iter().filter_map(|entry| {
            if !mapping_entry_targets_action(entry, target.action) {
                return None;
            }
            if !target.allowed_mapping_scopes.is_empty()
                && !target
                    .allowed_mapping_scopes
                    .iter()
                    .any(|scope| *scope == entry.scope_label.as_str())
            {
                return None;
            }
            Some(MappingBadge {
                text: entry.source_label.clone(),
                source_kind: entry.source_kind,
                built_in: false,
            })
        }));

        badges.sort_by_key(|badge| {
            (
                mapping_source_sort_key(badge.source_kind),
                if badge.built_in { 0 } else { 1 },
                badge.text.clone(),
            )
        });
        badges.dedup_by(|left, right| {
            left.text == right.text
                && left.source_kind == right.source_kind
                && left.built_in == right.built_in
        });

        let title = match target.display_scope {
            Some(scope) => format!("{} ({scope})", action_label(target.action)),
            None => action_label(target.action).to_string(),
        };
        let total_bindings = badges.len();

        ActionDiscoverabilitySummary {
            title,
            badges,
            total_bindings,
        }
    }

    fn draw_mapping_badges<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        bounds: Rect,
        badges: &[MappingBadge],
        total_bindings: usize,
        max_badges: usize,
        max_label_width: usize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut cursor_x = bounds.x;
        let visible = badges.len().min(max_badges);
        for badge in badges.iter().take(visible) {
            let label = compact_badge_text(&badge.text, max_label_width);
            let draw_label = format!("{} {}", badge_kind_prefix(badge.source_kind), label);
            let width = crate::ui::text_width(&draw_label, 1) + 10;
            if cursor_x + width as i32 > bounds.x + bounds.width() as i32 {
                break;
            }
            let chip = Rect::new(
                cursor_x,
                bounds.y + 2,
                width,
                bounds.height().saturating_sub(4),
            );
            let (fill, text) = mapping_badge_palette(badge);
            canvas.set_draw_color(fill);
            canvas.fill_rect(chip)?;
            crate::ui::draw_text_fitted(
                canvas,
                &draw_label,
                Rect::new(chip.x + 5, chip.y + 2, chip.width().saturating_sub(10), 8),
                1,
                text,
            )?;
            cursor_x += chip.width() as i32 + 4;
        }

        let remaining = total_bindings.saturating_sub(visible);
        if remaining > 0 {
            let draw_label = format!("+{remaining}");
            let width = crate::ui::text_width(&draw_label, 1) + 10;
            if cursor_x + width as i32 <= bounds.x + bounds.width() as i32 {
                let chip = Rect::new(
                    cursor_x,
                    bounds.y + 2,
                    width,
                    bounds.height().saturating_sub(4),
                );
                canvas.set_draw_color(Color::RGB(56, 64, 80));
                canvas.fill_rect(chip)?;
                crate::ui::draw_text_fitted(
                    canvas,
                    &draw_label,
                    Rect::new(chip.x + 5, chip.y + 2, chip.width().saturating_sub(10), 8),
                    1,
                    Color::RGB(228, 232, 238),
                )?;
            }
        }

        Ok(())
    }

    fn draw_timeline_discoverability_overlay<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        content_bounds: Rect,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for (rect, target) in self.timeline_discoverability_targets(content_bounds) {
            self.draw_inline_discoverability_badges(canvas, rect, target)?;
        }
        Ok(())
    }

    fn draw_routing_discoverability_overlay<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        content_bounds: Rect,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for (rect, target) in self.routing_discoverability_targets(content_bounds) {
            self.draw_inline_discoverability_badges(canvas, rect, target)?;
        }
        Ok(())
    }

    fn draw_direct_mapping_targets<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        tabs_bounds: Rect,
        content_bounds: Rect,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if self.direct_mapping_state.mode == DirectMappingMode::Inactive {
            return Ok(());
        }

        for page in self.direct_mapping_tab_targets(tabs_bounds) {
            canvas.set_draw_color(Color::RGB(132, 84, 84));
            canvas.draw_rect(page.hit_rect)?;
        }

        for target in self.direct_mapping_targets(content_bounds) {
            canvas.set_draw_color(Color::RGB(176, 116, 72));
            canvas.draw_rect(Rect::new(
                target.hit_rect.x - 1,
                target.hit_rect.y - 1,
                target.hit_rect.width().saturating_add(2),
                target.hit_rect.height().saturating_add(2),
            ))?;
            if self.direct_mapping_state.mode == DirectMappingMode::AwaitingInput(target) {
                canvas.set_draw_color(Color::RGB(252, 146, 126));
                canvas.draw_rect(Rect::new(
                    target.hit_rect.x - 3,
                    target.hit_rect.y - 3,
                    target.hit_rect.width().saturating_add(6),
                    target.hit_rect.height().saturating_add(6),
                ))?;
            }
        }

        Ok(())
    }

    fn draw_inline_discoverability_badges<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        anchor: Rect,
        target: DiscoverabilityTarget,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let summary = self.summarize_discoverability_target(target);
        if summary.badges.is_empty() {
            return Ok(());
        }

        if let Some(slot) = target.overlay_slot {
            return self.draw_compact_discoverability_slot(canvas, slot, &summary);
        }

        let max_badges = if anchor.width() <= 24 || anchor.height() <= 12 {
            1
        } else {
            2
        };
        let badge_height = 10_u32;
        let label_width = if max_badges == 1 { 4 } else { 6 };
        let y = if anchor.height() <= 12 {
            anchor.y - badge_height as i32 - 2
        } else {
            anchor.y + 2
        };
        let x = if anchor.width() >= 44 {
            anchor.x + anchor.width() as i32 - 32
        } else {
            anchor.x + anchor.width() as i32 + 3
        };
        let bounds = Rect::new(x, y, 72, badge_height + 4);
        self.draw_mapping_badges(
            canvas,
            bounds,
            &summary.badges,
            summary.total_bindings,
            max_badges,
            label_width,
        )
    }

    fn draw_compact_discoverability_slot<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        slot: Rect,
        summary: &ActionDiscoverabilitySummary,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let built_in_count = summary.badges.iter().filter(|badge| badge.built_in).count();
        let user_count = summary
            .badges
            .iter()
            .filter(|badge| !badge.built_in)
            .count();

        if built_in_count > 0 && user_count > 0 {
            let left_width = (slot.width() / 2).max(1);
            let right_width = slot.width().saturating_sub(left_width);
            canvas.set_draw_color(Color::RGB(64, 84, 126));
            canvas.fill_rect(Rect::new(slot.x, slot.y, left_width, slot.height()))?;
            canvas.set_draw_color(Color::RGB(88, 128, 76));
            canvas.fill_rect(Rect::new(
                slot.x + left_width as i32,
                slot.y,
                right_width,
                slot.height(),
            ))?;
        } else if user_count > 0 {
            canvas.set_draw_color(Color::RGB(88, 128, 76));
            canvas.fill_rect(slot)?;
        } else {
            canvas.set_draw_color(Color::RGB(64, 84, 126));
            canvas.fill_rect(slot)?;
        }

        let count_text = if summary.total_bindings >= 10 {
            "+".to_string()
        } else {
            summary.total_bindings.to_string()
        };
        crate::ui::draw_text_fitted(
            canvas,
            &count_text,
            Rect::new(
                slot.x + 1,
                slot.y + 1,
                slot.width().saturating_sub(2),
                slot.height().saturating_sub(2),
            ),
            1,
            Color::RGB(244, 244, 236),
        )?;

        Ok(())
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

    fn draw_mappings_page<T: RenderTarget>(
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
            crate::ui::draw_text_fitted(
                canvas,
                if entry.source_kind == MappingSourceKind::Midi {
                    &entry.source_device_label
                } else {
                    "--"
                },
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
                &entry.target_label,
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

        canvas.set_draw_color(Color::RGB(26, 32, 46));
        canvas.fill_rect(footer_bounds)?;
        let footer_tokens = [
            ("Tap row", Color::RGB(62, 78, 106)),
            ("Tap field", Color::RGB(74, 88, 118)),
            ("Tap again act", Color::RGB(82, 100, 136)),
            ("W Write", Color::RGB(96, 82, 52)),
            ("F8 Direct", Color::RGB(128, 78, 78)),
            ("N New", Color::RGB(66, 96, 84)),
            ("Del Remove", Color::RGB(110, 74, 74)),
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
            "Shift+Left/Right Field  Q/E Adjust  Enter Learn/Toggle",
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

    fn draw_midi_io_page<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        content_bounds: Rect,
    ) -> Result<(), Box<dyn std::error::Error>> {
        canvas.set_draw_color(Color::RGB(22, 28, 42));
        canvas.fill_rect(content_bounds)?;
        canvas.set_draw_color(Color::RGB(88, 96, 120));
        canvas.draw_rect(content_bounds)?;

        let (header_bounds, lists_bounds) = crate::ui::split_top_strip(content_bounds, 28, 10)?;
        let columns = crate::ui::equal_columns(lists_bounds, 2, 14);
        let input_bounds = columns[0];
        let output_bounds = columns[1];
        crate::ui::draw_text_fitted(
            canvas,
            "MIDI I/O",
            Rect::new(header_bounds.x + 8, header_bounds.y + 8, 160, 14),
            2,
            Color::RGB(244, 232, 146),
        )?;
        crate::ui::draw_text_fitted(
            canvas,
            "Select default inputs and outputs",
            Rect::new(header_bounds.x + 188, header_bounds.y + 12, 220, 8),
            1,
            Color::RGB(184, 194, 206),
        )?;

        let input_header = Rect::new(input_bounds.x, input_bounds.y, input_bounds.width(), 22);
        let output_header = Rect::new(output_bounds.x, output_bounds.y, output_bounds.width(), 22);
        canvas.set_draw_color(Color::RGB(28, 34, 50));
        canvas.fill_rect(input_header)?;
        canvas.fill_rect(output_header)?;
        canvas.set_draw_color(Color::RGB(88, 96, 120));
        canvas.draw_rect(input_header)?;
        canvas.draw_rect(output_header)?;
        crate::ui::draw_text_fitted(
            canvas,
            "Inputs",
            Rect::new(input_header.x + 8, input_header.y + 7, 96, 8),
            2,
            Color::RGB(214, 242, 220),
        )?;
        crate::ui::draw_text_fitted(
            canvas,
            "Outputs",
            Rect::new(output_header.x + 8, output_header.y + 7, 96, 8),
            2,
            Color::RGB(246, 212, 194),
        )?;

        self.draw_device_list(
            canvas,
            Rect::new(
                input_bounds.x,
                input_header.y + input_header.height() as i32 + 6,
                input_bounds.width(),
                input_bounds
                    .height()
                    .saturating_sub(input_header.height().saturating_add(28)),
            ),
            &self.midi_devices.inputs,
            self.page_state.midi_io.selected_input_index,
            self.midi_devices.selected_input,
            self.page_state.midi_io.focus == MidiIoListFocus::Inputs,
            Color::RGB(78, 196, 164),
            "Input",
        )?;
        self.draw_device_list(
            canvas,
            Rect::new(
                output_bounds.x,
                output_header.y + output_header.height() as i32 + 6,
                output_bounds.width(),
                output_bounds
                    .height()
                    .saturating_sub(output_header.height().saturating_add(28)),
            ),
            &self.midi_devices.outputs,
            self.page_state.midi_io.selected_output_index,
            self.midi_devices.selected_output,
            self.page_state.midi_io.focus == MidiIoListFocus::Outputs,
            Color::RGB(224, 132, 90),
            "Output",
        )?;

        Ok(())
    }

    fn draw_device_list<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        bounds: Rect,
        ports: &[MidiPortRef],
        selected_index: usize,
        active_index: Option<usize>,
        focused: bool,
        accent: Color,
        role_label: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        canvas.set_draw_color(Color::RGB(22, 28, 42));
        canvas.fill_rect(bounds)?;
        canvas.set_draw_color(if focused {
            Color::RGB(242, 232, 150)
        } else {
            Color::RGB(88, 96, 120)
        });
        canvas.draw_rect(bounds)?;

        let rows = crate::ui::stacked_rows(
            crate::ui::inset_rect(bounds, 10, 10)?,
            ports.len().max(1),
            8,
        );
        for (index, row) in rows.into_iter().enumerate().take(ports.len()) {
            let is_selected = index == selected_index;
            let is_active = active_index == Some(index);

            canvas.set_draw_color(if is_selected {
                Color::RGB(56, 70, 100)
            } else {
                Color::RGB(28, 34, 50)
            });
            canvas.fill_rect(row)?;
            canvas.set_draw_color(if is_selected {
                Color::RGB(244, 232, 146)
            } else {
                Color::RGB(70, 80, 102)
            });
            canvas.draw_rect(row)?;

            let status = Rect::new(row.x + 6, row.y + 6, 16, row.height().saturating_sub(12));
            canvas.set_draw_color(if is_active {
                accent
            } else {
                Color::RGB(72, 76, 84)
            });
            canvas.fill_rect(status)?;

            let selected_badge_width = if is_selected { 24 } else { 0 };
            let active_badge_width = if is_active { 24 } else { 0 };
            let reserved_badge_width = selected_badge_width + active_badge_width;
            let header_rect = Rect::new(
                status.x + status.width() as i32 + 8,
                row.y + 8,
                row.width()
                    .saturating_sub(40)
                    .saturating_sub(reserved_badge_width as u32),
                8,
            );
            let body_rect = Rect::new(
                status.x + status.width() as i32 + 8,
                row.y + 20,
                row.width().saturating_sub(40),
                row.height().saturating_sub(28),
            );
            canvas.set_draw_color(if is_selected {
                Color::RGB(216, 224, 238)
            } else {
                Color::RGB(182, 194, 212)
            });
            canvas.fill_rect(body_rect)?;
            crate::ui::draw_text_fitted(
                canvas,
                &ports[index].name,
                header_rect,
                1,
                Color::RGB(230, 236, 244),
            )?;
            if is_active {
                let active_badge = Rect::new(
                    row.x + row.width() as i32 - 12 - active_badge_width - selected_badge_width,
                    row.y + 8,
                    active_badge_width as u32,
                    8,
                );
                canvas.set_draw_color(accent);
                canvas.fill_rect(active_badge)?;
                crate::ui::draw_text_fitted(
                    canvas,
                    if role_label == "Input" { "Def" } else { "Def" },
                    Rect::new(
                        active_badge.x + 3,
                        active_badge.y,
                        active_badge.width().saturating_sub(6),
                        8,
                    ),
                    1,
                    Color::RGB(22, 28, 36),
                )?;
            }
            if is_selected {
                let selected_badge = Rect::new(
                    row.x + row.width() as i32 - 12 - selected_badge_width,
                    row.y + 8,
                    selected_badge_width as u32,
                    8,
                );
                canvas.set_draw_color(Color::RGB(244, 232, 146));
                canvas.fill_rect(selected_badge)?;
                crate::ui::draw_text_fitted(
                    canvas,
                    "Sel",
                    Rect::new(
                        selected_badge.x + 3,
                        selected_badge.y,
                        selected_badge.width().saturating_sub(6),
                        8,
                    ),
                    1,
                    Color::RGB(24, 28, 36),
                )?;
            }
        }

        Ok(())
    }

    fn draw_routing_page<T: RenderTarget>(
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
            "Routing",
            Rect::new(content_bounds.x + 8, content_bounds.y + 8, 140, 14),
            2,
            Color::RGB(244, 232, 146),
        )?;
        crate::ui::draw_text_fitted(
            canvas,
            "Active Track Routing",
            Rect::new(content_bounds.x + 184, content_bounds.y + 12, 180, 8),
            1,
            Color::RGB(184, 194, 206),
        )?;

        let inner = crate::ui::inset_rect(content_bounds, 12, 32)?;
        let (header, body) = crate::ui::split_top_strip(inner, 48, 10)?;
        let active_track = self
            .project
            .active_track()
            .expect("demo project has tracks");

        canvas.set_draw_color(Color::RGB(54, 70, 104));
        canvas.fill_rect(header)?;
        canvas.set_draw_color(Color::RGB(244, 232, 146));
        canvas.draw_rect(header)?;

        let meta_badges = [
            (
                Rect::new(
                    header.x + 8,
                    header.y + 8,
                    90,
                    header.height().saturating_sub(16),
                ),
                Color::RGB(220, 124, 100),
                format!("Active T{}", self.project.active_track_index + 1),
            ),
            (
                Rect::new(
                    header.x + 106,
                    header.y + 8,
                    92,
                    header.height().saturating_sub(16),
                ),
                if active_track.state.passthrough {
                    Color::RGB(72, 188, 180)
                } else {
                    Color::RGB(92, 100, 112)
                },
                format!("Thru {}", on_off(active_track.state.passthrough)),
            ),
        ];
        for (rect, color, label) in meta_badges {
            canvas.set_draw_color(color);
            canvas.fill_rect(rect)?;
            crate::ui::draw_text_fitted(
                canvas,
                &label,
                Rect::new(rect.x + 6, rect.y + 4, rect.width().saturating_sub(12), 8),
                1,
                Color::RGB(24, 28, 36),
            )?;
        }
        let state_badge = Rect::new(
            header.x + header.width() as i32 - 122,
            header.y + 8,
            112,
            header.height().saturating_sub(16),
        );
        canvas.set_draw_color(Color::RGB(70, 86, 118));
        canvas.fill_rect(state_badge)?;
        crate::ui::draw_text_fitted(
            canvas,
            "Tap value",
            Rect::new(
                state_badge.x + 6,
                state_badge.y + 4,
                state_badge.width().saturating_sub(12),
                8,
            ),
            1,
            Color::RGB(244, 244, 236),
        )?;
        crate::ui::draw_text_fitted(
            canvas,
            &active_track.name,
            Rect::new(
                header.x + 208,
                header.y + 8,
                (state_badge.x - header.x - 220).max(0) as u32,
                8,
            ),
            1,
            Color::RGB(244, 244, 236),
        )?;
        crate::ui::draw_text_fitted(
            canvas,
            "Input and output routing for the active track",
            Rect::new(
                header.x + 208,
                header.y + 24,
                (state_badge.x - header.x - 220).max(0) as u32,
                8,
            ),
            1,
            Color::RGB(208, 216, 228),
        )?;

        let rows = crate::ui::stacked_rows(body, RoutingField::ALL.len(), 10);
        for (index, field) in RoutingField::ALL.iter().copied().enumerate() {
            let row = rows[index];
            let selected = field == self.page_state.selected_routing_field;

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

            let label = Rect::new(row.x + 8, row.y + 8, 18, row.height().saturating_sub(16));
            canvas.set_draw_color(Color::RGB(110, 120, 140));
            canvas.fill_rect(label)?;

            let value_color = match field {
                RoutingField::InputDevice => Color::RGB(94, 186, 152),
                RoutingField::InputChannel => Color::RGB(106, 152, 218),
                RoutingField::OutputDevice => Color::RGB(218, 142, 98),
                RoutingField::OutputChannel => Color::RGB(208, 122, 160),
                RoutingField::Passthrough => {
                    if active_track.state.passthrough {
                        Color::RGB(92, 220, 216)
                    } else {
                        Color::RGB(112, 118, 126)
                    }
                }
            };
            let label_text_rect = Rect::new(row.x + 34, row.y + 8, 112, 8);
            let value = Rect::new(
                row.x + 156,
                row.y + 8,
                row.width().saturating_sub(220),
                row.height().saturating_sub(16),
            );
            let affordance = Rect::new(
                row.x + row.width() as i32 - 72,
                row.y + 8,
                62,
                row.height().saturating_sub(16),
            );
            let left_adjust = Rect::new(
                value.x + 4,
                value.y + 4,
                20,
                value.height().saturating_sub(8),
            );
            let right_adjust = Rect::new(
                value.x + value.width() as i32 - 24,
                value.y + 4,
                20,
                value.height().saturating_sub(8),
            );
            canvas.set_draw_color(value_color);
            canvas.fill_rect(value)?;
            if field != RoutingField::Passthrough {
                canvas.set_draw_color(Color::RGB(34, 42, 56));
                canvas.fill_rect(left_adjust)?;
                canvas.fill_rect(right_adjust)?;
            }
            canvas.set_draw_color(if selected {
                Color::RGB(244, 232, 146)
            } else {
                Color::RGB(96, 104, 122)
            });
            canvas.fill_rect(affordance)?;
            canvas.set_draw_color(if selected {
                Color::RGB(252, 244, 178)
            } else {
                Color::RGB(124, 132, 146)
            });
            canvas.draw_rect(affordance)?;
            crate::ui::draw_text_fitted(
                canvas,
                field.label(),
                label_text_rect,
                1,
                Color::RGB(244, 244, 236),
            )?;
            if field == RoutingField::Passthrough {
                let bool_chip = Rect::new(
                    value.x + 6,
                    value.y + 2,
                    54,
                    value.height().saturating_sub(4),
                );
                canvas.set_draw_color(if active_track.state.passthrough {
                    Color::RGB(52, 156, 150)
                } else {
                    Color::RGB(88, 94, 102)
                });
                canvas.fill_rect(bool_chip)?;
                crate::ui::draw_text_fitted(
                    canvas,
                    &self.routing_field_value(active_track, field),
                    Rect::new(
                        bool_chip.x + 6,
                        bool_chip.y + 3,
                        bool_chip.width().saturating_sub(12),
                        8,
                    ),
                    1,
                    Color::RGB(244, 244, 236),
                )?;
            } else {
                crate::ui::draw_text_fitted(
                    canvas,
                    "-",
                    Rect::new(
                        left_adjust.x + 7,
                        left_adjust.y + 3,
                        left_adjust.width().saturating_sub(14),
                        8,
                    ),
                    1,
                    Color::RGB(222, 228, 236),
                )?;
                crate::ui::draw_text_fitted(
                    canvas,
                    "+",
                    Rect::new(
                        right_adjust.x + 7,
                        right_adjust.y + 3,
                        right_adjust.width().saturating_sub(14),
                        8,
                    ),
                    1,
                    Color::RGB(222, 228, 236),
                )?;
                crate::ui::draw_text_fitted(
                    canvas,
                    &self.routing_field_value(active_track, field),
                    Rect::new(
                        value.x + 30,
                        value.y + 6,
                        value.width().saturating_sub(60),
                        8,
                    ),
                    1,
                    Color::RGB(24, 28, 36),
                )?;
            }
            crate::ui::draw_text_fitted(
                canvas,
                if field == RoutingField::Passthrough {
                    "Toggle"
                } else if selected {
                    "Tap +/-"
                } else {
                    "Select"
                },
                Rect::new(
                    affordance.x + 6,
                    affordance.y + 4,
                    affordance.width().saturating_sub(12),
                    8,
                ),
                1,
                Color::RGB(24, 28, 36),
            )?;
        }

        if self.overlay_state.active == Some(AppOverlay::Discoverability) {
            self.draw_routing_discoverability_overlay(canvas, content_bounds)?;
        }

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
                    "trekr | Page:{} (Tab/F1-F4) | Mode:{} | F5 Overlay:{} | F7 Discover:{} | F8 Direct:{} | W Toggle Mode | N New | Del Remove | Shift+Left/Right Field:{} | Learn:{} | Up/Down Select | Source:{} {} | Device:{} | Target:{} | Scope:{} | Enabled:{}",
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
                "trekr | Page:{} (Tab/F1-F4) | T{} {} | F8 Direct:{} | Up/Down Field | Q/E Adjust | Enter Toggle | Field:{} | In:{} {} | Out:{} {} | Thru:{}",
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
            ),
        };
        window.set_title(&title)?;
        Ok(())
    }

    fn toggle_direct_mapping_mode(&mut self) {
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
        self.direct_mapping_state.mode = DirectMappingMode::Inactive;
        self.direct_mapping_state.origin = DirectMappingOrigin::InPlace;
        self.direct_mapping_state.status_message = Some(message.to_string());
        self.sync_midi_inputs();
    }

    fn apply_action(&mut self, action: AppAction) -> AppControl {
        match action {
            AppAction::Quit => AppControl::Quit,
            AppAction::ShowPage(page) => {
                self.page_state.current_page = page;
                self.sync_midi_inputs();
                AppControl::Continue
            }
            AppAction::ShowNextPage => {
                self.page_state.current_page = self.page_state.current_page.next();
                self.sync_midi_inputs();
                AppControl::Continue
            }
            AppAction::ShowPreviousPage => {
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
                    self.silence_all_tracks();
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
                AppControl::Continue
            }
            AppAction::ResetGlobalLoop => {
                self.project.loop_region = self.project.full_song_range();
                self.project.transport.loop_enabled = true;
                self.playhead_ticks = self.playhead_ticks.clamp(
                    self.project.loop_region.start_ticks,
                    self.project.loop_region.end_ticks(),
                );
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
                AppControl::Continue
            }
            AppAction::SetCurrentTrackLoopStart => {
                let edit_ticks = self.current_edit_ticks();
                if let Some(track) = self.project.active_track_mut() {
                    track.loop_region.set_start_preserving_end(edit_ticks);
                }
                AppControl::Continue
            }
            AppAction::SetCurrentTrackLoopEnd => {
                let edit_ticks = self.current_edit_ticks();
                if let Some(track) = self.project.active_track_mut() {
                    track.loop_region.set_end(edit_ticks);
                }
                AppControl::Continue
            }
            AppAction::SetGlobalLoopStart => {
                let edit_ticks = self.current_edit_ticks();
                self.project
                    .loop_region
                    .set_start_preserving_end(edit_ticks);
                AppControl::Continue
            }
            AppAction::SetGlobalLoopEnd => {
                let edit_ticks = self.current_edit_ticks();
                self.project.loop_region.set_end(edit_ticks);
                AppControl::Continue
            }
            AppAction::NudgeCurrentTrackLoopBackward => {
                let delta = -(self.nudge_step_ticks() as i64);
                if let Some(track) = self.project.active_track_mut() {
                    track.loop_region.shift_by(delta);
                }
                AppControl::Continue
            }
            AppAction::NudgeCurrentTrackLoopForward => {
                let delta = self.nudge_step_ticks() as i64;
                if let Some(track) = self.project.active_track_mut() {
                    track.loop_region.shift_by(delta);
                }
                AppControl::Continue
            }
            AppAction::NudgeGlobalLoopBackward => {
                let delta = -(self.nudge_step_ticks() as i64);
                self.project.loop_region.shift_by(delta);
                AppControl::Continue
            }
            AppAction::NudgeGlobalLoopForward => {
                let delta = self.nudge_step_ticks() as i64;
                self.project.loop_region.shift_by(delta);
                AppControl::Continue
            }
            AppAction::ShortenCurrentTrackLoop => {
                let step = self.nudge_step_ticks();
                if let Some(track) = self.project.active_track_mut() {
                    track.loop_region.shorten_by(step);
                }
                AppControl::Continue
            }
            AppAction::ExtendCurrentTrackLoop => {
                let step = self.nudge_step_ticks();
                if let Some(track) = self.project.active_track_mut() {
                    track.loop_region.extend_by(step);
                }
                AppControl::Continue
            }
            AppAction::HalfCurrentTrackLoop => {
                if let Some(track) = self.project.active_track_mut() {
                    track.loop_region.half_length();
                }
                AppControl::Continue
            }
            AppAction::DoubleCurrentTrackLoop => {
                if let Some(track) = self.project.active_track_mut() {
                    track.loop_region.double_length();
                }
                AppControl::Continue
            }
            AppAction::ShortenGlobalLoop => {
                let step = self.nudge_step_ticks();
                self.project.loop_region.shorten_by(step);
                AppControl::Continue
            }
            AppAction::ExtendGlobalLoop => {
                let step = self.nudge_step_ticks();
                self.project.loop_region.extend_by(step);
                AppControl::Continue
            }
            AppAction::HalfGlobalLoop => {
                self.project.loop_region.half_length();
                AppControl::Continue
            }
            AppAction::DoubleGlobalLoop => {
                self.project.loop_region.double_length();
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
            AppAction::SelectNextTrack => {
                self.project.select_next_track();
                AppControl::Continue
            }
            AppAction::SelectPreviousTrack => {
                self.project.select_previous_track();
                AppControl::Continue
            }
            AppAction::SelectTrack(index) => {
                self.project.select_track(index);
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
            self.advance_linked_playhead();
            return;
        }

        if !self.project.transport.playing {
            return;
        }

        let previous_ticks = self.transport_ticks;
        let ticks_per_second = self.project.transport.ticks_per_second();
        let advanced_ticks =
            (delta.as_nanos() as u128 * u128::from(ticks_per_second)) / 1_000_000_000_u128;
        self.transport_ticks = self.transport_ticks.saturating_add(advanced_ticks as u64);
        self.playhead_ticks = self.transport_ticks;

        if self.project.transport.loop_enabled {
            let loop_region = self.project.loop_region;
            if loop_region.length_ticks > 0 {
                let relative = self.transport_ticks.saturating_sub(loop_region.start_ticks);
                self.playhead_ticks =
                    loop_region.start_ticks + (relative % loop_region.length_ticks.max(1));
            }
        }

        self.dispatch_midi_notes(previous_ticks, advanced_ticks as u64);
    }

    fn advance_linked_playhead(&mut self) {
        self.link_snapshot = self.link.refresh();
        self.project.transport.tempo_bpm =
            self.link_snapshot.tempo_bpm.round().clamp(20.0, 400.0) as u16;
        if self.project.transport.link_start_stop_sync {
            self.project.transport.playing = self.link_snapshot.is_playing;
        }
        if !self.project.transport.playing {
            return;
        }

        let previous_ticks = self.transport_ticks;
        let linked_ticks = (self.link_snapshot.beat.max(0.0)
            * f64::from(self.project.transport.ppqn.max(1)))
        .round() as u64;
        self.transport_ticks = linked_ticks;
        self.playhead_ticks = linked_ticks;

        if self.project.transport.loop_enabled {
            let loop_region = self.project.loop_region;
            if loop_region.length_ticks > 0 {
                let relative = self.transport_ticks.saturating_sub(loop_region.start_ticks);
                self.playhead_ticks =
                    loop_region.start_ticks + (relative % loop_region.length_ticks.max(1));
            }
        }

        if linked_ticks < previous_ticks {
            self.silence_all_tracks();
            return;
        }
        self.dispatch_midi_notes(previous_ticks, linked_ticks.saturating_sub(previous_ticks));
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

    fn effective_track_playhead(&self, track: &Track) -> u64 {
        let raw = self.playhead_ticks;
        if !track.state.loop_enabled || track.loop_region.length_ticks == 0 {
            return raw;
        }

        track.loop_region.start_ticks + (raw % track.loop_region.length_ticks)
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

    fn record_context(&self, track: &Track) -> Option<crate::project::RecordContext> {
        if track.state.loop_enabled {
            Some(crate::project::RecordContext {
                range: track.loop_region,
                wrap_basis_ticks: track.loop_region.start_ticks,
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
        }
        self.sync_midi_inputs();
    }

    fn select_previous_page_item(&mut self) {
        match self.page_state.current_page {
            AppPage::Timeline => self.project.select_previous_track(),
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
        match self.page_state.current_page {
            AppPage::Timeline => self.project.select_next_track(),
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
        if self.page_state.current_page == AppPage::Mappings
            && self.page_state.mapping_mode == MappingPageMode::Write
        {
            self.page_state.selected_mapping_field =
                self.previous_enabled_mapping_field(self.page_state.selected_mapping_field);
            self.page_state.mapping_midi_learn_armed = false;
        }
    }

    fn select_next_page_field(&mut self) {
        if self.page_state.current_page == AppPage::Mappings
            && self.page_state.mapping_mode == MappingPageMode::Write
        {
            self.page_state.selected_mapping_field =
                self.next_enabled_mapping_field(self.page_state.selected_mapping_field);
            self.page_state.mapping_midi_learn_armed = false;
        }
    }

    fn adjust_page_item(&mut self, delta: i32) {
        match self.page_state.current_page {
            AppPage::Timeline => {}
            AppPage::Mappings => {
                if self.page_state.mapping_mode == MappingPageMode::Write
                    && !self.mappings.is_empty()
                {
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
        match self.page_state.current_page {
            AppPage::Timeline => {}
            AppPage::Mappings => {
                if self.page_state.mapping_mode == MappingPageMode::Write
                    && !self.mappings.is_empty()
                {
                    self.activate_mapping_field();
                }
            }
            AppPage::MidiIo => match self.page_state.midi_io.focus {
                MidiIoListFocus::Inputs => {
                    self.midi_devices
                        .set_selected_input(self.page_state.midi_io.selected_input_index);
                    self.sync_midi_inputs();
                }
                MidiIoListFocus::Outputs => self
                    .midi_devices
                    .set_selected_output(self.page_state.midi_io.selected_output_index),
            },
            AppPage::Routing => {
                if self.page_state.selected_routing_field == RoutingField::Passthrough {
                    if let Some(track) = self.project.active_track_mut() {
                        track.state.passthrough = !track.state.passthrough;
                    }
                }
            }
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
                entry.scope_label = default_scope_label(&entry.target_label, track_count);
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

    fn activate_mapping_field(&mut self) {
        let index = self.page_state.selected_mapping_index;
        let field = self.page_state.selected_mapping_field;
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
            MappingField::Target => {
                entry.target_label = cycle_mapping_target_label(&entry.target_label, 1).to_string();
                entry.scope_label = default_scope_label(&entry.target_label, track_count);
                self.page_state.mapping_midi_learn_armed = false;
            }
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

    fn adjust_routing_field(&mut self, delta: i32) {
        let current_input = self.midi_devices.selected_input_port().cloned();
        let current_output = self.midi_devices.selected_output_port().cloned();
        if let Some(track) = self.project.active_track_mut() {
            match self.page_state.selected_routing_field {
                RoutingField::InputDevice => {
                    track.routing.input_port = cycle_optional_port(
                        track.routing.input_port.as_ref(),
                        &self.midi_devices.inputs,
                        delta,
                    );
                    self.sync_midi_inputs();
                }
                RoutingField::InputChannel => {
                    track.routing.input_channel =
                        cycle_input_channel(track.routing.input_channel, delta);
                }
                RoutingField::OutputDevice => {
                    track.routing.output_port = cycle_optional_port(
                        track.routing.output_port.as_ref(),
                        &self.midi_devices.outputs,
                        delta,
                    );
                }
                RoutingField::OutputChannel => {
                    track.routing.output_channel =
                        cycle_output_channel(track.routing.output_channel, delta);
                }
                RoutingField::Passthrough => {
                    track.state.passthrough = !track.state.passthrough;
                    if track.routing.input_port.is_none() {
                        track.routing.input_port = current_input;
                    }
                    if track.routing.output_port.is_none() {
                        track.routing.output_port = current_output;
                    }
                    self.sync_midi_inputs();
                }
            }
        }
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
                .map(|track| self.record_capture_ticks(track))
                .unwrap_or(self.playhead_ticks);

            if let Some(track) = self.project.tracks.get_mut(index) {
                match event.message {
                    MidiInputMessage::NoteOn { pitch, velocity } => {
                        if track.active_take.is_some() {
                            track.record_note_on(pitch, velocity, input_ticks);
                        }
                        if track.state.passthrough {
                            if let (Some(port), Some(channel)) = (
                                track.routing.output_port.as_ref(),
                                track.routing.output_channel,
                            ) {
                                let _ = self.midi_output.send_note_on(
                                    port,
                                    channel.clamp(1, 16),
                                    pitch,
                                    velocity,
                                );
                            }
                        }
                    }
                    MidiInputMessage::NoteOff { pitch } => {
                        if track.active_take.is_some() {
                            track.record_note_off(pitch, input_ticks);
                        }
                        if track.state.passthrough {
                            if let (Some(port), Some(channel)) = (
                                track.routing.output_port.as_ref(),
                                track.routing.output_channel,
                            ) {
                                let _ = self.midi_output.send_note_off(
                                    port,
                                    channel.clamp(1, 16),
                                    pitch,
                                );
                            }
                        }
                    }
                    MidiInputMessage::ControlChange { .. } => {}
                }
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
        self.direct_mapping_state.mode = DirectMappingMode::Inactive;
        self.direct_mapping_state.origin = DirectMappingOrigin::InPlace;
        self.direct_mapping_state.status_message = Some(if same_target {
            format!(
                "Updated {} ({}) to {}.",
                target.target_label, target.scope_label, source_label
            )
        } else {
            format!(
                "Mapped {} ({}) to {}.",
                target.target_label, target.scope_label, source_label
            )
        });
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

        let global_loop = self
            .project
            .transport
            .loop_enabled
            .then_some(self.project.loop_region);
        let track_events: Vec<(Option<MidiPortRef>, u8, Vec<(u64, bool, u8, u8)>)> = self
            .project
            .tracks
            .iter()
            .map(|track| {
                let channel = track.routing.output_channel.unwrap_or(1).clamp(1, 16);
                let port = track.routing.output_port.clone();
                let loop_range = if track.state.loop_enabled {
                    Some(track.loop_region)
                } else {
                    global_loop
                };
                let events =
                    scheduled_note_events(track, previous_ticks, advanced_ticks, loop_range);
                (port, channel, events)
            })
            .collect();

        for (port, channel, events) in track_events {
            let Some(port) = port else {
                continue;
            };

            for (_, note_on, pitch, velocity) in events {
                let _ = if note_on {
                    self.midi_output
                        .send_note_on(&port, channel, pitch, velocity)
                } else {
                    self.midi_output.send_note_off(&port, channel, pitch)
                };
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
            let _ = self.midi_output.send_all_notes_off(&port, channel);
        }
    }

    fn routing_field_value(&self, track: &Track, field: RoutingField) -> String {
        match field {
            RoutingField::InputDevice => port_name(track.routing.input_port.as_ref()).to_string(),
            RoutingField::InputChannel => input_channel_label(track.routing.input_channel),
            RoutingField::OutputDevice => port_name(track.routing.output_port.as_ref()).to_string(),
            RoutingField::OutputChannel => output_channel_label(track.routing.output_channel),
            RoutingField::Passthrough => on_off(track.state.passthrough).to_string(),
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
        if matches!(
            event,
            sdl3::event::Event::KeyDown {
                keycode: Some(sdl3::keyboard::Keycode::Escape),
                repeat: false,
                ..
            }
        ) && self.direct_mapping_state.mode != DirectMappingMode::Inactive
        {
            self.cancel_direct_mapping("Canceled direct mapping.");
            return Some(AppControl::Continue);
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
        let (tabs_bounds, content_bounds) = crate::ui::split_top_strip(inset, 28, 12).ok()?;

        if let Some(control) =
            self.handle_direct_mapping_pointer_down(tabs_bounds, content_bounds, x, y, source)
        {
            return Some(control);
        }

        if let Some(page) = self.hit_page_tab(tabs_bounds, x, y) {
            return Some(self.apply_action_with_source(AppAction::ShowPage(page), source));
        }

        match self.page_state.current_page {
            AppPage::Timeline => self.handle_timeline_pointer(content_bounds, x, y, source),
            AppPage::Mappings => self.handle_mappings_pointer(content_bounds, x, y, source),
            AppPage::MidiIo => self.handle_midi_io_pointer(content_bounds, x, y, source),
            AppPage::Routing => self.handle_routing_pointer(content_bounds, x, y, source),
        }
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
        let raw_targets = match self.page_state.current_page {
            AppPage::Timeline => self.timeline_discoverability_targets(content_bounds),
            AppPage::Routing => self.routing_discoverability_targets(content_bounds),
            AppPage::Mappings | AppPage::MidiIo => Vec::new(),
        };

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

    fn handle_timeline_pointer(
        &mut self,
        content_bounds: Rect,
        x: i32,
        y: i32,
        source: crate::actions::ActionSource,
    ) -> Option<AppControl> {
        let (header_bounds, body_bounds) =
            crate::ui::split_top_strip(content_bounds, 28, 6).ok()?;
        let (transport_bounds, timeline_bounds) =
            crate::ui::split_top_strip(body_bounds, 24, 8).ok()?;
        if rect_contains(self.global_loop_reset_button_rect(header_bounds), x, y) {
            return Some(self.apply_action_with_source(AppAction::ResetGlobalLoop, source));
        }

        for (rect, action) in self.transport_chip_actions(transport_bounds) {
            if rect_contains(rect, x, y) {
                return Some(self.apply_action_with_source(action, source));
            }
        }

        let columns = crate::ui::track_column_pairs(timeline_bounds, self.project.tracks.len());
        for (index, (full_bounds, detail_bounds)) in columns.into_iter().enumerate() {
            let status_rect = crate::ui::track_status_rect(
                crate::ui::union_rect(full_bounds, detail_bounds),
                self.timeline_flow,
            );
            for indicator in crate::ui::track_indicators(status_rect) {
                if !rect_contains(indicator.rect, x, y) {
                    continue;
                }

                self.project.active_track_index = index;
                let target = track_indicator_target(indicator.kind, Some(indicator.rect))?;
                return Some(self.apply_action_with_source(target.action, source));
            }
        }

        None
    }

    fn handle_mappings_pointer(
        &mut self,
        content_bounds: Rect,
        x: i32,
        y: i32,
        source: crate::actions::ActionSource,
    ) -> Option<AppControl> {
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

    fn handle_midi_io_pointer(
        &mut self,
        content_bounds: Rect,
        x: i32,
        y: i32,
        _source: crate::actions::ActionSource,
    ) -> Option<AppControl> {
        let (_, lists_bounds) = crate::ui::split_top_strip(content_bounds, 28, 10).ok()?;
        let columns = crate::ui::equal_columns(lists_bounds, 2, 14);
        let input_bounds = columns[0];
        let output_bounds = columns[1];
        let input_header = Rect::new(input_bounds.x, input_bounds.y, input_bounds.width(), 22);
        let output_header = Rect::new(output_bounds.x, output_bounds.y, output_bounds.width(), 22);

        if rect_contains(input_header, x, y) {
            self.page_state.midi_io.focus = MidiIoListFocus::Inputs;
            return Some(AppControl::Continue);
        }
        if rect_contains(output_header, x, y) {
            self.page_state.midi_io.focus = MidiIoListFocus::Outputs;
            return Some(AppControl::Continue);
        }

        let input_list = Rect::new(
            input_bounds.x,
            input_header.y + input_header.height() as i32 + 6,
            input_bounds.width(),
            input_bounds
                .height()
                .saturating_sub(input_header.height().saturating_add(28)),
        );
        let output_list = Rect::new(
            output_bounds.x,
            output_header.y + output_header.height() as i32 + 6,
            output_bounds.width(),
            output_bounds
                .height()
                .saturating_sub(output_header.height().saturating_add(28)),
        );

        if let Some(index) =
            self.hit_device_list_row(input_list, self.midi_devices.inputs.len(), x, y)
        {
            self.page_state.midi_io.focus = MidiIoListFocus::Inputs;
            self.page_state.midi_io.selected_input_index = index;
            self.midi_devices.set_selected_input(index);
            self.sync_midi_inputs();
            return Some(AppControl::Continue);
        }

        if let Some(index) =
            self.hit_device_list_row(output_list, self.midi_devices.outputs.len(), x, y)
        {
            self.page_state.midi_io.focus = MidiIoListFocus::Outputs;
            self.page_state.midi_io.selected_output_index = index;
            self.midi_devices.set_selected_output(index);
            return Some(AppControl::Continue);
        }

        None
    }

    fn handle_routing_pointer(
        &mut self,
        content_bounds: Rect,
        x: i32,
        y: i32,
        _source: crate::actions::ActionSource,
    ) -> Option<AppControl> {
        let inner = crate::ui::inset_rect(content_bounds, 12, 32).ok()?;
        let (header, body) = crate::ui::split_top_strip(inner, 48, 10).ok()?;

        let meta_active = Rect::new(
            header.x + 8,
            header.y + 8,
            90,
            header.height().saturating_sub(16),
        );
        let meta_thru = Rect::new(
            header.x + 106,
            header.y + 8,
            92,
            header.height().saturating_sub(16),
        );
        if rect_contains(meta_active, x, y) {
            self.project.select_next_track();
            return Some(AppControl::Continue);
        }
        if rect_contains(meta_thru, x, y) {
            self.page_state.selected_routing_field = RoutingField::Passthrough;
            self.activate_page_item();
            return Some(AppControl::Continue);
        }

        let rows = crate::ui::stacked_rows(body, RoutingField::ALL.len(), 10);
        for (index, field) in RoutingField::ALL.iter().copied().enumerate() {
            let row = rows[index];
            if !rect_contains(row, x, y) {
                continue;
            }
            self.page_state.selected_routing_field = field;
            if field == RoutingField::Passthrough {
                self.activate_page_item();
                return Some(AppControl::Continue);
            }
            let value = Rect::new(
                row.x + 156,
                row.y + 8,
                row.width().saturating_sub(220),
                row.height().saturating_sub(16),
            );
            let affordance = Rect::new(
                row.x + row.width() as i32 - 72,
                row.y + 8,
                62,
                row.height().saturating_sub(16),
            );
            if rect_contains(value, x, y) {
                let delta = if x < value.x + value.width() as i32 / 2 {
                    -1
                } else {
                    1
                };
                self.adjust_routing_field(delta);
            } else if rect_contains(affordance, x, y) {
                self.adjust_routing_field(1);
            }
            return Some(AppControl::Continue);
        }

        None
    }

    fn hit_page_tab(&self, bounds: Rect, x: i32, y: i32) -> Option<AppPage> {
        let tabs = crate::ui::equal_columns(bounds, AppPage::ALL.len(), 10);
        AppPage::ALL
            .iter()
            .copied()
            .zip(tabs)
            .find_map(|(page, rect)| rect_contains(rect, x, y).then_some(page))
    }

    fn hit_device_list_row(&self, bounds: Rect, count: usize, x: i32, y: i32) -> Option<usize> {
        let rows =
            crate::ui::stacked_rows(crate::ui::inset_rect(bounds, 10, 10).ok()?, count.max(1), 8);
        rows.into_iter()
            .enumerate()
            .take(count)
            .find_map(|(index, rect)| rect_contains(rect, x, y).then_some(index))
    }

    fn discoverability_target_at(&self, x: i32, y: i32) -> Option<DiscoverabilityTarget> {
        if self.overlay_state.active == Some(AppOverlay::MappingsQuickView) {
            return None;
        }
        let surface = crate::ui::surface_rect(self.viewport_size.0, self.viewport_size.1);
        let inset = crate::ui::inset_rect(surface, 24, 24).ok()?;
        let (_, page_area_bounds) = crate::ui::split_top_strip(inset, 28, 12).ok()?;
        let footer_height = 22_u32;
        let footer_gap = 8_i32;
        let content_bounds = Rect::new(
            page_area_bounds.x,
            page_area_bounds.y,
            page_area_bounds.width(),
            page_area_bounds
                .height()
                .saturating_sub(footer_height)
                .saturating_sub(footer_gap as u32),
        );

        let targets = match self.page_state.current_page {
            AppPage::Timeline => self.timeline_discoverability_targets(content_bounds),
            AppPage::Routing => self.routing_discoverability_targets(content_bounds),
            AppPage::Mappings | AppPage::MidiIo => Vec::new(),
        };

        targets
            .into_iter()
            .find_map(|(rect, target)| rect_contains(rect, x, y).then_some(target))
    }

    fn timeline_discoverability_targets(
        &self,
        content_bounds: Rect,
    ) -> Vec<(Rect, DiscoverabilityTarget)> {
        let mut targets = Vec::new();
        let (header_bounds, body_bounds) =
            crate::ui::split_top_strip(content_bounds, 28, 6).expect("timeline layout");
        let (transport_bounds, timeline_bounds) =
            crate::ui::split_top_strip(body_bounds, 24, 8).expect("timeline transport");
        targets.push((
            self.global_loop_reset_button_rect(header_bounds),
            DiscoverabilityTarget {
                action: AppAction::ResetGlobalLoop,
                display_scope: Some("Global"),
                allowed_mapping_scopes: &["Global"],
                overlay_slot: None,
            },
        ));
        for (rect, action) in self.transport_chip_actions(transport_bounds) {
            let display_scope = if action == AppAction::ToggleRecording {
                Some("Armed/Active")
            } else {
                Some("Global")
            };
            let allowed_mapping_scopes: &'static [&'static str] =
                if action == AppAction::ToggleRecording {
                    &["Armed/Active", "Active Track"]
                } else {
                    &["Global"]
                };
            targets.push((
                rect,
                DiscoverabilityTarget {
                    action,
                    display_scope,
                    allowed_mapping_scopes,
                    overlay_slot: None,
                },
            ));
        }

        let columns = crate::ui::track_column_pairs(timeline_bounds, self.project.tracks.len());
        for (full_bounds, detail_bounds) in columns {
            targets.extend(self.track_discoverability_targets(full_bounds, detail_bounds));
        }

        targets
    }

    fn track_discoverability_targets(
        &self,
        full_bounds: Rect,
        detail_bounds: Rect,
    ) -> Vec<(Rect, DiscoverabilityTarget)> {
        let mut targets = Vec::new();
        let status_rect = crate::ui::track_status_rect(
            crate::ui::union_rect(full_bounds, detail_bounds),
            self.timeline_flow,
        );
        let label_rect = crate::ui::track_label_rect(full_bounds, self.timeline_flow);
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

        let passthrough_hit = Rect::new(full_bounds.x, label_rect.y, 14, label_rect.height());
        targets.push((
            passthrough_hit,
            DiscoverabilityTarget {
                action: AppAction::ToggleCurrentTrackPassthrough,
                display_scope: Some("Active Track"),
                allowed_mapping_scopes: &["Active Track"],
                overlay_slot: None,
            },
        ));

        let detail_label_rect = crate::ui::track_label_rect(detail_bounds, self.timeline_flow);
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

    fn routing_discoverability_targets(
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

        let rows = crate::ui::stacked_rows(body, RoutingField::ALL.len(), 10);
        for (index, field) in RoutingField::ALL.iter().copied().enumerate() {
            if field != RoutingField::Passthrough {
                continue;
            }
            let row = rows[index];
            let value = Rect::new(
                row.x + 156,
                row.y + 8,
                row.width().saturating_sub(220),
                row.height().saturating_sub(16),
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

    fn global_loop_reset_button_rect(&self, header_bounds: Rect) -> Rect {
        let width = crate::ui::text_width("Reset Song Loop", 1) + 18;
        Rect::new(
            header_bounds.x + header_bounds.width() as i32 - width as i32 - 8,
            header_bounds.y + 4,
            width,
            header_bounds.height().saturating_sub(8),
        )
    }

    fn transport_chip_specs(&self) -> Vec<TransportChipSpec> {
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
                label: format!("Rec {}", on_off(self.project.transport.recording)),
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
            TransportChipSpec {
                label: format!(
                    "RecWrap {}",
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
                label: format!("SongLoop {}", on_off(self.project.transport.loop_enabled)),
                action: Some(AppAction::ToggleGlobalLoop),
                fill: Color::RGB(116, 96, 54),
            },
            TransportChipSpec {
                label: format!("Tempo {}", self.project.transport.tempo_bpm),
                action: None,
                fill: Color::RGB(70, 100, 120),
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

    fn transport_chip_actions(&self, bounds: Rect) -> Vec<(Rect, AppAction)> {
        let mut cursor_x = bounds.x + 6;
        let mut rects = Vec::new();
        for chip_spec in self.transport_chip_specs() {
            let width = crate::ui::text_width(&chip_spec.label, 1) + 12;
            let chip = Rect::new(
                cursor_x,
                bounds.y + 4,
                width,
                bounds.height().saturating_sub(8),
            );
            if let Some(action) = chip_spec.action {
                rects.push((chip, action));
            }
            cursor_x += chip.width() as i32 + 6;
        }

        let divider = Rect::new(
            cursor_x + 4,
            bounds.y + 4,
            1,
            bounds.height().saturating_sub(8),
        );
        cursor_x = divider.x + 8 + 28;
        let sync_badges = [
            (
                format!("Link {}", on_off(self.project.transport.link_enabled)),
                AppAction::ToggleLinkEnabled,
            ),
            (
                format!(
                    "Sync {}",
                    on_off(self.project.transport.link_start_stop_sync)
                ),
                AppAction::ToggleLinkStartStopSync,
            ),
        ];
        for (label, action) in sync_badges {
            let width = crate::ui::text_width(&label, 1) + 12;
            let chip = Rect::new(
                cursor_x,
                bounds.y + 4,
                width,
                bounds.height().saturating_sub(8),
            );
            rects.push((chip, action));
            cursor_x += chip.width() as i32 + 6;
        }

        rects
    }
}

#[derive(Debug, Clone, Copy)]
struct CaptureSpec {
    page: AppPage,
    overlay: Option<AppOverlay>,
    filename: &'static str,
}

fn capture_specs() -> [CaptureSpec; 5] {
    [
        CaptureSpec {
            page: AppPage::Timeline,
            overlay: None,
            filename: "timeline.png",
        },
        CaptureSpec {
            page: AppPage::Mappings,
            overlay: None,
            filename: "mappings.png",
        },
        CaptureSpec {
            page: AppPage::Mappings,
            overlay: Some(AppOverlay::MappingsQuickView),
            filename: "mappings-overlay.png",
        },
        CaptureSpec {
            page: AppPage::MidiIo,
            overlay: None,
            filename: "midi-io.png",
        },
        CaptureSpec {
            page: AppPage::Routing,
            overlay: None,
            filename: "routing.png",
        },
    ]
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

fn scheduled_note_events(
    track: &Track,
    previous_ticks: u64,
    advanced_ticks: u64,
    loop_range: Option<crate::timeline::LoopRegion>,
) -> Vec<(u64, bool, u8, u8)> {
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

    let mut events = Vec::new();
    for (segment_start, segment_end) in segments {
        for note in &track.midi_notes {
            if note.start_ticks >= segment_start && note.start_ticks < segment_end {
                events.push((note.start_ticks, true, note.pitch, note.velocity));
            }
            if note.end_ticks() >= segment_start && note.end_ticks() < segment_end {
                events.push((note.end_ticks(), false, note.pitch, note.velocity));
            }
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

fn mapping_source_label(source: MappingSourceKind) -> &'static str {
    match source {
        MappingSourceKind::Key => "Key",
        MappingSourceKind::Midi => "MIDI",
        MappingSourceKind::Osc => "OSC",
    }
}

fn compact_scope_label(scope: &str) -> &str {
    match scope {
        "Active Track" => "Act Track",
        "Armed/Active" => "Armed/Act",
        "Global" => "Global",
        "Relative" => "Relative",
        "Absolute" => "Absolute",
        other => other,
    }
}

fn mapping_target_label_for_action(action: AppAction) -> Option<&'static str> {
    match action {
        AppAction::TogglePlayback => Some("Play/Stop"),
        AppAction::ToggleRecording => Some("Record"),
        AppAction::CycleRecordMode => Some("Record Mode"),
        AppAction::ToggleLoopRecordingExtension => Some("Loop Recording Wrap"),
        AppAction::ToggleGlobalLoop => Some("Song Loop"),
        AppAction::ResetGlobalLoop => Some("Reset Song Loop"),
        AppAction::ToggleCurrentTrackLoop => Some("Track Loop"),
        AppAction::ToggleCurrentTrackArm => Some("Track Arm"),
        AppAction::ToggleCurrentTrackMute => Some("Track Mute"),
        AppAction::ToggleCurrentTrackSolo => Some("Track Solo"),
        AppAction::ToggleCurrentTrackPassthrough => Some("Passthrough"),
        AppAction::ToggleLinkEnabled => Some("Link Enable"),
        AppAction::ToggleLinkStartStopSync => Some("Link Start/Stop"),
        _ => None,
    }
}

fn direct_mapping_key_label(event: &sdl3::event::Event) -> Option<String> {
    let sdl3::event::Event::KeyDown {
        keycode: Some(keycode),
        keymod,
        repeat: false,
        ..
    } = event
    else {
        return None;
    };

    if matches!(
        keycode,
        sdl3::keyboard::Keycode::LShift
            | sdl3::keyboard::Keycode::RShift
            | sdl3::keyboard::Keycode::LCtrl
            | sdl3::keyboard::Keycode::RCtrl
            | sdl3::keyboard::Keycode::LAlt
            | sdl3::keyboard::Keycode::RAlt
            | sdl3::keyboard::Keycode::LGui
            | sdl3::keyboard::Keycode::RGui
            | sdl3::keyboard::Keycode::Mode
            | sdl3::keyboard::Keycode::Escape
            | sdl3::keyboard::Keycode::F8
    ) {
        return None;
    }

    let key_label = keycode_mapping_label(*keycode)?;
    Some(with_modifier_prefixes(key_label, *keymod))
}

fn with_modifier_prefixes(key_label: &str, keymod: sdl3::keyboard::Mod) -> String {
    let mut label = String::new();
    if keymod.intersects(sdl3::keyboard::Mod::LCTRLMOD | sdl3::keyboard::Mod::RCTRLMOD) {
        label.push_str("Ctrl+");
    }
    if keymod.intersects(sdl3::keyboard::Mod::LALTMOD | sdl3::keyboard::Mod::RALTMOD) {
        label.push_str("Alt+");
    }
    if keymod.intersects(sdl3::keyboard::Mod::LSHIFTMOD | sdl3::keyboard::Mod::RSHIFTMOD) {
        label.push_str("Shift+");
    }
    label.push_str(key_label);
    label
}

fn keycode_mapping_label(keycode: sdl3::keyboard::Keycode) -> Option<&'static str> {
    match keycode {
        sdl3::keyboard::Keycode::Space => Some("Space"),
        sdl3::keyboard::Keycode::Tab => Some("Tab"),
        sdl3::keyboard::Keycode::Return => Some("Enter"),
        sdl3::keyboard::Keycode::Delete => Some("Delete"),
        sdl3::keyboard::Keycode::Home => Some("Home"),
        sdl3::keyboard::Keycode::Left => Some("Left"),
        sdl3::keyboard::Keycode::Right => Some("Right"),
        sdl3::keyboard::Keycode::Up => Some("Up"),
        sdl3::keyboard::Keycode::Down => Some("Down"),
        sdl3::keyboard::Keycode::LeftBracket => Some("["),
        sdl3::keyboard::Keycode::RightBracket => Some("]"),
        sdl3::keyboard::Keycode::Comma => Some(","),
        sdl3::keyboard::Keycode::Period => Some("."),
        sdl3::keyboard::Keycode::Minus => Some("-"),
        sdl3::keyboard::Keycode::Equals => Some("="),
        sdl3::keyboard::Keycode::Slash => Some("/"),
        sdl3::keyboard::Keycode::Backslash => Some("\\"),
        sdl3::keyboard::Keycode::F1 => Some("F1"),
        sdl3::keyboard::Keycode::F2 => Some("F2"),
        sdl3::keyboard::Keycode::F3 => Some("F3"),
        sdl3::keyboard::Keycode::F4 => Some("F4"),
        sdl3::keyboard::Keycode::F5 => Some("F5"),
        sdl3::keyboard::Keycode::F6 => Some("F6"),
        sdl3::keyboard::Keycode::F7 => Some("F7"),
        sdl3::keyboard::Keycode::_0 => Some("0"),
        sdl3::keyboard::Keycode::_1 => Some("1"),
        sdl3::keyboard::Keycode::_2 => Some("2"),
        sdl3::keyboard::Keycode::_3 => Some("3"),
        sdl3::keyboard::Keycode::_4 => Some("4"),
        sdl3::keyboard::Keycode::_5 => Some("5"),
        sdl3::keyboard::Keycode::_6 => Some("6"),
        sdl3::keyboard::Keycode::_7 => Some("7"),
        sdl3::keyboard::Keycode::_8 => Some("8"),
        sdl3::keyboard::Keycode::_9 => Some("9"),
        sdl3::keyboard::Keycode::A => Some("A"),
        sdl3::keyboard::Keycode::B => Some("B"),
        sdl3::keyboard::Keycode::C => Some("C"),
        sdl3::keyboard::Keycode::D => Some("D"),
        sdl3::keyboard::Keycode::E => Some("E"),
        sdl3::keyboard::Keycode::F => Some("F"),
        sdl3::keyboard::Keycode::G => Some("G"),
        sdl3::keyboard::Keycode::H => Some("H"),
        sdl3::keyboard::Keycode::I => Some("I"),
        sdl3::keyboard::Keycode::J => Some("J"),
        sdl3::keyboard::Keycode::K => Some("K"),
        sdl3::keyboard::Keycode::L => Some("L"),
        sdl3::keyboard::Keycode::M => Some("M"),
        sdl3::keyboard::Keycode::N => Some("N"),
        sdl3::keyboard::Keycode::O => Some("O"),
        sdl3::keyboard::Keycode::P => Some("P"),
        sdl3::keyboard::Keycode::Q => Some("Q"),
        sdl3::keyboard::Keycode::R => Some("R"),
        sdl3::keyboard::Keycode::S => Some("S"),
        sdl3::keyboard::Keycode::T => Some("T"),
        sdl3::keyboard::Keycode::U => Some("U"),
        sdl3::keyboard::Keycode::V => Some("V"),
        sdl3::keyboard::Keycode::W => Some("W"),
        sdl3::keyboard::Keycode::X => Some("X"),
        sdl3::keyboard::Keycode::Y => Some("Y"),
        sdl3::keyboard::Keycode::Z => Some("Z"),
        _ => None,
    }
}

fn mapping_field_index(field: MappingField) -> usize {
    match field {
        MappingField::SourceKind => 0,
        MappingField::SourceDevice => 1,
        MappingField::SourceValue => 2,
        MappingField::Target => 3,
        MappingField::Scope => 4,
        MappingField::Enabled => 5,
    }
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
fn quantize_label(quantize: crate::transport::QuantizeMode) -> &'static str {
    match quantize {
        crate::transport::QuantizeMode::Off => "Off",
        crate::transport::QuantizeMode::Pulse => "Pulse",
        crate::transport::QuantizeMode::Sixteenth => "1/16",
        crate::transport::QuantizeMode::Eighth => "1/8",
        crate::transport::QuantizeMode::Quarter => "1/4",
        crate::transport::QuantizeMode::Bar => "Bar",
    }
}

fn action_source_label(source: ActionSource) -> &'static str {
    match source {
        ActionSource::Keyboard => "Keyboard",
        ActionSource::Pointer => "Pointer",
        ActionSource::Midi => "MIDI",
        ActionSource::Touch => "Touch",
        ActionSource::Remote => "Remote",
        ActionSource::Internal => "Internal",
    }
}

fn mapping_source_sort_key(source_kind: MappingSourceKind) -> usize {
    match source_kind {
        MappingSourceKind::Key => 0,
        MappingSourceKind::Midi => 1,
        MappingSourceKind::Osc => 2,
    }
}

fn badge_kind_prefix(source_kind: MappingSourceKind) -> &'static str {
    match source_kind {
        MappingSourceKind::Key => "K",
        MappingSourceKind::Midi => "M",
        MappingSourceKind::Osc => "O",
    }
}

fn mapping_badge_palette(badge: &MappingBadge) -> (Color, Color) {
    match (badge.built_in, badge.source_kind) {
        (true, MappingSourceKind::Key) => (Color::RGB(64, 84, 126), Color::RGB(244, 244, 236)),
        (true, MappingSourceKind::Midi) => (Color::RGB(88, 94, 116), Color::RGB(236, 240, 246)),
        (true, MappingSourceKind::Osc) => (Color::RGB(84, 90, 112), Color::RGB(236, 240, 246)),
        (false, MappingSourceKind::Key) => (Color::RGB(88, 128, 76), Color::RGB(246, 248, 232)),
        (false, MappingSourceKind::Midi) => (Color::RGB(170, 104, 62), Color::RGB(250, 242, 228)),
        (false, MappingSourceKind::Osc) => (Color::RGB(148, 82, 104), Color::RGB(248, 238, 244)),
    }
}

fn compact_badge_text(text: &str, max_len: usize) -> String {
    let compact = text
        .replace("Shift+", "S+")
        .replace("Space", "Spc")
        .replace("Left", "Lf")
        .replace("Right", "Rt")
        .replace("Active", "Act");
    if compact.chars().count() <= max_len {
        compact
    } else {
        compact.chars().take(max_len).collect()
    }
}

fn track_indicator_target(
    kind: crate::ui::TrackIndicatorKind,
    overlay_slot: Option<Rect>,
) -> Option<DiscoverabilityTarget> {
    match kind {
        crate::ui::TrackIndicatorKind::Armed => Some(DiscoverabilityTarget {
            action: AppAction::ToggleCurrentTrackArm,
            display_scope: Some("Active Track"),
            allowed_mapping_scopes: &["Active Track"],
            overlay_slot,
        }),
        crate::ui::TrackIndicatorKind::Recording => Some(DiscoverabilityTarget {
            action: AppAction::ToggleRecording,
            display_scope: Some("Armed/Active"),
            allowed_mapping_scopes: &["Armed/Active", "Active Track"],
            overlay_slot,
        }),
        crate::ui::TrackIndicatorKind::Muted => Some(DiscoverabilityTarget {
            action: AppAction::ToggleCurrentTrackMute,
            display_scope: Some("Active Track"),
            allowed_mapping_scopes: &["Active Track"],
            overlay_slot,
        }),
        crate::ui::TrackIndicatorKind::Solo => Some(DiscoverabilityTarget {
            action: AppAction::ToggleCurrentTrackSolo,
            display_scope: Some("Active Track"),
            allowed_mapping_scopes: &["Active Track"],
            overlay_slot,
        }),
    }
}

fn port_name(port: Option<&MidiPortRef>) -> &str {
    port.map(|value| value.name.as_str()).unwrap_or("none")
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

fn on_off(value: bool) -> &'static str {
    if value { "on" } else { "off" }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AppControl {
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
        App, AppControl, AppOverlay, DirectMappingMode, DirectMappingOrigin, DirectMappingTarget,
        DiscoverabilityTarget, LastActionStatus, cycle_input_channel, cycle_optional_port,
        cycle_output_channel, mapping_field_index,
    };
    use crate::actions::{ActionSource, AppAction};
    use crate::mapping::{MappingEntry, MappingSourceKind, default_mapping_source_device};
    use crate::midi_io::{MidiInputEvent, MidiInputMessage, MidiPortRef};
    use crate::pages::{AppPage, MappingField, MappingPageMode, MidiIoListFocus, RoutingField};
    use crate::routing::MidiChannelFilter;
    use crate::transport::{QuantizeMode, RecordMode};
    use crate::ui::TimelineFlow;
    use sdl3::rect::Rect;

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
            .transport_chip_specs()
            .into_iter()
            .map(|chip| chip.label)
            .collect::<Vec<_>>();
        assert!(labels.iter().any(|label| label == "RecWrap Extend"));

        app.apply_action(AppAction::ToggleLoopRecordingExtension);
        let labels = app
            .transport_chip_specs()
            .into_iter()
            .map(|chip| chip.label)
            .collect::<Vec<_>>();
        assert!(labels.iter().any(|label| label == "RecWrap Clamp"));
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
        app.playhead_ticks = 480;

        assert_eq!(
            app.effective_track_playhead(app.project.active_track().unwrap()),
            2_400
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
        assert!(
            !summary
                .badges
                .iter()
                .any(|badge| badge.text == "/track/active/arm")
        );
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
        assert_eq!(app.direct_mapping_state.mode, DirectMappingMode::Inactive);
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
        assert_eq!(app.direct_mapping_state.mode, DirectMappingMode::Inactive);
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
    fn toggle_recording_creates_visible_take_content() {
        let mut app = App::new();
        app.project.active_track_mut().unwrap().clear_content();
        app.transport_ticks = 0;
        app.playhead_ticks = 0;

        app.apply_action(AppAction::ToggleRecording);
        assert!(app.project.transport.recording);
        assert!(app.project.transport.playing);

        let input_port = app
            .project
            .active_track()
            .and_then(|track| track.routing.input_port.clone())
            .unwrap_or_else(|| MidiPortRef::new("Keystep 37"));
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

        assert_eq!(
            app.project.active_track().unwrap().regions,
            vec![crate::timeline::Region::new(1_680, 240)]
        );
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

        assert_eq!(
            app.project.active_track().unwrap().regions,
            vec![crate::timeline::Region::new(960, 960)]
        );
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

        assert_eq!(preview, Some(crate::timeline::Region::new(960, 960)));
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
    fn looped_track_preview_clamps_to_loop_end_when_extension_is_off() {
        let mut app = App::new();
        let track = app.project.active_track_mut().unwrap();
        track.clear_content();
        track.state.loop_enabled = true;
        track.loop_region = crate::timeline::LoopRegion::new(2_880, 1_920);
        app.project.transport.quantize = crate::transport::QuantizeMode::Off;
        app.project.transport.loop_enabled = false;
        app.project.transport.loop_recording_extends_clip = false;
        app.transport_ticks = 4_600;
        app.playhead_ticks = 4_600;

        app.apply_action(AppAction::ToggleRecording);
        app.transport_ticks = 4_900;
        app.playhead_ticks = 2_980;

        let active_track = app.project.active_track().unwrap();
        let preview = active_track.preview_region(
            app.project.transport,
            app.record_capture_ticks(active_track),
            app.record_context(active_track),
        );

        assert_eq!(preview, Some(crate::timeline::Region::new(4_600, 200)));
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
    fn timeline_track_arm_indicator_is_clickable() {
        let mut app = App::new();
        let content_bounds = Rect::new(40, 40, 1200, 620);
        let (_, body_bounds) =
            crate::ui::split_top_strip(content_bounds, 28, 6).expect("timeline content");
        let (_, timeline_bounds) =
            crate::ui::split_top_strip(body_bounds, 24, 8).expect("timeline body");
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
            crate::ui::split_top_strip(body_bounds, 24, 8).expect("timeline body");
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
}
