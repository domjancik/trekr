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
    playhead_ticks: u64,
}

impl App {
    pub fn new() -> Self {
        Self {
            project: Project::demo(),
            engine_config: EngineConfig::default(),
            layout_mode: LayoutMode::FixedFit,
            timeline_flow: TimelineFlow::DownwardColumns,
            keyboard_bindings: KeyboardBindings,
            playhead_ticks: 0,
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
        let mut last_frame_at = started_at;
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

            let now = Instant::now();
            self.advance_playhead(now.saturating_duration_since(last_frame_at));
            last_frame_at = now;

            self.update_window_title(canvas.window_mut())?;
            self.draw(&mut canvas)?;
            std::thread::sleep(Duration::from_millis(16));
        }

        Ok(())
    }

    fn draw(
        &self,
        canvas: &mut sdl3::render::Canvas<sdl3::video::Window>,
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
            track_index,
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
            track_index + 10,
            track.loop_region.start_ticks,
            track.loop_region.length_ticks,
            self.effective_track_playhead(track),
            is_active,
            true,
            track,
        )?;

        Ok(())
    }

    fn draw_track_subcolumn(
        &self,
        canvas: &mut sdl3::render::Canvas<sdl3::video::Window>,
        bounds: Rect,
        accent: Color,
        _seed: usize,
        view_start_ticks: u64,
        range_ticks: u64,
        playhead_ticks: u64,
        is_active: bool,
        detail: bool,
        track: &crate::project::Track,
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

        let header = crate::ui::track_header_rect(bounds, self.timeline_flow);
        canvas.set_draw_color(accent);
        canvas.fill_rect(header)?;

        if track.state.passthrough {
            let rail = crate::ui::passthrough_rail_rect(bounds);
            canvas.set_draw_color(Color::RGB(74, 210, 214));
            canvas.fill_rect(rail)?;
        }

        if !detail && track.state.loop_enabled {
            let loop_highlight = crate::ui::range_highlight_rect(
                bounds,
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

        for guide in crate::ui::timeline_guides(bounds, self.timeline_flow) {
            canvas.set_draw_color(Color::RGB(52, 62, 84));
            canvas.fill_rect(guide)?;
        }

        if detail {
            let loop_tag = crate::ui::detail_badge_rect(header);
            canvas.set_draw_color(
                if track.state.loop_enabled && self.project.transport.loop_enabled {
                    Color::RGB(252, 192, 104)
                } else {
                    Color::RGB(88, 82, 76)
                },
            );
            canvas.fill_rect(loop_tag)?;
        }

        for badge in crate::ui::header_badges(header) {
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

        let note_range = crate::timeline::LoopRegion::new(view_start_ticks, range_ticks.max(1));
        for note in crate::ui::note_rects(bounds, &track.midi_notes, note_range, self.timeline_flow)
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
            bounds,
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

    fn update_window_title(
        &self,
        window: &mut sdl3::video::Window,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let active = self
            .project
            .active_track()
            .expect("demo project always has tracks");
        let title = format!(
            "trekr | T{} {} | Tick:{} | Space Play:{} | [ ] TrackLoop:{}-{} | , . Nudge | Shift+[ ] SongLoop:{}-{} | Shift+, . Nudge | G GlobalLoop:{} | L Loop:{} | A Arm:{} | M Mute:{} | S Solo:{} | I Thru:{}",
            self.project.active_track_index + 1,
            active.name,
            self.playhead_ticks,
            active.loop_region.start_ticks,
            active.loop_region.end_ticks(),
            self.project.loop_region.start_ticks,
            self.project.loop_region.end_ticks(),
            on_off(self.project.transport.playing),
            on_off(active.state.loop_enabled),
            on_off(active.state.armed),
            on_off(active.state.muted),
            on_off(active.state.soloed),
            on_off(active.state.passthrough),
            on_off(self.project.transport.loop_enabled),
        );
        window.set_title(&title)?;
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

    fn effective_track_playhead(&self, track: &crate::project::Track) -> u64 {
        let raw = self.playhead_ticks;
        if !track.state.loop_enabled || track.loop_region.length_ticks == 0 {
            return raw;
        }

        track.loop_region.start_ticks + (raw % track.loop_region.length_ticks)
    }
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
    fn window_title_reports_active_track_state() {
        let mut app = App::new();
        app.apply_action(AppAction::TogglePlayback);
        app.apply_action(AppAction::ToggleCurrentTrackArm);
        app.apply_action(AppAction::ToggleCurrentTrackLoop);

        let title = format!(
            "trekr | T{} {} | Space Play:{} | G GlobalLoop:{} | L Loop:{} | A Arm:{} | M Mute:{} | S Solo:{} | I Thru:{}",
            app.project.active_track_index + 1,
            app.project.active_track().unwrap().name,
            super::on_off(app.project.transport.playing),
            super::on_off(app.project.transport.loop_enabled),
            super::on_off(app.project.active_track().unwrap().state.loop_enabled),
            super::on_off(app.project.active_track().unwrap().state.armed),
            super::on_off(app.project.active_track().unwrap().state.muted),
            super::on_off(app.project.active_track().unwrap().state.soloed),
            super::on_off(app.project.active_track().unwrap().state.passthrough),
        );

        assert!(title.contains("Play:on"));
        assert!(title.contains("Loop:on"));
        assert!(title.contains("Arm:on"));
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
}
