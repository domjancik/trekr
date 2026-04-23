use super::*;

pub(super) fn recall_stored_loop_slot_index(action: AppAction) -> Option<usize> {
    match action {
        AppAction::RecallStoredLoopSlot1 => Some(0),
        AppAction::RecallStoredLoopSlot2 => Some(1),
        AppAction::RecallStoredLoopSlot3 => Some(2),
        AppAction::RecallStoredLoopSlot4 => Some(3),
        AppAction::RecallStoredLoopSlot5 => Some(4),
        AppAction::RecallStoredLoopSlot6 => Some(5),
        AppAction::RecallStoredLoopSlot7 => Some(6),
        AppAction::RecallStoredLoopSlot8 => Some(7),
        _ => None,
    }
}

pub(super) fn store_stored_loop_slot_index(action: AppAction) -> Option<usize> {
    match action {
        AppAction::StoreCurrentLoopToSlot1 => Some(0),
        AppAction::StoreCurrentLoopToSlot2 => Some(1),
        AppAction::StoreCurrentLoopToSlot3 => Some(2),
        AppAction::StoreCurrentLoopToSlot4 => Some(3),
        AppAction::StoreCurrentLoopToSlot5 => Some(4),
        AppAction::StoreCurrentLoopToSlot6 => Some(5),
        AppAction::StoreCurrentLoopToSlot7 => Some(6),
        AppAction::StoreCurrentLoopToSlot8 => Some(7),
        _ => None,
    }
}

pub(super) fn clear_stored_loop_slot_index(action: AppAction) -> Option<usize> {
    match action {
        AppAction::ClearStoredLoopSlot1 => Some(0),
        AppAction::ClearStoredLoopSlot2 => Some(1),
        AppAction::ClearStoredLoopSlot3 => Some(2),
        AppAction::ClearStoredLoopSlot4 => Some(3),
        AppAction::ClearStoredLoopSlot5 => Some(4),
        AppAction::ClearStoredLoopSlot6 => Some(5),
        AppAction::ClearStoredLoopSlot7 => Some(6),
        AppAction::ClearStoredLoopSlot8 => Some(7),
        _ => None,
    }
}

pub(super) fn stored_loop_slot_recall_action(slot_index: usize) -> Option<AppAction> {
    match slot_index {
        0 => Some(AppAction::RecallStoredLoopSlot1),
        1 => Some(AppAction::RecallStoredLoopSlot2),
        2 => Some(AppAction::RecallStoredLoopSlot3),
        3 => Some(AppAction::RecallStoredLoopSlot4),
        4 => Some(AppAction::RecallStoredLoopSlot5),
        5 => Some(AppAction::RecallStoredLoopSlot6),
        6 => Some(AppAction::RecallStoredLoopSlot7),
        7 => Some(AppAction::RecallStoredLoopSlot8),
        _ => None,
    }
}

pub(super) fn stored_loop_slot_color(slot_index: usize) -> Color {
    match slot_index % STORED_LOOP_SLOT_COUNT {
        0 => Color::RGB(214, 124, 118),
        1 => Color::RGB(214, 176, 98),
        2 => Color::RGB(184, 206, 108),
        3 => Color::RGB(114, 198, 174),
        4 => Color::RGB(114, 168, 214),
        5 => Color::RGB(144, 138, 214),
        6 => Color::RGB(204, 132, 206),
        _ => Color::RGB(210, 144, 164),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::actions::ActionSource;
    use crate::transport::LaunchQuantizeMode;
    use sdl3::rect::Rect;

    #[test]
    fn stored_loop_slot_index_mappings_cover_all_slots() {
        assert_eq!(recall_stored_loop_slot_index(AppAction::RecallStoredLoopSlot1), Some(0));
        assert_eq!(
            store_stored_loop_slot_index(AppAction::StoreCurrentLoopToSlot8),
            Some(7)
        );
        assert_eq!(clear_stored_loop_slot_index(AppAction::ClearStoredLoopSlot4), Some(3));
        assert_eq!(stored_loop_slot_recall_action(5), Some(AppAction::RecallStoredLoopSlot6));
        assert_eq!(stored_loop_slot_recall_action(99), None);
    }

    #[test]
    fn stored_loop_actions_store_and_recall_active_track_slot() {
        let mut app = App::new();
        let track = app.project.active_track_mut().unwrap();
        track.loop_region = crate::timeline::LoopRegion::new(1_920, 960);

        app.apply_action(AppAction::StoreCurrentLoopToSlot2);
        let track = app.project.active_track_mut().unwrap();
        track.loop_region = crate::timeline::LoopRegion::new(0, 4_800);
        track.state.loop_enabled = false;

        app.apply_action(AppAction::RecallStoredLoopSlot2);

        let track = app.project.active_track().unwrap();
        assert_eq!(track.loop_region, crate::timeline::LoopRegion::new(1_920, 960));
        assert!(track.state.loop_enabled);
        assert_eq!(track.active_stored_loop_slot(), Some(1));
    }

    #[test]
    fn timeline_stored_loop_slot_is_clickable_for_recall() {
        let mut app = App::new();
        app.project.active_track_index = 1;
        {
            let track = &mut app.project.tracks[1];
            track.loop_region = crate::timeline::LoopRegion::new(2_880, 960);
            track.store_current_loop_to_slot(0);
            track.loop_region = crate::timeline::LoopRegion::new(0, 3_840);
        }

        let content_bounds = Rect::new(40, 40, 1200, 620);
        let (_, body_bounds) =
            crate::ui::split_top_strip(content_bounds, 28, 6).expect("timeline content");
        let (_, timeline_bounds) =
            crate::ui::split_top_strip(body_bounds, transport_strip_height(), 8)
                .expect("timeline body");
        let columns = crate::ui::track_column_pairs(timeline_bounds, app.project.tracks.len());
        let (full_bounds, detail_bounds) = columns[1];
        let (_, body_detail_bounds) = app.track_column_body_bounds(full_bounds, detail_bounds);
        let detail_label_rect =
            super::timeline::layout::timeline_subcolumn_label_rect(body_detail_bounds, app.timeline_flow);
        let (_, slot_rect) = app.stored_loop_slot_rects(detail_label_rect)[0];

        let control = app.handle_timeline_pointer(
            content_bounds,
            slot_rect.x + slot_rect.width() as i32 / 2,
            slot_rect.y + slot_rect.height() as i32 / 2,
            ActionSource::Pointer,
        );

        assert_eq!(control, Some(AppControl::Continue));
        assert_eq!(app.project.active_track_index, 1);
        assert_eq!(
            app.project.tracks[1].loop_region,
            crate::timeline::LoopRegion::new(2_880, 960)
        );
    }

    #[test]
    fn manual_loop_edit_unlinks_active_stored_loop_slot() {
        let mut app = App::new();
        let track = app.project.active_track_mut().unwrap();
        track.loop_region = crate::timeline::LoopRegion::new(1_920, 960);
        assert!(track.store_current_loop_to_slot(2));
        assert_eq!(track.active_stored_loop_slot(), Some(2));

        app.transport_ticks = 0;
        app.apply_action(AppAction::SetCurrentTrackLoopStart);

        assert_eq!(app.project.active_track().unwrap().active_stored_loop_slot(), None);
    }

    #[test]
    fn quantized_stored_loop_recall_queues_and_resolves_at_boundary() {
        let mut app = App::new();
        app.project.transport.stored_loop_recall_quantized = true;
        app.project.transport.stored_loop_launch_quantize = LaunchQuantizeMode::Quarter;
        app.project.transport.playing = true;
        app.transport_ticks = 1_000;
        app.playhead_ticks = 1_000;

        {
            let track = app.project.active_track_mut().unwrap();
            track.loop_region = crate::timeline::LoopRegion::new(0, 960);
            assert!(track.store_current_loop_to_slot(0));
            track.loop_region = crate::timeline::LoopRegion::new(1_920, 960);
            assert!(track.store_current_loop_to_slot(1));
            track.loop_region = crate::timeline::LoopRegion::new(0, 960);
        }

        app.apply_action(AppAction::RecallStoredLoopSlot2);

        let track = app.project.active_track().unwrap();
        assert_eq!(track.active_stored_loop_slot(), None);
        assert_eq!(track.queued_stored_loop_slot(), Some(1));
        assert_eq!(track.loop_region, crate::timeline::LoopRegion::new(0, 960));

        app.process_queued_stored_loop_recalls(1_000, 1_920);
        let track = app.project.active_track().unwrap();
        assert_eq!(track.active_stored_loop_slot(), Some(1));
        assert_eq!(track.queued_stored_loop_slot(), None);
    }

    #[test]
    fn stored_loop_recall_is_blocked_on_recording_track() {
        let mut app = App::new();
        app.project.transport.stored_loop_recall_quantized = true;
        app.project.transport.stored_loop_launch_quantize = LaunchQuantizeMode::Bar;
        app.project.transport.playing = true;

        let track = app.project.active_track_mut().unwrap();
        track.loop_region = crate::timeline::LoopRegion::new(960, 960);
        assert!(track.store_current_loop_to_slot(0));
        track.loop_region = crate::timeline::LoopRegion::new(0, 960);
        track.begin_recording(0);

        app.apply_action(AppAction::RecallStoredLoopSlot1);

        let track = app.project.active_track().unwrap();
        assert_eq!(track.loop_region, crate::timeline::LoopRegion::new(0, 960));
        assert_eq!(track.queued_stored_loop_slot(), None);
        assert_eq!(track.active_stored_loop_slot(), None);
    }

    #[test]
    fn stored_loop_recall_enables_track_loop_before_queueing() {
        let mut app = App::new();
        app.project.transport.stored_loop_recall_quantized = true;
        app.project.transport.stored_loop_launch_quantize = LaunchQuantizeMode::LoopEnd;
        app.project.transport.playing = true;
        app.transport_ticks = 1_000;
        app.playhead_ticks = 1_000;

        let track = app.project.active_track_mut().unwrap();
        track.state.loop_enabled = false;
        track.loop_region = crate::timeline::LoopRegion::new(0, 960);
        assert!(track.store_current_loop_to_slot(0));

        app.apply_action(AppAction::RecallStoredLoopSlot1);

        let track = app.project.active_track().unwrap();
        assert!(track.state.loop_enabled);
        assert_eq!(track.queued_stored_loop_slot(), Some(0));
    }

    #[test]
    fn stored_loop_slot_rects_expand_to_fit_available_label_width() {
        let app = App::new();
        let wide = Rect::new(0, 0, 120, 14);
        let narrow = Rect::new(0, 0, 44, 14);

        assert_eq!(app.stored_loop_slot_rects(wide).len(), STORED_LOOP_SLOT_COUNT);
        assert!(app.stored_loop_slot_rects(narrow).len() < STORED_LOOP_SLOT_COUNT);
    }
}
