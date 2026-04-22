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

    pub(super) fn draw_timeline_fx_row<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        context: TimelineContext,
        slot_index: usize,
        slot: &MidiFxSlot,
        layout: TimelineFxRowLayout,
        selected: bool,
        text_color: Color,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let enabled_fill = if slot.enabled {
            Color::RGB(54, 176, 100)
        } else {
            Color::RGB(88, 54, 62)
        };
        let enabled_selected =
            selected && self.page_state.selected_timeline_fx_field == TimelineFxField::Enabled;
        canvas.set_draw_color(if selected {
            Color::RGB(44, 50, 70)
        } else {
            Color::RGB(36, 42, 58)
        });
        canvas.fill_rect(layout.row)?;
        canvas.set_draw_color(enabled_fill);
        canvas.fill_rect(layout.enabled)?;
        canvas.set_draw_color(if enabled_selected {
            Color::RGB(252, 236, 156)
        } else if slot.enabled {
            Color::RGB(210, 248, 214)
        } else {
            Color::RGB(196, 142, 154)
        });
        canvas.draw_rect(layout.enabled)?;
        if layout.enabled.width() > 4 && layout.enabled.height() > 4 {
            canvas.set_draw_color(if slot.enabled {
                Color::RGB(32, 108, 62)
            } else {
                Color::RGB(64, 36, 44)
            });
            canvas.draw_rect(Rect::new(
                layout.enabled.x + 1,
                layout.enabled.y + 1,
                layout.enabled.width().saturating_sub(2),
                layout.enabled.height().saturating_sub(2),
            ))?;
        }
        let show_kind_title = layout.kind.height() > 0;
        let enabled_label = timeline_fx_enabled_chip_label(slot, show_kind_title);
        if !enabled_label.is_empty() {
            crate::ui::draw_text_fitted(
                canvas,
                enabled_label,
                centered_text_rect(layout.enabled),
                1,
                Color::RGB(244, 244, 236),
            )?;
        }

        if show_kind_title {
            let kind_fill = if selected
                && self.page_state.selected_timeline_fx_field == TimelineFxField::Kind
            {
                Color::RGB(78, 90, 126)
            } else {
                Color::RGB(52, 58, 80)
            };
            canvas.set_draw_color(kind_fill);
            canvas.fill_rect(layout.kind)?;
            crate::ui::draw_text_fitted(
                canvas,
                timeline_fx_kind_display(slot, layout.kind.width()),
                Rect::new(
                    layout.kind.x + 2,
                    layout.kind.y + ((layout.kind.height() as i32 - 8) / 2).max(0),
                    layout.kind.width().saturating_sub(4),
                    8,
                ),
                1,
                text_color,
            )?;
        }

        let params = slot.effect.inline_parameters();
        let window_start = self
            .timeline_fx_param_window_for_slot(context, slot_index)
            .min(params.len().saturating_sub(1));
        let primary = params.get(window_start);
        let secondary = params.get(window_start + 1);
        self.draw_timeline_fx_param_zone(
            canvas,
            layout.param_primary,
            primary,
            selected && self.page_state.selected_timeline_fx_field == TimelineFxField::ParamPrimary,
            text_color,
        )?;
        self.draw_timeline_fx_param_zone(
            canvas,
            layout.param_secondary,
            secondary,
            selected
                && self.page_state.selected_timeline_fx_field == TimelineFxField::ParamSecondary,
            text_color,
        )?;

        let overflow_selected =
            selected && self.page_state.selected_timeline_fx_field == TimelineFxField::Scroll;
        self.draw_timeline_fx_overflow_zone(
            canvas,
            layout.overflow,
            params.len(),
            window_start,
            overflow_selected,
            text_color,
        )?;

        let move_selected =
            selected && self.page_state.selected_timeline_fx_field == TimelineFxField::Move;
        self.draw_timeline_fx_move_zone(canvas, layout.move_up, "↑", move_selected, text_color)?;
        self.draw_timeline_fx_move_zone(canvas, layout.move_down, "↓", move_selected, text_color)?;
        self.draw_timeline_fx_delete_zone(canvas, layout.delete, text_color)?;
        if selected {
            canvas.set_draw_color(Color::RGB(244, 232, 146));
            let underline_y = layout.row.y + layout.row.height() as i32 - 1;
            canvas.draw_line(
                sdl3::rect::Point::new(layout.row.x, underline_y),
                sdl3::rect::Point::new(layout.row.x + layout.row.width() as i32 - 1, underline_y),
            )?;
        }
        Ok(())
    }

    pub(super) fn draw_timeline_fx_add_row<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        context: TimelineContext,
        layout: TimelineFxRowLayout,
        selected: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let text_color = Color::RGB(226, 232, 238);
        canvas.set_draw_color(if selected {
            Color::RGB(82, 92, 128)
        } else {
            Color::RGB(40, 46, 64)
        });
        canvas.fill_rect(layout.row)?;
        canvas.set_draw_color(if selected {
            Color::RGB(244, 232, 146)
        } else {
            Color::RGB(90, 98, 116)
        });
        if selected {
            let underline_y = layout.row.y + layout.row.height() as i32 - 1;
            canvas.draw_line(
                sdl3::rect::Point::new(layout.row.x, underline_y),
                sdl3::rect::Point::new(layout.row.x + layout.row.width() as i32 - 1, underline_y),
            )?;
        } else {
            canvas.draw_rect(layout.row)?;
        }
        canvas.set_draw_color(Color::RGB(52, 58, 80));
        canvas.fill_rect(layout.enabled)?;
        crate::ui::draw_text_fitted(
            canvas,
            "+",
            centered_text_rect(layout.enabled),
            1,
            text_color,
        )?;
        if layout.kind.height() > 0 {
            crate::ui::draw_text_fitted(
                canvas,
                if context == TimelineContext::InputFx {
                    "Add Input FX"
                } else {
                    "Add Output FX"
                },
                Rect::new(
                    layout.kind.x + 3,
                    layout.kind.y + ((layout.kind.height() as i32 - 8) / 2).max(0),
                    (layout.row.x + layout.row.width() as i32 - layout.kind.x - 6).max(0) as u32,
                    8,
                ),
                1,
                text_color,
            )?;
        }
        Ok(())
    }

    fn draw_timeline_fx_param_zone<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        rect: Rect,
        param: Option<&MidiFxInlineParam>,
        selected: bool,
        text_color: Color,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if rect.height() == 0 {
            return Ok(());
        }
        canvas.set_draw_color(if selected {
            Color::RGB(82, 92, 128)
        } else {
            Color::RGB(52, 58, 80)
        });
        canvas.fill_rect(rect)?;
        if let Some(param) = param {
            let display = if rect.width() >= 26 {
                format!(
                    "{} {}",
                    timeline_param_compact_label(param.label),
                    param.value
                )
            } else if rect.width() >= 18 {
                param.value.clone()
            } else {
                param.value.clone()
            };
            crate::ui::draw_text_fitted(
                canvas,
                &display,
                Rect::new(
                    rect.x + 3,
                    rect.y + ((rect.height() as i32 - 8) / 2).max(0),
                    rect.width().saturating_sub(6),
                    8,
                ),
                1,
                text_color,
            )?;
        } else {
            crate::ui::draw_text_fitted(
                canvas,
                "--",
                centered_text_rect(rect),
                1,
                Color::RGB(160, 166, 178),
            )?;
        }
        Ok(())
    }

    fn draw_timeline_fx_overflow_zone<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        rect: Rect,
        param_count: usize,
        window_start: usize,
        selected: bool,
        text_color: Color,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if rect.height() == 0 {
            return Ok(());
        }
        canvas.set_draw_color(if selected {
            Color::RGB(82, 92, 128)
        } else {
            Color::RGB(52, 58, 80)
        });
        canvas.fill_rect(rect)?;
        if param_count <= 2 {
            crate::ui::draw_text_fitted(
                canvas,
                "--",
                centered_text_rect(rect),
                1,
                Color::RGB(160, 166, 178),
            )?;
            return Ok(());
        }
        let indicator = timeline_fx_overflow_label(param_count, window_start);
        crate::ui::draw_text_fitted(canvas, &indicator, centered_text_rect(rect), 1, text_color)?;
        let track_rect = Rect::new(
            rect.x + 2,
            rect.y + rect.height() as i32 - 3,
            rect.width().saturating_sub(4),
            1,
        );
        canvas.set_draw_color(Color::RGB(116, 126, 150));
        canvas.fill_rect(track_rect)?;
        let thumb_width = (track_rect.width() / param_count.max(1) as u32).max(2);
        let max_start = param_count.saturating_sub(2).max(1);
        let thumb_x = track_rect.x
            + (((track_rect.width().saturating_sub(thumb_width)) as usize * window_start)
                / max_start) as i32;
        canvas.set_draw_color(Color::RGB(236, 238, 228));
        canvas.fill_rect(Rect::new(thumb_x, track_rect.y, thumb_width, 1))?;
        Ok(())
    }

    fn draw_timeline_fx_move_zone<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        rect: Rect,
        label: &str,
        selected: bool,
        text_color: Color,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if rect.height() == 0 {
            return Ok(());
        }
        canvas.set_draw_color(if selected {
            Color::RGB(82, 92, 128)
        } else {
            Color::RGB(52, 58, 80)
        });
        canvas.fill_rect(rect)?;
        crate::ui::draw_text_fitted(canvas, label, centered_text_rect(rect), 1, text_color)?;
        Ok(())
    }

    fn draw_timeline_fx_delete_zone<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        rect: Rect,
        text_color: Color,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if rect.height() == 0 {
            return Ok(());
        }
        canvas.set_draw_color(Color::RGB(108, 56, 62));
        canvas.fill_rect(rect)?;
        canvas.set_draw_color(Color::RGB(204, 124, 132));
        canvas.draw_rect(rect)?;
        crate::ui::draw_text_fitted(canvas, "X", centered_text_rect(rect), 1, text_color)?;
        Ok(())
    }

    pub(super) fn timeline_fx_hit(
        &self,
        context: TimelineContext,
        band_rect: Rect,
        _track: &Track,
        x: i32,
        y: i32,
    ) -> Option<TimelineFxRowRef> {
        let chain_kind = context.chain_kind()?;
        let displayed = self.displayed_timeline_fx_slot_indices_for_track(_track, chain_kind);
        let selected_row = (self
            .project
            .active_track()
            .is_some_and(|active| std::ptr::eq(active, _track))
            && self.page_state.selected_timeline_context == context)
            .then(|| self.selected_timeline_fx_row(chain_kind));
        let chain = self.fx_chain(_track, chain_kind);
        self.timeline_fx_row_layouts(band_rect, &displayed, chain, selected_row)
            .into_iter()
            .enumerate()
            .find_map(|(row_index, layout)| {
                rect_contains(layout.row, x, y).then_some(TimelineFxRowRef {
                    context,
                    row_index,
                    slot_index: displayed.get(row_index).copied().flatten(),
                    layout,
                })
            })
    }

    fn handle_timeline_fx_pointer_hit(
        &mut self,
        hit: TimelineFxRowRef,
        x: i32,
        y: i32,
        source: ActionSource,
        _was_selected: bool,
    ) -> Option<AppControl> {
        self.normalize_timeline_fx_selection();
        let layout = hit.layout;
        if hit.slot_index.is_none() && rect_contains(layout.row, x, y) {
            return Some(self.apply_action_with_source(AppAction::AddSelectedTimelineFx, source));
        }
        if rect_contains(layout.enabled, x, y) {
            self.page_state.selected_timeline_fx_field = TimelineFxField::Enabled;
            return Some(
                self.apply_action_with_source(AppAction::ToggleSelectedTimelineFx, source),
            );
        }
        if rect_contains(layout.kind, x, y) {
            self.page_state.selected_timeline_fx_field = TimelineFxField::Kind;
            return Some(self.apply_action_with_source(AppAction::AdjustPageItemForward, source));
        }
        if rect_contains(layout.param_primary, x, y) {
            self.page_state.selected_timeline_fx_field = TimelineFxField::ParamPrimary;
            return Some(self.apply_action_with_source(AppAction::AdjustPageItemForward, source));
        }
        if rect_contains(layout.param_secondary, x, y) {
            self.page_state.selected_timeline_fx_field = TimelineFxField::ParamSecondary;
            return Some(self.apply_action_with_source(AppAction::AdjustPageItemForward, source));
        }
        if rect_contains(layout.overflow, x, y) {
            self.page_state.selected_timeline_fx_field = TimelineFxField::Scroll;
            return Some(self.apply_action_with_source(AppAction::AdjustPageItemForward, source));
        }
        if rect_contains(layout.move_up, x, y) {
            self.page_state.selected_timeline_fx_field = TimelineFxField::Move;
            return Some(self.apply_action_with_source(AppAction::AdjustPageItemBackward, source));
        }
        if rect_contains(layout.move_down, x, y) {
            self.page_state.selected_timeline_fx_field = TimelineFxField::Move;
            return Some(self.apply_action_with_source(AppAction::AdjustPageItemForward, source));
        }
        if rect_contains(layout.delete, x, y) {
            return Some(
                self.apply_action_with_source(AppAction::DeleteSelectedTimelineFx, source),
            );
        }
        Some(AppControl::Continue)
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

    pub(crate) fn handle_timeline_pointer(
        &mut self,
        content_bounds: Rect,
        x: i32,
        y: i32,
        source: crate::actions::ActionSource,
    ) -> Option<AppControl> {
        let (header_bounds, body_bounds) =
            crate::ui::split_top_strip(content_bounds, 28, 6).ok()?;
        let (transport_bounds, timeline_bounds) =
            crate::ui::split_top_strip(body_bounds, transport_strip_height(), 8).ok()?;
        if rect_contains(self.focused_track_view_button_rect(header_bounds), x, y) {
            return Some(self.apply_action_with_source(AppAction::ToggleFocusedTrackView, source));
        }
        if rect_contains(self.global_loop_reset_button_rect(header_bounds), x, y) {
            return Some(self.apply_action_with_source(AppAction::ResetGlobalLoop, source));
        }

        for (rect, action) in self.transport_chip_actions(transport_bounds) {
            if rect_contains(rect, x, y) {
                return Some(self.apply_action_with_source(action, source));
            }
        }

        for layout in self.visible_timeline_track_layouts(timeline_bounds) {
            let index = layout.track_index;
            let full_label_rect = layout.full_label_rect;
            let detail_label_rect = layout.detail_label_rect;
            let full_content_rect = layout.full_content_rect;
            let detail_content_rect = layout.detail_content_rect;
            for indicator in crate::ui::track_indicators(layout.status_rect) {
                if !rect_contains(indicator.rect, x, y) {
                    continue;
                }

                self.project.active_track_index = index;
                let target = track_indicator_target(indicator.kind, Some(indicator.rect))?;
                return Some(self.apply_action_with_source(target.action, source));
            }

            if rect_contains(self.track_passthrough_button_rect(full_label_rect), x, y) {
                self.project.active_track_index = index;
                return Some(
                    self.apply_action_with_source(AppAction::ToggleCurrentTrackPassthrough, source),
                );
            }

            if let Some(action) = self.recording_clip_scroll_control_hit(
                full_label_rect,
                &self.project.tracks[index],
                x,
                y,
            ) {
                self.project.active_track_index = index;
                return Some(self.apply_action_with_source(action, source));
            }

            if rect_contains(self.recording_view_chip_rect(full_label_rect), x, y) {
                self.project.active_track_index = index;
                return Some(
                    self.apply_action_with_source(
                        AppAction::ToggleCurrentTrackRecordingView,
                        source,
                    ),
                );
            }

            if let Some(hit) = self.timeline_fx_hit(
                TimelineContext::OutputFx,
                layout.output_fx_rect,
                &self.project.tracks[index],
                x,
                y,
            ) {
                let was_selected = self.project.active_track_index == index
                    && self.page_state.selected_timeline_context == hit.context
                    && self.selected_timeline_fx_row(MidiFxChainKind::Output) == hit.row_index;
                self.project.active_track_index = index;
                self.page_state.selected_timeline_context = hit.context;
                self.set_selected_timeline_fx_row(MidiFxChainKind::Output, hit.row_index);
                return self.handle_timeline_fx_pointer_hit(hit, x, y, source, was_selected);
            }

            if self.project.tracks[index]
                .selected_recording_clip()
                .is_some()
            {
                let (mute_rect, delete_rect) = self.recording_clip_control_rects(full_label_rect);
                if rect_contains(mute_rect, x, y) {
                    self.project.active_track_index = index;
                    return Some(self.apply_action_with_source(
                        AppAction::ToggleSelectedRecordingClipMute,
                        source,
                    ));
                }
                if rect_contains(delete_rect, x, y) {
                    self.project.active_track_index = index;
                    return Some(
                        self.apply_action_with_source(
                            AppAction::DeleteSelectedRecordingClip,
                            source,
                        ),
                    );
                }
            }

            if let Some(hit) = self.timeline_fx_hit(
                TimelineContext::InputFx,
                layout.input_fx_rect,
                &self.project.tracks[index],
                x,
                y,
            ) {
                let was_selected = self.project.active_track_index == index
                    && self.page_state.selected_timeline_context == hit.context
                    && self.selected_timeline_fx_row(MidiFxChainKind::Input) == hit.row_index;
                self.project.active_track_index = index;
                self.page_state.selected_timeline_context = hit.context;
                self.set_selected_timeline_fx_row(MidiFxChainKind::Input, hit.row_index);
                return self.handle_timeline_fx_pointer_hit(hit, x, y, source, was_selected);
            }
            for (slot_index, slot_rect) in self.stored_loop_slot_rects(detail_label_rect) {
                if !rect_contains(slot_rect, x, y) {
                    continue;
                }
                self.project.active_track_index = index;
                if let Some(action) = stored_loop_slot_recall_action(slot_index) {
                    return Some(self.apply_action_with_source(action, source));
                }
            }

            for content_rect in [full_content_rect, detail_content_rect] {
                if let Some(clip_id) =
                    self.recording_lane_hit_clip(content_rect, &self.project.tracks[index], x, y)
                {
                    self.project.active_track_index = index;
                    return Some(self.apply_action_with_source(
                        AppAction::SelectRecordingClip(clip_id),
                        source,
                    ));
                }
            }
        }

        None
    }

    pub(crate) fn timeline_discoverability_targets(
        &self,
        content_bounds: Rect,
    ) -> Vec<(Rect, DiscoverabilityTarget)> {
        let mut targets = Vec::new();
        let (header_bounds, body_bounds) =
            crate::ui::split_top_strip(content_bounds, 28, 6).expect("timeline layout");
        let (transport_bounds, timeline_bounds) =
            crate::ui::split_top_strip(body_bounds, transport_strip_height(), 8)
                .expect("timeline transport");
        targets.push((
            self.focused_track_view_button_rect(header_bounds),
            DiscoverabilityTarget {
                action: AppAction::ToggleFocusedTrackView,
                display_scope: Some("Global"),
                allowed_mapping_scopes: &["Global"],
                overlay_slot: None,
            },
        ));
        targets.push((
            self.global_loop_reset_button_rect(header_bounds),
            DiscoverabilityTarget {
                action: AppAction::ResetGlobalLoop,
                display_scope: Some("Global"),
                allowed_mapping_scopes: &["Global"],
                overlay_slot: None,
            },
        ));
        for (rect, action) in self.transport_chip_actions(transport_bounds) {
            let display_scope = if action == AppAction::ToggleRecording {
                Some("Armed/Active")
            } else {
                Some("Global")
            };
            let allowed_mapping_scopes: &'static [&'static str] =
                if action == AppAction::ToggleRecording {
                    &["Armed/Active", "Active Track"]
                } else {
                    &["Global"]
                };
            targets.push((
                rect,
                DiscoverabilityTarget {
                    action,
                    display_scope,
                    allowed_mapping_scopes,
                    overlay_slot: None,
                },
            ));
        }

        for layout in self.visible_timeline_track_layouts(timeline_bounds) {
            let track = &self.project.tracks[layout.track_index];
            targets.extend(self.track_discoverability_targets(layout, track));
        }

        targets
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

    fn focused_track_view_button_rect(&self, header_bounds: Rect) -> Rect {
        let width = crate::ui::text_width("Track All", 1) + 18;
        Rect::new(
            header_bounds.x + header_bounds.width() as i32 - 240,
            header_bounds.y + 4,
            width.max(78),
            header_bounds.height().saturating_sub(8),
        )
    }

    pub(super) fn visible_timeline_track_layouts(
        &self,
        timeline_bounds: Rect,
    ) -> Vec<TimelineTrackLayout> {
        self.visible_track_columns(timeline_bounds)
            .into_iter()
            .map(|(index, full_bounds, detail_bounds)| {
                self.timeline_track_layout(index, full_bounds, detail_bounds)
            })
            .collect()
    }

    fn transport_chip_actions(&self, bounds: Rect) -> Vec<(Rect, AppAction)> {
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
