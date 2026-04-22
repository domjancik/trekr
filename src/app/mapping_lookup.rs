use super::*;
use crate::mapping::search_mapping_targets;
use sdl3::event::Event;
use sdl3::keyboard::{Keycode, Mod};

pub(super) fn mapping_target_lookup_input(event: &Event) -> Option<String> {
    let Event::KeyDown {
        keycode: Some(keycode),
        keymod,
        repeat: false,
        ..
    } = event
    else {
        return None;
    };

    if keymod.intersects(
        Mod::LCTRLMOD | Mod::RCTRLMOD | Mod::LALTMOD | Mod::RALTMOD | Mod::LGUIMOD | Mod::RGUIMOD,
    ) {
        return None;
    }

    let input = match keycode {
        Keycode::Space => " ".to_string(),
        Keycode::Minus => "-".to_string(),
        Keycode::Slash => "/".to_string(),
        Keycode::_0 => "0".to_string(),
        Keycode::_1 => "1".to_string(),
        Keycode::_2 => "2".to_string(),
        Keycode::_3 => "3".to_string(),
        Keycode::_4 => "4".to_string(),
        Keycode::_5 => "5".to_string(),
        Keycode::_6 => "6".to_string(),
        Keycode::_7 => "7".to_string(),
        Keycode::_8 => "8".to_string(),
        Keycode::_9 => "9".to_string(),
        Keycode::A => "a".to_string(),
        Keycode::B => "b".to_string(),
        Keycode::C => "c".to_string(),
        Keycode::D => "d".to_string(),
        Keycode::E => "e".to_string(),
        Keycode::F => "f".to_string(),
        Keycode::G => "g".to_string(),
        Keycode::H => "h".to_string(),
        Keycode::I => "i".to_string(),
        Keycode::J => "j".to_string(),
        Keycode::K => "k".to_string(),
        Keycode::L => "l".to_string(),
        Keycode::M => "m".to_string(),
        Keycode::N => "n".to_string(),
        Keycode::O => "o".to_string(),
        Keycode::P => "p".to_string(),
        Keycode::Q => "q".to_string(),
        Keycode::R => "r".to_string(),
        Keycode::S => "s".to_string(),
        Keycode::T => "t".to_string(),
        Keycode::U => "u".to_string(),
        Keycode::V => "v".to_string(),
        Keycode::W => "w".to_string(),
        Keycode::X => "x".to_string(),
        Keycode::Y => "y".to_string(),
        Keycode::Z => "z".to_string(),
        _ => return None,
    };

    Some(input)
}

impl App {
    pub(super) fn mapping_target_lookup_results(&self) -> Vec<&'static str> {
        let Some(lookup) = self.target_lookup_state.active.as_ref() else {
            return Vec::new();
        };
        search_mapping_targets(&lookup.query)
    }

    pub(super) fn mapping_target_lookup_highlighted_label(&self) -> Option<&'static str> {
        let results = self.mapping_target_lookup_results();
        let lookup = self.target_lookup_state.active.as_ref()?;
        if results.is_empty() {
            None
        } else {
            Some(
                results[lookup
                    .highlighted_index
                    .min(results.len().saturating_sub(1))],
            )
        }
    }

    pub(super) fn clear_mapping_target_lookup(&mut self) {
        self.target_lookup_state.active = None;
    }

    pub(super) fn open_mapping_target_lookup(&mut self) {
        if self.page_state.current_page != AppPage::Mappings
            || self.page_state.mapping_mode != MappingPageMode::Write
            || self.page_state.selected_mapping_field != MappingField::Target
        {
            return;
        }

        let Some(entry) = self.mappings.get(self.page_state.selected_mapping_index) else {
            return;
        };

        self.page_state.mapping_midi_learn_armed = false;
        self.target_lookup_state.active = Some(ActiveMappingTargetLookup {
            original_target_label: entry.target_label.clone(),
            original_scope_label: entry.scope_label.clone(),
            query: String::new(),
            highlighted_index: 0,
        });
    }

    pub(super) fn cancel_mapping_target_lookup(&mut self) {
        let Some(lookup) = self.target_lookup_state.active.take() else {
            return;
        };
        let Some(entry) = self.mappings.get_mut(self.page_state.selected_mapping_index) else {
            return;
        };
        entry.target_label = lookup.original_target_label;
        entry.scope_label = lookup.original_scope_label;
    }

    pub(super) fn mapping_target_lookup_is_active(&self) -> bool {
        self.target_lookup_state.active.is_some()
    }

    pub(super) fn move_mapping_target_lookup_highlight(&mut self, delta: i32) {
        let results = self.mapping_target_lookup_results();
        let Some(lookup) = self.target_lookup_state.active.as_mut() else {
            return;
        };
        if results.is_empty() {
            lookup.highlighted_index = 0;
            return;
        }
        let max_index = results.len().saturating_sub(1) as i32;
        let current = lookup
            .highlighted_index
            .min(results.len().saturating_sub(1)) as i32;
        lookup.highlighted_index = (current + delta).clamp(0, max_index) as usize;
    }

    pub(super) fn append_mapping_target_lookup_text(&mut self, text: &str) {
        let Some(lookup) = self.target_lookup_state.active.as_mut() else {
            return;
        };
        if text.is_empty() {
            return;
        }
        lookup.query.push_str(text);
        lookup.highlighted_index = 0;
    }

    pub(super) fn backspace_mapping_target_lookup(&mut self) {
        let Some(lookup) = self.target_lookup_state.active.as_mut() else {
            return;
        };
        lookup.query.pop();
        lookup.highlighted_index = 0;
    }

    pub(super) fn commit_mapping_target_lookup_label(&mut self, target_label: &'static str) {
        let Some(entry) = self.mappings.get_mut(self.page_state.selected_mapping_index) else {
            self.clear_mapping_target_lookup();
            return;
        };
        let track_count = self.project.tracks.len();
        let preserved_scope =
            mapping_scope_valid_for_target(target_label, &entry.scope_label, track_count);
        let previous_scope = entry.scope_label.clone();
        entry.target_label = target_label.to_string();
        if !preserved_scope {
            entry.scope_label = default_scope_label(target_label, track_count);
        }
        self.target_lookup_state.active = None;
        self.direct_mapping_state.status_message = Some(if preserved_scope {
            format!(
                "Selected target {target_label}. Scope preserved: {}.",
                entry.scope_label
            )
        } else {
            format!(
                "Selected target {target_label}. Scope changed: {previous_scope} -> {}.",
                entry.scope_label
            )
        });
    }

    pub(super) fn commit_mapping_target_lookup(&mut self) {
        if let Some(target_label) = self.mapping_target_lookup_highlighted_label() {
            self.commit_mapping_target_lookup_label(target_label);
        }
    }

    pub(super) fn mapping_target_lookup_layout(
        &self,
        content_bounds: Rect,
    ) -> Option<MappingTargetLookupLayout> {
        self.target_lookup_state.active.as_ref()?;
        let list_bounds = Rect::new(
            content_bounds.x + 8,
            content_bounds.y + 44,
            content_bounds.width().saturating_sub(16),
            content_bounds.height().saturating_sub(68),
        );
        let row_gap = 3_i32;
        let row_height = 18_i32;
        let stride = row_height + row_gap;
        let visible_rows = ((list_bounds.height() as i32 + row_gap) / stride).max(1) as usize;
        let selected_index = self
            .page_state
            .selected_mapping_index
            .min(self.mappings.len().saturating_sub(1));
        let start_index = if self.mappings.len() <= visible_rows {
            0
        } else {
            selected_index
                .saturating_sub(visible_rows / 2)
                .min(self.mappings.len() - visible_rows)
        };
        if selected_index < start_index || selected_index >= start_index + visible_rows {
            return None;
        }

        let visible_index = selected_index - start_index;
        let row = Rect::new(
            list_bounds.x,
            list_bounds.y + visible_index as i32 * stride,
            list_bounds.width(),
            row_height as u32,
        );
        let target_cell = self.mapping_row_cells(row)[mapping_field_index(MappingField::Target)];
        let result_count = self.mapping_target_lookup_results().len().max(1).min(6);
        let panel_width = target_cell.width().max(180);
        let panel_height = 12 + result_count as u32 * 12;
        let panel_x = target_cell.x;
        let preferred_y = target_cell.y + target_cell.height() as i32 + 2;
        let max_y = content_bounds.y + content_bounds.height() as i32 - panel_height as i32 - 24;
        let panel_y = preferred_y.min(max_y.max(content_bounds.y + 28));
        let results_len = self.mapping_target_lookup_results().len();
        let highlighted_index = self
            .target_lookup_state
            .active
            .as_ref()
            .map(|lookup| lookup.highlighted_index)
            .unwrap_or(0)
            .min(results_len.saturating_sub(1));
        let start_index = if results_len <= result_count {
            0
        } else {
            highlighted_index
                .saturating_sub(result_count / 2)
                .min(results_len - result_count)
        };

        Some(MappingTargetLookupLayout {
            target_cell,
            results_panel: Rect::new(panel_x, panel_y, panel_width, panel_height),
            start_index,
            visible_count: result_count,
        })
    }

    pub(super) fn draw_mapping_target_lookup<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        content_bounds: Rect,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let Some(lookup) = self.target_lookup_state.active.as_ref() else {
            return Ok(());
        };
        let Some(layout) = self.mapping_target_lookup_layout(content_bounds) else {
            return Ok(());
        };
        let results = self.mapping_target_lookup_results();

        canvas.set_draw_color(Color::RGB(108, 84, 52));
        canvas.draw_rect(Rect::new(
            layout.target_cell.x - 1,
            layout.target_cell.y - 1,
            layout.target_cell.width().saturating_add(2),
            layout.target_cell.height().saturating_add(2),
        ))?;

        canvas.set_draw_color(Color::RGB(28, 32, 44));
        canvas.fill_rect(layout.results_panel)?;

        let query_text = if lookup.query.is_empty() {
            "Type to search targets".to_string()
        } else {
            format!("Find: {}", lookup.query)
        };
        crate::ui::draw_text_fitted(
            canvas,
            &query_text,
            Rect::new(
                layout.results_panel.x + 6,
                layout.results_panel.y + 4,
                layout.results_panel.width().saturating_sub(12),
                8,
            ),
            1,
            if lookup.query.is_empty() {
                Color::RGB(152, 162, 176)
            } else {
                Color::RGB(242, 238, 228)
            },
        )?;

        for row_index in 0..layout.visible_count {
            let item_rect = Rect::new(
                layout.results_panel.x + 4,
                layout.results_panel.y + 14 + row_index as i32 * 12,
                layout.results_panel.width().saturating_sub(8),
                10,
            );
            let result_index = layout.start_index + row_index;
            let highlighted =
                result_index == lookup.highlighted_index && result_index < results.len();
            canvas.set_draw_color(if highlighted {
                Color::RGB(88, 98, 132)
            } else {
                Color::RGB(40, 46, 62)
            });
            canvas.fill_rect(item_rect)?;

            let label = results
                .get(result_index)
                .copied()
                .unwrap_or("No matching targets");
            crate::ui::draw_text_fitted(
                canvas,
                label,
                Rect::new(
                    item_rect.x + 4,
                    item_rect.y + 1,
                    item_rect.width().saturating_sub(8),
                    8,
                ),
                1,
                if highlighted {
                    Color::RGB(248, 244, 232)
                } else {
                    Color::RGB(210, 216, 224)
                },
            )?;
        }

        canvas.set_draw_color(Color::RGB(164, 142, 96));
        canvas.draw_rect(layout.results_panel)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mappings_target_lookup_uses_canonical_page_actions_while_open() {
        let mut app = App::new();
        app.apply_action(AppAction::ShowPage(AppPage::Mappings));
        app.apply_action(AppAction::ToggleMappingsWriteMode);
        app.page_state.selected_mapping_field = MappingField::Target;
        app.apply_action(AppAction::ActivatePageItem);

        assert!(app.target_lookup_state.active.is_some());
        assert_eq!(
            app.target_lookup_state
                .active
                .as_ref()
                .map(|lookup| lookup.highlighted_index),
            Some(0)
        );

        app.apply_action(AppAction::SelectNextPageItem);
        assert_eq!(
            app.target_lookup_state
                .active
                .as_ref()
                .map(|lookup| lookup.highlighted_index),
            Some(1)
        );

        app.apply_action(AppAction::AdjustPageItemForward);
        assert_eq!(
            app.target_lookup_state
                .active
                .as_ref()
                .map(|lookup| lookup.highlighted_index),
            Some(2)
        );

        let expected = app.mapping_target_lookup_highlighted_label();
        app.apply_action(AppAction::ActivatePageItem);

        assert_eq!(app.mappings[0].target_label.as_str(), expected.unwrap());
        assert!(app.target_lookup_state.active.is_none());
    }

    #[test]
    fn mappings_target_lookup_next_and_previous_clamp_and_scroll_instead_of_wrapping() {
        let mut app = App::new();
        app.apply_action(AppAction::ShowPage(AppPage::Mappings));
        app.apply_action(AppAction::ToggleMappingsWriteMode);
        app.page_state.selected_mapping_field = MappingField::Target;
        app.apply_action(AppAction::ActivatePageItem);

        let result_len = app.mapping_target_lookup_results().len();
        assert!(result_len > 6);

        for _ in 0..(result_len + 3) {
            app.apply_action(AppAction::SelectNextPageItem);
        }
        assert_eq!(
            app.target_lookup_state
                .active
                .as_ref()
                .map(|lookup| lookup.highlighted_index),
            Some(result_len - 1)
        );

        let content_bounds = Rect::new(0, 0, 960, 540);
        let layout = app
            .mapping_target_lookup_layout(content_bounds)
            .expect("lookup layout");
        assert_eq!(layout.visible_count, 6);
        assert_eq!(layout.start_index, result_len - layout.visible_count);

        app.apply_action(AppAction::SelectNextPageItem);
        assert_eq!(
            app.target_lookup_state
                .active
                .as_ref()
                .map(|lookup| lookup.highlighted_index),
            Some(result_len - 1)
        );

        for _ in 0..(result_len + 3) {
            app.apply_action(AppAction::SelectPreviousPageItem);
        }
        assert_eq!(
            app.target_lookup_state
                .active
                .as_ref()
                .map(|lookup| lookup.highlighted_index),
            Some(0)
        );
        let layout = app
            .mapping_target_lookup_layout(content_bounds)
            .expect("lookup layout");
        assert_eq!(layout.start_index, 0);
    }
}
