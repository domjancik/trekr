use crate::actions::{AppAction, KeyboardBindings};
use crate::engine::EngineConfig;
use crate::project::Project;
use crate::render::PaneRenderModel;
use crate::ui::{LayoutMode, TimelineFlow};
use sdl3::pixels::Color;
use sdl3::rect::Rect;
use std::time::{Duration, Instant};

/// App is the top-level composition root for the first vertical slice.
pub struct App {
    project: Project,
    engine_config: EngineConfig,
    layout_mode: LayoutMode,
    timeline_flow: TimelineFlow,
    keyboard_bindings: KeyboardBindings,
}

impl App {
    pub fn new() -> Self {
        Self {
            project: Project::demo(),
            engine_config: EngineConfig::default(),
            layout_mode: LayoutMode::FixedFit,
            timeline_flow: TimelineFlow::DownwardColumns,
            keyboard_bindings: KeyboardBindings,
        }
    }

    pub fn bootstrap_summary(&self) -> String {
        let full_song = PaneRenderModel::full_song(&self.project);
        let loop_detail = PaneRenderModel::loop_detail(&self.project);

        format!(
            "trekr bootstrap: project='{}', tracks={}, layout={:?}, sample_rate={}, song_ticks={}, loop_ticks={}, playing={}, loop_enabled={}",
            self.project.name,
            self.project.tracks.len(),
            self.layout_mode,
            self.engine_config.sample_rate_hz,
            full_song.range.length_ticks,
            loop_detail.range.length_ticks,
            self.project.transport.playing,
            self.project.transport.loop_enabled
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
                if let Some(action_event) = self.keyboard_bindings.resolve(&event) {
                    if self.apply_action(action_event.action) == AppControl::Quit {
                        break 'running;
                    }
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

        canvas.set_draw_color(Color::RGB(20, 27, 40));
        canvas.fill_rect(content)?;

        for guide in crate::ui::timeline_guides(content, self.timeline_flow) {
            canvas.set_draw_color(Color::RGB(52, 62, 84));
            canvas.fill_rect(guide)?;
        }

        for (index, lane) in lanes.iter().enumerate() {
            let lane_color = if index % 2 == 0 {
                accent
            } else {
                Color::RGB(
                    accent.r.saturating_sub(22),
                    accent.g.saturating_sub(22),
                    accent.b.saturating_sub(22),
                )
            };
            canvas.set_draw_color(lane_color);
            canvas.fill_rect(crate::ui::track_header_rect(*lane, self.timeline_flow))?;

            canvas.set_draw_color(Color::RGB(180, 188, 205));
            canvas.draw_rect(*lane)?;

            for block in crate::ui::region_blocks(*lane, index, self.timeline_flow) {
                canvas.set_draw_color(Color::RGB(210, 222, 236));
                canvas.fill_rect(block)?;
                canvas.set_draw_color(Color::RGB(245, 247, 250));
                canvas.draw_rect(block)?;
            }
        }

        let playhead = crate::ui::playhead_rect(
            content,
            self.timeline_flow,
            pane.range.length_ticks,
            elapsed.as_millis() as u64,
        )?;
        canvas.set_draw_color(if self.project.transport.playing {
            Color::RGB(248, 240, 132)
        } else {
            Color::RGB(140, 150, 162)
        });
        canvas.fill_rect(playhead)?;

        Ok(())
    }

    fn apply_action(&mut self, action: AppAction) -> AppControl {
        match action {
            AppAction::Quit => AppControl::Quit,
            AppAction::ToggleTimelineFlow => {
                self.timeline_flow = self.timeline_flow.toggle();
                AppControl::Continue
            }
            AppAction::TogglePlayback => {
                self.project.transport.playing = !self.project.transport.playing;
                AppControl::Continue
            }
            AppAction::ToggleLoopEnabled => {
                self.project.transport.loop_enabled = !self.project.transport.loop_enabled;
                AppControl::Continue
            }
            AppAction::SetTimelineFlow(flow) => {
                self.timeline_flow = flow;
                AppControl::Continue
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AppControl {
    Continue,
    Quit,
}

#[cfg(test)]
mod tests {
    use super::{App, AppControl};
    use crate::actions::AppAction;
    use crate::ui::TimelineFlow;

    #[test]
    fn apply_action_toggles_timeline_flow() {
        let mut app = App::new();
        assert_eq!(app.timeline_flow, TimelineFlow::DownwardColumns);

        let control = app.apply_action(AppAction::ToggleTimelineFlow);

        assert_eq!(control, AppControl::Continue);
        assert_eq!(app.timeline_flow, TimelineFlow::AcrossRows);
    }

    #[test]
    fn apply_action_toggles_transport_flags() {
        let mut app = App::new();
        assert!(!app.project.transport.playing);
        assert!(app.project.transport.loop_enabled);

        app.apply_action(AppAction::TogglePlayback);
        app.apply_action(AppAction::ToggleLoopEnabled);

        assert!(app.project.transport.playing);
        assert!(!app.project.transport.loop_enabled);
    }
}
