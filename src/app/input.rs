use super::shell::layout::page_tabs_layout;
use super::*;
use crate::distributed::RemoteUiIntent;

impl App {
    pub(crate) fn handle_remote_pointer_hover(&mut self, x: i32, y: i32) -> AppControl {
        self.status_state.hovered_target =
            if self.direct_mapping_state.mode == DirectMappingMode::Inactive {
                self.discoverability_target_at(x, y)
            } else {
                None
            };
        if self.status_state.hovered_target.is_some() {
            self.direct_mapping_state.status_message = None;
        }
        AppControl::Continue
    }

    pub(crate) fn handle_remote_pointer_down(
        &mut self,
        x: i32,
        y: i32,
        source: crate::actions::ActionSource,
    ) -> Option<AppControl> {
        self.handle_pointer_down(x, y, source)
    }

    pub(crate) fn resolve_remote_key_intent(
        &self,
        keycode: sdl3::keyboard::Keycode,
        keymod: sdl3::keyboard::Mod,
        repeat: bool,
    ) -> Option<RemoteUiIntent> {
        let event = sdl3::event::Event::KeyDown {
            timestamp: 0,
            window_id: 0,
            keycode: Some(keycode),
            scancode: None,
            keymod,
            repeat,
            which: 0,
            raw: 0,
        };

        if self.target_lookup_state.active.is_some() {
            match keycode {
                sdl3::keyboard::Keycode::Escape if !repeat => {
                    return Some(RemoteUiIntent::Action {
                        action: AppAction::CancelCurrentMode,
                    });
                }
                sdl3::keyboard::Keycode::Backspace if !repeat => {
                    return Some(RemoteUiIntent::BackspaceMappingTargetLookup);
                }
                sdl3::keyboard::Keycode::Tab if !repeat => return None,
                _ => {
                    if let Some(input) = mapping_target_lookup_input(&event) {
                        return Some(RemoteUiIntent::AppendMappingTargetLookupText { text: input });
                    }
                }
            }
        }

        if matches!(keycode, sdl3::keyboard::Keycode::Escape)
            && !repeat
            && self.direct_mapping_state.mode != DirectMappingMode::Inactive
        {
            return Some(RemoteUiIntent::Action {
                action: AppAction::CancelCurrentMode,
            });
        }

        if let Some(source_label) = direct_mapping_key_label(&event) {
            if self.direct_mapping_state.mode != DirectMappingMode::Inactive {
                if matches!(
                    self.direct_mapping_state.mode,
                    DirectMappingMode::AwaitingInput(_)
                ) {
                    return Some(RemoteUiIntent::CaptureDirectMappingKey {
                        source_label: source_label.clone(),
                    });
                }
            }
            let mapping_actions = self.resolve_key_mapping_actions(&source_label);
            if !mapping_actions.is_empty() {
                return Some(RemoteUiIntent::Actions {
                    actions: mapping_actions,
                });
            }
        }

        self.keyboard_bindings
            .resolve(&event)
            .map(|action_event| RemoteUiIntent::Action {
                action: action_event.action,
            })
    }

    pub(crate) fn resolve_remote_pointer_intent(
        &self,
        x: i32,
        y: i32,
        source: crate::actions::ActionSource,
    ) -> Option<RemoteUiIntent> {
        let surface =
            crate::ui::surface_rect(self.viewport_size.0, self.viewport_size.1, self.ui_metrics());
        let inset = crate::ui::inset_rect(surface, 24, 24).ok()?;
        let (tabs_bounds, content_bounds, _) = self.page_frame_layout(inset).ok()?;

        if self.direct_mapping_state.mode != DirectMappingMode::Inactive {
            if self.page_state.current_page == AppPage::Mappings {
                let direct_badge = Rect::new(content_bounds.x + 532, content_bounds.y + 8, 154, 16);
                if rect_contains(direct_badge, x, y) {
                    return Some(RemoteUiIntent::Action {
                        action: AppAction::ToggleDirectMappingMode,
                    });
                }
            }

            if let Some(page) = self.hit_page_tab(tabs_bounds, x, y) {
                return Some(RemoteUiIntent::Action {
                    action: AppAction::ShowPage(page),
                });
            }

            if let Some(target) = self.direct_mapping_target_at(content_bounds, x, y) {
                return Some(RemoteUiIntent::BeginDirectMappingInput {
                    action: target.action,
                    target_label: target.target_label.to_string(),
                    scope_label: target.scope_label.to_string(),
                    display_scope: target.display_scope.map(str::to_string),
                });
            }
            return None;
        }

        if let Some(page) = self.hit_page_tab(tabs_bounds, x, y) {
            return Some(RemoteUiIntent::Action {
                action: AppAction::ShowPage(page),
            });
        }

        crate::page_widgets::resolve_page_pointer_intent(
            self.page_state.current_page,
            self,
            content_bounds,
            x,
            y,
            source,
        )
    }

    fn find_direct_mapping_target_descriptor(
        &self,
        action: AppAction,
        target_label: &str,
        scope_label: &str,
        display_scope: Option<&str>,
    ) -> Option<DirectMappingTarget> {
        let surface =
            crate::ui::surface_rect(self.viewport_size.0, self.viewport_size.1, self.ui_metrics());
        let inset = crate::ui::inset_rect(surface, 24, 24).ok()?;
        let (_, content_bounds, _) = self.page_frame_layout(inset).ok()?;
        self.direct_mapping_targets(content_bounds)
            .into_iter()
            .find(|target| {
                target.action == action
                    && target.target_label == target_label
                    && target.scope_label == scope_label
                    && target.display_scope == display_scope
            })
    }

    pub(crate) fn apply_remote_ui_intent(&mut self, intent: RemoteUiIntent) -> Option<AppControl> {
        match intent {
            RemoteUiIntent::Action { action } => {
                Some(self.apply_action_with_source(action, ActionSource::Remote))
            }
            RemoteUiIntent::Actions { actions } => {
                for action in actions {
                    let control = self.apply_action_with_source(action, ActionSource::Remote);
                    if control == AppControl::Quit {
                        return Some(control);
                    }
                }
                Some(AppControl::Continue)
            }
            RemoteUiIntent::TrackAction {
                track_index,
                action,
            } => {
                self.project.active_track_index =
                    track_index.min(self.project.tracks.len().saturating_sub(1));
                Some(self.apply_action_with_source(action, ActionSource::Remote))
            }
            RemoteUiIntent::BeginDirectMappingInput {
                action,
                target_label,
                scope_label,
                display_scope,
            } => {
                if let Some(target) = self.find_direct_mapping_target_descriptor(
                    action,
                    &target_label,
                    &scope_label,
                    display_scope.as_deref(),
                ) {
                    self.direct_mapping_state.mode = DirectMappingMode::AwaitingInput(target);
                    self.direct_mapping_state.status_message = None;
                    self.sync_midi_inputs();
                }
                Some(AppControl::Continue)
            }
            RemoteUiIntent::CaptureDirectMappingKey { source_label } => {
                if let DirectMappingMode::AwaitingInput(target) = self.direct_mapping_state.mode {
                    self.commit_direct_mapping_source(
                        MappingSourceKind::Key,
                        target,
                        &default_mapping_source_device(),
                        &source_label,
                    );
                }
                Some(AppControl::Continue)
            }
            RemoteUiIntent::SelectMappingRow { index } => {
                self.page_state.selected_mapping_index =
                    index.min(self.mappings.len().saturating_sub(1));
                self.normalize_selected_mapping_field();
                self.page_state.mapping_midi_learn_armed = false;
                self.clear_mapping_target_lookup();
                Some(AppControl::Continue)
            }
            RemoteUiIntent::ActivateMappingField {
                index,
                field,
                activate,
            } => {
                self.page_state.selected_mapping_index =
                    index.min(self.mappings.len().saturating_sub(1));
                self.normalize_selected_mapping_field();
                self.page_state.mapping_midi_learn_armed = false;
                self.clear_mapping_target_lookup();
                self.page_state.selected_mapping_field = field;
                if activate {
                    self.activate_mapping_field();
                }
                Some(AppControl::Continue)
            }
            RemoteUiIntent::CommitMappingTargetLookupLabel { label } => {
                let results = self.mapping_target_lookup_results();
                if let Some(found) = results.into_iter().find(|candidate| *candidate == label) {
                    self.commit_mapping_target_lookup_label(found);
                }
                Some(AppControl::Continue)
            }
            RemoteUiIntent::AppendMappingTargetLookupText { text } => {
                self.append_mapping_target_lookup_text(&text);
                Some(AppControl::Continue)
            }
            RemoteUiIntent::BackspaceMappingTargetLookup => {
                self.backspace_mapping_target_lookup();
                Some(AppControl::Continue)
            }
            RemoteUiIntent::CancelMappingTargetLookup => {
                self.cancel_mapping_target_lookup();
                Some(AppControl::Continue)
            }
            RemoteUiIntent::SetMidiIoFocus { focus } => {
                self.page_state.midi_io.focus = focus;
                Some(AppControl::Continue)
            }
            RemoteUiIntent::SelectMidiInput { index } => {
                self.page_state.midi_io.focus = MidiIoListFocus::Inputs;
                self.page_state.midi_io.selected_input_index = index;
                self.set_preferred_default_input_from_index(index);
                Some(AppControl::Continue)
            }
            RemoteUiIntent::SelectMidiOutput { index } => {
                self.page_state.midi_io.focus = MidiIoListFocus::Outputs;
                self.page_state.midi_io.selected_output_index = index;
                self.set_preferred_default_output_from_index(index);
                Some(AppControl::Continue)
            }
            RemoteUiIntent::RoutingAdjustField { field, delta } => {
                self.page_state.selected_routing_field = field;
                self.adjust_routing_field(delta);
                Some(AppControl::Continue)
            }
            RemoteUiIntent::RoutingActivateField { field } => {
                self.page_state.selected_routing_field = field;
                self.activate_page_item();
                Some(AppControl::Continue)
            }
            RemoteUiIntent::TimelineFxClick {
                track_index,
                context,
                row_index,
                field,
                action,
            } => {
                self.project.active_track_index =
                    track_index.min(self.project.tracks.len().saturating_sub(1));
                self.page_state.selected_timeline_context = context;
                if let Some(chain_kind) = context.chain_kind() {
                    self.set_selected_timeline_fx_row(chain_kind, row_index);
                }
                if let Some(field) = field {
                    self.page_state.selected_timeline_fx_field = field;
                }
                action
                    .map(|action| self.apply_action_with_source(action, ActionSource::Remote))
                    .or(Some(AppControl::Continue))
            }
        }
    }

    pub(super) fn poll_midi_input(&mut self) {
        let events = self.midi_input.drain_events();
        for event in events {
            self.handle_midi_input_event(event);
        }
    }

    pub(super) fn handle_midi_input_event(&mut self, event: MidiInputEvent) {
        if self.capture_direct_mapping_input(&event) {
            return;
        }

        if self.capture_mapping_midi_learn(&event) {
            return;
        }

        let mapping_actions = self.resolve_midi_mapping_actions(&event);
        for action in mapping_actions {
            let _ = self.apply_action_with_source(action, ActionSource::Midi);
        }

        let matching_tracks: Vec<usize> = self
            .project
            .tracks
            .iter()
            .enumerate()
            .filter(|(_, track)| {
                self.resolved_input_port(&track.routing.input_port) == Some(&event.port)
                    && match track.routing.input_channel {
                        MidiChannelFilter::Omni => true,
                        MidiChannelFilter::Channel(channel) => channel == event.channel,
                    }
            })
            .map(|(index, _)| index)
            .collect();

        for index in matching_tracks {
            let input_ticks = self
                .project
                .tracks
                .get(index)
                .map(|track| self.live_input_event_ticks(track))
                .unwrap_or(self.playhead_ticks);

            let Some(track_view) = self.project.tracks.get(index) else {
                continue;
            };

            let (
                record_mode,
                monitor_input_fx,
                passthrough,
                output_port,
                output_channel,
                input_chain,
                output_chain,
            ) = (
                track_view.midi_fx.record_input_fx_mode,
                track_view.midi_fx.monitor_input_fx,
                track_view.state.passthrough,
                track_view
                    .routing
                    .output_port
                    .cloned_resolved(self.default_output_port()),
                track_view.routing.output_channel,
                track_view.midi_fx.input_fx.clone(),
                track_view.midi_fx.output_fx.clone(),
            );

            match event.message {
                MidiInputMessage::NoteOn { pitch, velocity } => {
                    let raw_event = LiveMidiFxEvent::NoteOn { pitch, velocity };
                    let (post_input_events, monitor_source_events) = self.monitor_source_events(
                        index,
                        raw_event,
                        &input_chain,
                        monitor_input_fx,
                        input_ticks,
                    );
                    if let Some(track) = self.project.tracks.get_mut(index) {
                        if track.active_take.is_some() {
                            let record_events =
                                if record_mode == crate::midi_fx::RecordInputFxMode::PostInputFx {
                                    post_input_events.clone()
                                } else {
                                    vec![LiveMidiFxEvent::NoteOn { pitch, velocity }]
                                };
                            for record_event in record_events {
                                if let LiveMidiFxEvent::NoteOn { pitch, velocity } = record_event {
                                    track.record_note_on(pitch, velocity, input_ticks);
                                }
                            }
                        }
                    }
                    self.propagate_live_clone_events(index, &post_input_events);
                    if passthrough {
                        self.send_live_monitor_events(
                            index,
                            &output_chain,
                            output_port.as_ref(),
                            output_channel,
                            monitor_source_events,
                            input_ticks,
                        );
                    }
                }
                MidiInputMessage::NoteOff { pitch } => {
                    let raw_event = LiveMidiFxEvent::NoteOff { pitch };
                    let (post_input_events, monitor_source_events) = self.monitor_source_events(
                        index,
                        raw_event,
                        &input_chain,
                        monitor_input_fx,
                        input_ticks,
                    );
                    if let Some(track) = self.project.tracks.get_mut(index) {
                        if track.active_take.is_some() {
                            let record_events =
                                if record_mode == crate::midi_fx::RecordInputFxMode::PostInputFx {
                                    post_input_events.clone()
                                } else {
                                    vec![LiveMidiFxEvent::NoteOff { pitch }]
                                };
                            for record_event in record_events {
                                if let LiveMidiFxEvent::NoteOff { pitch } = record_event {
                                    track.record_note_off(pitch, input_ticks);
                                }
                            }
                        }
                    }
                    self.propagate_live_clone_events(index, &post_input_events);
                    if passthrough {
                        self.send_live_monitor_events(
                            index,
                            &output_chain,
                            output_port.as_ref(),
                            output_channel,
                            monitor_source_events,
                            input_ticks,
                        );
                    }
                }
                MidiInputMessage::ControlChange { .. } => {}
            }
        }
    }

    pub(super) fn capture_mapping_midi_learn(&mut self, event: &MidiInputEvent) -> bool {
        if self.page_state.current_page != AppPage::Mappings
            || self.page_state.mapping_mode != MappingPageMode::Write
            || !self.page_state.mapping_midi_learn_armed
        {
            return false;
        }

        let index = self.page_state.selected_mapping_index;
        let Some(entry) = self.mappings.get_mut(index) else {
            return false;
        };

        entry.source_kind = MappingSourceKind::Midi;
        entry.source_device_label = event.port.name.clone();
        entry.source_label = midi_learn_label(event);
        entry.enabled = true;
        self.page_state.mapping_midi_learn_armed = false;
        true
    }

    pub(super) fn resolve_midi_mapping_actions(&self, event: &MidiInputEvent) -> Vec<AppAction> {
        self.mappings
            .iter()
            .filter(|entry| midi_mapping_matches_event(entry, event))
            .flat_map(|entry| mapping_entry_to_actions(entry, event))
            .collect()
    }

    pub(super) fn resolve_key_mapping_actions(&self, source_label: &str) -> Vec<AppAction> {
        self.mappings
            .iter()
            .filter(|entry| {
                entry.enabled
                    && entry.source_kind == MappingSourceKind::Key
                    && entry.source_label == source_label
            })
            .flat_map(mapping_entry_key_actions)
            .collect()
    }

    pub(super) fn handle_pointer_event(
        &mut self,
        event: &sdl3::event::Event,
    ) -> Option<AppControl> {
        if let Some((x, y)) = pointer_hover_position(event, self.viewport_size) {
            return Some(self.handle_remote_pointer_hover(x, y));
        }

        let (x, y, source) = pointer_down_position(event, self.viewport_size)?;
        self.handle_remote_pointer_down(x, y, source)
    }

    pub(super) fn handle_keyboard_event(
        &mut self,
        event: &sdl3::event::Event,
    ) -> Option<AppControl> {
        if let Some(control) = self.handle_clip_align_keyboard_event(event) {
            return Some(control);
        }

        if self.target_lookup_state.active.is_some() {
            match event {
                sdl3::event::Event::KeyDown {
                    keycode: Some(sdl3::keyboard::Keycode::Escape),
                    repeat: false,
                    ..
                } => {
                    return Some(self.apply_action_with_source(
                        AppAction::CancelCurrentMode,
                        crate::actions::ActionSource::Keyboard,
                    ));
                }
                sdl3::event::Event::KeyDown {
                    keycode: Some(sdl3::keyboard::Keycode::Backspace),
                    repeat: false,
                    ..
                } => {
                    self.backspace_mapping_target_lookup();
                    return Some(AppControl::Continue);
                }
                sdl3::event::Event::KeyDown {
                    keycode: Some(sdl3::keyboard::Keycode::Tab),
                    repeat: false,
                    ..
                } => {
                    return Some(AppControl::Continue);
                }
                _ => {
                    if let Some(input) = mapping_target_lookup_input(event) {
                        self.append_mapping_target_lookup_text(&input);
                        return Some(AppControl::Continue);
                    }
                }
            }
        }

        if matches!(
            event,
            sdl3::event::Event::KeyDown {
                keycode: Some(sdl3::keyboard::Keycode::Escape),
                repeat: false,
                ..
            }
        ) && self.direct_mapping_state.mode != DirectMappingMode::Inactive
        {
            return Some(self.apply_action_with_source(
                AppAction::CancelCurrentMode,
                crate::actions::ActionSource::Keyboard,
            ));
        }

        if let Some(source_label) = direct_mapping_key_label(event) {
            if self.direct_mapping_state.mode != DirectMappingMode::Inactive {
                if let DirectMappingMode::AwaitingInput(target) = self.direct_mapping_state.mode {
                    self.commit_direct_mapping_source(
                        MappingSourceKind::Key,
                        target,
                        &default_mapping_source_device(),
                        &source_label,
                    );
                }
                return Some(AppControl::Continue);
            }

            let mapping_actions = self.resolve_key_mapping_actions(&source_label);
            if !mapping_actions.is_empty() {
                for action in mapping_actions {
                    let control = self.apply_action_with_source(action, ActionSource::Keyboard);
                    if control == AppControl::Quit {
                        return Some(control);
                    }
                }
                return Some(AppControl::Continue);
            }
        }

        self.keyboard_bindings.resolve(event).map(|action_event| {
            self.apply_action_with_source(action_event.action, action_event.source)
        })
    }

    fn handle_pointer_down(
        &mut self,
        x: i32,
        y: i32,
        source: crate::actions::ActionSource,
    ) -> Option<AppControl> {
        let surface = crate::ui::surface_rect(
            self.viewport_size.0,
            self.viewport_size.1,
            self.ui_metrics(),
        );
        let inset = crate::ui::inset_rect(
            surface,
            self.ui_metrics().frame_inset_x_px,
            self.ui_metrics().frame_inset_y_px,
        )
        .ok()?;
        let (tabs_bounds, content_bounds, _) = self.page_frame_layout(inset).ok()?;

        if let Some(control) = self.handle_clip_align_pointer_down(content_bounds, x, y, source) {
            return Some(control);
        }

        if let Some(control) =
            self.handle_direct_mapping_pointer_down(tabs_bounds, content_bounds, x, y, source)
        {
            return Some(control);
        }

        if let Some(page) = self.hit_page_tab(tabs_bounds, x, y) {
            return Some(self.apply_action_with_source(AppAction::ShowPage(page), source));
        }

        handle_page_pointer(
            self.page_state.current_page,
            self,
            content_bounds,
            x,
            y,
            source,
        )
    }

    pub(super) fn handle_direct_mapping_pointer_down(
        &mut self,
        tabs_bounds: Rect,
        content_bounds: Rect,
        x: i32,
        y: i32,
        source: crate::actions::ActionSource,
    ) -> Option<AppControl> {
        if self.direct_mapping_state.mode == DirectMappingMode::Inactive {
            return None;
        }

        if self.page_state.current_page == AppPage::Mappings {
            let direct_badge = self.mappings_page_layout(content_bounds).direct_badge;
            if rect_contains(direct_badge, x, y) {
                return Some(
                    self.apply_action_with_source(AppAction::ToggleDirectMappingMode, source),
                );
            }
        }

        if let Some(page) = self.hit_page_tab(tabs_bounds, x, y) {
            return Some(self.apply_action_with_source(AppAction::ShowPage(page), source));
        }

        if let Some(target) = self.direct_mapping_target_at(content_bounds, x, y) {
            self.direct_mapping_state.mode = DirectMappingMode::AwaitingInput(target);
            self.direct_mapping_state.status_message = None;
            self.sync_midi_inputs();
            return Some(AppControl::Continue);
        }

        Some(AppControl::Continue)
    }

    fn direct_mapping_target_at(
        &self,
        content_bounds: Rect,
        x: i32,
        y: i32,
    ) -> Option<DirectMappingTarget> {
        self.direct_mapping_targets(content_bounds)
            .into_iter()
            .find(|target| rect_contains(target.hit_rect, x, y))
    }

    pub(super) fn direct_mapping_targets(&self, content_bounds: Rect) -> Vec<DirectMappingTarget> {
        let raw_targets =
            page_discoverability_targets(self.page_state.current_page, self, content_bounds);

        raw_targets
            .into_iter()
            .filter_map(|(rect, target)| {
                mapping_target_label_for_action(target.action).map(|target_label| {
                    DirectMappingTarget {
                        action: target.action,
                        target_label,
                        scope_label: target
                            .allowed_mapping_scopes
                            .first()
                            .copied()
                            .unwrap_or("Global"),
                        display_scope: target.display_scope,
                        hit_rect: rect,
                    }
                })
            })
            .collect()
    }

    pub(super) fn direct_mapping_tab_targets(
        &self,
        _tabs_bounds: Rect,
    ) -> Vec<DirectMappingTarget> {
        Vec::new()
    }

    pub(super) fn hit_page_tab(&self, bounds: Rect, x: i32, y: i32) -> Option<AppPage> {
        let (_, tabs_bounds) = page_tabs_layout(bounds);
        let tabs = crate::ui::equal_columns(
            tabs_bounds,
            AppPage::ALL.len(),
            self.ui_metrics().tabs_column_gap_px,
        );
        AppPage::ALL
            .iter()
            .copied()
            .zip(tabs)
            .find_map(|(page, rect)| rect_contains(rect, x, y).then_some(page))
    }
}

pub(crate) fn rect_contains(rect: Rect, x: i32, y: i32) -> bool {
    x >= rect.x
        && x < rect.x + rect.width() as i32
        && y >= rect.y
        && y < rect.y + rect.height() as i32
}

pub(crate) fn pointer_down_position(
    event: &sdl3::event::Event,
    viewport_size: (u32, u32),
) -> Option<(i32, i32, crate::actions::ActionSource)> {
    match event {
        sdl3::event::Event::MouseButtonDown { x, y, .. } => {
            Some((*x as i32, *y as i32, crate::actions::ActionSource::Pointer))
        }
        sdl3::event::Event::FingerDown { x, y, .. } => Some((
            (*x * viewport_size.0 as f32) as i32,
            (*y * viewport_size.1 as f32) as i32,
            crate::actions::ActionSource::Touch,
        )),
        _ => None,
    }
}

pub(crate) fn pointer_hover_position(
    event: &sdl3::event::Event,
    viewport_size: (u32, u32),
) -> Option<(i32, i32)> {
    match event {
        sdl3::event::Event::MouseMotion { x, y, .. } => Some((*x as i32, *y as i32)),
        sdl3::event::Event::FingerMotion { x, y, .. } => Some((
            (*x * viewport_size.0 as f32) as i32,
            (*y * viewport_size.1 as f32) as i32,
        )),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mapping::{MappingEntry, MappingSourceKind, default_mapping_source_device};

    #[test]
    fn pointer_position_uses_render_coordinates_for_mouse() {
        let event = sdl3::event::Event::MouseButtonDown {
            timestamp: 0,
            window_id: 1,
            which: 0,
            mouse_btn: sdl3::mouse::MouseButton::Left,
            clicks: 1,
            x: 512.5,
            y: 288.25,
        };

        assert_eq!(
            pointer_down_position(&event, (1280, 720)),
            Some((512, 288, crate::actions::ActionSource::Pointer))
        );
    }

    #[test]
    fn pointer_position_uses_converted_render_coordinates_for_touch() {
        let event = sdl3::event::Event::FingerDown {
            timestamp: 0,
            touch_id: 1,
            finger_id: 1,
            x: 0.5,
            y: 0.5,
            dx: 0.0,
            dy: 0.0,
            pressure: 1.0,
        };

        assert_eq!(
            pointer_down_position(&event, (1280, 720)),
            Some((640, 360, crate::actions::ActionSource::Touch))
        );
    }

    #[test]
    fn direct_mapping_pointer_can_retarget_while_awaiting_input() {
        let mut app = App::new();
        let surface =
            crate::ui::surface_rect(app.viewport_size.0, app.viewport_size.1, app.ui_metrics());
        let inset = crate::ui::inset_rect(
            surface,
            app.ui_metrics().frame_inset_x_px,
            app.ui_metrics().frame_inset_y_px,
        )
        .expect("surface inset");
        let (tabs_bounds, page_area_bounds) = crate::ui::split_top_strip(
            inset,
            app.ui_metrics().tabs_height_px,
            app.ui_metrics().tabs_gap_px,
        )
        .expect("page split");
        let content_bounds = Rect::new(
            page_area_bounds.x(),
            page_area_bounds.y(),
            page_area_bounds.width(),
            page_area_bounds.height().saturating_sub(30),
        );
        app.direct_mapping_state.mode = DirectMappingMode::AwaitingInput(DirectMappingTarget {
            action: AppAction::TogglePlayback,
            target_label: "Play/Stop",
            scope_label: "Global",
            display_scope: Some("Global"),
            hit_rect: Rect::new(0, 0, 10, 10),
        });

        let record_target = app
            .direct_mapping_targets(content_bounds)
            .into_iter()
            .find(|target| target.target_label == "Record" && target.scope_label == "Armed/Active")
            .expect("record target");
        let point_x = record_target.hit_rect.x() + (record_target.hit_rect.width() / 2) as i32;
        let point_y = record_target.hit_rect.y() + (record_target.hit_rect.height() / 2) as i32;

        let control = app.handle_direct_mapping_pointer_down(
            tabs_bounds,
            content_bounds,
            point_x,
            point_y,
            ActionSource::Pointer,
        );

        assert_eq!(control, Some(AppControl::Continue));
        assert_eq!(
            app.direct_mapping_state.mode,
            DirectMappingMode::AwaitingInput(record_target)
        );
    }

    #[test]
    fn remote_key_intent_uses_semantic_actions_for_custom_key_mappings() {
        let mut app = App::new();
        app.mappings = vec![MappingEntry {
            source_kind: MappingSourceKind::Key,
            source_device_label: default_mapping_source_device(),
            source_label: "Space".to_string(),
            target_label: "Record".to_string(),
            scope_label: "Armed/Active".to_string(),
            enabled: true,
        }];

        assert_eq!(
            app.resolve_remote_key_intent(
                sdl3::keyboard::Keycode::Space,
                sdl3::keyboard::Mod::NOMOD,
                false,
            ),
            Some(RemoteUiIntent::Actions {
                actions: vec![AppAction::ToggleRecording],
            })
        );
    }

    #[test]
    fn remote_key_intent_uses_capture_intent_for_direct_mapping_keys() {
        let mut app = App::new();
        app.direct_mapping_state.mode = DirectMappingMode::AwaitingInput(DirectMappingTarget {
            action: AppAction::TogglePlayback,
            target_label: "Play/Stop",
            scope_label: "Global",
            display_scope: Some("Global"),
            hit_rect: Rect::new(0, 0, 10, 10),
        });

        assert_eq!(
            app.resolve_remote_key_intent(
                sdl3::keyboard::Keycode::R,
                sdl3::keyboard::Mod::LCTRLMOD | sdl3::keyboard::Mod::LSHIFTMOD,
                false,
            ),
            Some(RemoteUiIntent::CaptureDirectMappingKey {
                source_label: "Ctrl+Shift+R".to_string(),
            })
        );
    }

    #[test]
    fn applying_capture_direct_mapping_key_intent_commits_mapping() {
        let mut app = App::new();
        app.mappings.clear();
        app.direct_mapping_state.mode = DirectMappingMode::AwaitingInput(DirectMappingTarget {
            action: AppAction::TogglePlayback,
            target_label: "Play/Stop",
            scope_label: "Global",
            display_scope: Some("Global"),
            hit_rect: Rect::new(0, 0, 10, 10),
        });

        let control = app.apply_remote_ui_intent(RemoteUiIntent::CaptureDirectMappingKey {
            source_label: "Ctrl+Shift+R".to_string(),
        });

        assert_eq!(control, Some(AppControl::Continue));
        assert_eq!(app.mappings.len(), 1);
        assert_eq!(app.mappings[0].source_kind, MappingSourceKind::Key);
        assert_eq!(app.mappings[0].source_label, "Ctrl+Shift+R");
        assert_eq!(app.mappings[0].target_label, "Play/Stop");
        assert_eq!(app.direct_mapping_state.mode, DirectMappingMode::Targeting);
    }
}
