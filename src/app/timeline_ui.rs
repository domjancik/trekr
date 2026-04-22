use super::*;

impl App {
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

    pub(super) fn draw_track_subcolumn<T: RenderTarget>(
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
