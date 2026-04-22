use crate::{
    present::window_present_plan,
    theme::{app_chrome, mappings as mappings_theme},
};

use super::*;

impl App {
    pub(super) fn draw_frame_surface(
        &self,
        pixel_format: PixelFormat,
    ) -> Result<sdl3::surface::Surface<'static>, Box<dyn std::error::Error>> {
        let width = self.viewport_size.0.max(1);
        let height = self.viewport_size.1.max(1);
        let surface = sdl3::surface::Surface::new(width, height, pixel_format)?;
        let mut canvas = surface.into_canvas()?;
        self.draw(&mut canvas)?;
        Ok(canvas.into_surface())
    }

    pub(super) fn draw<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.draw_scene(canvas)?;
        canvas.present();
        Ok(())
    }

    pub(super) fn page_frame_layout(
        &self,
        inset: Rect,
    ) -> Result<(Rect, Rect, Rect), Box<dyn std::error::Error>> {
        let (tabs_bounds, page_area_bounds) = crate::ui::split_top_strip(inset, 28, 12)?;
        let footer_height = 22_u32;
        let footer_gap = 8_i32;
        let footer_bounds = Rect::new(
            page_area_bounds.x,
            page_area_bounds.y + page_area_bounds.height() as i32 - footer_height as i32,
            page_area_bounds.width(),
            footer_height,
        );
        let content_bounds = Rect::new(
            page_area_bounds.x,
            page_area_bounds.y,
            page_area_bounds.width(),
            page_area_bounds
                .height()
                .saturating_sub(footer_height)
                .saturating_sub(footer_gap as u32),
        );
        Ok((tabs_bounds, content_bounds, footer_bounds))
    }

    fn draw_scene<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (width, height) = active_draw_size(canvas.output_size()?, self.viewport_size);
        let surface = crate::ui::surface_rect(width, height);
        let inset = crate::ui::inset_rect(surface, 24, 24)?;
        let (tabs_bounds, content_bounds, footer_bounds) = self.page_frame_layout(inset)?;

        canvas.set_draw_color(app_chrome::WINDOW_CLEAR);
        canvas.clear();

        canvas.set_draw_color(app_chrome::SURFACE_FILL);
        canvas.fill_rect(surface)?;
        canvas.set_draw_color(app_chrome::SURFACE_BORDER);
        canvas.draw_rect(surface)?;

        if preferred_branding_width(tabs_bounds.width()) == 0 {
            self.draw_frame_brand_fallback(canvas, surface)?;
        }
        self.draw_page_tabs(canvas, tabs_bounds)?;

        render_page(self.page_state.current_page, self, canvas, content_bounds)?;

        self.draw_direct_mapping_targets(canvas, tabs_bounds, content_bounds)?;
        self.draw_overlay(canvas, inset)?;
        self.draw_footer(canvas, footer_bounds)?;
        Ok(())
    }

    fn draw_frame_brand_fallback<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        surface: Rect,
    ) -> Result<(), Box<dyn std::error::Error>> {
        branding::draw_frame_brand_fallback(canvas, surface)
    }

    pub(super) fn configure_window_canvas(
        &mut self,
        canvas: &mut Canvas<sdl3::video::Window>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let scale = effective_ui_scale(canvas.window().display_scale(), self.ui_scale_override);
        let output_size = canvas.output_size()?;
        self.viewport_size = logical_viewport_size(output_size, scale);
        canvas.set_scale(scale, scale)?;
        Ok(())
    }

    pub(super) fn draw_window(
        &self,
        canvas: &mut Canvas<sdl3::video::Window>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (scale_x, scale_y) = canvas.scale();
        if !should_interpolate_window_scale(self.ui_scaling_mode, scale_x, scale_y) {
            return self.draw(canvas);
        }

        let output_size = canvas.output_size()?;
        let present_plan = window_present_plan(output_size, true, app_chrome::WINDOW_CLEAR);
        let logical_size = self.viewport_size;
        let texture_creator = canvas.texture_creator();
        let mut frame = texture_creator.create_texture_target(
            Some(texture_creator.default_pixel_format()),
            logical_size.0.max(1),
            logical_size.1.max(1),
        )?;
        frame.set_scale_mode(sdl3::render::ScaleMode::Linear);

        let mut draw_result: Result<(), Box<dyn std::error::Error>> = Ok(());
        canvas.with_texture_canvas(&mut frame, |texture_canvas| {
            draw_result = (|| -> Result<(), Box<dyn std::error::Error>> {
                texture_canvas.set_scale(1.0, 1.0)?;
                self.draw_scene(texture_canvas)
            })();
        })?;
        draw_result?;

        canvas.set_scale(1.0, 1.0)?;
        canvas.set_draw_color(present_plan.clear_color);
        canvas.clear();
        canvas.copy(&frame, None, present_plan.destination)?;
        canvas.present();
        canvas.set_scale(scale_x, scale_y)?;
        Ok(())
    }

    fn draw_overlay<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        bounds: Rect,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match self.overlay_state.active {
            Some(AppOverlay::MappingsQuickView) => self.draw_mappings_overlay(canvas, bounds),
            Some(AppOverlay::Discoverability) | None => Ok(()),
        }
    }

    fn draw_page_tabs<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        bounds: Rect,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (branding_bounds, tabs_bounds) = page_tabs_layout(bounds);
        self.draw_branding(canvas, branding_bounds)?;

        let tabs = crate::ui::equal_columns(tabs_bounds, AppPage::ALL.len(), 10);
        for (index, page) in AppPage::ALL.iter().copied().enumerate() {
            let tab = tabs[index];
            let active = page == self.page_state.current_page;
            canvas.set_draw_color(if active {
                app_chrome::TAB_ACTIVE_FILL
            } else {
                app_chrome::TAB_INACTIVE_FILL
            });
            canvas.fill_rect(tab)?;
            canvas.set_draw_color(if active {
                app_chrome::TAB_ACTIVE_BORDER
            } else {
                app_chrome::TAB_INACTIVE_BORDER
            });
            canvas.draw_rect(tab)?;

            let accent = Rect::new(tab.x + 6, tab.y + 6, 18, tab.height().saturating_sub(12));
            let color = match page {
                AppPage::Timeline => Color::RGB(84, 144, 220),
                AppPage::Mappings => Color::RGB(212, 168, 84),
                AppPage::MidiIo => Color::RGB(96, 200, 164),
                AppPage::Routing => Color::RGB(224, 112, 112),
            };
            canvas.set_draw_color(color);
            canvas.fill_rect(accent)?;
            crate::ui::draw_text_fitted(
                canvas,
                page.label(),
                Rect::new(tab.x + 30, tab.y + 8, tab.width().saturating_sub(36), 8),
                1,
                if active {
                    app_chrome::TAB_TEXT_ACTIVE
                } else {
                    app_chrome::TAB_TEXT_INACTIVE
                },
            )?;
        }

        Ok(())
    }

    fn draw_branding<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        bounds: Rect,
    ) -> Result<(), Box<dyn std::error::Error>> {
        branding::draw_branding(canvas, bounds, self.startup_started_at.elapsed())
    }

    pub(super) fn draw_transport_chip<T: RenderTarget>(
        canvas: &mut Canvas<T>,
        chip: Rect,
        spec: &TransportChipSpec,
    ) -> Result<(), Box<dyn std::error::Error>> {
        canvas.set_draw_color(spec.fill);
        canvas.fill_rect(chip)?;
        crate::ui::draw_text_fitted(
            canvas,
            &spec.label,
            Rect::new(chip.x + 5, chip.y + 2, chip.width().saturating_sub(10), 8),
            1,
            app_chrome::ACTION_TEXT,
        )?;
        Ok(())
    }

    fn draw_footer<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        bounds: Rect,
    ) -> Result<(), Box<dyn std::error::Error>> {
        canvas.set_draw_color(app_chrome::FOOTER_BG);
        canvas.fill_rect(bounds)?;
        canvas.set_draw_color(app_chrome::SURFACE_BORDER);
        canvas.draw_rect(bounds)?;

        let overlay_chips = [
            (
                "F5 Mappings",
                self.overlay_state.active == Some(AppOverlay::MappingsQuickView),
                Color::RGB(156, 122, 68),
            ),
            (
                "F7 Discover",
                self.overlay_state.active == Some(AppOverlay::Discoverability),
                Color::RGB(72, 136, 166),
            ),
            (
                "F8 Direct",
                self.direct_mapping_state.mode != DirectMappingMode::Inactive,
                Color::RGB(188, 82, 82),
            ),
        ];
        let mut right_edge = bounds.x + bounds.width() as i32 - 6;
        for (label, active, color) in overlay_chips.into_iter().rev() {
            let width = crate::ui::text_width(label, 1) + 10;
            let chip = Rect::new(
                right_edge - width as i32,
                bounds.y + 5,
                width,
                bounds.height().saturating_sub(10),
            );
            canvas.set_draw_color(if active {
                color
            } else {
                app_chrome::FOOTER_CHIP_INACTIVE
            });
            canvas.fill_rect(chip)?;
            crate::ui::draw_text_fitted(
                canvas,
                label,
                Rect::new(chip.x + 5, chip.y + 2, chip.width().saturating_sub(10), 8),
                1,
                if active {
                    app_chrome::FOOTER_TEXT_ACTIVE
                } else {
                    app_chrome::FOOTER_TEXT_INACTIVE
                },
            )?;
            right_edge = chip.x - 6;
        }

        if let Some((title, detail, badges)) = self.direct_mapping_footer_content() {
            let label_width = crate::ui::text_width(&title, 1) + 4;
            let label_rect = Rect::new(bounds.x + 8, bounds.y + 7, label_width, 8);
            crate::ui::draw_text_fitted(canvas, &title, label_rect, 1, Color::RGB(248, 228, 208))?;
            let detail_left = label_rect.x + label_rect.width() as i32 + 8;
            let detail_width = (right_edge - detail_left).max(0) as u32;
            if !badges.is_empty() {
                self.draw_mapping_badges(
                    canvas,
                    Rect::new(
                        detail_left,
                        bounds.y + 3,
                        detail_width,
                        bounds.height().saturating_sub(6),
                    ),
                    &badges,
                    badges.len(),
                    4,
                    10,
                )?;
            } else {
                crate::ui::draw_text_fitted(
                    canvas,
                    &detail,
                    Rect::new(detail_left, bounds.y + 7, detail_width, 8),
                    1,
                    Color::RGB(214, 200, 188),
                )?;
            }
        } else if let Some(target) = self.status_state.hovered_target {
            let summary = self.summarize_discoverability_target(target);
            let label_width = crate::ui::text_width(&summary.title, 1) + 4;
            let label_rect = Rect::new(bounds.x + 8, bounds.y + 7, label_width, 8);
            crate::ui::draw_text_fitted(
                canvas,
                &summary.title,
                label_rect,
                1,
                app_chrome::ACTION_TEXT,
            )?;
            let badges_left = label_rect.x + label_rect.width() as i32 + 8;
            let badges_width = (right_edge - badges_left).max(0) as u32;
            if summary.badges.is_empty() {
                crate::ui::draw_text_fitted(
                    canvas,
                    "No mappings",
                    Rect::new(badges_left, bounds.y + 7, badges_width, 8),
                    1,
                    Color::RGB(168, 178, 194),
                )?;
            } else {
                self.draw_mapping_badges(
                    canvas,
                    Rect::new(
                        badges_left,
                        bounds.y + 3,
                        badges_width,
                        bounds.height().saturating_sub(6),
                    ),
                    &summary.badges,
                    summary.total_bindings,
                    4,
                    10,
                )?;
            }
        } else if let Some((title, detail)) = self.timeline_fx_footer_content() {
            let label_width = crate::ui::text_width(&title, 1) + 4;
            let label_rect = Rect::new(bounds.x + 8, bounds.y + 7, label_width, 8);
            crate::ui::draw_text_fitted(canvas, &title, label_rect, 1, mappings_theme::PAGE_TITLE)?;
            crate::ui::draw_text_fitted(
                canvas,
                &detail,
                Rect::new(
                    label_rect.x + label_rect.width() as i32 + 8,
                    bounds.y + 7,
                    (right_edge - label_rect.x - label_rect.width() as i32 - 12).max(0) as u32,
                    8,
                ),
                1,
                app_chrome::DETAIL_TEXT,
            )?;
        } else {
            let last_action = self
                .status_state
                .last_action
                .map(|status| {
                    format!(
                        "Last Action: {} via {}",
                        action_label(status.action),
                        action_source_label(status.source)
                    )
                })
                .unwrap_or_else(|| "Last Action: Ready".to_string());
            crate::ui::draw_text_fitted(
                canvas,
                &last_action,
                Rect::new(
                    bounds.x + 8,
                    bounds.y + 7,
                    (right_edge - bounds.x - 12).max(0) as u32,
                    8,
                ),
                1,
                app_chrome::DETAIL_TEXT,
            )?;
        }

        Ok(())
    }

    pub(super) fn mapping_row_cells(&self, row: Rect) -> [Rect; 6] {
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
        canvas.set_draw_color(mappings_theme::PAGE_BG);
        canvas.fill_rect(content_bounds)?;
        canvas.set_draw_color(app_chrome::SURFACE_BORDER);
        canvas.draw_rect(content_bounds)?;
        crate::ui::draw_text_fitted(
            canvas,
            "Mappings",
            Rect::new(content_bounds.x + 8, content_bounds.y + 8, 180, 14),
            2,
            mappings_theme::PAGE_TITLE,
        )?;
        let overview_badge = Rect::new(content_bounds.x + 200, content_bounds.y + 8, 188, 16);
        canvas.set_draw_color(if self.page_state.mapping_mode == MappingPageMode::Write {
            mappings_theme::WRITE_MODE_ACTIVE
        } else {
            mappings_theme::WRITE_MODE_INACTIVE
        });
        canvas.fill_rect(overview_badge)?;
        canvas.set_draw_color(mappings_theme::PAGE_TITLE);
        canvas.draw_rect(overview_badge)?;
        crate::ui::draw_text_fitted(
            canvas,
            &format!("Tap Mode: {}", self.page_state.mapping_mode.label()),
            Rect::new(content_bounds.x + 208, content_bounds.y + 12, 170, 8),
            1,
            Color::RGB(236, 242, 248),
        )?;
        let learn_badge = Rect::new(content_bounds.x + 392, content_bounds.y + 8, 136, 16);
        canvas.set_draw_color(if self.page_state.mapping_midi_learn_armed {
            mappings_theme::LEARN_ARMED
        } else {
            mappings_theme::LEARN_IDLE
        });
        canvas.fill_rect(learn_badge)?;
        canvas.set_draw_color(
            if self.page_state.selected_mapping_field == MappingField::SourceValue
                && self.page_state.mapping_mode == MappingPageMode::Write
            {
                Color::RGB(252, 232, 146)
            } else {
                Color::RGB(96, 108, 132)
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
            Color::RGB(236, 240, 246),
        )?;
        let direct_badge = Rect::new(content_bounds.x + 532, content_bounds.y + 8, 154, 16);
        canvas.set_draw_color(
            if self.direct_mapping_state.mode == DirectMappingMode::Inactive {
                Color::RGB(54, 62, 82)
            } else {
                mappings_theme::DIRECT_ARMED_FILL
            },
        );
        canvas.fill_rect(direct_badge)?;
        canvas.set_draw_color(
            if self.direct_mapping_state.mode == DirectMappingMode::Inactive {
                mappings_theme::DIRECT_IDLE_BORDER
            } else {
                mappings_theme::DIRECT_ARMED_BORDER
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
            Color::RGB(242, 238, 234),
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
            Color::RGB(154, 166, 182),
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
                mappings_theme::ROW_SELECTED_FILL
            } else {
                mappings_theme::ROW_IDLE_FILL
            });
            canvas.fill_rect(row)?;
            canvas.set_draw_color(if selected {
                mappings_theme::PAGE_TITLE
            } else {
                mappings_theme::ROW_IDLE_BORDER
            });
            canvas.draw_rect(row)?;

            let cells = self.mapping_row_cells(row);
            let source_rect = Rect::new(cells[0].x, cells[0].y, 14, cells[0].height());
            let source_color = match entry.source_kind {
                MappingSourceKind::Key => Color::RGB(98, 148, 232),
                MappingSourceKind::Midi => Color::RGB(96, 202, 146),
                MappingSourceKind::Osc => Color::RGB(220, 154, 88),
            };
            canvas.set_draw_color(source_color);
            canvas.fill_rect(source_rect)?;

            let enabled_rect = Rect::new(cells[5].x + 6, cells[5].y, 14, cells[5].height());
            canvas.set_draw_color(if entry.enabled {
                Color::RGB(132, 220, 120)
            } else {
                Color::RGB(92, 96, 102)
            });
            canvas.fill_rect(enabled_rect)?;

            let kind_rect = cells[0];
            let device_rect = cells[1];
            let trigger_rect = cells[2];
            let target_rect = cells[3];
            let scope_rect = cells[4];
            canvas.set_draw_color(if selected {
                Color::RGB(66, 80, 112)
            } else {
                Color::RGB(42, 50, 70)
            });
            canvas.fill_rect(kind_rect)?;
            canvas.fill_rect(trigger_rect)?;
            canvas.fill_rect(device_rect)?;
            canvas.set_draw_color(if entry.enabled {
                Color::RGB(182, 194, 212)
            } else {
                Color::RGB(104, 112, 124)
            });
            canvas.fill_rect(target_rect)?;
            canvas.set_draw_color(Color::RGB(66, 74, 88));
            canvas.fill_rect(scope_rect)?;
            canvas.fill_rect(cells[5])?;
            if selected && self.page_state.mapping_mode == MappingPageMode::Write {
                let field_rect = cells[mapping_field_index(self.page_state.selected_mapping_field)];
                canvas.set_draw_color(
                    if self.page_state.mapping_midi_learn_armed
                        && self.page_state.selected_mapping_field == MappingField::SourceValue
                    {
                        Color::RGB(120, 42, 42)
                    } else {
                        Color::RGB(92, 98, 64)
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
                app_chrome::ACTION_TEXT,
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
                app_chrome::ACTION_TEXT,
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
                    Color::RGB(226, 234, 244)
                } else {
                    Color::RGB(124, 132, 146)
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
                Color::RGB(24, 28, 36),
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
                Color::RGB(236, 238, 242),
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
                Color::RGB(236, 238, 242),
            )?;

            if selected && self.page_state.mapping_mode == MappingPageMode::Write {
                let field_rect = cells[mapping_field_index(self.page_state.selected_mapping_field)];
                canvas.set_draw_color(
                    if self.page_state.mapping_midi_learn_armed
                        && self.page_state.selected_mapping_field == MappingField::SourceValue
                    {
                        Color::RGB(252, 126, 126)
                    } else {
                        Color::RGB(252, 232, 146)
                    },
                );
                canvas.draw_rect(field_rect)?;
                let tap_tag = Rect::new(row.x + row.width() as i32 - 68, row.y + 3, 34, 12);
                canvas.set_draw_color(Color::RGB(86, 98, 124));
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
                    app_chrome::ACTION_TEXT,
                )?;
            }
        }

        self.draw_mapping_target_lookup(canvas, content_bounds)?;

        canvas.set_draw_color(Color::RGB(26, 32, 46));
        canvas.fill_rect(footer_bounds)?;
        let footer_tokens = [
            ("Tap row", Color::RGB(62, 78, 106)),
            ("Tap field", Color::RGB(74, 88, 118)),
            ("Tap again act", Color::RGB(82, 100, 136)),
            ("W Write", Color::RGB(96, 82, 52)),
            ("F8 Direct", Color::RGB(128, 78, 78)),
            ("N New", Color::RGB(66, 96, 84)),
            ("Del/Bsp Remove", Color::RGB(110, 74, 74)),
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
                app_chrome::ACTION_TEXT,
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
            Color::RGB(154, 166, 182),
        )?;

        Ok(())
    }

    fn draw_mappings_overlay<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        bounds: Rect,
    ) -> Result<(), Box<dyn std::error::Error>> {
        canvas.set_draw_color(Color::RGBA(10, 14, 24, 220));
        canvas.fill_rect(bounds)?;

        let panel = Rect::new(
            bounds.x + 84,
            bounds.y + 44,
            bounds.width() - 168,
            bounds.height() - 88,
        );
        canvas.set_draw_color(Color::RGB(24, 30, 44));
        canvas.fill_rect(panel)?;
        canvas.set_draw_color(mappings_theme::PAGE_TITLE);
        canvas.draw_rect(panel)?;
        let title_bounds = Rect::new(panel.x + 12, panel.y + 12, 220, 14);
        crate::ui::draw_text_fitted(
            canvas,
            "Mappings Overlay",
            title_bounds,
            2,
            mappings_theme::PAGE_TITLE,
        )?;
        crate::ui::draw_text_fitted(
            canvas,
            "F5 Close",
            Rect::new(panel.x + 12, panel.y + 32, 58, 8),
            1,
            app_chrome::DETAIL_TEXT,
        )?;
        crate::ui::draw_text_fitted(
            canvas,
            "W Write",
            Rect::new(panel.x + 80, panel.y + 32, 52, 8),
            1,
            app_chrome::DETAIL_TEXT,
        )?;
        crate::ui::draw_text_fitted(
            canvas,
            "Trigger",
            Rect::new(panel.x + 12, panel.y + 46, 56, 8),
            1,
            Color::RGB(150, 162, 180),
        )?;
        crate::ui::draw_text_fitted(
            canvas,
            "Action",
            Rect::new(panel.x + 146, panel.y + 46, 48, 8),
            1,
            Color::RGB(150, 162, 180),
        )?;
        crate::ui::draw_text_fitted(
            canvas,
            "Scope",
            Rect::new(panel.x + panel.width() as i32 - 126, panel.y + 46, 44, 8),
            1,
            Color::RGB(150, 162, 180),
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
                Color::RGB(58, 72, 102)
            } else {
                Color::RGB(34, 42, 60)
            });
            canvas.fill_rect(row)?;
            canvas.set_draw_color(if selected {
                mappings_theme::PAGE_TITLE
            } else {
                Color::RGB(82, 92, 114)
            });
            canvas.draw_rect(row)?;

            crate::ui::draw_text_fitted(
                canvas,
                &entry.source_label,
                Rect::new(row.x + 8, row.y + 5, 126, 8),
                1,
                app_chrome::ACTION_TEXT,
            )?;
            crate::ui::draw_text_fitted(
                canvas,
                &entry.target_label,
                Rect::new(row.x + 146, row.y + 5, 210, 8),
                1,
                Color::RGB(208, 220, 236),
            )?;
            crate::ui::draw_text_fitted(
                canvas,
                compact_scope_label(&entry.scope_label),
                Rect::new(row.x + row.width() as i32 - 126, row.y + 5, 90, 8),
                1,
                Color::RGB(182, 192, 210),
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
            Color::RGB(160, 170, 184),
        )?;

        Ok(())
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
    fn page_frame_layout_matches_draw_content_height_contract() {
        let app = App::new();
        let surface = crate::ui::surface_rect(1280, 720);
        let inset = crate::ui::inset_rect(surface, 24, 24).expect("inset");
        let (_, content_bounds, footer_bounds) = app.page_frame_layout(inset).expect("layout");
        let (_, page_area_bounds) = crate::ui::split_top_strip(inset, 28, 12).expect("page split");

        assert_eq!(content_bounds.y, page_area_bounds.y);
        assert_eq!(
            footer_bounds.y + footer_bounds.height() as i32,
            page_area_bounds.y + page_area_bounds.height() as i32
        );
        assert_eq!(content_bounds.height() + 22 + 8, page_area_bounds.height());
    }
}
