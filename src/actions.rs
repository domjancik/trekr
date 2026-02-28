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
    ToggleMappingsOverlay,
    ToggleMappingsWriteMode,
    TogglePlayback,
    ToggleRecording,
    ToggleGlobalLoop,
    ResetGlobalLoop,
    ClearCurrentTrackContent,
    ClearAllTrackContent,
    ToggleCurrentTrackLoop,
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
    ShortenGlobalLoop,
    ExtendGlobalLoop,
    HalfGlobalLoop,
    DoubleGlobalLoop,
    ToggleCurrentTrackArm,
    ToggleCurrentTrackMute,
    ToggleCurrentTrackSolo,
    ToggleCurrentTrackPassthrough,
    SelectNextTrack,
    SelectPreviousTrack,
    SelectTrack(usize),
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
                keycode: Some(Keycode::R),
                repeat: false,
                ..
            } => Some(ActionEvent::new(
                AppAction::ToggleRecording,
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
                repeat: false,
                ..
            } => Some(ActionEvent::new(
                AppAction::ToggleCurrentTrackLoop,
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
                keycode: Some(Keycode::A),
                repeat: false,
                ..
            } => Some(ActionEvent::new(
                AppAction::ToggleCurrentTrackArm,
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
                repeat: false,
                ..
            } => Some(ActionEvent::new(
                AppAction::SelectPreviousPageItem,
                ActionSource::Keyboard,
            )),
            Event::KeyDown {
                keycode: Some(Keycode::Down),
                repeat: false,
                ..
            } => Some(ActionEvent::new(
                AppAction::SelectNextPageItem,
                ActionSource::Keyboard,
            )),
            Event::KeyDown {
                keycode: Some(Keycode::Q),
                repeat: false,
                ..
            } => Some(ActionEvent::new(
                AppAction::AdjustPageItemBackward,
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
                repeat: false,
                ..
            } => Some(ActionEvent::new(
                AppAction::ActivatePageItem,
                ActionSource::Keyboard,
            )),
            Event::KeyDown {
                keycode: Some(Keycode::Right),
                repeat: false,
                ..
            } => Some(ActionEvent::new(
                AppAction::SelectNextTrack,
                ActionSource::Keyboard,
            )),
            Event::KeyDown {
                keycode: Some(Keycode::Left),
                repeat: false,
                ..
            } => Some(ActionEvent::new(
                AppAction::SelectPreviousTrack,
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

        assert_eq!(
            KeyboardBindings.resolve(&overlay).unwrap().action,
            AppAction::ToggleMappingsOverlay
        );
        assert_eq!(
            KeyboardBindings.resolve(&write_mode).unwrap().action,
            AppAction::ToggleMappingsWriteMode
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
