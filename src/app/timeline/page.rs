use super::*;

impl App {
    pub(crate) fn draw_timeline_page<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        content_bounds: Rect,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let theme = self.theme();
        let (header_bounds, transport_bounds, timeline_bounds) =
            self.timeline_page_layout(content_bounds)?;
        let reset_button = self.global_loop_reset_button_rect(header_bounds);
        let focus_button = self.focused_track_view_button_rect(header_bounds);
        canvas.set_draw_color(theme.app_chrome.surface_fill);
        canvas.fill_rect(header_bounds)?;
        canvas.set_draw_color(theme.app_chrome.surface_border);
        canvas.draw_rect(header_bounds)?;
        crate::ui::draw_text_fitted(
            canvas,
            "Timeline",
            Rect::new(header_bounds.x + 8, header_bounds.y + 8, 84, 8),
            1,
            theme.app_chrome.action_text,
        )?;
        crate::ui::draw_text_fitted(
            canvas,
            "Vertical",
            Rect::new(header_bounds.x + 96, header_bounds.y + 8, 54, 8),
            1,
            theme.app_chrome.detail_text,
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
            theme.app_chrome.detail_text,
        )?;
        canvas.set_draw_color(if self.focused_track_view {
            theme.app_chrome.tab_active_fill
        } else {
            theme.app_chrome.tab_inactive_fill
        });
        let focus_fill = if self.focused_track_view {
            theme.app_chrome.tab_active_fill
        } else {
            theme.app_chrome.tab_inactive_fill
        };
        canvas.set_draw_color(focus_fill);
        canvas.fill_rect(focus_button)?;
        canvas.set_draw_color(theme.app_chrome.surface_border);
        canvas.draw_rect(focus_button)?;
        let focus_label = if self.focused_track_view {
            format!("Track T{}", self.project.active_track_index + 1)
        } else {
            "Track All".to_string()
        };
        crate::ui::draw_text_fitted(
            canvas,
            &focus_label,
            crate::app::support::ui_helpers::horizontally_center_text_rect(
                &focus_label,
                crate::app::support::ui_helpers::chrome_compact_text_rect(focus_button),
                1,
            ),
            1,
            contrasting_text_color(focus_fill, theme),
        )?;
        let reset_fill = theme.transport.song_loop;
        canvas.set_draw_color(reset_fill);
        canvas.fill_rect(reset_button)?;
        canvas.set_draw_color(theme.app_chrome.surface_border);
        canvas.draw_rect(reset_button)?;
        crate::ui::draw_text_fitted(
            canvas,
            "Reset Song Loop",
            crate::app::support::ui_helpers::horizontally_center_text_rect(
                "Reset Song Loop",
                crate::app::support::ui_helpers::chrome_compact_text_rect(reset_button),
                1,
            ),
            1,
            contrasting_text_color(reset_fill, theme),
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
        let theme = self.theme();
        let full_accent = if track.state.armed {
            theme.transport.record_active
        } else if is_active {
            theme.app_chrome.tab_accent_timeline
        } else {
            theme.app_chrome.tab_inactive_fill
        };
        let detail_accent = if detail_range != track.loop_region {
            theme.app_chrome.tab_accent_mappings
        } else if track.state.loop_enabled && self.project.transport.loop_enabled {
            theme.transport.song_loop
        } else if is_active {
            theme.app_chrome.footer_chip_mappings
        } else {
            theme.app_chrome.footer_chip_inactive
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
            canvas.set_draw_color(self.theme().mappings.page_title);
            canvas.fill_rect(rect)?;
        }
        Ok(())
    }

    pub(crate) fn timeline_context_indicator_rect_for_layout(
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
    pub(crate) fn timeline_context_indicator_rect(
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
        let theme = self.theme();
        canvas.set_draw_color(theme.app_chrome.surface_fill);
        canvas.fill_rect(status_rect)?;
        canvas.set_draw_color(if is_active {
            theme.app_chrome.surface_border
        } else {
            theme.app_chrome.overlay_row_idle_border
        });
        canvas.draw_rect(status_rect)?;

        for indicator in crate::ui::track_indicators(status_rect, self.ui_metrics()) {
            let (enabled, fill, border, label) = match indicator.kind {
                crate::ui::TrackIndicatorKind::Armed => (
                    track.state.armed,
                    theme.transport.record_active,
                    theme.transport.record_active,
                    if indicator.rect.width() >= 24 {
                        "ARM"
                    } else {
                        "A"
                    },
                ),
                crate::ui::TrackIndicatorKind::Recording => (
                    track.active_take.is_some(),
                    theme.transport.record_active,
                    theme.transport.record_active,
                    if indicator.rect.width() >= 24 {
                        "REC"
                    } else {
                        "R"
                    },
                ),
                crate::ui::TrackIndicatorKind::Muted => (
                    track.state.muted,
                    if theme.preset == ThemePreset::HighContrastLight {
                        Color::RGB(96, 96, 96)
                    } else {
                        theme.app_chrome.footer_chip_inactive
                    },
                    theme.app_chrome.surface_border,
                    if indicator.rect.width() >= 24 {
                        "MUT"
                    } else {
                        "M"
                    },
                ),
                crate::ui::TrackIndicatorKind::Solo => (
                    track.state.soloed,
                    theme.transport.play_active,
                    theme.transport.play_active,
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
                theme.app_chrome.tab_inactive_fill
            } else {
                theme.app_chrome.footer_chip_inactive
            });
            canvas.fill_rect(indicator.rect)?;
            canvas.set_draw_color(if enabled {
                border
            } else {
                theme.app_chrome.overlay_row_idle_border
            });
            canvas.draw_rect(indicator.rect)?;
            crate::ui::draw_text_fitted(
                canvas,
                label,
                crate::app::support::ui_helpers::chrome_compact_text_rect(indicator.rect),
                1,
                if enabled {
                    contrasting_text_color(fill, theme)
                } else {
                    theme.app_chrome.detail_text
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
        let theme = self.theme();
        canvas.set_draw_color(theme.app_chrome.surface_fill);
        canvas.fill_rect(bounds)?;
        canvas.set_draw_color(theme.app_chrome.surface_border);
        canvas.draw_rect(bounds)?;

        let top_specs = self.transport_top_chip_specs();
        let bottom_specs = self.transport_bottom_chip_specs();
        let link_specs = self.transport_link_chip_specs();
        let status_specs = self.transport_status_chip_specs();
        let layout = self.transport_strip_layout(bounds);
        let top_y = layout.top_y;
        let bottom_y = layout.bottom_y;
        let right_top_y = layout.right_top_y;
        let chip_height = layout.chip_height;
        let right_panel = layout.right_panel;
        let left_max = right_panel.x - 12;

        let mut cursor_x = bounds.x + 6;
        for spec in &top_specs {
            let width = crate::ui::text_width(&spec.label, 1) + 10;
            let chip = Rect::new(cursor_x, top_y, width, chip_height);
            if chip.x + chip.width() as i32 > left_max {
                break;
            }
            self.draw_transport_chip(canvas, chip, spec)?;
            cursor_x += chip.width() as i32 + 6;
        }

        cursor_x = bounds.x + 6;
        for spec in &bottom_specs {
            let width = crate::ui::text_width(&spec.label, 1) + 10;
            let chip = Rect::new(cursor_x, bottom_y, width, chip_height);
            if chip.x + chip.width() as i32 > left_max {
                break;
            }
            self.draw_transport_chip(canvas, chip, spec)?;
            cursor_x += chip.width() as i32 + 6;
        }

        canvas.set_draw_color(theme.app_chrome.overlay_panel_fill);
        canvas.fill_rect(right_panel)?;
        canvas.set_draw_color(theme.app_chrome.surface_border);
        canvas.draw_rect(right_panel)?;
        if !layout.compact {
            crate::ui::draw_text_fitted(
                canvas,
                "LINK",
                Rect::new(right_panel.x + 6, layout.right_header_y, 28, 8),
                1,
                theme.app_chrome.detail_text,
            )?;
            crate::ui::draw_text_fitted(
                canvas,
                "F6 / SHIFT+F6",
                Rect::new(
                    right_panel.x + right_panel.width() as i32 - 86,
                    layout.right_header_y,
                    80,
                    8,
                ),
                1,
                theme.app_chrome.detail_text,
            )?;
        }

        cursor_x = right_panel.x + 6;
        let mut truncated_link_row = false;
        for spec in &link_specs {
            let width = crate::ui::text_width(&spec.label, 1) + 10;
            let chip = Rect::new(cursor_x, right_top_y, width, chip_height);
            if chip.x + chip.width() as i32 > right_panel.x + right_panel.width() as i32 - 6 {
                truncated_link_row = true;
                break;
            }
            self.draw_transport_chip(canvas, chip, spec)?;
            cursor_x += chip.width() as i32 + 6;
        }
        if truncated_link_row {
            crate::ui::draw_text_fitted(
                canvas,
                "(...)",
                Rect::new(
                    right_panel.x + right_panel.width() as i32 - 32,
                    right_top_y + 1,
                    28,
                    chip_height.saturating_sub(2),
                ),
                1,
                theme.app_chrome.detail_text,
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
            self.draw_transport_chip(canvas, chip, spec)?;
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
                theme.app_chrome.detail_text,
            )?;
        }

        Ok(())
    }

    pub(crate) fn global_loop_reset_button_rect(&self, header_bounds: Rect) -> Rect {
        let width = crate::ui::text_width("Reset Song Loop", 1) + 18;
        Rect::new(
            header_bounds.x + header_bounds.width() as i32 - width as i32 - 8,
            header_bounds.y + 3,
            width,
            header_bounds.height().saturating_sub(6),
        )
    }

    pub(crate) fn focused_track_view_button_rect(&self, header_bounds: Rect) -> Rect {
        let width = crate::ui::text_width("Track All", 1) + 18;
        Rect::new(
            header_bounds.x + header_bounds.width() as i32 - 240,
            header_bounds.y + 3,
            width.max(78),
            header_bounds.height().saturating_sub(6),
        )
    }

    pub(crate) fn transport_chip_actions(&self, bounds: Rect) -> Vec<(Rect, AppAction)> {
        let mut rects = Vec::new();
        let layout = self.transport_strip_layout(bounds);
        let top_y = layout.top_y;
        let bottom_y = layout.bottom_y;
        let right_top_y = layout.right_top_y;
        let chip_height = layout.chip_height;
        let right_panel_x = layout.right_panel.x;
        let right_panel_right = layout.right_panel.x + layout.right_panel.width() as i32 - 6;
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
            let chip = Rect::new(cursor_x, right_top_y, width, chip_height);
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

    fn transport_strip_layout(&self, bounds: Rect) -> TransportStripLayout {
        let compact = bounds.height() < 30;
        let chip_height = if compact { 8 } else { 11 };
        let top_y = bounds.y + 4;
        let bottom_y = bounds.y + bounds.height() as i32 - chip_height as i32 - 4;
        let right_panel_width = self.transport_right_panel_width(bounds);
        let right_panel = Rect::new(
            bounds.x + bounds.width() as i32 - right_panel_width as i32 - 6,
            bounds.y + 3,
            right_panel_width,
            bounds.height().saturating_sub(6),
        );
        let right_header_y = if compact {
            right_panel.y + 2
        } else {
            right_panel.y + 4
        };
        let right_top_y = if compact { top_y } else { bounds.y + 6 };
        TransportStripLayout {
            top_y,
            bottom_y,
            right_top_y,
            right_header_y,
            chip_height,
            right_panel,
            compact,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct TransportStripLayout {
    top_y: i32,
    bottom_y: i32,
    right_top_y: i32,
    right_header_y: i32,
    chip_height: u32,
    right_panel: Rect,
    compact: bool,
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
        let (_, _, timeline_bounds) = app
            .timeline_page_layout(content_bounds)
            .expect("timeline content");
        let columns = crate::ui::track_column_pairs(
            timeline_bounds,
            app.project.tracks.len(),
            app.ui_metrics(),
        );
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
    fn tiny_density_transport_actions_stay_inside_strip_bounds() {
        let mut app = App::new();
        app.set_ui_density_preset(crate::ui_density::UiDensityPreset::Tiny);
        let bounds = Rect::new(40, 40, 1200, transport_strip_height(app.ui_metrics()));

        for (rect, _) in app.transport_chip_actions(bounds) {
            assert!(rect.x >= bounds.x);
            assert!(rect.y >= bounds.y);
            assert!(rect.x + rect.width() as i32 <= bounds.x + bounds.width() as i32);
            assert!(rect.y + rect.height() as i32 <= bounds.y + bounds.height() as i32);
        }
    }
}
