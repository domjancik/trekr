use crate::pages::AppPage;
use crate::ui::TimelineFlow;
use sdl3::event::Event;
use sdl3::keyboard::{Keycode, Mod};

/// The canonical application command layer.
///
/// All control surfaces should resolve into these actions before mutating app
/// state so inputs remain remappable and transport behavior stays consistent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppAction {
    Quit,
    ShowPage(AppPage),
    ShowNextPage,
    ShowPreviousPage,
    SelectPreviousPageItem,
    SelectNextPageItem,
    AdjustPageItemBackward,
    AdjustPageItemForward,
    ActivatePageItem,
    ReverseActivatePageItem,
    ToggleMappingsOverlay,
    ToggleDiscoverabilityOverlay,
    ToggleDirectMappingMode,
    ToggleMappingsWriteMode,
    AddMappingRow,
    RemoveSelectedMapping,
    DeletePageItem,
    SelectPreviousPageField,
    SelectNextPageField,
    TogglePlayback,
    ToggleRecording,
    CycleRecordMode,
    ToggleLoopRecordingExtension,
    ToggleLinkEnabled,
    ToggleLinkStartStopSync,
    ToggleGlobalLoop,
    ResetGlobalLoop,
    ClearCurrentTrackContent,
    ClearAllTrackContent,
    ToggleCurrentTrackLoop,
    ToggleStoredLoopRecallQuantize,
    CycleStoredLoopLaunchQuantize,
    SetCurrentTrackLoopStart,
    SetCurrentTrackLoopEnd,
    SetGlobalLoopStart,
    SetGlobalLoopEnd,
    NudgeCurrentTrackLoopBackward,
    NudgeCurrentTrackLoopForward,
    NudgeGlobalLoopBackward,
    NudgeGlobalLoopForward,
    ShortenCurrentTrackLoop,
    ExtendCurrentTrackLoop,
    HalfCurrentTrackLoop,
    DoubleCurrentTrackLoop,
    RecallStoredLoopSlot1,
    RecallStoredLoopSlot2,
    RecallStoredLoopSlot3,
    RecallStoredLoopSlot4,
    RecallStoredLoopSlot5,
    RecallStoredLoopSlot6,
    RecallStoredLoopSlot7,
    RecallStoredLoopSlot8,
    StoreCurrentLoopToSlot1,
    StoreCurrentLoopToSlot2,
    StoreCurrentLoopToSlot3,
    StoreCurrentLoopToSlot4,
    StoreCurrentLoopToSlot5,
    StoreCurrentLoopToSlot6,
    StoreCurrentLoopToSlot7,
    StoreCurrentLoopToSlot8,
    ClearStoredLoopSlot1,
    ClearStoredLoopSlot2,
    ClearStoredLoopSlot3,
    ClearStoredLoopSlot4,
    ClearStoredLoopSlot5,
    ClearStoredLoopSlot6,
    ClearStoredLoopSlot7,
    ClearStoredLoopSlot8,
    ShortenGlobalLoop,
    ExtendGlobalLoop,
    HalfGlobalLoop,
    DoubleGlobalLoop,
    ToggleCurrentTrackArm,
    ToggleCurrentTrackMute,
    ToggleCurrentTrackSolo,
    ToggleCurrentTrackPassthrough,
    ToggleCurrentTrackRecordingView,
    SelectRecordingClip(u64),
    SelectPreviousRecordingClip,
    SelectNextRecordingClip,
    ToggleSelectedRecordingClipMute,
    DeleteSelectedRecordingClip,
    ToggleSelectedTimelineFx,
    CycleSelectedTimelineFxKind,
    AdjustSelectedTimelineFxPrimary,
    AdjustSelectedTimelineFxSecondary,
    ScrollSelectedTimelineFxWindow,
    MoveSelectedTimelineFxUp,
    MoveSelectedTimelineFxDown,
    AddSelectedTimelineFx,
    DeleteSelectedTimelineFx,
    ToggleFocusedTrackView,
    SelectNextTrack,
    SelectPreviousTrack,
    SelectTrack(usize),
    SelectNotesAtPlayhead,
    SelectNotesAtPlayheadAdd,
    DeselectTrackNotes,
    SelectNextNote,
    SelectPreviousNote,
    FocusFirstSelectedNote,
    FocusLastSelectedNote,
    ExtendNoteSelectionForward,
    ExtendNoteSelectionBackward,
    ExtendNoteSelectionBoth,
    ContractNoteSelection,
    NudgeSelectedNotesEarlier,
    NudgeSelectedNotesLater,
    NudgeSelectedNotesUp,
    NudgeSelectedNotesDown,
    BeginNoteAdditiveSelectionHold,
    EndNoteAdditiveSelectionHold,
    StartRecording,
    StopRecording,
    SetTimelineFlow(TimelineFlow),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionSource {
    Keyboard,
    Pointer,
    Midi,
    Touch,
    Remote,
    Internal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActionEvent {
    pub action: AppAction,
    pub source: ActionSource,
}

impl ActionEvent {
    pub fn new(action: AppAction, source: ActionSource) -> Self {
        Self { action, source }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct KeyboardBindings;

impl KeyboardBindings {
    pub fn resolve(self, event: &Event) -> Option<ActionEvent> {
        match event {
            Event::Quit { .. } => Some(ActionEvent::new(AppAction::Quit, ActionSource::Keyboard)),
            Event::KeyDown {
                keycode: Some(Keycode::Escape),
                ..
            } => Some(ActionEvent::new(AppAction::Quit, ActionSource::Keyboard)),
            Event::KeyDown {
                keycode: Some(Keycode::T),
                keymod,
                repeat: false,
                ..
            } => Some(ActionEvent::new(
                if keymod.intersects(Mod::LSHIFTMOD | Mod::RSHIFTMOD) {
                    AppAction::SelectNotesAtPlayheadAdd
                } else {
                    AppAction::SelectNotesAtPlayhead
                },
                ActionSource::Keyboard,
            )),
            Event::KeyDown {
                keycode: Some(Keycode::V),
                keymod,
                repeat: false,
                ..
            } if keymod.intersects(Mod::LSHIFTMOD | Mod::RSHIFTMOD) => Some(ActionEvent::new(
                AppAction::ToggleCurrentTrackRecordingView,
                ActionSource::Keyboard,
            )),
            Event::KeyDown {
                keycode: Some(Keycode::V),
                repeat: false,
                ..
            } => Some(ActionEvent::new(
                AppAction::DeselectTrackNotes,
                ActionSource::Keyboard,
            )),
            Event::KeyDown {
                keycode: Some(Keycode::J),
                keymod,
                repeat: false,
                ..
            } if keymod.intersects(Mod::LSHIFTMOD | Mod::RSHIFTMOD) => Some(ActionEvent::new(
                AppAction::SelectPreviousRecordingClip,
                ActionSource::Keyboard,
            )),
            Event::KeyDown {
                keycode: Some(Keycode::J),
                repeat: false,
                ..
            } => Some(ActionEvent::new(
                AppAction::SelectPreviousNote,
                ActionSource::Keyboard,
            )),
            Event::KeyDown {
                keycode: Some(Keycode::K),
                keymod,
                repeat: false,
                ..
            } if keymod.intersects(Mod::LSHIFTMOD | Mod::RSHIFTMOD) => Some(ActionEvent::new(
                AppAction::SelectNextRecordingClip,
                ActionSource::Keyboard,
            )),
            Event::KeyDown {
                keycode: Some(Keycode::K),
                repeat: false,
                ..
            } => Some(ActionEvent::new(
                AppAction::SelectNextNote,
                ActionSource::Keyboard,
            )),
            Event::KeyDown {
                keycode: Some(Keycode::U),
                repeat: false,
                ..
            } => Some(ActionEvent::new(
                AppAction::FocusFirstSelectedNote,
                ActionSource::Keyboard,
            )),
            Event::KeyDown {
                keycode: Some(Keycode::O),
                repeat: false,
                ..
            } => Some(ActionEvent::new(
                AppAction::FocusLastSelectedNote,
                ActionSource::Keyboard,
            )),
            Event::KeyDown {
                keycode: Some(Keycode::H),
                repeat: false,
                ..
            } => Some(ActionEvent::new(
                AppAction::ExtendNoteSelectionBackward,
                ActionSource::Keyboard,
            )),
            Event::KeyDown {
                keycode: Some(Keycode::P),
                repeat: false,
                ..
            } => Some(ActionEvent::new(
                AppAction::ExtendNoteSelectionForward,
                ActionSource::Keyboard,
            )),
            Event::KeyDown {
                keycode: Some(Keycode::Y),
                repeat: false,
                ..
            } => Some(ActionEvent::new(
                AppAction::ExtendNoteSelectionBoth,
                ActionSource::Keyboard,
            )),
            Event::KeyDown {
                keycode: Some(Keycode::B),
                repeat: false,
                ..
            } => Some(ActionEvent::new(
                AppAction::ContractNoteSelection,
                ActionSource::Keyboard,
            )),
            Event::KeyDown {
                keycode: Some(Keycode::Z),
                repeat: false,
                ..
            } => Some(ActionEvent::new(
                AppAction::NudgeSelectedNotesEarlier,
                ActionSource::Keyboard,
            )),
            Event::KeyDown {
                keycode: Some(Keycode::X),
                repeat: false,
                ..
            } => Some(ActionEvent::new(
                AppAction::NudgeSelectedNotesLater,
                ActionSource::Keyboard,
            )),
            Event::KeyDown {
                keycode: Some(Keycode::D),
                repeat: false,
                ..
            } => Some(ActionEvent::new(
                AppAction::NudgeSelectedNotesDown,
                ActionSource::Keyboard,
            )),
            Event::KeyDown {
                keycode: Some(Keycode::F),
                repeat: false,
                ..
            } => Some(ActionEvent::new(
                AppAction::NudgeSelectedNotesUp,
                ActionSource::Keyboard,
            )),
            Event::KeyDown {
                keycode: Some(Keycode::Space),
                repeat: false,
                ..
            } => Some(ActionEvent::new(
                AppAction::TogglePlayback,
                ActionSource::Keyboard,
            )),
            Event::KeyDown {
                keycode: Some(Keycode::Tab),
                keymod,
                repeat: false,
                ..
            } => Some(ActionEvent::new(
                if keymod.intersects(Mod::LSHIFTMOD | Mod::RSHIFTMOD) {
                    AppAction::ShowPreviousPage
                } else {
                    AppAction::ShowNextPage
                },
                ActionSource::Keyboard,
            )),
            Event::KeyDown {
                keycode: Some(Keycode::F1),
                repeat: false,
                ..
            } => Some(ActionEvent::new(
                AppAction::ShowPage(AppPage::Timeline),
                ActionSource::Keyboard,
            )),
            Event::KeyDown {
                keycode: Some(Keycode::F2),
                repeat: false,
                ..
            } => Some(ActionEvent::new(
                AppAction::ShowPage(AppPage::Mappings),
                ActionSource::Keyboard,
            )),
            Event::KeyDown {
                keycode: Some(Keycode::F3),
                repeat: false,
                ..
            } => Some(ActionEvent::new(
                AppAction::ShowPage(AppPage::MidiIo),
                ActionSource::Keyboard,
            )),
            Event::KeyDown {
                keycode: Some(Keycode::F4),
                repeat: false,
                ..
            } => Some(ActionEvent::new(
                AppAction::ShowPage(AppPage::Routing),
                ActionSource::Keyboard,
            )),
            Event::KeyDown {
                keycode: Some(Keycode::F5),
                repeat: false,
                ..
            } => Some(ActionEvent::new(
                AppAction::ToggleMappingsOverlay,
                ActionSource::Keyboard,
            )),
            Event::KeyDown {
                keycode: Some(Keycode::F6),
                keymod,
                repeat: false,
                ..
            } => Some(ActionEvent::new(
                if keymod.intersects(Mod::LSHIFTMOD | Mod::RSHIFTMOD) {
                    AppAction::ToggleLinkStartStopSync
                } else {
                    AppAction::ToggleLinkEnabled
                },
                ActionSource::Keyboard,
            )),
            Event::KeyDown {
                keycode: Some(Keycode::F7),
                repeat: false,
                ..
            } => Some(ActionEvent::new(
                AppAction::ToggleDiscoverabilityOverlay,
                ActionSource::Keyboard,
            )),
            Event::KeyDown {
                keycode: Some(Keycode::F8),
                keymod,
                repeat: false,
                ..
            } => Some(ActionEvent::new(
                if keymod.intersects(Mod::LSHIFTMOD | Mod::RSHIFTMOD) {
                    AppAction::ToggleFocusedTrackView
                } else {
                    AppAction::ToggleDirectMappingMode
                },
                ActionSource::Keyboard,
            )),
            Event::KeyDown {
                keycode: Some(Keycode::G),
                repeat: false,
                ..
            } => Some(ActionEvent::new(
                AppAction::ToggleGlobalLoop,
                ActionSource::Keyboard,
            )),
            Event::KeyDown {
                keycode: Some(Keycode::W),
                repeat: false,
                ..
            } => Some(ActionEvent::new(
                AppAction::ToggleMappingsWriteMode,
                ActionSource::Keyboard,
            )),
            Event::KeyDown {
                keycode: Some(Keycode::N),
                repeat: false,
                ..
            } => Some(ActionEvent::new(
                AppAction::AddMappingRow,
                ActionSource::Keyboard,
            )),
            Event::KeyDown {
                keycode: Some(Keycode::R),
                keymod,
                repeat: false,
                ..
            } => Some(ActionEvent::new(
                if keymod.intersects(Mod::LSHIFTMOD | Mod::RSHIFTMOD) {
                    AppAction::CycleRecordMode
                } else {
                    AppAction::ToggleRecording
                },
                ActionSource::Keyboard,
            )),
            Event::KeyDown {
                keycode: Some(Keycode::Home),
                repeat: false,
                ..
            } => Some(ActionEvent::new(
                AppAction::ResetGlobalLoop,
                ActionSource::Keyboard,
            )),
            Event::KeyDown {
                keycode: Some(Keycode::C),
                keymod,
                repeat: false,
                ..
            } => Some(ActionEvent::new(
                if keymod.intersects(Mod::LSHIFTMOD | Mod::RSHIFTMOD) {
                    AppAction::ClearAllTrackContent
                } else {
                    AppAction::ClearCurrentTrackContent
                },
                ActionSource::Keyboard,
            )),
            Event::KeyDown {
                keycode: Some(Keycode::L),
                keymod,
                repeat: false,
                ..
            } => Some(ActionEvent::new(
                if keymod.intersects(Mod::LSHIFTMOD | Mod::RSHIFTMOD) {
                    AppAction::ToggleStoredLoopRecallQuantize
                } else {
                    AppAction::ToggleCurrentTrackLoop
                },
                ActionSource::Keyboard,
            )),
            Event::KeyDown {
                keycode: Some(Keycode::LeftBracket),
                keymod,
                repeat: false,
                ..
            } => Some(ActionEvent::new(
                if keymod.intersects(Mod::LSHIFTMOD | Mod::RSHIFTMOD) {
                    AppAction::SetGlobalLoopStart
                } else {
                    AppAction::SetCurrentTrackLoopStart
                },
                ActionSource::Keyboard,
            )),
            Event::KeyDown {
                keycode: Some(Keycode::RightBracket),
                keymod,
                repeat: false,
                ..
            } => Some(ActionEvent::new(
                if keymod.intersects(Mod::LSHIFTMOD | Mod::RSHIFTMOD) {
                    AppAction::SetGlobalLoopEnd
                } else {
                    AppAction::SetCurrentTrackLoopEnd
                },
                ActionSource::Keyboard,
            )),
            Event::KeyDown {
                keycode: Some(Keycode::Comma),
                keymod,
                repeat: false,
                ..
            } => Some(ActionEvent::new(
                if keymod.intersects(Mod::LSHIFTMOD | Mod::RSHIFTMOD) {
                    AppAction::NudgeGlobalLoopBackward
                } else {
                    AppAction::NudgeCurrentTrackLoopBackward
                },
                ActionSource::Keyboard,
            )),
            Event::KeyDown {
                keycode: Some(Keycode::Period),
                keymod,
                repeat: false,
                ..
            } => Some(ActionEvent::new(
                if keymod.intersects(Mod::LSHIFTMOD | Mod::RSHIFTMOD) {
                    AppAction::NudgeGlobalLoopForward
                } else {
                    AppAction::NudgeCurrentTrackLoopForward
                },
                ActionSource::Keyboard,
            )),
            Event::KeyDown {
                keycode: Some(Keycode::Minus),
                keymod,
                repeat: false,
                ..
            } => Some(ActionEvent::new(
                if keymod.intersects(Mod::LSHIFTMOD | Mod::RSHIFTMOD) {
                    AppAction::ShortenGlobalLoop
                } else {
                    AppAction::ShortenCurrentTrackLoop
                },
                ActionSource::Keyboard,
            )),
            Event::KeyDown {
                keycode: Some(Keycode::Equals),
                keymod,
                repeat: false,
                ..
            } => Some(ActionEvent::new(
                if keymod.intersects(Mod::LSHIFTMOD | Mod::RSHIFTMOD) {
                    AppAction::ExtendGlobalLoop
                } else {
                    AppAction::ExtendCurrentTrackLoop
                },
                ActionSource::Keyboard,
            )),
            Event::KeyDown {
                keycode: Some(Keycode::Slash),
                keymod,
                repeat: false,
                ..
            } => Some(ActionEvent::new(
                if keymod.intersects(Mod::LSHIFTMOD | Mod::RSHIFTMOD) {
                    AppAction::HalfGlobalLoop
                } else {
                    AppAction::HalfCurrentTrackLoop
                },
                ActionSource::Keyboard,
            )),
            Event::KeyDown {
                keycode: Some(Keycode::Backslash),
                keymod,
                repeat: false,
                ..
            } => Some(ActionEvent::new(
                if keymod.intersects(Mod::LSHIFTMOD | Mod::RSHIFTMOD) {
                    AppAction::DoubleGlobalLoop
                } else {
                    AppAction::DoubleCurrentTrackLoop
                },
                ActionSource::Keyboard,
            )),
            Event::KeyDown {
                keycode: Some(keycode),
                keymod,
                repeat: false,
                ..
            } if stored_loop_slot_shortcut(*keycode, *keymod).is_some() => Some(ActionEvent::new(
                stored_loop_slot_shortcut(*keycode, *keymod).expect("stored loop shortcut checked"),
                ActionSource::Keyboard,
            )),
            Event::KeyDown {
                keycode: Some(Keycode::A),
                repeat: false,
                ..
            } => Some(ActionEvent::new(
                AppAction::ToggleCurrentTrackArm,
                ActionSource::Keyboard,
            )),
            Event::KeyDown {
                keycode: Some(Keycode::M),
                keymod,
                repeat: false,
                ..
            } if keymod.intersects(Mod::LSHIFTMOD | Mod::RSHIFTMOD) => Some(ActionEvent::new(
                AppAction::ToggleSelectedRecordingClipMute,
                ActionSource::Keyboard,
            )),
            Event::KeyDown {
                keycode: Some(Keycode::M),
                repeat: false,
                ..
            } => Some(ActionEvent::new(
                AppAction::ToggleCurrentTrackMute,
                ActionSource::Keyboard,
            )),
            Event::KeyDown {
                keycode: Some(Keycode::S),
                repeat: false,
                ..
            } => Some(ActionEvent::new(
                AppAction::ToggleCurrentTrackSolo,
                ActionSource::Keyboard,
            )),
            Event::KeyDown {
                keycode: Some(Keycode::I),
                repeat: false,
                ..
            } => Some(ActionEvent::new(
                AppAction::ToggleCurrentTrackPassthrough,
                ActionSource::Keyboard,
            )),
            Event::KeyDown {
                keycode: Some(Keycode::Up),
                keymod,
                repeat: false,
                ..
            } => Some(ActionEvent::new(
                if keymod.intersects(Mod::LSHIFTMOD | Mod::RSHIFTMOD) {
                    AppAction::SelectPreviousPageField
                } else {
                    AppAction::SelectPreviousPageItem
                },
                ActionSource::Keyboard,
            )),
            Event::KeyDown {
                keycode: Some(Keycode::Down),
                keymod,
                repeat: false,
                ..
            } => Some(ActionEvent::new(
                if keymod.intersects(Mod::LSHIFTMOD | Mod::RSHIFTMOD) {
                    AppAction::SelectNextPageField
                } else {
                    AppAction::SelectNextPageItem
                },
                ActionSource::Keyboard,
            )),
            Event::KeyDown {
                keycode: Some(Keycode::Q),
                keymod,
                repeat: false,
                ..
            } => Some(ActionEvent::new(
                if keymod.intersects(Mod::LSHIFTMOD | Mod::RSHIFTMOD) {
                    AppAction::CycleStoredLoopLaunchQuantize
                } else {
                    AppAction::AdjustPageItemBackward
                },
                ActionSource::Keyboard,
            )),
            Event::KeyDown {
                keycode: Some(Keycode::E),
                repeat: false,
                ..
            } => Some(ActionEvent::new(
                AppAction::AdjustPageItemForward,
                ActionSource::Keyboard,
            )),
            Event::KeyDown {
                keycode: Some(Keycode::Return),
                keymod,
                repeat: false,
                ..
            } => Some(ActionEvent::new(
                if keymod.intersects(Mod::LSHIFTMOD | Mod::RSHIFTMOD) {
                    AppAction::ReverseActivatePageItem
                } else {
                    AppAction::ActivatePageItem
                },
                ActionSource::Keyboard,
            )),
            Event::KeyDown {
                keycode: Some(Keycode::Delete | Keycode::Backspace),
                keymod,
                repeat: false,
                ..
            } => Some(ActionEvent::new(
                if keymod.intersects(Mod::LSHIFTMOD | Mod::RSHIFTMOD) {
                    AppAction::DeleteSelectedRecordingClip
                } else {
                    AppAction::DeletePageItem
                },
                ActionSource::Keyboard,
            )),
            Event::KeyDown {
                keycode: Some(Keycode::Right),
                keymod,
                repeat: false,
                ..
            } => Some(ActionEvent::new(
                if keymod.intersects(Mod::LSHIFTMOD | Mod::RSHIFTMOD) {
                    AppAction::SelectNextPageField
                } else {
                    AppAction::SelectNextTrack
                },
                ActionSource::Keyboard,
            )),
            Event::KeyDown {
                keycode: Some(Keycode::Left),
                keymod,
                repeat: false,
                ..
            } => Some(ActionEvent::new(
                if keymod.intersects(Mod::LSHIFTMOD | Mod::RSHIFTMOD) {
                    AppAction::SelectPreviousPageField
                } else {
                    AppAction::SelectPreviousTrack
                },
                ActionSource::Keyboard,
            )),
            Event::KeyDown {
                keycode: Some(keycode),
                keymod,
                repeat: false,
                ..
            } if !keymod
                .intersects(Mod::LALTMOD | Mod::RALTMOD | Mod::LCTRLMOD | Mod::RCTRLMOD) =>
            {
                digit_track_index(*keycode).map(|index| {
                    ActionEvent::new(AppAction::SelectTrack(index), ActionSource::Keyboard)
                })
            }
            _ => None,
        }
    }
}

pub fn action_label(action: AppAction) -> &'static str {
    match action {
        AppAction::Quit => "Quit",
        AppAction::ShowPage(page) => match page {
            AppPage::Timeline => "Show Timeline",
            AppPage::Mappings => "Show Mappings",
            AppPage::MidiIo => "Show MIDI I/O",
            AppPage::Routing => "Show Routing",
        },
        AppAction::ShowNextPage => "Next Page",
        AppAction::ShowPreviousPage => "Previous Page",
        AppAction::SelectPreviousPageItem => "Previous Page Item",
        AppAction::SelectNextPageItem => "Next Page Item",
        AppAction::AdjustPageItemBackward => "Adjust Page Item Backward",
        AppAction::AdjustPageItemForward => "Adjust Page Item Forward",
        AppAction::ActivatePageItem => "Activate Page Item",
        AppAction::ReverseActivatePageItem => "Reverse Activate Page Item",
        AppAction::ToggleMappingsOverlay => "Mappings Overlay",
        AppAction::ToggleDiscoverabilityOverlay => "Mapping Discoverability",
        AppAction::ToggleDirectMappingMode => "Direct Mapping Mode",
        AppAction::ToggleMappingsWriteMode => "Mappings Write Mode",
        AppAction::AddMappingRow => "Add Mapping",
        AppAction::RemoveSelectedMapping => "Remove Mapping",
        AppAction::DeletePageItem => "Delete Page Item",
        AppAction::SelectPreviousPageField => "Previous Mapping Field",
        AppAction::SelectNextPageField => "Next Mapping Field",
        AppAction::TogglePlayback => "Play/Stop",
        AppAction::ToggleRecording => "Record",
        AppAction::CycleRecordMode => "Record Mode",
        AppAction::ToggleLoopRecordingExtension => "Loop Recording Wrap",
        AppAction::ToggleLinkEnabled => "Link Enable",
        AppAction::ToggleLinkStartStopSync => "Link Start/Stop",
        AppAction::ToggleGlobalLoop => "Song Loop",
        AppAction::ResetGlobalLoop => "Reset Song Loop",
        AppAction::ClearCurrentTrackContent => "Clear Track",
        AppAction::ClearAllTrackContent => "Clear All",
        AppAction::ToggleCurrentTrackLoop => "Track Loop",
        AppAction::ToggleStoredLoopRecallQuantize => "Stored Loop Recall Quantize",
        AppAction::CycleStoredLoopLaunchQuantize => "Stored Loop Launch Quantize",
        AppAction::SetCurrentTrackLoopStart => "Set Track Loop Start",
        AppAction::SetCurrentTrackLoopEnd => "Set Track Loop End",
        AppAction::SetGlobalLoopStart => "Set Song Loop Start",
        AppAction::SetGlobalLoopEnd => "Set Song Loop End",
        AppAction::NudgeCurrentTrackLoopBackward => "Nudge Track Loop Back",
        AppAction::NudgeCurrentTrackLoopForward => "Nudge Track Loop Forward",
        AppAction::NudgeGlobalLoopBackward => "Nudge Song Loop Back",
        AppAction::NudgeGlobalLoopForward => "Nudge Song Loop Forward",
        AppAction::ShortenCurrentTrackLoop => "Shorten Track Loop",
        AppAction::ExtendCurrentTrackLoop => "Extend Track Loop",
        AppAction::HalfCurrentTrackLoop => "Half Track Loop",
        AppAction::DoubleCurrentTrackLoop => "Double Track Loop",
        AppAction::RecallStoredLoopSlot1 => "Recall Stored Loop Slot 1",
        AppAction::RecallStoredLoopSlot2 => "Recall Stored Loop Slot 2",
        AppAction::RecallStoredLoopSlot3 => "Recall Stored Loop Slot 3",
        AppAction::RecallStoredLoopSlot4 => "Recall Stored Loop Slot 4",
        AppAction::RecallStoredLoopSlot5 => "Recall Stored Loop Slot 5",
        AppAction::RecallStoredLoopSlot6 => "Recall Stored Loop Slot 6",
        AppAction::RecallStoredLoopSlot7 => "Recall Stored Loop Slot 7",
        AppAction::RecallStoredLoopSlot8 => "Recall Stored Loop Slot 8",
        AppAction::StoreCurrentLoopToSlot1 => "Store Current Loop To Slot 1",
        AppAction::StoreCurrentLoopToSlot2 => "Store Current Loop To Slot 2",
        AppAction::StoreCurrentLoopToSlot3 => "Store Current Loop To Slot 3",
        AppAction::StoreCurrentLoopToSlot4 => "Store Current Loop To Slot 4",
        AppAction::StoreCurrentLoopToSlot5 => "Store Current Loop To Slot 5",
        AppAction::StoreCurrentLoopToSlot6 => "Store Current Loop To Slot 6",
        AppAction::StoreCurrentLoopToSlot7 => "Store Current Loop To Slot 7",
        AppAction::StoreCurrentLoopToSlot8 => "Store Current Loop To Slot 8",
        AppAction::ClearStoredLoopSlot1 => "Clear Stored Loop Slot 1",
        AppAction::ClearStoredLoopSlot2 => "Clear Stored Loop Slot 2",
        AppAction::ClearStoredLoopSlot3 => "Clear Stored Loop Slot 3",
        AppAction::ClearStoredLoopSlot4 => "Clear Stored Loop Slot 4",
        AppAction::ClearStoredLoopSlot5 => "Clear Stored Loop Slot 5",
        AppAction::ClearStoredLoopSlot6 => "Clear Stored Loop Slot 6",
        AppAction::ClearStoredLoopSlot7 => "Clear Stored Loop Slot 7",
        AppAction::ClearStoredLoopSlot8 => "Clear Stored Loop Slot 8",
        AppAction::ShortenGlobalLoop => "Shorten Song Loop",
        AppAction::ExtendGlobalLoop => "Extend Song Loop",
        AppAction::HalfGlobalLoop => "Half Song Loop",
        AppAction::DoubleGlobalLoop => "Double Song Loop",
        AppAction::ToggleCurrentTrackArm => "Track Arm",
        AppAction::ToggleCurrentTrackMute => "Track Mute",
        AppAction::ToggleCurrentTrackSolo => "Track Solo",
        AppAction::ToggleCurrentTrackPassthrough => "Passthrough",
        AppAction::ToggleCurrentTrackRecordingView => "Recording View",
        AppAction::SelectRecordingClip(_) => "Select Recording Clip",
        AppAction::SelectPreviousRecordingClip => "Previous Recording Clip",
        AppAction::SelectNextRecordingClip => "Next Recording Clip",
        AppAction::ToggleSelectedRecordingClipMute => "Recording Clip Mute",
        AppAction::DeleteSelectedRecordingClip => "Delete Recording Clip",
        AppAction::ToggleSelectedTimelineFx => "Toggle Timeline FX",
        AppAction::CycleSelectedTimelineFxKind => "Cycle Timeline FX Kind",
        AppAction::AdjustSelectedTimelineFxPrimary => "Adjust Timeline FX Param 1",
        AppAction::AdjustSelectedTimelineFxSecondary => "Adjust Timeline FX Param 2",
        AppAction::ScrollSelectedTimelineFxWindow => "Scroll Timeline FX Params",
        AppAction::MoveSelectedTimelineFxUp => "Move Timeline FX Up",
        AppAction::MoveSelectedTimelineFxDown => "Move Timeline FX Down",
        AppAction::AddSelectedTimelineFx => "Add Timeline FX",
        AppAction::DeleteSelectedTimelineFx => "Delete Timeline FX",
        AppAction::ToggleFocusedTrackView => "Focused Track View",
        AppAction::SelectNextTrack => "Next Track",
        AppAction::SelectPreviousTrack => "Previous Track",
        AppAction::SelectTrack(_) => "Select Track",
        AppAction::SelectNotesAtPlayhead => "Select Notes At Playhead",
        AppAction::SelectNotesAtPlayheadAdd => "Add Notes At Playhead",
        AppAction::DeselectTrackNotes => "Deselect Track Notes",
        AppAction::SelectNextNote => "Select Next Note",
        AppAction::SelectPreviousNote => "Select Previous Note",
        AppAction::FocusFirstSelectedNote => "Focus First Selected Note",
        AppAction::FocusLastSelectedNote => "Focus Last Selected Note",
        AppAction::ExtendNoteSelectionForward => "Extend Note Selection Forward",
        AppAction::ExtendNoteSelectionBackward => "Extend Note Selection Backward",
        AppAction::ExtendNoteSelectionBoth => "Extend Note Selection Both",
        AppAction::ContractNoteSelection => "Contract Note Selection",
        AppAction::NudgeSelectedNotesEarlier => "Nudge Selected Notes Earlier",
        AppAction::NudgeSelectedNotesLater => "Nudge Selected Notes Later",
        AppAction::NudgeSelectedNotesUp => "Nudge Selected Notes Up",
        AppAction::NudgeSelectedNotesDown => "Nudge Selected Notes Down",
        AppAction::BeginNoteAdditiveSelectionHold => "Begin Note Additive Select",
        AppAction::EndNoteAdditiveSelectionHold => "End Note Additive Select",
        AppAction::StartRecording => "Start Recording",
        AppAction::StopRecording => "Stop Recording",
        AppAction::SetTimelineFlow(_) => "Timeline Flow",
    }
}

pub fn built_in_keyboard_binding_labels(action: AppAction) -> &'static [&'static str] {
    match action {
        AppAction::ShowPage(AppPage::Timeline) => &["F1"],
        AppAction::ShowPage(AppPage::Mappings) => &["F2"],
        AppAction::ShowPage(AppPage::MidiIo) => &["F3"],
        AppAction::ShowPage(AppPage::Routing) => &["F4"],
        AppAction::ShowNextPage => &["Tab"],
        AppAction::ShowPreviousPage => &["Shift+Tab"],
        AppAction::SelectPreviousPageItem => &["Up"],
        AppAction::SelectNextPageItem => &["Down"],
        AppAction::AdjustPageItemBackward => &["Q"],
        AppAction::AdjustPageItemForward => &["E"],
        AppAction::ActivatePageItem => &["Enter"],
        AppAction::ReverseActivatePageItem => &["Shift+Enter"],
        AppAction::ToggleMappingsOverlay => &["F5"],
        AppAction::ToggleDiscoverabilityOverlay => &["F7"],
        AppAction::ToggleDirectMappingMode => &["F8"],
        AppAction::ToggleMappingsWriteMode => &["W"],
        AppAction::AddMappingRow => &["N"],
        AppAction::RemoveSelectedMapping => &["Delete", "Backspace"],
        AppAction::DeletePageItem => &["Delete"],
        AppAction::SelectPreviousPageField => &["Shift+Left", "Shift+Up"],
        AppAction::SelectNextPageField => &["Shift+Right", "Shift+Down"],
        AppAction::TogglePlayback => &["Space"],
        AppAction::ToggleRecording => &["R"],
        AppAction::CycleRecordMode => &["Shift+R"],
        AppAction::ToggleLoopRecordingExtension => &[],
        AppAction::ToggleLinkEnabled => &["F6"],
        AppAction::ToggleLinkStartStopSync => &["Shift+F6"],
        AppAction::ToggleGlobalLoop => &["G"],
        AppAction::ResetGlobalLoop => &["Home"],
        AppAction::ClearCurrentTrackContent => &["C"],
        AppAction::ClearAllTrackContent => &["Shift+C"],
        AppAction::ToggleCurrentTrackLoop => &["L"],
        AppAction::ToggleStoredLoopRecallQuantize => &["Shift+L"],
        AppAction::CycleStoredLoopLaunchQuantize => &["Shift+Q"],
        AppAction::SetCurrentTrackLoopStart => &["["],
        AppAction::SetCurrentTrackLoopEnd => &["]"],
        AppAction::SetGlobalLoopStart => &["Shift+["],
        AppAction::SetGlobalLoopEnd => &["Shift+]"],
        AppAction::NudgeCurrentTrackLoopBackward => &[","],
        AppAction::NudgeCurrentTrackLoopForward => &["."],
        AppAction::NudgeGlobalLoopBackward => &["Shift+,"],
        AppAction::NudgeGlobalLoopForward => &["Shift+."],
        AppAction::ShortenCurrentTrackLoop => &["-"],
        AppAction::ExtendCurrentTrackLoop => &["="],
        AppAction::HalfCurrentTrackLoop => &["/"],
        AppAction::DoubleCurrentTrackLoop => &["\\"],
        AppAction::RecallStoredLoopSlot1 => &["Numpad1", "Alt+1"],
        AppAction::RecallStoredLoopSlot2 => &["Numpad2", "Alt+2"],
        AppAction::RecallStoredLoopSlot3 => &["Numpad3", "Alt+3"],
        AppAction::RecallStoredLoopSlot4 => &["Numpad4", "Alt+4"],
        AppAction::RecallStoredLoopSlot5 => &["Numpad5", "Alt+5"],
        AppAction::RecallStoredLoopSlot6 => &["Numpad6", "Alt+6"],
        AppAction::RecallStoredLoopSlot7 => &["Numpad7", "Alt+7"],
        AppAction::RecallStoredLoopSlot8 => &["Numpad8", "Alt+8"],
        AppAction::StoreCurrentLoopToSlot1 => &["Shift+Numpad1", "Shift+Alt+1"],
        AppAction::StoreCurrentLoopToSlot2 => &["Shift+Numpad2", "Shift+Alt+2"],
        AppAction::StoreCurrentLoopToSlot3 => &["Shift+Numpad3", "Shift+Alt+3"],
        AppAction::StoreCurrentLoopToSlot4 => &["Shift+Numpad4", "Shift+Alt+4"],
        AppAction::StoreCurrentLoopToSlot5 => &["Shift+Numpad5", "Shift+Alt+5"],
        AppAction::StoreCurrentLoopToSlot6 => &["Shift+Numpad6", "Shift+Alt+6"],
        AppAction::StoreCurrentLoopToSlot7 => &["Shift+Numpad7", "Shift+Alt+7"],
        AppAction::StoreCurrentLoopToSlot8 => &["Shift+Numpad8", "Shift+Alt+8"],
        AppAction::ClearStoredLoopSlot1 => &["Ctrl+Numpad1", "Ctrl+Alt+1"],
        AppAction::ClearStoredLoopSlot2 => &["Ctrl+Numpad2", "Ctrl+Alt+2"],
        AppAction::ClearStoredLoopSlot3 => &["Ctrl+Numpad3", "Ctrl+Alt+3"],
        AppAction::ClearStoredLoopSlot4 => &["Ctrl+Numpad4", "Ctrl+Alt+4"],
        AppAction::ClearStoredLoopSlot5 => &["Ctrl+Numpad5", "Ctrl+Alt+5"],
        AppAction::ClearStoredLoopSlot6 => &["Ctrl+Numpad6", "Ctrl+Alt+6"],
        AppAction::ClearStoredLoopSlot7 => &["Ctrl+Numpad7", "Ctrl+Alt+7"],
        AppAction::ClearStoredLoopSlot8 => &["Ctrl+Numpad8", "Ctrl+Alt+8"],
        AppAction::ShortenGlobalLoop => &["Shift+-"],
        AppAction::ExtendGlobalLoop => &["Shift+="],
        AppAction::HalfGlobalLoop => &["Shift+/"],
        AppAction::DoubleGlobalLoop => &["Shift+\\"],
        AppAction::ToggleCurrentTrackArm => &["A"],
        AppAction::ToggleCurrentTrackMute => &["M"],
        AppAction::ToggleCurrentTrackSolo => &["S"],
        AppAction::ToggleCurrentTrackPassthrough => &["I"],
        AppAction::ToggleCurrentTrackRecordingView => &["Shift+V"],
        AppAction::SelectRecordingClip(_) => &[],
        AppAction::SelectPreviousRecordingClip => &["Shift+J"],
        AppAction::SelectNextRecordingClip => &["Shift+K"],
        AppAction::ToggleSelectedRecordingClipMute => &["Shift+M"],
        AppAction::DeleteSelectedRecordingClip => &["Shift+Delete", "Shift+Backspace"],
        AppAction::ToggleSelectedTimelineFx => &["Shift+M"],
        AppAction::CycleSelectedTimelineFxKind => &["Q/E"],
        AppAction::AdjustSelectedTimelineFxPrimary => &["Q/E"],
        AppAction::AdjustSelectedTimelineFxSecondary => &["Q/E"],
        AppAction::ScrollSelectedTimelineFxWindow => &["Q/E"],
        AppAction::MoveSelectedTimelineFxUp => &["Q"],
        AppAction::MoveSelectedTimelineFxDown => &["E"],
        AppAction::AddSelectedTimelineFx => &["Q/E"],
        AppAction::DeleteSelectedTimelineFx => &["Delete"],
        AppAction::ToggleFocusedTrackView => &["Shift+F8"],
        AppAction::SelectNextTrack => &["Right"],
        AppAction::SelectPreviousTrack => &["Left"],
        AppAction::SelectTrack(_) => &["1-9"],
        AppAction::SelectNotesAtPlayhead => &["T"],
        AppAction::SelectNotesAtPlayheadAdd => &["Shift+T"],
        AppAction::DeselectTrackNotes => &["V"],
        AppAction::SelectNextNote => &["K"],
        AppAction::SelectPreviousNote => &["J"],
        AppAction::FocusFirstSelectedNote => &["U"],
        AppAction::FocusLastSelectedNote => &["O"],
        AppAction::ExtendNoteSelectionForward => &["P"],
        AppAction::ExtendNoteSelectionBackward => &["H"],
        AppAction::ExtendNoteSelectionBoth => &["Y"],
        AppAction::ContractNoteSelection => &["B"],
        AppAction::NudgeSelectedNotesEarlier => &["Z"],
        AppAction::NudgeSelectedNotesLater => &["X"],
        AppAction::NudgeSelectedNotesUp => &["F"],
        AppAction::NudgeSelectedNotesDown => &["D"],
        AppAction::BeginNoteAdditiveSelectionHold => &[],
        AppAction::EndNoteAdditiveSelectionHold => &[],
        AppAction::StartRecording => &[],
        AppAction::StopRecording => &[],
        AppAction::Quit => &["Escape"],
        AppAction::SetTimelineFlow(_) => &[],
    }
}

fn digit_track_index(keycode: Keycode) -> Option<usize> {
    match keycode {
        Keycode::_1 => Some(0),
        Keycode::_2 => Some(1),
        Keycode::_3 => Some(2),
        Keycode::_4 => Some(3),
        Keycode::_5 => Some(4),
        Keycode::_6 => Some(5),
        Keycode::_7 => Some(6),
        Keycode::_8 => Some(7),
        Keycode::_9 => Some(8),
        _ => None,
    }
}

fn stored_loop_slot_shortcut(keycode: Keycode, keymod: Mod) -> Option<AppAction> {
    let ctrl = keymod.intersects(Mod::LCTRLMOD | Mod::RCTRLMOD);
    let alt = keymod.intersects(Mod::LALTMOD | Mod::RALTMOD);
    let shift = keymod.intersects(Mod::LSHIFTMOD | Mod::RSHIFTMOD);

    if !ctrl && !alt && !shift {
        return stored_loop_shortcut_slot_from_numpad(keycode)
            .and_then(recall_stored_loop_slot_action);
    }

    if !ctrl && alt && !shift {
        return stored_loop_shortcut_slot_from_digit(keycode)
            .and_then(recall_stored_loop_slot_action);
    }

    if !ctrl && !alt && shift {
        return stored_loop_shortcut_slot_from_numpad(keycode)
            .and_then(store_stored_loop_slot_action);
    }

    if !ctrl && alt && shift {
        return stored_loop_shortcut_slot_from_digit(keycode)
            .and_then(store_stored_loop_slot_action);
    }

    if ctrl && !alt && !shift {
        return stored_loop_shortcut_slot_from_numpad(keycode)
            .and_then(clear_stored_loop_slot_action);
    }

    if ctrl && alt && !shift {
        return stored_loop_shortcut_slot_from_digit(keycode)
            .and_then(clear_stored_loop_slot_action);
    }

    None
}

fn stored_loop_shortcut_slot_from_numpad(keycode: Keycode) -> Option<usize> {
    match keycode {
        Keycode::Kp1 => Some(0),
        Keycode::Kp2 => Some(1),
        Keycode::Kp3 => Some(2),
        Keycode::Kp4 => Some(3),
        Keycode::Kp5 => Some(4),
        Keycode::Kp6 => Some(5),
        Keycode::Kp7 => Some(6),
        Keycode::Kp8 => Some(7),
        _ => None,
    }
}

fn stored_loop_shortcut_slot_from_digit(keycode: Keycode) -> Option<usize> {
    match keycode {
        Keycode::_1 => Some(0),
        Keycode::_2 => Some(1),
        Keycode::_3 => Some(2),
        Keycode::_4 => Some(3),
        Keycode::_5 => Some(4),
        Keycode::_6 => Some(5),
        Keycode::_7 => Some(6),
        Keycode::_8 => Some(7),
        _ => None,
    }
}

fn recall_stored_loop_slot_action(slot: usize) -> Option<AppAction> {
    match slot {
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

fn store_stored_loop_slot_action(slot: usize) -> Option<AppAction> {
    match slot {
        0 => Some(AppAction::StoreCurrentLoopToSlot1),
        1 => Some(AppAction::StoreCurrentLoopToSlot2),
        2 => Some(AppAction::StoreCurrentLoopToSlot3),
        3 => Some(AppAction::StoreCurrentLoopToSlot4),
        4 => Some(AppAction::StoreCurrentLoopToSlot5),
        5 => Some(AppAction::StoreCurrentLoopToSlot6),
        6 => Some(AppAction::StoreCurrentLoopToSlot7),
        7 => Some(AppAction::StoreCurrentLoopToSlot8),
        _ => None,
    }
}

fn clear_stored_loop_slot_action(slot: usize) -> Option<AppAction> {
    match slot {
        0 => Some(AppAction::ClearStoredLoopSlot1),
        1 => Some(AppAction::ClearStoredLoopSlot2),
        2 => Some(AppAction::ClearStoredLoopSlot3),
        3 => Some(AppAction::ClearStoredLoopSlot4),
        4 => Some(AppAction::ClearStoredLoopSlot5),
        5 => Some(AppAction::ClearStoredLoopSlot6),
        6 => Some(AppAction::ClearStoredLoopSlot7),
        7 => Some(AppAction::ClearStoredLoopSlot8),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{ActionSource, AppAction, KeyboardBindings};
    use crate::pages::AppPage;
    use sdl3::event::Event;
    use sdl3::keyboard::{Keycode, Mod};

    #[test]
    fn keyboard_bindings_map_escape_to_quit() {
        let event = Event::KeyDown {
            timestamp: 0,
            window_id: 0,
            keycode: Some(Keycode::Escape),
            scancode: None,
            keymod: Mod::NOMOD,
            repeat: false,
            which: 0,
            raw: 0,
        };

        let resolved = KeyboardBindings.resolve(&event).expect("quit action");
        assert_eq!(resolved.action, AppAction::Quit);
        assert_eq!(resolved.source, ActionSource::Keyboard);
    }

    #[test]
    fn keyboard_bindings_ignore_repeated_space() {
        let event = Event::KeyDown {
            timestamp: 0,
            window_id: 0,
            keycode: Some(Keycode::Space),
            scancode: None,
            keymod: Mod::NOMOD,
            repeat: true,
            which: 0,
            raw: 0,
        };

        assert!(KeyboardBindings.resolve(&event).is_none());
    }

    #[test]
    fn keyboard_bindings_map_number_keys_to_absolute_tracks() {
        let event = Event::KeyDown {
            timestamp: 0,
            window_id: 0,
            keycode: Some(Keycode::_4),
            scancode: None,
            keymod: Mod::NOMOD,
            repeat: false,
            which: 0,
            raw: 0,
        };

        let resolved = KeyboardBindings.resolve(&event).expect("track select");
        assert_eq!(resolved.action, AppAction::SelectTrack(3));
    }

    #[test]
    fn keyboard_bindings_map_stored_loop_recall_and_store_shortcuts() {
        let numpad = Event::KeyDown {
            timestamp: 0,
            window_id: 0,
            keycode: Some(Keycode::Kp3),
            scancode: None,
            keymod: Mod::NOMOD,
            repeat: false,
            which: 0,
            raw: 0,
        };
        let fallback = Event::KeyDown {
            timestamp: 0,
            window_id: 0,
            keycode: Some(Keycode::_7),
            scancode: None,
            keymod: Mod::LALTMOD,
            repeat: false,
            which: 0,
            raw: 0,
        };
        let store_numpad = Event::KeyDown {
            timestamp: 0,
            window_id: 0,
            keycode: Some(Keycode::Kp2),
            scancode: None,
            keymod: Mod::LSHIFTMOD,
            repeat: false,
            which: 0,
            raw: 0,
        };
        let store_fallback = Event::KeyDown {
            timestamp: 0,
            window_id: 0,
            keycode: Some(Keycode::_6),
            scancode: None,
            keymod: Mod::LSHIFTMOD | Mod::LALTMOD,
            repeat: false,
            which: 0,
            raw: 0,
        };
        let clear_numpad = Event::KeyDown {
            timestamp: 0,
            window_id: 0,
            keycode: Some(Keycode::Kp4),
            scancode: None,
            keymod: Mod::LCTRLMOD,
            repeat: false,
            which: 0,
            raw: 0,
        };
        let clear_fallback = Event::KeyDown {
            timestamp: 0,
            window_id: 0,
            keycode: Some(Keycode::_5),
            scancode: None,
            keymod: Mod::LCTRLMOD | Mod::LALTMOD,
            repeat: false,
            which: 0,
            raw: 0,
        };

        assert_eq!(
            KeyboardBindings.resolve(&numpad).unwrap().action,
            AppAction::RecallStoredLoopSlot3
        );
        assert_eq!(
            KeyboardBindings.resolve(&fallback).unwrap().action,
            AppAction::RecallStoredLoopSlot7
        );
        assert_eq!(
            KeyboardBindings.resolve(&store_numpad).unwrap().action,
            AppAction::StoreCurrentLoopToSlot2
        );
        assert_eq!(
            KeyboardBindings.resolve(&store_fallback).unwrap().action,
            AppAction::StoreCurrentLoopToSlot6
        );
        assert_eq!(
            KeyboardBindings.resolve(&clear_numpad).unwrap().action,
            AppAction::ClearStoredLoopSlot4
        );
        assert_eq!(
            KeyboardBindings.resolve(&clear_fallback).unwrap().action,
            AppAction::ClearStoredLoopSlot5
        );
    }

    #[test]
    fn keyboard_bindings_map_page_shortcuts() {
        let next = Event::KeyDown {
            timestamp: 0,
            window_id: 0,
            keycode: Some(Keycode::Tab),
            scancode: None,
            keymod: Mod::NOMOD,
            repeat: false,
            which: 0,
            raw: 0,
        };
        let direct = Event::KeyDown {
            timestamp: 0,
            window_id: 0,
            keycode: Some(Keycode::F3),
            scancode: None,
            keymod: Mod::NOMOD,
            repeat: false,
            which: 0,
            raw: 0,
        };

        assert_eq!(
            KeyboardBindings.resolve(&next).unwrap().action,
            AppAction::ShowNextPage
        );
        assert_eq!(
            KeyboardBindings.resolve(&direct).unwrap().action,
            AppAction::ShowPage(AppPage::MidiIo)
        );
    }

    #[test]
    fn keyboard_bindings_map_mappings_overlay_and_write_mode() {
        let overlay = Event::KeyDown {
            timestamp: 0,
            window_id: 0,
            keycode: Some(Keycode::F5),
            scancode: None,
            keymod: Mod::NOMOD,
            repeat: false,
            which: 0,
            raw: 0,
        };
        let write_mode = Event::KeyDown {
            timestamp: 0,
            window_id: 0,
            keycode: Some(Keycode::W),
            scancode: None,
            keymod: Mod::NOMOD,
            repeat: false,
            which: 0,
            raw: 0,
        };
        let discoverability = Event::KeyDown {
            timestamp: 0,
            window_id: 0,
            keycode: Some(Keycode::F7),
            scancode: None,
            keymod: Mod::NOMOD,
            repeat: false,
            which: 0,
            raw: 0,
        };
        let direct_mapping = Event::KeyDown {
            timestamp: 0,
            window_id: 0,
            keycode: Some(Keycode::F8),
            scancode: None,
            keymod: Mod::NOMOD,
            repeat: false,
            which: 0,
            raw: 0,
        };
        let add_mapping = Event::KeyDown {
            timestamp: 0,
            window_id: 0,
            keycode: Some(Keycode::N),
            scancode: None,
            keymod: Mod::NOMOD,
            repeat: false,
            which: 0,
            raw: 0,
        };
        let delete_mapping = Event::KeyDown {
            timestamp: 0,
            window_id: 0,
            keycode: Some(Keycode::Delete),
            scancode: None,
            keymod: Mod::NOMOD,
            repeat: false,
            which: 0,
            raw: 0,
        };
        let backspace_mapping = Event::KeyDown {
            timestamp: 0,
            window_id: 0,
            keycode: Some(Keycode::Backspace),
            scancode: None,
            keymod: Mod::NOMOD,
            repeat: false,
            which: 0,
            raw: 0,
        };

        assert_eq!(
            KeyboardBindings.resolve(&overlay).unwrap().action,
            AppAction::ToggleMappingsOverlay
        );
        assert_eq!(
            KeyboardBindings.resolve(&write_mode).unwrap().action,
            AppAction::ToggleMappingsWriteMode
        );
        assert_eq!(
            KeyboardBindings.resolve(&discoverability).unwrap().action,
            AppAction::ToggleDiscoverabilityOverlay
        );
        assert_eq!(
            KeyboardBindings.resolve(&direct_mapping).unwrap().action,
            AppAction::ToggleDirectMappingMode
        );
        assert_eq!(
            KeyboardBindings.resolve(&add_mapping).unwrap().action,
            AppAction::AddMappingRow
        );
        assert_eq!(
            KeyboardBindings.resolve(&delete_mapping).unwrap().action,
            AppAction::DeletePageItem
        );
        assert_eq!(
            KeyboardBindings.resolve(&backspace_mapping).unwrap().action,
            AppAction::RemoveSelectedMapping
        );
    }

    #[test]
    fn keyboard_bindings_map_page_navigation_controls() {
        let up = Event::KeyDown {
            timestamp: 0,
            window_id: 0,
            keycode: Some(Keycode::Up),
            scancode: None,
            keymod: Mod::NOMOD,
            repeat: false,
            which: 0,
            raw: 0,
        };
        let adjust = Event::KeyDown {
            timestamp: 0,
            window_id: 0,
            keycode: Some(Keycode::E),
            scancode: None,
            keymod: Mod::NOMOD,
            repeat: false,
            which: 0,
            raw: 0,
        };

        assert_eq!(
            KeyboardBindings.resolve(&up).unwrap().action,
            AppAction::SelectPreviousPageItem
        );
        assert_eq!(
            KeyboardBindings.resolve(&adjust).unwrap().action,
            AppAction::AdjustPageItemForward
        );
    }

    #[test]
    fn keyboard_bindings_map_shift_up_down_to_page_field_navigation() {
        let shift_up = Event::KeyDown {
            timestamp: 0,
            window_id: 0,
            keycode: Some(Keycode::Up),
            scancode: None,
            keymod: Mod::LSHIFTMOD,
            repeat: false,
            which: 0,
            raw: 0,
        };
        let shift_down = Event::KeyDown {
            timestamp: 0,
            window_id: 0,
            keycode: Some(Keycode::Down),
            scancode: None,
            keymod: Mod::LSHIFTMOD,
            repeat: false,
            which: 0,
            raw: 0,
        };

        assert_eq!(
            KeyboardBindings.resolve(&shift_up).unwrap().action,
            AppAction::SelectPreviousPageField
        );
        assert_eq!(
            KeyboardBindings.resolve(&shift_down).unwrap().action,
            AppAction::SelectNextPageField
        );
    }

    #[test]
    fn keyboard_bindings_map_shift_enter_to_reverse_activate_page_item() {
        let shift_enter = Event::KeyDown {
            timestamp: 0,
            window_id: 0,
            keycode: Some(Keycode::Return),
            scancode: None,
            keymod: Mod::LSHIFTMOD,
            repeat: false,
            which: 0,
            raw: 0,
        };

        assert_eq!(
            KeyboardBindings.resolve(&shift_enter).unwrap().action,
            AppAction::ReverseActivatePageItem
        );
    }

    #[test]
    fn keyboard_bindings_map_note_edit_controls() {
        let select = Event::KeyDown {
            timestamp: 0,
            window_id: 0,
            keycode: Some(Keycode::T),
            scancode: None,
            keymod: Mod::NOMOD,
            repeat: false,
            which: 0,
            raw: 0,
        };
        let add = Event::KeyDown {
            timestamp: 0,
            window_id: 0,
            keycode: Some(Keycode::T),
            scancode: None,
            keymod: Mod::LSHIFTMOD,
            repeat: false,
            which: 0,
            raw: 0,
        };
        let nudge = Event::KeyDown {
            timestamp: 0,
            window_id: 0,
            keycode: Some(Keycode::F),
            scancode: None,
            keymod: Mod::NOMOD,
            repeat: false,
            which: 0,
            raw: 0,
        };
        let view = Event::KeyDown {
            timestamp: 0,
            window_id: 0,
            keycode: Some(Keycode::V),
            scancode: None,
            keymod: Mod::LSHIFTMOD,
            repeat: false,
            which: 0,
            raw: 0,
        };
        let prev_clip = Event::KeyDown {
            timestamp: 0,
            window_id: 0,
            keycode: Some(Keycode::J),
            scancode: None,
            keymod: Mod::LSHIFTMOD,
            repeat: false,
            which: 0,
            raw: 0,
        };
        let next_clip = Event::KeyDown {
            timestamp: 0,
            window_id: 0,
            keycode: Some(Keycode::K),
            scancode: None,
            keymod: Mod::LSHIFTMOD,
            repeat: false,
            which: 0,
            raw: 0,
        };
        let mute_clip = Event::KeyDown {
            timestamp: 0,
            window_id: 0,
            keycode: Some(Keycode::M),
            scancode: None,
            keymod: Mod::LSHIFTMOD,
            repeat: false,
            which: 0,
            raw: 0,
        };
        let delete_clip = Event::KeyDown {
            timestamp: 0,
            window_id: 0,
            keycode: Some(Keycode::Delete),
            scancode: None,
            keymod: Mod::LSHIFTMOD,
            repeat: false,
            which: 0,
            raw: 0,
        };
        let backspace_clip = Event::KeyDown {
            timestamp: 0,
            window_id: 0,
            keycode: Some(Keycode::Backspace),
            scancode: None,
            keymod: Mod::LSHIFTMOD,
            repeat: false,
            which: 0,
            raw: 0,
        };
        let focus_track = Event::KeyDown {
            timestamp: 0,
            window_id: 0,
            keycode: Some(Keycode::F8),
            scancode: None,
            keymod: Mod::LSHIFTMOD,
            repeat: false,
            which: 0,
            raw: 0,
        };

        assert_eq!(
            KeyboardBindings.resolve(&select).unwrap().action,
            AppAction::SelectNotesAtPlayhead
        );
        assert_eq!(
            KeyboardBindings.resolve(&add).unwrap().action,
            AppAction::SelectNotesAtPlayheadAdd
        );
        assert_eq!(
            KeyboardBindings.resolve(&nudge).unwrap().action,
            AppAction::NudgeSelectedNotesUp
        );
        assert_eq!(
            KeyboardBindings.resolve(&view).unwrap().action,
            AppAction::ToggleCurrentTrackRecordingView
        );
        assert_eq!(
            KeyboardBindings.resolve(&prev_clip).unwrap().action,
            AppAction::SelectPreviousRecordingClip
        );
        assert_eq!(
            KeyboardBindings.resolve(&next_clip).unwrap().action,
            AppAction::SelectNextRecordingClip
        );
        assert_eq!(
            KeyboardBindings.resolve(&mute_clip).unwrap().action,
            AppAction::ToggleSelectedRecordingClipMute
        );
        assert_eq!(
            KeyboardBindings.resolve(&delete_clip).unwrap().action,
            AppAction::DeleteSelectedRecordingClip
        );
        assert_eq!(
            KeyboardBindings.resolve(&backspace_clip).unwrap().action,
            AppAction::DeleteSelectedRecordingClip
        );
        assert_eq!(
            KeyboardBindings.resolve(&focus_track).unwrap().action,
            AppAction::ToggleFocusedTrackView
        );
    }

    #[test]
    fn keyboard_bindings_map_brackets_to_loop_actions() {
        let local = Event::KeyDown {
            timestamp: 0,
            window_id: 0,
            keycode: Some(Keycode::LeftBracket),
            scancode: None,
            keymod: Mod::NOMOD,
            repeat: false,
            which: 0,
            raw: 0,
        };
        let global = Event::KeyDown {
            timestamp: 0,
            window_id: 0,
            keycode: Some(Keycode::RightBracket),
            scancode: None,
            keymod: Mod::LSHIFTMOD,
            repeat: false,
            which: 0,
            raw: 0,
        };

        assert_eq!(
            KeyboardBindings.resolve(&local).unwrap().action,
            AppAction::SetCurrentTrackLoopStart
        );
        assert_eq!(
            KeyboardBindings.resolve(&global).unwrap().action,
            AppAction::SetGlobalLoopEnd
        );
    }

    #[test]
    fn keyboard_bindings_map_home_to_global_loop_reset() {
        let event = Event::KeyDown {
            timestamp: 0,
            window_id: 0,
            keycode: Some(Keycode::Home),
            scancode: None,
            keymod: Mod::NOMOD,
            repeat: false,
            which: 0,
            raw: 0,
        };

        assert_eq!(
            KeyboardBindings.resolve(&event).unwrap().action,
            AppAction::ResetGlobalLoop
        );
    }

    #[test]
    fn keyboard_bindings_map_record_and_clear_shortcuts() {
        let record = Event::KeyDown {
            timestamp: 0,
            window_id: 0,
            keycode: Some(Keycode::R),
            scancode: None,
            keymod: Mod::NOMOD,
            repeat: false,
            which: 0,
            raw: 0,
        };
        let record_mode = Event::KeyDown {
            timestamp: 0,
            window_id: 0,
            keycode: Some(Keycode::R),
            scancode: None,
            keymod: Mod::LSHIFTMOD,
            repeat: false,
            which: 0,
            raw: 0,
        };
        let clear_track = Event::KeyDown {
            timestamp: 0,
            window_id: 0,
            keycode: Some(Keycode::C),
            scancode: None,
            keymod: Mod::NOMOD,
            repeat: false,
            which: 0,
            raw: 0,
        };
        let clear_all = Event::KeyDown {
            timestamp: 0,
            window_id: 0,
            keycode: Some(Keycode::C),
            scancode: None,
            keymod: Mod::LSHIFTMOD,
            repeat: false,
            which: 0,
            raw: 0,
        };

        assert_eq!(
            KeyboardBindings.resolve(&record).unwrap().action,
            AppAction::ToggleRecording
        );
        assert_eq!(
            KeyboardBindings.resolve(&record_mode).unwrap().action,
            AppAction::CycleRecordMode
        );
        assert_eq!(
            KeyboardBindings.resolve(&clear_track).unwrap().action,
            AppAction::ClearCurrentTrackContent
        );
        assert_eq!(
            KeyboardBindings.resolve(&clear_all).unwrap().action,
            AppAction::ClearAllTrackContent
        );
    }

    #[test]
    fn keyboard_bindings_map_comma_period_to_nudges() {
        let local = Event::KeyDown {
            timestamp: 0,
            window_id: 0,
            keycode: Some(Keycode::Comma),
            scancode: None,
            keymod: Mod::NOMOD,
            repeat: false,
            which: 0,
            raw: 0,
        };
        let global = Event::KeyDown {
            timestamp: 0,
            window_id: 0,
            keycode: Some(Keycode::Period),
            scancode: None,
            keymod: Mod::LSHIFTMOD,
            repeat: false,
            which: 0,
            raw: 0,
        };

        assert_eq!(
            KeyboardBindings.resolve(&local).unwrap().action,
            AppAction::NudgeCurrentTrackLoopBackward
        );
        assert_eq!(
            KeyboardBindings.resolve(&global).unwrap().action,
            AppAction::NudgeGlobalLoopForward
        );
    }

    #[test]
    fn keyboard_bindings_map_resize_shortcuts() {
        let shorten = Event::KeyDown {
            timestamp: 0,
            window_id: 0,
            keycode: Some(Keycode::Minus),
            scancode: None,
            keymod: Mod::NOMOD,
            repeat: false,
            which: 0,
            raw: 0,
        };
        let extend = Event::KeyDown {
            timestamp: 0,
            window_id: 0,
            keycode: Some(Keycode::Equals),
            scancode: None,
            keymod: Mod::LSHIFTMOD,
            repeat: false,
            which: 0,
            raw: 0,
        };
        let half = Event::KeyDown {
            timestamp: 0,
            window_id: 0,
            keycode: Some(Keycode::Slash),
            scancode: None,
            keymod: Mod::NOMOD,
            repeat: false,
            which: 0,
            raw: 0,
        };
        let double = Event::KeyDown {
            timestamp: 0,
            window_id: 0,
            keycode: Some(Keycode::Backslash),
            scancode: None,
            keymod: Mod::LSHIFTMOD,
            repeat: false,
            which: 0,
            raw: 0,
        };

        assert_eq!(
            KeyboardBindings.resolve(&shorten).unwrap().action,
            AppAction::ShortenCurrentTrackLoop
        );
        assert_eq!(
            KeyboardBindings.resolve(&extend).unwrap().action,
            AppAction::ExtendGlobalLoop
        );
        assert_eq!(
            KeyboardBindings.resolve(&half).unwrap().action,
            AppAction::HalfCurrentTrackLoop
        );
        assert_eq!(
            KeyboardBindings.resolve(&double).unwrap().action,
            AppAction::DoubleGlobalLoop
        );
    }
}
