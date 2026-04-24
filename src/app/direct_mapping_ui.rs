use super::*;

impl App {
    pub(super) fn toggle_direct_mapping_mode(&mut self) {
        self.clear_mapping_target_lookup();
        if self.direct_mapping_state.mode == DirectMappingMode::Inactive {
            self.direct_mapping_state.mode = DirectMappingMode::Targeting;
            self.direct_mapping_state.origin = if self.page_state.current_page == AppPage::Mappings
            {
                DirectMappingOrigin::MappingsPage
            } else {
                DirectMappingOrigin::InPlace
            };
            self.direct_mapping_state.status_message = None;
            self.page_state.mapping_midi_learn_armed = false;
            if self.overlay_state.active == Some(AppOverlay::MappingsQuickView) {
                self.overlay_state.active = None;
            }
        } else {
            self.cancel_direct_mapping("Canceled direct mapping.");
        }
        self.sync_midi_inputs();
    }

    pub(super) fn cancel_direct_mapping(&mut self, message: &str) {
        self.clear_mapping_target_lookup();
        self.direct_mapping_state.mode = DirectMappingMode::Inactive;
        self.direct_mapping_state.origin = DirectMappingOrigin::InPlace;
        self.direct_mapping_state.status_message = Some(message.to_string());
        self.sync_midi_inputs();
    }

    pub(super) fn capture_direct_mapping_input(&mut self, event: &MidiInputEvent) -> bool {
        let DirectMappingMode::AwaitingInput(target) = self.direct_mapping_state.mode else {
            return false;
        };

        match event.message {
            MidiInputMessage::NoteOn { .. } | MidiInputMessage::ControlChange { .. } => {}
            MidiInputMessage::NoteOff { .. } => return false,
        }

        self.commit_direct_mapping_source(
            MappingSourceKind::Midi,
            event.port.protocol,
            target,
            &event.port.name,
            &midi_learn_label(event),
        );
        true
    }

    pub(super) fn commit_direct_mapping_source(
        &mut self,
        source_kind: MappingSourceKind,
        source_protocol: MidiTransportProtocol,
        target: DirectMappingTarget,
        source_device_label: &str,
        source_label: &str,
    ) {
        let target_index = self.find_unique_direct_mapping_target_row(
            source_kind,
            target.target_label,
            target.scope_label,
        );
        let source_index =
            self.find_direct_mapping_source_row(source_kind, source_device_label, source_label);

        let index = if let Some(index) = source_index {
            if let Some(target_index) = target_index.filter(|target_index| *target_index != index) {
                if let Some(entry) = self.mappings.get_mut(target_index) {
                    entry.enabled = false;
                }
            }
            index
        } else if let Some(index) = target_index {
            index
        } else {
            let entry = MappingEntry {
                source_kind,
                source_protocol,
                source_device_label: source_device_label.to_string(),
                source_label: source_label.to_string(),
                target_label: target.target_label.to_string(),
                scope_label: target.scope_label.to_string(),
                enabled: true,
            };
            self.mappings.push(entry);
            self.mappings.len() - 1
        };

        let same_target = self.mappings.get(index).is_some_and(|entry| {
            entry.target_label == target.target_label && entry.scope_label == target.scope_label
        });
        if let Some(entry) = self.mappings.get_mut(index) {
            entry.source_kind = source_kind;
            entry.source_protocol = source_protocol;
            entry.source_device_label = source_device_label.to_string();
            entry.source_label = source_label.to_string();
            entry.target_label = target.target_label.to_string();
            entry.scope_label = target.scope_label.to_string();
            entry.enabled = true;
        }
        self.page_state.selected_mapping_index = index;
        if self.direct_mapping_state.origin == DirectMappingOrigin::MappingsPage {
            self.page_state.current_page = AppPage::Mappings;
        }
        let message = if same_target {
            format!(
                "Updated {} ({}) to {}. Select another control to continue, or press Esc to finish.",
                target.target_label, target.scope_label, source_label
            )
        } else {
            format!(
                "Mapped {} ({}) to {}. Select another control to continue, or press Esc to finish.",
                target.target_label, target.scope_label, source_label
            )
        };
        self.direct_mapping_state.mode = DirectMappingMode::Targeting;
        self.direct_mapping_state.status_message = Some(message);
        self.sync_midi_inputs();
    }

    pub(super) fn find_unique_direct_mapping_target_row(
        &self,
        source_kind: MappingSourceKind,
        target_label: &str,
        scope_label: &str,
    ) -> Option<usize> {
        let mut matches = self
            .mappings
            .iter()
            .enumerate()
            .filter(|(_, entry)| {
                entry.target_label == target_label
                    && entry.scope_label == scope_label
                    && entry.source_kind == source_kind
            })
            .map(|(index, _)| index);
        let first = matches.next()?;
        matches.next().is_none().then_some(first)
    }

    pub(super) fn find_direct_mapping_source_row(
        &self,
        source_kind: MappingSourceKind,
        device_label: &str,
        source_label: &str,
    ) -> Option<usize> {
        self.mappings.iter().position(|entry| {
            entry.source_kind == source_kind
                && entry.source_device_label == device_label
                && entry.source_label == source_label
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn direct_mapping_shortcut_toggles_targeting_mode() {
        let mut app = App::new();

        app.apply_action(AppAction::ToggleDirectMappingMode);
        assert_eq!(app.direct_mapping_state.mode, DirectMappingMode::Targeting);

        app.apply_action(AppAction::ToggleDirectMappingMode);
        assert_eq!(app.direct_mapping_state.mode, DirectMappingMode::Inactive);
    }

    #[test]
    fn direct_mapping_input_creates_new_mapping_for_target() {
        let mut app = App::new();
        app.mappings.clear();
        app.direct_mapping_state.mode = DirectMappingMode::AwaitingInput(DirectMappingTarget {
            action: AppAction::TogglePlayback,
            target_label: "Play/Stop",
            scope_label: "Global",
            display_scope: Some("Global"),
            hit_rect: Rect::new(0, 0, 10, 10),
        });

        app.handle_midi_input_event(MidiInputEvent {
            port: MidiPortRef::new("In A"),
            channel: 1,
            message: MidiInputMessage::ControlChange {
                controller: 24,
                value: 127,
            },
        });

        assert_eq!(app.mappings.len(), 1);
        assert_eq!(app.mappings[0].target_label, "Play/Stop");
        assert_eq!(app.mappings[0].scope_label, "Global");
        assert_eq!(app.mappings[0].source_device_label, "In A");
        assert_eq!(app.mappings[0].source_label, "CC24 Ch1");
        assert!(app.mappings[0].enabled);
        assert_eq!(app.page_state.current_page, AppPage::Timeline);
        assert_eq!(app.direct_mapping_state.mode, DirectMappingMode::Targeting);
    }

    #[test]
    fn direct_mapping_from_mappings_page_returns_to_mappings() {
        let mut app = App::new();
        app.mappings.clear();
        app.page_state.current_page = AppPage::Mappings;
        app.direct_mapping_state.origin = DirectMappingOrigin::MappingsPage;
        app.direct_mapping_state.mode = DirectMappingMode::AwaitingInput(DirectMappingTarget {
            action: AppAction::TogglePlayback,
            target_label: "Play/Stop",
            scope_label: "Global",
            display_scope: Some("Global"),
            hit_rect: Rect::new(0, 0, 10, 10),
        });

        app.handle_midi_input_event(MidiInputEvent {
            port: MidiPortRef::new("In A"),
            channel: 1,
            message: MidiInputMessage::ControlChange {
                controller: 24,
                value: 127,
            },
        });

        assert_eq!(app.page_state.current_page, AppPage::Mappings);
        assert_eq!(app.direct_mapping_state.mode, DirectMappingMode::Targeting);
    }

    #[test]
    fn direct_mapping_reuses_unique_target_row() {
        let mut app = App::new();
        app.mappings = vec![MappingEntry {
            source_kind: MappingSourceKind::Midi,
            source_protocol: crate::mapping::default_mapping_source_protocol(),
            source_device_label: "Old Port".to_string(),
            source_label: "CC20 Ch1".to_string(),
            target_label: "Track Arm".to_string(),
            scope_label: "Active Track".to_string(),
            enabled: true,
        }];
        app.direct_mapping_state.mode = DirectMappingMode::AwaitingInput(DirectMappingTarget {
            action: AppAction::ToggleCurrentTrackArm,
            target_label: "Track Arm",
            scope_label: "Active Track",
            display_scope: Some("Active Track"),
            hit_rect: Rect::new(0, 0, 10, 10),
        });

        app.handle_midi_input_event(MidiInputEvent {
            port: MidiPortRef::new("New Port"),
            channel: 2,
            message: MidiInputMessage::ControlChange {
                controller: 21,
                value: 127,
            },
        });

        assert_eq!(app.mappings.len(), 1);
        assert_eq!(app.mappings[0].source_device_label, "New Port");
        assert_eq!(app.mappings[0].source_label, "CC21 Ch2");
        assert_eq!(app.mappings[0].target_label, "Track Arm");
        assert_eq!(app.mappings[0].scope_label, "Active Track");
        assert!(app.mappings[0].enabled);
    }

    #[test]
    fn direct_mapping_moves_existing_source_and_disables_old_target_row() {
        let mut app = App::new();
        app.mappings = vec![
            MappingEntry {
                source_kind: MappingSourceKind::Midi,
                source_protocol: crate::mapping::default_mapping_source_protocol(),
                source_device_label: "Port A".to_string(),
                source_label: "CC20 Ch1".to_string(),
                target_label: "Play/Stop".to_string(),
                scope_label: "Global".to_string(),
                enabled: true,
            },
            MappingEntry {
                source_kind: MappingSourceKind::Midi,
                source_protocol: crate::mapping::default_mapping_source_protocol(),
                source_device_label: "Port B".to_string(),
                source_label: "CC21 Ch1".to_string(),
                target_label: "Track Arm".to_string(),
                scope_label: "Active Track".to_string(),
                enabled: true,
            },
        ];
        app.direct_mapping_state.mode = DirectMappingMode::AwaitingInput(DirectMappingTarget {
            action: AppAction::ToggleCurrentTrackArm,
            target_label: "Track Arm",
            scope_label: "Active Track",
            display_scope: Some("Active Track"),
            hit_rect: Rect::new(0, 0, 10, 10),
        });

        app.handle_midi_input_event(MidiInputEvent {
            port: MidiPortRef::new("Port A"),
            channel: 1,
            message: MidiInputMessage::ControlChange {
                controller: 20,
                value: 127,
            },
        });

        assert_eq!(app.mappings.len(), 2);
        assert_eq!(app.mappings[0].target_label, "Track Arm");
        assert_eq!(app.mappings[0].scope_label, "Active Track");
        assert!(app.mappings[0].enabled);
        assert!(!app.mappings[1].enabled);
    }

    #[test]
    fn direct_mapping_cancel_message_yields_to_hover_summary() {
        let mut app = App::new();
        app.cancel_direct_mapping("Canceled direct mapping.");
        app.status_state.hovered_target = Some(DiscoverabilityTarget {
            action: AppAction::TogglePlayback,
            display_scope: Some("Global"),
            allowed_mapping_scopes: &["Global"],
            overlay_slot: None,
        });

        assert!(app.direct_mapping_footer_content().is_none());
    }

    #[test]
    fn direct_mapping_keyboard_capture_supports_modifiers() {
        let mut app = App::new();
        app.mappings.clear();
        app.direct_mapping_state.mode = DirectMappingMode::AwaitingInput(DirectMappingTarget {
            action: AppAction::TogglePlayback,
            target_label: "Play/Stop",
            scope_label: "Global",
            display_scope: Some("Global"),
            hit_rect: Rect::new(0, 0, 10, 10),
        });

        let control = app.handle_keyboard_event(&sdl3::event::Event::KeyDown {
            timestamp: 0,
            window_id: 0,
            keycode: Some(sdl3::keyboard::Keycode::R),
            scancode: None,
            keymod: sdl3::keyboard::Mod::LCTRLMOD | sdl3::keyboard::Mod::LSHIFTMOD,
            repeat: false,
            which: 0,
            raw: 0,
        });

        assert_eq!(control, Some(AppControl::Continue));
        assert_eq!(app.mappings.len(), 1);
        assert_eq!(app.mappings[0].source_kind, MappingSourceKind::Key);
        assert_eq!(app.mappings[0].source_label, "Ctrl+Shift+R");
        assert_eq!(app.mappings[0].target_label, "Play/Stop");
        assert_eq!(app.direct_mapping_state.mode, DirectMappingMode::Targeting);
    }

    #[test]
    fn direct_mapping_keyboard_path_reserves_escape_and_f8_for_cancel() {
        let mut app = App::new();
        app.direct_mapping_state.mode = DirectMappingMode::AwaitingInput(DirectMappingTarget {
            action: AppAction::TogglePlayback,
            target_label: "Play/Stop",
            scope_label: "Global",
            display_scope: Some("Global"),
            hit_rect: Rect::new(0, 0, 10, 10),
        });

        let escape = app.handle_keyboard_event(&sdl3::event::Event::KeyDown {
            timestamp: 0,
            window_id: 0,
            keycode: Some(sdl3::keyboard::Keycode::Escape),
            scancode: None,
            keymod: sdl3::keyboard::Mod::NOMOD,
            repeat: false,
            which: 0,
            raw: 0,
        });
        assert_eq!(escape, Some(AppControl::Continue));
        assert!(
            app.mappings.is_empty()
                || app
                    .mappings
                    .iter()
                    .all(|entry| entry.target_label != "Play/Stop"
                        || entry.source_kind != MappingSourceKind::Key
                        || entry.source_label != "Escape")
        );
        assert_eq!(app.direct_mapping_state.mode, DirectMappingMode::Inactive);

        app.direct_mapping_state.mode = DirectMappingMode::AwaitingInput(DirectMappingTarget {
            action: AppAction::TogglePlayback,
            target_label: "Play/Stop",
            scope_label: "Global",
            display_scope: Some("Global"),
            hit_rect: Rect::new(0, 0, 10, 10),
        });
        let f8 = app.handle_keyboard_event(&sdl3::event::Event::KeyDown {
            timestamp: 0,
            window_id: 0,
            keycode: Some(sdl3::keyboard::Keycode::F8),
            scancode: None,
            keymod: sdl3::keyboard::Mod::NOMOD,
            repeat: false,
            which: 0,
            raw: 0,
        });
        assert_eq!(f8, Some(AppControl::Continue));
        assert_eq!(app.direct_mapping_state.mode, DirectMappingMode::Inactive);
    }
}
