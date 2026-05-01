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

        let left_specs = self.transport_left_button_specs();
        let right_specs = self.transport_right_button_specs();
        let layout = self.transport_strip_layout(bounds);
        for (index, spec) in left_specs.iter().enumerate() {
            if let Some(chip) = layout.left_button_rect(index, left_specs.len()) {
                self.draw_transport_chip(canvas, chip, spec)?;
            }
        }
        for (index, spec) in right_specs.iter().enumerate() {
            if let Some(chip) = layout.right_button_rect(index, right_specs.len()) {
                self.draw_transport_chip(canvas, chip, spec)?;
            }
        }
        self.draw_tempo_pad(canvas, layout.tempo_pad_bounds)?;
        self.draw_transport_status(canvas, layout.status_text_bounds)?;

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
        let left_specs = self.transport_left_button_specs();
        let left_count = left_specs.len();
        for (index, chip_spec) in left_specs.into_iter().enumerate() {
            if let (Some(chip), Some(action)) =
                (layout.left_button_rect(index, left_count), chip_spec.action)
            {
                rects.push((chip, action));
            }
        }
        let right_specs = self.transport_right_button_specs();
        let right_count = right_specs.len();
        for (index, chip_spec) in right_specs.into_iter().enumerate() {
            if let (Some(chip), Some(action)) = (
                layout.right_button_rect(index, right_count),
                chip_spec.action,
            ) {
                rects.push((chip, action));
            }
        }
        rects.extend(self.tempo_pad_actions(layout.tempo_pad_bounds));

        rects
    }

    fn transport_strip_layout(&self, bounds: Rect) -> TransportStripLayout {
        let right_specs = self.transport_right_button_specs();
        let right_gap = 2;
        let side_inset = 3;
        let status_width = self.transport_status_width();
        let tempo_pad_width = 98_u32;
        let right_buttons_width = button_row_width(right_specs.len(), 74, right_gap) as u32;
        let right_group_width = status_width
            .saturating_add(right_buttons_width)
            .saturating_add(tempo_pad_width)
            .saturating_add((right_gap * 2) as u32);
        let right_group_bounds = Rect::new(
            bounds.x + bounds.width() as i32 - right_group_width as i32 - side_inset,
            bounds.y + 3,
            right_group_width,
            bounds.height().saturating_sub(6),
        );
        let tempo_pad_bounds = Rect::new(
            right_group_bounds.x + right_group_bounds.width() as i32 - tempo_pad_width as i32,
            right_group_bounds.y,
            tempo_pad_width,
            right_group_bounds.height(),
        );
        let right_buttons_bounds = Rect::new(
            tempo_pad_bounds.x - right_gap - right_buttons_width as i32,
            right_group_bounds.y,
            right_buttons_width,
            right_group_bounds.height(),
        );
        let status_text_bounds = Rect::new(
            right_group_bounds.x,
            right_group_bounds.y,
            status_width,
            right_group_bounds.height(),
        );
        let buttons_bounds = Rect::new(
            bounds.x + 3,
            bounds.y + 3,
            right_group_bounds.x.saturating_sub(bounds.x + 5) as u32,
            bounds.height().saturating_sub(6),
        );
        TransportStripLayout {
            left_buttons_bounds: buttons_bounds,
            right_buttons_bounds,
            tempo_pad_bounds,
            status_text_bounds,
        }
    }

    fn transport_status_width(&self) -> u32 {
        let (quantize, peers) = self.transport_status_lines();
        crate::ui::text_width(&quantize, 1)
            .max(crate::ui::text_width(&peers, 1))
            .saturating_add(2)
    }

    fn transport_status_lines(&self) -> (String, String) {
        let quantize = format!("Q {}", quantize_label(self.project.transport.quantize));
        let peers_label = if self.ui_density_preset == crate::ui_density::UiDensityPreset::Tiny {
            "Prs"
        } else {
            "Peers"
        };
        let peers = format!("{} {}", peers_label, self.link_snapshot.peers);
        (quantize, peers)
    }

    fn draw_transport_status<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        bounds: Rect,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let theme = self.theme();
        let (quantize, peers) = self.transport_status_lines();
        let quantize_width = crate::ui::text_width(&quantize, 1).min(bounds.width());
        let peers_width = crate::ui::text_width(&peers, 1).min(bounds.width());
        let top_y = bounds.y + if bounds.height() <= 26 { 3 } else { 6 };
        let bottom_y =
            bounds.y + bounds.height() as i32 - if bounds.height() <= 26 { 11 } else { 14 };
        crate::ui::draw_text_fitted(
            canvas,
            &quantize,
            Rect::new(
                bounds.x + bounds.width() as i32 - quantize_width as i32 - 1,
                top_y,
                quantize_width,
                8,
            ),
            1,
            theme.app_chrome.detail_text,
        )?;
        crate::ui::draw_text_fitted(
            canvas,
            &peers,
            Rect::new(
                bounds.x + bounds.width() as i32 - peers_width as i32 - 1,
                bottom_y,
                peers_width,
                8,
            ),
            1,
            theme.app_chrome.detail_text,
        )?;
        Ok(())
    }

    fn draw_tempo_pad<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        bounds: Rect,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let theme = self.theme();
        canvas.set_draw_color(theme.transport.tempo);
        canvas.fill_rect(bounds)?;
        canvas.set_draw_color(theme.app_chrome.surface_border);
        canvas.draw_rect(bounds)?;

        let layout = TempoPadLayout::new(bounds);
        for (rect, label) in [
            (layout.top_left, "-"),
            (layout.top_right, "+"),
            (layout.bottom_left, "/"),
            (layout.bottom_right, "*"),
        ] {
            canvas.set_draw_color(theme.app_chrome.surface_border);
            canvas.draw_rect(rect)?;
            crate::ui::draw_text_fitted(
                canvas,
                label,
                crate::app::support::ui_helpers::horizontally_center_text_rect(
                    label,
                    crate::app::support::ui_helpers::chrome_compact_text_rect(rect),
                    1,
                ),
                1,
                contrasting_text_color(theme.transport.tempo, theme),
            )?;
        }

        canvas.set_draw_color(theme.app_chrome.surface_border);
        canvas.draw_rect(layout.center)?;
        crate::ui::draw_text_fitted(
            canvas,
            &self.project.transport.tempo_bpm.to_string(),
            crate::app::support::ui_helpers::horizontally_center_text_rect(
                &self.project.transport.tempo_bpm.to_string(),
                Rect::new(
                    layout.center.x + 2,
                    layout.center.y + 5,
                    layout.center.width().saturating_sub(4),
                    8,
                ),
                1,
            ),
            1,
            contrasting_text_color(theme.transport.tempo, theme),
        )?;
        crate::ui::draw_text_fitted(
            canvas,
            "TAP",
            crate::app::support::ui_helpers::horizontally_center_text_rect(
                "TAP",
                Rect::new(
                    layout.center.x + 2,
                    layout.center.y + layout.center.height() as i32 - 12,
                    layout.center.width().saturating_sub(4),
                    8,
                ),
                1,
            ),
            1,
            contrasting_text_color(theme.transport.tempo, theme),
        )?;
        Ok(())
    }

    fn tempo_pad_actions(&self, bounds: Rect) -> Vec<(Rect, AppAction)> {
        let layout = TempoPadLayout::new(bounds);
        vec![
            (layout.top_left, AppAction::DecreaseTempo),
            (layout.top_right, AppAction::IncreaseTempo),
            (layout.bottom_left, AppAction::HalfTempo),
            (layout.bottom_right, AppAction::DoubleTempo),
            (layout.center, AppAction::TapTempo),
        ]
    }
}

#[derive(Debug, Clone, Copy)]
struct TransportStripLayout {
    left_buttons_bounds: Rect,
    right_buttons_bounds: Rect,
    tempo_pad_bounds: Rect,
    status_text_bounds: Rect,
}

#[derive(Debug, Clone, Copy)]
struct TempoPadLayout {
    top_left: Rect,
    top_right: Rect,
    bottom_left: Rect,
    bottom_right: Rect,
    center: Rect,
}

impl TempoPadLayout {
    fn new(bounds: Rect) -> Self {
        let cell_width = (bounds.width() / 3).max(12);
        let cell_height = (bounds.height() / 2).max(12);
        let left_x = bounds.x;
        let center_x = bounds.x + cell_width as i32;
        let right_x = bounds.x + bounds.width() as i32 - cell_width as i32;
        let top_y = bounds.y;
        let bottom_y = bounds.y + bounds.height() as i32 - cell_height as i32;
        Self {
            top_left: Rect::new(left_x, top_y, cell_width, cell_height),
            top_right: Rect::new(right_x, top_y, cell_width, cell_height),
            bottom_left: Rect::new(left_x, bottom_y, cell_width, cell_height),
            bottom_right: Rect::new(right_x, bottom_y, cell_width, cell_height),
            center: Rect::new(
                center_x,
                bounds.y,
                bounds.width().saturating_sub(cell_width.saturating_mul(2)),
                bounds.height(),
            ),
        }
    }
}

impl TransportStripLayout {
    fn left_button_rect(self, index: usize, count: usize) -> Option<Rect> {
        row_button_rect(self.left_buttons_bounds, index, count, 74, 2, false)
    }

    fn right_button_rect(self, index: usize, count: usize) -> Option<Rect> {
        row_button_rect(self.right_buttons_bounds, index, count, 74, 2, true)
    }
}

fn button_row_width(count: usize, max_button_width: i32, gap: i32) -> i32 {
    if count == 0 {
        return 0;
    }
    (max_button_width * count as i32) + gap * count.saturating_sub(1) as i32
}

fn row_button_rect(
    bounds: Rect,
    index: usize,
    count: usize,
    max_button_width: i32,
    gap: i32,
    right_align: bool,
) -> Option<Rect> {
    if count == 0 || index >= count || bounds.width() == 0 {
        return None;
    }
    let total_gap = gap * count.saturating_sub(1) as i32;
    let available_width = bounds.width() as i32 - total_gap;
    if available_width <= 0 {
        return None;
    }
    let button_width = (available_width / count as i32)
        .min(max_button_width)
        .max(8);
    let used_width = button_width * count as i32 + total_gap;
    let start_x = if right_align {
        bounds.x + bounds.width() as i32 - used_width
    } else {
        bounds.x
    };
    Some(Rect::new(
        start_x + index as i32 * (button_width + gap),
        bounds.y,
        button_width as u32,
        bounds.height(),
    ))
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
            .transport_left_button_specs()
            .into_iter()
            .map(|chip| (chip.label, chip.sublabel.unwrap_or_default()))
            .collect::<Vec<_>>();
        assert!(
            labels
                .iter()
                .any(|(label, value)| label == "Rec Wrap" && value == "Ext")
        );
        assert!(
            labels
                .iter()
                .any(|(label, value)| label == "Harmony" && value == "C")
        );

        app.apply_action(AppAction::ToggleLoopRecordingExtension);
        let labels = app
            .transport_left_button_specs()
            .into_iter()
            .map(|chip| (chip.label, chip.sublabel.unwrap_or_default()))
            .collect::<Vec<_>>();
        assert!(
            labels
                .iter()
                .any(|(label, value)| label == "Rec Wrap" && value == "Clamp")
        );
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

    #[test]
    fn tiny_density_transport_status_lines_do_not_overlap() {
        let mut app = App::new();
        app.set_ui_density_preset(crate::ui_density::UiDensityPreset::Tiny);
        let bounds = Rect::new(0, 0, 40, 26);
        let top_y = bounds.y + if bounds.height() <= 26 { 3 } else { 6 };
        let bottom_y =
            bounds.y + bounds.height() as i32 - if bounds.height() <= 26 { 11 } else { 14 };

        assert!(top_y + 8 <= bottom_y);
        let (_, peers) = app.transport_status_lines();
        assert!(peers.starts_with("Prs "));
    }

    #[test]
    fn wide_transport_layout_caps_button_width() {
        let layout = TransportStripLayout {
            left_buttons_bounds: Rect::new(40, 40, 900, 36),
            right_buttons_bounds: Rect::new(950, 40, 150, 36),
            tempo_pad_bounds: Rect::new(1446, 40, 98, 36),
            status_text_bounds: Rect::new(1550, 40, 84, 36),
        };

        let first = layout.left_button_rect(0, 10).expect("first button");
        let last = layout.left_button_rect(9, 10).expect("last button");

        assert_eq!(first.width(), 74);
        assert_eq!(last.width(), 74);
        assert_eq!(first.x, layout.left_buttons_bounds.x);
        assert!(
            last.x + (last.width() as i32)
                < layout.left_buttons_bounds.x + layout.left_buttons_bounds.width() as i32
        );
    }
}
