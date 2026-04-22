use super::*;

pub(super) fn indexed_notes(
    track: &Track,
    recording_clip_id: Option<u64>,
) -> Vec<(usize, crate::project::MidiNote)> {
    track
        .midi_notes
        .iter()
        .copied()
        .enumerate()
        .filter(|(_, note)| match recording_clip_id {
            Some(clip_id) => note.recording_clip_id == Some(clip_id),
            None => note.recording_clip_id.is_none(),
        })
        .collect()
}

pub(super) fn indexed_all_notes(track: &Track) -> Vec<(usize, crate::project::MidiNote)> {
    track.midi_notes.iter().copied().enumerate().collect()
}

impl App {
    pub(super) fn draw_recording_view_controls<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        label_rect: Rect,
        _content_rect: Rect,
        track: &Track,
        clip_controls: Option<(Rect, Rect)>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if track.recording_view == RecordingView::Stacked {
            let can_scroll_left = self.can_select_previous_recording_clip(track);
            let can_scroll_right = self.can_select_next_recording_clip(track);
            let (left_rect, right_rect) = self.recording_view_scroll_control_rects(label_rect);
            canvas.set_draw_color(if can_scroll_left {
                Color::RGB(74, 82, 98)
            } else {
                Color::RGB(48, 54, 68)
            });
            canvas.fill_rect(left_rect)?;
            canvas.set_draw_color(if can_scroll_left {
                Color::RGB(202, 212, 224)
            } else {
                Color::RGB(112, 118, 130)
            });
            canvas.draw_rect(left_rect)?;
            crate::ui::draw_text_fitted(
                canvas,
                "<",
                Rect::new(
                    left_rect.x + 6,
                    left_rect.y + 1,
                    left_rect.width().saturating_sub(12),
                    8,
                ),
                1,
                if can_scroll_left {
                    Color::RGB(244, 244, 236)
                } else {
                    Color::RGB(144, 150, 160)
                },
            )?;
            canvas.set_draw_color(if can_scroll_right {
                Color::RGB(74, 82, 98)
            } else {
                Color::RGB(48, 54, 68)
            });
            canvas.fill_rect(right_rect)?;
            canvas.set_draw_color(if can_scroll_right {
                Color::RGB(202, 212, 224)
            } else {
                Color::RGB(112, 118, 130)
            });
            canvas.draw_rect(right_rect)?;
            crate::ui::draw_text_fitted(
                canvas,
                ">",
                Rect::new(
                    right_rect.x + 6,
                    right_rect.y + 1,
                    right_rect.width().saturating_sub(12),
                    8,
                ),
                1,
                if can_scroll_right {
                    Color::RGB(244, 244, 236)
                } else {
                    Color::RGB(144, 150, 160)
                },
            )?;
        }
        let view_rect = self.recording_view_chip_rect(label_rect);
        canvas.set_draw_color(match track.recording_view {
            RecordingView::Overlay => Color::RGB(50, 84, 126),
            RecordingView::Stacked => Color::RGB(124, 98, 48),
        });
        canvas.fill_rect(view_rect)?;
        canvas.set_draw_color(Color::RGB(232, 228, 208));
        canvas.draw_rect(view_rect)?;
        crate::ui::draw_text_fitted(
            canvas,
            match track.recording_view {
                RecordingView::Overlay => "OVR",
                RecordingView::Stacked => "STK",
            },
            Rect::new(
                view_rect.x + 3,
                view_rect.y + 1,
                view_rect.width().saturating_sub(6),
                view_rect.height().saturating_sub(2),
            ),
            1,
            Color::RGB(248, 244, 236),
        )?;

        if let (Some(selected_clip), Some((mute_rect, delete_rect))) =
            (track.selected_recording_clip(), clip_controls)
        {
            canvas.set_draw_color(if selected_clip.muted {
                Color::RGB(120, 118, 112)
            } else {
                Color::RGB(84, 122, 92)
            });
            canvas.fill_rect(mute_rect)?;
            canvas.set_draw_color(Color::RGB(228, 232, 216));
            canvas.draw_rect(mute_rect)?;
            crate::ui::draw_text_fitted(
                canvas,
                if selected_clip.muted { "ON" } else { "M" },
                Rect::new(
                    mute_rect.x + 2,
                    mute_rect.y + 1,
                    mute_rect.width().saturating_sub(4),
                    mute_rect.height().saturating_sub(2),
                ),
                1,
                Color::RGB(246, 244, 236),
            )?;

            canvas.set_draw_color(Color::RGB(132, 74, 70));
            canvas.fill_rect(delete_rect)?;
            canvas.set_draw_color(Color::RGB(240, 220, 210));
            canvas.draw_rect(delete_rect)?;
            crate::ui::draw_text_fitted(
                canvas,
                "X",
                Rect::new(
                    delete_rect.x + 2,
                    delete_rect.y + 1,
                    delete_rect.width().saturating_sub(4),
                    delete_rect.height().saturating_sub(2),
                ),
                1,
                Color::RGB(250, 242, 236),
            )?;
        }

        Ok(())
    }

    pub(super) fn draw_track_recording_content<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        content_rect: Rect,
        track: &Track,
        note_range: crate::timeline::LoopRegion,
        is_active: bool,
        detail: bool,
        selected_note_indices: &[usize],
        focused_note_index: Option<usize>,
        anchor_note_index: Option<usize>,
        preview_region: Option<crate::timeline::Region>,
        preview_notes: &[crate::project::MidiNote],
    ) -> Result<(), Box<dyn std::error::Error>> {
        if track.recording_view == RecordingView::Stacked
            && (!track.recording_clips().is_empty() || preview_region.is_some())
        {
            let unowned_regions: Vec<_> = track
                .regions
                .iter()
                .copied()
                .filter(|region| region.recording_clip_id.is_none())
                .collect();
            let unowned_notes = indexed_notes(track, None);
            self.draw_region_entries(
                canvas,
                content_rect,
                &unowned_regions,
                note_range,
                track,
                is_active,
                track.state.muted,
            )?;
            self.draw_note_entries(
                canvas,
                content_rect,
                &unowned_notes,
                note_range,
                track,
                detail,
                track.state.muted,
                selected_note_indices,
                focused_note_index,
                anchor_note_index,
            )?;

            for lane in self.recording_lane_layouts(content_rect, track) {
                canvas.set_draw_color(if lane.preview {
                    Color::RGB(54, 32, 36)
                } else if lane.selected {
                    Color::RGB(46, 62, 94)
                } else {
                    Color::RGB(26, 34, 48)
                });
                canvas.fill_rect(lane.rect)?;
                canvas.set_draw_color(if lane.preview {
                    Color::RGB(248, 122, 122)
                } else if lane.selected {
                    Color::RGB(248, 226, 134)
                } else {
                    Color::RGB(76, 92, 118)
                });
                canvas.draw_rect(lane.rect)?;

                if lane.preview {
                    if let Some(preview_region) =
                        preview_region.filter(|region| region.intersects(note_range))
                    {
                        for region in crate::ui::region_rects(
                            lane.rect,
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

                    for note in crate::ui::note_rects(
                        lane.rect,
                        preview_notes,
                        note_range,
                        self.timeline_flow,
                    ) {
                        canvas.set_draw_color(Color::RGBA(238, 108, 108, 176));
                        canvas.fill_rect(note.rect)?;
                        canvas.set_draw_color(Color::RGB(255, 176, 176));
                        canvas.draw_rect(note.rect)?;
                    }
                } else if let Some(clip_id) = lane.clip_id {
                    let lane_muted = track.state.muted || lane.muted;
                    let lane_regions: Vec<_> = track
                        .regions
                        .iter()
                        .copied()
                        .filter(|region| region.recording_clip_id == Some(clip_id))
                        .collect();
                    let lane_notes = indexed_notes(track, Some(clip_id));
                    self.draw_region_entries(
                        canvas,
                        lane.rect,
                        &lane_regions,
                        note_range,
                        track,
                        is_active,
                        lane_muted,
                    )?;
                    self.draw_note_entries(
                        canvas,
                        lane.rect,
                        &lane_notes,
                        note_range,
                        track,
                        detail,
                        lane_muted,
                        selected_note_indices,
                        focused_note_index,
                        anchor_note_index,
                    )?;
                }
            }

            return Ok(());
        }

        self.draw_region_entries(
            canvas,
            content_rect,
            &track.regions,
            note_range,
            track,
            is_active,
            track.state.muted,
        )?;
        self.draw_note_entries(
            canvas,
            content_rect,
            &indexed_all_notes(track),
            note_range,
            track,
            detail,
            track.state.muted,
            selected_note_indices,
            focused_note_index,
            anchor_note_index,
        )?;
        Ok(())
    }

    fn draw_region_entries<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        lane_rect: Rect,
        regions: &[crate::timeline::Region],
        note_range: crate::timeline::LoopRegion,
        track: &Track,
        is_active: bool,
        muted_override: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for source_region in regions.iter().copied() {
            let region_muted =
                muted_override || track.recording_clip_is_muted(source_region.recording_clip_id);
            let Some(region) = crate::ui::region_rects(
                lane_rect,
                &[source_region],
                note_range,
                self.timeline_flow,
            )
            .into_iter()
            .next() else {
                continue;
            };
            canvas.set_draw_color(if region.clipped {
                Color::RGB(108, 88, 56)
            } else if region_muted {
                Color::RGB(42, 46, 56)
            } else {
                Color::RGB(44, 54, 76)
            });
            canvas.fill_rect(region.rect)?;
            canvas.set_draw_color(if is_active {
                Color::RGB(212, 196, 122)
            } else {
                Color::RGB(96, 106, 126)
            });
            canvas.draw_rect(region.rect)?;
        }

        Ok(())
    }

    fn draw_note_entries<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        lane_rect: Rect,
        note_entries: &[(usize, crate::project::MidiNote)],
        note_range: crate::timeline::LoopRegion,
        track: &Track,
        detail: bool,
        muted_override: bool,
        selected_note_indices: &[usize],
        focused_note_index: Option<usize>,
        anchor_note_index: Option<usize>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let notes: Vec<_> = note_entries.iter().map(|(_, note)| *note).collect();
        for note in crate::ui::note_rects(lane_rect, &notes, note_range, self.timeline_flow) {
            let absolute_index = note_entries[note.source_index].0;
            let note_muted = muted_override
                || track
                    .recording_clip_is_muted(note_entries[note.source_index].1.recording_clip_id);
            let selected = selected_note_indices.contains(&absolute_index);
            let focused = focused_note_index == Some(absolute_index);
            let anchored = anchor_note_index == Some(absolute_index);
            canvas.set_draw_color(if selected && detail {
                Color::RGB(112, 174, 228)
            } else if selected {
                Color::RGB(88, 136, 194)
            } else if note_muted {
                Color::RGB(92, 100, 112)
            } else if note.clipped {
                Color::RGB(244, 204, 132)
            } else {
                Color::RGB(210, 222, 236)
            });
            canvas.fill_rect(note.rect)?;
            canvas.set_draw_color(if focused {
                Color::RGB(252, 246, 158)
            } else if anchored {
                Color::RGB(180, 226, 176)
            } else if selected {
                Color::RGB(224, 238, 248)
            } else if note_muted {
                Color::RGB(128, 134, 144)
            } else {
                Color::RGB(245, 247, 250)
            });
            canvas.draw_rect(note.rect)?;
            if focused {
                let inner = Rect::new(
                    note.rect.x + 1,
                    note.rect.y + 1,
                    note.rect.width().saturating_sub(2).max(1),
                    note.rect.height().saturating_sub(2).max(1),
                );
                canvas.set_draw_color(Color::RGB(252, 208, 88));
                canvas.draw_rect(inner)?;
            }
        }

        Ok(())
    }

    pub(super) fn recording_lane_layouts(
        &self,
        content_rect: Rect,
        track: &Track,
    ) -> Vec<RecordingLaneLayout> {
        let gap = 2;
        let window = self.recording_lane_window(track, self.recording_lane_capacity(content_rect));
        let visible_clips = &track.recording_clips()[window.committed_start..window.committed_end];
        let lane_count = window.visible_committed + usize::from(window.show_preview);
        let lane_rects = match self.timeline_flow {
            TimelineFlow::DownwardColumns => {
                crate::ui::equal_columns(content_rect, lane_count, gap)
            }
            TimelineFlow::AcrossRows => crate::ui::stacked_rows(content_rect, lane_count, gap),
        };

        let mut layouts: Vec<_> = visible_clips
            .iter()
            .zip(lane_rects.iter().copied())
            .map(|(clip, rect)| RecordingLaneLayout {
                clip_id: Some(clip.id),
                rect,
                selected: track.selected_recording_clip_id == Some(clip.id),
                muted: clip.muted,
                preview: false,
            })
            .collect();

        if window.show_preview {
            if let Some(rect) = lane_rects.get(window.visible_committed).copied() {
                layouts.push(RecordingLaneLayout {
                    clip_id: None,
                    rect,
                    selected: false,
                    muted: false,
                    preview: true,
                });
            }
        }

        layouts
    }

    pub(super) fn recording_lane_hit_clip(
        &self,
        content_rect: Rect,
        track: &Track,
        x: i32,
        y: i32,
    ) -> Option<u64> {
        self.recording_lane_layouts(content_rect, track)
            .into_iter()
            .find_map(|lane| {
                rect_contains(lane.rect, x, y)
                    .then_some(lane.clip_id)
                    .flatten()
            })
    }

    pub(super) fn recording_lane_capacity(&self, content_rect: Rect) -> usize {
        match self.timeline_flow {
            TimelineFlow::DownwardColumns => {
                let min_lane_width = 15_i32;
                let gap = 2_i32;
                (((content_rect.width() as i32 + gap) / (min_lane_width + gap)).max(1)) as usize
            }
            TimelineFlow::AcrossRows => {
                let min_lane_height = 26_i32;
                let gap = 2_i32;
                (((content_rect.height() as i32 + gap) / (min_lane_height + gap)).max(1)) as usize
            }
        }
    }

    pub(super) fn recording_view_chip_rect(&self, label_rect: Rect) -> Rect {
        let top_y = label_rect.y + label_rect.height() as i32 - 10;
        let right = label_rect.x + label_rect.width() as i32 - 4;
        Rect::new(right - 26, top_y, 26, 8)
    }

    pub(super) fn track_passthrough_button_rect(&self, label_rect: Rect) -> Rect {
        Rect::new(
            label_rect.x + 4,
            label_rect.y + 3,
            label_rect.width().saturating_sub(8).min(30),
            8,
        )
    }

    fn stored_loop_visible_slot_count(&self, label_rect: Rect) -> usize {
        let slot_w = 8_i32;
        let gap = 2_i32;
        let side_padding = 8_i32;
        let min_name_space = 24_i32;
        let available = label_rect.width() as i32 - side_padding - min_name_space;
        if available < slot_w {
            return 0;
        }
        (((available + gap) / (slot_w + gap)).max(0) as usize).min(STORED_LOOP_SLOT_COUNT)
    }

    pub(super) fn stored_loop_slot_rects(&self, label_rect: Rect) -> Vec<(usize, Rect)> {
        let visible_slots = self
            .stored_loop_visible_slot_count(label_rect)
            .min(STORED_LOOP_SLOT_COUNT);
        let slot_w = 8_u32;
        let slot_h = 7_u32;
        let gap = 2_i32;
        let mut rects = Vec::with_capacity(visible_slots);
        for slot_index in 0..visible_slots {
            rects.push((
                slot_index,
                Rect::new(
                    label_rect.x + 4 + slot_index as i32 * (slot_w as i32 + gap),
                    label_rect.y + 2,
                    slot_w,
                    slot_h,
                ),
            ));
        }
        rects
    }

    pub(super) fn recording_view_scroll_control_rects(&self, label_rect: Rect) -> (Rect, Rect) {
        let top_y = label_rect.y + label_rect.height() as i32 - 10;
        let view_rect = self.recording_view_chip_rect(label_rect);
        let right_rect = Rect::new(view_rect.x - 16, top_y, 12, 8);
        let left_rect = Rect::new(right_rect.x - 14, top_y, 12, 8);
        (left_rect, right_rect)
    }

    fn selected_recording_clip_index(&self, track: &Track) -> Option<usize> {
        track.selected_recording_clip_id.and_then(|selected_id| {
            track
                .recording_clips()
                .iter()
                .position(|clip| clip.id == selected_id)
        })
    }

    fn can_select_previous_recording_clip(&self, track: &Track) -> bool {
        self.selected_recording_clip_index(track)
            .map(|index| index > 0)
            .unwrap_or(false)
    }

    fn can_select_next_recording_clip(&self, track: &Track) -> bool {
        self.selected_recording_clip_index(track)
            .map(|index| index + 1 < track.recording_clips().len())
            .unwrap_or(false)
    }

    pub(super) fn sync_active_track_recording_clip_scroll(&mut self) {
        let Some(full_bounds) = self.active_track_full_bounds() else {
            return;
        };
        let content_rect = crate::ui::track_content_rect(full_bounds, self.timeline_flow);
        let total_capacity = self.recording_lane_capacity(content_rect).max(1);
        let Some(track) = self.project.active_track_mut() else {
            return;
        };
        let total_lanes = track.recording_clips.len() + usize::from(track.active_take.is_some());
        if total_lanes == 0 {
            track.recording_clip_scroll = 0;
            return;
        }

        let visible_lanes = total_capacity.min(total_lanes);
        let max_start = total_lanes.saturating_sub(visible_lanes);
        track.recording_clip_scroll = track.recording_clip_scroll.min(max_start);
        if track.active_take.is_some() {
            track.recording_clip_scroll = track.recording_clip_scroll.max(max_start);
        }
        let Some(selected_id) = track.selected_recording_clip_id else {
            return;
        };
        if track.active_take.is_some() {
            return;
        }
        let Some(selected_index) = track
            .recording_clips
            .iter()
            .position(|clip| clip.id == selected_id)
        else {
            return;
        };
        if selected_index < track.recording_clip_scroll {
            track.recording_clip_scroll = selected_index;
        } else if selected_index >= track.recording_clip_scroll + visible_lanes {
            track.recording_clip_scroll = selected_index + 1 - visible_lanes;
        }
    }

    pub(super) fn recording_clip_scroll_control_hit(
        &self,
        label_rect: Rect,
        track: &Track,
        x: i32,
        y: i32,
    ) -> Option<AppAction> {
        if track.recording_view != RecordingView::Stacked {
            return None;
        }
        let (left_rect, right_rect) = self.recording_view_scroll_control_rects(label_rect);
        if rect_contains(left_rect, x, y) && self.can_select_previous_recording_clip(track) {
            return Some(AppAction::SelectPreviousRecordingClip);
        }
        if rect_contains(right_rect, x, y) && self.can_select_next_recording_clip(track) {
            return Some(AppAction::SelectNextRecordingClip);
        }
        None
    }

    pub(super) fn recording_clip_control_rects(&self, label_rect: Rect) -> (Rect, Rect) {
        let top_y = label_rect.y + 3;
        let right = label_rect.x + label_rect.width() as i32 - 4;
        (
            Rect::new(right - 28, top_y, 12, 8),
            Rect::new(right - 12, top_y, 12, 8),
        )
    }

    pub(super) fn recording_clip_scrollbar_rects(
        &self,
        content_rect: Rect,
        track: &Track,
    ) -> Option<(Rect, Rect)> {
        if track.recording_view != RecordingView::Stacked {
            return None;
        }

        let total_lanes = track.recording_clips.len() + usize::from(track.active_take.is_some());
        if total_lanes == 0 {
            return None;
        }

        let window = self.recording_lane_window(track, self.recording_lane_capacity(content_rect));
        let visible_lanes = window.visible_total.clamp(1, total_lanes);
        let start = window.start;
        let rail = Rect::new(
            content_rect.x + 4,
            content_rect.y,
            content_rect.width().saturating_sub(8),
            2,
        );
        if rail.width() == 0 {
            return None;
        }

        let thumb_width = ((rail.width() as usize * visible_lanes) / total_lanes)
            .max(6)
            .min(rail.width() as usize) as u32;
        let max_offset = rail.width().saturating_sub(thumb_width) as i32;
        let max_start = total_lanes.saturating_sub(visible_lanes);
        let thumb_x = if max_start == 0 {
            rail.x
        } else {
            rail.x + (max_offset as i64 * start as i64 / max_start as i64) as i32
        };
        let thumb = Rect::new(thumb_x, rail.y, thumb_width, 2);
        Some((rail, thumb))
    }

    pub(super) fn draw_recording_clip_scrollbar<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        content_rect: Rect,
        track: &Track,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let Some((rail, thumb)) = self.recording_clip_scrollbar_rects(content_rect, track) else {
            return Ok(());
        };
        canvas.set_draw_color(Color::RGB(92, 100, 120));
        canvas.fill_rect(rail)?;
        canvas.set_draw_color(Color::RGB(244, 214, 118));
        canvas.fill_rect(thumb)?;
        Ok(())
    }

    fn recording_lane_window(&self, track: &Track, total_capacity: usize) -> RecordingLaneWindow {
        let total_capacity = total_capacity.max(1);
        let committed_len = track.recording_clips().len();
        let preview_index = track.active_take.as_ref().map(|_| committed_len);
        let total_lanes = committed_len + usize::from(preview_index.is_some());
        if total_lanes == 0 {
            return RecordingLaneWindow {
                start: 0,
                visible_total: 0,
                committed_start: 0,
                committed_end: 0,
                visible_committed: 0,
                show_preview: false,
            };
        }

        let visible_total = total_capacity.min(total_lanes);
        let max_start = total_lanes.saturating_sub(visible_total);
        let mut start = track.recording_clip_scroll.min(max_start);
        if let Some(preview_index) = preview_index {
            start = start.max(preview_index + 1 - visible_total);
        }
        let end = start + visible_total;
        let committed_start = start.min(committed_len);
        let committed_end = end.min(committed_len);
        let show_preview = preview_index.is_some_and(|preview_index| preview_index >= start);

        RecordingLaneWindow {
            start,
            visible_total,
            committed_start,
            committed_end,
            visible_committed: committed_end.saturating_sub(committed_start),
            show_preview,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::actions::AppAction;
    use crate::project::RecordingView;
    use crate::timeline::RecordingTake;

    #[test]
    fn recording_clip_actions_update_active_track_clip_state() {
        let mut app = App::new();
        let transport = app.project.transport;
        {
            let track = app.project.active_track_mut().unwrap();
            track.clear_content();
            track.commit_take(transport, RecordingTake::new(0).release(480), None);
            track.commit_take(transport, RecordingTake::new(960).release(1_440), None);
        }

        app.apply_action(AppAction::ToggleCurrentTrackRecordingView);
        assert_eq!(
            app.project.active_track().unwrap().recording_view,
            RecordingView::Stacked
        );

        app.apply_action(AppAction::SelectPreviousRecordingClip);
        let selected_before_delete = app
            .project
            .active_track()
            .unwrap()
            .selected_recording_clip_id
            .expect("selected clip");
        app.apply_action(AppAction::ToggleSelectedRecordingClipMute);
        assert!(
            app.project
                .active_track()
                .unwrap()
                .selected_recording_clip()
                .unwrap()
                .muted
        );

        app.apply_action(AppAction::DeleteSelectedRecordingClip);
        let active = app.project.active_track().unwrap();
        assert_eq!(active.recording_clips.len(), 1);
        assert_ne!(active.recording_clips[0].id, selected_before_delete);
    }

    #[test]
    fn stacked_all_track_layout_shows_at_least_three_recording_lanes() {
        let app = App::new();
        let timeline_bounds = Rect::new(0, 0, 1000, 420);
        let (_, full_bounds, _) = app.visible_track_columns(timeline_bounds)[0];
        let content_rect = crate::ui::track_content_rect(full_bounds, app.timeline_flow);

        assert!(app.recording_lane_capacity(content_rect) >= 3);
    }

    #[test]
    fn stacked_view_shows_preview_lane_while_recording() {
        let mut app = App::new();
        let transport = app.project.transport;
        {
            let track = app.project.active_track_mut().unwrap();
            track.clear_content();
            track.recording_view = RecordingView::Stacked;
            track.commit_take(transport, RecordingTake::new(0).release(480), None);
        }

        app.transport_ticks = 960;
        app.playhead_ticks = 960;
        app.apply_action(AppAction::ToggleRecording);

        let timeline_bounds = Rect::new(0, 0, 1000, 420);
        let (_, full_bounds, _) = app.visible_track_columns(timeline_bounds)[0];
        let content_rect = crate::ui::track_content_rect(full_bounds, app.timeline_flow);
        let layouts = app.recording_lane_layouts(content_rect, app.project.active_track().unwrap());

        assert_eq!(layouts.len(), 2);
        assert!(layouts.iter().any(|lane| lane.preview));
    }

    #[test]
    fn stacked_view_preview_lane_shifts_visible_window_as_committed() {
        let mut app = App::new();
        let transport = app.project.transport;
        let trailing_clip_ids = {
            let track = app.project.active_track_mut().unwrap();
            track.clear_content();
            track.recording_view = RecordingView::Stacked;
            for index in 0..5 {
                let start = index * 480;
                track.commit_take(
                    transport,
                    RecordingTake::new(start).release(start + 240),
                    None,
                );
            }
            let trailing = vec![track.recording_clips[3].id, track.recording_clips[4].id];
            track.recording_clip_scroll = 2;
            track.active_take = Some(RecordingTake::new(2400));
            trailing
        };

        let content_rect = Rect::new(0, 0, 49, 200);
        assert_eq!(app.recording_lane_capacity(content_rect), 3);
        let layouts = app.recording_lane_layouts(content_rect, app.project.active_track().unwrap());

        assert_eq!(layouts.len(), 3);
        assert_eq!(layouts[0].clip_id, Some(trailing_clip_ids[0]));
        assert_eq!(layouts[1].clip_id, Some(trailing_clip_ids[1]));
        assert!(layouts[2].preview);
    }

    #[test]
    fn stacked_scrollbar_thumb_tracks_clip_window_position() {
        let mut app = App::new();
        let transport = app.project.transport;
        {
            let track = app.project.active_track_mut().unwrap();
            track.clear_content();
            track.recording_view = RecordingView::Stacked;
            for index in 0..5 {
                let start = index * 480;
                track.commit_take(
                    transport,
                    RecordingTake::new(start).release(start + 240),
                    None,
                );
            }
            track.recording_clip_scroll = 0;
        }

        let timeline_bounds = Rect::new(0, 0, 1000, 420);
        let (_, full_bounds, _) = app.visible_track_columns(timeline_bounds)[0];
        let content_rect = crate::ui::track_content_rect(full_bounds, app.timeline_flow);
        let (_, thumb_before) = app
            .recording_clip_scrollbar_rects(content_rect, app.project.active_track().unwrap())
            .expect("scrollbar");

        app.project
            .active_track_mut()
            .unwrap()
            .recording_clip_scroll = 2;
        let (_, thumb_after) = app
            .recording_clip_scrollbar_rects(content_rect, app.project.active_track().unwrap())
            .expect("scrollbar");

        assert!(thumb_after.x > thumb_before.x);
    }
}
