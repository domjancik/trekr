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
}
