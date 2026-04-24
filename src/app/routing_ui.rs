use super::*;
use crate::midi_fx::MidiFxInlineParam;

impl App {
    pub(crate) fn routing_discoverability_targets(
        &self,
        content_bounds: Rect,
    ) -> Vec<(Rect, DiscoverabilityTarget)> {
        let mut targets = Vec::new();
        let inner = crate::ui::inset_rect(content_bounds, 12, 32).expect("routing inner");
        let (header, body) = crate::ui::split_top_strip(inner, 48, 10).expect("routing layout");
        targets.push((
            Rect::new(
                header.x + 106,
                header.y + 8,
                92,
                header.height().saturating_sub(16),
            ),
            DiscoverabilityTarget {
                action: AppAction::ToggleCurrentTrackPassthrough,
                display_scope: Some("Active Track"),
                allowed_mapping_scopes: &["Active Track"],
                overlay_slot: None,
            },
        ));

        for (field, row) in self.routing_field_rects(body) {
            if field != RoutingField::Passthrough {
                continue;
            }
            let control_height = row.height().saturating_sub(20).max(10);
            let control_y = row.y + row.height() as i32 - control_height as i32 - 6;
            let value = Rect::new(
                row.x + 8,
                control_y,
                row.width().saturating_sub(64),
                control_height,
            );
            targets.push((
                value,
                DiscoverabilityTarget {
                    action: AppAction::ToggleCurrentTrackPassthrough,
                    display_scope: Some("Active Track"),
                    allowed_mapping_scopes: &["Active Track"],
                    overlay_slot: None,
                },
            ));
        }

        targets
    }

    pub(crate) fn draw_routing_page<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        content_bounds: Rect,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let theme = self.theme();
        canvas.set_draw_color(theme.io_pages.page_bg);
        canvas.fill_rect(content_bounds)?;
        canvas.set_draw_color(theme.io_pages.page_border);
        canvas.draw_rect(content_bounds)?;
        crate::ui::draw_text_fitted(
            canvas,
            "Routing",
            Rect::new(content_bounds.x + 8, content_bounds.y + 8, 140, 14),
            2,
            theme.io_pages.page_title,
        )?;
        crate::ui::draw_text_fitted(
            canvas,
            "Active Track Routing",
            Rect::new(content_bounds.x + 184, content_bounds.y + 12, 180, 8),
            1,
            theme.io_pages.subtitle,
        )?;

        let inner = crate::ui::inset_rect(content_bounds, 12, 32)?;
        let (header, body) = crate::ui::split_top_strip(inner, 48, 10)?;
        let active_track = self
            .project
            .active_track()
            .expect("demo project has tracks");

        canvas.set_draw_color(theme.io_pages.routing_header_fill);
        canvas.fill_rect(header)?;
        canvas.set_draw_color(theme.io_pages.routing_header_border);
        canvas.draw_rect(header)?;

        let meta_badges = [
            (
                Rect::new(
                    header.x + 8,
                    header.y + 8,
                    90,
                    header.height().saturating_sub(16),
                ),
                theme.io_pages.routing_meta_active_fill,
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
                    theme.io_pages.routing_meta_thru_on_fill
                } else {
                    theme.io_pages.routing_meta_thru_off_fill
                },
                format!("Thru {}", on_off(active_track.state.passthrough)),
            ),
        ];
        for (rect, color, label) in meta_badges {
            canvas.set_draw_color(color);
            canvas.fill_rect(rect)?;
            canvas.set_draw_color(theme.io_pages.routing_header_border);
            canvas.draw_rect(rect)?;
            crate::ui::draw_text_fitted(
                canvas,
                &label,
                Rect::new(rect.x + 6, rect.y + 4, rect.width().saturating_sub(12), 8),
                1,
                contrasting_text_color(color, theme),
            )?;
        }
        let state_badge = Rect::new(
            header.x + header.width() as i32 - 122,
            header.y + 8,
            112,
            header.height().saturating_sub(16),
        );
        canvas.set_draw_color(theme.io_pages.routing_state_badge_fill);
        canvas.fill_rect(state_badge)?;
        canvas.set_draw_color(theme.io_pages.routing_header_border);
        canvas.draw_rect(state_badge)?;
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
            contrasting_text_color(theme.io_pages.routing_state_badge_fill, theme),
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
            theme.io_pages.routing_track_name_text,
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
            theme.io_pages.routing_help_text,
        )?;

        let (signal_panel, input_fx_panel, rec_panel, output_fx_panel) =
            self.routing_panel_rects(body);
        self.draw_routing_group_panel(
            canvas,
            signal_panel,
            "Signal",
            "Ports, channels, and thru",
            theme.io_pages.routing_group_signal,
        )?;
        self.draw_routing_group_panel(
            canvas,
            input_fx_panel,
            "Input FX",
            "Clone and input-shaping chain",
            theme.io_pages.routing_group_input_fx,
        )?;
        self.draw_routing_group_panel(
            canvas,
            rec_panel,
            "Rec/Mon",
            "Input recording and monitor mode",
            theme.io_pages.routing_group_rec_mon,
        )?;
        self.draw_routing_group_panel(
            canvas,
            output_fx_panel,
            "Output FX",
            "Post-track playback shaping",
            theme.io_pages.routing_group_output_fx,
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
                theme.io_pages.routing_row_selected_fill
            } else {
                theme.io_pages.routing_row_idle_fill
            });
            canvas.fill_rect(row)?;
            canvas.set_draw_color(if selected {
                theme.io_pages.routing_row_selected_border
            } else {
                theme.io_pages.routing_row_idle_border
            });
            canvas.draw_rect(row)?;

            let value_color = match field {
                RoutingField::InputDevice => theme.io_pages.routing_value_input_device,
                RoutingField::InputChannel => theme.io_pages.routing_value_input_channel,
                RoutingField::OutputDevice => theme.io_pages.routing_value_output_device,
                RoutingField::OutputChannel => theme.io_pages.routing_value_output_channel,
                RoutingField::RecordInputFx => theme.io_pages.routing_value_record_fx,
                RoutingField::MonitorInputFx => theme.io_pages.routing_value_monitor_fx,
                RoutingField::InputFxSlot
                | RoutingField::InputFxKind
                | RoutingField::InputFxEnabled
                | RoutingField::InputFxParam1
                | RoutingField::InputFxParam2
                | RoutingField::InputFxMore => theme.io_pages.routing_value_input_fx,
                RoutingField::OutputFxSlot
                | RoutingField::OutputFxKind
                | RoutingField::OutputFxEnabled
                | RoutingField::OutputFxParam1
                | RoutingField::OutputFxParam2
                | RoutingField::OutputFxMore => theme.io_pages.routing_value_output_fx,
                RoutingField::Passthrough => {
                    if active_track.state.passthrough {
                        theme.io_pages.routing_value_passthrough_on
                    } else {
                        theme.io_pages.routing_value_passthrough_off
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
                canvas.set_draw_color(theme.io_pages.routing_adjust_fill);
                canvas.fill_rect(left_adjust)?;
                canvas.fill_rect(right_adjust)?;
            }
            canvas.set_draw_color(if selected {
                theme.io_pages.routing_affordance_selected_fill
            } else {
                theme.io_pages.routing_affordance_idle_fill
            });
            canvas.fill_rect(affordance)?;
            canvas.set_draw_color(if selected {
                theme.io_pages.routing_affordance_selected_border
            } else {
                theme.io_pages.routing_affordance_idle_border
            });
            canvas.draw_rect(affordance)?;
            crate::ui::draw_text_fitted(
                canvas,
                &field_label,
                centered_text_rect(label_text_rect),
                1,
                if selected {
                    contrasting_text_color(theme.io_pages.routing_row_selected_fill, theme)
                } else {
                    theme.io_pages.routing_field_label
                },
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
                let toggle_fill = if toggled_on {
                    theme.io_pages.routing_toggle_on_fill
                } else {
                    theme.io_pages.routing_toggle_off_fill
                };
                canvas.set_draw_color(toggle_fill);
                canvas.fill_rect(bool_chip)?;
                canvas.set_draw_color(if toggled_on {
                    theme.io_pages.routing_toggle_on_border
                } else {
                    theme.io_pages.routing_toggle_off_border
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
                    contrasting_text_color(toggle_fill, theme),
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
                    theme.io_pages.routing_adjust_text,
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
                    theme.io_pages.routing_adjust_text,
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
                    contrasting_text_color(value_color, theme),
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
                if selected {
                    contrasting_text_color(theme.io_pages.routing_affordance_selected_fill, theme)
                } else {
                    contrasting_text_color(theme.io_pages.routing_affordance_idle_fill, theme)
                },
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

    fn routing_group_rows(
        &self,
        panel: Rect,
        fields: &[RoutingField],
    ) -> Vec<(RoutingField, Rect)> {
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
        let theme = self.theme();
        canvas.set_draw_color(theme.io_pages.panel_bg);
        canvas.fill_rect(panel)?;
        canvas.set_draw_color(accent);
        canvas.draw_rect(panel)?;
        let header = Rect::new(
            panel.x + 8,
            panel.y + 6,
            panel.width().saturating_sub(16),
            12,
        );
        crate::ui::draw_text_fitted(
            canvas,
            title,
            Rect::new(header.x, header.y, 74, 8),
            1,
            accent,
        )?;
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
            theme.io_pages.subtitle,
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
                format!(
                    "Slot {}",
                    self.selected_fx_slot_index(MidiFxChainKind::Input) + 1
                )
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
            RoutingField::InputFxMore => {
                self.selected_fx_overflow_label(track, MidiFxChainKind::Input)
            }
            RoutingField::OutputFxSlot => {
                format!(
                    "Slot {}",
                    self.selected_fx_slot_index(MidiFxChainKind::Output) + 1
                )
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

    pub(super) fn selected_fx_slot_index(&self, chain_kind: MidiFxChainKind) -> usize {
        match chain_kind {
            MidiFxChainKind::Input => self
                .page_state
                .selected_input_fx_slot
                .min(MIDI_FX_SLOT_COUNT - 1),
            MidiFxChainKind::Output => self
                .page_state
                .selected_output_fx_slot
                .min(MIDI_FX_SLOT_COUNT - 1),
        }
    }

    pub(super) fn set_selected_fx_slot_index(&mut self, chain_kind: MidiFxChainKind, index: usize) {
        let clamped = index.min(MIDI_FX_SLOT_COUNT - 1);
        match chain_kind {
            MidiFxChainKind::Input => self.page_state.selected_input_fx_slot = clamped,
            MidiFxChainKind::Output => self.page_state.selected_output_fx_slot = clamped,
        }
    }

    pub(super) fn selected_fx_param_window(&self, chain_kind: MidiFxChainKind) -> usize {
        let Some(track) = self.project.active_track() else {
            return 0;
        };
        let slot_index = self.selected_fx_slot_index(chain_kind);
        let windows = match chain_kind {
            MidiFxChainKind::Input => &track.midi_fx.timeline_ui.input_param_windows,
            MidiFxChainKind::Output => &track.midi_fx.timeline_ui.output_param_windows,
        };
        windows.get(slot_index).copied().unwrap_or(0)
    }

    pub(super) fn set_selected_fx_param_window(
        &mut self,
        chain_kind: MidiFxChainKind,
        start: usize,
    ) {
        let slot_index = self.selected_fx_slot_index(chain_kind);
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

    pub(super) fn fx_chain<'a>(
        &self,
        track: &'a Track,
        chain_kind: MidiFxChainKind,
    ) -> &'a [Option<MidiFxSlot>] {
        match chain_kind {
            MidiFxChainKind::Input => &track.midi_fx.input_fx,
            MidiFxChainKind::Output => &track.midi_fx.output_fx,
        }
    }

    pub(super) fn selected_fx_slot<'a>(
        &self,
        track: &'a Track,
        chain_kind: MidiFxChainKind,
    ) -> Option<&'a MidiFxSlot> {
        self.fx_chain(track, chain_kind)
            .get(self.selected_fx_slot_index(chain_kind))
            .and_then(|slot| slot.as_ref())
    }

    pub(super) fn selected_fx_visible_params(
        &self,
        track: &Track,
        chain_kind: MidiFxChainKind,
    ) -> (
        Option<MidiFxInlineParam>,
        Option<MidiFxInlineParam>,
        usize,
        usize,
    ) {
        let Some(slot) = self.selected_fx_slot(track, chain_kind) else {
            return (None, None, 0, 0);
        };
        let params = slot.effect.inline_parameters();
        let window_start = self
            .selected_fx_param_window(chain_kind)
            .min(params.len().saturating_sub(1));
        (
            params.get(window_start).cloned(),
            params.get(window_start + 1).cloned(),
            params.len(),
            window_start,
        )
    }

    pub(super) fn selected_fx_overflow_label(
        &self,
        track: &Track,
        chain_kind: MidiFxChainKind,
    ) -> String {
        let (_, _, param_count, window_start) = self.selected_fx_visible_params(track, chain_kind);
        super::timeline::fx_ui::timeline_fx_overflow_label(param_count, window_start)
    }

    fn adjust_fx_slot_index(&mut self, chain_kind: MidiFxChainKind, delta: i32) {
        let current = self.selected_fx_slot_index(chain_kind) as i32;
        let next = (current + delta).rem_euclid(MIDI_FX_SLOT_COUNT as i32) as usize;
        self.set_selected_fx_slot_index(chain_kind, next);
    }

    fn adjust_fx_kind(&mut self, chain_kind: MidiFxChainKind, delta: i32) {
        let track_count = self.project.tracks.len();
        let slot_index = self.selected_fx_slot_index(chain_kind);
        let Some(track) = self.project.active_track_mut() else {
            return;
        };
        let chain = match chain_kind {
            MidiFxChainKind::Input => &mut track.midi_fx.input_fx,
            MidiFxChainKind::Output => &mut track.midi_fx.output_fx,
        };
        if slot_index >= chain.len() {
            return;
        }
        let current = chain[slot_index].as_ref();
        chain[slot_index] = cycle_fx_kind(current, delta);
        if let Some(slot) = chain[slot_index].as_mut() {
            if let MidiFx::TrackClone { source_track } = &mut slot.effect {
                let max_source = track_count.saturating_sub(1);
                *source_track = (*source_track).min(max_source);
            }
        }
        self.set_selected_fx_param_window(chain_kind, 0);
    }

    fn toggle_fx_enabled(&mut self, chain_kind: MidiFxChainKind) {
        let slot_index = self.selected_fx_slot_index(chain_kind);
        let Some(track) = self.project.active_track_mut() else {
            return;
        };
        let chain = match chain_kind {
            MidiFxChainKind::Input => &mut track.midi_fx.input_fx,
            MidiFxChainKind::Output => &mut track.midi_fx.output_fx,
        };
        if let Some(Some(slot)) = chain.get_mut(slot_index) {
            slot.enabled = !slot.enabled;
        }
    }

    fn adjust_fx_parameter(
        &mut self,
        chain_kind: MidiFxChainKind,
        visible_offset: usize,
        delta: i32,
    ) {
        let slot_index = self.selected_fx_slot_index(chain_kind);
        let track_count = self.project.tracks.len();
        let ppqn = self.project.transport.ppqn;
        let active_track_index = self.project.active_track_index;
        let parameter_index = self.selected_fx_param_window(chain_kind) + visible_offset;
        let source_muted: Vec<bool> = self
            .project
            .tracks
            .iter()
            .map(|track| track.state.muted)
            .collect();
        let Some(track) = self.project.active_track_mut() else {
            return;
        };
        let chain = match chain_kind {
            MidiFxChainKind::Input => &mut track.midi_fx.input_fx,
            MidiFxChainKind::Output => &mut track.midi_fx.output_fx,
        };
        let Some(Some(slot)) = chain.get_mut(slot_index) else {
            return;
        };
        slot.effect
            .adjust_inline_parameter(parameter_index, delta, track_count, ppqn);
        if let MidiFx::TrackClone { source_track } = &mut slot.effect {
            if *source_track == active_track_index && track_count > 1 {
                *source_track = if delta >= 0 { 1 } else { track_count - 1 };
            }
            if source_muted.get(*source_track).copied().unwrap_or(false) && track_count > 1 {
                let step = if delta >= 0 { 1 } else { track_count - 1 };
                *source_track = (*source_track + step) % track_count;
            }
        }
    }

    fn scroll_fx_parameter_window(&mut self, chain_kind: MidiFxChainKind, delta: i32) {
        let Some(track) = self.project.active_track() else {
            return;
        };
        let Some(slot) = self.selected_fx_slot(track, chain_kind) else {
            return;
        };
        let param_count = slot.effect.inline_parameters().len();
        let max_start = param_count.saturating_sub(2);
        let current = self.selected_fx_param_window(chain_kind);
        let next = (current as i32 + delta).clamp(0, max_start as i32) as usize;
        self.set_selected_fx_param_window(chain_kind, next);
    }

    pub(super) fn adjust_routing_field(&mut self, delta: i32) {
        let current_input = self.midi_devices.selected_input_port().cloned();
        let current_output = self.midi_devices.selected_output_port().cloned();
        match self.page_state.selected_routing_field {
            RoutingField::InputDevice => {
                if let Some(track) = self.project.active_track_mut() {
                    track.routing.input_port = cycle_optional_port(
                        track.routing.input_port.as_ref(),
                        &self.midi_devices.inputs,
                        delta,
                    );
                }
                self.sync_midi_inputs();
            }
            RoutingField::InputChannel => {
                if let Some(track) = self.project.active_track_mut() {
                    track.routing.input_channel =
                        cycle_input_channel(track.routing.input_channel, delta);
                }
            }
            RoutingField::OutputDevice => {
                if let Some(track) = self.project.active_track_mut() {
                    track.routing.output_port = cycle_optional_port(
                        track.routing.output_port.as_ref(),
                        &self.midi_devices.outputs,
                        delta,
                    );
                }
            }
            RoutingField::OutputChannel => {
                if let Some(track) = self.project.active_track_mut() {
                    track.routing.output_channel =
                        cycle_output_channel(track.routing.output_channel, delta);
                }
            }
            RoutingField::Passthrough => {
                if let Some(track) = self.project.active_track_mut() {
                    track.state.passthrough = !track.state.passthrough;
                    if track.routing.input_port.is_none() {
                        track.routing.input_port = current_input;
                    }
                    if track.routing.output_port.is_none() {
                        track.routing.output_port = current_output;
                    }
                }
                self.sync_midi_inputs();
            }
            RoutingField::RecordInputFx => {
                if let Some(track) = self.project.active_track_mut() {
                    track.midi_fx.record_input_fx_mode =
                        track.midi_fx.record_input_fx_mode.toggle();
                }
            }
            RoutingField::MonitorInputFx => {
                if let Some(track) = self.project.active_track_mut() {
                    track.midi_fx.monitor_input_fx = !track.midi_fx.monitor_input_fx;
                }
            }
            RoutingField::InputFxSlot => self.adjust_fx_slot_index(MidiFxChainKind::Input, delta),
            RoutingField::InputFxKind => self.adjust_fx_kind(MidiFxChainKind::Input, delta),
            RoutingField::InputFxEnabled => self.toggle_fx_enabled(MidiFxChainKind::Input),
            RoutingField::InputFxParam1 => {
                self.adjust_fx_parameter(MidiFxChainKind::Input, 0, delta)
            }
            RoutingField::InputFxParam2 => {
                self.adjust_fx_parameter(MidiFxChainKind::Input, 1, delta)
            }
            RoutingField::InputFxMore => {
                self.scroll_fx_parameter_window(MidiFxChainKind::Input, delta)
            }
            RoutingField::OutputFxSlot => self.adjust_fx_slot_index(MidiFxChainKind::Output, delta),
            RoutingField::OutputFxKind => self.adjust_fx_kind(MidiFxChainKind::Output, delta),
            RoutingField::OutputFxEnabled => self.toggle_fx_enabled(MidiFxChainKind::Output),
            RoutingField::OutputFxParam1 => {
                self.adjust_fx_parameter(MidiFxChainKind::Output, 0, delta)
            }
            RoutingField::OutputFxParam2 => {
                self.adjust_fx_parameter(MidiFxChainKind::Output, 1, delta)
            }
            RoutingField::OutputFxMore => {
                self.scroll_fx_parameter_window(MidiFxChainKind::Output, delta)
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
                let delta = if x < value.x + value.width() as i32 / 2 {
                    -1
                } else {
                    1
                };
                self.adjust_routing_field(delta);
            } else if rect_contains(affordance, x, y) {
                self.adjust_routing_field(1);
            }
            return Some(AppControl::Continue);
        }

        None
    }
}

pub(super) fn routing_field_short_label(field: RoutingField) -> &'static str {
    match field {
        RoutingField::InputDevice => "Input Device",
        RoutingField::InputChannel => "Input Chan",
        RoutingField::OutputDevice => "Output Device",
        RoutingField::OutputChannel => "Output Chan",
        RoutingField::Passthrough => "Thru",
        RoutingField::RecordInputFx => "Rec FX",
        RoutingField::MonitorInputFx => "Mon FX",
        RoutingField::InputFxSlot | RoutingField::OutputFxSlot => "Slot",
        RoutingField::InputFxKind | RoutingField::OutputFxKind => "Kind",
        RoutingField::InputFxEnabled | RoutingField::OutputFxEnabled => "On",
        RoutingField::InputFxParam1 | RoutingField::OutputFxParam1 => "P1",
        RoutingField::InputFxParam2 | RoutingField::OutputFxParam2 => "P2",
        RoutingField::InputFxMore | RoutingField::OutputFxMore => "More",
    }
}

fn visible_param_label(param: Option<&MidiFxInlineParam>, fallback: &'static str) -> String {
    param
        .map(|param| param.label.to_string())
        .unwrap_or_else(|| fallback.to_string())
}

pub(super) fn cycle_optional_port(
    current: Option<&MidiPortRef>,
    ports: &[MidiPortRef],
    delta: i32,
) -> Option<MidiPortRef> {
    if ports.is_empty() {
        return None;
    }

    let option_count = ports.len() as i32 + 1;
    let current_index = current
        .and_then(|port| ports.iter().position(|candidate| candidate == port))
        .map(|index| index as i32 + 1)
        .unwrap_or(0);
    let next_index = (current_index + delta).rem_euclid(option_count);
    if next_index == 0 {
        None
    } else {
        ports.get((next_index - 1) as usize).cloned()
    }
}

pub(super) fn cycle_input_channel(current: MidiChannelFilter, delta: i32) -> MidiChannelFilter {
    let current_index = match current {
        MidiChannelFilter::Omni => 0,
        MidiChannelFilter::Channel(channel) => i32::from(channel.clamp(1, 16)),
    };
    let next_index = (current_index + delta).rem_euclid(17);
    if next_index == 0 {
        MidiChannelFilter::Omni
    } else {
        MidiChannelFilter::Channel(next_index as u8)
    }
}

pub(super) fn cycle_output_channel(current: Option<u8>, delta: i32) -> Option<u8> {
    let current_index = current
        .map(|value| i32::from(value.clamp(1, 16)))
        .unwrap_or(0);
    let next_index = (current_index + delta).rem_euclid(17);
    if next_index == 0 {
        None
    } else {
        Some(next_index as u8)
    }
}

fn input_channel_label(channel: MidiChannelFilter) -> String {
    match channel {
        MidiChannelFilter::Omni => "Omni".to_string(),
        MidiChannelFilter::Channel(channel) => format!("Ch{channel}"),
    }
}

fn output_channel_label(channel: Option<u8>) -> String {
    match channel {
        Some(channel) => format!("Ch{}", channel.clamp(1, 16)),
        None => "None".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::routing::MidiChannelFilter;

    #[test]
    fn cycle_helpers_wrap_through_expected_ranges() {
        let app = App::new();
        assert_eq!(
            cycle_optional_port(None, &app.midi_devices.outputs, 1)
                .unwrap()
                .name,
            app.midi_devices.outputs[0].name
        );
        assert_eq!(
            cycle_input_channel(MidiChannelFilter::Omni, 1),
            MidiChannelFilter::Channel(1)
        );
        assert_eq!(cycle_output_channel(None, -1), Some(16));
    }

    #[test]
    fn routing_page_adjusts_active_track_routing() {
        let mut app = App::new();
        app.apply_action(AppAction::ShowPage(AppPage::Routing));
        app.page_state.selected_routing_field = RoutingField::OutputChannel;

        let before = app.project.active_track().unwrap().routing.output_channel;
        app.apply_action(AppAction::AdjustPageItemForward);

        assert_ne!(
            app.project.active_track().unwrap().routing.output_channel,
            before
        );
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
        for rect in [
            input_slot, input_kind, input_on, input_p1, input_p2, input_more,
        ] {
            assert!(input_panel.contains_point((rect.x, rect.y)));
            assert!(input_panel.contains_point((
                rect.x + rect.width() as i32 - 1,
                rect.y + rect.height() as i32 - 1
            )));
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
        assert_eq!(
            routing_field_short_label(RoutingField::InputFxEnabled),
            "On"
        );
        assert_eq!(routing_field_short_label(RoutingField::InputFxParam1), "P1");
        assert_eq!(routing_field_short_label(RoutingField::InputFxParam2), "P2");
        assert_eq!(routing_field_short_label(RoutingField::InputFxMore), "More");
    }
}
