use super::*;
use crate::midi_fx::MidiFxInlineParam;

fn is_high_contrast_light(theme: &Theme) -> bool {
    theme.preset == ThemePreset::HighContrastLight
}

fn is_high_contrast_dark(theme: &Theme) -> bool {
    theme.preset == ThemePreset::HighContrastDark
}

impl App {
    pub(crate) fn timeline_fx_discoverability_targets_for_track(
        &self,
        track: &Track,
        context: TimelineContext,
        band: Rect,
    ) -> Vec<(Rect, DiscoverabilityTarget)> {
        let Some(chain_kind) = context.chain_kind() else {
            return Vec::new();
        };
        let chain = self.fx_chain(track, chain_kind);
        let rows = self.displayed_timeline_fx_slot_indices_for_track(track, chain_kind);
        let selected = match chain_kind {
            MidiFxChainKind::Input => track.midi_fx.timeline_ui.input_selected_row,
            MidiFxChainKind::Output => track.midi_fx.timeline_ui.output_selected_row,
        }
        .min(3);
        let mut targets = Vec::new();
        let mut add = |rect: Rect, action: AppAction| {
            if rect.x >= 0 {
                targets.push((
                    rect,
                    DiscoverabilityTarget {
                        action,
                        display_scope: Some("Active Track"),
                        allowed_mapping_scopes: &["Active Track"],
                        overlay_slot: None,
                    },
                ));
            }
        };
        for (i, layout) in self
            .timeline_fx_row_layouts(band, &rows, chain, Some(selected))
            .into_iter()
            .enumerate()
        {
            add(
                layout.kind,
                if chain[i].is_some() {
                    AppAction::CycleSelectedTimelineFxKind
                } else {
                    AppAction::AddSelectedTimelineFx
                },
            );
            if let Some(slot) = chain[i].as_ref() {
                for (p, action) in [
                    AppAction::AdjustSelectedTimelineFxPrimary,
                    AppAction::AdjustSelectedTimelineFxSecondary,
                    AppAction::AdjustSelectedTimelineFxThird,
                    AppAction::AdjustSelectedTimelineFxFourth,
                ]
                .into_iter()
                .enumerate()
                .take(slot.effect.inline_parameters().len())
                {
                    add(layout.parameters[p], action);
                }
                add(layout.move_up, AppAction::MoveSelectedTimelineFxUp);
                add(layout.move_down, AppAction::MoveSelectedTimelineFxDown);
            }
        }
        targets
    }

    pub(crate) fn select_timeline_fx_row(&mut self, delta: i32) {
        let Some(chain) = self.page_state.selected_timeline_context.chain_kind() else {
            return;
        };
        if self.page_state.selected_timeline_fx_field == TimelineFxField::ParamPrimary {
            self.scroll_selected_timeline_fx_parameter_window(delta);
        } else {
            let count = self.displayed_timeline_fx_slot_indices(chain).len();
            if count > 0 {
                let next = (self.selected_timeline_fx_row(chain) as i32 + delta)
                    .rem_euclid(count as i32) as usize;
                self.set_selected_timeline_fx_row(chain, next);
            }
        }
    }

    pub(crate) fn selected_timeline_fx_row(&self, chain_kind: MidiFxChainKind) -> usize {
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

    pub(crate) fn set_selected_timeline_fx_row(
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

    #[cfg(test)]
    pub(crate) fn active_timeline_fx_slot_indices(
        &self,
        chain_kind: MidiFxChainKind,
    ) -> Vec<usize> {
        let Some(track) = self.project.active_track() else {
            return Vec::new();
        };
        self.active_timeline_fx_slot_indices_for_track(track, chain_kind)
    }

    #[cfg(test)]
    pub(crate) fn active_timeline_fx_slot_indices_for_track(
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

    pub(crate) fn displayed_timeline_fx_slot_indices(
        &self,
        chain_kind: MidiFxChainKind,
    ) -> Vec<Option<usize>> {
        let Some(track) = self.project.active_track() else {
            return Vec::new();
        };
        self.displayed_timeline_fx_slot_indices_for_track(track, chain_kind)
    }

    pub(crate) fn displayed_timeline_fx_slot_indices_for_track(
        &self,
        track: &Track,
        chain_kind: MidiFxChainKind,
    ) -> Vec<Option<usize>> {
        (0..self.fx_chain(track, chain_kind).len().min(4))
            .map(Some)
            .collect()
    }

    pub(crate) fn selected_timeline_fx_slot_index(
        &self,
        chain_kind: MidiFxChainKind,
    ) -> Option<usize> {
        self.displayed_timeline_fx_slot_indices(chain_kind)
            .get(self.selected_timeline_fx_row(chain_kind))
            .copied()
            .flatten()
    }

    pub(crate) fn selected_timeline_fx_slot<'a>(
        &self,
        track: &'a Track,
        chain_kind: MidiFxChainKind,
    ) -> Option<&'a MidiFxSlot> {
        self.selected_timeline_fx_slot_index(chain_kind)
            .and_then(|slot_index| self.fx_chain(track, chain_kind).get(slot_index))
            .and_then(|slot| slot.as_ref())
    }

    pub(crate) fn selected_timeline_fx_param_window(&self, chain_kind: MidiFxChainKind) -> usize {
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

    pub(crate) fn timeline_fx_param_window_for_slot(
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

    pub(crate) fn set_selected_timeline_fx_param_window(
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

    pub(crate) fn normalize_timeline_fx_selection(&mut self) {
        if let Some(chain_kind) = self.page_state.selected_timeline_context.chain_kind() {
            let displayed = self.displayed_timeline_fx_slot_indices(chain_kind);
            if displayed.is_empty() {
                self.set_selected_timeline_fx_row(chain_kind, 0);
                return;
            }
            self.set_selected_timeline_fx_row(
                chain_kind,
                self.selected_timeline_fx_row(chain_kind),
            );
        }
    }

    pub(crate) fn adjust_timeline_context(&mut self, delta: i32) {
        let Some(chain) = self.page_state.selected_timeline_context.chain_kind() else {
            return;
        };
        if self.page_state.selected_timeline_fx_field == TimelineFxField::ParamPrimary {
            self.adjust_selected_timeline_fx_parameter(
                self.selected_timeline_fx_param_window(chain),
                delta,
            );
        } else {
            self.adjust_selected_timeline_fx_kind(delta);
        }
    }

    pub(crate) fn activate_timeline_context_item(&mut self) {
        if self
            .page_state
            .selected_timeline_context
            .chain_kind()
            .is_some()
        {
            self.page_state.selected_timeline_fx_field =
                if self.page_state.selected_timeline_fx_field == TimelineFxField::ParamPrimary {
                    TimelineFxField::Kind
                } else {
                    TimelineFxField::ParamPrimary
                };
        }
    }

    pub(crate) fn reverse_activate_timeline_context_item(&mut self) {
        self.activate_timeline_context_item();
    }

    pub(crate) fn toggle_selected_timeline_fx_enabled(&mut self) {
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

    pub(crate) fn adjust_selected_timeline_fx_kind(&mut self, delta: i32) {
        let Some(chain_kind) = self.page_state.selected_timeline_context.chain_kind() else {
            return;
        };
        let Some(index) = self.selected_timeline_fx_slot_index(chain_kind) else {
            return;
        };
        if let Some(track) = self.project.active_track_mut() {
            let chain = match chain_kind {
                MidiFxChainKind::Input => &mut track.midi_fx.input_fx,
                MidiFxChainKind::Output => &mut track.midi_fx.output_fx,
            };
            if let Some(entry) = chain.get_mut(index) {
                *entry = match entry.as_ref() {
                    Some(slot) => Some(cycle_existing_fx_kind(slot, delta)),
                    None => cycle_fx_kind(None, delta),
                };
            }
        }
        self.set_selected_timeline_fx_param_window(chain_kind, 0);
        self.handle_timeline_fx_configuration_changed();
    }

    pub(crate) fn add_selected_timeline_fx(&mut self) {
        let Some(chain) = self.page_state.selected_timeline_context.chain_kind() else {
            return;
        };
        if self
            .project
            .active_track()
            .and_then(|track| self.selected_timeline_fx_slot(track, chain))
            .is_none()
        {
            self.adjust_selected_timeline_fx_kind(1);
        }
    }

    pub(crate) fn delete_selected_timeline_fx(&mut self) {
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

    pub(crate) fn adjust_selected_timeline_fx_parameter(
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
        let parameter_index = visible_offset;
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

    pub(crate) fn scroll_selected_timeline_fx_parameter_window(&mut self, delta: i32) {
        let Some(chain) = self.page_state.selected_timeline_context.chain_kind() else {
            return;
        };
        let count = self
            .project
            .active_track()
            .and_then(|track| self.selected_timeline_fx_slot(track, chain))
            .map(|slot| slot.effect.inline_parameters().len())
            .unwrap_or(0);
        if count > 0 {
            let next = (self.selected_timeline_fx_param_window(chain) as i32 + delta)
                .rem_euclid(count as i32) as usize;
            self.set_selected_timeline_fx_param_window(chain, next);
        }
    }

    pub(crate) fn move_selected_timeline_fx(&mut self, delta: i32) {
        let Some(chain_kind) = self.page_state.selected_timeline_context.chain_kind() else {
            return;
        };
        let Some(source) = self.selected_timeline_fx_slot_index(chain_kind) else {
            return;
        };
        let count = self.displayed_timeline_fx_slot_indices(chain_kind).len();
        if count == 0 {
            return;
        }
        let target = (source as i32 + delta.signum()).clamp(0, count as i32 - 1) as usize;
        if source == target {
            return;
        }
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
            if chain[source].is_none() {
                return;
            }
            chain.swap(source, target);
            windows.swap(source, target);
        }
        self.set_selected_timeline_fx_row(chain_kind, target);
        self.handle_timeline_fx_configuration_changed();
    }

    pub(crate) fn draw_timeline_fx_row<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        context: TimelineContext,
        slot_index: usize,
        slot: &MidiFxSlot,
        layout: TimelineFxRowLayout,
        selected: bool,
        text_color: Color,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let theme = self.theme();
        let slot_focus =
            selected && self.page_state.selected_timeline_fx_field != TimelineFxField::ParamPrimary;
        let fill = if slot_focus {
            theme.io_pages.routing_affordance_selected_fill
        } else if slot.enabled {
            theme.io_pages.panel_bg
        } else {
            theme.io_pages.routing_toggle_off_fill
        };
        canvas.set_draw_color(fill);
        canvas.fill_rect(layout.row)?;
        draw_fx_icon(canvas, layout.row, slot, theme.text_on_fill(fill))?;
        canvas.set_draw_color(if selected {
            theme.io_pages.focus_border
        } else {
            theme.io_pages.row_idle_border
        });
        canvas.draw_rect(layout.row)?;
        if slot_focus {
            self.draw_fx_focus(canvas, layout.row, fill)?;
        }
        let params = slot.effect.inline_parameters();
        let active_param = self
            .timeline_fx_param_window_for_slot(context, slot_index)
            .min(params.len().saturating_sub(1));
        for (index, rect) in layout.parameters.into_iter().enumerate() {
            self.draw_timeline_fx_param_zone(
                canvas,
                rect,
                params.get(index),
                selected && !slot_focus && index == active_param,
                text_color,
            )?;
        }
        if selected && !slot_focus && layout.parameters[0].x >= 0 {
            canvas.set_draw_color(theme.io_pages.focus_border);
            canvas.draw_rect(crate::ui::union_rect(
                layout.parameters[0],
                layout.parameters[3],
            ))?;
        }
        self.draw_fx_reorder_arrows(canvas, layout, text_color)?;
        Ok(())
    }

    fn draw_fx_focus<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        rect: Rect,
        fill: Color,
    ) -> Result<(), Box<dyn std::error::Error>> {
        canvas.set_draw_color(self.theme().text_on_fill(fill));
        canvas.draw_rect(rect)?;
        if rect.width() > 4 && rect.height() > 4 {
            canvas.draw_rect(Rect::new(
                rect.x + 2,
                rect.y + 2,
                rect.width() - 4,
                rect.height() - 4,
            ))?;
        }
        Ok(())
    }

    fn draw_fx_reorder_arrows<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        layout: TimelineFxRowLayout,
        color: Color,
    ) -> Result<(), Box<dyn std::error::Error>> {
        canvas.set_draw_color(color);
        for (rect, up) in [(layout.move_up, true), (layout.move_down, false)] {
            if rect.x < 0 {
                continue;
            }
            let cx = rect.x + rect.width() as i32 / 2;
            let cy = rect.y + rect.height() as i32 / 2;
            for dy in 0..3 {
                let y = cy + if up { dy - 1 } else { 1 - dy };
                canvas.draw_line(
                    sdl3::rect::Point::new(cx - dy, y),
                    sdl3::rect::Point::new(cx + dy, y),
                )?;
            }
        }
        Ok(())
    }

    pub(crate) fn draw_timeline_fx_add_row<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        _context: TimelineContext,
        layout: TimelineFxRowLayout,
        selected: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let theme = self.theme();
        let focus =
            selected && self.page_state.selected_timeline_fx_field != TimelineFxField::ParamPrimary;
        let fill = if focus {
            theme.io_pages.routing_affordance_selected_fill
        } else {
            theme.io_pages.panel_bg
        };
        canvas.set_draw_color(fill);
        canvas.fill_rect(layout.row)?;
        canvas.set_draw_color(if selected {
            theme.io_pages.focus_border
        } else {
            theme.io_pages.row_idle_border
        });
        canvas.draw_rect(layout.row)?;
        let rect = Rect::new(
            layout.row.x + (layout.row.width() as i32 - 6) / 2,
            layout.row.y + (layout.row.height() as i32 - 7) / 2,
            6,
            7,
        );
        crate::ui::draw_text_fitted(canvas, "+", rect, 1, theme.text_on_fill(fill))?;
        if focus {
            self.draw_fx_focus(canvas, layout.row, fill)?;
        }
        for rect in layout.parameters {
            self.draw_timeline_fx_param_zone(canvas, rect, None, false, theme.text_on_fill(fill))?;
        }
        if selected && !focus && layout.parameters[0].x >= 0 {
            canvas.set_draw_color(theme.io_pages.focus_border);
            canvas.draw_rect(crate::ui::union_rect(
                layout.parameters[0],
                layout.parameters[3],
            ))?;
        }
        Ok(())
    }

    fn draw_timeline_fx_param_zone<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        rect: Rect,
        param: Option<&MidiFxInlineParam>,
        selected: bool,
        _text_color: Color,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if rect.x < 0 {
            return Ok(());
        }
        let theme = self.theme();
        let fill = if selected {
            theme.io_pages.routing_affordance_selected_fill
        } else {
            theme.io_pages.routing_adjust_fill
        };
        canvas.set_draw_color(fill);
        canvas.fill_rect(rect)?;
        canvas.set_draw_color(theme.io_pages.row_idle_border);
        canvas.draw_rect(rect)?;
        if let Some(param) = param {
            let scale = if rect.height() >= 24
                && crate::ui::text_width(&param.value, 2) + 6 <= rect.width()
            {
                2
            } else {
                1
            };
            let tw = crate::ui::text_width(&param.value, scale);
            crate::ui::draw_text_fitted(
                canvas,
                &param.value,
                Rect::new(
                    rect.x + ((rect.width() as i32 - tw as i32) / 2).max(3),
                    rect.y + ((rect.height() as i32 - 7 * scale as i32) / 2).max(0),
                    rect.width().saturating_sub(6),
                    7 * scale,
                ),
                scale,
                theme.text_on_fill(fill),
            )?;
        }
        if selected {
            self.draw_fx_focus(canvas, rect, fill)?;
        }
        Ok(())
    }

    pub(crate) fn timeline_fx_hit(
        &self,
        context: TimelineContext,
        band: Rect,
        track: &Track,
        x: i32,
        y: i32,
    ) -> Option<TimelineFxRowRef> {
        let chain_kind = context.chain_kind()?;
        let displayed = self.displayed_timeline_fx_slot_indices_for_track(track, chain_kind);
        let selected = match chain_kind {
            MidiFxChainKind::Input => track.midi_fx.timeline_ui.input_selected_row,
            MidiFxChainKind::Output => track.midi_fx.timeline_ui.output_selected_row,
        }
        .min(3);
        self.timeline_fx_row_layouts(
            band,
            &displayed,
            self.fx_chain(track, chain_kind),
            Some(selected),
        )
        .into_iter()
        .enumerate()
        .find_map(|(row_index, layout)| {
            (rect_contains(layout.row, x, y)
                || layout.parameters.iter().any(|r| rect_contains(*r, x, y))
                || rect_contains(layout.move_up, x, y)
                || rect_contains(layout.move_down, x, y))
            .then_some(TimelineFxRowRef {
                context,
                row_index,
                slot_index: displayed[row_index],
                layout,
            })
        })
    }

    pub(crate) fn handle_timeline_fx_pointer_hit(
        &mut self,
        hit: TimelineFxRowRef,
        x: i32,
        y: i32,
        source: ActionSource,
        was_selected: bool,
    ) -> Option<AppControl> {
        self.status_state.hovered_target = None;
        self.status_state.hovered_fx_detail = None;
        let layout = hit.layout;
        if rect_contains(layout.move_up, x, y) || rect_contains(layout.move_down, x, y) {
            self.page_state.selected_timeline_fx_field = TimelineFxField::Kind;
            return Some(self.apply_action_with_source(
                if rect_contains(layout.move_up, x, y) {
                    AppAction::MoveSelectedTimelineFxUp
                } else {
                    AppAction::MoveSelectedTimelineFxDown
                },
                source,
            ));
        }
        if let Some(index) = layout
            .parameters
            .iter()
            .position(|r| rect_contains(*r, x, y))
        {
            let chain = hit.context.chain_kind()?;
            let count = self
                .project
                .active_track()
                .and_then(|track| self.selected_timeline_fx_slot(track, chain))
                .map(|slot| slot.effect.inline_parameters().len())
                .unwrap_or(0);
            if index < count {
                self.page_state.selected_timeline_fx_field = TimelineFxField::ParamPrimary;
                self.set_selected_timeline_fx_param_window(chain, index);
                let cell = layout.parameters[index];
                let action = if x < cell.x + cell.width() as i32 / 2 {
                    AppAction::AdjustPageItemBackward
                } else {
                    AppAction::AdjustPageItemForward
                };
                return Some(self.apply_action_with_source(action, source));
            }
        } else {
            let cycle =
                was_selected && self.page_state.selected_timeline_fx_field == TimelineFxField::Kind;
            self.page_state.selected_timeline_fx_field = TimelineFxField::Kind;
            if cycle {
                return Some(
                    self.apply_action_with_source(AppAction::CycleSelectedTimelineFxKind, source),
                );
            }
        }
        Some(AppControl::Continue)
    }

    fn timeline_fx_card_height(&self) -> i32 {
        match self.ui_density_preset {
            UiDensityPreset::Default => 44,
            UiDensityPreset::Compact => 38,
            UiDensityPreset::Tiny => 32,
            UiDensityPreset::Touch => 52,
        }
    }

    pub(crate) fn timeline_fx_band_heights(&self) -> (i32, i32) {
        let input = self
            .project
            .tracks
            .iter()
            .map(|track| {
                displayed_track_fx_band_height(
                    &track.midi_fx.input_fx,
                    self.timeline_fx_card_height(),
                )
            })
            .max()
            .unwrap_or(displayed_track_fx_band_height(
                &[],
                self.timeline_fx_card_height(),
            ));
        let output = self
            .project
            .tracks
            .iter()
            .map(|track| {
                displayed_track_fx_band_height(
                    &track.midi_fx.output_fx,
                    self.timeline_fx_card_height(),
                )
            })
            .max()
            .unwrap_or(displayed_track_fx_band_height(
                &[],
                self.timeline_fx_card_height(),
            ));
        (input, output)
    }

    pub(crate) fn draw_track_fx_bands<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        layout: TimelineTrackLayout,
        track: &Track,
        is_active: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let theme = self.theme();
        let high_contrast = is_high_contrast_light(theme);
        let high_contrast_dark = is_high_contrast_dark(theme);
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
                    if high_contrast {
                        Color::RGB(244, 244, 244)
                    } else if high_contrast_dark {
                        Color::RGB(36, 36, 36)
                    } else {
                        Color::RGB(78, 128, 198)
                    }
                } else if is_active {
                    if high_contrast {
                        Color::RGB(236, 236, 236)
                    } else if high_contrast_dark {
                        Color::RGB(24, 24, 24)
                    } else {
                        Color::RGB(56, 70, 94)
                    }
                } else {
                    if high_contrast {
                        Color::RGB(248, 248, 248)
                    } else if high_contrast_dark {
                        Color::RGB(16, 16, 16)
                    } else {
                        Color::RGB(46, 56, 74)
                    }
                }
            } else if enabled {
                if high_contrast {
                    Color::RGB(244, 244, 244)
                } else if high_contrast_dark {
                    Color::RGB(36, 36, 36)
                } else {
                    Color::RGB(172, 108, 156)
                }
            } else if is_active {
                if high_contrast {
                    Color::RGB(236, 236, 236)
                } else if high_contrast_dark {
                    Color::RGB(24, 24, 24)
                } else {
                    Color::RGB(84, 68, 94)
                }
            } else {
                if high_contrast {
                    Color::RGB(248, 248, 248)
                } else if high_contrast_dark {
                    Color::RGB(16, 16, 16)
                } else {
                    Color::RGB(64, 58, 76)
                }
            };
            let border = if enabled {
                if high_contrast {
                    if context == TimelineContext::InputFx {
                        theme.app_chrome.tab_accent_midi_io
                    } else {
                        theme.app_chrome.tab_accent_routing
                    }
                } else if high_contrast_dark {
                    Color::RGB(255, 255, 255)
                } else {
                    Color::RGB(236, 238, 228)
                }
            } else if is_active {
                if high_contrast {
                    Color::RGB(0, 0, 0)
                } else if high_contrast_dark {
                    Color::RGB(255, 255, 255)
                } else {
                    Color::RGB(176, 184, 198)
                }
            } else {
                if high_contrast {
                    Color::RGB(128, 128, 128)
                } else if high_contrast_dark {
                    Color::RGB(96, 96, 96)
                } else {
                    Color::RGB(120, 126, 140)
                }
            };
            canvas.set_draw_color(fill);
            canvas.fill_rect(rect)?;
            canvas.set_draw_color(border);
            canvas.draw_rect(rect)?;

            let selected_row = match chain_kind {
                MidiFxChainKind::Input => track.midi_fx.timeline_ui.input_selected_row,
                MidiFxChainKind::Output => track.midi_fx.timeline_ui.output_selected_row,
            }
            .min(3);
            let layouts =
                self.timeline_fx_row_layouts(rect, &displayed_rows, chain, Some(selected_row));
            for (line_index, (display_row, layout)) in
                displayed_rows.iter().zip(layouts.iter()).enumerate()
            {
                let selected = is_active
                    && self.page_state.selected_timeline_context == context
                    && line_index == selected_row;
                if let Some((slot_index, slot)) =
                    display_row.and_then(|index| chain[index].as_ref().map(|slot| (index, slot)))
                {
                    let text_color = if high_contrast {
                        if slot.enabled {
                            Color::RGB(0, 0, 0)
                        } else {
                            Color::RGB(64, 64, 64)
                        }
                    } else if high_contrast_dark {
                        Color::RGB(255, 255, 255)
                    } else if slot.enabled {
                        Color::RGB(248, 244, 236)
                    } else {
                        Color::RGB(198, 202, 210)
                    };
                    self.draw_timeline_fx_row(
                        canvas, context, slot_index, slot, *layout, selected, text_color,
                    )?;
                } else {
                    self.draw_timeline_fx_add_row(canvas, context, *layout, selected)?;
                }
            }
        }
        Ok(())
    }

    pub(crate) fn timeline_fx_row_layouts(
        &self,
        band: Rect,
        displayed: &[Option<usize>],
        _chain: &[Option<MidiFxSlot>],
        selected: Option<usize>,
    ) -> Vec<TimelineFxRowLayout> {
        let hidden = Rect::new(-10000, -10000, 1, 1);
        let side = (band.height().saturating_sub(6) / 2)
            .max(7)
            .min((band.width().saturating_sub(24) / 4).max(7));
        let px = band.x + 2 + (side * 2 + 2) as i32 + 12;
        let pw = (band.right() - px - 2).max(2) as u32;
        let ph = band.height().saturating_sub(4).max(2);
        let chosen = selected.filter(|i| *i < displayed.len()).unwrap_or(0);
        displayed
            .iter()
            .enumerate()
            .map(|(i, _)| {
                let row = Rect::new(
                    band.x + 2 + (i % 2) as i32 * (side + 2) as i32,
                    band.y + 2 + (i / 2) as i32 * (side + 2) as i32,
                    side,
                    side,
                );
                let parameters = std::array::from_fn(|p| {
                    if i == chosen {
                        Rect::new(
                            px + (p % 2) as i32 * (pw / 2) as i32,
                            band.y + 2 + (p / 2) as i32 * (ph / 2) as i32,
                            if p % 2 == 0 { pw / 2 } else { pw - pw / 2 },
                            if p / 2 == 0 { ph / 2 } else { ph - ph / 2 },
                        )
                    } else {
                        hidden
                    }
                });
                TimelineFxRowLayout {
                    row,
                    kind: row,
                    enabled: hidden,
                    param_primary: parameters[0],
                    param_secondary: parameters[1],
                    parameters,
                    overflow: hidden,
                    move_up: if i == chosen {
                        Rect::new(px - 10, band.y + 2, 8, ph / 2)
                    } else {
                        hidden
                    },
                    move_down: if i == chosen {
                        Rect::new(px - 10, band.y + 2 + (ph / 2) as i32, 8, ph - ph / 2)
                    } else {
                        hidden
                    },
                    delete: hidden,
                }
            })
            .collect()
    }

    pub(crate) fn timeline_fx_hover_detail_at(&self, x: i32, y: i32) -> Option<String> {
        if self.page_state.current_page != AppPage::Timeline {
            return None;
        }
        let metrics = self.ui_metrics();
        let surface = crate::ui::surface_rect(self.viewport_size.0, self.viewport_size.1, metrics);
        let inset =
            crate::ui::inset_rect(surface, metrics.frame_inset_x_px, metrics.frame_inset_y_px)
                .ok()?;
        let (_, content, _) = self.page_frame_layout(inset).ok()?;
        let (_, _, timeline) = self.timeline_page_layout(content).ok()?;
        for layout in self.visible_timeline_track_layouts(timeline) {
            let track = &self.project.tracks[layout.track_index];
            for (context, band) in [
                (TimelineContext::InputFx, layout.input_fx_rect),
                (TimelineContext::OutputFx, layout.output_fx_rect),
            ] {
                let Some(hit) = self.timeline_fx_hit(context, band, track, x, y) else {
                    continue;
                };
                let slot_index = hit.slot_index?;
                let slot = self.fx_chain(track, context.chain_kind()?)[slot_index].as_ref();
                let detail = if let Some(slot) = slot {
                    let params = slot.effect.inline_parameters();
                    let parameter = hit
                        .layout
                        .parameters
                        .iter()
                        .position(|r| rect_contains(*r, x, y))
                        .and_then(|index| params.get(index));
                    if let Some(p) = parameter {
                        format!(
                            "{} | {}: {} | Left - / Right +",
                            slot.effect.kind().label(),
                            p.label,
                            p.value
                        )
                    } else {
                        format!(
                            "{} | {}",
                            slot.effect.kind().label(),
                            if slot.enabled { "Enabled" } else { "Bypassed" }
                        )
                    }
                } else {
                    "Empty slot | Select, then click again to add".to_string()
                };
                return Some(format!(
                    "Track {} | {} {} | {}",
                    layout.track_index + 1,
                    context.label(),
                    slot_index + 1,
                    detail
                ));
            }
        }
        None
    }

    pub(crate) fn timeline_fx_footer_content(&self) -> Option<(String, String)> {
        if self.page_state.current_page != AppPage::Timeline {
            return None;
        }
        let context = self.page_state.selected_timeline_context;
        let chain = context.chain_kind()?;
        let track = self.project.active_track()?;
        let index = self.selected_timeline_fx_row(chain);
        let params_focus =
            self.page_state.selected_timeline_fx_field == TimelineFxField::ParamPrimary;
        let label = if let Some(slot) = self.selected_timeline_fx_slot(track, chain) {
            let params = slot.effect.inline_parameters();
            let parameter = self
                .selected_timeline_fx_param_window(chain)
                .min(params.len().saturating_sub(1));
            let control = if params_focus {
                params
                    .get(parameter)
                    .map(|p| format!("{}: {}", p.label, p.value))
                    .unwrap_or_default()
            } else {
                "Effect type".to_string()
            };
            format!(
                "{} {} {}{} | {}",
                context.label(),
                index + 1,
                slot.effect.kind().label(),
                if slot.enabled { "" } else { " [BYPASS]" },
                control
            )
        } else {
            format!(
                "{} {} Empty slot{}",
                context.label(),
                index + 1,
                if params_focus { " | No parameters" } else { "" }
            )
        };
        Some((label, if params_focus {"Shift+Up/Down ctx  Shift+1-4 FX  Up/Down param  Q/E value  Enter FX  Shift+M bypass"} else {"Shift+Up/Down ctx  Shift+1-4 slot  Q/E kind  Enter params  Ctrl+Up/Down move  Shift+M bypass  Del remove"}.to_string()))
    }
}

// Seven-pixel pictograms use the same integer grid as Trekr's bitmap lettering.
pub(super) fn draw_fx_icon<T: RenderTarget>(
    canvas: &mut Canvas<T>,
    rect: Rect,
    slot: &MidiFxSlot,
    color: Color,
) -> Result<(), Box<dyn std::error::Error>> {
    use crate::midi_fx::MidiFxKind;
    if rect.height() < 7 {
        crate::ui::draw_text_fitted(
            canvas,
            timeline_fx_enabled_chip_label(slot, false),
            rect,
            1,
            color,
        )?;
        return Ok(());
    }
    let rows: [u8; 7] = match slot.effect.kind() {
        MidiFxKind::Arp => [
            0b0000011, 0b0000010, 0b0001110, 0b0001000, 0b0111000, 0b0100000, 0b1100000,
        ],
        MidiFxKind::NoteFilter => [
            0b1111111, 0b0100010, 0b0010100, 0b0001000, 0b0001000, 0b0001000, 0b0001000,
        ],
        MidiFxKind::Transpose => [
            0b0100000, 0b1110000, 0b0100010, 0b0100010, 0b0100010, 0b0000111, 0b0000010,
        ],
        MidiFxKind::Velocity => [
            0b0000001, 0b0000001, 0b0000101, 0b0000101, 0b0010101, 0b0010101, 0b1010101,
        ],
        MidiFxKind::Duration => [
            0b1000001, 0b1000001, 0b1000001, 0b1111111, 0b1000001, 0b1000001, 0b1000001,
        ],
        MidiFxKind::ScaleFilter => [
            0b1010101, 0b0000000, 0b1111111, 0b0010100, 0b0001000, 0b0001000, 0b0011100,
        ],
        MidiFxKind::ScaleQuantize => [
            0b0000111, 0b0000100, 0b0011100, 0b0010000, 0b1110000, 0b0000000, 0b1010101,
        ],
        MidiFxKind::ChordQuantize => [
            0b0101010, 0b0101010, 0b0101010, 0b0101010, 0b1101010, 0b0011010, 0b0000110,
        ],
        MidiFxKind::Delay => [
            0b0011100, 0b0100010, 0b1001001, 0b1001101, 0b1000001, 0b0100010, 0b0011100,
        ],
        MidiFxKind::TrackClone => [
            0b1111100, 0b1000100, 0b1011111, 0b1010101, 0b1111101, 0b0010001, 0b0011111,
        ],
    };
    let scale = if rect.height() >= 16 && rect.width() >= 16 {
        2
    } else {
        1
    };
    let x = rect.x + (rect.width() as i32 - 7 * scale) / 2;
    let y = rect.y + (rect.height() as i32 - 7 * scale) / 2;
    canvas.set_draw_color(color);
    for (row, bits) in rows.into_iter().enumerate() {
        for col in 0..7 {
            if bits & (1 << (6 - col)) != 0 {
                canvas.fill_rect(Rect::new(
                    x + col * scale,
                    y + row as i32 * scale,
                    scale as u32,
                    scale as u32,
                ))?;
            }
        }
    }
    if !slot.enabled {
        canvas.draw_line(
            sdl3::rect::Point::new(rect.x, rect.bottom() - 1),
            sdl3::rect::Point::new(rect.right() - 1, rect.y),
        )?;
    }
    Ok(())
}

pub(crate) fn timeline_fx_enabled_chip_label(
    slot: &MidiFxSlot,
    show_kind_title: bool,
) -> &'static str {
    if show_kind_title {
        ""
    } else {
        slot.effect.kind().compact_label()
    }
}

#[cfg(test)]
pub(crate) fn timeline_fx_kind_display(slot: &MidiFxSlot, width: u32) -> &'static str {
    if crate::ui::text_width(slot.effect.kind().label(), 1) + 4 <= width {
        slot.effect.kind().label()
    } else if width >= 20 {
        slot.effect.kind().short_label()
    } else {
        slot.effect.kind().compact_label()
    }
}

pub(crate) fn timeline_fx_overflow_label(param_count: usize, window_start: usize) -> String {
    if param_count <= 2 {
        "--".to_string()
    } else {
        let window_count = param_count.saturating_sub(1).max(1);
        format!("{}/{}", window_start + 1, window_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fx_cards_keep_all_visible_hit_targets_disjoint() {
        let app = App::new();
        let chain = vec![Some(MidiFxSlot {
            enabled: true,
            effect: MidiFx::default_for_kind(crate::midi_fx::MidiFxKind::Arp),
        })];
        for height in [14, 38] {
            for width in [120, 184, 300] {
                let layout = app.timeline_fx_row_layouts(
                    Rect::new(10, 10, width, height),
                    &[Some(0)],
                    &chain,
                    Some(0),
                )[0];
                let controls = [
                    layout.enabled,
                    layout.kind,
                    layout.param_primary,
                    layout.param_secondary,
                    layout.overflow,
                    layout.move_up,
                    layout.move_down,
                    layout.delete,
                ];
                for (i, a) in controls.iter().enumerate() {
                    if a.x < layout.row.x {
                        continue;
                    }
                    for b in &controls[i + 1..] {
                        if b.x >= layout.row.x {
                            assert!(
                                !super::super::layout::rects_overlap(*a, *b),
                                "overlap at {width}x{height}: {a:?}, {b:?}"
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn fx_slots_keep_geometry_constant_as_the_chain_fills() {
        let mut app = App::new();
        for (density, height) in [
            (UiDensityPreset::Default, 48),
            (UiDensityPreset::Compact, 42),
            (UiDensityPreset::Tiny, 36),
            (UiDensityPreset::Touch, 56),
        ] {
            app.set_ui_density_preset(density);
            let mut baseline = None;
            for count in 0..=4 {
                let chain = (0..4)
                    .map(|i| (i < count).then(MidiFxSlot::default))
                    .collect::<Vec<_>>();
                app.project.tracks[0].midi_fx.output_fx = chain.clone();
                assert_eq!(app.timeline_fx_band_heights().1, height);
                let rows = app.timeline_fx_row_layouts(
                    Rect::new(10, 10, 184, height as u32),
                    &[Some(0), Some(1), Some(2), Some(3)],
                    &chain,
                    Some(0),
                );
                if let Some(previous) = &baseline {
                    assert_eq!(previous, &rows);
                } else {
                    baseline = Some(rows.clone());
                }
                assert_eq!(rows.len(), 4);
                for r in &rows {
                    assert_eq!(r.row.width(), r.row.height());
                    assert!(r.row.bottom() <= 10 + height);
                }
                assert_eq!(rows[0].row.y, rows[1].row.y);
                assert_eq!(rows[2].row.y, rows[3].row.y);
                assert!(rows[2].row.y > rows[0].row.y);
            }
        }
    }

    #[test]
    fn fx_inspector_exposes_all_parameters_without_overlaps() {
        let app = App::new();
        for kind in crate::midi_fx::MidiFxKind::ALL {
            let chain = vec![
                Some(MidiFxSlot {
                    enabled: true,
                    effect: MidiFx::default_for_kind(kind)
                });
                4
            ];
            for selected in 0..4 {
                let rows = app.timeline_fx_row_layouts(
                    Rect::new(10, 10, 184, 48),
                    &[Some(0), Some(1), Some(2), Some(3)],
                    &chain,
                    Some(selected),
                );
                let mut controls = rows.iter().map(|r| r.row).collect::<Vec<_>>();
                controls.extend(rows[selected].parameters);
                controls.extend([rows[selected].move_up, rows[selected].move_down]);
                assert!(chain[0].as_ref().unwrap().effect.inline_parameters().len() <= 4);
                for (i, a) in controls.iter().enumerate() {
                    assert!(a.x >= 10 && a.right() <= 194 && a.y >= 10 && a.bottom() <= 58);
                    for b in &controls[i + 1..] {
                        assert!(!super::super::layout::rects_overlap(*a, *b));
                    }
                }
            }
        }
    }

    #[test]
    fn fx_focus_is_visible_on_icon_and_each_parameter() {
        let slot = MidiFxSlot {
            enabled: true,
            effect: MidiFx::default_for_kind(crate::midi_fx::MidiFxKind::Arp),
        };
        for parameter in 0..3 {
            for field in [TimelineFxField::Kind, TimelineFxField::ParamPrimary] {
                let mut app = App::new();
                app.page_state.selected_timeline_fx_field = field;
                app.project.tracks[0]
                    .midi_fx
                    .timeline_ui
                    .output_param_windows[0] = parameter;
                let layout = app.timeline_fx_row_layouts(
                    Rect::new(10, 10, 184, 48),
                    &[Some(0)],
                    &[Some(slot.clone())],
                    Some(0),
                )[0];
                let mut canvas =
                    sdl3::surface::Surface::new(220, 80, sdl3::pixels::PixelFormat::RGBA32)
                        .unwrap()
                        .into_canvas()
                        .unwrap();
                app.draw_timeline_fx_row(
                    &mut canvas,
                    TimelineContext::OutputFx,
                    0,
                    &slot,
                    layout,
                    true,
                    app.theme().text_on_dark,
                )
                .unwrap();
                canvas.present();
                let focus = if field == TimelineFxField::Kind {
                    layout.kind
                } else {
                    layout.parameters[parameter]
                };
                let readback = crate::app::capture::readback_rect_rgba(&canvas, focus, (220, 80));
                assert_eq!(
                    crate::app::capture::readback_color_at(&readback, focus.x + 2, focus.y + 2),
                    Some(
                        app.theme()
                            .text_on_fill(app.theme().io_pages.routing_affordance_selected_fill)
                    )
                );
            }
        }
    }

    #[test]
    fn fx_tiles_render_a_distinct_icon_for_every_effect() {
        let app = App::new();
        let mut signatures = std::collections::HashSet::new();
        for kind in crate::midi_fx::MidiFxKind::ALL {
            let slot = MidiFxSlot {
                enabled: true,
                effect: MidiFx::default_for_kind(kind),
            };
            let layout = app.timeline_fx_row_layouts(
                Rect::new(10, 10, 184, 48),
                &[Some(0)],
                &[Some(slot.clone())],
                None,
            )[0];
            let surface =
                sdl3::surface::Surface::new(220, 80, sdl3::pixels::PixelFormat::RGBA32).unwrap();
            let mut canvas = surface.into_canvas().unwrap();
            app.draw_timeline_fx_row(
                &mut canvas,
                TimelineContext::OutputFx,
                0,
                &slot,
                layout,
                false,
                app.theme().text_on_dark,
            )
            .unwrap();
            canvas.present();
            let readback = crate::app::capture::readback_rect_rgba(&canvas, layout.kind, (220, 80));
            let mut signature = Vec::new();
            for y in layout.kind.y..layout.kind.bottom() {
                for x in layout.kind.x..layout.kind.right() {
                    let c = crate::app::capture::readback_color_at(&readback, x, y).unwrap();
                    signature.push((c.r, c.g, c.b));
                }
            }
            assert!(signatures.insert(signature), "duplicate icon for {kind:?}");
        }
    }

    #[test]
    fn fx_tiles_populated_lane_renders_at_every_density() {
        use crate::midi_fx::MidiFxKind;
        for density in [
            UiDensityPreset::Default,
            UiDensityPreset::Compact,
            UiDensityPreset::Tiny,
            UiDensityPreset::Touch,
        ] {
            let mut app = App::new();
            app.set_ui_density_preset(density);
            app.seed_capture_demo_timeline_overlaps();
            app.project.tracks[0].midi_fx.output_fx = [
                MidiFxKind::Arp,
                MidiFxKind::ScaleFilter,
                MidiFxKind::ScaleQuantize,
                MidiFxKind::Delay,
            ]
            .into_iter()
            .enumerate()
            .map(|(i, kind)| {
                Some(MidiFxSlot {
                    enabled: i != 1,
                    effect: MidiFx::default_for_kind(kind),
                })
            })
            .collect();
            app.project.tracks[0].midi_fx.input_fx = [
                MidiFxKind::NoteFilter,
                MidiFxKind::Transpose,
                MidiFxKind::Velocity,
                MidiFxKind::Duration,
            ]
            .into_iter()
            .map(|kind| {
                Some(MidiFxSlot {
                    enabled: true,
                    effect: MidiFx::default_for_kind(kind),
                })
            })
            .collect();
            app.project.tracks[1].midi_fx.output_fx =
                [MidiFxKind::ChordQuantize, MidiFxKind::TrackClone]
                    .into_iter()
                    .map(|kind| {
                        Some(MidiFxSlot {
                            enabled: true,
                            effect: MidiFx::default_for_kind(kind),
                        })
                    })
                    .chain([None, None])
                    .collect();
            app.page_state.selected_timeline_context = TimelineContext::OutputFx;
            app.page_state.selected_timeline_fx_field = TimelineFxField::ParamPrimary;
            app.set_selected_timeline_fx_param_window(MidiFxChainKind::Output, 2);
            app.set_selected_timeline_fx_row(MidiFxChainKind::Output, 0);
            let surface =
                sdl3::surface::Surface::new(1280, 720, sdl3::pixels::PixelFormat::RGBA32).unwrap();
            let mut canvas = surface.into_canvas().unwrap();
            app.draw(&mut canvas).unwrap();
            let displayed = app.displayed_timeline_fx_slot_indices(MidiFxChainKind::Output);
            let tiles = app.timeline_fx_row_layouts(
                Rect::new(10, 10, 184, app.timeline_fx_band_heights().1 as u32),
                &displayed,
                &app.project.tracks[0].midi_fx.output_fx,
                Some(0),
            );
            assert_eq!(tiles[0].row.y, tiles[1].row.y);
            assert!(tiles[2].row.y > tiles[0].row.y);
            if let Some(dir) = std::env::var_os("TREKR_FX_REVIEW_DIR") {
                let path = std::path::PathBuf::from(dir);
                std::fs::create_dir_all(&path).unwrap();
                app.capture_surface_to_png(
                    canvas.surface(),
                    &path.join(format!("fx-lane-{}.png", density.label())),
                )
                .unwrap();
            }
        }
    }

    #[test]
    fn fx_card_names_expand_when_the_full_name_fits() {
        let slot = MidiFxSlot {
            enabled: true,
            effect: MidiFx::default_for_kind(crate::midi_fx::MidiFxKind::ScaleQuantize),
        };
        assert_eq!(timeline_fx_kind_display(&slot, 100), "Scale Quantize");
        assert_eq!(timeline_fx_kind_display(&slot, 25), "SQT");
    }

    use crate::actions::ActionSource;

    #[test]
    fn timeline_track_fx_row_click_selects_output_fx_context() {
        let mut app = App::new();
        let content_bounds = Rect::new(40, 40, 1200, 620);
        let (_, body_bounds) =
            crate::ui::split_top_strip(content_bounds, 28, 6).expect("timeline content");
        let (_, timeline_bounds) =
            crate::ui::split_top_strip(body_bounds, transport_strip_height(app.ui_metrics()), 8)
                .expect("timeline body");
        let columns = crate::ui::track_column_pairs(
            timeline_bounds,
            app.project.tracks.len(),
            app.ui_metrics(),
        );
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
    fn timeline_fx_icon_click_selects_without_mutating_effect() {
        let mut app = App::new();
        app.page_state.selected_timeline_context = TimelineContext::OutputFx;
        let before = app.project.tracks[0].midi_fx.output_fx[0].clone();
        let row = app.timeline_fx_row_layouts(
            Rect::new(10, 10, 184, 48),
            &[Some(0)],
            &app.project.tracks[0].midi_fx.output_fx,
            Some(0),
        )[0];
        let hit = TimelineFxRowRef {
            context: TimelineContext::OutputFx,
            row_index: 0,
            slot_index: Some(0),
            layout: row,
        };
        app.handle_timeline_fx_pointer_hit(
            hit,
            row.row.x + 5,
            row.row.y + 5,
            ActionSource::Pointer,
            false,
        );
        assert_eq!(
            app.page_state.selected_timeline_fx_field,
            TimelineFxField::Kind
        );
        assert_eq!(app.project.tracks[0].midi_fx.output_fx[0], before);
    }

    #[test]
    fn timeline_empty_slot_click_selects_without_inserting() {
        let mut app = App::new();
        app.page_state.selected_timeline_context = TimelineContext::OutputFx;
        app.project.tracks[0].midi_fx.output_fx = vec![None; 4];
        app.set_selected_timeline_fx_row(MidiFxChainKind::Output, 3);
        let layout = app.timeline_fx_row_layouts(
            Rect::new(10, 10, 184, 48),
            &[Some(0), Some(1), Some(2), Some(3)],
            &app.project.tracks[0].midi_fx.output_fx,
            Some(3),
        )[3];
        app.handle_timeline_fx_pointer_hit(
            TimelineFxRowRef {
                context: TimelineContext::OutputFx,
                row_index: 3,
                slot_index: Some(3),
                layout,
            },
            layout.row.x + 5,
            layout.row.y + 5,
            ActionSource::Pointer,
            false,
        );
        assert!(
            app.project.tracks[0]
                .midi_fx
                .output_fx
                .iter()
                .all(Option::is_none)
        );
        assert_eq!(app.selected_timeline_fx_row(MidiFxChainKind::Output), 3);
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
            crate::ui::split_top_strip(body_bounds, transport_strip_height(app.ui_metrics()), 8)
                .expect("timeline body");
        let columns = crate::ui::track_column_pairs(
            timeline_bounds,
            app.project.tracks.len(),
            app.ui_metrics(),
        );
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
    fn timeline_inspector_position_is_independent_of_selected_slot() {
        let app = App::new();
        let mut previous = None;
        for i in 0..4 {
            let rows = app.timeline_fx_row_layouts(
                Rect::new(10, 10, 184, 48),
                &[Some(0), Some(1), Some(2), Some(3)],
                &app.project.tracks[0].midi_fx.output_fx,
                Some(i),
            );
            if let Some(p) = previous {
                assert_eq!(rows[i].parameters, p);
            }
            previous = Some(rows[i].parameters);
        }
    }

    #[test]
    fn timeline_fx_layout_fits_supported_density_widths() {
        let app = App::new();
        for width in [120, 184, 300] {
            for height in [36, 42, 48, 56] {
                let band = Rect::new(10, 10, width, height);
                let rows = app.timeline_fx_row_layouts(
                    band,
                    &[Some(0), Some(1), Some(2), Some(3)],
                    &app.project.tracks[0].midi_fx.output_fx,
                    Some(2),
                );
                for rect in rows.iter().map(|r| r.row).chain(rows[2].parameters) {
                    assert!(
                        rect.x >= band.x
                            && rect.right() <= band.right()
                            && rect.bottom() <= band.bottom()
                    );
                }
            }
        }
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
            crate::ui::split_top_strip(body_bounds, transport_strip_height(app.ui_metrics()), 8)
                .expect("timeline body");
        let columns = crate::ui::track_column_pairs(
            timeline_bounds,
            app.project.tracks.len(),
            app.ui_metrics(),
        );
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
    fn timeline_fx_parameters_follow_reading_order() {
        let app = App::new();
        let row = app.timeline_fx_row_layouts(
            Rect::new(10, 10, 184, 48),
            &[Some(0)],
            &app.project.tracks[0].midi_fx.output_fx,
            Some(0),
        )[0];
        assert_eq!(row.parameters[0].y, row.parameters[1].y);
        assert_eq!(row.parameters[2].y, row.parameters[3].y);
        assert_eq!(row.parameters[0].x, row.parameters[2].x);
        assert!(row.parameters[2].y > row.parameters[0].y);
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
    fn timeline_fx_reorder_triangle_uses_same_swap_action() {
        let mut app = App::new();
        app.page_state.selected_timeline_context = TimelineContext::OutputFx;
        app.project.tracks[0].midi_fx.output_fx =
            vec![Some(MidiFxSlot::default()), None, None, None];
        let layout = app.timeline_fx_row_layouts(
            Rect::new(10, 10, 184, 48),
            &[Some(0), Some(1), Some(2), Some(3)],
            &app.project.tracks[0].midi_fx.output_fx,
            Some(0),
        )[0];
        app.handle_timeline_fx_pointer_hit(
            TimelineFxRowRef {
                context: TimelineContext::OutputFx,
                row_index: 0,
                slot_index: Some(0),
                layout,
            },
            layout.move_down.x + 2,
            layout.move_down.y + 2,
            ActionSource::Pointer,
            true,
        );
        assert!(app.project.tracks[0].midi_fx.output_fx[0].is_none());
        assert!(app.project.tracks[0].midi_fx.output_fx[1].is_some());
        assert_eq!(app.selected_timeline_fx_row(MidiFxChainKind::Output), 1);
    }

    #[test]
    fn output_fx_bottom_padding_does_not_hit_tile() {
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
            crate::ui::split_top_strip(body_bounds, transport_strip_height(app.ui_metrics()), 8)
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

        assert!(y >= row.y + row.height() as i32);
        assert!(
            app.timeline_fx_hit(
                TimelineContext::OutputFx,
                layout.output_fx_rect,
                &app.project.tracks[0],
                x,
                y,
            )
            .is_none()
        );
    }

    #[test]
    fn canonical_output_fx_row_point_does_not_land_in_body_content() {
        let app = App::new();
        let content_bounds = Rect::new(40, 40, 1200, 620);
        let (_, body_bounds) =
            crate::ui::split_top_strip(content_bounds, 28, 6).expect("timeline content");
        let (_, timeline_bounds) =
            crate::ui::split_top_strip(body_bounds, transport_strip_height(app.ui_metrics()), 8)
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
    fn timeline_empty_slot_enter_switches_focus_without_inserting() {
        let mut app = App::new();
        app.page_state.selected_timeline_context = TimelineContext::OutputFx;
        app.page_state.selected_timeline_fx_field = TimelineFxField::Kind;
        app.project.tracks[0].midi_fx.output_fx = vec![None; 4];
        app.apply_action(AppAction::ActivatePageItem);
        assert_eq!(
            app.page_state.selected_timeline_fx_field,
            TimelineFxField::ParamPrimary
        );
        assert!(
            app.project.tracks[0]
                .midi_fx
                .output_fx
                .iter()
                .all(Option::is_none)
        );
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
    fn timeline_add_action_fills_selected_hole_without_moving_selection() {
        let mut app = App::new();
        app.page_state.selected_timeline_context = TimelineContext::OutputFx;
        app.project.tracks[0].midi_fx.output_fx =
            vec![Some(MidiFxSlot::default()), None, None, None];
        app.set_selected_timeline_fx_row(MidiFxChainKind::Output, 3);
        app.apply_action(AppAction::AddSelectedTimelineFx);
        assert!(app.project.tracks[0].midi_fx.output_fx[3].is_some());
        assert!(app.project.tracks[0].midi_fx.output_fx[1].is_none());
        assert_eq!(app.selected_timeline_fx_row(MidiFxChainKind::Output), 3);
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

        assert!(
            app.selected_timeline_fx_slot(
                app.project.active_track().unwrap(),
                MidiFxChainKind::Output
            )
            .is_some()
        );
    }

    #[test]
    fn timeline_kind_adjust_keeps_existing_row_visible() {
        let mut app = App::new();
        app.page_state.current_page = AppPage::Timeline;
        app.page_state.selected_timeline_context = TimelineContext::OutputFx;
        app.page_state.selected_timeline_fx_field = TimelineFxField::Kind;

        for _ in 0..16 {
            app.adjust_page_item(-1);
            assert!(
                app.selected_timeline_fx_slot(
                    app.project.active_track().unwrap(),
                    MidiFxChainKind::Output
                )
                .is_some()
            );
        }
    }
}
#[cfg(test)]
mod split_fx_tests {
    use super::*;
    fn app() -> App {
        let mut app = App::new();
        app.page_state.selected_timeline_context = TimelineContext::OutputFx;
        app.page_state.selected_timeline_fx_field = TimelineFxField::Kind;
        app.project.tracks[0].midi_fx.output_fx = vec![None; 4];
        app
    }
    #[test]
    fn split_fx_keeps_empty_slots_addressable() {
        let mut app = app();
        assert_eq!(
            app.displayed_timeline_fx_slot_indices(MidiFxChainKind::Output),
            vec![Some(0), Some(1), Some(2), Some(3)]
        );
        app.set_selected_timeline_fx_row(MidiFxChainKind::Output, 2);
        app.apply_action(AppAction::AdjustPageItemForward);
        assert!(app.project.tracks[0].midi_fx.output_fx[2].is_some());
        assert!(app.project.tracks[0].midi_fx.output_fx[0].is_none());
        app.apply_action(AppAction::DeleteSelectedTimelineFx);
        assert_eq!(app.selected_timeline_fx_row(MidiFxChainKind::Output), 2);
        assert!(
            app.project.tracks[0]
                .midi_fx
                .output_fx
                .iter()
                .all(Option::is_none)
        );
    }
    #[test]
    fn split_fx_enter_switches_areas_and_up_down_select_parameters() {
        let mut app = app();
        app.project.tracks[0].midi_fx.output_fx[0] = Some(MidiFxSlot {
            enabled: true,
            effect: MidiFx::default_for_kind(crate::midi_fx::MidiFxKind::Arp),
        });
        app.apply_action(AppAction::ActivatePageItem);
        app.apply_action(AppAction::SelectNextPageItem);
        app.apply_action(AppAction::SelectNextPageItem);
        assert_eq!(app.selected_timeline_fx_row(MidiFxChainKind::Output), 0);
        let before = app.project.tracks[0].midi_fx.output_fx[0]
            .as_ref()
            .unwrap()
            .effect
            .inline_parameters();
        app.apply_action(AppAction::AdjustPageItemBackward);
        let after = app.project.tracks[0].midi_fx.output_fx[0]
            .as_ref()
            .unwrap()
            .effect
            .inline_parameters();
        assert_eq!(before[0].value, after[0].value);
        assert_ne!(before[2].value, after[2].value);
        app.apply_action(AppAction::ActivatePageItem);
        assert_eq!(
            app.page_state.selected_timeline_fx_field,
            TimelineFxField::Kind
        );
        app.apply_action(AppAction::ActivatePageItem);
        assert_eq!(
            app.selected_timeline_fx_param_window(MidiFxChainKind::Output),
            2
        );
    }
    #[test]
    fn split_fx_reorder_swaps_into_empty_slot_and_follows_effect() {
        let mut app = app();
        app.project.tracks[0].midi_fx.output_fx[0] = Some(MidiFxSlot::default());
        app.apply_action(AppAction::MoveSelectedTimelineFxDown);
        assert!(app.project.tracks[0].midi_fx.output_fx[0].is_none());
        assert!(app.project.tracks[0].midi_fx.output_fx[1].is_some());
        assert_eq!(app.selected_timeline_fx_row(MidiFxChainKind::Output), 1);
    }
    #[test]
    fn split_fx_layout_has_two_square_slot_rows_and_fixed_inspector() {
        let app = app();
        let chain = &app.project.tracks[0].midi_fx.output_fx;
        let rows = app.timeline_fx_row_layouts(
            Rect::new(10, 10, 184, 48),
            &[Some(0), Some(1), Some(2), Some(3)],
            chain,
            Some(0),
        );
        assert_eq!(rows[0].row.width(), rows[0].row.height());
        assert_eq!(rows[0].row.y, rows[1].row.y);
        assert!(rows[2].row.y > rows[0].row.y);
        assert_eq!(rows[0].row.x, rows[2].row.x);
        assert!(rows[0].param_primary.x > rows[1].row.right());
    }
}

#[cfg(test)]
mod split_fx_input_tests {
    use super::*;
    fn key(app: &mut App, code: sdl3::keyboard::Keycode, shift: bool) {
        app.handle_keyboard_event(&sdl3::event::Event::KeyDown {
            timestamp: 0,
            window_id: 0,
            which: 0,
            raw: 0,
            keycode: Some(code),
            scancode: None,
            keymod: if shift {
                sdl3::keyboard::Mod::LSHIFTMOD
            } else {
                sdl3::keyboard::Mod::NOMOD
            },
            repeat: false,
        });
    }
    #[test]
    fn split_fx_keyboard_flow_preserves_track_and_context_navigation() {
        use sdl3::keyboard::Keycode as K;
        let mut app = App::new();
        app.mappings.clear();
        app.page_state.selected_timeline_context = TimelineContext::OutputFx;
        app.page_state.selected_timeline_fx_field = TimelineFxField::Kind;
        app.project.tracks[0].midi_fx.output_fx = vec![None; 4];
        key(&mut app, K::Down, false);
        key(&mut app, K::E, false);
        assert!(app.project.tracks[0].midi_fx.output_fx[1].is_some());
        app.handle_keyboard_event(&sdl3::event::Event::KeyDown {
            timestamp: 0,
            window_id: 0,
            which: 0,
            raw: 0,
            keycode: Some(K::Down),
            scancode: None,
            keymod: sdl3::keyboard::Mod::LCTRLMOD,
            repeat: false,
        });
        assert_eq!(app.selected_timeline_fx_row(MidiFxChainKind::Output), 2);
        assert_eq!(
            app.page_state.selected_timeline_context,
            TimelineContext::OutputFx
        );
        key(&mut app, K::Return, false);
        key(&mut app, K::Delete, false);
        assert!(app.project.tracks[0].midi_fx.output_fx[2].is_some());
        let enabled = app.project.tracks[0].midi_fx.output_fx[2]
            .as_ref()
            .unwrap()
            .enabled;
        key(&mut app, K::M, true);
        assert_ne!(
            app.project.tracks[0].midi_fx.output_fx[2]
                .as_ref()
                .unwrap()
                .enabled,
            enabled
        );
        key(&mut app, K::Return, false);
        key(&mut app, K::Delete, false);
        assert!(app.project.tracks[0].midi_fx.output_fx[2].is_none());
        assert_eq!(app.selected_timeline_fx_row(MidiFxChainKind::Output), 2);
        key(&mut app, K::Left, true);
        assert_eq!(
            app.page_state.selected_timeline_context,
            TimelineContext::TrackTimeline
        );
        key(&mut app, K::Right, false);
        assert_eq!(app.project.active_track_index, 1);
    }
    #[test]
    fn split_fx_mapped_cc_changes_kind_bypasses_and_reorders_same_slot() {
        let mut app = App::new();
        app.mappings.clear();
        app.page_state.selected_timeline_context = TimelineContext::OutputFx;
        app.page_state.selected_timeline_fx_field = TimelineFxField::Kind;
        app.project.tracks[0].midi_fx.output_fx = vec![None; 4];
        let mut entry = crate::mapping::MappingEntry::default_new();
        entry.source_kind = crate::mapping::MappingSourceKind::Midi;
        entry.source_label = "CC20".to_string();
        entry.scope_label = "Active Track".to_string();
        entry.enabled = true;
        let event = crate::midi_io::MidiInputEvent {
            port: crate::midi_io::MidiPortRef {
                name: "Test".to_string(),
            },
            channel: 0,
            message: crate::midi_io::MidiInputMessage::ControlChange {
                controller: 20,
                value: 127,
            },
        };
        for label in [
            "Add Timeline FX",
            "Toggle Timeline FX",
            "Move Timeline FX Down",
            "Delete Timeline FX",
        ] {
            entry.target_label = label.to_string();
            app.mappings = vec![entry.clone()];
            let actions = app.resolve_midi_mapping_actions(&event);
            assert_eq!(actions.len(), 1, "{label}");
            for action in actions {
                app.apply_action_with_source(action, ActionSource::Midi);
            }
            match label {
                "Add Timeline FX" => assert!(app.project.tracks[0].midi_fx.output_fx[0].is_some()),
                "Toggle Timeline FX" => assert!(
                    !app.project.tracks[0].midi_fx.output_fx[0]
                        .as_ref()
                        .unwrap()
                        .enabled
                ),
                "Move Timeline FX Down" => {
                    assert!(app.project.tracks[0].midi_fx.output_fx[1].is_some())
                }
                _ => assert!(app.project.tracks[0].midi_fx.output_fx[1].is_none()),
            }
        }
    }
}
#[cfg(test)]
mod split_fx_hover_tests {
    use super::*;
    #[test]
    fn split_fx_hover_names_parameter_on_inactive_track_without_changing_selection() {
        let mut app = App::new();
        app.project.tracks[1].midi_fx.output_fx[0] = Some(MidiFxSlot {
            enabled: true,
            effect: MidiFx::default_for_kind(crate::midi_fx::MidiFxKind::Arp),
        });
        let metrics = app.ui_metrics();
        let surface = crate::ui::surface_rect(app.viewport_size.0, app.viewport_size.1, metrics);
        let inset =
            crate::ui::inset_rect(surface, metrics.frame_inset_x_px, metrics.frame_inset_y_px)
                .unwrap();
        let (_, content, _) = app.page_frame_layout(inset).unwrap();
        let (_, _, timeline) = app.timeline_page_layout(content).unwrap();
        let track_layout = app
            .visible_timeline_track_layouts(timeline)
            .into_iter()
            .find(|l| l.track_index == 1)
            .unwrap();
        let rows = app.timeline_fx_row_layouts(
            track_layout.output_fx_rect,
            &[Some(0), Some(1), Some(2), Some(3)],
            &app.project.tracks[1].midi_fx.output_fx,
            Some(0),
        );
        let cell = rows[0].parameters[2];
        app.handle_pointer_event(&sdl3::event::Event::MouseMotion {
            timestamp: 0,
            window_id: 0,
            which: 0,
            mousestate: sdl3::mouse::MouseState::from_sdl_state(0),
            x: (cell.x + 4) as f32,
            y: (cell.y + 4) as f32,
            xrel: 0.0,
            yrel: 0.0,
        });
        let target = app.status_state.hovered_target.unwrap();
        let summary = app.summarize_discoverability_target(target);
        assert!(summary.title.contains("Gate: 100%"), "{}", summary.title);
        assert!(summary.title.contains("Track 2"));
        assert_eq!(app.project.active_track_index, 0);
    }
}
#[cfg(test)]
mod fx_slot_shortcut_tests {
    use super::*;
    #[test]
    fn shift_digits_select_slots_preserving_focus_track_and_effects() {
        use sdl3::keyboard::{Keycode, Mod};
        for context in [TimelineContext::InputFx, TimelineContext::OutputFx] {
            for field in [TimelineFxField::Kind, TimelineFxField::ParamPrimary] {
                let mut app = App::new();
                app.mappings.clear();
                app.page_state.selected_timeline_context = context;
                app.page_state.selected_timeline_fx_field = field;
                let input = app.project.tracks[0].midi_fx.input_fx.clone();
                let output = app.project.tracks[0].midi_fx.output_fx.clone();
                for (slot, key) in [Keycode::_1, Keycode::_2, Keycode::_3, Keycode::_4]
                    .into_iter()
                    .enumerate()
                {
                    app.handle_keyboard_event(&sdl3::event::Event::KeyDown {
                        timestamp: 0,
                        window_id: 0,
                        which: 0,
                        raw: 0,
                        keycode: Some(key),
                        scancode: None,
                        keymod: Mod::LSHIFTMOD,
                        repeat: false,
                    });
                    assert_eq!(
                        app.project.active_track_index, 0,
                        "shift digit must not select track"
                    );
                    assert_eq!(
                        app.selected_timeline_fx_row(context.chain_kind().unwrap()),
                        slot
                    );
                    assert_eq!(app.page_state.selected_timeline_fx_field, field);
                    assert_eq!(app.page_state.selected_timeline_context, context);
                    assert_eq!(app.project.tracks[0].midi_fx.input_fx, input);
                    assert_eq!(app.project.tracks[0].midi_fx.output_fx, output);
                }
            }
        }
    }
}
#[cfg(test)]
mod fx_context_shortcut_tests {
    use super::*;
    fn key(app: &mut App, key: sdl3::keyboard::Keycode, mods: sdl3::keyboard::Mod) {
        app.handle_keyboard_event(&sdl3::event::Event::KeyDown {
            timestamp: 0,
            window_id: 0,
            which: 0,
            raw: 0,
            keycode: Some(key),
            scancode: None,
            keymod: mods,
            repeat: false,
        });
    }
    #[test]
    fn shift_up_down_exits_both_fx_focus_areas_and_ctrl_reorders() {
        use sdl3::keyboard::{Keycode as K, Mod};
        for field in [TimelineFxField::Kind, TimelineFxField::ParamPrimary] {
            let mut app = App::new();
            app.mappings.clear();
            app.page_state.selected_timeline_context = TimelineContext::OutputFx;
            app.page_state.selected_timeline_fx_field = field;
            key(&mut app, K::Up, Mod::LSHIFTMOD);
            assert_eq!(
                app.page_state.selected_timeline_context,
                TimelineContext::TrackTimeline
            );
            key(&mut app, K::Down, Mod::LSHIFTMOD);
            assert_eq!(
                app.page_state.selected_timeline_context,
                TimelineContext::OutputFx
            );
            app.page_state.selected_timeline_context = TimelineContext::InputFx;
            key(&mut app, K::Down, Mod::LSHIFTMOD);
            assert_eq!(
                app.page_state.selected_timeline_context,
                TimelineContext::TrackTimeline
            );
        }
        let mut app = App::new();
        app.mappings.clear();
        app.page_state.selected_timeline_context = TimelineContext::OutputFx;
        app.page_state.selected_timeline_fx_field = TimelineFxField::Kind;
        app.project.tracks[0].midi_fx.output_fx =
            vec![Some(MidiFxSlot::default()), None, None, None];
        key(&mut app, K::Down, Mod::LCTRLMOD);
        assert_eq!(app.selected_timeline_fx_row(MidiFxChainKind::Output), 1);
        assert!(app.project.tracks[0].midi_fx.output_fx[0].is_none());
        assert!(app.project.tracks[0].midi_fx.output_fx[1].is_some());
        assert_eq!(
            app.page_state.selected_timeline_context,
            TimelineContext::OutputFx
        );
    }
}
#[cfg(test)]
mod fx_pointer_edit_tests {
    use super::*;
    fn geometry(app: &App, context: TimelineContext, slot: usize) -> (Rect, TimelineFxRowLayout) {
        let content = Rect::new(40, 40, 1200, 620);
        let (_, _, timeline) = app.timeline_page_layout(content).unwrap();
        let layout = app
            .visible_timeline_track_layouts(timeline)
            .into_iter()
            .find(|r| r.track_index == 0)
            .unwrap();
        let band = layout.fx_rect(context);
        let rows = app.timeline_fx_row_layouts(
            band,
            &[Some(0), Some(1), Some(2), Some(3)],
            app.fx_chain(&app.project.tracks[0], context.chain_kind().unwrap()),
            Some(slot),
        );
        (content, rows[slot])
    }
    #[test]
    fn repeated_effect_click_cycles_after_selection_for_mouse_and_touch() {
        for source in [ActionSource::Pointer, ActionSource::Touch] {
            let mut app = App::new();
            app.project.tracks[0].midi_fx.output_fx = vec![Some(MidiFxSlot::default()); 4];
            app.page_state.selected_timeline_context = TimelineContext::TrackTimeline;
            let (content, layout) = geometry(&app, TimelineContext::OutputFx, 2);
            let original = app.project.tracks[0].midi_fx.output_fx[2].clone();
            app.handle_timeline_pointer(content, layout.row.x + 5, layout.row.y + 5, source);
            assert_eq!(app.project.tracks[0].midi_fx.output_fx[2], original);
            app.handle_timeline_pointer(content, layout.row.x + 5, layout.row.y + 5, source);
            assert_ne!(
                app.project.tracks[0].midi_fx.output_fx[2]
                    .as_ref()
                    .unwrap()
                    .effect
                    .kind(),
                original.as_ref().unwrap().effect.kind()
            );
            assert_eq!(app.selected_timeline_fx_row(MidiFxChainKind::Output), 2);
            assert_eq!(app.project.tracks[0].midi_fx.output_fx[0], original);
        }
    }
    #[test]
    fn parameter_halves_edit_immediately_in_both_directions_and_undo() {
        for context in [TimelineContext::InputFx, TimelineContext::OutputFx] {
            for source in [ActionSource::Pointer, ActionSource::Touch] {
                let mut app = App::new();
                let chain = context.chain_kind().unwrap();
                let effect = Some(MidiFxSlot {
                    enabled: true,
                    effect: MidiFx::default_for_kind(crate::midi_fx::MidiFxKind::Arp),
                });
                match chain {
                    MidiFxChainKind::Input => app.project.tracks[0].midi_fx.input_fx[0] = effect,
                    MidiFxChainKind::Output => app.project.tracks[0].midi_fx.output_fx[0] = effect,
                };
                app.page_state.selected_timeline_context = TimelineContext::TrackTimeline;
                let (content, layout) = geometry(&app, context, 0);
                let cell = layout.parameters[2];
                app.handle_timeline_pointer(content, cell.x + 1, cell.y + 4, source);
                assert_eq!(
                    app.selected_timeline_fx_slot(&app.project.tracks[0], chain)
                        .unwrap()
                        .effect
                        .inline_parameters()[2]
                        .value,
                    "90%"
                );
                assert_eq!(app.selected_timeline_fx_param_window(chain), 2);
                assert_eq!(
                    app.page_state.selected_timeline_fx_field,
                    TimelineFxField::ParamPrimary
                );
                app.handle_timeline_pointer(
                    content,
                    cell.x + cell.width() as i32 / 2,
                    cell.y + 4,
                    source,
                );
                assert_eq!(
                    app.selected_timeline_fx_slot(&app.project.tracks[0], chain)
                        .unwrap()
                        .effect
                        .inline_parameters()[2]
                        .value,
                    "100%"
                );
                app.apply_action(AppAction::UndoTimeline);
                assert_eq!(
                    app.selected_timeline_fx_slot(&app.project.tracks[0], chain)
                        .unwrap()
                        .effect
                        .inline_parameters()[2]
                        .value,
                    "90%"
                );
            }
        }
    }
    #[test]
    fn repeated_empty_slot_click_adds_in_that_slot() {
        let mut app = App::new();
        app.project.tracks[0].midi_fx.output_fx = vec![None; 4];
        app.page_state.selected_timeline_context = TimelineContext::TrackTimeline;
        let (content, layout) = geometry(&app, TimelineContext::OutputFx, 3);
        app.handle_timeline_pointer(
            content,
            layout.row.x + 4,
            layout.row.y + 4,
            ActionSource::Pointer,
        );
        assert!(
            app.project.tracks[0]
                .midi_fx
                .output_fx
                .iter()
                .all(Option::is_none)
        );
        app.handle_timeline_pointer(
            content,
            layout.row.x + 4,
            layout.row.y + 4,
            ActionSource::Pointer,
        );
        assert!(app.project.tracks[0].midi_fx.output_fx[3].is_some());
        assert!(app.project.tracks[0].midi_fx.output_fx[0].is_none());
    }
}
#[cfg(test)]
mod fx_menu_regression_tests {
    use super::*;
    #[test]
    fn fx_racks_have_equal_four_pixel_timeline_gaps() {
        let mut app = App::new();
        for density in [
            UiDensityPreset::Default,
            UiDensityPreset::Compact,
            UiDensityPreset::Tiny,
            UiDensityPreset::Touch,
        ] {
            app.set_ui_density_preset(density);
            let layout = app.timeline_track_layout(
                0,
                Rect::new(20, 50, 90, 600),
                Rect::new(116, 50, 90, 600),
            );
            assert_eq!(layout.body_full_bounds.y - layout.input_fx_rect.bottom(), 4);
            assert_eq!(
                layout.output_fx_rect.y - layout.body_full_bounds.bottom(),
                4
            );
        }
    }
    #[test]
    fn right_clicking_selected_fx_does_not_cycle_before_menu_choice() {
        let mut app = App::new();
        app.page_state.selected_timeline_context = TimelineContext::OutputFx;
        app.page_state.selected_timeline_fx_field = TimelineFxField::Kind;
        let before = app.project.tracks[0].midi_fx.output_fx.clone();
        let metrics = app.ui_metrics();
        let surface = crate::ui::surface_rect(app.viewport_size.0, app.viewport_size.1, metrics);
        let inset =
            crate::ui::inset_rect(surface, metrics.frame_inset_x_px, metrics.frame_inset_y_px)
                .unwrap();
        let (_, content, _) = app.page_frame_layout(inset).unwrap();
        let (_, _, timeline) = app.timeline_page_layout(content).unwrap();
        let layout = app.visible_timeline_track_layouts(timeline)[0];
        let rows = app.timeline_fx_row_layouts(
            layout.output_fx_rect,
            &[Some(0), Some(1), Some(2), Some(3)],
            &before,
            Some(0),
        );
        app.handle_pointer_event(&sdl3::event::Event::MouseButtonDown {
            timestamp: 0,
            window_id: 0,
            which: 0,
            mouse_btn: sdl3::mouse::MouseButton::Right,
            clicks: 1,
            x: (rows[0].row.x + 3) as f32,
            y: (rows[0].row.y + 3) as f32,
        });
        assert_eq!(app.project.tracks[0].midi_fx.output_fx, before);
    }
}
