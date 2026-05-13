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
            self.direct_mapping_state.jump_input_active = false;
            self.direct_mapping_state.jump_query.clear();
            self.page_state.mapping_midi_learn_armed = false;
            if self.overlay_state.active == Some(AppOverlay::MappingsQuickView) {
                self.overlay_state.active = None;
            }
            self.reset_direct_mapping_target_selection();
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
        self.direct_mapping_state.current_target_index = None;
        self.direct_mapping_state.jump_input_active = false;
        self.direct_mapping_state.jump_query.clear();
        self.sync_midi_inputs();
    }

    pub(super) fn direct_mapping_targets_for_current_page(&self) -> Vec<DirectMappingTarget> {
        let surface = crate::ui::surface_rect(
            self.viewport_size.0,
            self.viewport_size.1,
            self.ui_metrics(),
        );
        let Some(inset) = crate::ui::inset_rect(
            surface,
            self.ui_metrics().frame_inset_x_px,
            self.ui_metrics().frame_inset_y_px,
        )
        .ok() else {
            return Vec::new();
        };
        let Some((_, content_bounds, _)) = self.page_frame_layout(inset).ok() else {
            return Vec::new();
        };
        self.direct_mapping_targets(content_bounds)
    }

    pub(super) fn reset_direct_mapping_target_selection(&mut self) {
        self.direct_mapping_state.current_target_index =
            (!self.direct_mapping_targets_for_current_page().is_empty()).then_some(0);
        self.direct_mapping_state.jump_query.clear();
    }

    pub(super) fn normalize_direct_mapping_target_index(&mut self) -> Option<usize> {
        let targets = self.direct_mapping_targets_for_current_page();
        if targets.is_empty() {
            self.direct_mapping_state.current_target_index = None;
            return None;
        }
        let index = self
            .direct_mapping_state
            .current_target_index
            .unwrap_or(0)
            .min(targets.len() - 1);
        self.direct_mapping_state.current_target_index = Some(index);
        Some(index)
    }

    pub(super) fn current_direct_mapping_target(&mut self) -> Option<DirectMappingTarget> {
        let index = self.normalize_direct_mapping_target_index()?;
        self.direct_mapping_targets_for_current_page()
            .get(index)
            .copied()
    }

    pub(super) fn set_direct_mapping_current_target(&mut self, target: DirectMappingTarget) {
        let targets = self.direct_mapping_targets_for_current_page();
        self.direct_mapping_state.current_target_index = targets
            .iter()
            .position(|candidate| self.direct_mapping_targets_match(*candidate, target));
        self.direct_mapping_state.jump_query.clear();
    }

    pub(super) fn direct_mapping_targets_match(
        &self,
        left: DirectMappingTarget,
        right: DirectMappingTarget,
    ) -> bool {
        left.action == right.action
            && left.target_label == right.target_label
            && left.scope_label == right.scope_label
    }

    pub(super) fn select_direct_mapping_target(&mut self, target: DirectMappingTarget) {
        self.set_direct_mapping_current_target(target);
        self.direct_mapping_state.mode = DirectMappingMode::AwaitingInput(target);
        self.direct_mapping_state.status_message = None;
        self.sync_midi_inputs();
    }

    pub(super) fn cycle_direct_mapping_target(&mut self, delta: i32) -> bool {
        let targets = self.direct_mapping_targets_for_current_page();
        if targets.is_empty() {
            self.direct_mapping_state.current_target_index = None;
            return false;
        }
        let count = targets.len() as i32;
        let current = self
            .direct_mapping_state
            .current_target_index
            .unwrap_or(0)
            .min(targets.len() - 1) as i32;
        let next = (current + delta).rem_euclid(count) as usize;
        self.direct_mapping_state.current_target_index = Some(next);
        self.direct_mapping_state.jump_input_active = false;
        self.direct_mapping_state.jump_query.clear();
        if matches!(
            self.direct_mapping_state.mode,
            DirectMappingMode::AwaitingInput(_)
        ) {
            self.direct_mapping_state.mode = DirectMappingMode::AwaitingInput(targets[next]);
            self.direct_mapping_state.status_message = None;
            self.sync_midi_inputs();
        }
        true
    }

    pub(super) fn move_direct_mapping_target(&mut self, dx: i32, dy: i32) -> bool {
        let targets = self.direct_mapping_targets_for_current_page();
        let Some(current_index) = self.normalize_direct_mapping_target_index() else {
            return false;
        };
        let current = targets[current_index];
        let current_center = (
            current.hit_rect.x + current.hit_rect.width() as i32 / 2,
            current.hit_rect.y + current.hit_rect.height() as i32 / 2,
        );

        let mut best_index = None;
        let mut best_score = (i32::MAX, i32::MAX);
        for (index, candidate) in targets.iter().copied().enumerate() {
            if index == current_index {
                continue;
            }
            let candidate_center = (
                candidate.hit_rect.x + candidate.hit_rect.width() as i32 / 2,
                candidate.hit_rect.y + candidate.hit_rect.height() as i32 / 2,
            );
            let delta_x = candidate_center.0 - current_center.0;
            let delta_y = candidate_center.1 - current_center.1;
            if dx != 0 && (delta_x.signum() != dx.signum() || delta_x == 0) {
                continue;
            }
            if dy != 0 && (delta_y.signum() != dy.signum() || delta_y == 0) {
                continue;
            }
            let primary = if dx != 0 {
                delta_x.abs()
            } else {
                delta_y.abs()
            };
            let secondary = if dx != 0 {
                delta_y.abs()
            } else {
                delta_x.abs()
            };
            let score = (primary, secondary);
            if score < best_score {
                best_score = score;
                best_index = Some(index);
            }
        }

        let Some(best_index) = best_index else {
            return false;
        };
        self.direct_mapping_state.current_target_index = Some(best_index);
        self.direct_mapping_state.jump_input_active = false;
        self.direct_mapping_state.jump_query.clear();
        if matches!(
            self.direct_mapping_state.mode,
            DirectMappingMode::AwaitingInput(_)
        ) {
            self.direct_mapping_state.mode = DirectMappingMode::AwaitingInput(targets[best_index]);
            self.direct_mapping_state.status_message = None;
            self.sync_midi_inputs();
        }
        true
    }

    pub(super) fn direct_mapping_hint_labels(
        &self,
        targets: &[DirectMappingTarget],
    ) -> Vec<String> {
        (0..targets.len()).map(direct_mapping_hint_label).collect()
    }

    pub(super) fn apply_direct_mapping_jump_key(&mut self, input: char) -> bool {
        if self.direct_mapping_state.mode != DirectMappingMode::Targeting {
            return false;
        }
        let targets = self.direct_mapping_targets_for_current_page();
        if targets.is_empty() {
            return false;
        }
        self.direct_mapping_state
            .jump_query
            .push(input.to_ascii_uppercase());
        let labels = self.direct_mapping_hint_labels(&targets);
        let query = self.direct_mapping_state.jump_query.clone();
        let matches = labels
            .iter()
            .enumerate()
            .filter(|(_, label)| label.starts_with(&query))
            .map(|(index, _)| index)
            .collect::<Vec<_>>();
        if matches.is_empty() {
            self.direct_mapping_state.jump_query.clear();
            self.direct_mapping_state.jump_input_active = false;
            self.direct_mapping_state.status_message =
                Some("No matching direct-map hint on this page.".to_string());
            return true;
        }
        self.direct_mapping_state.current_target_index = Some(matches[0]);
        if let Some(exact_match) = labels.iter().position(|label| *label == query) {
            self.direct_mapping_state.jump_query.clear();
            self.direct_mapping_state.jump_input_active = false;
            self.select_direct_mapping_target(targets[exact_match]);
            return true;
        }
        self.direct_mapping_state.status_message = None;
        true
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
            target,
            &event.port.name,
            &midi_learn_label(event),
        );
        true
    }

    pub(super) fn commit_direct_mapping_source(
        &mut self,
        source_kind: MappingSourceKind,
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
        self.direct_mapping_state.jump_input_active = false;
        self.direct_mapping_state.status_message = Some(message);
        self.set_direct_mapping_current_target(target);
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

fn direct_mapping_hint_label(mut index: usize) -> String {
    let mut label = String::new();
    loop {
        let remainder = index % 26;
        label.insert(0, (b'A' + remainder as u8) as char);
        if index < 26 {
            break;
        }
        index = index / 26 - 1;
    }
    label
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn direct_mapping_shortcut_toggles_targeting_mode() {
        let mut app = App::new();

        app.apply_action(AppAction::ToggleDirectMappingMode);
        assert_eq!(app.direct_mapping_state.mode, DirectMappingMode::Targeting);
        assert!(app.direct_mapping_state.current_target_index.is_some());

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
                source_device_label: "Port A".to_string(),
                source_label: "CC20 Ch1".to_string(),
                target_label: "Play/Stop".to_string(),
                scope_label: "Global".to_string(),
                enabled: true,
            },
            MappingEntry {
                source_kind: MappingSourceKind::Midi,
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

    #[test]
    fn direct_mapping_hint_labels_progress_like_vimium_sequences() {
        assert_eq!(direct_mapping_hint_label(0), "A");
        assert_eq!(direct_mapping_hint_label(25), "Z");
        assert_eq!(direct_mapping_hint_label(26), "AA");
        assert_eq!(direct_mapping_hint_label(27), "AB");
    }
}
