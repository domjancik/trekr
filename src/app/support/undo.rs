use super::*;
use crate::undo::{
    MappingsUndoState, TimelineUndoState, UiUndoState, UndoDomain, UndoEntry, UndoHistory,
    UndoOverlayState, UndoSnapshot, UndoTransaction,
};

#[derive(Debug, Clone, Default)]
struct CandidateSnapshots {
    timeline: Option<TimelineUndoState>,
    mappings: Option<MappingsUndoState>,
    ui: Option<UiUndoState>,
}

impl App {
    pub fn set_undo_history(&mut self, history: UndoHistory) {
        self.undo_history = history;
    }

    pub fn undo_history(&self) -> &UndoHistory {
        &self.undo_history
    }

    pub(crate) fn apply_action(&mut self, action: AppAction) -> AppControl {
        match action {
            AppAction::Undo => return self.perform_undo(None),
            AppAction::Redo => return self.perform_redo(None),
            AppAction::UndoTimeline => return self.perform_undo(Some(UndoDomain::Timeline)),
            AppAction::RedoTimeline => return self.perform_redo(Some(UndoDomain::Timeline)),
            AppAction::UndoMappings => return self.perform_undo(Some(UndoDomain::Mappings)),
            AppAction::RedoMappings => return self.perform_redo(Some(UndoDomain::Mappings)),
            AppAction::UndoUi => return self.perform_undo(Some(UndoDomain::Ui)),
            AppAction::RedoUi => return self.perform_redo(Some(UndoDomain::Ui)),
            _ => {}
        }

        let candidate_domains = self.undo_candidate_domains(action);
        let before = self.capture_candidate_snapshots(&candidate_domains);
        let control = self.apply_action_inner(action);
        self.record_action_history(action, before, candidate_domains);
        control
    }

    fn undo_candidate_domains(&self, action: AppAction) -> Vec<UndoDomain> {
        match action {
            AppAction::Quit
            | AppAction::Undo
            | AppAction::Redo
            | AppAction::UndoTimeline
            | AppAction::RedoTimeline
            | AppAction::UndoMappings
            | AppAction::RedoMappings
            | AppAction::UndoUi
            | AppAction::RedoUi
            | AppAction::ToggleLinkEnabled
            | AppAction::ToggleLinkStartStopSync
            | AppAction::TogglePlayback
            | AppAction::ToggleRecording
            | AppAction::StartRecording
            | AppAction::StopRecording
            | AppAction::BeginNoteAdditiveSelectionHold
            | AppAction::EndNoteAdditiveSelectionHold => Vec::new(),
            AppAction::CycleRecordMode
            | AppAction::ToggleLoopRecordingExtension
            | AppAction::ToggleGlobalLoop
            | AppAction::ResetGlobalLoop
            | AppAction::ClearCurrentTrackContent
            | AppAction::ClearAllTrackContent
            | AppAction::ToggleCurrentTrackLoop
            | AppAction::ToggleStoredLoopRecallQuantize
            | AppAction::CycleStoredLoopLaunchQuantize
            | AppAction::SetCurrentTrackLoopStart
            | AppAction::SetCurrentTrackLoopEnd
            | AppAction::SetGlobalLoopStart
            | AppAction::SetGlobalLoopEnd
            | AppAction::NudgeCurrentTrackLoopBackward
            | AppAction::NudgeCurrentTrackLoopForward
            | AppAction::NudgeGlobalLoopBackward
            | AppAction::NudgeGlobalLoopForward
            | AppAction::ShortenCurrentTrackLoop
            | AppAction::ExtendCurrentTrackLoop
            | AppAction::HalfCurrentTrackLoop
            | AppAction::DoubleCurrentTrackLoop
            | AppAction::RecallStoredLoopSlot1
            | AppAction::RecallStoredLoopSlot2
            | AppAction::RecallStoredLoopSlot3
            | AppAction::RecallStoredLoopSlot4
            | AppAction::RecallStoredLoopSlot5
            | AppAction::RecallStoredLoopSlot6
            | AppAction::RecallStoredLoopSlot7
            | AppAction::RecallStoredLoopSlot8
            | AppAction::StoreCurrentLoopToSlot1
            | AppAction::StoreCurrentLoopToSlot2
            | AppAction::StoreCurrentLoopToSlot3
            | AppAction::StoreCurrentLoopToSlot4
            | AppAction::StoreCurrentLoopToSlot5
            | AppAction::StoreCurrentLoopToSlot6
            | AppAction::StoreCurrentLoopToSlot7
            | AppAction::StoreCurrentLoopToSlot8
            | AppAction::ClearStoredLoopSlot1
            | AppAction::ClearStoredLoopSlot2
            | AppAction::ClearStoredLoopSlot3
            | AppAction::ClearStoredLoopSlot4
            | AppAction::ClearStoredLoopSlot5
            | AppAction::ClearStoredLoopSlot6
            | AppAction::ClearStoredLoopSlot7
            | AppAction::ClearStoredLoopSlot8
            | AppAction::ShortenGlobalLoop
            | AppAction::ExtendGlobalLoop
            | AppAction::HalfGlobalLoop
            | AppAction::DoubleGlobalLoop
            | AppAction::ToggleCurrentTrackArm
            | AppAction::ToggleCurrentTrackMute
            | AppAction::ToggleCurrentTrackSolo
            | AppAction::ToggleCurrentTrackPassthrough
            | AppAction::ToggleCurrentTrackRecordingView
            | AppAction::SelectRecordingClip(_)
            | AppAction::SelectPreviousRecordingClip
            | AppAction::SelectNextRecordingClip
            | AppAction::ToggleSelectedRecordingClipMute
            | AppAction::DeleteSelectedRecordingClip
            | AppAction::SelectTrack(_)
            | AppAction::SelectNextTrack
            | AppAction::SelectPreviousTrack
            | AppAction::SelectNotesAtPlayhead
            | AppAction::SelectNotesAtPlayheadAdd
            | AppAction::DeselectTrackNotes
            | AppAction::SelectNextNote
            | AppAction::SelectPreviousNote
            | AppAction::FocusFirstSelectedNote
            | AppAction::FocusLastSelectedNote
            | AppAction::ExtendNoteSelectionForward
            | AppAction::ExtendNoteSelectionBackward
            | AppAction::ExtendNoteSelectionBoth
            | AppAction::ContractNoteSelection
            | AppAction::NudgeSelectedNotesEarlier
            | AppAction::NudgeSelectedNotesLater
            | AppAction::NudgeSelectedNotesUp
            | AppAction::NudgeSelectedNotesDown => vec![UndoDomain::Timeline],
            AppAction::ShowPage(_)
            | AppAction::ShowNextPage
            | AppAction::ShowPreviousPage
            | AppAction::ToggleMappingsOverlay
            | AppAction::ToggleDiscoverabilityOverlay
            | AppAction::ToggleDirectMappingMode
            | AppAction::ToggleMappingsWriteMode
            | AppAction::SelectPreviousPageField
            | AppAction::SelectNextPageField
            | AppAction::ToggleFocusedTrackView
            | AppAction::SetTimelineFlow(_) => vec![UndoDomain::Ui],
            AppAction::AddMappingRow | AppAction::RemoveSelectedMapping => {
                vec![UndoDomain::Mappings]
            }
            AppAction::SelectPreviousPageItem | AppAction::SelectNextPageItem => {
                match self.page_state.current_page {
                    AppPage::Timeline => vec![UndoDomain::Timeline],
                    _ => vec![UndoDomain::Ui],
                }
            }
            AppAction::AdjustPageItemBackward
            | AppAction::AdjustPageItemForward
            | AppAction::ReverseActivatePageItem
            | AppAction::DeletePageItem => match self.page_state.current_page {
                AppPage::Mappings => vec![UndoDomain::Mappings, UndoDomain::Ui],
                AppPage::MidiIo => vec![UndoDomain::Ui],
                AppPage::Routing => vec![UndoDomain::Timeline],
                AppPage::Timeline => Vec::new(),
            },
            AppAction::ActivatePageItem => match self.page_state.current_page {
                AppPage::Mappings => vec![UndoDomain::Mappings, UndoDomain::Ui],
                AppPage::MidiIo => vec![UndoDomain::Ui],
                AppPage::Routing => vec![UndoDomain::Timeline],
                AppPage::Timeline => Vec::new(),
            },
            AppAction::CancelCurrentMode => vec![UndoDomain::Ui],
            AppAction::CycleGlobalHarmonyRoot
            | AppAction::ToggleSelectedTimelineFx
            | AppAction::CycleSelectedTimelineFxKind
            | AppAction::AdjustSelectedTimelineFxPrimary
            | AppAction::AdjustSelectedTimelineFxSecondary
            | AppAction::ScrollSelectedTimelineFxWindow
            | AppAction::MoveSelectedTimelineFxUp
            | AppAction::MoveSelectedTimelineFxDown
            | AppAction::AddSelectedTimelineFx
            | AppAction::DeleteSelectedTimelineFx => vec![UndoDomain::Timeline],
        }
    }

    fn capture_candidate_snapshots(&self, domains: &[UndoDomain]) -> CandidateSnapshots {
        CandidateSnapshots {
            timeline: domains
                .contains(&UndoDomain::Timeline)
                .then(|| self.capture_timeline_undo_state()),
            mappings: domains
                .contains(&UndoDomain::Mappings)
                .then(|| self.capture_mappings_undo_state()),
            ui: domains
                .contains(&UndoDomain::Ui)
                .then(|| self.capture_ui_undo_state()),
        }
    }

    fn capture_timeline_undo_state(&self) -> TimelineUndoState {
        TimelineUndoState {
            project: self.project.clone(),
        }
    }

    fn capture_mappings_undo_state(&self) -> MappingsUndoState {
        MappingsUndoState {
            mappings: self.mappings.clone(),
            selected_mapping_index: self.page_state.selected_mapping_index,
            selected_mapping_field: self.page_state.selected_mapping_field,
        }
    }

    fn capture_ui_undo_state(&self) -> UiUndoState {
        UiUndoState {
            page_state: self.page_state,
            timeline_flow: self.timeline_flow,
            overlay_state: match self.overlay_state.active {
                Some(AppOverlay::MappingsQuickView) => UndoOverlayState::MappingsQuickView,
                Some(AppOverlay::Discoverability) => UndoOverlayState::Discoverability,
                None => UndoOverlayState::None,
            },
            focused_track_view: self.focused_track_view,
            direct_mapping_active: self.direct_mapping_state.mode != DirectMappingMode::Inactive,
        }
    }

    fn record_action_history(
        &mut self,
        action: AppAction,
        before: CandidateSnapshots,
        domains: Vec<UndoDomain>,
    ) {
        if domains.is_empty() {
            return;
        }

        let mut entries = Vec::new();
        let label = action_label(action).to_string();

        if let Some(before_timeline) = before.timeline {
            let after = self.capture_timeline_undo_state();
            if before_timeline != after {
                entries.push(UndoEntry {
                    domain: UndoDomain::Timeline,
                    label: label.clone(),
                    before: UndoSnapshot::Timeline(before_timeline),
                    after: UndoSnapshot::Timeline(after),
                });
            }
        }
        if let Some(before_mappings) = before.mappings {
            let after = self.capture_mappings_undo_state();
            if before_mappings != after {
                entries.push(UndoEntry {
                    domain: UndoDomain::Mappings,
                    label: label.clone(),
                    before: UndoSnapshot::Mappings(before_mappings),
                    after: UndoSnapshot::Mappings(after),
                });
            }
        }
        if let Some(before_ui) = before.ui {
            let after = self.capture_ui_undo_state();
            if before_ui != after {
                entries.push(UndoEntry {
                    domain: UndoDomain::Ui,
                    label: label.clone(),
                    before: UndoSnapshot::Ui(before_ui),
                    after: UndoSnapshot::Ui(after),
                });
            }
        }

        if self.undo_history.push_transaction(label, entries) {
            self.status_state.history_message = None;
        }
    }

    fn perform_undo(&mut self, domain: Option<UndoDomain>) -> AppControl {
        let transaction = match domain {
            Some(domain) => self.undo_history.undo_domain(domain),
            None => self.undo_history.undo_global(),
        };
        if let Some(transaction) = transaction {
            self.apply_undo_transaction(&transaction, true);
            self.set_history_message("Undid", &transaction);
        } else {
            self.status_state.history_message = Some(match domain {
                Some(domain) => format!("Last Action: Nothing to undo in {}", domain.label()),
                None => "Last Action: Nothing to undo".to_string(),
            });
        }
        AppControl::Continue
    }

    fn perform_redo(&mut self, domain: Option<UndoDomain>) -> AppControl {
        let transaction = match domain {
            Some(domain) => self.undo_history.redo_domain(domain),
            None => self.undo_history.redo_global(),
        };
        if let Some(transaction) = transaction {
            self.apply_undo_transaction(&transaction, false);
            self.set_history_message("Redid", &transaction);
        } else {
            self.status_state.history_message = Some(match domain {
                Some(domain) => format!("Last Action: Nothing to redo in {}", domain.label()),
                None => "Last Action: Nothing to redo".to_string(),
            });
        }
        AppControl::Continue
    }

    fn apply_undo_transaction(&mut self, transaction: &UndoTransaction, use_before: bool) {
        for entry in &transaction.entries {
            let snapshot = if use_before {
                &entry.before
            } else {
                &entry.after
            };
            self.apply_undo_snapshot(snapshot);
        }
        self.sync_midi_inputs();
        self.sync_active_track_recording_clip_scroll();
    }

    fn apply_undo_snapshot(&mut self, snapshot: &UndoSnapshot) {
        match snapshot {
            UndoSnapshot::Timeline(state) => {
                let session_transport = self.project.transport;
                let mut restored_project = state.project.clone();
                restored_project.transport.playing = session_transport.playing;
                restored_project.transport.recording = session_transport.recording;
                restored_project.transport.link_enabled = session_transport.link_enabled;
                restored_project.transport.link_start_stop_sync =
                    session_transport.link_start_stop_sync;
                self.project = restored_project;
            }
            UndoSnapshot::Mappings(state) => {
                self.mappings = state.mappings.clone();
                self.page_state.selected_mapping_index = state
                    .selected_mapping_index
                    .min(self.mappings.len().saturating_sub(1));
                self.page_state.selected_mapping_field = state.selected_mapping_field;
                self.normalize_selected_mapping_field();
            }
            UndoSnapshot::Ui(state) => {
                self.page_state = state.page_state;
                self.timeline_flow = state.timeline_flow;
                self.overlay_state.active = match state.overlay_state {
                    UndoOverlayState::None => None,
                    UndoOverlayState::MappingsQuickView => Some(AppOverlay::MappingsQuickView),
                    UndoOverlayState::Discoverability => Some(AppOverlay::Discoverability),
                };
                self.focused_track_view = state.focused_track_view;
                self.direct_mapping_state.mode = if state.direct_mapping_active {
                    DirectMappingMode::Targeting
                } else {
                    DirectMappingMode::Inactive
                };
                self.direct_mapping_state.origin = DirectMappingOrigin::InPlace;
                self.direct_mapping_state.status_message = None;
            }
        }
    }

    fn set_history_message(&mut self, verb: &str, transaction: &UndoTransaction) {
        let domain_label = if let [entry] = transaction.entries.as_slice() {
            entry.domain.label().to_string()
        } else {
            "Multiple".to_string()
        };
        self.status_state.history_message = Some(format!(
            "Last Action: {} {} ({})",
            verb, transaction.label, domain_label
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generic_undo_and_redo_restore_mapping_addition() {
        let mut app = App::new();
        let original_len = app.mappings.len();
        app.apply_action(AppAction::ShowPage(AppPage::Mappings));
        app.apply_action(AppAction::ToggleMappingsWriteMode);

        app.apply_action(AppAction::AddMappingRow);
        assert_eq!(app.mappings.len(), original_len + 1);

        app.apply_action(AppAction::Undo);
        assert_eq!(app.mappings.len(), original_len);

        app.apply_action(AppAction::Redo);
        assert_eq!(app.mappings.len(), original_len + 1);
    }

    #[test]
    fn undo_mappings_leaves_timeline_changes_applied() {
        let mut app = App::new();
        let original_active_track = app.project.active_track_index;
        let original_mapping_len = app.mappings.len();
        app.apply_action(AppAction::ShowPage(AppPage::Mappings));
        app.apply_action(AppAction::ToggleMappingsWriteMode);

        app.apply_action(AppAction::SelectNextTrack);
        app.apply_action(AppAction::AddMappingRow);
        app.apply_action(AppAction::UndoMappings);

        assert_eq!(app.mappings.len(), original_mapping_len);
        assert_eq!(app.project.active_track_index, original_active_track + 1);
    }

    #[test]
    fn undo_timeline_leaves_ui_changes_applied() {
        let mut app = App::new();
        let original_page = app.page_state.current_page;
        let original_track = app.project.active_track_index;

        app.apply_action(AppAction::ShowNextPage);
        app.apply_action(AppAction::SelectNextTrack);
        app.apply_action(AppAction::UndoTimeline);

        assert_eq!(app.project.active_track_index, original_track);
        assert_eq!(app.page_state.current_page, original_page.next());
    }

    #[test]
    fn undo_ui_restores_page_navigation_without_touching_timeline() {
        let mut app = App::new();
        let original_page = app.page_state.current_page;

        app.apply_action(AppAction::SelectNextTrack);
        let changed_track = app.project.active_track_index;
        app.apply_action(AppAction::ShowNextPage);
        app.apply_action(AppAction::UndoUi);

        assert_eq!(app.page_state.current_page, original_page);
        assert_eq!(app.project.active_track_index, changed_track);
    }
}
