use super::*;

impl App {
    pub(crate) fn mapping_row_cells(&self, row: Rect) -> [Rect; 6] {
        let type_rect = Rect::new(row.x + 4, row.y + 3, 46, row.height().saturating_sub(6));
        let source_rect = Rect::new(
            type_rect.x + type_rect.width() as i32 + 6,
            row.y + 3,
            92,
            row.height().saturating_sub(6),
        );
        let device_rect = Rect::new(
            source_rect.x + source_rect.width() as i32 + 6,
            row.y + 3,
            98,
            row.height().saturating_sub(6),
        );
        let enabled_rect = Rect::new(
            row.x + row.width() as i32 - 34,
            row.y + 3,
            28,
            row.height().saturating_sub(6),
        );
        let scope_rect = Rect::new(
            enabled_rect.x - 80,
            row.y + 3,
            72,
            row.height().saturating_sub(6),
        );
        let target_rect = Rect::new(
            device_rect.x + device_rect.width() as i32 + 6,
            row.y + 3,
            (scope_rect.x - (device_rect.x + device_rect.width() as i32 + 12)).max(48) as u32,
            row.height().saturating_sub(6),
        );
        [
            type_rect,
            source_rect,
            device_rect,
            target_rect,
            scope_rect,
            enabled_rect,
        ]
    }

    pub(crate) fn draw_mappings_page<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        content_bounds: Rect,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let theme = self.theme();
        canvas.set_draw_color(theme.mappings.page_bg);
        canvas.fill_rect(content_bounds)?;
        canvas.set_draw_color(theme.app_chrome.surface_border);
        canvas.draw_rect(content_bounds)?;
        crate::ui::draw_text_fitted(
            canvas,
            "Mappings",
            Rect::new(content_bounds.x + 8, content_bounds.y + 8, 180, 14),
            2,
            theme.mappings.page_title,
        )?;
        let overview_badge = Rect::new(content_bounds.x + 200, content_bounds.y + 8, 188, 16);
        canvas.set_draw_color(if self.page_state.mapping_mode == MappingPageMode::Write {
            theme.mappings.write_mode_active
        } else {
            theme.mappings.write_mode_inactive
        });
        canvas.fill_rect(overview_badge)?;
        canvas.set_draw_color(theme.mappings.page_title);
        canvas.draw_rect(overview_badge)?;
        crate::ui::draw_text_fitted(
            canvas,
            &format!("Tap Mode: {}", self.page_state.mapping_mode.label()),
            Rect::new(content_bounds.x + 208, content_bounds.y + 12, 170, 8),
            1,
            theme.mappings.overview_text,
        )?;
        let learn_badge = Rect::new(content_bounds.x + 392, content_bounds.y + 8, 136, 16);
        canvas.set_draw_color(if self.page_state.mapping_midi_learn_armed {
            theme.mappings.learn_armed
        } else {
            theme.mappings.learn_idle
        });
        canvas.fill_rect(learn_badge)?;
        canvas.set_draw_color(
            if self.page_state.selected_mapping_field == MappingField::SourceValue
                && self.page_state.mapping_mode == MappingPageMode::Write
            {
                theme.mappings.learn_selected_border
            } else {
                theme.mappings.learn_idle_border
            },
        );
        canvas.draw_rect(learn_badge)?;
        crate::ui::draw_text_fitted(
            canvas,
            if self.page_state.mapping_midi_learn_armed {
                "Tap Learn: waiting"
            } else {
                "Tap Learn: idle"
            },
            Rect::new(learn_badge.x + 8, learn_badge.y + 4, 120, 8),
            1,
            theme.mappings.learn_text,
        )?;
        let direct_badge = Rect::new(content_bounds.x + 532, content_bounds.y + 8, 154, 16);
        canvas.set_draw_color(
            if self.direct_mapping_state.mode == DirectMappingMode::Inactive {
                theme.mappings.direct_badge_idle_fill
            } else {
                theme.mappings.direct_armed_fill
            },
        );
        canvas.fill_rect(direct_badge)?;
        canvas.set_draw_color(
            if self.direct_mapping_state.mode == DirectMappingMode::Inactive {
                theme.mappings.direct_idle_border
            } else {
                theme.mappings.direct_armed_border
            },
        );
        canvas.draw_rect(direct_badge)?;
        crate::ui::draw_text_fitted(
            canvas,
            if self.direct_mapping_state.mode == DirectMappingMode::Inactive {
                "Tap Direct Map"
            } else {
                "Tap Direct: armed"
            },
            Rect::new(direct_badge.x + 8, direct_badge.y + 4, 138, 8),
            1,
            theme.mappings.direct_text,
        )?;
        crate::ui::draw_text_fitted(
            canvas,
            &format!(
                "Rows {} / {}",
                self.page_state
                    .selected_mapping_index
                    .saturating_add(1)
                    .min(self.mappings.len()),
                self.mappings.len()
            ),
            Rect::new(
                content_bounds.x + content_bounds.width() as i32 - 100,
                content_bounds.y + 12,
                92,
                8,
            ),
            1,
            theme.mappings.meta_text,
        )?;

        let footer_bounds = Rect::new(
            content_bounds.x + 8,
            content_bounds.y + content_bounds.height() as i32 - 20,
            content_bounds.width().saturating_sub(16),
            12,
        );
        let list_bounds = Rect::new(
            content_bounds.x + 8,
            content_bounds.y + 44,
            content_bounds.width().saturating_sub(16),
            content_bounds.height().saturating_sub(68),
        );
        let header_row = Rect::new(
            list_bounds.x,
            content_bounds.y + 30,
            list_bounds.width(),
            10,
        );
        let header_cells = self.mapping_row_cells(Rect::new(
            header_row.x,
            header_row.y,
            header_row.width(),
            18,
        ));
        for (index, field) in MappingField::ALL.iter().enumerate() {
            crate::ui::draw_text_fitted(
                canvas,
                field.label(),
                Rect::new(
                    header_cells[index].x,
                    header_row.y,
                    header_cells[index].width(),
                    8,
                ),
                1,
                Color::RGB(154, 166, 182),
            )?;
        }
        let row_gap = 3_i32;
        let row_height = 18_i32;
        let stride = row_height + row_gap;
        let visible_rows = ((list_bounds.height() as i32 + row_gap) / stride).max(1) as usize;
        let selected_index = self
            .page_state
            .selected_mapping_index
            .min(self.mappings.len().saturating_sub(1));
        let start_index = if self.mappings.len() <= visible_rows {
            0
        } else {
            selected_index
                .saturating_sub(visible_rows / 2)
                .min(self.mappings.len() - visible_rows)
        };

        for visible_index in 0..visible_rows {
            let index = start_index + visible_index;
            if index >= self.mappings.len() {
                break;
            }
            let row = Rect::new(
                list_bounds.x,
                list_bounds.y + visible_index as i32 * stride,
                list_bounds.width(),
                row_height as u32,
            );
            let entry = &self.mappings[index];
            let selected = index == self.page_state.selected_mapping_index;
            canvas.set_draw_color(if selected {
                theme.mappings.row_selected_fill
            } else {
                theme.mappings.row_idle_fill
            });
            canvas.fill_rect(row)?;
            canvas.set_draw_color(if selected {
                theme.mappings.page_title
            } else {
                theme.mappings.row_idle_border
            });
            canvas.draw_rect(row)?;

            let cells = self.mapping_row_cells(row);
            let source_rect = Rect::new(cells[0].x, cells[0].y, 14, cells[0].height());
            let source_color = match entry.source_kind {
                MappingSourceKind::Key => theme.mappings.source_kind_key,
                MappingSourceKind::Midi => theme.mappings.source_kind_midi,
                MappingSourceKind::Osc => theme.mappings.source_kind_osc,
            };
            canvas.set_draw_color(source_color);
            canvas.fill_rect(source_rect)?;

            let enabled_rect = Rect::new(cells[5].x + 6, cells[5].y, 14, cells[5].height());
            canvas.set_draw_color(if entry.enabled {
                theme.mappings.enabled_fill_on
            } else {
                theme.mappings.enabled_fill_off
            });
            canvas.fill_rect(enabled_rect)?;

            let kind_rect = cells[0];
            let device_rect = cells[1];
            let trigger_rect = cells[2];
            let target_rect = cells[3];
            let scope_rect = cells[4];
            canvas.set_draw_color(if selected {
                theme.mappings.field_fill_selected
            } else {
                theme.mappings.field_fill_idle
            });
            canvas.fill_rect(kind_rect)?;
            canvas.fill_rect(trigger_rect)?;
            canvas.fill_rect(device_rect)?;
            canvas.set_draw_color(if entry.enabled {
                theme.mappings.target_fill_enabled
            } else {
                theme.mappings.target_fill_disabled
            });
            canvas.fill_rect(target_rect)?;
            canvas.set_draw_color(theme.mappings.scope_fill);
            canvas.fill_rect(scope_rect)?;
            canvas.fill_rect(cells[5])?;
            if selected && self.page_state.mapping_mode == MappingPageMode::Write {
                let field_rect = cells[mapping_field_index(self.page_state.selected_mapping_field)];
                canvas.set_draw_color(
                    if self.page_state.mapping_midi_learn_armed
                        && self.page_state.selected_mapping_field == MappingField::SourceValue
                    {
                        theme.mappings.write_field_learn
                    } else {
                        theme.mappings.write_field_active
                    },
                );
                canvas.fill_rect(field_rect)?;
            }
            crate::ui::draw_text_fitted(
                canvas,
                mapping_source_label(entry.source_kind),
                Rect::new(
                    kind_rect.x + 18,
                    row.y + 5,
                    kind_rect.width().saturating_sub(22),
                    8,
                ),
                1,
                theme.app_chrome.action_text,
            )?;
            crate::ui::draw_text_fitted(
                canvas,
                &entry.source_label,
                Rect::new(
                    trigger_rect.x + 4,
                    row.y + 5,
                    trigger_rect.width().saturating_sub(8),
                    8,
                ),
                1,
                theme.app_chrome.action_text,
            )?;
            let mapping_device_label = if entry.source_kind == MappingSourceKind::Midi {
                if entry.source_device_label != default_mapping_source_device()
                    && !self.input_port_is_available(&entry.source_device_label)
                {
                    format!("{} (offline)", entry.source_device_label)
                } else {
                    entry.source_device_label.clone()
                }
            } else {
                "--".to_string()
            };
            crate::ui::draw_text_fitted(
                canvas,
                &mapping_device_label,
                Rect::new(
                    device_rect.x + 4,
                    row.y + 5,
                    device_rect.width().saturating_sub(8),
                    8,
                ),
                1,
                if entry.source_kind == MappingSourceKind::Midi {
                    theme.mappings.device_text_active
                } else {
                    theme.mappings.device_text_inactive
                },
            )?;
            crate::ui::draw_text_fitted(
                canvas,
                &if selected
                    && self.page_state.mapping_mode == MappingPageMode::Write
                    && self.page_state.selected_mapping_field == MappingField::Target
                    && self.target_lookup_state.active.is_some()
                {
                    self.target_lookup_state
                        .active
                        .as_ref()
                        .map(|lookup| {
                            if lookup.query.is_empty() {
                                "Search target…".to_string()
                            } else {
                                format!("Search: {}", lookup.query)
                            }
                        })
                        .unwrap_or_else(|| entry.target_label.clone())
                } else {
                    entry.target_label.clone()
                },
                Rect::new(
                    target_rect.x + 4,
                    row.y + 5,
                    target_rect.width().saturating_sub(8),
                    8,
                ),
                1,
                theme.mappings.target_text,
            )?;
            crate::ui::draw_text_fitted(
                canvas,
                compact_scope_label(&entry.scope_label),
                Rect::new(
                    scope_rect.x + 4,
                    row.y + 5,
                    scope_rect.width().saturating_sub(8),
                    8,
                ),
                1,
                theme.mappings.scope_text,
            )?;
            crate::ui::draw_text_fitted(
                canvas,
                if entry.enabled { "On" } else { "Off" },
                Rect::new(
                    cells[5].x + 2,
                    row.y + 5,
                    cells[5].width().saturating_sub(4),
                    8,
                ),
                1,
                theme.mappings.scope_text,
            )?;

            if selected && self.page_state.mapping_mode == MappingPageMode::Write {
                let field_rect = cells[mapping_field_index(self.page_state.selected_mapping_field)];
                canvas.set_draw_color(
                    if self.page_state.mapping_midi_learn_armed
                        && self.page_state.selected_mapping_field == MappingField::SourceValue
                    {
                        theme.mappings.write_field_border_learn
                    } else {
                        theme.mappings.write_field_border
                    },
                );
                canvas.draw_rect(field_rect)?;
                let tap_tag = Rect::new(row.x + row.width() as i32 - 68, row.y + 3, 34, 12);
                canvas.set_draw_color(theme.mappings.tap_badge_fill);
                canvas.fill_rect(tap_tag)?;
                crate::ui::draw_text_fitted(
                    canvas,
                    "Tap",
                    Rect::new(
                        tap_tag.x + 6,
                        tap_tag.y + 2,
                        tap_tag.width().saturating_sub(12),
                        8,
                    ),
                    1,
                    theme.app_chrome.action_text,
                )?;
            }
        }

        self.draw_mapping_target_lookup(canvas, content_bounds)?;

        canvas.set_draw_color(theme.mappings.footer_bg);
        canvas.fill_rect(footer_bounds)?;
        let footer_tokens = [
            ("Tap row", theme.mappings.footer_token_row),
            ("Tap field", theme.mappings.footer_token_field),
            ("Tap again act", theme.mappings.footer_token_act),
            ("W Write", theme.mappings.footer_token_write),
            ("F8 Direct", theme.mappings.footer_token_direct),
            ("N New", theme.mappings.footer_token_new),
            ("Del/Bsp Remove", theme.mappings.footer_token_remove),
        ];
        let mut footer_x = footer_bounds.x + 6;
        for (label, fill) in footer_tokens {
            let token = Rect::new(
                footer_x,
                footer_bounds.y + 1,
                crate::ui::text_width(label, 1) + 12,
                footer_bounds.height().saturating_sub(2),
            );
            canvas.set_draw_color(fill);
            canvas.fill_rect(token)?;
            crate::ui::draw_text_fitted(
                canvas,
                label,
                Rect::new(
                    token.x + 6,
                    token.y + 2,
                    token.width().saturating_sub(12),
                    8,
                ),
                1,
                theme.app_chrome.action_text,
            )?;
            footer_x += token.width() as i32 + 6;
        }
        crate::ui::draw_text_fitted(
            canvas,
            if self.target_lookup_state.active.is_some() {
                "Type filter  Up/Down Select  Enter Commit  Esc Cancel  Tab stays in lookup"
            } else {
                "Shift+Left/Right Field  Q/E Adjust  Enter Learn/Toggle"
            },
            Rect::new(
                footer_x + 6,
                footer_bounds.y + 2,
                footer_bounds
                    .width()
                    .saturating_sub((footer_x - footer_bounds.x) as u32)
                    .saturating_sub(12),
                8,
            ),
            1,
            theme.mappings.meta_text,
        )?;

        Ok(())
    }

    pub(crate) fn draw_mappings_overlay<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        bounds: Rect,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let theme = self.theme();
        canvas.set_draw_color(theme.app_chrome.overlay_backdrop);
        canvas.fill_rect(bounds)?;

        let panel = Rect::new(
            bounds.x + 84,
            bounds.y + 44,
            bounds.width() - 168,
            bounds.height() - 88,
        );
        canvas.set_draw_color(theme.app_chrome.overlay_panel_fill);
        canvas.fill_rect(panel)?;
        canvas.set_draw_color(theme.mappings.page_title);
        canvas.draw_rect(panel)?;
        let title_bounds = Rect::new(panel.x + 12, panel.y + 12, 220, 14);
        crate::ui::draw_text_fitted(
            canvas,
            "Mappings Overlay",
            title_bounds,
            2,
            theme.mappings.page_title,
        )?;
        crate::ui::draw_text_fitted(
            canvas,
            "F5 Close",
            Rect::new(panel.x + 12, panel.y + 32, 58, 8),
            1,
            theme.app_chrome.detail_text,
        )?;
        crate::ui::draw_text_fitted(
            canvas,
            "W Write",
            Rect::new(panel.x + 80, panel.y + 32, 52, 8),
            1,
            theme.app_chrome.detail_text,
        )?;
        crate::ui::draw_text_fitted(
            canvas,
            "Trigger",
            Rect::new(panel.x + 12, panel.y + 46, 56, 8),
            1,
            theme.app_chrome.overlay_header_text,
        )?;
        crate::ui::draw_text_fitted(
            canvas,
            "Action",
            Rect::new(panel.x + 146, panel.y + 46, 48, 8),
            1,
            theme.app_chrome.overlay_header_text,
        )?;
        crate::ui::draw_text_fitted(
            canvas,
            "Scope",
            Rect::new(panel.x + panel.width() as i32 - 126, panel.y + 46, 44, 8),
            1,
            theme.app_chrome.overlay_header_text,
        )?;

        let list_bounds = crate::ui::inset_rect(panel, 12, 66)?;
        let row_height = 18_i32;
        let row_gap = 3_i32;
        let stride = row_height + row_gap;
        let visible_rows = ((list_bounds.height() as i32 + row_gap) / stride).max(1) as usize;
        let selected_index = self
            .page_state
            .selected_mapping_index
            .min(self.mappings.len().saturating_sub(1));
        let start_index = if self.mappings.len() <= visible_rows {
            0
        } else {
            selected_index
                .saturating_sub(visible_rows / 2)
                .min(self.mappings.len() - visible_rows)
        };

        for visible_index in 0..visible_rows {
            let index = start_index + visible_index;
            if index >= self.mappings.len() {
                break;
            }
            let row = Rect::new(
                list_bounds.x,
                list_bounds.y + visible_index as i32 * stride,
                list_bounds.width(),
                row_height as u32,
            );
            let entry = &self.mappings[index];
            let selected = index == self.page_state.selected_mapping_index;
            canvas.set_draw_color(if selected {
                theme.app_chrome.overlay_row_selected_fill
            } else {
                theme.app_chrome.overlay_row_idle_fill
            });
            canvas.fill_rect(row)?;
            canvas.set_draw_color(if selected {
                theme.mappings.page_title
            } else {
                theme.app_chrome.overlay_row_idle_border
            });
            canvas.draw_rect(row)?;

            crate::ui::draw_text_fitted(
                canvas,
                &entry.source_label,
                Rect::new(row.x + 8, row.y + 5, 126, 8),
                1,
                theme.app_chrome.action_text,
            )?;
            crate::ui::draw_text_fitted(
                canvas,
                &entry.target_label,
                Rect::new(row.x + 146, row.y + 5, 210, 8),
                1,
                theme.app_chrome.overlay_target_text,
            )?;
            crate::ui::draw_text_fitted(
                canvas,
                compact_scope_label(&entry.scope_label),
                Rect::new(row.x + row.width() as i32 - 126, row.y + 5, 90, 8),
                1,
                theme.app_chrome.overlay_scope_text,
            )?;
        }

        crate::ui::draw_text_fitted(
            canvas,
            &format!(
                "Rows {}-{} / {}",
                start_index.saturating_add(1),
                (start_index + visible_rows).min(self.mappings.len()),
                self.mappings.len()
            ),
            Rect::new(panel.x + panel.width() as i32 - 116, panel.y + 34, 104, 8),
            1,
            theme.app_chrome.overlay_meta_text,
        )?;

        Ok(())
    }
}

impl App {
    pub(crate) fn adjust_mapping_field(&mut self, delta: i32) {
        let index = self.page_state.selected_mapping_index;
        let field = self.page_state.selected_mapping_field;
        let track_count = self.project.tracks.len();
        let mapping_device_names = self
            .midi_devices
            .inputs
            .iter()
            .map(|port| port.name.clone())
            .collect::<Vec<_>>();
        let Some(entry) = self.mappings.get_mut(index) else {
            return;
        };

        self.page_state.mapping_midi_learn_armed = false;
        match field {
            MappingField::SourceKind => {
                entry.source_kind = cycle_mapping_source_kind(entry.source_kind, delta);
                if entry.source_kind != MappingSourceKind::Midi {
                    entry.source_device_label = default_mapping_source_device();
                }
                entry.source_label = default_source_label(entry.source_kind).to_string();
                self.normalize_selected_mapping_field();
            }
            MappingField::SourceDevice => {
                if entry.source_kind == MappingSourceKind::Midi {
                    entry.source_device_label = cycle_mapping_source_device_label(
                        &entry.source_device_label,
                        &mapping_device_names,
                        delta,
                    );
                }
            }
            MappingField::SourceValue => {
                entry.source_label =
                    cycle_mapping_source_label(entry.source_kind, &entry.source_label, delta)
                        .to_string();
            }
            MappingField::Target => {
                entry.target_label =
                    cycle_mapping_target_label(&entry.target_label, delta).to_string();
                if !mapping_scope_valid_for_target(
                    &entry.target_label,
                    &entry.scope_label,
                    track_count,
                ) {
                    entry.scope_label = default_scope_label(&entry.target_label, track_count);
                }
            }
            MappingField::Scope => {
                entry.scope_label = cycle_mapping_scope_value(
                    &entry.scope_label,
                    delta,
                    &entry.target_label,
                    track_count,
                );
            }
            MappingField::Enabled => {
                entry.enabled = delta > 0;
            }
        }
    }

    pub(crate) fn reverse_activate_page_item(&mut self) {
        match self.page_state.current_page {
            AppPage::Timeline => self.reverse_activate_timeline_context_item(),
            _ => self.activate_page_item(),
        }
    }

    pub(crate) fn activate_mapping_field(&mut self) {
        let index = self.page_state.selected_mapping_index;
        let field = self.page_state.selected_mapping_field;

        if field == MappingField::Target {
            self.open_mapping_target_lookup();
            return;
        }

        let track_count = self.project.tracks.len();
        let Some(entry) = self.mappings.get_mut(index) else {
            return;
        };

        match field {
            MappingField::SourceKind => {
                entry.source_kind = cycle_mapping_source_kind(entry.source_kind, 1);
                if entry.source_kind != MappingSourceKind::Midi {
                    entry.source_device_label = default_mapping_source_device();
                }
                entry.source_label = default_source_label(entry.source_kind).to_string();
                self.page_state.mapping_midi_learn_armed = false;
                self.normalize_selected_mapping_field();
            }
            MappingField::SourceDevice => {
                if entry.source_kind == MappingSourceKind::Midi {
                    let mapping_device_names = self
                        .midi_devices
                        .inputs
                        .iter()
                        .map(|port| port.name.clone())
                        .collect::<Vec<_>>();
                    entry.source_device_label = cycle_mapping_source_device_label(
                        &entry.source_device_label,
                        &mapping_device_names,
                        1,
                    );
                }
                self.page_state.mapping_midi_learn_armed = false;
            }
            MappingField::SourceValue => {
                if entry.source_kind == MappingSourceKind::Midi {
                    self.page_state.mapping_midi_learn_armed =
                        !self.page_state.mapping_midi_learn_armed;
                    self.sync_midi_inputs();
                } else {
                    entry.source_label =
                        cycle_mapping_source_label(entry.source_kind, &entry.source_label, 1)
                            .to_string();
                }
            }
            MappingField::Target => {}
            MappingField::Scope => {
                entry.scope_label = cycle_mapping_scope_value(
                    &entry.scope_label,
                    1,
                    &entry.target_label,
                    track_count,
                );
                self.page_state.mapping_midi_learn_armed = false;
            }
            MappingField::Enabled => {
                entry.enabled = !entry.enabled;
                self.page_state.mapping_midi_learn_armed = false;
            }
        }
    }

    pub(crate) fn add_mapping_row(&mut self) {
        if self.page_state.current_page != AppPage::Mappings
            || self.page_state.mapping_mode != MappingPageMode::Write
        {
            return;
        }

        self.clear_mapping_target_lookup();
        let insert_index = self
            .page_state
            .selected_mapping_index
            .min(self.mappings.len());
        let mut entry = self
            .mappings
            .get(insert_index)
            .cloned()
            .unwrap_or_else(MappingEntry::default_new);
        entry.enabled = false;
        entry.scope_label = default_scope_label(&entry.target_label, self.project.tracks.len());
        self.mappings
            .insert(insert_index + usize::from(!self.mappings.is_empty()), entry);
        self.page_state.selected_mapping_index =
            (insert_index + usize::from(!self.mappings.is_empty())).min(self.mappings.len() - 1);
        self.normalize_selected_mapping_field();
        self.page_state.mapping_midi_learn_armed = false;
    }

    pub(crate) fn remove_selected_mapping(&mut self) {
        if self.page_state.current_page != AppPage::Mappings
            || self.page_state.mapping_mode != MappingPageMode::Write
            || self.mappings.is_empty()
        {
            return;
        }

        self.clear_mapping_target_lookup();
        self.mappings.remove(self.page_state.selected_mapping_index);
        if self.mappings.is_empty() {
            self.mappings.push(MappingEntry::default_new());
        }
        self.page_state.selected_mapping_index = self
            .page_state
            .selected_mapping_index
            .min(self.mappings.len().saturating_sub(1));
        self.normalize_selected_mapping_field();
        self.page_state.mapping_midi_learn_armed = false;
    }

    pub(crate) fn next_enabled_mapping_field(&self, start: MappingField) -> MappingField {
        let mut field = start;
        for _ in 0..MappingField::ALL.len() {
            field = field.next();
            if self.mapping_field_enabled(field) {
                return field;
            }
        }
        start
    }

    pub(crate) fn previous_enabled_mapping_field(&self, start: MappingField) -> MappingField {
        let mut field = start;
        for _ in 0..MappingField::ALL.len() {
            field = field.previous();
            if self.mapping_field_enabled(field) {
                return field;
            }
        }
        start
    }

    pub(crate) fn normalize_selected_mapping_field(&mut self) {
        if !self.mapping_field_enabled(self.page_state.selected_mapping_field) {
            self.page_state.selected_mapping_field =
                self.next_enabled_mapping_field(self.page_state.selected_mapping_field);
        }
    }

    pub(crate) fn mapping_field_enabled(&self, field: MappingField) -> bool {
        let Some(entry) = self.mappings.get(self.page_state.selected_mapping_index) else {
            return field != MappingField::SourceDevice;
        };
        !matches!(field, MappingField::SourceDevice) || entry.source_kind == MappingSourceKind::Midi
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mappings_overlay_toggles_on_and_off() {
        let mut app = App::new();
        assert!(app.overlay_state.active.is_none());

        app.apply_action(AppAction::ToggleMappingsOverlay);
        assert_eq!(
            app.overlay_state.active,
            Some(AppOverlay::MappingsQuickView)
        );

        app.apply_action(AppAction::ToggleMappingsOverlay);
        assert!(app.overlay_state.active.is_none());
    }

    #[test]
    fn mappings_page_is_read_only() {
        let mut app = App::new();
        app.apply_action(AppAction::ShowPage(AppPage::Mappings));
        let before = app.mappings[0].enabled;

        app.apply_action(AppAction::ActivatePageItem);
        assert_eq!(app.mappings[0].enabled, before);
    }

    #[test]
    fn mappings_page_write_mode_can_edit_enabled_state() {
        let mut app = App::new();
        app.apply_action(AppAction::ShowPage(AppPage::Mappings));
        let before = app.mappings[0].enabled;

        app.apply_action(AppAction::ToggleMappingsWriteMode);
        app.page_state.selected_mapping_field = MappingField::Enabled;
        app.apply_action(AppAction::ActivatePageItem);

        assert_ne!(app.mappings[0].enabled, before);
    }

    #[test]
    fn mappings_page_write_mode_cycles_selected_field() {
        let mut app = App::new();
        app.apply_action(AppAction::ShowPage(AppPage::Mappings));
        app.apply_action(AppAction::ToggleMappingsWriteMode);
        assert_eq!(app.page_state.mapping_mode, MappingPageMode::Write);
        assert_eq!(
            app.page_state.selected_mapping_field,
            MappingField::SourceValue
        );

        app.apply_action(AppAction::SelectNextPageField);
        assert_eq!(app.page_state.selected_mapping_field, MappingField::Target);
    }

    #[test]
    fn mappings_page_write_mode_can_add_and_remove_rows() {
        let mut app = App::new();
        app.apply_action(AppAction::ShowPage(AppPage::Mappings));
        app.apply_action(AppAction::ToggleMappingsWriteMode);
        let original_len = app.mappings.len();
        let selected_index = app.page_state.selected_mapping_index;

        app.apply_action(AppAction::AddMappingRow);

        assert_eq!(app.mappings.len(), original_len + 1);
        assert_eq!(app.page_state.selected_mapping_index, selected_index + 1);
        assert!(!app.mappings[app.page_state.selected_mapping_index].enabled);

        app.apply_action(AppAction::RemoveSelectedMapping);

        assert_eq!(app.mappings.len(), original_len);
        assert!(app.page_state.selected_mapping_index < app.mappings.len());
    }

    #[test]
    fn mappings_target_lookup_opens_and_commits_filtered_result() {
        let mut app = App::new();
        app.apply_action(AppAction::ShowPage(AppPage::Mappings));
        app.apply_action(AppAction::ToggleMappingsWriteMode);
        app.page_state.selected_mapping_field = MappingField::Target;
        app.mappings[0].target_label = "Play/Stop".to_string();
        app.mappings[0].scope_label = "Global".to_string();

        app.apply_action(AppAction::ActivatePageItem);
        assert!(app.target_lookup_state.active.is_some());

        let _ = app.handle_keyboard_event(&sdl3::event::Event::KeyDown {
            timestamp: 0,
            window_id: 0,
            which: 0,
            scancode: None,
            keycode: Some(sdl3::keyboard::Keycode::A),
            keymod: sdl3::keyboard::Mod::NOMOD,
            repeat: false,
            raw: 0,
        });
        let _ = app.handle_keyboard_event(&sdl3::event::Event::KeyDown {
            timestamp: 0,
            window_id: 0,
            which: 0,
            scancode: None,
            keycode: Some(sdl3::keyboard::Keycode::R),
            keymod: sdl3::keyboard::Mod::NOMOD,
            repeat: false,
            raw: 0,
        });
        let _ = app.handle_keyboard_event(&sdl3::event::Event::KeyDown {
            timestamp: 0,
            window_id: 0,
            which: 0,
            scancode: None,
            keycode: Some(sdl3::keyboard::Keycode::M),
            keymod: sdl3::keyboard::Mod::NOMOD,
            repeat: false,
            raw: 0,
        });
        let _ = app.handle_keyboard_event(&sdl3::event::Event::KeyDown {
            timestamp: 0,
            window_id: 0,
            which: 0,
            scancode: None,
            keycode: Some(sdl3::keyboard::Keycode::Return),
            keymod: sdl3::keyboard::Mod::NOMOD,
            repeat: false,
            raw: 0,
        });

        assert_eq!(app.mappings[0].target_label, "Track Arm");
        assert_eq!(app.mappings[0].scope_label, "Active Track");
        assert!(app.target_lookup_state.active.is_none());
    }

    #[test]
    fn mappings_target_lookup_resets_invalid_scope_and_escape_cancels() {
        let mut app = App::new();
        app.apply_action(AppAction::ShowPage(AppPage::Mappings));
        app.apply_action(AppAction::ToggleMappingsWriteMode);
        app.page_state.selected_mapping_field = MappingField::Target;
        app.mappings[0].target_label = "Track Arm".to_string();
        app.mappings[0].scope_label = "Track 3".to_string();

        app.apply_action(AppAction::ActivatePageItem);
        let _ = app.handle_keyboard_event(&sdl3::event::Event::KeyDown {
            timestamp: 0,
            window_id: 0,
            which: 0,
            scancode: None,
            keycode: Some(sdl3::keyboard::Keycode::P),
            keymod: sdl3::keyboard::Mod::NOMOD,
            repeat: false,
            raw: 0,
        });
        let _ = app.handle_keyboard_event(&sdl3::event::Event::KeyDown {
            timestamp: 0,
            window_id: 0,
            which: 0,
            scancode: None,
            keycode: Some(sdl3::keyboard::Keycode::L),
            keymod: sdl3::keyboard::Mod::NOMOD,
            repeat: false,
            raw: 0,
        });
        let _ = app.handle_keyboard_event(&sdl3::event::Event::KeyDown {
            timestamp: 0,
            window_id: 0,
            which: 0,
            scancode: None,
            keycode: Some(sdl3::keyboard::Keycode::A),
            keymod: sdl3::keyboard::Mod::NOMOD,
            repeat: false,
            raw: 0,
        });
        let _ = app.handle_keyboard_event(&sdl3::event::Event::KeyDown {
            timestamp: 0,
            window_id: 0,
            which: 0,
            scancode: None,
            keycode: Some(sdl3::keyboard::Keycode::Y),
            keymod: sdl3::keyboard::Mod::NOMOD,
            repeat: false,
            raw: 0,
        });
        let _ = app.handle_keyboard_event(&sdl3::event::Event::KeyDown {
            timestamp: 0,
            window_id: 0,
            which: 0,
            scancode: None,
            keycode: Some(sdl3::keyboard::Keycode::Return),
            keymod: sdl3::keyboard::Mod::NOMOD,
            repeat: false,
            raw: 0,
        });

        assert_eq!(app.mappings[0].target_label, "Play/Stop");
        assert_eq!(app.mappings[0].scope_label, "Global");

        app.mappings[0].target_label = "Track Arm".to_string();
        app.mappings[0].scope_label = "Track 3".to_string();
        app.apply_action(AppAction::ActivatePageItem);
        let _ = app.handle_keyboard_event(&sdl3::event::Event::KeyDown {
            timestamp: 0,
            window_id: 0,
            which: 0,
            scancode: None,
            keycode: Some(sdl3::keyboard::Keycode::Escape),
            keymod: sdl3::keyboard::Mod::NOMOD,
            repeat: false,
            raw: 0,
        });

        assert_eq!(app.mappings[0].target_label, "Track Arm");
        assert_eq!(app.mappings[0].scope_label, "Track 3");
        assert!(app.target_lookup_state.active.is_none());
        assert_eq!(
            app.status_state
                .last_action
                .as_ref()
                .map(|status| status.action),
            Some(AppAction::CancelCurrentMode)
        );
    }

    #[test]
    fn mappings_page_scope_cycles_into_absolute_track_targets() {
        let mut app = App::new();
        app.apply_action(AppAction::ShowPage(AppPage::Mappings));
        app.apply_action(AppAction::ToggleMappingsWriteMode);
        app.page_state.selected_mapping_index = 0;
        app.page_state.selected_mapping_field = MappingField::Target;

        app.mappings[0].target_label = "Track Arm".to_string();
        app.mappings[0].scope_label = "Active Track".to_string();
        app.apply_action(AppAction::SelectNextPageField);
        app.apply_action(AppAction::AdjustPageItemForward);
        assert_eq!(app.mappings[0].scope_label, "Track 1");

        app.apply_action(AppAction::AdjustPageItemBackward);
        assert_eq!(app.mappings[0].scope_label, "Active Track");
    }

    #[test]
    fn mappings_page_skips_device_field_for_non_midi_rows() {
        let mut app = App::new();
        app.apply_action(AppAction::ShowPage(AppPage::Mappings));
        app.apply_action(AppAction::ToggleMappingsWriteMode);
        app.mappings[0].source_kind = MappingSourceKind::Key;
        app.page_state.selected_mapping_field = MappingField::SourceKind;

        app.apply_action(AppAction::SelectNextPageField);

        assert_eq!(
            app.page_state.selected_mapping_field,
            MappingField::SourceValue
        );
    }

    #[test]
    fn switching_away_from_midi_disables_device_field() {
        let mut app = App::new();
        app.apply_action(AppAction::ShowPage(AppPage::Mappings));
        app.apply_action(AppAction::ToggleMappingsWriteMode);
        app.mappings[0].source_kind = MappingSourceKind::Midi;
        app.mappings[0].source_device_label = "Port A".to_string();
        app.page_state.selected_mapping_field = MappingField::SourceDevice;

        app.page_state.selected_mapping_field = MappingField::SourceKind;
        app.apply_action(AppAction::ActivatePageItem);

        assert_ne!(app.mappings[0].source_kind, MappingSourceKind::Midi);
        assert_eq!(
            app.mappings[0].source_device_label,
            default_mapping_source_device()
        );
        assert_ne!(
            app.page_state.selected_mapping_field,
            MappingField::SourceDevice
        );
    }

    #[test]
    fn mapping_row_cells_match_field_order_for_device_and_source() {
        let app = App::new();
        let cells = app.mapping_row_cells(Rect::new(0, 0, 400, 18));

        assert!(
            cells[mapping_field_index(MappingField::SourceDevice)].x
                < cells[mapping_field_index(MappingField::SourceValue)].x
        );
    }

    #[test]
    fn midi_learn_updates_selected_mapping_source() {
        let mut app = App::new();
        app.apply_action(AppAction::ShowPage(AppPage::Mappings));
        app.apply_action(AppAction::ToggleMappingsWriteMode);
        app.page_state.selected_mapping_field = MappingField::SourceValue;
        app.mappings[0].source_kind = MappingSourceKind::Midi;
        app.apply_action(AppAction::ActivatePageItem);
        assert!(app.page_state.mapping_midi_learn_armed);

        app.handle_midi_input_event(MidiInputEvent {
            port: MidiPortRef::new("In A"),
            channel: 3,
            message: MidiInputMessage::ControlChange {
                controller: 24,
                value: 127,
            },
        });

        assert_eq!(app.mappings[0].source_label, "CC24 Ch3");
        assert_eq!(app.mappings[0].source_device_label, "In A");
        assert!(!app.page_state.mapping_midi_learn_armed);
    }

    #[test]
    fn mappings_page_syncs_all_inputs_for_midi_learn() {
        let mut app = App::new();
        app.midi_devices.inputs = vec![MidiPortRef::new("In A"), MidiPortRef::new("In B")];
        for track in &mut app.project.tracks {
            track.routing.input_port = None;
        }

        app.apply_action(AppAction::ShowPage(AppPage::Mappings));
        app.apply_action(AppAction::ToggleMappingsWriteMode);
        app.mappings[0].source_kind = MappingSourceKind::Midi;
        app.page_state.selected_mapping_field = MappingField::SourceValue;
        app.apply_action(AppAction::ActivatePageItem);

        let connected = app.midi_input.requested_port_names();
        assert!(app.page_state.mapping_midi_learn_armed);
        assert_eq!(connected, vec!["In A".to_string(), "In B".to_string()]);
    }

    #[test]
    fn midi_mapping_triggers_action_for_matching_device() {
        let mut app = App::new();
        app.project.select_track(1);
        app.project.tracks[1].state.armed = false;
        app.mappings = vec![MappingEntry {
            source_kind: MappingSourceKind::Midi,
            source_device_label: "Port A".to_string(),
            source_label: "CC20".to_string(),
            target_label: "Track Arm".to_string(),
            scope_label: "Active Track".to_string(),
            enabled: true,
        }];

        app.handle_midi_input_event(MidiInputEvent {
            port: MidiPortRef::new("Port A"),
            channel: 1,
            message: MidiInputMessage::ControlChange {
                controller: 20,
                value: 127,
            },
        });

        assert!(app.project.tracks[1].state.armed);
    }

    #[test]
    fn midi_mapping_ignores_non_matching_device() {
        let mut app = App::new();
        app.project.select_track(1);
        app.project.tracks[1].state.armed = false;
        app.mappings = vec![MappingEntry {
            source_kind: MappingSourceKind::Midi,
            source_device_label: "Port A".to_string(),
            source_label: "CC20".to_string(),
            target_label: "Track Arm".to_string(),
            scope_label: "Active Track".to_string(),
            enabled: true,
        }];

        app.handle_midi_input_event(MidiInputEvent {
            port: MidiPortRef::new("Port B"),
            channel: 1,
            message: MidiInputMessage::ControlChange {
                controller: 20,
                value: 127,
            },
        });

        assert!(!app.project.tracks[1].state.armed);
    }

    #[test]
    fn midi_mapping_can_target_absolute_track_scope() {
        let mut app = App::new();
        app.project.select_track(0);
        app.project.tracks[2].state.armed = false;
        app.mappings = vec![MappingEntry {
            source_kind: MappingSourceKind::Midi,
            source_device_label: "Any MIDI".to_string(),
            source_label: "CC20".to_string(),
            target_label: "Track Arm".to_string(),
            scope_label: "Track 3".to_string(),
            enabled: true,
        }];

        app.handle_midi_input_event(MidiInputEvent {
            port: MidiPortRef::new("Port A"),
            channel: 1,
            message: MidiInputMessage::ControlChange {
                controller: 20,
                value: 127,
            },
        });

        assert_eq!(app.project.active_track_index, 2);
        assert!(app.project.tracks[2].state.armed);
    }
}
