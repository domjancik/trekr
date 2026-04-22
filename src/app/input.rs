use super::*;

impl App {
    pub(super) fn handle_pointer_event(
        &mut self,
        event: &sdl3::event::Event,
    ) -> Option<AppControl> {
        if let Some((x, y)) = pointer_hover_position(event, self.viewport_size) {
            self.status_state.hovered_target =
                if self.direct_mapping_state.mode == DirectMappingMode::Inactive {
                    self.discoverability_target_at(x, y)
                } else {
                    None
                };
            if self.status_state.hovered_target.is_some() {
                self.direct_mapping_state.status_message = None;
            }
            return Some(AppControl::Continue);
        }

        let (x, y, source) = pointer_down_position(event, self.viewport_size)?;
        self.handle_pointer_down(x, y, source)
    }

    pub(super) fn handle_keyboard_event(
        &mut self,
        event: &sdl3::event::Event,
    ) -> Option<AppControl> {
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
        let surface = crate::ui::surface_rect(self.viewport_size.0, self.viewport_size.1);
        let inset = crate::ui::inset_rect(surface, 24, 24).ok()?;
        let (tabs_bounds, content_bounds, _) = self.page_frame_layout(inset).ok()?;

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
            let direct_badge = Rect::new(content_bounds.x + 532, content_bounds.y + 8, 154, 16);
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
        let tabs = crate::ui::equal_columns(tabs_bounds, AppPage::ALL.len(), 10);
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
