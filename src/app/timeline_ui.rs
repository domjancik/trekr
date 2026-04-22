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
        if track.state.passthrough {
            canvas.set_draw_color(Color::RGB(74, 210, 214));
            canvas.fill_rect(Rect::new(
                bounds.x + 1,
                bounds.y + 1,
                2,
                bounds.height().saturating_sub(2),
            ))?;
        }

        let label_rect = timeline_subcolumn_label_rect(bounds, self.timeline_flow);
        let content_rect = timeline_subcolumn_content_rect(bounds, self.timeline_flow);
        canvas.set_draw_color(accent);
        canvas.fill_rect(label_rect)?;

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
        let top_row_y = label_rect.y + 3;
        let bottom_row_y = label_rect.y + label_rect.height() as i32 - 10;
        let clip_controls = if !detail && track.selected_recording_clip().is_some() {
            Some(self.recording_clip_control_rects(label_rect))
        } else {
            None
        };
        let name_right = clip_controls
            .map(|(mute_rect, _)| mute_rect.x - 4)
            .unwrap_or(label_rect.x + label_rect.width() as i32 - 4);
        let label_left = if detail {
            let slot_rects = self.stored_loop_slot_rects(label_rect);
            let active_slot = track.active_stored_loop_slot();
            let queued_slot = track.queued_stored_loop_slot();
            for (slot_index, slot_rect) in &slot_rects {
                let filled = track.stored_loop_slot(*slot_index).is_some();
                let active = active_slot == Some(*slot_index);
                let queued = queued_slot == Some(*slot_index);
                canvas.set_draw_color(if active {
                    Color::RGB(238, 186, 112)
                } else if queued {
                    Color::RGB(104, 146, 172)
                } else if filled {
                    Color::RGB(132, 118, 98)
                } else {
                    Color::RGB(72, 70, 68)
                });
                canvas.fill_rect(*slot_rect)?;
                canvas.set_draw_color(if active {
                    Color::RGB(252, 228, 164)
                } else if queued {
                    Color::RGB(176, 222, 246)
                } else if filled {
                    Color::RGB(184, 168, 138)
                } else {
                    Color::RGB(122, 120, 116)
                });
                canvas.draw_rect(*slot_rect)?;
                crate::ui::draw_text_fitted(
                    canvas,
                    &(slot_index + 1).to_string(),
                    Rect::new(
                        slot_rect.x + 1,
                        slot_rect.y + 1,
                        slot_rect.width().saturating_sub(2),
                        slot_rect.height().saturating_sub(2),
                    ),
                    1,
                    if active {
                        Color::RGB(26, 20, 16)
                    } else if queued {
                        Color::RGB(16, 26, 34)
                    } else if filled {
                        Color::RGB(38, 34, 28)
                    } else {
                        Color::RGB(180, 178, 172)
                    },
                )?;
            }
            if STORED_LOOP_SLOT_COUNT > slot_rects.len() {
                let overflow = format!("+{}", STORED_LOOP_SLOT_COUNT - slot_rects.len());
                if let Some((_, last_slot_rect)) = slot_rects.last() {
                    crate::ui::draw_text_fitted(
                        canvas,
                        &overflow,
                        Rect::new(
                            last_slot_rect.x + last_slot_rect.width() as i32 + 3,
                            last_slot_rect.y + 1,
                            14,
                            7,
                        ),
                        1,
                        Color::RGB(210, 194, 160),
                    )?;
                }
            }
            slot_rects
                .last()
                .map(|(_, rect)| rect.x + rect.width() as i32 + 5)
                .unwrap_or(label_rect.x + 4)
        } else {
            let passthrough_button = self.track_passthrough_button_rect(label_rect);
            canvas.set_draw_color(if track.state.passthrough {
                Color::RGB(74, 210, 214)
            } else {
                Color::RGB(44, 70, 94)
            });
            canvas.fill_rect(passthrough_button)?;
            canvas.set_draw_color(if track.state.passthrough {
                Color::RGB(210, 246, 248)
            } else {
                Color::RGB(144, 170, 194)
            });
            canvas.draw_rect(passthrough_button)?;
            crate::ui::draw_text_fitted(
                canvas,
                "THRU",
                Rect::new(
                    passthrough_button.x + 2,
                    passthrough_button.y + 1,
                    passthrough_button.width().saturating_sub(4),
                    passthrough_button.height().saturating_sub(2),
                ),
                1,
                if track.state.passthrough {
                    Color::RGB(10, 28, 34)
                } else {
                    Color::RGB(230, 236, 240)
                },
            )?;
            passthrough_button.x + passthrough_button.width() as i32 + 4
        };
        crate::ui::draw_text_fitted(
            canvas,
            &track.name,
            Rect::new(
                label_left,
                top_row_y,
                (name_right - label_left).max(0) as u32,
                8,
            ),
            1,
            Color::RGB(244, 244, 236),
        )?;

        let role_badge = if detail {
            crate::ui::detail_badge_rect(label_rect)
        } else {
            Rect::new(
                label_rect.x + 4,
                bottom_row_y,
                label_rect.width().saturating_sub(8).min(28),
                8,
            )
        };
        canvas.set_draw_color(if detail {
            if track.state.loop_enabled && self.project.transport.loop_enabled {
                Color::RGB(252, 192, 104)
            } else {
                Color::RGB(88, 82, 76)
            }
        } else {
            Color::RGB(38, 58, 90)
        });
        canvas.fill_rect(role_badge)?;
        canvas.set_draw_color(if detail {
            Color::RGB(238, 214, 172)
        } else {
            Color::RGB(188, 204, 226)
        });
        canvas.draw_rect(role_badge)?;
        crate::ui::draw_text_fitted(
            canvas,
            if detail { "LOOP" } else { "SONG" },
            Rect::new(
                role_badge.x + 2,
                role_badge.y + 1,
                role_badge.width().saturating_sub(4),
                role_badge.height().saturating_sub(2),
            ),
            1,
            if detail {
                Color::RGB(28, 22, 18)
            } else {
                Color::RGB(244, 244, 236)
            },
        )?;
        if !detail {
            self.draw_recording_view_controls(
                canvas,
                label_rect,
                content_rect,
                track,
                clip_controls,
            )?;
        }

        let note_range = crate::timeline::LoopRegion::new(view_start_ticks, range_ticks.max(1));
        let selected_note_indices = track.selected_note_indices();
        let focused_note_index = track.focused_note_index();
        let anchor_note_index = track.anchor_note_index();
        let preview_region = track.preview_region(
            self.project.transport,
            self.record_capture_ticks(track),
            self.record_context(track),
        );
        let preview_notes = track.preview_notes(
            self.project.transport,
            self.record_capture_ticks(track),
            self.record_context(track),
        );
        self.draw_track_recording_content(
            canvas,
            content_rect,
            track,
            note_range,
            is_active,
            detail,
            selected_note_indices.as_slice(),
            focused_note_index,
            anchor_note_index,
            preview_region,
            preview_notes.as_slice(),
        )?;

        if track.recording_view != RecordingView::Stacked {
            if let Some(preview_region) = preview_region {
                if preview_region.intersects(note_range) {
                    for region in crate::ui::region_rects(
                        content_rect,
                        &[preview_region],
                        note_range,
                        self.timeline_flow,
                    ) {
                        if detail {
                            canvas.set_draw_color(Color::RGBA(214, 72, 72, 124));
                            canvas.fill_rect(region.rect)?;
                        }
                        canvas.set_draw_color(Color::RGB(248, 122, 122));
                        canvas.draw_rect(region.rect)?;
                    }
                }
            }

            for note in crate::ui::note_rects(
                content_rect,
                preview_notes.as_slice(),
                note_range,
                self.timeline_flow,
            ) {
                canvas.set_draw_color(Color::RGBA(238, 108, 108, 176));
                canvas.fill_rect(note.rect)?;
                canvas.set_draw_color(Color::RGB(255, 176, 176));
                canvas.draw_rect(note.rect)?;
            }
        }

        self.draw_track_loop_markers(canvas, content_rect, note_range, track)?;

        let playhead = crate::ui::playhead_rect_in_range(
            content_rect,
            self.timeline_flow,
            view_start_ticks,
            range_ticks.max(1),
            playhead_ticks,
        )?;
        if !detail && track.recording_view == RecordingView::Stacked && is_active {
            canvas.set_draw_color(if self.project.transport.playing {
                Color::RGB(248, 240, 132)
            } else {
                Color::RGB(140, 150, 162)
            });
            canvas.fill_rect(playhead)?;
            self.draw_recording_clip_scrollbar(canvas, content_rect, track)?;
        } else {
            canvas.set_draw_color(if self.project.transport.playing {
                Color::RGB(248, 240, 132)
            } else {
                Color::RGB(140, 150, 162)
            });
            canvas.fill_rect(playhead)?;
        }
        for tick in crate::ui::timeline_ruler_ticks(content_rect, self.timeline_flow) {
            canvas.set_draw_color(Color::RGB(166, 178, 198));
            canvas.fill_rect(tick)?;
        }

        Ok(())
    }

    fn draw_recording_view_controls<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        label_rect: Rect,
        _content_rect: Rect,
        track: &Track,
        clip_controls: Option<(Rect, Rect)>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if track.recording_view == RecordingView::Stacked {
            let can_scroll_left = self.can_select_previous_recording_clip(track);
            let can_scroll_right = self.can_select_next_recording_clip(track);
            let (left_rect, right_rect) = self.recording_view_scroll_control_rects(label_rect);
            canvas.set_draw_color(if can_scroll_left {
                Color::RGB(74, 82, 98)
            } else {
                Color::RGB(48, 54, 68)
            });
            canvas.fill_rect(left_rect)?;
            canvas.set_draw_color(if can_scroll_left {
                Color::RGB(202, 212, 224)
            } else {
                Color::RGB(112, 118, 130)
            });
            canvas.draw_rect(left_rect)?;
            crate::ui::draw_text_fitted(
                canvas,
                "<",
                Rect::new(
                    left_rect.x + 6,
                    left_rect.y + 1,
                    left_rect.width().saturating_sub(12),
                    8,
                ),
                1,
                if can_scroll_left {
                    Color::RGB(244, 244, 236)
                } else {
                    Color::RGB(144, 150, 160)
                },
            )?;
            canvas.set_draw_color(if can_scroll_right {
                Color::RGB(74, 82, 98)
            } else {
                Color::RGB(48, 54, 68)
            });
            canvas.fill_rect(right_rect)?;
            canvas.set_draw_color(if can_scroll_right {
                Color::RGB(202, 212, 224)
            } else {
                Color::RGB(112, 118, 130)
            });
            canvas.draw_rect(right_rect)?;
            crate::ui::draw_text_fitted(
                canvas,
                ">",
                Rect::new(
                    right_rect.x + 6,
                    right_rect.y + 1,
                    right_rect.width().saturating_sub(12),
                    8,
                ),
                1,
                if can_scroll_right {
                    Color::RGB(244, 244, 236)
                } else {
                    Color::RGB(144, 150, 160)
                },
            )?;
        }
        let view_rect = self.recording_view_chip_rect(label_rect);
        canvas.set_draw_color(match track.recording_view {
            RecordingView::Overlay => Color::RGB(50, 84, 126),
            RecordingView::Stacked => Color::RGB(124, 98, 48),
        });
        canvas.fill_rect(view_rect)?;
        canvas.set_draw_color(Color::RGB(232, 228, 208));
        canvas.draw_rect(view_rect)?;
        crate::ui::draw_text_fitted(
            canvas,
            match track.recording_view {
                RecordingView::Overlay => "OVR",
                RecordingView::Stacked => "STK",
            },
            Rect::new(
                view_rect.x + 3,
                view_rect.y + 1,
                view_rect.width().saturating_sub(6),
                view_rect.height().saturating_sub(2),
            ),
            1,
            Color::RGB(248, 244, 236),
        )?;

        if let (Some(selected_clip), Some((mute_rect, delete_rect))) =
            (track.selected_recording_clip(), clip_controls)
        {
            canvas.set_draw_color(if selected_clip.muted {
                Color::RGB(120, 118, 112)
            } else {
                Color::RGB(84, 122, 92)
            });
            canvas.fill_rect(mute_rect)?;
            canvas.set_draw_color(Color::RGB(228, 232, 216));
            canvas.draw_rect(mute_rect)?;
            crate::ui::draw_text_fitted(
                canvas,
                if selected_clip.muted { "ON" } else { "M" },
                Rect::new(
                    mute_rect.x + 2,
                    mute_rect.y + 1,
                    mute_rect.width().saturating_sub(4),
                    mute_rect.height().saturating_sub(2),
                ),
                1,
                Color::RGB(246, 244, 236),
            )?;

            canvas.set_draw_color(Color::RGB(132, 74, 70));
            canvas.fill_rect(delete_rect)?;
            canvas.set_draw_color(Color::RGB(240, 220, 210));
            canvas.draw_rect(delete_rect)?;
            crate::ui::draw_text_fitted(
                canvas,
                "X",
                Rect::new(
                    delete_rect.x + 2,
                    delete_rect.y + 1,
                    delete_rect.width().saturating_sub(4),
                    delete_rect.height().saturating_sub(2),
                ),
                1,
                Color::RGB(250, 242, 236),
            )?;
        }

        Ok(())
    }

    fn draw_track_recording_content<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        content_rect: Rect,
        track: &Track,
        note_range: crate::timeline::LoopRegion,
        is_active: bool,
        detail: bool,
        selected_note_indices: &[usize],
        focused_note_index: Option<usize>,
        anchor_note_index: Option<usize>,
        preview_region: Option<crate::timeline::Region>,
        preview_notes: &[crate::project::MidiNote],
    ) -> Result<(), Box<dyn std::error::Error>> {
        if track.recording_view == RecordingView::Stacked
            && (!track.recording_clips().is_empty() || preview_region.is_some())
        {
            let unowned_regions: Vec<_> = track
                .regions
                .iter()
                .copied()
                .filter(|region| region.recording_clip_id.is_none())
                .collect();
            let unowned_notes = indexed_notes(track, None);
            self.draw_region_entries(
                canvas,
                content_rect,
                &unowned_regions,
                note_range,
                track,
                is_active,
                track.state.muted,
            )?;
            self.draw_note_entries(
                canvas,
                content_rect,
                &unowned_notes,
                note_range,
                track,
                detail,
                track.state.muted,
                selected_note_indices,
                focused_note_index,
                anchor_note_index,
            )?;

            for lane in self.recording_lane_layouts(content_rect, track) {
                canvas.set_draw_color(if lane.preview {
                    Color::RGB(54, 32, 36)
                } else if lane.selected {
                    Color::RGB(46, 62, 94)
                } else {
                    Color::RGB(26, 34, 48)
                });
                canvas.fill_rect(lane.rect)?;
                canvas.set_draw_color(if lane.preview {
                    Color::RGB(248, 122, 122)
                } else if lane.selected {
                    Color::RGB(248, 226, 134)
                } else {
                    Color::RGB(76, 92, 118)
                });
                canvas.draw_rect(lane.rect)?;

                if lane.preview {
                    if let Some(preview_region) =
                        preview_region.filter(|region| region.intersects(note_range))
                    {
                        for region in crate::ui::region_rects(
                            lane.rect,
                            &[preview_region],
                            note_range,
                            self.timeline_flow,
                        ) {
                            if detail {
                                canvas.set_draw_color(Color::RGBA(214, 72, 72, 124));
                                canvas.fill_rect(region.rect)?;
                            }
                            canvas.set_draw_color(Color::RGB(248, 122, 122));
                            canvas.draw_rect(region.rect)?;
                        }
                    }

                    for note in crate::ui::note_rects(
                        lane.rect,
                        preview_notes,
                        note_range,
                        self.timeline_flow,
                    ) {
                        canvas.set_draw_color(Color::RGBA(238, 108, 108, 176));
                        canvas.fill_rect(note.rect)?;
                        canvas.set_draw_color(Color::RGB(255, 176, 176));
                        canvas.draw_rect(note.rect)?;
                    }
                } else if let Some(clip_id) = lane.clip_id {
                    let lane_muted = track.state.muted || lane.muted;
                    let lane_regions: Vec<_> = track
                        .regions
                        .iter()
                        .copied()
                        .filter(|region| region.recording_clip_id == Some(clip_id))
                        .collect();
                    let lane_notes = indexed_notes(track, Some(clip_id));
                    self.draw_region_entries(
                        canvas,
                        lane.rect,
                        &lane_regions,
                        note_range,
                        track,
                        is_active,
                        lane_muted,
                    )?;
                    self.draw_note_entries(
                        canvas,
                        lane.rect,
                        &lane_notes,
                        note_range,
                        track,
                        detail,
                        lane_muted,
                        selected_note_indices,
                        focused_note_index,
                        anchor_note_index,
                    )?;
                }
            }

            return Ok(());
        }

        self.draw_region_entries(
            canvas,
            content_rect,
            &track.regions,
            note_range,
            track,
            is_active,
            track.state.muted,
        )?;
        self.draw_note_entries(
            canvas,
            content_rect,
            &indexed_all_notes(track),
            note_range,
            track,
            detail,
            track.state.muted,
            selected_note_indices,
            focused_note_index,
            anchor_note_index,
        )?;
        Ok(())
    }

    fn draw_region_entries<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        lane_rect: Rect,
        regions: &[crate::timeline::Region],
        note_range: crate::timeline::LoopRegion,
        track: &Track,
        is_active: bool,
        muted_override: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for source_region in regions.iter().copied() {
            let region_muted =
                muted_override || track.recording_clip_is_muted(source_region.recording_clip_id);
            let Some(region) = crate::ui::region_rects(
                lane_rect,
                &[source_region],
                note_range,
                self.timeline_flow,
            )
            .into_iter()
            .next() else {
                continue;
            };
            canvas.set_draw_color(if region.clipped {
                Color::RGB(108, 88, 56)
            } else if region_muted {
                Color::RGB(42, 46, 56)
            } else {
                Color::RGB(44, 54, 76)
            });
            canvas.fill_rect(region.rect)?;
            canvas.set_draw_color(if is_active {
                Color::RGB(212, 196, 122)
            } else {
                Color::RGB(96, 106, 126)
            });
            canvas.draw_rect(region.rect)?;
        }

        Ok(())
    }

    fn draw_note_entries<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        lane_rect: Rect,
        note_entries: &[(usize, crate::project::MidiNote)],
        note_range: crate::timeline::LoopRegion,
        track: &Track,
        detail: bool,
        muted_override: bool,
        selected_note_indices: &[usize],
        focused_note_index: Option<usize>,
        anchor_note_index: Option<usize>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let notes: Vec<_> = note_entries.iter().map(|(_, note)| *note).collect();
        for note in crate::ui::note_rects(lane_rect, &notes, note_range, self.timeline_flow) {
            let absolute_index = note_entries[note.source_index].0;
            let note_muted = muted_override
                || track
                    .recording_clip_is_muted(note_entries[note.source_index].1.recording_clip_id);
            let selected = selected_note_indices.contains(&absolute_index);
            let focused = focused_note_index == Some(absolute_index);
            let anchored = anchor_note_index == Some(absolute_index);
            canvas.set_draw_color(if selected && detail {
                Color::RGB(112, 174, 228)
            } else if selected {
                Color::RGB(88, 136, 194)
            } else if note_muted {
                Color::RGB(92, 100, 112)
            } else if note.clipped {
                Color::RGB(244, 204, 132)
            } else {
                Color::RGB(210, 222, 236)
            });
            canvas.fill_rect(note.rect)?;
            canvas.set_draw_color(if focused {
                Color::RGB(252, 246, 158)
            } else if anchored {
                Color::RGB(180, 226, 176)
            } else if selected {
                Color::RGB(224, 238, 248)
            } else if note_muted {
                Color::RGB(128, 134, 144)
            } else {
                Color::RGB(245, 247, 250)
            });
            canvas.draw_rect(note.rect)?;
            if focused {
                let inner = Rect::new(
                    note.rect.x + 1,
                    note.rect.y + 1,
                    note.rect.width().saturating_sub(2).max(1),
                    note.rect.height().saturating_sub(2).max(1),
                );
                canvas.set_draw_color(Color::RGB(252, 208, 88));
                canvas.draw_rect(inner)?;
            }
        }

        Ok(())
    }

    pub(super) fn recording_lane_layouts(
        &self,
        content_rect: Rect,
        track: &Track,
    ) -> Vec<RecordingLaneLayout> {
        let gap = 2;
        let window = self.recording_lane_window(track, self.recording_lane_capacity(content_rect));
        let visible_clips = &track.recording_clips()[window.committed_start..window.committed_end];
        let lane_count = window.visible_committed + usize::from(window.show_preview);
        let lane_rects = match self.timeline_flow {
            TimelineFlow::DownwardColumns => {
                crate::ui::equal_columns(content_rect, lane_count, gap)
            }
            TimelineFlow::AcrossRows => crate::ui::stacked_rows(content_rect, lane_count, gap),
        };

        let mut layouts: Vec<_> = visible_clips
            .iter()
            .zip(lane_rects.iter().copied())
            .map(|(clip, rect)| RecordingLaneLayout {
                clip_id: Some(clip.id),
                rect,
                selected: track.selected_recording_clip_id == Some(clip.id),
                muted: clip.muted,
                preview: false,
            })
            .collect();

        if window.show_preview {
            if let Some(rect) = lane_rects.get(window.visible_committed).copied() {
                layouts.push(RecordingLaneLayout {
                    clip_id: None,
                    rect,
                    selected: false,
                    muted: false,
                    preview: true,
                });
            }
        }

        layouts
    }

    fn recording_lane_hit_clip(
        &self,
        content_rect: Rect,
        track: &Track,
        x: i32,
        y: i32,
    ) -> Option<u64> {
        self.recording_lane_layouts(content_rect, track)
            .into_iter()
            .find_map(|lane| {
                rect_contains(lane.rect, x, y)
                    .then_some(lane.clip_id)
                    .flatten()
            })
    }

    pub(super) fn recording_lane_capacity(&self, content_rect: Rect) -> usize {
        match self.timeline_flow {
            TimelineFlow::DownwardColumns => {
                let min_lane_width = 15_i32;
                let gap = 2_i32;
                (((content_rect.width() as i32 + gap) / (min_lane_width + gap)).max(1)) as usize
            }
            TimelineFlow::AcrossRows => {
                let min_lane_height = 26_i32;
                let gap = 2_i32;
                (((content_rect.height() as i32 + gap) / (min_lane_height + gap)).max(1)) as usize
            }
        }
    }

    pub(super) fn recording_view_chip_rect(&self, label_rect: Rect) -> Rect {
        let top_y = label_rect.y + label_rect.height() as i32 - 10;
        let right = label_rect.x + label_rect.width() as i32 - 4;
        Rect::new(right - 26, top_y, 26, 8)
    }

    pub(super) fn track_passthrough_button_rect(&self, label_rect: Rect) -> Rect {
        Rect::new(
            label_rect.x + 4,
            label_rect.y + 3,
            label_rect.width().saturating_sub(8).min(30),
            8,
        )
    }

    fn stored_loop_visible_slot_count(&self, label_rect: Rect) -> usize {
        let slot_w = 8_i32;
        let gap = 2_i32;
        let side_padding = 8_i32;
        let min_name_space = 24_i32;
        let available = label_rect.width() as i32 - side_padding - min_name_space;
        if available < slot_w {
            return 0;
        }
        (((available + gap) / (slot_w + gap)).max(0) as usize).min(STORED_LOOP_SLOT_COUNT)
    }

    pub(super) fn stored_loop_slot_rects(&self, label_rect: Rect) -> Vec<(usize, Rect)> {
        let visible_slots = self
            .stored_loop_visible_slot_count(label_rect)
            .min(STORED_LOOP_SLOT_COUNT);
        let slot_w = 8_u32;
        let slot_h = 7_u32;
        let gap = 2_i32;
        let mut rects = Vec::with_capacity(visible_slots);
        for slot_index in 0..visible_slots {
            rects.push((
                slot_index,
                Rect::new(
                    label_rect.x + 4 + slot_index as i32 * (slot_w as i32 + gap),
                    label_rect.y + 2,
                    slot_w,
                    slot_h,
                ),
            ));
        }
        rects
    }

    pub(super) fn track_column_body_bounds(
        &self,
        full_bounds: Rect,
        detail_bounds: Rect,
    ) -> (Rect, Rect) {
        let pair_bounds = crate::ui::union_rect(full_bounds, detail_bounds);
        let status_rect = crate::ui::track_status_rect(pair_bounds, self.timeline_flow);
        let (top_band_height, bottom_band_height) = self.timeline_fx_band_heights();
        let top_gap = 4_i32;
        let bottom_gap = 4_i32;
        let top_reserve = (status_rect.y + status_rect.height() as i32 + top_gap + top_band_height
            - pair_bounds.y)
            .max(0);
        let bottom_reserve = (bottom_gap + bottom_band_height).max(0);
        let new_height = full_bounds
            .height()
            .saturating_sub(top_reserve as u32)
            .saturating_sub(bottom_reserve as u32);
        let full = Rect::new(
            full_bounds.x,
            full_bounds.y + top_reserve,
            full_bounds.width(),
            new_height,
        );
        let detail = Rect::new(
            detail_bounds.x,
            detail_bounds.y + top_reserve,
            detail_bounds.width(),
            new_height,
        );
        (full, detail)
    }

    fn timeline_fx_band_heights(&self) -> (i32, i32) {
        let input = self
            .project
            .tracks
            .iter()
            .map(|track| displayed_track_fx_band_height(&track.midi_fx.input_fx))
            .max()
            .unwrap_or(displayed_track_fx_band_height(&[]));
        let output = self
            .project
            .tracks
            .iter()
            .map(|track| displayed_track_fx_band_height(&track.midi_fx.output_fx))
            .max()
            .unwrap_or(displayed_track_fx_band_height(&[]));
        (input, output)
    }

    pub(super) fn track_fx_band_rects(
        &self,
        full_bounds: Rect,
        detail_bounds: Rect,
        _track: &Track,
    ) -> (Rect, Rect) {
        let pair_bounds = crate::ui::union_rect(full_bounds, detail_bounds);
        let status_rect = crate::ui::track_status_rect(pair_bounds, self.timeline_flow);
        let (body_full_bounds, body_detail_bounds) =
            self.track_column_body_bounds(full_bounds, detail_bounds);
        let body_pair_bounds = crate::ui::union_rect(body_full_bounds, body_detail_bounds);
        let (top_band_height, bottom_band_height) = self.timeline_fx_band_heights();
        let top = Rect::new(
            pair_bounds.x + 4,
            status_rect.y + status_rect.height() as i32 + 4,
            pair_bounds.width().saturating_sub(8),
            top_band_height as u32,
        );
        let bottom = Rect::new(
            pair_bounds.x + 4,
            body_pair_bounds.y + body_pair_bounds.height() as i32 + 4,
            pair_bounds.width().saturating_sub(8),
            bottom_band_height as u32,
        );
        (top, bottom)
    }

    fn draw_track_fx_bands<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        layout: TimelineTrackLayout,
        track: &Track,
        is_active: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for (context, rect) in [
            (TimelineContext::InputFx, layout.input_fx_rect),
            (TimelineContext::OutputFx, layout.output_fx_rect),
        ] {
            let chain_kind = context.chain_kind().expect("fx context");
            let chain = self.fx_chain(track, chain_kind);
            let active_slots: Vec<(usize, &MidiFxSlot)> = chain
                .iter()
                .enumerate()
                .filter_map(|(index, slot)| slot.as_ref().map(|slot| (index, slot)))
                .collect();
            let displayed_rows =
                self.displayed_timeline_fx_slot_indices_for_track(track, chain_kind);
            let enabled = active_slots.iter().any(|(_, slot)| slot.enabled);
            let fill = if context == TimelineContext::InputFx {
                if enabled {
                    Color::RGB(78, 128, 198)
                } else if is_active {
                    Color::RGB(56, 70, 94)
                } else {
                    Color::RGB(46, 56, 74)
                }
            } else if enabled {
                Color::RGB(172, 108, 156)
            } else if is_active {
                Color::RGB(84, 68, 94)
            } else {
                Color::RGB(64, 58, 76)
            };
            let border = if enabled {
                Color::RGB(236, 238, 228)
            } else if is_active {
                Color::RGB(176, 184, 198)
            } else {
                Color::RGB(120, 126, 140)
            };
            canvas.set_draw_color(fill);
            canvas.fill_rect(rect)?;
            canvas.set_draw_color(border);
            canvas.draw_rect(rect)?;

            let selected_row = if is_active && self.page_state.selected_timeline_context == context
            {
                self.selected_timeline_fx_row(chain_kind)
            } else {
                usize::MAX
            };
            let layouts =
                self.timeline_fx_row_layouts(rect, &displayed_rows, chain, Some(selected_row));
            for (line_index, (display_row, layout)) in
                displayed_rows.iter().zip(layouts.iter()).enumerate()
            {
                let selected = line_index == selected_row;
                if let Some(slot_index) = display_row {
                    let slot = chain[*slot_index].as_ref().expect("timeline slot");
                    let text_color = if slot.enabled {
                        Color::RGB(248, 244, 236)
                    } else {
                        Color::RGB(198, 202, 210)
                    };
                    self.draw_timeline_fx_row(
                        canvas,
                        context,
                        *slot_index,
                        slot,
                        *layout,
                        selected,
                        text_color,
                    )?;
                } else {
                    self.draw_timeline_fx_add_row(canvas, context, *layout, selected)?;
                }
            }
        }
        Ok(())
    }

    pub(super) fn timeline_fx_row_layouts(
        &self,
        band_rect: Rect,
        displayed_rows: &[Option<usize>],
        chain: &[Option<MidiFxSlot>],
        _selected_row: Option<usize>,
    ) -> Vec<TimelineFxRowLayout> {
        fn empty_row_rect(row: Rect) -> Rect {
            Rect::new(-10_000, row.y, 1, 1)
        }

        fn take_right(row: Rect, right: &mut i32, width: i32, gap: i32) -> Rect {
            if width <= 0 || *right - width < row.x {
                return empty_row_rect(row);
            }
            let rect = Rect::new(*right - width, row.y, width as u32, row.height());
            *right = rect.x - gap;
            rect
        }

        let row_count = displayed_rows.len().max(1);
        let line_height = 8_i32;
        let line_gap = 2_i32;
        let top_padding = 2_i32;
        let row_y = band_rect.y + top_padding;
        let row_width = band_rect.width().saturating_sub(4);
        let rows: Vec<Rect> = (0..row_count)
            .map(|row_index| {
                Rect::new(
                    band_rect.x + 2,
                    row_y + row_index as i32 * (line_height + line_gap),
                    row_width,
                    line_height as u32,
                )
            })
            .collect();
        rows.into_iter()
            .enumerate()
            .map(|(row_index, row)| {
                let gap = 1;
                let available = row.width() as i32;
                let enabled_width = available.clamp(10, 14);
                let delete_width = available.clamp(5, 6);
                let param_min_width = if available >= 72 { 18 } else { 12 };
                let move_width = if available >= 132 { 6 } else { 0 };
                let (kind_width, visible_param_count, total_param_count) = displayed_rows
                    .get(row_index)
                    .and_then(|slot_index| slot_index.and_then(|index| chain.get(index)))
                    .and_then(|slot| slot.as_ref())
                    .map(|slot| {
                        (
                            timeline_fx_kind_target_width(slot, available as u32) as i32,
                            slot.effect.inline_parameters().len().min(2),
                            slot.effect.inline_parameters().len(),
                        )
                    })
                    .unwrap_or((12, 0, 0));

                let enabled = Rect::new(row.x, row.y, enabled_width as u32, row.height());
                let kind_x = enabled.x + enabled.width() as i32 + gap;
                let kind = Rect::new(kind_x, row.y, kind_width.max(0) as u32, row.height());
                let params_x = kind.x + kind.width() as i32 + gap;
                let mut move_down_width = move_width;
                let mut move_up_width = move_width;
                let mut overflow_width = if total_param_count > 2 {
                    if available >= 72 {
                        10
                    } else {
                        8
                    }
                } else {
                    0
                };
                let mut show_secondary = visible_param_count >= 2;
                loop {
                    let right_fixed_width = delete_width
                        + move_down_width
                        + move_up_width
                        + overflow_width
                        + gap // kind -> params
                        + gap; // params -> delete
                    let right_fixed_gaps = i32::from(move_down_width > 0)
                        + i32::from(move_up_width > 0)
                        + i32::from(overflow_width > 0);
                    let params_total_width = available
                        - enabled_width
                        - kind_width
                        - right_fixed_width
                        - right_fixed_gaps * gap
                        - gap; // enabled -> kind
                    let required_param_width = if show_secondary {
                        param_min_width * 2 + gap
                    } else {
                        param_min_width
                    };
                    if params_total_width >= required_param_width {
                        let mut right = row.x + row.width() as i32;
                        let delete = take_right(row, &mut right, delete_width, gap);
                        let move_down = take_right(row, &mut right, move_down_width, gap);
                        let move_up = take_right(row, &mut right, move_up_width, gap);
                        let overflow = take_right(row, &mut right, overflow_width, gap);
                        let param_right = delete.x - gap;
                        let available_param_width = (param_right - params_x).max(0);
                        let (param_primary, param_secondary) = if show_secondary {
                            let primary_width = (available_param_width - gap) / 2;
                            let secondary_width = available_param_width - gap - primary_width;
                            let primary = Rect::new(
                                params_x,
                                row.y,
                                primary_width.max(0) as u32,
                                row.height(),
                            );
                            let secondary_x = primary.x + primary.width() as i32 + gap;
                            let secondary = Rect::new(
                                secondary_x,
                                row.y,
                                secondary_width.max(0) as u32,
                                row.height(),
                            );
                            (primary, secondary)
                        } else {
                            (
                                Rect::new(
                                    params_x,
                                    row.y,
                                    available_param_width.max(0) as u32,
                                    row.height(),
                                ),
                                empty_row_rect(row),
                            )
                        };
                        return TimelineFxRowLayout {
                            row,
                            enabled,
                            kind,
                            param_primary,
                            param_secondary,
                            overflow,
                            move_up,
                            move_down,
                            delete,
                        };
                    }

                    if move_down_width > 0 {
                        move_down_width = 0;
                    } else if move_up_width > 0 {
                        move_up_width = 0;
                    } else if overflow_width > 0 {
                        overflow_width = 0;
                    } else if show_secondary {
                        show_secondary = false;
                    } else {
                        let mut right = row.x + row.width() as i32;
                        let delete = take_right(row, &mut right, delete_width, gap);
                        let move_down = empty_row_rect(row);
                        let move_up = empty_row_rect(row);
                        let overflow = empty_row_rect(row);
                        let param_right = delete.x - gap;
                        let available_param_width = (param_right - params_x).max(0);
                        let param_primary = Rect::new(
                            params_x,
                            row.y,
                            available_param_width.max(0) as u32,
                            row.height(),
                        );
                        return TimelineFxRowLayout {
                            row,
                            enabled,
                            kind,
                            param_primary,
                            param_secondary: empty_row_rect(row),
                            overflow,
                            move_up,
                            move_down,
                            delete,
                        };
                    }
                }
            })
            .collect()
    }

    fn draw_track_loop_markers<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        content_rect: Rect,
        note_range: crate::timeline::LoopRegion,
        track: &Track,
    ) -> Result<(), Box<dyn std::error::Error>> {
        #[derive(Clone)]
        struct LoopMarker {
            range: crate::timeline::LoopRegion,
            label: String,
            color: Color,
            emphasized: bool,
            queued: bool,
        }
        #[derive(Clone, Copy)]
        struct MarkerSpan {
            color: Color,
            start: i32,
            end: i32,
        }

        let active_slot = track.active_stored_loop_slot();
        let queued_slot = track.queued_stored_loop_slot();
        let mut markers = Vec::new();
        for slot_index in 0..STORED_LOOP_SLOT_COUNT {
            let Some(stored_loop) = track.stored_loop_slot(slot_index) else {
                continue;
            };
            markers.push(LoopMarker {
                range: stored_loop.as_loop_region(),
                label: (slot_index + 1).to_string(),
                color: stored_loop_slot_color(slot_index),
                emphasized: active_slot == Some(slot_index),
                queued: queued_slot == Some(slot_index),
            });
        }

        if active_slot.is_none() {
            markers.push(LoopMarker {
                range: track.loop_region,
                label: "L".to_string(),
                color: if track.state.loop_enabled {
                    Color::RGB(242, 190, 112)
                } else {
                    Color::RGB(128, 122, 112)
                },
                emphasized: true,
                queued: false,
            });
        }

        let mut spans = Vec::new();
        for marker in markers.iter() {
            if !loop_regions_intersect(marker.range, note_range) {
                continue;
            }

            let span_rect = crate::ui::range_highlight_rect(
                content_rect,
                self.timeline_flow,
                note_range.start_ticks,
                note_range.length_ticks,
                marker.range,
            );
            let (start, end) = match self.timeline_flow {
                TimelineFlow::DownwardColumns => (
                    span_rect.y,
                    span_rect.y + span_rect.height().max(1) as i32 - 1,
                ),
                TimelineFlow::AcrossRows => (
                    span_rect.x,
                    span_rect.x + span_rect.width().max(1) as i32 - 1,
                ),
            };
            spans.push(MarkerSpan {
                color: marker.color,
                start,
                end,
            });
        }

        if spans.is_empty() {
            return Ok(());
        }

        let side_thickness = 4_i32;
        let primary_tick = Color::RGB(252, 238, 194);
        let queued_tick = Color::RGB(184, 226, 248);
        let secondary_tick = Color::RGB(218, 224, 232);
        let side_major = side_thickness.max(1) as u32;
        let content_bg = if track.state.muted {
            Color::RGB(16, 18, 24)
        } else {
            Color::RGB(20, 27, 40)
        };

        match self.timeline_flow {
            TimelineFlow::DownwardColumns => {
                let x = content_rect.x + 1;
                let usable_width = (content_rect.x + content_rect.width() as i32 - x).max(1);
                let band_width = side_major.min(usable_width as u32);
                let start_y = content_rect.y;
                let end_y = content_rect.y + content_rect.height() as i32 - 2;
                if end_y < start_y {
                    return Ok(());
                }
                let mut placed_label_rects = Vec::new();
                let label_spacing = 9_i32;

                for y in start_y..=end_y {
                    let colors = spans
                        .iter()
                        .filter(|span| y >= span.start && y <= span.end)
                        .map(|span| span.color)
                        .collect::<Vec<_>>();
                    if colors.is_empty() {
                        continue;
                    }
                    if let Some(color) = interlaced_color_at(&colors, (y - start_y).max(0) as usize)
                    {
                        canvas.set_draw_color(color);
                        canvas.fill_rect(Rect::new(x, y, band_width, 1))?;
                    }
                }

                for marker in markers.iter() {
                    if !loop_regions_intersect(marker.range, note_range) {
                        continue;
                    }
                    let span_rect = crate::ui::range_highlight_rect(
                        content_rect,
                        self.timeline_flow,
                        note_range.start_ticks,
                        note_range.length_ticks,
                        marker.range,
                    );
                    let line_h = span_rect.height().max(1);
                    let marker_start_y = span_rect.y.clamp(start_y, end_y);
                    let end_marker_y = (span_rect.y + line_h as i32 - 1).clamp(start_y, end_y);
                    if marker_start_y > end_marker_y {
                        continue;
                    }
                    canvas.set_draw_color(if marker.emphasized {
                        primary_tick
                    } else if marker.queued {
                        queued_tick
                    } else {
                        secondary_tick
                    });
                    canvas.fill_rect(Rect::new(x, marker_start_y, band_width.min(4), 1))?;
                    canvas.fill_rect(Rect::new(x, end_marker_y, band_width.min(4), 1))?;

                    let marker_mid_y = marker_start_y + (end_marker_y - marker_start_y) / 2;
                    let label_y = (marker_mid_y - 3).clamp(
                        content_rect.y,
                        content_rect.y + content_rect.height() as i32 - 7,
                    );
                    let mut label_rect = Rect::new(x + band_width as i32 + 3, label_y, 8, 7);
                    for offset_step in 0..8 {
                        let candidate = Rect::new(
                            x + band_width as i32 + 3 + offset_step * label_spacing,
                            label_y,
                            8,
                            7,
                        );
                        if !placed_label_rects
                            .iter()
                            .any(|existing| rects_overlap(*existing, candidate))
                        {
                            label_rect = candidate;
                            break;
                        }
                    }
                    placed_label_rects.push(label_rect);
                    let label_readback = readback_rect_rgba(canvas, label_rect, self.viewport_size);
                    crate::ui::draw_text_fitted_inverted(
                        canvas,
                        marker.label.as_str(),
                        label_rect,
                        1,
                        |px, py| readback_color_at(&label_readback, px, py).unwrap_or(content_bg),
                    )?;
                    if marker.emphasized {
                        draw_loop_label_underline(
                            canvas,
                            marker.label.as_str(),
                            label_rect,
                            content_rect,
                            self.viewport_size,
                            content_bg,
                        )?;
                    } else if marker.queued {
                        canvas.set_draw_color(queued_tick);
                        canvas.fill_rect(Rect::new(
                            label_rect.x,
                            (label_rect.y + label_rect.height() as i32 + 1)
                                .min(content_rect.y + content_rect.height() as i32 - 1),
                            label_rect.width().min(4),
                            1,
                        ))?;
                    }
                }
            }
            TimelineFlow::AcrossRows => {
                let y = content_rect.y + 1;
                let usable_height = (content_rect.y + content_rect.height() as i32 - y).max(1);
                let band_height = side_major.min(usable_height as u32);
                let start_x = content_rect.x;
                let end_x = content_rect.x + content_rect.width() as i32 - 1;
                let mut placed_label_rects = Vec::new();
                let label_spacing = 9_i32;

                for x in start_x..=end_x {
                    let colors = spans
                        .iter()
                        .filter(|span| x >= span.start && x <= span.end)
                        .map(|span| span.color)
                        .collect::<Vec<_>>();
                    if colors.is_empty() {
                        continue;
                    }
                    for pixel in 0..band_height as usize {
                        if let Some(color) = interlaced_color_at(&colors, pixel) {
                            canvas.set_draw_color(color);
                            canvas.fill_rect(Rect::new(x, y + pixel as i32, 1, 1))?;
                        }
                    }
                }

                for marker in markers.iter() {
                    if !loop_regions_intersect(marker.range, note_range) {
                        continue;
                    }
                    let span_rect = crate::ui::range_highlight_rect(
                        content_rect,
                        self.timeline_flow,
                        note_range.start_ticks,
                        note_range.length_ticks,
                        marker.range,
                    );
                    let line_w = span_rect.width().max(1);
                    let end_marker_x = span_rect.x + line_w as i32 - 1;
                    canvas.set_draw_color(if marker.emphasized {
                        primary_tick
                    } else if marker.queued {
                        queued_tick
                    } else {
                        secondary_tick
                    });
                    canvas.fill_rect(Rect::new(span_rect.x, y, 1, band_height.min(4)))?;
                    canvas.fill_rect(Rect::new(end_marker_x, y, 1, band_height.min(4)))?;

                    let label_x = (span_rect.x + line_w as i32 / 2 - 3).clamp(
                        content_rect.x,
                        content_rect.x + content_rect.width() as i32 - 7,
                    );
                    let mut label_rect = Rect::new(label_x, y + band_height as i32 + 3, 7, 6);
                    for offset_step in 0..8 {
                        let candidate =
                            Rect::new(label_x + offset_step * label_spacing, label_rect.y, 7, 6);
                        if !placed_label_rects
                            .iter()
                            .any(|existing| rects_overlap(*existing, candidate))
                        {
                            label_rect = candidate;
                            break;
                        }
                    }
                    placed_label_rects.push(label_rect);
                    let label_readback = readback_rect_rgba(canvas, label_rect, self.viewport_size);
                    crate::ui::draw_text_fitted_inverted(
                        canvas,
                        marker.label.as_str(),
                        label_rect,
                        1,
                        |px, py| readback_color_at(&label_readback, px, py).unwrap_or(content_bg),
                    )?;
                    if marker.emphasized {
                        draw_loop_label_underline(
                            canvas,
                            marker.label.as_str(),
                            label_rect,
                            content_rect,
                            self.viewport_size,
                            content_bg,
                        )?;
                    } else if marker.queued {
                        canvas.set_draw_color(queued_tick);
                        canvas.fill_rect(Rect::new(
                            label_rect.x,
                            (label_rect.y + label_rect.height() as i32 + 1)
                                .min(content_rect.y + content_rect.height() as i32 - 1),
                            label_rect.width().min(4),
                            1,
                        ))?;
                    }
                }
            }
        }

        Ok(())
    }

    pub(super) fn recording_view_scroll_control_rects(&self, label_rect: Rect) -> (Rect, Rect) {
        let top_y = label_rect.y + label_rect.height() as i32 - 10;
        let view_rect = self.recording_view_chip_rect(label_rect);
        let right_rect = Rect::new(view_rect.x - 16, top_y, 12, 8);
        let left_rect = Rect::new(right_rect.x - 14, top_y, 12, 8);
        (left_rect, right_rect)
    }

    fn selected_recording_clip_index(&self, track: &Track) -> Option<usize> {
        track.selected_recording_clip_id.and_then(|selected_id| {
            track
                .recording_clips()
                .iter()
                .position(|clip| clip.id == selected_id)
        })
    }

    fn can_select_previous_recording_clip(&self, track: &Track) -> bool {
        self.selected_recording_clip_index(track)
            .map(|index| index > 0)
            .unwrap_or(false)
    }

    fn can_select_next_recording_clip(&self, track: &Track) -> bool {
        self.selected_recording_clip_index(track)
            .map(|index| index + 1 < track.recording_clips().len())
            .unwrap_or(false)
    }

    pub(super) fn sync_active_track_recording_clip_scroll(&mut self) {
        let Some(full_bounds) = self.active_track_full_bounds() else {
            return;
        };
        let content_rect = crate::ui::track_content_rect(full_bounds, self.timeline_flow);
        let total_capacity = self.recording_lane_capacity(content_rect).max(1);
        let Some(track) = self.project.active_track_mut() else {
            return;
        };
        let total_lanes = track.recording_clips.len() + usize::from(track.active_take.is_some());
        if total_lanes == 0 {
            track.recording_clip_scroll = 0;
            return;
        }

        let visible_lanes = total_capacity.min(total_lanes);
        let max_start = total_lanes.saturating_sub(visible_lanes);
        track.recording_clip_scroll = track.recording_clip_scroll.min(max_start);
        if track.active_take.is_some() {
            track.recording_clip_scroll = track.recording_clip_scroll.max(max_start);
        }
        let Some(selected_id) = track.selected_recording_clip_id else {
            return;
        };
        if track.active_take.is_some() {
            return;
        }
        let Some(selected_index) = track
            .recording_clips
            .iter()
            .position(|clip| clip.id == selected_id)
        else {
            return;
        };
        if selected_index < track.recording_clip_scroll {
            track.recording_clip_scroll = selected_index;
        } else if selected_index >= track.recording_clip_scroll + visible_lanes {
            track.recording_clip_scroll = selected_index + 1 - visible_lanes;
        }
    }

    fn recording_clip_scroll_control_hit(
        &self,
        label_rect: Rect,
        track: &Track,
        x: i32,
        y: i32,
    ) -> Option<AppAction> {
        if track.recording_view != RecordingView::Stacked {
            return None;
        }
        let (left_rect, right_rect) = self.recording_view_scroll_control_rects(label_rect);
        if rect_contains(left_rect, x, y) && self.can_select_previous_recording_clip(track) {
            return Some(AppAction::SelectPreviousRecordingClip);
        }
        if rect_contains(right_rect, x, y) && self.can_select_next_recording_clip(track) {
            return Some(AppAction::SelectNextRecordingClip);
        }
        None
    }

    pub(super) fn recording_clip_control_rects(&self, label_rect: Rect) -> (Rect, Rect) {
        let top_y = label_rect.y + 3;
        let right = label_rect.x + label_rect.width() as i32 - 4;
        (
            Rect::new(right - 28, top_y, 12, 8),
            Rect::new(right - 12, top_y, 12, 8),
        )
    }

    pub(super) fn recording_clip_scrollbar_rects(
        &self,
        content_rect: Rect,
        track: &Track,
    ) -> Option<(Rect, Rect)> {
        if track.recording_view != RecordingView::Stacked {
            return None;
        }

        let total_lanes = track.recording_clips.len() + usize::from(track.active_take.is_some());
        if total_lanes == 0 {
            return None;
        }

        let window = self.recording_lane_window(track, self.recording_lane_capacity(content_rect));
        let visible_lanes = window.visible_total.clamp(1, total_lanes);
        let start = window.start;
        let rail = Rect::new(
            content_rect.x + 4,
            content_rect.y,
            content_rect.width().saturating_sub(8),
            2,
        );
        if rail.width() == 0 {
            return None;
        }

        let thumb_width = ((rail.width() as usize * visible_lanes) / total_lanes)
            .max(6)
            .min(rail.width() as usize) as u32;
        let max_offset = rail.width().saturating_sub(thumb_width) as i32;
        let max_start = total_lanes.saturating_sub(visible_lanes);
        let thumb_x = if max_start == 0 {
            rail.x
        } else {
            rail.x + (max_offset as i64 * start as i64 / max_start as i64) as i32
        };
        let thumb = Rect::new(thumb_x, rail.y, thumb_width, 2);
        Some((rail, thumb))
    }

    fn draw_recording_clip_scrollbar<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        content_rect: Rect,
        track: &Track,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let Some((rail, thumb)) = self.recording_clip_scrollbar_rects(content_rect, track) else {
            return Ok(());
        };
        canvas.set_draw_color(Color::RGB(92, 100, 120));
        canvas.fill_rect(rail)?;
        canvas.set_draw_color(Color::RGB(244, 214, 118));
        canvas.fill_rect(thumb)?;
        Ok(())
    }

    fn recording_lane_window(&self, track: &Track, total_capacity: usize) -> RecordingLaneWindow {
        let total_capacity = total_capacity.max(1);
        let committed_len = track.recording_clips().len();
        let preview_index = track.active_take.as_ref().map(|_| committed_len);
        let total_lanes = committed_len + usize::from(preview_index.is_some());
        if total_lanes == 0 {
            return RecordingLaneWindow {
                start: 0,
                visible_total: 0,
                committed_start: 0,
                committed_end: 0,
                visible_committed: 0,
                show_preview: false,
            };
        }

        let visible_total = total_capacity.min(total_lanes);
        let max_start = total_lanes.saturating_sub(visible_total);
        let mut start = track.recording_clip_scroll.min(max_start);
        if let Some(preview_index) = preview_index {
            start = start.max(preview_index + 1 - visible_total);
        }
        let end = start + visible_total;
        let committed_start = start.min(committed_len);
        let committed_end = end.min(committed_len);
        let show_preview = preview_index.is_some_and(|preview_index| preview_index >= start);

        RecordingLaneWindow {
            start,
            visible_total,
            committed_start,
            committed_end,
            visible_committed: committed_end.saturating_sub(committed_start),
            show_preview,
        }
    }

    pub(super) fn timeline_fx_footer_content(&self) -> Option<(String, String)> {
        if self.page_state.current_page != AppPage::Timeline {
            return None;
        }
        let context = self.page_state.selected_timeline_context;
        let chain_kind = context.chain_kind()?;
        let track = self.project.active_track()?;
        if let Some(slot) = self.selected_timeline_fx_slot(track, chain_kind) {
            Some((
                format!(
                    "{} {}",
                    context.label(),
                    self.page_state.selected_timeline_fx_field.label()
                ),
                format!(
                    "Shift+Left/Right ctx  Up/Down row  Enter field  Q/E edit  Delete remove  {}",
                    slot.effect.kind().label()
                ),
            ))
        } else {
            Some((
                format!("{} Add", context.label()),
                "Shift+Left/Right ctx  Up/Down row  Q/E or click add row".to_string(),
            ))
        }
    }
}
