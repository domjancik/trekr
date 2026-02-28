use crate::ui::TimelineFlow;
use sdl3::event::Event;
use sdl3::keyboard::Keycode;

/// The canonical application command layer.
///
/// All control surfaces should resolve into these actions before mutating app
/// state so inputs remain remappable and transport behavior stays consistent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppAction {
    Quit,
    ToggleTimelineFlow,
    TogglePlayback,
    ToggleLoopEnabled,
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
                AppAction::ToggleTimelineFlow,
                ActionSource::Keyboard,
            )),
            Event::KeyDown {
                keycode: Some(Keycode::P),
                repeat: false,
                ..
            } => Some(ActionEvent::new(
                AppAction::TogglePlayback,
                ActionSource::Keyboard,
            )),
            Event::KeyDown {
                keycode: Some(Keycode::L),
                repeat: false,
                ..
            } => Some(ActionEvent::new(
                AppAction::ToggleLoopEnabled,
                ActionSource::Keyboard,
            )),
            _ => None,
        }
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
}
