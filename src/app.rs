use crate::actions::{AppAction, KeyboardBindings};
use crate::engine::EngineConfig;
use crate::link::{LinkRuntime, LinkSnapshot};
use crate::mapping::{
    MappingEntry, MappingSourceKind, cycle_mapping_scope_label, cycle_mapping_source_kind,
    cycle_mapping_source_label, cycle_mapping_target_label, default_source_label, demo_mappings,
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
use crate::transport::RecordMode;
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
    viewport_size: (u32, u32),
    transport_ticks: u64,
    playhead_ticks: u64,
    link_snapshot: LinkSnapshot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AppOverlay {
    MappingsQuickView,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct OverlayState {
    active: Option<AppOverlay>,
}

pub struct UiCaptureOptions {
    pub output_dir: PathBuf,
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
        app.sync_midi_inputs();
        app
    }

    pub fn persisted_state(&self) -> PersistedAppState {
        PersistedAppState {
            project: self.project.clone(),
            page_state: self.page_state,
            timeline_flow: self.timeline_flow,
            mappings: self.mappings.clone(),
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
            viewport_size: (1280, 720),
            transport_ticks: 0,
            playhead_ticks: 0,
            link_snapshot,
        }
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
        let sdl_context = sdl3::init()?;
        let video = sdl_context.video()?;
        let window = video
            .window("trekr", 1280, 720)
            .position_centered()
            .resizable()
            .build()
            .map_err(|err| err.to_string())?;
        let mut canvas = window.into_canvas();
        let mut event_pump = sdl_context.event_pump()?;
        let started_at = Instant::now();
        let mut last_frame_at = started_at;
        let auto_exit_after = std::env::var("TREKR_EXIT_AFTER_MS")
            .ok()
            .and_then(|value| value.parse::<u64>().ok())
            .map(Duration::from_millis);

        'running: loop {
            for event in event_pump.poll_iter() {
                if let Some(action_event) = self.pointer_action(&event) {
                    if self.apply_action(action_event.action) == AppControl::Quit {
                        break 'running;
                    }
                    continue;
                }

                if let Some(action_event) = self.keyboard_bindings.resolve(&event) {
                    if self.apply_action(action_event.action) == AppControl::Quit {
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
            self.viewport_size = canvas.output_size()?;

            self.update_window_title(canvas.window_mut())?;
            self.draw(&mut canvas)?;
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
            self.draw(&mut canvas)?;
            let output_path = options.output_dir.join(spec.filename);
            self.capture_surface_to_png(canvas.surface(), &output_path)?;
        }

        self.overlay_state.active = None;

        Ok(())
    }

    fn draw<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (width, height) = canvas.output_size()?;
        let surface = crate::ui::surface_rect(width, height);
        let inset = crate::ui::inset_rect(surface, 24, 24)?;
        let (tabs_bounds, content_bounds) = crate::ui::split_top_strip(inset, 28, 12)?;

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

        self.draw_overlay(canvas, inset)?;

        canvas.present();
        Ok(())
    }

    fn draw_overlay<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        bounds: Rect,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match self.overlay_state.active {
            Some(AppOverlay::MappingsQuickView) => self.draw_mappings_overlay(canvas, bounds),
            None => Ok(()),
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
        let record_mode_badge = Rect::new(header_bounds.x + 96, header_bounds.y + 6, 108, 14);
        canvas.set_draw_color(match self.project.transport.record_mode {
            RecordMode::Overdub => Color::RGB(54, 82, 126),
            RecordMode::Replace => Color::RGB(122, 66, 48),
        });
        canvas.fill_rect(record_mode_badge)?;
        crate::ui::draw_text_fitted(
            canvas,
            &format!("Rec {}", self.project.transport.record_mode.label()),
            Rect::new(
                record_mode_badge.x + 6,
                record_mode_badge.y + 4,
                record_mode_badge.width().saturating_sub(12),
                8,
            ),
            1,
            Color::RGB(244, 244, 236),
        )?;
        crate::ui::draw_text_fitted(
            canvas,
            "Full + loop detail",
            Rect::new(
                record_mode_badge.x + record_mode_badge.width() as i32 + 12,
                header_bounds.y + 8,
                116,
                8,
            ),
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
        let full_accent = if track.state.armed {
            Color::RGB(148, 54, 54)
        } else if is_active {
            Color::RGB(42, 90, 168)
        } else {
            Color::RGB(36, 58, 92)
        };
        let detail_accent = if track.state.loop_enabled && self.project.transport.loop_enabled {
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
            track.loop_region.start_ticks,
            track.loop_region.length_ticks,
            self.effective_track_playhead(track),
            is_active,
            true,
            track,
        )?;

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

        let status_rect = crate::ui::track_status_rect(bounds, self.timeline_flow);
        let label_rect = crate::ui::track_label_rect(bounds, self.timeline_flow);
        let content_rect = crate::ui::track_content_rect(bounds, self.timeline_flow);
        canvas.set_draw_color(Color::RGB(26, 34, 52));
        canvas.fill_rect(status_rect)?;
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

        for badge in crate::ui::header_badges(status_rect) {
            let color = match badge.kind {
                crate::ui::HeaderBadgeKind::TrackIndex => {
                    if is_active {
                        Color::RGB(250, 244, 200)
                    } else {
                        Color::RGB(208, 216, 228)
                    }
                }
                crate::ui::HeaderBadgeKind::Armed => {
                    if track.state.armed {
                        Color::RGB(250, 110, 110)
                    } else {
                        Color::RGB(82, 74, 74)
                    }
                }
                crate::ui::HeaderBadgeKind::Muted => {
                    if track.state.muted {
                        Color::RGB(136, 140, 150)
                    } else {
                        Color::RGB(72, 76, 86)
                    }
                }
                crate::ui::HeaderBadgeKind::Solo => {
                    if track.state.soloed {
                        Color::RGB(124, 214, 132)
                    } else {
                        Color::RGB(70, 84, 70)
                    }
                }
            };
            canvas.set_draw_color(color);
            canvas.fill_rect(badge.rect)?;
        }

        if track.active_take.is_some() {
            let record_badge = Rect::new(
                status_rect.x + status_rect.width() as i32 - 18,
                status_rect.y + 4,
                14,
                status_rect.height().saturating_sub(8),
            );
            canvas.set_draw_color(Color::RGB(238, 88, 88));
            canvas.fill_rect(record_badge)?;
        }

        let label_left = label_rect.x + 4;
        let label_right_margin = if detail { 26 } else { 4 };
        crate::ui::draw_text_fitted(
            canvas,
            if detail { "Loop" } else { &track.name },
            Rect::new(
                label_left,
                label_rect.y + 14,
                (label_rect.width() as i32 - (label_left - label_rect.x) - label_right_margin)
                    .max(0) as u32,
                8,
            ),
            1,
            Color::RGB(244, 244, 236),
        )?;

        let note_range = crate::timeline::LoopRegion::new(view_start_ticks, range_ticks.max(1));
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
            self.record_head_ticks(track),
            self.record_range(track),
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
                self.record_head_ticks(track),
                self.record_range(track),
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
            canvas.set_draw_color(if track.state.muted {
                Color::RGB(92, 100, 112)
            } else if note.clipped {
                Color::RGB(244, 204, 132)
            } else {
                Color::RGB(210, 222, 236)
            });
            canvas.fill_rect(note.rect)?;
            canvas.set_draw_color(if track.state.muted {
                Color::RGB(128, 134, 144)
            } else {
                Color::RGB(245, 247, 250)
            });
            canvas.draw_rect(note.rect)?;
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

        let chips = [
            (
                format!("Play {}", on_off(self.project.transport.playing)),
                if self.project.transport.playing {
                    Color::RGB(96, 162, 122)
                } else {
                    Color::RGB(74, 84, 102)
                },
            ),
            (
                format!("Rec {}", on_off(self.project.transport.recording)),
                if self.project.transport.recording {
                    Color::RGB(180, 76, 76)
                } else {
                    Color::RGB(88, 78, 82)
                },
            ),
            (
                format!("Mode {}", self.project.transport.record_mode.label()),
                Color::RGB(76, 94, 136),
            ),
            (
                format!("SongLoop {}", on_off(self.project.transport.loop_enabled)),
                Color::RGB(116, 96, 54),
            ),
            (
                format!("Tempo {}", self.project.transport.tempo_bpm),
                Color::RGB(70, 100, 120),
            ),
        ];

        let mut cursor_x = bounds.x + 6;
        for (label, fill) in chips {
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
            if cursor_x >= bounds.x + bounds.width() as i32 - 120 {
                break;
            }
        }

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
        let mut right_cursor = bounds.x + bounds.width() as i32 - 8;
        for (label, fill) in right_badges.into_iter().rev() {
            let width = crate::ui::text_width(&label, 1) + 12;
            right_cursor -= width as i32;
            let chip = Rect::new(
                right_cursor,
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
            right_cursor -= 6;
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
                (right_cursor - hint_width as i32 - 4).max(cursor_x + 6),
                bounds.y + 8,
                hint_width,
                8,
            ),
            1,
            Color::RGB(166, 176, 192),
        )?;

        Ok(())
    }

    fn mapping_row_cells(&self, row: Rect) -> [Rect; 5] {
        let type_rect = Rect::new(row.x + 4, row.y + 3, 46, row.height().saturating_sub(6));
        let source_rect = Rect::new(
            type_rect.x + type_rect.width() as i32 + 6,
            row.y + 3,
            116,
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
            source_rect.x + source_rect.width() as i32 + 6,
            row.y + 3,
            (scope_rect.x - (source_rect.x + source_rect.width() as i32 + 12)).max(48) as u32,
            row.height().saturating_sub(6),
        );
        [
            type_rect,
            source_rect,
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
        canvas.set_draw_color(Color::RGB(50, 62, 88));
        canvas.fill_rect(overview_badge)?;
        crate::ui::draw_text_fitted(
            canvas,
            &format!("Mode: {}", self.page_state.mapping_mode.label()),
            Rect::new(content_bounds.x + 208, content_bounds.y + 12, 170, 8),
            1,
            Color::RGB(206, 214, 224),
        )?;
        let learn_badge = Rect::new(content_bounds.x + 392, content_bounds.y + 8, 136, 16);
        canvas.set_draw_color(if self.page_state.mapping_midi_learn_armed {
            Color::RGB(146, 62, 62)
        } else {
            Color::RGB(44, 56, 78)
        });
        canvas.fill_rect(learn_badge)?;
        crate::ui::draw_text_fitted(
            canvas,
            if self.page_state.mapping_midi_learn_armed {
                "Learn: waiting"
            } else {
                "Learn: idle"
            },
            Rect::new(learn_badge.x + 8, learn_badge.y + 4, 120, 8),
            1,
            Color::RGB(236, 240, 246),
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

            let enabled_rect = Rect::new(cells[4].x + 6, cells[4].y, 14, cells[4].height());
            canvas.set_draw_color(if entry.enabled {
                Color::RGB(132, 220, 120)
            } else {
                Color::RGB(92, 96, 102)
            });
            canvas.fill_rect(enabled_rect)?;

            let kind_rect = cells[0];
            let trigger_rect = cells[1];
            let target_rect = cells[2];
            let scope_rect = cells[3];
            canvas.set_draw_color(if selected {
                Color::RGB(66, 80, 112)
            } else {
                Color::RGB(42, 50, 70)
            });
            canvas.fill_rect(kind_rect)?;
            canvas.fill_rect(trigger_rect)?;
            canvas.set_draw_color(if entry.enabled {
                Color::RGB(182, 194, 212)
            } else {
                Color::RGB(104, 112, 124)
            });
            canvas.fill_rect(target_rect)?;
            canvas.set_draw_color(Color::RGB(66, 74, 88));
            canvas.fill_rect(scope_rect)?;
            canvas.fill_rect(cells[4])?;
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
                    cells[4].x + 2,
                    row.y + 5,
                    cells[4].width().saturating_sub(4),
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
            }
        }

        canvas.set_draw_color(Color::RGB(26, 32, 46));
        canvas.fill_rect(footer_bounds)?;
        crate::ui::draw_text_fitted(
            canvas,
            "F5 Overlay  W Write  Shift+Left/Right Field  Q/E Adjust  Enter Learn/Toggle",
            Rect::new(
                footer_bounds.x + 6,
                footer_bounds.y + 2,
                footer_bounds.width().saturating_sub(12),
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
            "F5 closes",
            Rect::new(panel.x + 12, panel.y + 32, 70, 8),
            1,
            Color::RGB(188, 198, 212),
        )?;
        crate::ui::draw_text_fitted(
            canvas,
            "W write mode",
            Rect::new(panel.x + 92, panel.y + 32, 86, 8),
            1,
            Color::RGB(188, 198, 212),
        )?;

        let list_bounds = crate::ui::inset_rect(panel, 12, 54)?;
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

            let active_badge_width = if is_active { 40 } else { 0 };
            let header_rect = Rect::new(
                status.x + status.width() as i32 + 8,
                row.y + 8,
                row.width()
                    .saturating_sub(40)
                    .saturating_sub(active_badge_width as u32),
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
                let active_badge = Rect::new(row.x + row.width() as i32 - 48, row.y + 8, 32, 8);
                canvas.set_draw_color(accent);
                canvas.fill_rect(active_badge)?;
                crate::ui::draw_text_fitted(
                    canvas,
                    "Def",
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
        let (header, body) = crate::ui::split_top_strip(inner, 40, 10)?;
        let active_track = self
            .project
            .active_track()
            .expect("demo project has tracks");

        canvas.set_draw_color(Color::RGB(54, 70, 104));
        canvas.fill_rect(header)?;
        canvas.set_draw_color(Color::RGB(244, 232, 146));
        canvas.draw_rect(header)?;

        let name_badge = Rect::new(
            header.x + 8,
            header.y + 8,
            52,
            header.height().saturating_sub(16),
        );
        canvas.set_draw_color(Color::RGB(220, 124, 100));
        canvas.fill_rect(name_badge)?;
        crate::ui::draw_text_fitted(
            canvas,
            "Act",
            Rect::new(
                name_badge.x + 4,
                name_badge.y + 4,
                name_badge.width().saturating_sub(8),
                8,
            ),
            1,
            Color::RGB(24, 28, 36),
        )?;
        let state_badge = Rect::new(
            header.x + header.width() as i32 - 118,
            header.y + 8,
            108,
            header.height().saturating_sub(16),
        );
        canvas.set_draw_color(if active_track.state.passthrough {
            Color::RGB(72, 188, 180)
        } else {
            Color::RGB(92, 100, 112)
        });
        canvas.fill_rect(state_badge)?;
        crate::ui::draw_text_fitted(
            canvas,
            if active_track.state.passthrough {
                "Thru On"
            } else {
                "Thru Off"
            },
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
            &format!("Track {}", self.project.active_track_index + 1),
            Rect::new(
                name_badge.x + name_badge.width() as i32 + 8,
                header.y + 9,
                74,
                8,
            ),
            1,
            Color::RGB(244, 244, 236),
        )?;
        crate::ui::draw_text_fitted(
            canvas,
            &active_track.name,
            Rect::new(
                name_badge.x + name_badge.width() as i32 + 8,
                header.y + 21,
                (state_badge.x - (name_badge.x + name_badge.width() as i32 + 16)).max(0) as u32,
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
                row.x + row.width() as i32 - 56,
                row.y + 8,
                46,
                row.height().saturating_sub(16),
            );
            canvas.set_draw_color(value_color);
            canvas.fill_rect(value)?;
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
                    &self.routing_field_value(active_track, field),
                    Rect::new(
                        value.x + 8,
                        value.y + 6,
                        value.width().saturating_sub(16),
                        8,
                    ),
                    1,
                    Color::RGB(24, 28, 36),
                )?;
            }
            crate::ui::draw_text_fitted(
                canvas,
                if selected { "Edit" } else { "View" },
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
                "trekr | Page:{} (Tab/F1-F4) | T{} {} | Tick:{} | Space Play:{} | R Rec:{} | Shift+R Mode:{} | F6 Link:{} Shift+F6 Sync:{} Peers:{} | C Clear Track | Shift+C Clear All | [ ] TrackLoop:{}-{} | , . Nudge | - = Resize | / \\ Half/Double | Shift+[ ] SongLoop:{}-{} | G:{} L:{} A:{} M:{} S:{} I:{}",
                self.page_state.current_page.label(),
                self.project.active_track_index + 1,
                active.name,
                self.playhead_ticks,
                on_off(self.project.transport.playing),
                on_off(self.project.transport.recording),
                self.project.transport.record_mode.label(),
                on_off(self.project.transport.link_enabled),
                on_off(self.project.transport.link_start_stop_sync),
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
                    "trekr | Page:{} (Tab/F1-F4) | Mode:{} | F5 Overlay:{} | W Toggle Mode | Shift+Left/Right Field:{} | Learn:{} | Up/Down Select | Source:{} {} | Target:{} | Scope:{} | Enabled:{}",
                    self.page_state.current_page.label(),
                    self.page_state.mapping_mode.label(),
                    on_off(self.overlay_state.active == Some(AppOverlay::MappingsQuickView)),
                    self.page_state.selected_mapping_field.label(),
                    on_off(self.page_state.mapping_midi_learn_armed),
                    mapping_source_label(selected.source_kind),
                    selected.source_label,
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
                    "trekr | Page:{} (Tab/F1-F4) | Focus:{} | Up/Down Select | Q/E Switch List | Enter Set Default | Selected:{} | Default In:{} | Default Out:{}",
                    self.page_state.current_page.label(),
                    focus,
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
                "trekr | Page:{} (Tab/F1-F4) | T{} {} | Up/Down Field | Q/E Adjust | Enter Toggle | Field:{} | In:{} {} | Out:{} {} | Thru:{}",
                self.page_state.current_page.label(),
                self.project.active_track_index + 1,
                active.name,
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

    fn apply_action(&mut self, action: AppAction) -> AppControl {
        match action {
            AppAction::Quit => AppControl::Quit,
            AppAction::ShowPage(page) => {
                self.page_state.current_page = page;
                AppControl::Continue
            }
            AppAction::ShowNextPage => {
                self.page_state.current_page = self.page_state.current_page.next();
                AppControl::Continue
            }
            AppAction::ShowPreviousPage => {
                self.page_state.current_page = self.page_state.current_page.previous();
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
                AppControl::Continue
            }
            AppAction::ToggleMappingsWriteMode => {
                self.page_state.mapping_mode = self.page_state.mapping_mode.toggle();
                self.page_state.mapping_midi_learn_armed = false;
                if self.page_state.mapping_mode == MappingPageMode::Overview {
                    self.page_state.selected_mapping_field = MappingField::SourceValue;
                }
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
            AppAction::CycleRecordMode => {
                self.project.transport.record_mode = self.project.transport.record_mode.next();
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

    fn record_range(&self, track: &Track) -> Option<crate::timeline::LoopRegion> {
        if track.state.loop_enabled {
            Some(track.loop_region)
        } else if self.project.transport.loop_enabled {
            Some(self.project.loop_region)
        } else {
            None
        }
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
                .map(|track| self.record_head_ticks(track))
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
                .map(|track| self.record_head_ticks(track))
                .unwrap_or(self.playhead_ticks);
            let record_range = self
                .project
                .tracks
                .get(index)
                .and_then(|track| self.record_range(track));
            if let Some(track) = self.project.tracks.get_mut(index) {
                if track.active_take.is_some() {
                    track.finish_recording(transport, release_ticks, record_range);
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
                self.page_state.selected_mapping_field.previous();
            self.page_state.mapping_midi_learn_armed = false;
        }
    }

    fn select_next_page_field(&mut self) {
        if self.page_state.current_page == AppPage::Mappings
            && self.page_state.mapping_mode == MappingPageMode::Write
        {
            self.page_state.selected_mapping_field = self.page_state.selected_mapping_field.next();
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
        let Some(entry) = self.mappings.get_mut(index) else {
            return;
        };

        self.page_state.mapping_midi_learn_armed = false;
        match field {
            MappingField::SourceKind => {
                entry.source_kind = cycle_mapping_source_kind(entry.source_kind, delta);
                entry.source_label = default_source_label(entry.source_kind).to_string();
            }
            MappingField::SourceValue => {
                entry.source_label =
                    cycle_mapping_source_label(entry.source_kind, &entry.source_label, delta)
                        .to_string();
            }
            MappingField::Target => {
                entry.target_label =
                    cycle_mapping_target_label(&entry.target_label, delta).to_string();
            }
            MappingField::Scope => {
                entry.scope_label =
                    cycle_mapping_scope_label(&entry.scope_label, delta).to_string();
            }
            MappingField::Enabled => {
                entry.enabled = delta > 0;
            }
        }
    }

    fn activate_mapping_field(&mut self) {
        let index = self.page_state.selected_mapping_index;
        let field = self.page_state.selected_mapping_field;
        let Some(entry) = self.mappings.get_mut(index) else {
            return;
        };

        match field {
            MappingField::SourceKind => {
                entry.source_kind = cycle_mapping_source_kind(entry.source_kind, 1);
                entry.source_label = default_source_label(entry.source_kind).to_string();
                self.page_state.mapping_midi_learn_armed = false;
            }
            MappingField::SourceValue => {
                if entry.source_kind == MappingSourceKind::Midi {
                    self.page_state.mapping_midi_learn_armed =
                        !self.page_state.mapping_midi_learn_armed;
                } else {
                    entry.source_label =
                        cycle_mapping_source_label(entry.source_kind, &entry.source_label, 1)
                            .to_string();
                }
            }
            MappingField::Target => {
                entry.target_label = cycle_mapping_target_label(&entry.target_label, 1).to_string();
                self.page_state.mapping_midi_learn_armed = false;
            }
            MappingField::Scope => {
                entry.scope_label = cycle_mapping_scope_label(&entry.scope_label, 1).to_string();
                self.page_state.mapping_midi_learn_armed = false;
            }
            MappingField::Enabled => {
                entry.enabled = !entry.enabled;
                self.page_state.mapping_midi_learn_armed = false;
            }
        }
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
        self.midi_input.sync_ports(&ports);
    }

    fn poll_midi_input(&mut self) {
        let events = self.midi_input.drain_events();
        for event in events {
            self.handle_midi_input_event(event);
        }
    }

    fn handle_midi_input_event(&mut self, event: MidiInputEvent) {
        if self.capture_mapping_midi_learn(&event) {
            return;
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
                .map(|track| self.record_head_ticks(track))
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
        entry.source_label = midi_learn_label(event);
        entry.enabled = true;
        self.page_state.mapping_midi_learn_armed = false;
        true
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

    fn pointer_action(&self, event: &sdl3::event::Event) -> Option<crate::actions::ActionEvent> {
        match event {
            sdl3::event::Event::MouseButtonDown { x, y, .. }
                if self.page_state.current_page == AppPage::Timeline
                    && rect_contains(
                        self.global_loop_reset_button_rect(self.timeline_header_bounds()),
                        *x as i32,
                        *y as i32,
                    ) =>
            {
                Some(crate::actions::ActionEvent::new(
                    AppAction::ResetGlobalLoop,
                    crate::actions::ActionSource::Pointer,
                ))
            }
            _ => None,
        }
    }

    fn timeline_header_bounds(&self) -> Rect {
        let surface = crate::ui::surface_rect(self.viewport_size.0, self.viewport_size.1);
        let inset = crate::ui::inset_rect(surface, 24, 24).expect("fixed app inset");
        let (_, content_bounds) =
            crate::ui::split_top_strip(inset, 28, 12).expect("fixed tabs split");
        let (header_bounds, _) =
            crate::ui::split_top_strip(content_bounds, 28, 10).expect("fixed timeline split");
        header_bounds
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

fn mapping_field_index(field: MappingField) -> usize {
    match field {
        MappingField::SourceKind => 0,
        MappingField::SourceValue => 1,
        MappingField::Target => 2,
        MappingField::Scope => 3,
        MappingField::Enabled => 4,
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

#[cfg(test)]
mod tests {
    use super::{
        App, AppControl, AppOverlay, cycle_input_channel, cycle_optional_port, cycle_output_channel,
    };
    use crate::actions::AppAction;
    use crate::mapping::MappingSourceKind;
    use crate::midi_io::{MidiInputEvent, MidiInputMessage, MidiPortRef};
    use crate::pages::{AppPage, MappingField, MappingPageMode, MidiIoListFocus, RoutingField};
    use crate::routing::MidiChannelFilter;
    use crate::transport::RecordMode;
    use crate::ui::TimelineFlow;

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
        assert!(!app.page_state.mapping_midi_learn_armed);
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
    fn midi_io_page_can_switch_focus_and_commit_default_ports() {
        let mut app = App::new();
        app.apply_action(AppAction::ShowPage(AppPage::MidiIo));
        app.apply_action(AppAction::SelectNextPageItem);
        app.apply_action(AppAction::ActivatePageItem);
        assert_eq!(app.midi_devices.selected_input, Some(1));

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
        app.playhead_ticks = 1_680;

        app.apply_action(AppAction::ToggleRecording);
        app.playhead_ticks = 1_200;
        app.apply_action(AppAction::ToggleRecording);

        assert_eq!(
            app.project.active_track().unwrap().regions,
            vec![crate::timeline::Region::new(1_680, 240)]
        );
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
}
