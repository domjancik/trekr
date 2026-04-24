use super::*;

#[derive(Debug, Clone, Copy)]
struct MidiIoPageLayout {
    header_bounds: Rect,
    input_header: Rect,
    output_header: Rect,
    input_list: Rect,
    output_list: Rect,
}

impl App {
    fn midi_io_page_layout(&self, content_bounds: Rect) -> Result<MidiIoPageLayout, String> {
        let metrics = self.ui_metrics();
        let (header_bounds, lists_bounds) = crate::ui::split_top_strip(
            content_bounds,
            metrics.midi_header_height_px,
            metrics.midi_header_gap_px,
        )?;
        let columns = crate::ui::equal_columns(lists_bounds, 2, metrics.midi_column_gap_px);
        let input_bounds = columns[0];
        let output_bounds = columns[1];
        let input_header = Rect::new(
            input_bounds.x,
            input_bounds.y,
            input_bounds.width(),
            metrics.midi_panel_header_height_px,
        );
        let output_header = Rect::new(
            output_bounds.x,
            output_bounds.y,
            output_bounds.width(),
            metrics.midi_panel_header_height_px,
        );
        Ok(MidiIoPageLayout {
            header_bounds,
            input_header,
            output_header,
            input_list: Rect::new(
                input_bounds.x,
                input_header.y + input_header.height() as i32 + metrics.midi_list_top_gap_px,
                input_bounds.width(),
                input_bounds.height().saturating_sub(
                    input_header
                        .height()
                        .saturating_add(metrics.midi_list_bottom_gap_px.max(0) as u32),
                ),
            ),
            output_list: Rect::new(
                output_bounds.x,
                output_header.y + output_header.height() as i32 + metrics.midi_list_top_gap_px,
                output_bounds.width(),
                output_bounds.height().saturating_sub(
                    output_header
                        .height()
                        .saturating_add(metrics.midi_list_bottom_gap_px.max(0) as u32),
                ),
            ),
        })
    }

    pub(crate) fn draw_midi_io_page<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        content_bounds: Rect,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let theme = self.theme();
        canvas.set_draw_color(theme.io_pages.page_bg);
        canvas.fill_rect(content_bounds)?;
        canvas.set_draw_color(theme.io_pages.page_border);
        canvas.draw_rect(content_bounds)?;

        let layout = self.midi_io_page_layout(content_bounds)?;
        let header_bounds = layout.header_bounds;
        crate::ui::draw_text_fitted(
            canvas,
            "MIDI I/O",
            Rect::new(header_bounds.x + 8, header_bounds.y + 8, 160, 14),
            2,
            theme.io_pages.page_title,
        )?;
        crate::ui::draw_text_fitted(
            canvas,
            "Auto refresh: on",
            Rect::new(header_bounds.x + 188, header_bounds.y + 8, 220, 8),
            1,
            theme.io_pages.subtitle,
        )?;
        let offline_input = self
            .preferred_default_input_name
            .as_deref()
            .filter(|name| !self.input_port_is_available(name));
        let offline_output = self
            .preferred_default_output_name
            .as_deref()
            .filter(|name| !self.output_port_is_available(name));
        let offline_summary = match (offline_input, offline_output) {
            (Some(input), Some(output)) => format!("Offline defaults: In {input} | Out {output}"),
            (Some(input), None) => format!("Offline default input: {input}"),
            (None, Some(output)) => format!("Offline default output: {output}"),
            (None, None) => "Select default inputs and outputs".to_string(),
        };
        crate::ui::draw_text_fitted(
            canvas,
            &offline_summary,
            Rect::new(header_bounds.x + 188, header_bounds.y + 18, 420, 8),
            1,
            if offline_input.is_some() || offline_output.is_some() {
                theme.io_pages.warning_text
            } else {
                theme.io_pages.subtitle
            },
        )?;

        canvas.set_draw_color(theme.io_pages.panel_bg);
        canvas.fill_rect(layout.input_header)?;
        canvas.fill_rect(layout.output_header)?;
        canvas.set_draw_color(theme.io_pages.page_border);
        canvas.draw_rect(layout.input_header)?;
        canvas.draw_rect(layout.output_header)?;
        crate::ui::draw_text_fitted(
            canvas,
            "Inputs",
            Rect::new(layout.input_header.x + 8, layout.input_header.y + 7, 96, 8),
            2,
            theme.io_pages.inputs_title,
        )?;
        crate::ui::draw_text_fitted(
            canvas,
            "Outputs",
            Rect::new(
                layout.output_header.x + 8,
                layout.output_header.y + 7,
                96,
                8,
            ),
            2,
            theme.io_pages.outputs_title,
        )?;

        self.draw_device_list(
            canvas,
            layout.input_list,
            &self.midi_devices.inputs,
            self.page_state.midi_io.selected_input_index,
            self.midi_devices.selected_input,
            self.page_state.midi_io.focus == MidiIoListFocus::Inputs,
            theme.app_chrome.tab_accent_midi_io,
            "Input",
        )?;
        self.draw_device_list(
            canvas,
            layout.output_list,
            &self.midi_devices.outputs,
            self.page_state.midi_io.selected_output_index,
            self.midi_devices.selected_output,
            self.page_state.midi_io.focus == MidiIoListFocus::Outputs,
            theme.app_chrome.tab_accent_mappings,
            "Output",
        )?;

        Ok(())
    }

    fn draw_device_list<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        bounds: Rect,
        ports: &[MidiPortRef],
        selected_index: usize,
        active_index: Option<usize>,
        focused: bool,
        accent: Color,
        role_label: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let theme = self.theme();
        canvas.set_draw_color(theme.io_pages.page_bg);
        canvas.fill_rect(bounds)?;
        canvas.set_draw_color(if focused {
            theme.io_pages.focus_border
        } else {
            theme.io_pages.page_border
        });
        canvas.draw_rect(bounds)?;

        let rows = crate::ui::stacked_rows(
            crate::ui::inset_rect(
                bounds,
                self.ui_metrics().midi_list_inset_px,
                self.ui_metrics().midi_list_inset_px,
            )?,
            ports.len().max(1),
            self.ui_metrics().row_gap_px,
        );
        for (index, row) in rows.into_iter().enumerate().take(ports.len()) {
            let is_selected = index == selected_index;
            let is_active = active_index == Some(index);

            canvas.set_draw_color(if is_selected {
                theme.io_pages.row_selected_bg
            } else {
                theme.io_pages.row_idle_bg
            });
            canvas.fill_rect(row)?;
            canvas.set_draw_color(if is_selected {
                theme.io_pages.row_selected_border
            } else {
                theme.io_pages.row_idle_border
            });
            canvas.draw_rect(row)?;

            let status = Rect::new(row.x + 6, row.y + 6, 16, row.height().saturating_sub(12));
            canvas.set_draw_color(if is_active {
                accent
            } else {
                theme.io_pages.device_status_idle
            });
            canvas.fill_rect(status)?;

            let selected_badge_width = if is_selected { 24 } else { 0 };
            let active_badge_width = if is_active { 24 } else { 0 };
            let reserved_badge_width = selected_badge_width + active_badge_width;
            let header_rect = Rect::new(
                status.x + status.width() as i32 + 8,
                row.y + 8,
                row.width()
                    .saturating_sub(40)
                    .saturating_sub(reserved_badge_width as u32),
                8,
            );
            let body_rect = Rect::new(
                status.x + status.width() as i32 + 8,
                row.y + 20,
                row.width().saturating_sub(40),
                row.height().saturating_sub(28),
            );
            let body_fill = if is_selected {
                theme.io_pages.device_body_selected
            } else {
                theme.io_pages.device_body_idle
            };
            canvas.set_draw_color(body_fill);
            canvas.fill_rect(body_rect)?;
            crate::ui::draw_text_fitted(
                canvas,
                &ports[index].name,
                header_rect,
                1,
                if is_selected {
                    contrasting_text_color(theme.io_pages.row_selected_bg, theme)
                } else {
                    theme.io_pages.label_text
                },
            )?;
            if is_active {
                let active_badge = Rect::new(
                    row.x + row.width() as i32 - 12 - active_badge_width - selected_badge_width,
                    row.y + 8,
                    active_badge_width as u32,
                    8,
                );
                canvas.set_draw_color(accent);
                canvas.fill_rect(active_badge)?;
                crate::ui::draw_text_fitted(
                    canvas,
                    if role_label == "Input" { "Def" } else { "Def" },
                    Rect::new(
                        active_badge.x + 3,
                        active_badge.y,
                        active_badge.width().saturating_sub(6),
                        8,
                    ),
                    1,
                    contrasting_text_color(accent, theme),
                )?;
            }
            if is_selected {
                let selected_badge = Rect::new(
                    row.x + row.width() as i32 - 12 - selected_badge_width,
                    row.y + 8,
                    selected_badge_width as u32,
                    8,
                );
                canvas.set_draw_color(theme.io_pages.selected_badge_fill);
                canvas.fill_rect(selected_badge)?;
                crate::ui::draw_text_fitted(
                    canvas,
                    "Sel",
                    Rect::new(
                        selected_badge.x + 3,
                        selected_badge.y,
                        selected_badge.width().saturating_sub(6),
                        8,
                    ),
                    1,
                    contrasting_text_color(theme.io_pages.selected_badge_fill, theme),
                )?;
            }
        }

        Ok(())
    }

    pub(crate) fn handle_midi_io_pointer(
        &mut self,
        content_bounds: Rect,
        x: i32,
        y: i32,
        _source: crate::actions::ActionSource,
    ) -> Option<AppControl> {
        let layout = self.midi_io_page_layout(content_bounds).ok()?;

        if rect_contains(layout.input_header, x, y) {
            self.page_state.midi_io.focus = MidiIoListFocus::Inputs;
            return Some(AppControl::Continue);
        }
        if rect_contains(layout.output_header, x, y) {
            self.page_state.midi_io.focus = MidiIoListFocus::Outputs;
            return Some(AppControl::Continue);
        }

        if let Some(index) =
            self.hit_device_list_row(layout.input_list, self.midi_devices.inputs.len(), x, y)
        {
            self.page_state.midi_io.focus = MidiIoListFocus::Inputs;
            self.page_state.midi_io.selected_input_index = index;
            self.set_preferred_default_input_from_index(index);
            return Some(AppControl::Continue);
        }

        if let Some(index) =
            self.hit_device_list_row(layout.output_list, self.midi_devices.outputs.len(), x, y)
        {
            self.page_state.midi_io.focus = MidiIoListFocus::Outputs;
            self.page_state.midi_io.selected_output_index = index;
            self.set_preferred_default_output_from_index(index);
            return Some(AppControl::Continue);
        }

        None
    }

    fn hit_device_list_row(&self, bounds: Rect, count: usize, x: i32, y: i32) -> Option<usize> {
        let rows = crate::ui::stacked_rows(
            crate::ui::inset_rect(
                bounds,
                self.ui_metrics().midi_list_inset_px,
                self.ui_metrics().midi_list_inset_px,
            )
            .ok()?,
            count.max(1),
            self.ui_metrics().row_gap_px,
        );
        rows.into_iter()
            .enumerate()
            .take(count)
            .find_map(|(index, rect)| rect_contains(rect, x, y).then_some(index))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn midi_io_page_can_switch_focus_and_commit_default_ports() {
        let mut app = App::new();
        app.midi_devices.inputs = vec![MidiPortRef::new("In A"), MidiPortRef::new("In B")];
        app.midi_devices.outputs = vec![MidiPortRef::new("Out A"), MidiPortRef::new("Out B")];
        app.apply_action(AppAction::ShowPage(AppPage::MidiIo));
        app.apply_action(AppAction::SelectNextPageItem);
        app.apply_action(AppAction::ActivatePageItem);
        assert_eq!(
            app.midi_devices.selected_input,
            Some(app.page_state.midi_io.selected_input_index)
        );

        app.apply_action(AppAction::AdjustPageItemForward);
        assert_eq!(app.page_state.midi_io.focus, MidiIoListFocus::Outputs);
    }

    #[test]
    fn midi_io_layout_matches_origin_main_at_default_density() {
        let app = App::new();
        let layout = app.midi_io_page_layout(Rect::new(0, 0, 800, 600)).unwrap();

        assert_eq!(layout.header_bounds, Rect::new(0, 0, 800, 28));
        assert_eq!(layout.input_header, Rect::new(0, 38, 393, 22));
        assert_eq!(layout.output_header, Rect::new(407, 38, 393, 22));
        assert_eq!(layout.input_list, Rect::new(0, 66, 393, 512));
        assert_eq!(layout.output_list, Rect::new(407, 66, 393, 512));
    }
}
