use super::*;

impl App {
    pub(crate) fn handle_clip_align_keyboard_event(
        &mut self,
        event: &sdl3::event::Event,
    ) -> Option<AppControl> {
        if self.clip_align_session.is_some() {
            match event {
                sdl3::event::Event::KeyDown {
                    keycode: Some(sdl3::keyboard::Keycode::Escape),
                    repeat: false,
                    ..
                } => {
                    return Some(self.apply_action_with_source(
                        AppAction::CloseRecordingClipAlign,
                        crate::actions::ActionSource::Keyboard,
                    ));
                }
                sdl3::event::Event::KeyDown {
                    keycode: Some(sdl3::keyboard::Keycode::Return),
                    repeat: false,
                    ..
                } => {
                    return Some(self.apply_action_with_source(
                        AppAction::ApplyRecordingClipAlign,
                        crate::actions::ActionSource::Keyboard,
                    ));
                }
                sdl3::event::Event::KeyDown {
                    keycode: Some(sdl3::keyboard::Keycode::Left),
                    keymod,
                    repeat: false,
                    ..
                } if keymod.intersects(
                    sdl3::keyboard::Mod::LSHIFTMOD | sdl3::keyboard::Mod::RSHIFTMOD,
                ) =>
                {
                    return Some(self.apply_action_with_source(
                        AppAction::SelectPreviousClipAlignField,
                        crate::actions::ActionSource::Keyboard,
                    ));
                }
                sdl3::event::Event::KeyDown {
                    keycode: Some(sdl3::keyboard::Keycode::Right),
                    keymod,
                    repeat: false,
                    ..
                } if keymod.intersects(
                    sdl3::keyboard::Mod::LSHIFTMOD | sdl3::keyboard::Mod::RSHIFTMOD,
                ) =>
                {
                    return Some(self.apply_action_with_source(
                        AppAction::SelectNextClipAlignField,
                        crate::actions::ActionSource::Keyboard,
                    ));
                }
                sdl3::event::Event::KeyDown {
                    keycode: Some(sdl3::keyboard::Keycode::Q),
                    keymod,
                    repeat: false,
                    ..
                } if !keymod.intersects(
                    sdl3::keyboard::Mod::LSHIFTMOD | sdl3::keyboard::Mod::RSHIFTMOD,
                ) =>
                {
                    return Some(self.apply_action_with_source(
                        AppAction::AdjustClipAlignFieldBackward,
                        crate::actions::ActionSource::Keyboard,
                    ));
                }
                sdl3::event::Event::KeyDown {
                    keycode: Some(sdl3::keyboard::Keycode::E),
                    repeat: false,
                    ..
                } => {
                    return Some(self.apply_action_with_source(
                        AppAction::AdjustClipAlignFieldForward,
                        crate::actions::ActionSource::Keyboard,
                    ));
                }
                _ => {}
            }
        }

        if matches!(
            event,
            sdl3::event::Event::KeyDown {
                keycode: Some(sdl3::keyboard::Keycode::Return),
                keymod,
                repeat: false,
                ..
            } if keymod.intersects(
                sdl3::keyboard::Mod::LSHIFTMOD | sdl3::keyboard::Mod::RSHIFTMOD,
            )
        ) && self.page_state.current_page == AppPage::Timeline
            && self.page_state.selected_timeline_context == TimelineContext::TrackTimeline
            && self
                .project
                .active_track()
                .and_then(|track| track.selected_recording_clip_or_only())
                .is_some()
        {
            return Some(self.apply_action_with_source(
                AppAction::OpenSelectedRecordingClipAlign,
                crate::actions::ActionSource::Keyboard,
            ));
        }

        None
    }

    pub(crate) fn clip_align_panel_rect(&self, content_bounds: Rect) -> Rect {
        Rect::new(content_bounds.x + 12, content_bounds.y + 36, 364, 228)
    }

    pub(crate) fn clip_align_field_rects(&self, panel_rect: Rect) -> Vec<(ClipAlignField, Rect)> {
        ClipAlignField::ALL
            .iter()
            .copied()
            .enumerate()
            .map(|(index, field)| {
                (
                    field,
                    Rect::new(
                        panel_rect.x + 10,
                        panel_rect.y + 28 + index as i32 * 20,
                        panel_rect.width().saturating_sub(20),
                        18,
                    ),
                )
            })
            .collect()
    }

    pub(crate) fn clip_align_button_rects(&self, panel_rect: Rect) -> (Rect, Rect) {
        (
            Rect::new(panel_rect.x + 10, panel_rect.y + 198, 104, 18),
            Rect::new(panel_rect.x + 120, panel_rect.y + 198, 104, 18),
        )
    }

    pub(crate) fn draw_clip_align_panel<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        content_bounds: Rect,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let Some(session) = self.clip_align_session.as_ref() else {
            return Ok(());
        };
        let theme = self.theme();
        let panel_rect = self.clip_align_panel_rect(content_bounds);
        canvas.set_draw_color(theme.app_chrome.overlay_panel_fill);
        canvas.fill_rect(panel_rect)?;
        canvas.set_draw_color(theme.app_chrome.surface_border);
        canvas.draw_rect(panel_rect)?;
        crate::ui::draw_text_fitted(
            canvas,
            "Clip Align",
            Rect::new(panel_rect.x + 10, panel_rect.y + 8, 72, 8),
            1,
            theme.mappings.page_title,
        )?;
        crate::ui::draw_text_fitted(
            canvas,
            &format!("T{} Clip {}", session.track_index + 1, session.clip_id),
            Rect::new(panel_rect.x + 92, panel_rect.y + 8, 188, 8),
            1,
            theme.app_chrome.overlay_header_text,
        )?;

        for (field, rect) in self.clip_align_field_rects(panel_rect) {
            let selected = field == session.selected_field;
            let fill = if selected {
                theme.app_chrome.overlay_row_selected_fill
            } else {
                theme.app_chrome.overlay_row_idle_fill
            };
            canvas.set_draw_color(fill);
            canvas.fill_rect(rect)?;
            let border = if selected {
                theme.mappings.page_title
            } else {
                theme.app_chrome.overlay_row_idle_border
            };
            canvas.set_draw_color(border);
            canvas.draw_rect(rect)?;
            let (label, value) = self.clip_align_field_label_value(field, session);
            let label_rect = Rect::new(rect.x + 6, rect.y + 5, 64, 8);
            let value_rect = Rect::new(rect.x + 76, rect.y + 5, rect.width().saturating_sub(82), 8);
            let field_text = crate::app::support::ui_helpers::contrasting_text_color(fill, theme);
            crate::ui::draw_text_fitted(canvas, &label.to_uppercase(), label_rect, 1, field_text)?;
            canvas.set_draw_color(if selected {
                theme.app_chrome.surface_border
            } else {
                theme.app_chrome.overlay_row_idle_border
            });
            canvas.draw_line(
                (rect.x + 70, rect.y + 3),
                (rect.x + 70, rect.y + rect.height() as i32 - 4),
            )?;
            crate::ui::draw_text_fitted(canvas, &value, value_rect, 1, field_text)?;
        }

        let (preview_title, preview_line) =
            if let Some(reason) = session.preview.blocked_reason.as_ref() {
                ("Status".to_string(), reason.clone())
            } else if session.preview.tempo_locked {
                (
                    format!(
                        "Fit {} -> {} ticks",
                        session.preview.source_length_ticks, session.preview.target_length_ticks
                    ),
                    format!(
                        "Tempo preview {} BPM, locked by Link",
                        session
                            .preview
                            .tempo_preview_bpm
                            .unwrap_or(self.project.transport.tempo_bpm)
                    ),
                )
            } else if let Some(tempo_bpm) = session.preview.tempo_preview_bpm {
                (
                    format!(
                        "Fit {} -> {} ticks",
                        session.preview.source_length_ticks, session.preview.target_length_ticks
                    ),
                    format!("Tempo preview {} BPM", tempo_bpm),
                )
            } else {
                (
                    format!(
                        "Fit {} -> {} ticks",
                        session.preview.source_length_ticks, session.preview.target_length_ticks
                    ),
                    "Tempo unchanged".to_string(),
                )
            };
        let preview_color = if session.preview.blocked_reason.is_some() {
            Color::RGB(240, 172, 172)
        } else if session.preview.tempo_locked {
            Color::RGB(240, 220, 160)
        } else {
            Color::RGB(190, 204, 214)
        };
        let preview_rect = Rect::new(
            panel_rect.x + 10,
            panel_rect.y + 160,
            panel_rect.width().saturating_sub(20),
            28,
        );
        canvas.set_draw_color(theme.app_chrome.surface_fill);
        canvas.fill_rect(preview_rect)?;
        canvas.set_draw_color(theme.app_chrome.overlay_row_idle_border);
        canvas.draw_rect(preview_rect)?;
        crate::ui::draw_text_fitted(
            canvas,
            &preview_title,
            Rect::new(
                preview_rect.x + 6,
                preview_rect.y + 5,
                preview_rect.width().saturating_sub(12),
                8,
            ),
            1,
            theme.app_chrome.overlay_meta_text,
        )?;
        crate::ui::draw_text_fitted(
            canvas,
            &preview_line,
            Rect::new(
                preview_rect.x + 6,
                preview_rect.y + 15,
                preview_rect.width().saturating_sub(12),
                8,
            ),
            1,
            preview_color,
        )?;

        let (apply_rect, cancel_rect) = self.clip_align_button_rects(panel_rect);
        let apply_fill = if session.preview.blocked_reason.is_none() {
            theme.transport.play_active
        } else {
            theme.app_chrome.footer_chip_inactive
        };
        canvas.set_draw_color(apply_fill);
        canvas.fill_rect(apply_rect)?;
        canvas.set_draw_color(theme.app_chrome.surface_border);
        canvas.draw_rect(apply_rect)?;
        crate::ui::draw_text_fitted(
            canvas,
            "Apply",
            Rect::new(
                apply_rect.x + 4,
                apply_rect.y + 4,
                apply_rect.width().saturating_sub(8),
                8,
            ),
            1,
            crate::app::support::ui_helpers::contrasting_text_color(apply_fill, theme),
        )?;

        let cancel_fill = theme.app_chrome.footer_chip_inactive;
        canvas.set_draw_color(cancel_fill);
        canvas.fill_rect(cancel_rect)?;
        canvas.set_draw_color(theme.app_chrome.surface_border);
        canvas.draw_rect(cancel_rect)?;
        crate::ui::draw_text_fitted(
            canvas,
            "Cancel",
            Rect::new(
                cancel_rect.x + 4,
                cancel_rect.y + 4,
                cancel_rect.width().saturating_sub(8),
                8,
            ),
            1,
            crate::app::support::ui_helpers::contrasting_text_color(cancel_fill, theme),
        )?;

        Ok(())
    }

    pub(crate) fn clip_align_field_label_value(
        &self,
        field: ClipAlignField,
        session: &ClipAlignSession,
    ) -> (&'static str, String) {
        match field {
            ClipAlignField::SourceStart => (
                "Start",
                match session.settings.source_start_mode {
                    ClipAlignSourceStartMode::ClipStart => "Clip Start",
                    ClipAlignSourceStartMode::FirstNote => "First Note",
                }
                .to_string(),
            ),
            ClipAlignField::SourceEnd => (
                "End",
                match session.settings.source_end_mode {
                    ClipAlignSourceEndMode::ClipEnd => "Clip End",
                    ClipAlignSourceEndMode::StartOfLastNote => "Start Last Note",
                    ClipAlignSourceEndMode::LastNoteEnd => "Last Note End",
                }
                .to_string(),
            ),
            ClipAlignField::TargetLength => (
                "Length",
                format!("{} bars", session.settings.target_length.bars()),
            ),
            ClipAlignField::Destination => (
                "Dest",
                match session.settings.destination {
                    ClipAlignDestination::TrackLoop => "Track Loop",
                    ClipAlignDestination::SongLoop => "Song Loop",
                }
                .to_string(),
            ),
            ClipAlignField::ApplyMode => (
                "Mode",
                match session.settings.apply_mode {
                    ClipAlignApplyMode::FitOnly => "Fit Only",
                    ClipAlignApplyMode::FitAndTempo => "Fit+Tempo",
                }
                .to_string(),
            ),
            ClipAlignField::LoopEnable => (
                "Loop",
                if session.settings.enable_loop_on_apply {
                    "Enable"
                } else {
                    "Keep"
                }
                .to_string(),
            ),
        }
    }

    pub(crate) fn clip_align_footer_content(&self) -> Option<(String, String)> {
        let session = self.clip_align_session.as_ref()?;
        let detail = if let Some(reason) = session.preview.blocked_reason.as_ref() {
            reason.clone()
        } else if session.preview.tempo_locked {
            format!(
                "{} bars, {} BPM preview, tempo locked",
                session.settings.target_length.bars(),
                session
                    .preview
                    .tempo_preview_bpm
                    .unwrap_or(self.project.transport.tempo_bpm)
            )
        } else if let Some(tempo_bpm) = session.preview.tempo_preview_bpm {
            format!(
                "{} -> {} ticks @ {} BPM",
                session.preview.source_length_ticks, session.preview.target_length_ticks, tempo_bpm
            )
        } else {
            format!(
                "{} -> {} ticks",
                session.preview.source_length_ticks, session.preview.target_length_ticks
            )
        };
        Some(("Clip Align".to_string(), detail))
    }

    pub(crate) fn clip_align_tempo_commit_allowed(&self) -> bool {
        !self.project.transport.link_enabled
    }

    pub(crate) fn clip_align_destination_start_ticks(
        &self,
        track_index: usize,
        destination: ClipAlignDestination,
    ) -> Option<u64> {
        match destination {
            ClipAlignDestination::TrackLoop => self
                .project
                .tracks
                .get(track_index)
                .map(|track| track.loop_region.start_ticks),
            ClipAlignDestination::SongLoop => Some(self.project.loop_region.start_ticks),
        }
    }

    pub(crate) fn open_selected_recording_clip_align(&mut self) {
        let track_index = self.project.active_track_index;
        let Some(track) = self.project.tracks.get(track_index) else {
            return;
        };
        let Some(clip) = track.selected_recording_clip_or_only() else {
            return;
        };
        let mut settings = self.clip_align_defaults;
        if let Some(suggested_target_length) =
            track.suggested_clip_align_target_length(self.project.transport, clip.id, settings)
        {
            settings.target_length = suggested_target_length;
        }
        let Some(destination_start_ticks) =
            self.clip_align_destination_start_ticks(track_index, settings.destination)
        else {
            return;
        };
        let Some(preview) = track.preview_clip_align(
            self.project.transport,
            clip.id,
            settings,
            destination_start_ticks,
            self.clip_align_tempo_commit_allowed(),
        ) else {
            return;
        };
        self.clip_align_session = Some(ClipAlignSession {
            track_index,
            clip_id: clip.id,
            selected_field: ClipAlignField::SourceStart,
            settings,
            preview,
        });
        self.overlay_state.active = None;
    }

    pub(crate) fn close_clip_align(&mut self) {
        self.clip_align_session = None;
    }

    pub(crate) fn refresh_clip_align_preview(&mut self) {
        let Some(session) = self.clip_align_session.as_ref() else {
            return;
        };
        let track_index = session.track_index;
        let clip_id = session.clip_id;
        let settings = session.settings;
        let Some(track) = self.project.tracks.get(track_index) else {
            self.clip_align_session = None;
            return;
        };
        let Some(destination_start_ticks) =
            self.clip_align_destination_start_ticks(track_index, settings.destination)
        else {
            return;
        };
        if let Some(preview) = track.preview_clip_align(
            self.project.transport,
            clip_id,
            settings,
            destination_start_ticks,
            self.clip_align_tempo_commit_allowed(),
        ) {
            if let Some(session) = self.clip_align_session.as_mut() {
                session.preview = preview;
            }
        }
    }

    pub(crate) fn adjust_clip_align_field(&mut self, delta: i32) {
        let Some(session) = self.clip_align_session.as_mut() else {
            return;
        };
        session.settings = match session.selected_field {
            ClipAlignField::SourceStart => ClipAlignSettings {
                source_start_mode: match (session.settings.source_start_mode, delta.signum()) {
                    (ClipAlignSourceStartMode::ClipStart, d) if d > 0 => {
                        ClipAlignSourceStartMode::FirstNote
                    }
                    (ClipAlignSourceStartMode::FirstNote, d) if d < 0 => {
                        ClipAlignSourceStartMode::ClipStart
                    }
                    (mode, _) => mode,
                },
                ..session.settings
            },
            ClipAlignField::SourceEnd => ClipAlignSettings {
                source_end_mode: match (session.settings.source_end_mode, delta.signum()) {
                    (ClipAlignSourceEndMode::ClipEnd, d) if d > 0 => {
                        ClipAlignSourceEndMode::StartOfLastNote
                    }
                    (ClipAlignSourceEndMode::StartOfLastNote, d) if d > 0 => {
                        ClipAlignSourceEndMode::LastNoteEnd
                    }
                    (ClipAlignSourceEndMode::LastNoteEnd, d) if d > 0 => {
                        ClipAlignSourceEndMode::ClipEnd
                    }
                    (ClipAlignSourceEndMode::ClipEnd, d) if d < 0 => {
                        ClipAlignSourceEndMode::LastNoteEnd
                    }
                    (ClipAlignSourceEndMode::StartOfLastNote, d) if d < 0 => {
                        ClipAlignSourceEndMode::ClipEnd
                    }
                    (ClipAlignSourceEndMode::LastNoteEnd, d) if d < 0 => {
                        ClipAlignSourceEndMode::StartOfLastNote
                    }
                    (mode, _) => mode,
                },
                ..session.settings
            },
            ClipAlignField::TargetLength => ClipAlignSettings {
                target_length: if delta < 0 {
                    session.settings.target_length.previous()
                } else {
                    session.settings.target_length.next()
                },
                ..session.settings
            },
            ClipAlignField::Destination => ClipAlignSettings {
                destination: session.settings.destination.toggle(),
                ..session.settings
            },
            ClipAlignField::ApplyMode => ClipAlignSettings {
                apply_mode: session.settings.apply_mode.toggle(),
                ..session.settings
            },
            ClipAlignField::LoopEnable => ClipAlignSettings {
                enable_loop_on_apply: !session.settings.enable_loop_on_apply,
                ..session.settings
            },
        };
        self.refresh_clip_align_preview();
    }

    pub(crate) fn apply_clip_align(&mut self) {
        let Some(session) = self.clip_align_session.clone() else {
            return;
        };
        let Some(destination_start_ticks) = self
            .clip_align_destination_start_ticks(session.track_index, session.settings.destination)
        else {
            return;
        };
        let tempo_commit_allowed = self.clip_align_tempo_commit_allowed();
        let apply_result = {
            let Some(track) = self.project.tracks.get_mut(session.track_index) else {
                self.close_clip_align();
                return;
            };
            track.apply_clip_align(
                self.project.transport,
                session.clip_id,
                session.settings,
                destination_start_ticks,
                tempo_commit_allowed,
            )
        };

        let Ok(result) = apply_result else {
            self.refresh_clip_align_preview();
            return;
        };

        match session.settings.destination {
            ClipAlignDestination::TrackLoop => {
                if let Some(track) = self.project.tracks.get_mut(session.track_index) {
                    track.loop_region.start_ticks = destination_start_ticks;
                    track.loop_region.length_ticks = session.preview.target_length_ticks.max(1);
                    if session.settings.enable_loop_on_apply {
                        track.state.loop_enabled = true;
                    }
                }
            }
            ClipAlignDestination::SongLoop => {
                self.project.loop_region.start_ticks = destination_start_ticks;
                self.project.loop_region.length_ticks = session.preview.target_length_ticks.max(1);
                if session.settings.enable_loop_on_apply {
                    self.project.transport.loop_enabled = true;
                }
            }
        }

        if let Some(tempo_bpm) = result.applied_tempo_bpm {
            self.project.transport.tempo_bpm = tempo_bpm;
            if self.project.transport.link_enabled && !self.midi_runtime.is_enabled() {
                self.link.commit_tempo(f64::from(tempo_bpm));
                self.link_snapshot = self.link.refresh();
            }
        }

        self.project.active_track_index = session.track_index;
        self.sync_active_track_recording_clip_scroll();
        self.clip_align_defaults = session.settings;
        self.clip_align_session = None;
    }

    pub(crate) fn clip_align_track_has_available_clip(&self, track: &Track) -> bool {
        track.selected_recording_clip_or_only().is_some()
    }

    pub(crate) fn handle_clip_align_pointer_down(
        &mut self,
        content_bounds: Rect,
        x: i32,
        y: i32,
        source: crate::actions::ActionSource,
    ) -> Option<AppControl> {
        if self.page_state.current_page != AppPage::Timeline || self.clip_align_session.is_none() {
            return None;
        }

        let panel_rect = self.clip_align_panel_rect(content_bounds);
        if !rect_contains(panel_rect, x, y) {
            return Some(AppControl::Continue);
        }

        let (apply_rect, cancel_rect) = self.clip_align_button_rects(panel_rect);
        if rect_contains(apply_rect, x, y) {
            return Some(self.apply_action_with_source(AppAction::ApplyRecordingClipAlign, source));
        }
        if rect_contains(cancel_rect, x, y) {
            return Some(self.apply_action_with_source(AppAction::CloseRecordingClipAlign, source));
        }

        for (field, rect) in self.clip_align_field_rects(panel_rect) {
            if !rect_contains(rect, x, y) {
                continue;
            }
            if let Some(session) = self.clip_align_session.as_mut() {
                session.selected_field = field;
            }
            if x < rect.x + rect.width() as i32 / 2 {
                self.adjust_clip_align_field(-1);
            } else {
                self.adjust_clip_align_field(1);
            }
            return Some(AppControl::Continue);
        }

        Some(AppControl::Continue)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::actions::AppAction;
    use crate::project::{ClipAlignTargetLength, MidiNote, RecordingClip, RecordingView};
    use crate::timeline::{RecordingTake, Region};

    #[test]
    fn clip_align_action_opens_and_applies_for_selected_clip() {
        let mut app = App::new();
        let transport = app.project.transport;
        {
            let track = app.project.active_track_mut().unwrap();
            track.clear_content();
            track.commit_take(transport, RecordingTake::new(0).release(960), None);
            track.recording_view = RecordingView::Stacked;
        }

        app.apply_action(AppAction::OpenSelectedRecordingClipAlign);
        assert!(app.clip_align_session.is_some());

        app.apply_action(AppAction::AdjustClipAlignFieldForward);
        app.apply_action(AppAction::ApplyRecordingClipAlign);

        assert!(app.clip_align_session.is_none());
        assert_eq!(
            app.project.active_track().unwrap().loop_region.length_ticks,
            3_840
        );
    }

    #[test]
    fn opening_clip_align_uses_tempo_aware_target_length_suggestion() {
        let mut app = App::new();
        let transport = app.project.transport;
        {
            let track = app.project.active_track_mut().unwrap();
            track.clear_content();
            track.recording_clips = vec![RecordingClip {
                id: 1,
                region: Region::new_recorded(0, 3_700, 1),
                muted: false,
                native_start_ticks: 0,
                native_end_ticks: 3_700,
                native_duration_ticks: 3_700,
                native_capture_tempo_bpm: transport.tempo_bpm,
            }];
            track.selected_recording_clip_id = Some(1);
            track.midi_notes = vec![
                MidiNote::new_recorded(60, 0, 120, 100, 1),
                MidiNote::new_recorded(62, 3_700, 120, 100, 1),
            ];
            track.recording_view = RecordingView::Stacked;
        }
        app.clip_align_defaults.target_length = ClipAlignTargetLength::Bar8;

        app.apply_action(AppAction::OpenSelectedRecordingClipAlign);

        assert_eq!(
            app.clip_align_session
                .as_ref()
                .map(|session| session.settings.target_length),
            Some(ClipAlignTargetLength::Bar1)
        );
    }
}
