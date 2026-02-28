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
    TogglePlayback,
    ToggleGlobalLoop,
    ToggleCurrentTrackLoop,
    SetCurrentTrackLoopStart,
    SetCurrentTrackLoopEnd,
    SetGlobalLoopStart,
    SetGlobalLoopEnd,
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
                keycode: Some(Keycode::G),
                repeat: false,
                ..
            } => Some(ActionEvent::new(
                AppAction::ToggleGlobalLoop,
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
}
