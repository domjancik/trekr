use super::*;

impl App {
    pub(super) fn visible_track_columns(&self, timeline_bounds: Rect) -> Vec<(usize, Rect, Rect)> {
        if self.project.tracks.is_empty() {
            return Vec::new();
        }

        if self.focused_track_view {
            return crate::ui::track_column_pairs(timeline_bounds, 1)
                .into_iter()
                .next()
                .map(|(full_bounds, detail_bounds)| {
                    vec![(self.project.active_track_index, full_bounds, detail_bounds)]
                })
                .unwrap_or_default();
        }

        crate::ui::track_column_pairs(timeline_bounds, self.project.tracks.len())
            .into_iter()
            .enumerate()
            .map(|(index, (full_bounds, detail_bounds))| (index, full_bounds, detail_bounds))
            .collect()
    }

    pub(super) fn timeline_track_layout(
        &self,
        track_index: usize,
        full_bounds: Rect,
        detail_bounds: Rect,
    ) -> TimelineTrackLayout {
        let pair_bounds = crate::ui::union_rect(full_bounds, detail_bounds);
        let status_rect = crate::ui::track_status_rect(pair_bounds, self.timeline_flow);
        let (body_full_bounds, body_detail_bounds) =
            self.track_column_body_bounds(full_bounds, detail_bounds);
        let full_label_rect = timeline_subcolumn_label_rect(body_full_bounds, self.timeline_flow);
        let detail_label_rect =
            timeline_subcolumn_label_rect(body_detail_bounds, self.timeline_flow);
        let full_content_rect =
            timeline_subcolumn_content_rect(body_full_bounds, self.timeline_flow);
        let detail_content_rect =
            timeline_subcolumn_content_rect(body_detail_bounds, self.timeline_flow);
        let (input_fx_rect, output_fx_rect) = self.track_fx_band_rects(
            full_bounds,
            detail_bounds,
            &self.project.tracks[track_index],
        );
        TimelineTrackLayout {
            track_index,
            full_bounds,
            detail_bounds,
            pair_bounds,
            status_rect,
            body_full_bounds,
            body_detail_bounds,
            full_label_rect,
            detail_label_rect,
            full_content_rect,
            detail_content_rect,
            input_fx_rect,
            output_fx_rect,
        }
    }

    pub(super) fn active_track_full_bounds(&self) -> Option<Rect> {
        let surface = crate::ui::surface_rect(self.viewport_size.0, self.viewport_size.1);
        let inset = crate::ui::inset_rect(surface, 24, 24).ok()?;
        let (_, content_bounds) = crate::ui::split_top_strip(inset, 28, 12).ok()?;
        let (_, body_bounds) = crate::ui::split_top_strip(content_bounds, 28, 6).ok()?;
        let (_, timeline_bounds) =
            crate::ui::split_top_strip(body_bounds, transport_strip_height(), 8).ok()?;
        self.visible_track_columns(timeline_bounds)
            .into_iter()
            .find(|(index, _, _)| *index == self.project.active_track_index)
            .map(|(_, full_bounds, _)| full_bounds)
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
}

pub(super) fn loop_regions_intersect(
    a: crate::timeline::LoopRegion,
    b: crate::timeline::LoopRegion,
) -> bool {
    a.start_ticks < b.end_ticks() && a.end_ticks() > b.start_ticks
}

pub(super) fn interlaced_color_at(colors: &[Color], pixel_index: usize) -> Option<Color> {
    (!colors.is_empty()).then_some(colors[pixel_index % colors.len()])
}

pub(super) fn rects_overlap(a: Rect, b: Rect) -> bool {
    a.x < b.x + b.width() as i32
        && a.x + a.width() as i32 > b.x
        && a.y < b.y + b.height() as i32
        && a.y + a.height() as i32 > b.y
}

pub(super) fn displayed_track_fx_band_height(chain: &[Option<MidiFxSlot>]) -> i32 {
    let line_height = 8_i32;
    let line_gap = 2_i32;
    let vertical_padding = 4_i32;
    let active = chain.iter().flatten().count();
    let show_add = active < chain.len().max(MIDI_FX_SLOT_COUNT);
    let line_count = (active + usize::from(show_add)).max(1) as i32;
    vertical_padding + line_count * line_height + (line_count - 1) * line_gap
}

pub(super) fn timeline_subcolumn_label_rect(lane: Rect, flow: TimelineFlow) -> Rect {
    match flow {
        TimelineFlow::DownwardColumns => Rect::new(lane.x, lane.y, lane.width(), 24),
        TimelineFlow::AcrossRows => Rect::new(lane.x, lane.y, 56, lane.height().saturating_sub(14)),
    }
}

pub(super) fn timeline_subcolumn_content_rect(lane: Rect, flow: TimelineFlow) -> Rect {
    match flow {
        TimelineFlow::DownwardColumns => Rect::new(
            lane.x,
            lane.y + 24,
            lane.width(),
            lane.height().saturating_sub(24),
        ),
        TimelineFlow::AcrossRows => Rect::new(
            lane.x + 56,
            lane.y,
            lane.width().saturating_sub(56),
            lane.height(),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interlaced_color_pattern_cycles_proportionally() {
        let b = Color::RGB(0, 0, 255);
        let r = Color::RGB(255, 0, 0);
        let g = Color::RGB(0, 255, 0);

        let two = [b, r];
        assert_eq!(
            (0..4)
                .filter_map(|pixel| interlaced_color_at(&two, pixel))
                .collect::<Vec<_>>(),
            vec![b, r, b, r]
        );

        let three = [r, b, g];
        assert_eq!(
            (0..6)
                .filter_map(|pixel| interlaced_color_at(&three, pixel))
                .collect::<Vec<_>>(),
            vec![r, b, g, r, b, g]
        );
    }

    #[test]
    fn timeline_body_label_controls_do_not_overlap_input_fx_band() {
        let app = App::new();
        let content_bounds = Rect::new(40, 40, 1200, 620);
        let (_, body_bounds) =
            crate::ui::split_top_strip(content_bounds, 28, 6).expect("timeline content");
        let (_, timeline_bounds) =
            crate::ui::split_top_strip(body_bounds, transport_strip_height(), 8)
                .expect("timeline body");
        let columns = crate::ui::track_column_pairs(timeline_bounds, app.project.tracks.len());
        let (full_bounds, detail_bounds) = columns[0];
        let (body_full_bounds, body_detail_bounds) =
            app.track_column_body_bounds(full_bounds, detail_bounds);
        let full_label_rect = timeline_subcolumn_label_rect(body_full_bounds, app.timeline_flow);
        let detail_label_rect =
            timeline_subcolumn_label_rect(body_detail_bounds, app.timeline_flow);
        let (input_fx_rect, _) =
            app.track_fx_band_rects(full_bounds, detail_bounds, &app.project.tracks[0]);
        let view_rect = app.recording_view_chip_rect(full_label_rect);
        let thru_rect = app.track_passthrough_button_rect(full_label_rect);
        let detail_badge = crate::ui::detail_badge_rect(detail_label_rect);
        let stored_slot = app.stored_loop_slot_rects(detail_label_rect)[0].1;
        let intersects = |a: Rect, b: Rect| {
            a.x < b.x + b.width() as i32
                && a.x + a.width() as i32 > b.x
                && a.y < b.y + b.height() as i32
                && a.y + a.height() as i32 > b.y
        };

        assert!(!intersects(input_fx_rect, view_rect));
        assert!(!intersects(input_fx_rect, thru_rect));
        assert!(!intersects(input_fx_rect, detail_badge));
        assert!(!intersects(input_fx_rect, stored_slot));
    }

    #[test]
    fn timeline_resized_content_rects_do_not_overlap_input_fx_band() {
        let app = App::new();
        let content_bounds = Rect::new(40, 40, 1200, 620);
        let (_, body_bounds) =
            crate::ui::split_top_strip(content_bounds, 28, 6).expect("timeline content");
        let (_, timeline_bounds) =
            crate::ui::split_top_strip(body_bounds, transport_strip_height(), 8)
                .expect("timeline body");
        let columns = crate::ui::track_column_pairs(timeline_bounds, app.project.tracks.len());
        let (full_bounds, detail_bounds) = columns[0];
        let (body_full_bounds, body_detail_bounds) =
            app.track_column_body_bounds(full_bounds, detail_bounds);
        let (input_band, _) =
            app.track_fx_band_rects(full_bounds, detail_bounds, &app.project.tracks[0]);
        let full_content = crate::ui::track_content_rect(body_full_bounds, app.timeline_flow);
        let detail_content = crate::ui::track_content_rect(body_detail_bounds, app.timeline_flow);
        let intersects = |a: Rect, b: Rect| {
            a.x < b.x + b.width() as i32
                && a.x + a.width() as i32 > b.x
                && a.y < b.y + b.height() as i32
                && a.y + a.height() as i32 > b.y
        };

        assert!(!intersects(input_band, full_content));
        assert!(!intersects(input_band, detail_content));
    }

    #[test]
    fn canonical_timeline_layout_keeps_output_fx_band_disjoint_from_body_content() {
        let app = App::new();
        let content_bounds = Rect::new(40, 40, 1200, 620);
        let (_, body_bounds) =
            crate::ui::split_top_strip(content_bounds, 28, 6).expect("timeline content");
        let (_, timeline_bounds) =
            crate::ui::split_top_strip(body_bounds, transport_strip_height(), 8)
                .expect("timeline body");
        let layout = app.visible_timeline_track_layouts(timeline_bounds)[0];
        let intersects = |a: Rect, b: Rect| {
            a.x < b.x + b.width() as i32
                && a.x + a.width() as i32 > b.x
                && a.y < b.y + b.height() as i32
                && a.y + a.height() as i32 > b.y
        };

        assert!(!intersects(layout.output_fx_rect, layout.full_content_rect));
        assert!(!intersects(layout.output_fx_rect, layout.detail_content_rect));
    }

    #[test]
    fn output_fx_band_starts_below_track_body_with_fixed_gap() {
        let app = App::new();
        let content_bounds = Rect::new(40, 40, 1200, 620);
        let (_, body_bounds) =
            crate::ui::split_top_strip(content_bounds, 28, 6).expect("timeline content");
        let (_, timeline_bounds) =
            crate::ui::split_top_strip(body_bounds, transport_strip_height(), 8)
                .expect("timeline body");
        let columns = crate::ui::track_column_pairs(timeline_bounds, app.project.tracks.len());
        let (full_bounds, detail_bounds) = columns[0];
        let (body_full_bounds, body_detail_bounds) =
            app.track_column_body_bounds(full_bounds, detail_bounds);
        let body_pair = crate::ui::union_rect(body_full_bounds, body_detail_bounds);
        let (_, output_rect) =
            app.track_fx_band_rects(full_bounds, detail_bounds, &app.project.tracks[0]);

        assert_eq!(output_rect.y, body_pair.y + body_pair.height() as i32 + 4);
    }
}
