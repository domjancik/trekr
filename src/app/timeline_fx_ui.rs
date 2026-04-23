use super::*;
use crate::midi_fx::MidiFxInlineParam;

impl App {
    pub(super) fn select_timeline_fx_row(&mut self, delta: i32) {
        let Some(chain_kind) = self.page_state.selected_timeline_context.chain_kind() else {
            return;
        };
        let displayed_rows = self.displayed_timeline_fx_slot_indices(chain_kind);
        if displayed_rows.is_empty() {
            return;
        }
        let current = self.selected_timeline_fx_row(chain_kind) as i32;
        let next = (current + delta).rem_euclid(displayed_rows.len() as i32) as usize;
        self.set_selected_timeline_fx_row(chain_kind, next);
    }

    pub(super) fn selected_timeline_fx_row(&self, chain_kind: MidiFxChainKind) -> usize {
        let Some(track) = self.project.active_track() else {
            return 0;
        };
        let stored = match chain_kind {
            MidiFxChainKind::Input => track.midi_fx.timeline_ui.input_selected_row,
            MidiFxChainKind::Output => track.midi_fx.timeline_ui.output_selected_row,
        };
        let len = self.displayed_timeline_fx_slot_indices(chain_kind).len();
        if len == 0 { 0 } else { stored.min(len - 1) }
    }

    pub(super) fn set_selected_timeline_fx_row(
        &mut self,
        chain_kind: MidiFxChainKind,
        row_index: usize,
    ) {
        let len = self.displayed_timeline_fx_slot_indices(chain_kind).len();
        let clamped = if len == 0 { 0 } else { row_index.min(len - 1) };
        if let Some(track) = self.project.active_track_mut() {
            match chain_kind {
                MidiFxChainKind::Input => track.midi_fx.timeline_ui.input_selected_row = clamped,
                MidiFxChainKind::Output => track.midi_fx.timeline_ui.output_selected_row = clamped,
            }
        }
    }

    pub(super) fn active_timeline_fx_slot_indices(
        &self,
        chain_kind: MidiFxChainKind,
    ) -> Vec<usize> {
        let Some(track) = self.project.active_track() else {
            return Vec::new();
        };
        self.active_timeline_fx_slot_indices_for_track(track, chain_kind)
    }

    pub(super) fn active_timeline_fx_slot_indices_for_track(
        &self,
        track: &Track,
        chain_kind: MidiFxChainKind,
    ) -> Vec<usize> {
        self.fx_chain(track, chain_kind)
            .iter()
            .enumerate()
            .filter_map(|(index, slot)| slot.as_ref().map(|_| index))
            .collect()
    }

    pub(super) fn displayed_timeline_fx_slot_indices(
        &self,
        chain_kind: MidiFxChainKind,
    ) -> Vec<Option<usize>> {
        let Some(track) = self.project.active_track() else {
            return Vec::new();
        };
        self.displayed_timeline_fx_slot_indices_for_track(track, chain_kind)
    }

    pub(super) fn displayed_timeline_fx_slot_indices_for_track(
        &self,
        track: &Track,
        chain_kind: MidiFxChainKind,
    ) -> Vec<Option<usize>> {
        let mut rows = self
            .active_timeline_fx_slot_indices_for_track(track, chain_kind)
            .into_iter()
            .map(Some)
            .collect::<Vec<_>>();
        if self
            .first_empty_timeline_fx_slot_index_for_track(track, chain_kind)
            .is_some()
        {
            rows.push(None);
        }
        rows
    }

    pub(super) fn first_empty_timeline_fx_slot_index_for_track(
        &self,
        track: &Track,
        chain_kind: MidiFxChainKind,
    ) -> Option<usize> {
        self.fx_chain(track, chain_kind)
            .iter()
            .enumerate()
            .find_map(|(index, slot)| slot.is_none().then_some(index))
    }

    pub(super) fn selected_timeline_fx_slot_index(
        &self,
        chain_kind: MidiFxChainKind,
    ) -> Option<usize> {
        self.displayed_timeline_fx_slot_indices(chain_kind)
            .get(self.selected_timeline_fx_row(chain_kind))
            .copied()
            .flatten()
    }

    pub(super) fn selected_timeline_fx_active_row_index(
        &self,
        chain_kind: MidiFxChainKind,
    ) -> Option<usize> {
        let selected_slot = self.selected_timeline_fx_slot_index(chain_kind)?;
        self.active_timeline_fx_slot_indices(chain_kind)
            .iter()
            .position(|slot_index| *slot_index == selected_slot)
    }

    pub(super) fn set_selected_timeline_fx_slot_index(
        &mut self,
        chain_kind: MidiFxChainKind,
        slot_index: usize,
    ) {
        let row_index = self
            .displayed_timeline_fx_slot_indices(chain_kind)
            .iter()
            .position(|candidate| *candidate == Some(slot_index))
            .unwrap_or(0);
        self.set_selected_timeline_fx_row(chain_kind, row_index);
    }

    pub(super) fn selected_timeline_fx_slot<'a>(
        &self,
        track: &'a Track,
        chain_kind: MidiFxChainKind,
    ) -> Option<&'a MidiFxSlot> {
        self.selected_timeline_fx_slot_index(chain_kind)
            .and_then(|slot_index| self.fx_chain(track, chain_kind).get(slot_index))
            .and_then(|slot| slot.as_ref())
    }

    pub(super) fn selected_timeline_fx_param_window(&self, chain_kind: MidiFxChainKind) -> usize {
        let Some(track) = self.project.active_track() else {
            return 0;
        };
        let Some(slot_index) = self.selected_timeline_fx_slot_index(chain_kind) else {
            return 0;
        };
        let windows = match chain_kind {
            MidiFxChainKind::Input => &track.midi_fx.timeline_ui.input_param_windows,
            MidiFxChainKind::Output => &track.midi_fx.timeline_ui.output_param_windows,
        };
        windows.get(slot_index).copied().unwrap_or(0)
    }

    pub(super) fn timeline_fx_param_window_for_slot(
        &self,
        context: TimelineContext,
        slot_index: usize,
    ) -> usize {
        let Some(track) = self.project.active_track() else {
            return 0;
        };
        let windows = match context.chain_kind() {
            Some(MidiFxChainKind::Input) => &track.midi_fx.timeline_ui.input_param_windows,
            Some(MidiFxChainKind::Output) => &track.midi_fx.timeline_ui.output_param_windows,
            None => return 0,
        };
        windows.get(slot_index).copied().unwrap_or(0)
    }

    pub(super) fn set_selected_timeline_fx_param_window(
        &mut self,
        chain_kind: MidiFxChainKind,
        start: usize,
    ) {
        let Some(slot_index) = self.selected_timeline_fx_slot_index(chain_kind) else {
            return;
        };
        if let Some(track) = self.project.active_track_mut() {
            let windows = match chain_kind {
                MidiFxChainKind::Input => &mut track.midi_fx.timeline_ui.input_param_windows,
                MidiFxChainKind::Output => &mut track.midi_fx.timeline_ui.output_param_windows,
            };
            if let Some(window) = windows.get_mut(slot_index) {
                *window = start;
            }
        }
    }

    pub(super) fn normalize_timeline_fx_selection(&mut self) {
        if let Some(chain_kind) = self.page_state.selected_timeline_context.chain_kind() {
            let displayed = self.displayed_timeline_fx_slot_indices(chain_kind);
            if displayed.is_empty() {
                self.set_selected_timeline_fx_row(chain_kind, 0);
                return;
            }
            self.set_selected_timeline_fx_row(chain_kind, self.selected_timeline_fx_row(chain_kind));
        }
    }

    pub(super) fn adjust_timeline_context(&mut self, delta: i32) {
        let Some(chain_kind) = self.page_state.selected_timeline_context.chain_kind() else {
            return;
        };
        if self.selected_timeline_fx_slot_index(chain_kind).is_none() {
            self.adjust_selected_timeline_fx_kind(delta);
        } else {
            match self.page_state.selected_timeline_fx_field {
                TimelineFxField::Enabled => self.toggle_selected_timeline_fx_enabled(),
                TimelineFxField::Kind => self.adjust_selected_timeline_fx_kind(delta),
                TimelineFxField::ParamPrimary => self.adjust_selected_timeline_fx_parameter(0, delta),
                TimelineFxField::ParamSecondary => self.adjust_selected_timeline_fx_parameter(1, delta),
                TimelineFxField::Scroll => self.scroll_selected_timeline_fx_parameter_window(delta),
                TimelineFxField::Move => self.move_selected_timeline_fx(delta),
            }
        }
        self.normalize_timeline_fx_selection();
        if self.page_state.selected_timeline_context.chain_kind() != Some(chain_kind) {
            self.page_state.selected_timeline_context = match chain_kind {
                MidiFxChainKind::Input => TimelineContext::InputFx,
                MidiFxChainKind::Output => TimelineContext::OutputFx,
            };
        }
    }

    pub(super) fn activate_timeline_context_item(&mut self) {
        let Some(chain_kind) = self.page_state.selected_timeline_context.chain_kind() else {
            return;
        };
        if self.selected_timeline_fx_slot_index(chain_kind).is_none() {
            self.add_selected_timeline_fx();
            return;
        }
        self.page_state.selected_timeline_fx_field = self.page_state.selected_timeline_fx_field.next();
    }

    pub(super) fn reverse_activate_timeline_context_item(&mut self) {
        let Some(chain_kind) = self.page_state.selected_timeline_context.chain_kind() else {
            return;
        };
        if self.selected_timeline_fx_slot_index(chain_kind).is_none() {
            self.activate_timeline_context_item();
            return;
        }
        self.page_state.selected_timeline_fx_field =
            self.page_state.selected_timeline_fx_field.previous();
    }

    pub(super) fn toggle_selected_timeline_fx_enabled(&mut self) {
        let Some(chain_kind) = self.page_state.selected_timeline_context.chain_kind() else {
            return;
        };
        let Some(slot_index) = self.selected_timeline_fx_slot_index(chain_kind) else {
            return;
        };
        let mut changed = false;
        if let Some(track) = self.project.active_track_mut() {
            let chain = match chain_kind {
                MidiFxChainKind::Input => &mut track.midi_fx.input_fx,
                MidiFxChainKind::Output => &mut track.midi_fx.output_fx,
            };
            if let Some(Some(slot)) = chain.get_mut(slot_index) {
                slot.enabled = !slot.enabled;
                changed = true;
            }
        }
        if changed {
            self.handle_timeline_fx_configuration_changed();
        }
    }

    pub(super) fn adjust_selected_timeline_fx_kind(&mut self, delta: i32) {
        let Some(chain_kind) = self.page_state.selected_timeline_context.chain_kind() else {
            return;
        };
        let selected_slot_index = self.selected_timeline_fx_slot_index(chain_kind);
        let mut changed = false;
        if let Some(track) = self.project.active_track_mut() {
            let chain = match chain_kind {
                MidiFxChainKind::Input => &mut track.midi_fx.input_fx,
                MidiFxChainKind::Output => &mut track.midi_fx.output_fx,
            };
            if let Some(slot_index) = selected_slot_index {
                if let Some(Some(slot)) = chain.get_mut(slot_index) {
                    *slot = cycle_existing_fx_kind(slot, delta);
                    changed = true;
                }
            } else if let Some(empty_slot) = chain.iter().position(|slot| slot.is_none()) {
                chain[empty_slot] = cycle_fx_kind(None, delta);
                self.set_selected_timeline_fx_slot_index(chain_kind, empty_slot);
                changed = true;
            }
        }
        self.normalize_timeline_fx_selection();
        if changed {
            self.handle_timeline_fx_configuration_changed();
        }
    }

    pub(super) fn add_selected_timeline_fx(&mut self) {
        let Some(chain_kind) = self.page_state.selected_timeline_context.chain_kind() else {
            return;
        };
        let selected_slot_index = self.selected_timeline_fx_slot_index(chain_kind);
        if selected_slot_index.is_some() {
            return;
        }
        self.adjust_selected_timeline_fx_kind(1);
    }

    pub(super) fn delete_selected_timeline_fx(&mut self) {
        let Some(chain_kind) = self.page_state.selected_timeline_context.chain_kind() else {
            return;
        };
        let Some(slot_index) = self.selected_timeline_fx_slot_index(chain_kind) else {
            return;
        };
        let mut changed = false;
        if let Some(track) = self.project.active_track_mut() {
            let (chain, windows) = match chain_kind {
                MidiFxChainKind::Input => (
                    &mut track.midi_fx.input_fx,
                    &mut track.midi_fx.timeline_ui.input_param_windows,
                ),
                MidiFxChainKind::Output => (
                    &mut track.midi_fx.output_fx,
                    &mut track.midi_fx.timeline_ui.output_param_windows,
                ),
            };
            chain[slot_index] = None;
            if let Some(window) = windows.get_mut(slot_index) {
                *window = 0;
            }
            changed = true;
        }
        self.normalize_timeline_fx_selection();
        if changed {
            self.handle_timeline_fx_configuration_changed();
        }
    }

    pub(super) fn adjust_selected_timeline_fx_parameter(
        &mut self,
        visible_offset: usize,
        delta: i32,
    ) {
        let Some(chain_kind) = self.page_state.selected_timeline_context.chain_kind() else {
            return;
        };
        let Some(slot_index) = self.selected_timeline_fx_slot_index(chain_kind) else {
            return;
        };
        let track_count = self.project.tracks.len();
        let ppqn = self.project.transport.ppqn;
        let window_start = self.selected_timeline_fx_param_window(chain_kind);
        let parameter_index = window_start + visible_offset;
        let mut changed = false;
        if let Some(track) = self.project.active_track_mut() {
            let chain = match chain_kind {
                MidiFxChainKind::Input => &mut track.midi_fx.input_fx,
                MidiFxChainKind::Output => &mut track.midi_fx.output_fx,
            };
            let Some(Some(slot)) = chain.get_mut(slot_index) else {
                return;
            };
            slot.effect
                .adjust_inline_parameter(parameter_index, delta, track_count, ppqn);
            changed = true;
        }
        if changed {
            self.handle_timeline_fx_configuration_changed();
        }
    }

    pub(super) fn scroll_selected_timeline_fx_parameter_window(&mut self, delta: i32) {
        let Some(chain_kind) = self.page_state.selected_timeline_context.chain_kind() else {
            return;
        };
        let Some(track) = self.project.active_track() else {
            return;
        };
        let Some(slot) = self.selected_timeline_fx_slot(track, chain_kind) else {
            return;
        };
        let param_count = slot.effect.inline_parameters().len();
        let max_start = param_count.saturating_sub(2);
        let current = self.selected_timeline_fx_param_window(chain_kind);
        let next = (current as i32 + delta).clamp(0, max_start as i32) as usize;
        self.set_selected_timeline_fx_param_window(chain_kind, next);
    }

    pub(super) fn move_selected_timeline_fx(&mut self, delta: i32) {
        let Some(chain_kind) = self.page_state.selected_timeline_context.chain_kind() else {
            return;
        };
        let active_slots = self.active_timeline_fx_slot_indices(chain_kind);
        if active_slots.len() < 2 {
            return;
        }
        let Some(row_index) = self.selected_timeline_fx_active_row_index(chain_kind) else {
            return;
        };
        let target_row = if delta < 0 {
            row_index.saturating_sub(1)
        } else {
            (row_index + 1).min(active_slots.len() - 1)
        };
        if row_index == target_row {
            return;
        }
        let source_slot = active_slots[row_index];
        let target_slot = active_slots[target_row];
        let mut changed = false;
        if let Some(track) = self.project.active_track_mut() {
            let (chain, windows) = match chain_kind {
                MidiFxChainKind::Input => (
                    &mut track.midi_fx.input_fx,
                    &mut track.midi_fx.timeline_ui.input_param_windows,
                ),
                MidiFxChainKind::Output => (
                    &mut track.midi_fx.output_fx,
                    &mut track.midi_fx.timeline_ui.output_param_windows,
                ),
            };
            chain.swap(source_slot, target_slot);
            windows.swap(source_slot, target_slot);
            changed = true;
        }
        self.set_selected_timeline_fx_row(chain_kind, target_row);
        if changed {
            self.handle_timeline_fx_configuration_changed();
        }
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

pub(super) fn timeline_fx_enabled_chip_label(
    slot: &MidiFxSlot,
    show_kind_title: bool,
) -> &'static str {
    if show_kind_title {
        ""
    } else {
        slot.effect.kind().compact_label()
    }
}

pub(super) fn timeline_fx_kind_display(slot: &MidiFxSlot, width: u32) -> &'static str {
    if width >= 20 {
        slot.effect.kind().short_label()
    } else {
        slot.effect.kind().compact_label()
    }
}

pub(super) fn timeline_fx_kind_target_width(slot: &MidiFxSlot, available: u32) -> u32 {
    let label = if available >= 72 {
        slot.effect.kind().short_label()
    } else {
        slot.effect.kind().compact_label()
    };
    let glyph_width = 5_u32;
    let padding = 8_u32;
    (label.len() as u32 * glyph_width + padding).clamp(20, 28)
}

pub(super) fn timeline_fx_overflow_label(param_count: usize, window_start: usize) -> String {
    if param_count <= 2 {
        "--".to_string()
    } else {
        let window_count = param_count.saturating_sub(1).max(1);
        format!("{}/{}", window_start + 1, window_count)
    }
}

fn timeline_param_compact_label(label: &str) -> &str {
    match label {
        "Rate" => "Rt",
        "Gate" => "Gt",
        "Low" => "Lo",
        "High" => "Hi",
        "List" => "Ls",
        "Semi" => "Sm",
        "Vel" => "Vl",
        "Len" => "Ln",
        "Root" => "Rt",
        "Tgt" => "Tg",
        "Dly" => "Dl",
        "Src" => "Sc",
        other => other,
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

    #[test]
    fn timeline_unselected_fx_row_prioritizes_kind_and_primary_value_width() {
        let mut app = App::new();
        app.project.tracks[0].midi_fx.output_fx[0] = Some(MidiFxSlot {
            enabled: true,
            effect: MidiFx::Arp {
                step_ticks: 240,
                order: crate::midi_fx::ArpOrder::Up,
                gate_percent: 100,
            },
        });
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

        assert!(layout.param_primary.width() > 0);
        assert!(layout.kind.width() < layout.row.width());
        assert!(layout.param_secondary.width() > 0);
        assert!(layout.delete.width() > 0);
    }

    #[test]
    fn timeline_fx_row_layout_drops_low_priority_controls_when_narrow() {
        let mut app = App::new();
        app.project.tracks[0].midi_fx.output_fx[0] = Some(MidiFxSlot {
            enabled: true,
            effect: MidiFx::Arp {
                step_ticks: 240,
                order: crate::midi_fx::ArpOrder::Up,
                gate_percent: 100,
            },
        });
        let displayed = vec![Some(0)];
        let layout = app.timeline_fx_row_layouts(
            Rect::new(10, 10, 56, 14),
            &displayed,
            &app.project.tracks[0].midi_fx.output_fx,
            None,
        )[0];
        let row_right = layout.row.x + layout.row.width() as i32;
        for rect in [
            layout.enabled,
            layout.kind,
            layout.param_primary,
            layout.param_secondary,
            layout.overflow,
            layout.move_up,
            layout.move_down,
            layout.delete,
        ] {
            if rect.x >= layout.row.x {
                assert!(rect.x + rect.width() as i32 <= row_right);
            }
        }
        assert!(layout.kind.width() > 0);
        assert!(layout.param_primary.width() > 0);
        assert!(layout.delete.width() > 0);
        assert!(layout.param_secondary.x < layout.row.x);
        assert!(layout.move_up.x < layout.row.x);
        assert!(layout.move_down.x < layout.row.x);
    }

    #[test]
    fn timeline_selected_fx_row_uses_same_compact_layout() {
        let mut app = App::new();
        app.project.tracks[0].midi_fx.output_fx[0] = Some(MidiFxSlot {
            enabled: true,
            effect: MidiFx::Arp {
                step_ticks: 240,
                order: crate::midi_fx::ArpOrder::Up,
                gate_percent: 100,
            },
        });
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
        let unselected_layout = app.timeline_fx_row_layouts(
            output_band,
            &displayed,
            &app.project.tracks[0].midi_fx.output_fx,
            None,
        )[0];
        let layout = app.timeline_fx_row_layouts(
            output_band,
            &displayed,
            &app.project.tracks[0].midi_fx.output_fx,
            Some(0),
        )[0];

        assert!(layout.param_secondary.width() > 0);
        assert!(layout.move_up.width() > 0);
        assert!(layout.move_down.width() > 0);
        assert!(layout.delete.width() > 0);
        assert_eq!(layout.kind.width(), unselected_layout.kind.width());
        assert_eq!(
            layout.param_secondary.width(),
            unselected_layout.param_secondary.width()
        );
        assert_eq!(layout.delete.width(), unselected_layout.delete.width());
    }

    #[test]
    fn timeline_fx_row_places_secondary_parameter_before_overflow() {
        let mut app = App::new();
        app.project.tracks[0].midi_fx.output_fx[0] = Some(MidiFxSlot {
            enabled: true,
            effect: MidiFx::Arp {
                step_ticks: 240,
                order: crate::midi_fx::ArpOrder::Up,
                gate_percent: 100,
            },
        });
        let displayed = vec![Some(0)];
        let layout = app.timeline_fx_row_layouts(
            Rect::new(10, 10, 120, 14),
            &displayed,
            &app.project.tracks[0].midi_fx.output_fx,
            Some(0),
        )[0];

        assert!(layout.param_secondary.width() > 0);
        assert!(layout.overflow.width() > 0);
        assert!(layout.param_secondary.x < layout.overflow.x);
    }

    #[test]
    fn overflow_label_uses_window_position() {
        assert_eq!(timeline_fx_overflow_label(2, 0), "--");
        assert_eq!(timeline_fx_overflow_label(3, 0), "1/2");
        assert_eq!(timeline_fx_overflow_label(3, 1), "2/2");
    }

    #[test]
    fn timeline_fx_kind_display_uses_short_labels_at_compact_widths() {
        let slot = MidiFxSlot {
            enabled: true,
            effect: MidiFx::Arp {
                step_ticks: 240,
                order: crate::midi_fx::ArpOrder::Up,
                gate_percent: 100,
            },
        };

        assert_eq!(timeline_fx_kind_display(&slot, 19), "AR");
        assert_eq!(timeline_fx_kind_display(&slot, 20), "ARP");
    }

    #[test]
    fn timeline_fx_row_splits_width_evenly_between_two_visible_params() {
        let mut app = App::new();
        app.project.tracks[0].midi_fx.output_fx[0] = Some(MidiFxSlot {
            enabled: true,
            effect: MidiFx::Arp {
                step_ticks: 240,
                order: crate::midi_fx::ArpOrder::Up,
                gate_percent: 100,
            },
        });
        let displayed = vec![Some(0)];
        let layout = app.timeline_fx_row_layouts(
            Rect::new(10, 10, 120, 14),
            &displayed,
            &app.project.tracks[0].midi_fx.output_fx,
            Some(0),
        )[0];

        assert!(layout.param_primary.width() > 0);
        assert!(layout.param_secondary.width() > 0);
        assert!(
            (layout.param_primary.width() as i32 - layout.param_secondary.width() as i32).abs()
                <= 1
        );
    }

    #[test]
    fn timeline_fx_enabled_chip_hides_label_when_kind_title_is_visible() {
        let slot = MidiFxSlot::default();
        assert_eq!(timeline_fx_enabled_chip_label(&slot, true), "");
    }

    #[test]
    fn timeline_fx_enabled_chip_uses_two_letter_code_when_kind_title_is_hidden() {
        let slot = MidiFxSlot::default();
        assert_eq!(timeline_fx_enabled_chip_label(&slot, false), "TR");
    }

    #[test]
    fn timeline_fx_enabled_and_kind_rects_are_disjoint() {
        let app = App::new();
        let displayed = app.displayed_timeline_fx_slot_indices(MidiFxChainKind::Output);
        let layout = app.timeline_fx_row_layouts(
            Rect::new(10, 10, 120, 14),
            &displayed,
            &app.project.tracks[0].midi_fx.output_fx,
            Some(0),
        )[0];
        assert!(layout.enabled.x + layout.enabled.width() as i32 <= layout.kind.x);
    }

    #[test]
    fn timeline_fx_delete_chip_click_removes_effect() {
        let mut app = App::new();
        app.project.active_track_mut().unwrap().midi_fx.output_fx =
            vec![Some(MidiFxSlot::default()), None, None, None];
        app.page_state.selected_timeline_context = TimelineContext::OutputFx;
        app.set_selected_timeline_fx_row(MidiFxChainKind::Output, 0);
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
        let before = app
            .active_timeline_fx_slot_indices(MidiFxChainKind::Output)
            .len();

        let control = app.handle_timeline_pointer(
            content_bounds,
            layout.delete.x + layout.delete.width() as i32 / 2,
            layout.delete.y + layout.delete.height() as i32 / 2,
            ActionSource::Pointer,
        );

        assert_eq!(control, Some(AppControl::Continue));
        let after = app
            .active_timeline_fx_slot_indices(MidiFxChainKind::Output)
            .len();
        assert_eq!(after, before - 1);
    }

    #[test]
    fn output_fx_lower_empty_band_space_does_not_hit_row() {
        let mut app = App::new();
        app.project.tracks[0].midi_fx.output_fx =
            vec![Some(MidiFxSlot::default()), None, None, None];
        app.project.tracks[1].midi_fx.output_fx = vec![
            Some(MidiFxSlot::default()),
            Some(MidiFxSlot::default()),
            Some(MidiFxSlot::default()),
            None,
        ];
        let content_bounds = Rect::new(40, 40, 1200, 620);
        let (_, body_bounds) =
            crate::ui::split_top_strip(content_bounds, 28, 6).expect("timeline content");
        let (_, timeline_bounds) =
            crate::ui::split_top_strip(body_bounds, transport_strip_height(), 8)
                .expect("timeline body");
        let layout = app.visible_timeline_track_layouts(timeline_bounds)[0];
        let displayed = app.displayed_timeline_fx_slot_indices(MidiFxChainKind::Output);
        let row = app.timeline_fx_row_layouts(
            layout.output_fx_rect,
            &displayed,
            &app.project.tracks[0].midi_fx.output_fx,
            Some(0),
        )[0]
        .row;
        let x = row.x + row.width() as i32 / 2;
        let y = layout.output_fx_rect.y + layout.output_fx_rect.height() as i32 - 2;

        assert!(y > row.y + row.height() as i32);
        assert!(app
            .timeline_fx_hit(
                TimelineContext::OutputFx,
                layout.output_fx_rect,
                &app.project.tracks[0],
                x,
                y,
            )
            .is_none());
    }

    #[test]
    fn canonical_output_fx_row_point_does_not_land_in_body_content() {
        let app = App::new();
        let content_bounds = Rect::new(40, 40, 1200, 620);
        let (_, body_bounds) =
            crate::ui::split_top_strip(content_bounds, 28, 6).expect("timeline content");
        let (_, timeline_bounds) =
            crate::ui::split_top_strip(body_bounds, transport_strip_height(), 8)
                .expect("timeline body");
        let layout = app.visible_timeline_track_layouts(timeline_bounds)[0];
        let displayed = app.displayed_timeline_fx_slot_indices(MidiFxChainKind::Output);
        let row = app.timeline_fx_row_layouts(
            layout.output_fx_rect,
            &displayed,
            &app.project.tracks[0].midi_fx.output_fx,
            Some(0),
        )[0]
        .row;
        let x = row.x + row.width() as i32 / 2;
        let y = row.y + row.height() as i32 / 2;

        assert!(!super::rect_contains(layout.full_content_rect, x, y));
        assert!(!super::rect_contains(layout.detail_content_rect, x, y));
    }

    #[test]
    fn reverse_activate_page_item_moves_timeline_fx_field_backward() {
        let mut app = App::new();
        app.page_state.current_page = AppPage::Timeline;
        app.page_state.selected_timeline_context = TimelineContext::OutputFx;
        app.page_state.selected_timeline_fx_field = TimelineFxField::ParamSecondary;

        app.reverse_activate_page_item();

        assert_eq!(
            app.page_state.selected_timeline_fx_field,
            TimelineFxField::ParamPrimary
        );
    }

    #[test]
    fn shift_m_action_toggles_selected_timeline_fx_when_fx_context_is_active() {
        let mut app = App::new();
        app.page_state.current_page = AppPage::Timeline;
        app.page_state.selected_timeline_context = TimelineContext::OutputFx;

        let before = app
            .selected_timeline_fx_slot(app.project.active_track().unwrap(), MidiFxChainKind::Output)
            .unwrap()
            .enabled;
        app.apply_action(AppAction::ToggleSelectedRecordingClipMute);
        let after = app
            .selected_timeline_fx_slot(app.project.active_track().unwrap(), MidiFxChainKind::Output)
            .unwrap()
            .enabled;
        assert_ne!(before, after);
    }

    #[test]
    fn timeline_add_row_adjust_inserts_even_when_non_kind_field_was_selected() {
        let mut app = App::new();
        app.page_state.current_page = AppPage::Timeline;
        app.page_state.selected_timeline_context = TimelineContext::OutputFx;
        let existing = app
            .active_timeline_fx_slot_indices(MidiFxChainKind::Output)
            .len();
        let add_row = app
            .displayed_timeline_fx_slot_indices(MidiFxChainKind::Output)
            .len()
            - 1;
        app.set_selected_timeline_fx_row(MidiFxChainKind::Output, add_row);
        app.page_state.selected_timeline_fx_field = TimelineFxField::Move;

        app.adjust_page_item(1);

        let after = app
            .active_timeline_fx_slot_indices(MidiFxChainKind::Output)
            .len();
        assert_eq!(after, existing + 1);
    }

    #[test]
    fn timeline_add_row_activate_inserts_new_fx() {
        let mut app = App::new();
        app.page_state.current_page = AppPage::Timeline;
        app.page_state.selected_timeline_context = TimelineContext::OutputFx;
        let existing = app
            .active_timeline_fx_slot_indices(MidiFxChainKind::Output)
            .len();
        let add_row = app
            .displayed_timeline_fx_slot_indices(MidiFxChainKind::Output)
            .len()
            - 1;
        app.set_selected_timeline_fx_row(MidiFxChainKind::Output, add_row);
        app.page_state.selected_timeline_fx_field = TimelineFxField::ParamSecondary;

        app.activate_page_item();

        let after = app
            .active_timeline_fx_slot_indices(MidiFxChainKind::Output)
            .len();
        assert_eq!(after, existing + 1);
    }

    #[test]
    fn timeline_add_row_kind_adjust_inserts_new_fx_into_empty_slot() {
        let mut app = App::new();
        app.page_state.current_page = AppPage::Timeline;
        app.page_state.selected_timeline_context = TimelineContext::OutputFx;
        let existing = app
            .active_timeline_fx_slot_indices(MidiFxChainKind::Output)
            .len();
        let add_row = app
            .displayed_timeline_fx_slot_indices(MidiFxChainKind::Output)
            .len()
            - 1;
        app.set_selected_timeline_fx_row(MidiFxChainKind::Output, add_row);
        app.page_state.selected_timeline_fx_field = TimelineFxField::Kind;

        app.adjust_page_item(1);

        let after = app
            .active_timeline_fx_slot_indices(MidiFxChainKind::Output)
            .len();
        assert_eq!(after, existing + 1);
    }

    #[test]
    fn timeline_add_row_selects_newly_inserted_fx_row() {
        let mut app = App::new();
        app.project.active_track_mut().unwrap().midi_fx.output_fx = vec![
            Some(MidiFxSlot::default()),
            Some(MidiFxSlot::default()),
            None,
            None,
        ];
        app.page_state.current_page = AppPage::Timeline;
        app.page_state.selected_timeline_context = TimelineContext::OutputFx;
        let add_row = app
            .displayed_timeline_fx_slot_indices(MidiFxChainKind::Output)
            .len()
            - 1;
        app.set_selected_timeline_fx_row(MidiFxChainKind::Output, add_row);
        app.page_state.selected_timeline_fx_field = TimelineFxField::Kind;

        app.adjust_page_item(1);

        assert_eq!(
            app.selected_timeline_fx_slot_index(MidiFxChainKind::Output),
            Some(2)
        );
        assert_eq!(
            app.selected_timeline_fx_active_row_index(MidiFxChainKind::Output),
            Some(2)
        );
    }

    #[test]
    fn timeline_move_after_insert_from_add_row_does_not_panic() {
        let mut app = App::new();
        app.project.active_track_mut().unwrap().midi_fx.output_fx = vec![
            Some(MidiFxSlot::default()),
            Some(MidiFxSlot::default()),
            None,
            None,
        ];
        app.page_state.current_page = AppPage::Timeline;
        app.page_state.selected_timeline_context = TimelineContext::OutputFx;
        let add_row = app
            .displayed_timeline_fx_slot_indices(MidiFxChainKind::Output)
            .len()
            - 1;
        app.set_selected_timeline_fx_row(MidiFxChainKind::Output, add_row);
        app.page_state.selected_timeline_fx_field = TimelineFxField::Kind;
        app.adjust_page_item(1);

        app.page_state.selected_timeline_fx_field = TimelineFxField::Move;
        app.adjust_page_item(1);

        assert!(app
            .selected_timeline_fx_slot(app.project.active_track().unwrap(), MidiFxChainKind::Output)
            .is_some());
    }
}
