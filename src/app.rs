use crate::actions::{AppAction, KeyboardBindings};
use crate::engine::EngineConfig;
use crate::mapping::{MappingEntry, MappingSourceKind, demo_mappings};
use crate::midi_io::{MidiDeviceCatalog, MidiOutputRuntime, MidiPortRef};
use crate::pages::{AppPage, AppPageState, MidiIoListFocus, RoutingField};
use crate::project::{Project, Track};
use crate::routing::MidiChannelFilter;
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
    midi_output: MidiOutputRuntime,
    mappings: Vec<MappingEntry>,
    viewport_size: (u32, u32),
    playhead_ticks: u64,
}

pub struct UiCaptureOptions {
    pub output_dir: PathBuf,
}

impl App {
    pub fn new() -> Self {
        let scanned_devices = MidiDeviceCatalog::scan();
        let mut app = Self {
            project: Project::demo(),
            engine_config: EngineConfig::default(),
            layout_mode: LayoutMode::FixedFit,
            timeline_flow: TimelineFlow::DownwardColumns,
            keyboard_bindings: KeyboardBindings,
            page_state: AppPageState::default(),
            midi_devices: scanned_devices,
            midi_output: MidiOutputRuntime::default(),
            mappings: demo_mappings(),
            viewport_size: (1280, 720),
            playhead_ticks: 0,
        };
        app.seed_demo_routing();
        app
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

        for page in AppPage::ALL {
            self.page_state.current_page = page;
            let surface = sdl3::surface::Surface::new(1280, 720, PixelFormat::RGBA32)?;
            let mut canvas = surface.into_canvas()?;
            self.draw(&mut canvas)?;
            let output_path = options.output_dir.join(capture_filename(page));
            self.capture_surface_to_png(canvas.surface(), &output_path)?;
        }

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

        canvas.present();
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
        let (header_bounds, timeline_bounds) = crate::ui::split_top_strip(content_bounds, 28, 10)?;
        let reset_button = self.global_loop_reset_button_rect(header_bounds);
        canvas.set_draw_color(Color::RGB(34, 44, 64));
        canvas.fill_rect(header_bounds)?;
        canvas.set_draw_color(Color::RGB(88, 96, 120));
        canvas.draw_rect(header_bounds)?;
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

        let label_left = label_rect.x + 4;
        let label_right_margin = if detail { 26 } else { 4 };
        crate::ui::draw_text_fitted(
            canvas,
            &track.name,
            Rect::new(
                label_left,
                label_rect.y + 6,
                (label_rect.width() as i32 - (label_left - label_rect.x) - label_right_margin).max(0)
                    as u32,
                8,
            ),
            1,
            Color::RGB(244, 244, 236),
        )?;

        let note_range = crate::timeline::LoopRegion::new(view_start_ticks, range_ticks.max(1));
        for note in crate::ui::note_rects(
            content_rect,
            &track.midi_notes,
            note_range,
            self.timeline_flow,
        )
        {
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
        crate::ui::draw_text_fitted(
            canvas,
            "Quick Overview / Read Only",
            Rect::new(content_bounds.x + 200, content_bounds.y + 12, 260, 8),
            1,
            Color::RGB(184, 194, 206),
        )?;

        let list_bounds = crate::ui::inset_rect(content_bounds, 8, 32)?;
        let rows = crate::ui::stacked_rows(list_bounds, self.mappings.len().max(1), 8);
        for (index, row) in rows.into_iter().enumerate().take(self.mappings.len()) {
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

            let source_rect = Rect::new(row.x + 6, row.y + 6, 24, row.height().saturating_sub(12));
            let source_color = match entry.source_kind {
                MappingSourceKind::Key => Color::RGB(98, 148, 232),
                MappingSourceKind::Midi => Color::RGB(96, 202, 146),
                MappingSourceKind::Osc => Color::RGB(220, 154, 88),
            };
            canvas.set_draw_color(source_color);
            canvas.fill_rect(source_rect)?;

            let enabled_rect = Rect::new(
                row.x + row.width() as i32 - 28,
                row.y + 6,
                22,
                row.height().saturating_sub(12),
            );
            canvas.set_draw_color(if entry.enabled {
                Color::RGB(132, 220, 120)
            } else {
                Color::RGB(92, 96, 102)
            });
            canvas.fill_rect(enabled_rect)?;

            let source_label_rect = Rect::new(
                source_rect.x + source_rect.width() as i32 + 8,
                row.y + 8,
                96,
                8,
            );
            let scope_rect = Rect::new(row.x + row.width() as i32 - 148, row.y + 20, 110, 8);
            let target_rect = Rect::new(
                source_label_rect.x + source_label_rect.width() as i32 + 8,
                row.y + 8,
                row.width().saturating_sub(288),
                row.height().saturating_sub(16),
            );
            canvas.set_draw_color(if entry.enabled {
                Color::RGB(182, 194, 212)
            } else {
                Color::RGB(104, 112, 124)
            });
            canvas.fill_rect(target_rect)?;
            crate::ui::draw_text_fitted(
                canvas,
                entry.source_label,
                source_label_rect,
                1,
                Color::RGB(244, 244, 236),
            )?;
            crate::ui::draw_text_fitted(
                canvas,
                entry.target_label,
                Rect::new(target_rect.x + 6, row.y + 8, target_rect.width().saturating_sub(12), 8),
                1,
                Color::RGB(24, 28, 36),
            )?;
            crate::ui::draw_text_fitted(
                canvas,
                entry.scope_label,
                scope_rect,
                1,
                Color::RGB(24, 28, 36),
            )?;
        }

        Ok(())
    }

    fn draw_midi_io_page<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        content_bounds: Rect,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let columns = crate::ui::equal_columns(content_bounds, 2, 14);
        let input_bounds = columns[0];
        let output_bounds = columns[1];
        crate::ui::draw_text_fitted(
            canvas,
            "MIDI I/O",
            Rect::new(content_bounds.x + 8, content_bounds.y + 8, 160, 14),
            2,
            Color::RGB(244, 232, 146),
        )?;

        self.draw_device_list(
            canvas,
            crate::ui::inset_rect(input_bounds, 0, 24)?,
            &self.midi_devices.inputs,
            self.page_state.midi_io.selected_input_index,
            self.midi_devices.selected_input,
            self.page_state.midi_io.focus == MidiIoListFocus::Inputs,
            Color::RGB(78, 196, 164),
        )?;
        self.draw_device_list(
            canvas,
            crate::ui::inset_rect(output_bounds, 0, 24)?,
            &self.midi_devices.outputs,
            self.page_state.midi_io.selected_output_index,
            self.midi_devices.selected_output,
            self.page_state.midi_io.focus == MidiIoListFocus::Outputs,
            Color::RGB(224, 132, 90),
        )?;
        crate::ui::draw_text_fitted(
            canvas,
            "Inputs",
            Rect::new(input_bounds.x + 8, input_bounds.y + 28, 80, 8),
            1,
            Color::RGB(214, 242, 220),
        )?;
        crate::ui::draw_text_fitted(
            canvas,
            "Outputs",
            Rect::new(output_bounds.x + 8, output_bounds.y + 28, 80, 8),
            1,
            Color::RGB(246, 212, 194),
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
                Color::RGB(42, 52, 74)
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

            let bar = Rect::new(
                status.x + status.width() as i32 + 8,
                row.y + 10,
                row.width().saturating_sub(40),
                row.height().saturating_sub(20),
            );
            canvas.set_draw_color(Color::RGB(182, 194, 212));
            canvas.fill_rect(bar)?;
            crate::ui::draw_text_fitted(
                canvas,
                &ports[index].name,
                Rect::new(bar.x + 4, row.y + 8, bar.width().saturating_sub(8), 8),
                1,
                Color::RGB(24, 28, 36),
            )?;
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

        let inner = crate::ui::inset_rect(content_bounds, 12, 32)?;
        let (header, body) = crate::ui::split_top_strip(inner, 40, 10)?;
        let active_track = self.project.active_track().expect("demo project has tracks");

        canvas.set_draw_color(Color::RGB(54, 70, 104));
        canvas.fill_rect(header)?;
        canvas.set_draw_color(Color::RGB(244, 232, 146));
        canvas.draw_rect(header)?;

        let name_badge = Rect::new(
            header.x + 8,
            header.y + 8,
            36,
            header.height().saturating_sub(16),
        );
        canvas.set_draw_color(Color::RGB(220, 124, 100));
        canvas.fill_rect(name_badge)?;
        let state_badge = Rect::new(
            header.x + header.width() as i32 - 34,
            header.y + 8,
            24,
            header.height().saturating_sub(16),
        );
        canvas.set_draw_color(if active_track.state.passthrough {
            Color::RGB(92, 220, 216)
        } else {
            Color::RGB(92, 100, 112)
        });
        canvas.fill_rect(state_badge)?;
        crate::ui::draw_text_fitted(
            canvas,
            &active_track.name,
            Rect::new(
                name_badge.x + name_badge.width() as i32 + 8,
                header.y + 12,
                (state_badge.x - (name_badge.x + name_badge.width() as i32 + 16)).max(0) as u32,
                14,
            ),
            2,
            Color::RGB(244, 244, 236),
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

            let label = Rect::new(row.x + 8, row.y + 8, 26, row.height().saturating_sub(16));
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
            let label_text_rect = Rect::new(row.x + 44, row.y + 8, 150, 8);
            let value = Rect::new(
                row.x + 210,
                row.y + 10,
                row.width().saturating_sub(220),
                row.height().saturating_sub(20),
            );
            canvas.set_draw_color(value_color);
            canvas.fill_rect(value)?;
            crate::ui::draw_text_fitted(
                canvas,
                field.label(),
                label_text_rect,
                1,
                Color::RGB(244, 244, 236),
            )?;
            crate::ui::draw_text_fitted(
                canvas,
                &self.routing_field_value(active_track, field),
                Rect::new(value.x + 6, row.y + 8, value.width().saturating_sub(12), 8),
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
        let active = self.project.active_track().expect("demo project always has tracks");
        let title = match self.page_state.current_page {
            AppPage::Timeline => format!(
                "trekr | Page:{} (Tab/F1-F4) | T{} {} | Tick:{} | Space Play:{} | [ ] TrackLoop:{}-{} | , . Nudge | - = Resize | / \\ Half/Double | Shift+[ ] SongLoop:{}-{} | G:{} L:{} A:{} M:{} S:{} I:{}",
                self.page_state.current_page.label(),
                self.project.active_track_index + 1,
                active.name,
                self.playhead_ticks,
                on_off(self.project.transport.playing),
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
                    "trekr | Page:{} (Tab/F1-F4) | Quick Overview Read Only | Up/Down Select | Source:{} {} | Target:{} | Scope:{} | Enabled:{}",
                    self.page_state.current_page.label(),
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
            AppAction::TogglePlayback => {
                self.project.transport.playing = !self.project.transport.playing;
                if !self.project.transport.playing {
                    self.silence_all_tracks();
                }
                AppControl::Continue
            }
            AppAction::ToggleGlobalLoop => {
                self.project.transport.loop_enabled = !self.project.transport.loop_enabled;
                AppControl::Continue
            }
            AppAction::ResetGlobalLoop => {
                self.project.loop_region = self.project.full_song_range();
                self.project.transport.loop_enabled = true;
                self.playhead_ticks = self
                    .playhead_ticks
                    .clamp(self.project.loop_region.start_ticks, self.project.loop_region.end_ticks());
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
                self.project.loop_region.set_start_preserving_end(edit_ticks);
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
        if !self.project.transport.playing {
            return;
        }

        let previous_ticks = self.playhead_ticks;
        let ticks_per_second = self.project.transport.ticks_per_second();
        let advanced_ticks =
            (delta.as_nanos() as u128 * u128::from(ticks_per_second)) / 1_000_000_000_u128;
        self.playhead_ticks = self.playhead_ticks.saturating_add(advanced_ticks as u64);

        if self.project.transport.loop_enabled {
            let loop_region = self.project.loop_region;
            if loop_region.length_ticks > 0 {
                let relative = self.playhead_ticks.saturating_sub(loop_region.start_ticks);
                self.playhead_ticks =
                    loop_region.start_ticks + (relative % loop_region.length_ticks.max(1));
            }
        }

        self.dispatch_midi_notes(previous_ticks, advanced_ticks as u64);
    }

    fn current_edit_ticks(&self) -> u64 {
        self.project.transport.quantize_to_nearest(self.playhead_ticks)
    }

    fn nudge_step_ticks(&self) -> u64 {
        self.project.transport.quantize_step_ticks().unwrap_or(1).max(1)
    }

    fn effective_track_playhead(&self, track: &Track) -> u64 {
        let raw = self.playhead_ticks;
        if !track.state.loop_enabled || track.loop_region.length_ticks == 0 {
            return raw;
        }

        track.loop_region.start_ticks + (raw % track.loop_region.length_ticks)
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
    }

    fn select_previous_page_item(&mut self) {
        match self.page_state.current_page {
            AppPage::Timeline => self.project.select_previous_track(),
            AppPage::Mappings => {
                if !self.mappings.is_empty() {
                    let count = self.mappings.len();
                    self.page_state.selected_mapping_index =
                        (self.page_state.selected_mapping_index + count - 1) % count;
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
                self.page_state.selected_routing_field = self.page_state.selected_routing_field.next();
            }
        }
    }

    fn adjust_page_item(&mut self, delta: i32) {
        match self.page_state.current_page {
            AppPage::Timeline => {}
            AppPage::Mappings => {}
            AppPage::MidiIo => {
                self.page_state.midi_io.focus = self.page_state.midi_io.focus.toggle();
            }
            AppPage::Routing => self.adjust_routing_field(delta),
        }
    }

    fn activate_page_item(&mut self) {
        match self.page_state.current_page {
            AppPage::Timeline => {}
            AppPage::Mappings => {}
            AppPage::MidiIo => match self.page_state.midi_io.focus {
                MidiIoListFocus::Inputs => self
                    .midi_devices
                    .set_selected_input(self.page_state.midi_io.selected_input_index),
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
                }
            }
        }
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
                let events = scheduled_note_events(track, previous_ticks, advanced_ticks, loop_range);
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
                track.routing
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
        let (_, content_bounds) = crate::ui::split_top_strip(inset, 28, 12).expect("fixed tabs split");
        let (header_bounds, _) =
            crate::ui::split_top_strip(content_bounds, 28, 10).expect("fixed timeline split");
        header_bounds
    }

    fn global_loop_reset_button_rect(&self, header_bounds: Rect) -> Rect {
        let width = crate::ui::text_width("Reset Song Loop", 1) + 18;
        Rect::new(
            header_bounds.x + 8,
            header_bounds.y + 4,
            width,
            header_bounds.height().saturating_sub(8),
        )
    }
}

fn capture_filename(page: AppPage) -> &'static str {
    match page {
        AppPage::Timeline => "timeline.png",
        AppPage::Mappings => "mappings.png",
        AppPage::MidiIo => "midi-io.png",
        AppPage::Routing => "routing.png",
    }
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
        .unwrap_or_else(|| vec![(previous_ticks, previous_ticks.saturating_add(advanced_ticks))]);

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

fn ranged_segments(previous_ticks: u64, advanced_ticks: u64, range: crate::timeline::LoopRegion) -> Vec<(u64, u64)> {
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
    let current_index = current.map(|value| i32::from(value.clamp(1, 16))).unwrap_or(0);
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
    use super::{App, AppControl, cycle_input_channel, cycle_optional_port, cycle_output_channel};
    use crate::actions::AppAction;
    use crate::pages::{AppPage, MidiIoListFocus, RoutingField};
    use crate::routing::MidiChannelFilter;
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

        assert_ne!(app.project.active_track().unwrap().routing.output_channel, before);
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
}
