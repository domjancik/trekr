use crate::engine::EngineConfig;
use crate::project::Project;
use crate::render::PaneRenderModel;
use crate::ui::{LayoutMode, TimelineFlow};
use sdl3::event::Event;
use sdl3::keyboard::Keycode;
use sdl3::pixels::Color;
use sdl3::rect::Rect;
use std::time::{Duration, Instant};

/// App is the top-level composition root for the first vertical slice.
pub struct App {
    project: Project,
    engine_config: EngineConfig,
    layout_mode: LayoutMode,
    timeline_flow: TimelineFlow,
}

impl App {
    pub fn new() -> Self {
        Self {
            project: Project::demo(),
            engine_config: EngineConfig::default(),
            layout_mode: LayoutMode::FixedFit,
            timeline_flow: TimelineFlow::DownwardColumns,
        }
    }

    pub fn bootstrap_summary(&self) -> String {
        let full_song = PaneRenderModel::full_song(&self.project);
        let loop_detail = PaneRenderModel::loop_detail(&self.project);

        format!(
            "trekr bootstrap: project='{}', tracks={}, layout={:?}, sample_rate={}, song_ticks={}, loop_ticks={}",
            self.project.name,
            self.project.tracks.len(),
            self.layout_mode,
            self.engine_config.sample_rate_hz,
            full_song.range.length_ticks,
            loop_detail.range.length_ticks
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
        let auto_exit_after = std::env::var("TREKR_EXIT_AFTER_MS")
            .ok()
            .and_then(|value| value.parse::<u64>().ok())
            .map(Duration::from_millis);

        'running: loop {
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => break 'running,
                    Event::KeyDown {
                        keycode: Some(Keycode::Space),
                        repeat: false,
                        ..
                    } => self.timeline_flow = self.timeline_flow.toggle(),
                    _ => {}
                }
            }

            if auto_exit_after.is_some_and(|limit| started_at.elapsed() >= limit) {
                break 'running;
            }

            self.draw(&mut canvas, started_at.elapsed())?;
            std::thread::sleep(Duration::from_millis(16));
        }

        Ok(())
    }

    fn draw(
        &self,
        canvas: &mut sdl3::render::Canvas<sdl3::video::Window>,
        elapsed: Duration,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (width, height) = canvas.output_size()?;
        let panes = crate::ui::split_panes(width, height);
        let full_song = PaneRenderModel::full_song(&self.project);
        let loop_detail = PaneRenderModel::loop_detail(&self.project);

        canvas.set_draw_color(Color::RGB(18, 24, 38));
        canvas.clear();

        self.draw_pane(
            canvas,
            panes[0],
            &full_song,
            elapsed,
            Color::RGB(36, 58, 92),
        )?;
        self.draw_pane(
            canvas,
            panes[1],
            &loop_detail,
            elapsed,
            Color::RGB(92, 58, 36),
        )?;
        let _ = canvas.present();

        Ok(())
    }

    fn draw_pane(
        &self,
        canvas: &mut sdl3::render::Canvas<sdl3::video::Window>,
        bounds: Rect,
        pane: &PaneRenderModel,
        elapsed: Duration,
        accent: Color,
    ) -> Result<(), Box<dyn std::error::Error>> {
        canvas.set_draw_color(Color::RGB(28, 34, 50));
        canvas.fill_rect(bounds)?;

        canvas.set_draw_color(Color::RGB(88, 96, 120));
        canvas.draw_rect(bounds)?;

        let content = crate::ui::inset_rect(bounds, 20, 20)?;
        let lanes = crate::ui::lane_rects(
            content,
            pane.visible_tracks,
            self.timeline_flow,
            pane.compaction,
        );

        for (index, lane) in lanes.iter().enumerate() {
            let lane_color = if index % 2 == 0 {
                accent
            } else {
                Color::RGB(
                    accent.r.saturating_sub(16),
                    accent.g.saturating_sub(16),
                    accent.b.saturating_sub(16),
                )
            };
            canvas.set_draw_color(lane_color);
            canvas.fill_rect(*lane)?;

            canvas.set_draw_color(Color::RGB(180, 188, 205));
            canvas.draw_rect(*lane)?;

            for block in crate::ui::region_blocks(*lane, index, self.timeline_flow) {
                canvas.set_draw_color(Color::RGB(210, 222, 236));
                canvas.fill_rect(block)?;
            }
        }

        let playhead = crate::ui::playhead_rect(
            content,
            self.timeline_flow,
            pane.range.length_ticks,
            elapsed.as_millis() as u64,
        )?;
        canvas.set_draw_color(Color::RGB(248, 240, 132));
        canvas.fill_rect(playhead)?;

        Ok(())
    }
}
