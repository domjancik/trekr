use super::*;

impl App {
    pub(crate) fn draw_timeline_page<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        content_bounds: Rect,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (header_bounds, body_bounds) = crate::ui::split_top_strip(content_bounds, 28, 6)?;
        let (transport_bounds, timeline_bounds) =
            crate::ui::split_top_strip(body_bounds, transport_strip_height(), 8)?;
        let reset_button = self.global_loop_reset_button_rect(header_bounds);
        let focus_button = self.focused_track_view_button_rect(header_bounds);
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
            if self.focused_track_view {
                "Focused track + loop detail"
            } else {
                "Song columns + loop detail"
            },
            Rect::new(header_bounds.x + 212, header_bounds.y + 8, 180, 8),
            1,
            Color::RGB(190, 198, 210),
        )?;
        canvas.set_draw_color(if self.focused_track_view {
            Color::RGB(76, 108, 142)
        } else {
            Color::RGB(66, 76, 96)
        });
        canvas.fill_rect(focus_button)?;
        canvas.set_draw_color(Color::RGB(206, 220, 232));
        canvas.draw_rect(focus_button)?;
        let focus_label = if self.focused_track_view {
            format!("Track T{}", self.project.active_track_index + 1)
        } else {
            "Track All".to_string()
        };
        crate::ui::draw_text_fitted(
            canvas,
            &focus_label,
            Rect::new(
                focus_button.x + 6,
                focus_button.y + 8,
                focus_button.width().saturating_sub(12),
                8,
            ),
            1,
            Color::RGB(248, 244, 236),
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

        for layout in self.visible_timeline_track_layouts(timeline_bounds) {
            let track = &self.project.tracks[layout.track_index];
            let is_active = layout.track_index == self.project.active_track_index;
            self.draw_track_column(canvas, layout, track, is_active)?;
        }

        if self.overlay_state.active == Some(AppOverlay::Discoverability) {
            self.draw_timeline_discoverability_overlay(canvas, content_bounds)?;
        }

        Ok(())
    }

    fn draw_track_column<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        layout: TimelineTrackLayout,
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
            layout.body_full_bounds,
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
            layout.body_detail_bounds,
            detail_accent,
            detail_range.start_ticks,
            detail_range.length_ticks,
            self.effective_track_playhead(track),
            is_active,
            true,
            track,
        )?;
        self.draw_track_fx_bands(canvas, layout, track, is_active)?;
        self.draw_track_status_strip(canvas, layout.status_rect, track, is_active)?;
        self.draw_timeline_context_highlight(canvas, layout, is_active)?;

        Ok(())
    }

    fn draw_timeline_context_highlight<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        layout: TimelineTrackLayout,
        is_active: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if !is_active {
            return Ok(());
        }
        if let Some(rect) = self.timeline_context_indicator_rect_for_layout(layout) {
            canvas.set_draw_color(Color::RGB(244, 232, 146));
            canvas.fill_rect(rect)?;
        }
        Ok(())
    }

    pub(super) fn timeline_context_indicator_rect_for_layout(
        &self,
        layout: TimelineTrackLayout,
    ) -> Option<Rect> {
        let context_rect = layout.fx_rect(self.page_state.selected_timeline_context);
        let indicator_x = context_rect.x.checked_sub(2)?;
        Some(Rect::new(
            indicator_x,
            context_rect.y,
            1,
            context_rect.height(),
        ))
    }

    #[cfg(test)]
    pub(super) fn timeline_context_indicator_rect(
        &self,
        full_bounds: Rect,
        detail_bounds: Rect,
        track: &Track,
    ) -> Option<Rect> {
        let track_index = self
            .project
            .tracks
            .iter()
            .position(|candidate| std::ptr::eq(candidate, track))?;
        let layout = self.timeline_track_layout(track_index, full_bounds, detail_bounds);
        self.timeline_context_indicator_rect_for_layout(layout)
    }

    fn draw_track_status_strip<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        status_rect: Rect,
        track: &Track,
        is_active: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
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
                        "MUT"
                    } else {
                        "M"
                    },
                ),
                crate::ui::TrackIndicatorKind::Solo => (
                    track.state.soloed,
                    Color::RGB(82, 162, 92),
                    Color::RGB(144, 224, 154),
                    if indicator.rect.width() >= 24 {
                        "SOL"
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

    fn draw_transport_strip<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        bounds: Rect,
    ) -> Result<(), Box<dyn std::error::Error>> {
        canvas.set_draw_color(Color::RGB(28, 36, 52));
        canvas.fill_rect(bounds)?;
        canvas.set_draw_color(Color::RGB(88, 96, 120));
        canvas.draw_rect(bounds)?;

        let top_y = bounds.y + 4;
        let bottom_y = bounds.y + 18;
        let chip_height = 10;

        let top_specs = self.transport_top_chip_specs();
        let bottom_specs = self.transport_bottom_chip_specs();
        let link_specs = self.transport_link_chip_specs();
        let status_specs = self.transport_status_chip_specs();
        let right_panel_width = self.transport_right_panel_width(bounds);
        let right_panel = Rect::new(
            bounds.x + bounds.width() as i32 - right_panel_width as i32 - 6,
            bounds.y + 3,
            right_panel_width,
            bounds.height().saturating_sub(6),
        );
        let left_max = right_panel.x - 12;

        let mut cursor_x = bounds.x + 6;
        for spec in &top_specs {
            let width = crate::ui::text_width(&spec.label, 1) + 10;
            let chip = Rect::new(cursor_x, top_y, width, chip_height);
            if chip.x + chip.width() as i32 > left_max {
                break;
            }
            Self::draw_transport_chip(canvas, chip, spec)?;
            cursor_x += chip.width() as i32 + 6;
        }

        cursor_x = bounds.x + 6;
        for spec in &bottom_specs {
            let width = crate::ui::text_width(&spec.label, 1) + 10;
            let chip = Rect::new(cursor_x, bottom_y, width, chip_height);
            if chip.x + chip.width() as i32 > left_max {
                break;
            }
            Self::draw_transport_chip(canvas, chip, spec)?;
            cursor_x += chip.width() as i32 + 6;
        }

        canvas.set_draw_color(Color::RGB(44, 54, 74));
        canvas.fill_rect(right_panel)?;
        canvas.set_draw_color(Color::RGB(86, 96, 114));
        canvas.draw_rect(right_panel)?;
        crate::ui::draw_text_fitted(
            canvas,
            "LINK",
            Rect::new(right_panel.x + 6, right_panel.y + 3, 28, 8),
            1,
            Color::RGB(164, 178, 196),
        )?;
        crate::ui::draw_text_fitted(
            canvas,
            "F6 / SHIFT+F6",
            Rect::new(
                right_panel.x + right_panel.width() as i32 - 86,
                right_panel.y + 3,
                80,
                8,
            ),
            1,
            Color::RGB(126, 138, 156),
        )?;

        cursor_x = right_panel.x + 6;
        let mut truncated_link_row = false;
        for spec in &link_specs {
            let width = crate::ui::text_width(&spec.label, 1) + 10;
            let chip = Rect::new(cursor_x, top_y, width, chip_height);
            if chip.x + chip.width() as i32 > right_panel.x + right_panel.width() as i32 - 6 {
                truncated_link_row = true;
                break;
            }
            Self::draw_transport_chip(canvas, chip, spec)?;
            cursor_x += chip.width() as i32 + 6;
        }
        if truncated_link_row {
            crate::ui::draw_text_fitted(
                canvas,
                "(...)",
                Rect::new(
                    right_panel.x + right_panel.width() as i32 - 32,
                    top_y + 1,
                    28,
                    chip_height.saturating_sub(2),
                ),
                1,
                Color::RGB(194, 204, 220),
            )?;
        }

        cursor_x = right_panel.x + 6;
        let mut truncated_status_row = false;
        for spec in &status_specs {
            let width = crate::ui::text_width(&spec.label, 1) + 10;
            let chip = Rect::new(cursor_x, bottom_y, width, chip_height);
            if chip.x + chip.width() as i32 > right_panel.x + right_panel.width() as i32 - 6 {
                truncated_status_row = true;
                break;
            }
            Self::draw_transport_chip(canvas, chip, spec)?;
            cursor_x += chip.width() as i32 + 6;
        }
        if truncated_status_row {
            crate::ui::draw_text_fitted(
                canvas,
                "(...)",
                Rect::new(
                    right_panel.x + right_panel.width() as i32 - 32,
                    bottom_y + 1,
                    28,
                    chip_height.saturating_sub(2),
                ),
                1,
                Color::RGB(194, 204, 220),
            )?;
        }

        Ok(())
    }

    pub(super) fn global_loop_reset_button_rect(&self, header_bounds: Rect) -> Rect {
        let width = crate::ui::text_width("Reset Song Loop", 1) + 18;
        Rect::new(
            header_bounds.x + header_bounds.width() as i32 - width as i32 - 8,
            header_bounds.y + 4,
            width,
            header_bounds.height().saturating_sub(8),
        )
    }

    pub(super) fn focused_track_view_button_rect(&self, header_bounds: Rect) -> Rect {
        let width = crate::ui::text_width("Track All", 1) + 18;
        Rect::new(
            header_bounds.x + header_bounds.width() as i32 - 240,
            header_bounds.y + 4,
            width.max(78),
            header_bounds.height().saturating_sub(8),
        )
    }

    pub(super) fn transport_chip_actions(&self, bounds: Rect) -> Vec<(Rect, AppAction)> {
        let mut rects = Vec::new();
        let top_y = bounds.y + 4;
        let bottom_y = bounds.y + 18;
        let chip_height = 10;
        let right_panel_width = self.transport_right_panel_width(bounds);
        let right_panel_x = bounds.x + bounds.width() as i32 - right_panel_width as i32 - 6;
        let right_panel_right = right_panel_x + right_panel_width as i32 - 6;
        let left_max = right_panel_x - 12;

        let mut cursor_x = bounds.x + 6;
        for chip_spec in self.transport_top_chip_specs() {
            let width = crate::ui::text_width(&chip_spec.label, 1) + 10;
            let chip = Rect::new(cursor_x, top_y, width, chip_height);
            if chip.x + chip.width() as i32 > left_max {
                break;
            }
            if let Some(action) = chip_spec.action {
                rects.push((chip, action));
            }
            cursor_x += chip.width() as i32 + 6;
        }

        cursor_x = bounds.x + 6;
        for chip_spec in self.transport_bottom_chip_specs() {
            let width = crate::ui::text_width(&chip_spec.label, 1) + 10;
            let chip = Rect::new(cursor_x, bottom_y, width, chip_height);
            if chip.x + chip.width() as i32 > left_max {
                break;
            }
            if let Some(action) = chip_spec.action {
                rects.push((chip, action));
            }
            cursor_x += chip.width() as i32 + 6;
        }

        cursor_x = right_panel_x + 6;
        for chip_spec in self.transport_link_chip_specs() {
            let width = crate::ui::text_width(&chip_spec.label, 1) + 10;
            let chip = Rect::new(cursor_x, top_y, width, chip_height);
            if chip.x + chip.width() as i32 > right_panel_right {
                break;
            }
            if let Some(action) = chip_spec.action {
                rects.push((chip, action));
            }
            cursor_x += chip.width() as i32 + 6;
        }

        cursor_x = right_panel_x + 6;
        for chip_spec in self.transport_status_chip_specs() {
            let width = crate::ui::text_width(&chip_spec.label, 1) + 10;
            let chip = Rect::new(cursor_x, bottom_y, width, chip_height);
            if chip.x + chip.width() as i32 > right_panel_right {
                break;
            }
            if let Some(action) = chip_spec.action {
                rects.push((chip, action));
            }
            cursor_x += chip.width() as i32 + 6;
        }

        rects
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::actions::AppAction;

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
}
