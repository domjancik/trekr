use crate::actions::{AppAction, KeyboardBindings};
use crate::engine::EngineConfig;
use crate::project::Project;
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
        format!(
            "trekr bootstrap: project='{}', tracks={}, active_track={}, layout={:?}, sample_rate={}, song_ticks={}, playing={}, loop_enabled={}",
            self.project.name,
            self.project.tracks.len(),
            self.project.active_track_index + 1,
            self.layout_mode,
            self.engine_config.sample_rate_hz,
            self.project.full_song_range().length_ticks,
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
        let surface = crate::ui::surface_rect(width, height);
        let columns = crate::ui::track_column_pairs(
            crate::ui::inset_rect(surface, 24, 24)?,
            self.project.tracks.len(),
        );

        canvas.set_draw_color(Color::RGB(18, 24, 38));
        canvas.clear();

        canvas.set_draw_color(Color::RGB(28, 34, 50));
        canvas.fill_rect(surface)?;
        canvas.set_draw_color(Color::RGB(88, 96, 120));
        canvas.draw_rect(surface)?;

        for (index, track) in self.project.tracks.iter().enumerate() {
            if let Some((full_bounds, detail_bounds)) = columns.get(index).copied() {
                let is_active = index == self.project.active_track_index;
                self.draw_track_column(
                    canvas,
                    full_bounds,
                    detail_bounds,
                    index,
                    track,
                    elapsed,
                    is_active,
                )?;
            }
        }
        let _ = canvas.present();

        Ok(())
    }

    fn draw_track_column(
        &self,
        canvas: &mut sdl3::render::Canvas<sdl3::video::Window>,
        full_bounds: Rect,
        detail_bounds: Rect,
        track_index: usize,
        track: &crate::project::Track,
        elapsed: Duration,
        is_active: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let full_accent = if is_active {
            Color::RGB(42, 90, 168)
        } else {
            Color::RGB(36, 58, 92)
        };
        let detail_accent = if is_active {
            Color::RGB(162, 88, 34)
        } else {
            Color::RGB(92, 58, 36)
        };

        self.draw_track_subcolumn(
            canvas,
            full_bounds,
            full_accent,
            track_index,
            track
                .loop_region
                .end_ticks()
                .max(self.project.full_song_range().end_ticks()),
            elapsed,
            is_active,
            false,
        )?;
        self.draw_track_subcolumn(
            canvas,
            detail_bounds,
            detail_accent,
            track_index + 10,
            track.loop_region.length_ticks,
            elapsed,
            is_active,
            true,
        )?;

        Ok(())
    }

    fn draw_track_subcolumn(
        &self,
        canvas: &mut sdl3::render::Canvas<sdl3::video::Window>,
        bounds: Rect,
        accent: Color,
        seed: usize,
        range_ticks: u64,
        elapsed: Duration,
        is_active: bool,
        detail: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        canvas.set_draw_color(Color::RGB(20, 27, 40));
        canvas.fill_rect(bounds)?;
        canvas.set_draw_color(if is_active {
            Color::RGB(240, 222, 116)
        } else {
            Color::RGB(88, 96, 120)
        });
        canvas.draw_rect(bounds)?;

        let header = crate::ui::track_header_rect(bounds, self.timeline_flow);
        canvas.set_draw_color(accent);
        canvas.fill_rect(header)?;

        for guide in crate::ui::timeline_guides(bounds, self.timeline_flow) {
            canvas.set_draw_color(Color::RGB(52, 62, 84));
            canvas.fill_rect(guide)?;
        }

        if detail {
            let loop_tag = crate::ui::detail_badge_rect(header);
            canvas.set_draw_color(Color::RGB(252, 192, 104));
            canvas.fill_rect(loop_tag)?;
        }

        for block in crate::ui::region_blocks(bounds, seed, self.timeline_flow) {
            canvas.set_draw_color(Color::RGB(210, 222, 236));
            canvas.fill_rect(block)?;
            canvas.set_draw_color(Color::RGB(245, 247, 250));
            canvas.draw_rect(block)?;
        }

        let playhead = crate::ui::playhead_rect(
            bounds,
            self.timeline_flow,
            range_ticks.max(1),
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
            AppAction::TogglePlayback => {
                self.project.transport.playing = !self.project.transport.playing;
                AppControl::Continue
            }
            AppAction::ToggleGlobalLoop => {
                self.project.transport.loop_enabled = !self.project.transport.loop_enabled;
                AppControl::Continue
            }
            AppAction::ToggleCurrentTrackLoop => {
                if let Some(track) = self.project.active_track_mut() {
                    track.state.loop_enabled = !track.state.loop_enabled;
                }
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
    fn app_still_supports_absolute_flow_override() {
        let mut app = App::new();
        let control = app.apply_action(AppAction::SetTimelineFlow(TimelineFlow::AcrossRows));

        assert_eq!(control, AppControl::Continue);
        assert_eq!(app.timeline_flow, TimelineFlow::AcrossRows);
    }
}
