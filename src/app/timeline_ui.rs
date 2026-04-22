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
}
