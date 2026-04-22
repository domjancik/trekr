use super::*;

impl App {
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

    pub(super) fn handle_timeline_fx_pointer_hit(
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

    pub(super) fn timeline_fx_band_heights(&self) -> (i32, i32) {
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

    pub(super) fn draw_track_fx_bands<T: RenderTarget>(
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::actions::ActionSource;

    #[test]
    fn timeline_track_fx_row_click_selects_output_fx_context() {
        let mut app = App::new();
        let content_bounds = Rect::new(40, 40, 1200, 620);
        let (_, body_bounds) =
            crate::ui::split_top_strip(content_bounds, 28, 6).expect("timeline content");
        let (_, timeline_bounds) =
            crate::ui::split_top_strip(body_bounds, transport_strip_height(), 8)
                .expect("timeline body");
        let columns = crate::ui::track_column_pairs(timeline_bounds, app.project.tracks.len());
        let (full_bounds, detail_bounds) = columns[0];
        let (_, output_band) =
            app.track_fx_band_rects(full_bounds, detail_bounds, &app.project.tracks[0]);
        let displayed = app.displayed_timeline_fx_slot_indices(MidiFxChainKind::Output);
        let row = app.timeline_fx_row_layouts(
            output_band,
            &displayed,
            &app.project.tracks[0].midi_fx.output_fx,
            None,
        )[0]
        .row;

        let control = app.handle_timeline_pointer(
            content_bounds,
            row.x + 2,
            row.y + row.height() as i32 / 2,
            ActionSource::Pointer,
        );

        assert_eq!(control, Some(AppControl::Continue));
        assert_eq!(
            app.page_state.selected_timeline_context,
            TimelineContext::OutputFx
        );
    }

    #[test]
    fn timeline_fx_adjust_and_move_actions_update_selected_output_row() {
        let mut app = App::new();
        app.page_state.current_page = AppPage::Timeline;
        app.page_state.selected_timeline_context = TimelineContext::OutputFx;
        app.page_state.selected_timeline_fx_field = TimelineFxField::Kind;

        let before_kind = app
            .selected_timeline_fx_slot(app.project.active_track().unwrap(), MidiFxChainKind::Output)
            .unwrap()
            .effect
            .kind();
        app.adjust_page_item(1);
        let after_kind = app
            .selected_timeline_fx_slot(app.project.active_track().unwrap(), MidiFxChainKind::Output)
            .unwrap()
            .effect
            .kind();
        assert_ne!(before_kind, after_kind);

        app.page_state.selected_timeline_fx_field = TimelineFxField::Move;
        let before_row = app.selected_timeline_fx_row(MidiFxChainKind::Output);
        app.adjust_page_item(1);
        let after_row = app.selected_timeline_fx_row(MidiFxChainKind::Output);
        assert!(after_row >= before_row);
    }

    #[test]
    fn timeline_fx_enabled_click_toggles_effect_without_changing_kind() {
        let mut app = App::new();
        app.project.active_track_mut().unwrap().midi_fx.output_fx =
            vec![Some(MidiFxSlot::default()), None, None, None];
        app.page_state.current_page = AppPage::Timeline;
        app.page_state.selected_timeline_context = TimelineContext::OutputFx;
        app.set_selected_timeline_fx_row(MidiFxChainKind::Output, 0);
        let before_enabled = app.project.tracks[0].midi_fx.output_fx[0]
            .as_ref()
            .unwrap()
            .enabled;
        let before_kind = app.project.tracks[0].midi_fx.output_fx[0]
            .as_ref()
            .unwrap()
            .effect
            .kind();
        let content_bounds = Rect::new(40, 40, 1200, 620);
        let (_, body_bounds) =
            crate::ui::split_top_strip(content_bounds, 28, 6).expect("timeline content");
        let (_, timeline_bounds) =
            crate::ui::split_top_strip(body_bounds, transport_strip_height(), 8)
                .expect("timeline body");
        let columns = crate::ui::track_column_pairs(timeline_bounds, app.project.tracks.len());
        let (full_bounds, detail_bounds) = columns[0];
        let (_, output_band) =
            app.track_fx_band_rects(full_bounds, detail_bounds, &app.project.tracks[0]);
        let displayed = app.displayed_timeline_fx_slot_indices(MidiFxChainKind::Output);
        let layout = app.timeline_fx_row_layouts(
            output_band,
            &displayed,
            &app.project.tracks[0].midi_fx.output_fx,
            Some(0),
        )[0];

        let control = app.handle_timeline_pointer(
            content_bounds,
            layout.enabled.x + layout.enabled.width() as i32 / 2,
            layout.enabled.y + layout.enabled.height() as i32 / 2,
            ActionSource::Pointer,
        );

        assert_eq!(control, Some(AppControl::Continue));
        assert_eq!(
            app.page_state.selected_timeline_fx_field,
            TimelineFxField::Enabled
        );
        let after_slot = app.project.tracks[0].midi_fx.output_fx[0].as_ref().unwrap();
        assert_ne!(after_slot.enabled, before_enabled);
        assert_eq!(after_slot.effect.kind(), before_kind);
    }

    #[test]
    fn timeline_add_row_click_inserts_effect_on_first_click() {
        let mut app = App::new();
        app.project.active_track_mut().unwrap().midi_fx.output_fx =
            vec![Some(MidiFxSlot::default()), None, None, None];
        let content_bounds = Rect::new(40, 40, 1200, 620);
        let (_, body_bounds) =
            crate::ui::split_top_strip(content_bounds, 28, 6).expect("timeline content");
        let (_, timeline_bounds) =
            crate::ui::split_top_strip(body_bounds, transport_strip_height(), 8)
                .expect("timeline body");
        let columns = crate::ui::track_column_pairs(timeline_bounds, app.project.tracks.len());
        let (full_bounds, detail_bounds) = columns[0];
        let (_, output_band) =
            app.track_fx_band_rects(full_bounds, detail_bounds, &app.project.tracks[0]);
        let displayed = app.displayed_timeline_fx_slot_indices(MidiFxChainKind::Output);
        let layouts = app.timeline_fx_row_layouts(
            output_band,
            &displayed,
            &app.project.tracks[0].midi_fx.output_fx,
            None,
        );
        let add_row = layouts.last().expect("add row").row;
        let before = app
            .active_timeline_fx_slot_indices(MidiFxChainKind::Output)
            .len();

        let control = app.handle_timeline_pointer(
            content_bounds,
            add_row.x + 4,
            add_row.y + add_row.height() as i32 / 2,
            ActionSource::Pointer,
        );

        assert_eq!(control, Some(AppControl::Continue));
        let after = app
            .active_timeline_fx_slot_indices(MidiFxChainKind::Output)
            .len();
        assert_eq!(after, before + 1);
    }

    #[test]
    fn timeline_fx_hover_targets_kind_action_not_routing() {
        let mut app = App::new();
        app.project.active_track_mut().unwrap().midi_fx.output_fx =
            vec![Some(MidiFxSlot::default()), None, None, None];
        let content_bounds = Rect::new(40, 40, 1200, 620);
        let (_, body_bounds) =
            crate::ui::split_top_strip(content_bounds, 28, 6).expect("timeline content");
        let (_, timeline_bounds) =
            crate::ui::split_top_strip(body_bounds, transport_strip_height(), 8)
                .expect("timeline body");
        let columns = crate::ui::track_column_pairs(timeline_bounds, app.project.tracks.len());
        let (full_bounds, detail_bounds) = columns[0];
        let (_, output_band) =
            app.track_fx_band_rects(full_bounds, detail_bounds, &app.project.tracks[0]);
        let displayed = app.displayed_timeline_fx_slot_indices(MidiFxChainKind::Output);
        let layout = app.timeline_fx_row_layouts(
            output_band,
            &displayed,
            &app.project.tracks[0].midi_fx.output_fx,
            None,
        )[0];

        let target = app
            .timeline_discoverability_targets(content_bounds)
            .into_iter()
            .find_map(|(rect, target)| {
                super::rect_contains(
                    rect,
                    layout.kind.x + layout.kind.width() as i32 / 2,
                    layout.kind.y + layout.kind.height() as i32 / 2,
                )
                .then_some(target)
            })
            .expect("discoverability target");

        assert_eq!(target.action, AppAction::CycleSelectedTimelineFxKind);
    }
}
