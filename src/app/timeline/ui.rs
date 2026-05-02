use super::*;

impl App {
    pub(crate) fn handle_timeline_pointer(
        &mut self,
        content_bounds: Rect,
        x: i32,
        y: i32,
        source: crate::actions::ActionSource,
    ) -> Option<AppControl> {
        let (header_bounds, transport_bounds, timeline_bounds) =
            self.timeline_page_layout(content_bounds).ok()?;
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
            for indicator in crate::ui::track_indicators(layout.status_rect, self.ui_metrics()) {
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

            if self.clip_align_track_has_available_clip(&self.project.tracks[index]) {
                let (align_rect, mute_rect, delete_rect) =
                    self.recording_clip_control_rects(full_label_rect);
                if rect_contains(align_rect, x, y) {
                    self.project.active_track_index = index;
                    return Some(self.apply_action_with_source(
                        AppAction::OpenSelectedRecordingClipAlign,
                        source,
                    ));
                }
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
        let (header_bounds, transport_bounds, timeline_bounds) = self
            .timeline_page_layout(content_bounds)
            .expect("timeline layout");
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

    pub(crate) fn track_discoverability_targets(
        &self,
        layout: TimelineTrackLayout,
        track: &Track,
    ) -> Vec<(Rect, DiscoverabilityTarget)> {
        let mut targets = Vec::new();
        let status_rect = layout.status_rect;
        let label_rect = layout.full_label_rect;
        let detail_label_rect = layout.detail_label_rect;
        if track.recording_view == RecordingView::Stacked {
            let (left_rect, right_rect) = self.recording_view_scroll_control_rects(label_rect);
            targets.push((
                left_rect,
                DiscoverabilityTarget {
                    action: AppAction::SelectPreviousRecordingClip,
                    display_scope: Some("Active Track"),
                    allowed_mapping_scopes: &["Active Track"],
                    overlay_slot: None,
                },
            ));
            targets.push((
                right_rect,
                DiscoverabilityTarget {
                    action: AppAction::SelectNextRecordingClip,
                    display_scope: Some("Active Track"),
                    allowed_mapping_scopes: &["Active Track"],
                    overlay_slot: None,
                },
            ));
        }
        targets.push((
            self.recording_view_chip_rect(label_rect),
            DiscoverabilityTarget {
                action: AppAction::ToggleCurrentTrackRecordingView,
                display_scope: Some("Active Track"),
                allowed_mapping_scopes: &["Active Track"],
                overlay_slot: None,
            },
        ));
        if self.clip_align_track_has_available_clip(track) {
            let (align_rect, mute_rect, delete_rect) =
                self.recording_clip_control_rects(label_rect);
            targets.push((
                align_rect,
                DiscoverabilityTarget {
                    action: AppAction::OpenSelectedRecordingClipAlign,
                    display_scope: Some("Active Track"),
                    allowed_mapping_scopes: &["Active Track"],
                    overlay_slot: None,
                },
            ));
            targets.push((
                mute_rect,
                DiscoverabilityTarget {
                    action: AppAction::ToggleSelectedRecordingClipMute,
                    display_scope: Some("Active Track"),
                    allowed_mapping_scopes: &["Active Track"],
                    overlay_slot: None,
                },
            ));
            targets.push((
                delete_rect,
                DiscoverabilityTarget {
                    action: AppAction::DeleteSelectedRecordingClip,
                    display_scope: Some("Active Track"),
                    allowed_mapping_scopes: &["Active Track"],
                    overlay_slot: None,
                },
            ));
        }
        for content_rect in [layout.full_content_rect, layout.detail_content_rect] {
            for lane in self.recording_lane_layouts(content_rect, track) {
                if let Some(clip_id) = lane.clip_id {
                    targets.push((
                        lane.rect,
                        DiscoverabilityTarget {
                            action: AppAction::SelectRecordingClip(clip_id),
                            display_scope: Some("Active Track"),
                            allowed_mapping_scopes: &["Active Track"],
                            overlay_slot: None,
                        },
                    ));
                }
            }
        }
        for indicator in crate::ui::track_indicators(status_rect, self.ui_metrics()) {
            if let Some(target) = track_indicator_target(indicator.kind, Some(indicator.rect)) {
                targets.push((
                    Rect::new(
                        indicator.rect.x - 2,
                        indicator.rect.y - 2,
                        indicator.rect.width().saturating_add(4),
                        indicator.rect.height().saturating_add(4),
                    ),
                    target,
                ));
            }
        }

        targets.push((
            self.track_passthrough_button_rect(label_rect),
            DiscoverabilityTarget {
                action: AppAction::ToggleCurrentTrackPassthrough,
                display_scope: Some("Active Track"),
                allowed_mapping_scopes: &["Active Track"],
                overlay_slot: None,
            },
        ));
        targets.extend(self.timeline_fx_discoverability_targets_for_track(
            track,
            TimelineContext::OutputFx,
            layout.output_fx_rect,
        ));
        targets.extend(self.timeline_fx_discoverability_targets_for_track(
            track,
            TimelineContext::InputFx,
            layout.input_fx_rect,
        ));
        for (slot_index, slot_rect) in self.stored_loop_slot_rects(detail_label_rect) {
            if let Some(action) = stored_loop_slot_recall_action(slot_index) {
                targets.push((
                    slot_rect,
                    DiscoverabilityTarget {
                        action,
                        display_scope: Some("Active Track"),
                        allowed_mapping_scopes: &["Active Track"],
                        overlay_slot: Some(slot_rect),
                    },
                ));
            }
        }
        targets.push((
            crate::ui::detail_badge_rect(detail_label_rect, self.ui_metrics()),
            DiscoverabilityTarget {
                action: AppAction::ToggleCurrentTrackLoop,
                display_scope: Some("Active Track"),
                allowed_mapping_scopes: &["Active Track"],
                overlay_slot: None,
            },
        ));

        targets
    }

    pub(crate) fn visible_timeline_track_layouts(
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::actions::ActionSource;

    #[test]
    fn timeline_track_arm_indicator_is_clickable() {
        let mut app = App::new();
        let content_bounds = Rect::new(40, 40, 1200, 620);
        let (_, _, timeline_bounds) = app
            .timeline_page_layout(content_bounds)
            .expect("timeline content");
        let columns = crate::ui::track_column_pairs(
            timeline_bounds,
            app.project.tracks.len(),
            app.ui_metrics(),
        );
        let (full_bounds, detail_bounds) = columns[1];
        let status_rect = crate::ui::track_status_rect(
            crate::ui::union_rect(full_bounds, detail_bounds),
            app.timeline_flow,
            app.ui_metrics(),
        );
        let arm_rect = crate::ui::track_indicators(status_rect, app.ui_metrics())[0].rect;

        let control = app.handle_timeline_pointer(
            content_bounds,
            arm_rect.x + arm_rect.width() as i32 / 2,
            arm_rect.y + arm_rect.height() as i32 / 2,
            ActionSource::Pointer,
        );

        assert_eq!(control, Some(AppControl::Continue));
        assert_eq!(app.project.active_track_index, 1);
        assert!(app.project.tracks[1].state.armed);
    }

    #[test]
    fn timeline_track_record_indicator_starts_recording_for_clicked_track() {
        let mut app = App::new();
        let content_bounds = Rect::new(40, 40, 1200, 620);
        let (_, _, timeline_bounds) = app
            .timeline_page_layout(content_bounds)
            .expect("timeline content");
        let columns = crate::ui::track_column_pairs(
            timeline_bounds,
            app.project.tracks.len(),
            app.ui_metrics(),
        );
        let (full_bounds, detail_bounds) = columns[2];
        let status_rect = crate::ui::track_status_rect(
            crate::ui::union_rect(full_bounds, detail_bounds),
            app.timeline_flow,
            app.ui_metrics(),
        );
        let record_rect = crate::ui::track_indicators(status_rect, app.ui_metrics())[1].rect;

        let control = app.handle_timeline_pointer(
            content_bounds,
            record_rect.x + record_rect.width() as i32 / 2,
            record_rect.y + record_rect.height() as i32 / 2,
            ActionSource::Pointer,
        );

        assert_eq!(control, Some(AppControl::Continue));
        assert_eq!(app.project.active_track_index, 2);
        assert!(app.project.transport.recording);
        assert!(app.project.transport.playing);
    }
}
