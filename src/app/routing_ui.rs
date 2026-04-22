use super::*;
use crate::theme::io_pages as io_theme;

impl App {
    pub(crate) fn draw_routing_page<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        content_bounds: Rect,
    ) -> Result<(), Box<dyn std::error::Error>> {
        canvas.set_draw_color(io_theme::PAGE_BG);
        canvas.fill_rect(content_bounds)?;
        canvas.set_draw_color(io_theme::PAGE_BORDER);
        canvas.draw_rect(content_bounds)?;
        crate::ui::draw_text_fitted(
            canvas,
            "Routing",
            Rect::new(content_bounds.x + 8, content_bounds.y + 8, 140, 14),
            2,
            io_theme::PAGE_TITLE,
        )?;
        crate::ui::draw_text_fitted(
            canvas,
            "Active Track Routing",
            Rect::new(content_bounds.x + 184, content_bounds.y + 12, 180, 8),
            1,
            io_theme::SUBTITLE,
        )?;

        let inner = crate::ui::inset_rect(content_bounds, 12, 32)?;
        let (header, body) = crate::ui::split_top_strip(inner, 48, 10)?;
        let active_track = self.project.active_track().expect("demo project has tracks");

        canvas.set_draw_color(Color::RGB(54, 70, 104));
        canvas.fill_rect(header)?;
        canvas.set_draw_color(Color::RGB(244, 232, 146));
        canvas.draw_rect(header)?;

        let meta_badges = [
            (
                Rect::new(
                    header.x + 8,
                    header.y + 8,
                    90,
                    header.height().saturating_sub(16),
                ),
                Color::RGB(220, 124, 100),
                format!("Active T{}", self.project.active_track_index + 1),
            ),
            (
                Rect::new(
                    header.x + 106,
                    header.y + 8,
                    92,
                    header.height().saturating_sub(16),
                ),
                if active_track.state.passthrough {
                    Color::RGB(72, 188, 180)
                } else {
                    Color::RGB(92, 100, 112)
                },
                format!("Thru {}", on_off(active_track.state.passthrough)),
            ),
        ];
        for (rect, color, label) in meta_badges {
            canvas.set_draw_color(color);
            canvas.fill_rect(rect)?;
            crate::ui::draw_text_fitted(
                canvas,
                &label,
                Rect::new(rect.x + 6, rect.y + 4, rect.width().saturating_sub(12), 8),
                1,
                Color::RGB(24, 28, 36),
            )?;
        }
        let state_badge = Rect::new(
            header.x + header.width() as i32 - 122,
            header.y + 8,
            112,
            header.height().saturating_sub(16),
        );
        canvas.set_draw_color(Color::RGB(70, 86, 118));
        canvas.fill_rect(state_badge)?;
        crate::ui::draw_text_fitted(
            canvas,
            "Tap value",
            Rect::new(
                state_badge.x + 6,
                state_badge.y + 4,
                state_badge.width().saturating_sub(12),
                8,
            ),
            1,
            Color::RGB(244, 244, 236),
        )?;
        crate::ui::draw_text_fitted(
            canvas,
            &active_track.name,
            Rect::new(
                header.x + 208,
                header.y + 8,
                (state_badge.x - header.x - 220).max(0) as u32,
                8,
            ),
            1,
            Color::RGB(244, 244, 236),
        )?;
        crate::ui::draw_text_fitted(
            canvas,
            "Input/output routing plus MIDI FX for the active track",
            Rect::new(
                header.x + 208,
                header.y + 24,
                (state_badge.x - header.x - 220).max(0) as u32,
                8,
            ),
            1,
            Color::RGB(208, 216, 228),
        )?;

        let (signal_panel, input_fx_panel, rec_panel, output_fx_panel) =
            self.routing_panel_rects(body);
        self.draw_routing_group_panel(
            canvas,
            signal_panel,
            "Signal",
            "Ports, channels, and thru",
            Color::RGB(94, 186, 152),
        )?;
        self.draw_routing_group_panel(
            canvas,
            input_fx_panel,
            "Input FX",
            "Clone and input-shaping chain",
            Color::RGB(104, 152, 214),
        )?;
        self.draw_routing_group_panel(
            canvas,
            rec_panel,
            "Rec/Mon",
            "Input recording and monitor mode",
            Color::RGB(112, 188, 152),
        )?;
        self.draw_routing_group_panel(
            canvas,
            output_fx_panel,
            "Output FX",
            "Post-track playback shaping",
            Color::RGB(198, 138, 186),
        )?;

        for (field, row) in self.routing_field_rects(body) {
            let selected = field == self.page_state.selected_routing_field;
            let is_toggle_field = matches!(
                field,
                RoutingField::Passthrough
                    | RoutingField::RecordInputFx
                    | RoutingField::MonitorInputFx
                    | RoutingField::InputFxEnabled
                    | RoutingField::OutputFxEnabled
            );

            canvas.set_draw_color(if selected {
                Color::RGB(52, 64, 92)
            } else {
                Color::RGB(34, 40, 58)
            });
            canvas.fill_rect(row)?;
            canvas.set_draw_color(if selected {
                Color::RGB(244, 232, 146)
            } else {
                Color::RGB(78, 88, 110)
            });
            canvas.draw_rect(row)?;

            let value_color = match field {
                RoutingField::InputDevice => Color::RGB(94, 186, 152),
                RoutingField::InputChannel => Color::RGB(106, 152, 218),
                RoutingField::OutputDevice => Color::RGB(218, 142, 98),
                RoutingField::OutputChannel => Color::RGB(208, 122, 160),
                RoutingField::RecordInputFx => Color::RGB(112, 188, 152),
                RoutingField::MonitorInputFx => Color::RGB(102, 182, 196),
                RoutingField::InputFxSlot
                | RoutingField::InputFxKind
                | RoutingField::InputFxEnabled
                | RoutingField::InputFxParam1
                | RoutingField::InputFxParam2
                | RoutingField::InputFxMore => Color::RGB(120, 152, 214),
                RoutingField::OutputFxSlot
                | RoutingField::OutputFxKind
                | RoutingField::OutputFxEnabled
                | RoutingField::OutputFxParam1
                | RoutingField::OutputFxParam2
                | RoutingField::OutputFxMore => Color::RGB(200, 138, 186),
                RoutingField::Passthrough => {
                    if active_track.state.passthrough {
                        Color::RGB(92, 220, 216)
                    } else {
                        Color::RGB(112, 118, 126)
                    }
                }
            };
            let (input_p1, input_p2, _, _) =
                self.selected_fx_visible_params(active_track, MidiFxChainKind::Input);
            let (output_p1, output_p2, _, _) =
                self.selected_fx_visible_params(active_track, MidiFxChainKind::Output);
            let field_label = match field {
                RoutingField::InputFxParam1 => visible_param_label(input_p1.as_ref(), "P1"),
                RoutingField::InputFxParam2 => visible_param_label(input_p2.as_ref(), "P2"),
                RoutingField::OutputFxParam1 => visible_param_label(output_p1.as_ref(), "P1"),
                RoutingField::OutputFxParam2 => visible_param_label(output_p2.as_ref(), "P2"),
                _ => routing_field_short_label(field).to_string(),
            };
            let control_height = row.height().saturating_sub(20).max(10);
            let control_y = row.y + row.height() as i32 - control_height as i32 - 6;
            let label_text_rect =
                Rect::new(row.x + 8, row.y + 4, row.width().saturating_sub(16), 8);
            let value = Rect::new(
                row.x + 8,
                control_y,
                row.width().saturating_sub(64),
                control_height,
            );
            let affordance = Rect::new(
                row.x + row.width() as i32 - 48,
                control_y,
                40,
                control_height,
            );
            let left_adjust = Rect::new(
                value.x + 3,
                value.y + 2,
                14,
                value.height().saturating_sub(4),
            );
            let right_adjust = Rect::new(
                value.x + value.width() as i32 - 17,
                value.y + 2,
                14,
                value.height().saturating_sub(4),
            );
            canvas.set_draw_color(value_color);
            canvas.fill_rect(value)?;
            if !is_toggle_field {
                canvas.set_draw_color(Color::RGB(34, 42, 56));
                canvas.fill_rect(left_adjust)?;
                canvas.fill_rect(right_adjust)?;
            }
            canvas.set_draw_color(if selected {
                Color::RGB(244, 232, 146)
            } else {
                Color::RGB(96, 104, 122)
            });
            canvas.fill_rect(affordance)?;
            canvas.set_draw_color(if selected {
                Color::RGB(252, 244, 178)
            } else {
                Color::RGB(124, 132, 146)
            });
            canvas.draw_rect(affordance)?;
            crate::ui::draw_text_fitted(
                canvas,
                &field_label,
                centered_text_rect(label_text_rect),
                1,
                Color::RGB(244, 244, 236),
            )?;
            if is_toggle_field {
                let bool_chip = Rect::new(
                    value.x + 6,
                    value.y + 1,
                    value.width().saturating_sub(12).min(64),
                    value.height().saturating_sub(2),
                );
                let toggled_on = matches!(
                    self.routing_field_value(active_track, field).as_str(),
                    "on" | "Post FX"
                );
                canvas.set_draw_color(if toggled_on {
                    Color::RGB(48, 170, 108)
                } else {
                    Color::RGB(82, 66, 74)
                });
                canvas.fill_rect(bool_chip)?;
                canvas.set_draw_color(if toggled_on {
                    Color::RGB(192, 250, 206)
                } else {
                    Color::RGB(172, 128, 140)
                });
                canvas.draw_rect(bool_chip)?;
                crate::ui::draw_text_fitted(
                    canvas,
                    &self.routing_field_value(active_track, field),
                    Rect::new(
                        bool_chip.x + 6,
                        bool_chip.y + ((bool_chip.height() as i32 - 8) / 2).max(0),
                        bool_chip.width().saturating_sub(12),
                        8,
                    ),
                    1,
                    Color::RGB(244, 244, 236),
                )?;
            } else {
                crate::ui::draw_text_fitted(
                    canvas,
                    "-",
                    Rect::new(
                        left_adjust.x + 3,
                        left_adjust.y + ((left_adjust.height() as i32 - 8) / 2).max(0),
                        left_adjust.width().saturating_sub(6),
                        8,
                    ),
                    1,
                    Color::RGB(222, 228, 236),
                )?;
                crate::ui::draw_text_fitted(
                    canvas,
                    "+",
                    Rect::new(
                        right_adjust.x + 3,
                        right_adjust.y + ((right_adjust.height() as i32 - 8) / 2).max(0),
                        right_adjust.width().saturating_sub(6),
                        8,
                    ),
                    1,
                    Color::RGB(222, 228, 236),
                )?;
                crate::ui::draw_text_fitted(
                    canvas,
                    &self.routing_field_value(active_track, field),
                    Rect::new(
                        value.x + 24,
                        value.y + ((value.height() as i32 - 8) / 2).max(0),
                        value.width().saturating_sub(48),
                        8,
                    ),
                    1,
                    contrasting_text_color(value_color),
                )?;
            }
            crate::ui::draw_text_fitted(
                canvas,
                if is_toggle_field {
                    "TGL"
                } else if selected {
                    "ADJ"
                } else {
                    "SET"
                },
                Rect::new(
                    affordance.x + 4,
                    affordance.y + ((affordance.height() as i32 - 8) / 2).max(0),
                    affordance.width().saturating_sub(12),
                    8,
                ),
                1,
                Color::RGB(24, 28, 36),
            )?;
        }

        if self.overlay_state.active == Some(AppOverlay::Discoverability) {
            self.draw_routing_discoverability_overlay(canvas, content_bounds)?;
        }

        Ok(())
    }

    pub(super) fn routing_panel_rects(&self, body: Rect) -> (Rect, Rect, Rect, Rect) {
        let gap = 12_i32;
        let signal_width = ((body.width() as i32 * 46) / 100).max(180) as u32;
        let right_width = body
            .width()
            .saturating_sub(signal_width)
            .saturating_sub(gap as u32);
        let signal_panel = Rect::new(body.x, body.y, signal_width, body.height());
        let right = Rect::new(
            body.x + signal_width as i32 + gap,
            body.y,
            right_width,
            body.height(),
        );
        let panel_gap = 10_i32;
        let rec_height = 72_u32.min(right.height());
        let remaining = right
            .height()
            .saturating_sub(rec_height)
            .saturating_sub((panel_gap * 2) as u32);
        let input_height = (remaining / 2).max(84);
        let output_height = right
            .height()
            .saturating_sub(rec_height)
            .saturating_sub(input_height)
            .saturating_sub((panel_gap * 2) as u32);
        let rec_panel = Rect::new(right.x, right.y, right.width(), rec_height);
        let input_fx_panel = Rect::new(
            right.x,
            right.y + rec_height as i32 + panel_gap,
            right.width(),
            input_height,
        );
        let output_fx_panel = Rect::new(
            right.x,
            input_fx_panel.y + input_fx_panel.height() as i32 + panel_gap,
            right.width(),
            output_height,
        );
        (signal_panel, input_fx_panel, rec_panel, output_fx_panel)
    }

    pub(super) fn routing_field_rects(&self, body: Rect) -> Vec<(RoutingField, Rect)> {
        const SIGNAL_FIELDS: [RoutingField; 5] = [
            RoutingField::InputDevice,
            RoutingField::InputChannel,
            RoutingField::OutputDevice,
            RoutingField::OutputChannel,
            RoutingField::Passthrough,
        ];
        const REC_FIELDS: [RoutingField; 2] =
            [RoutingField::RecordInputFx, RoutingField::MonitorInputFx];
        const INPUT_FX_FIELDS: [RoutingField; 6] = [
            RoutingField::InputFxSlot,
            RoutingField::InputFxKind,
            RoutingField::InputFxEnabled,
            RoutingField::InputFxParam1,
            RoutingField::InputFxParam2,
            RoutingField::InputFxMore,
        ];
        const OUTPUT_FX_FIELDS: [RoutingField; 6] = [
            RoutingField::OutputFxSlot,
            RoutingField::OutputFxKind,
            RoutingField::OutputFxEnabled,
            RoutingField::OutputFxParam1,
            RoutingField::OutputFxParam2,
            RoutingField::OutputFxMore,
        ];

        let (signal_panel, input_fx_panel, rec_panel, output_fx_panel) =
            self.routing_panel_rects(body);
        let mut rects = Vec::with_capacity(RoutingField::ALL.len());
        rects.extend(self.routing_group_rows(signal_panel, &SIGNAL_FIELDS));
        rects.extend(self.routing_group_rows(input_fx_panel, &INPUT_FX_FIELDS));
        rects.extend(self.routing_group_rows(rec_panel, &REC_FIELDS));
        rects.extend(self.routing_group_rows(output_fx_panel, &OUTPUT_FX_FIELDS));
        rects
    }

    fn routing_group_rows(&self, panel: Rect, fields: &[RoutingField]) -> Vec<(RoutingField, Rect)> {
        let inner = crate::ui::inset_rect(panel, 10, 10).unwrap_or(panel);
        let rows_bounds = Rect::new(
            inner.x,
            inner.y + 18,
            inner.width(),
            inner.height().saturating_sub(18),
        );
        match fields.len() {
            2 => {
                let columns = crate::ui::equal_columns(rows_bounds, 2, 8);
                vec![(fields[0], columns[0]), (fields[1], columns[1])]
            }
            4 => {
                let row_rects = crate::ui::stacked_rows(rows_bounds, 2, 8);
                let mut rects = Vec::with_capacity(4);
                for row_index in 0..2 {
                    let columns = crate::ui::equal_columns(row_rects[row_index], 2, 8);
                    rects.push((fields[row_index * 2], columns[0]));
                    rects.push((fields[row_index * 2 + 1], columns[1]));
                }
                rects
            }
            6 => {
                let row_rects = crate::ui::stacked_rows(rows_bounds, 3, 8);
                let mut rects = Vec::with_capacity(6);
                for row_index in 0..3 {
                    let columns = crate::ui::equal_columns(row_rects[row_index], 2, 8);
                    rects.push((fields[row_index * 2], columns[0]));
                    rects.push((fields[row_index * 2 + 1], columns[1]));
                }
                rects
            }
            _ => {
                let rows = crate::ui::stacked_rows(rows_bounds, fields.len().max(1), 8);
                fields.iter().copied().zip(rows).collect()
            }
        }
    }

    fn draw_routing_group_panel<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        panel: Rect,
        title: &str,
        subtitle: &str,
        accent: Color,
    ) -> Result<(), Box<dyn std::error::Error>> {
        canvas.set_draw_color(Color::RGB(24, 30, 46));
        canvas.fill_rect(panel)?;
        canvas.set_draw_color(accent);
        canvas.draw_rect(panel)?;
        let header = Rect::new(
            panel.x + 8,
            panel.y + 6,
            panel.width().saturating_sub(16),
            12,
        );
        crate::ui::draw_text_fitted(canvas, title, Rect::new(header.x, header.y, 74, 8), 1, accent)?;
        crate::ui::draw_text_fitted(
            canvas,
            subtitle,
            Rect::new(
                header.x + 78,
                header.y,
                header.width().saturating_sub(78),
                8,
            ),
            1,
            Color::RGB(176, 186, 198),
        )?;
        Ok(())
    }

    fn routing_field_value(&self, track: &Track, field: RoutingField) -> String {
        match field {
            RoutingField::InputDevice => track
                .routing
                .input_port
                .as_ref()
                .map(|port| {
                    if self.input_port_is_available(&port.name) {
                        port.name.clone()
                    } else {
                        format!("{} (offline)", port.name)
                    }
                })
                .unwrap_or_else(|| "none".to_string()),
            RoutingField::InputChannel => input_channel_label(track.routing.input_channel),
            RoutingField::OutputDevice => track
                .routing
                .output_port
                .as_ref()
                .map(|port| {
                    if self.output_port_is_available(&port.name) {
                        port.name.clone()
                    } else {
                        format!("{} (offline)", port.name)
                    }
                })
                .unwrap_or_else(|| "none".to_string()),
            RoutingField::OutputChannel => output_channel_label(track.routing.output_channel),
            RoutingField::Passthrough => on_off(track.state.passthrough).to_string(),
            RoutingField::RecordInputFx => track.midi_fx.record_input_fx_mode.label().to_string(),
            RoutingField::MonitorInputFx => on_off(track.midi_fx.monitor_input_fx).to_string(),
            RoutingField::InputFxSlot => {
                format!("Slot {}", self.selected_fx_slot_index(MidiFxChainKind::Input) + 1)
            }
            RoutingField::InputFxKind => self
                .selected_fx_slot(track, MidiFxChainKind::Input)
                .map(|slot| slot.effect.kind().label().to_string())
                .unwrap_or_else(|| "None".to_string()),
            RoutingField::InputFxEnabled => self
                .selected_fx_slot(track, MidiFxChainKind::Input)
                .map(|slot| on_off(slot.enabled).to_string())
                .unwrap_or_else(|| "None".to_string()),
            RoutingField::InputFxParam1 => self
                .selected_fx_visible_params(track, MidiFxChainKind::Input)
                .0
                .map(|param| param.value)
                .unwrap_or_else(|| "--".to_string()),
            RoutingField::InputFxParam2 => self
                .selected_fx_visible_params(track, MidiFxChainKind::Input)
                .1
                .map(|param| param.value)
                .unwrap_or_else(|| "--".to_string()),
            RoutingField::InputFxMore => self.selected_fx_overflow_label(track, MidiFxChainKind::Input),
            RoutingField::OutputFxSlot => {
                format!("Slot {}", self.selected_fx_slot_index(MidiFxChainKind::Output) + 1)
            }
            RoutingField::OutputFxKind => self
                .selected_fx_slot(track, MidiFxChainKind::Output)
                .map(|slot| slot.effect.kind().label().to_string())
                .unwrap_or_else(|| "None".to_string()),
            RoutingField::OutputFxEnabled => self
                .selected_fx_slot(track, MidiFxChainKind::Output)
                .map(|slot| on_off(slot.enabled).to_string())
                .unwrap_or_else(|| "None".to_string()),
            RoutingField::OutputFxParam1 => self
                .selected_fx_visible_params(track, MidiFxChainKind::Output)
                .0
                .map(|param| param.value)
                .unwrap_or_else(|| "--".to_string()),
            RoutingField::OutputFxParam2 => self
                .selected_fx_visible_params(track, MidiFxChainKind::Output)
                .1
                .map(|param| param.value)
                .unwrap_or_else(|| "--".to_string()),
            RoutingField::OutputFxMore => {
                self.selected_fx_overflow_label(track, MidiFxChainKind::Output)
            }
        }
    }

    pub(crate) fn handle_routing_pointer(
        &mut self,
        content_bounds: Rect,
        x: i32,
        y: i32,
        _source: crate::actions::ActionSource,
    ) -> Option<AppControl> {
        let inner = crate::ui::inset_rect(content_bounds, 12, 32).ok()?;
        let (header, body) = crate::ui::split_top_strip(inner, 48, 10).ok()?;

        let meta_active = Rect::new(
            header.x + 8,
            header.y + 8,
            90,
            header.height().saturating_sub(16),
        );
        let meta_thru = Rect::new(
            header.x + 106,
            header.y + 8,
            92,
            header.height().saturating_sub(16),
        );
        if rect_contains(meta_active, x, y) {
            self.project.select_next_track();
            return Some(AppControl::Continue);
        }
        if rect_contains(meta_thru, x, y) {
            self.page_state.selected_routing_field = RoutingField::Passthrough;
            self.activate_page_item();
            return Some(AppControl::Continue);
        }

        for (field, row) in self.routing_field_rects(body) {
            if !rect_contains(row, x, y) {
                continue;
            }
            self.page_state.selected_routing_field = field;
            if matches!(
                field,
                RoutingField::Passthrough
                    | RoutingField::RecordInputFx
                    | RoutingField::MonitorInputFx
                    | RoutingField::InputFxEnabled
                    | RoutingField::OutputFxEnabled
            ) {
                self.activate_page_item();
                return Some(AppControl::Continue);
            }
            let control_height = row.height().saturating_sub(20).max(10);
            let control_y = row.y + row.height() as i32 - control_height as i32 - 6;
            let value = Rect::new(
                row.x + 8,
                control_y,
                row.width().saturating_sub(64),
                control_height,
            );
            let affordance = Rect::new(
                row.x + row.width() as i32 - 48,
                control_y,
                40,
                control_height,
            );
            if rect_contains(value, x, y) {
                let delta = if x < value.x + value.width() as i32 / 2 { -1 } else { 1 };
                self.adjust_routing_field(delta);
            } else if rect_contains(affordance, x, y) {
                self.adjust_routing_field(1);
            }
            return Some(AppControl::Continue);
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn routing_page_adjusts_active_track_routing() {
        let mut app = App::new();
        app.apply_action(AppAction::ShowPage(AppPage::Routing));
        app.page_state.selected_routing_field = RoutingField::OutputChannel;

        let before = app.project.active_track().unwrap().routing.output_channel;
        app.apply_action(AppAction::AdjustPageItemForward);

        assert_ne!(app.project.active_track().unwrap().routing.output_channel, before);
    }

    #[test]
    fn routing_fx_panels_use_two_column_grid_for_six_fields() {
        let app = App::new();
        let body = Rect::new(0, 0, 900, 520);
        let (_, input_panel, _, output_panel) = app.routing_panel_rects(body);
        let rects = app.routing_field_rects(body);

        let input_slot = rects
            .iter()
            .find(|(field, _)| *field == RoutingField::InputFxSlot)
            .map(|(_, rect)| *rect)
            .unwrap();
        let input_kind = rects
            .iter()
            .find(|(field, _)| *field == RoutingField::InputFxKind)
            .map(|(_, rect)| *rect)
            .unwrap();
        let input_on = rects
            .iter()
            .find(|(field, _)| *field == RoutingField::InputFxEnabled)
            .map(|(_, rect)| *rect)
            .unwrap();
        let input_p1 = rects
            .iter()
            .find(|(field, _)| *field == RoutingField::InputFxParam1)
            .map(|(_, rect)| *rect)
            .unwrap();
        let input_p2 = rects
            .iter()
            .find(|(field, _)| *field == RoutingField::InputFxParam2)
            .map(|(_, rect)| *rect)
            .unwrap();
        let input_more = rects
            .iter()
            .find(|(field, _)| *field == RoutingField::InputFxMore)
            .map(|(_, rect)| *rect)
            .unwrap();

        assert_eq!(input_slot.y, input_kind.y);
        assert_eq!(input_on.y, input_p1.y);
        assert_eq!(input_p2.y, input_more.y);
        assert!(input_slot.x < input_kind.x);
        assert!(input_on.x < input_p1.x);
        assert!(input_p2.x < input_more.x);
        for rect in [input_slot, input_kind, input_on, input_p1, input_p2, input_more] {
            assert!(input_panel.contains_point((rect.x, rect.y)));
            assert!(input_panel.contains_point((rect.x + rect.width() as i32 - 1, rect.y + rect.height() as i32 - 1)));
        }

        let output_slot = rects
            .iter()
            .find(|(field, _)| *field == RoutingField::OutputFxSlot)
            .map(|(_, rect)| *rect)
            .unwrap();
        let output_kind = rects
            .iter()
            .find(|(field, _)| *field == RoutingField::OutputFxKind)
            .map(|(_, rect)| *rect)
            .unwrap();
        assert_eq!(output_slot.y, output_kind.y);
        assert!(output_slot.x < output_kind.x);
        assert!(output_panel.contains_point((output_kind.x, output_kind.y)));
    }

    #[test]
    fn routing_field_short_labels_match_compact_fx_grid() {
        assert_eq!(routing_field_short_label(RoutingField::InputFxSlot), "Slot");
        assert_eq!(routing_field_short_label(RoutingField::InputFxKind), "Kind");
        assert_eq!(routing_field_short_label(RoutingField::InputFxEnabled), "On");
        assert_eq!(routing_field_short_label(RoutingField::InputFxParam1), "P1");
        assert_eq!(routing_field_short_label(RoutingField::InputFxParam2), "P2");
        assert_eq!(routing_field_short_label(RoutingField::InputFxMore), "More");
    }
}
