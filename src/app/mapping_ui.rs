use crate::actions::AppAction;
use sdl3::event::Event;
use sdl3::keyboard::{Keycode, Mod};
use sdl3::rect::Rect;

use super::types::DiscoverabilityTarget;

pub(super) fn mapping_target_label_for_action(action: AppAction) -> Option<&'static str> {
    match action {
        AppAction::TogglePlayback => Some("Play/Stop"),
        AppAction::ToggleRecording => Some("Record"),
        AppAction::CycleRecordMode => Some("Record Mode"),
        AppAction::ToggleLoopRecordingExtension => Some("Loop Recording Wrap"),
        AppAction::ToggleGlobalLoop => Some("Song Loop"),
        AppAction::CycleGlobalHarmonyRoot => Some("Global Harmony Root"),
        AppAction::ResetGlobalLoop => Some("Reset Song Loop"),
        AppAction::ToggleCurrentTrackLoop => Some("Track Loop"),
        AppAction::ToggleStoredLoopRecallQuantize => Some("Stored Loop Recall Quantize"),
        AppAction::CycleStoredLoopLaunchQuantize => Some("Stored Loop Launch Quantize"),
        AppAction::RecallStoredLoopSlot1 => Some("Recall Stored Loop Slot 1"),
        AppAction::RecallStoredLoopSlot2 => Some("Recall Stored Loop Slot 2"),
        AppAction::RecallStoredLoopSlot3 => Some("Recall Stored Loop Slot 3"),
        AppAction::RecallStoredLoopSlot4 => Some("Recall Stored Loop Slot 4"),
        AppAction::RecallStoredLoopSlot5 => Some("Recall Stored Loop Slot 5"),
        AppAction::RecallStoredLoopSlot6 => Some("Recall Stored Loop Slot 6"),
        AppAction::RecallStoredLoopSlot7 => Some("Recall Stored Loop Slot 7"),
        AppAction::RecallStoredLoopSlot8 => Some("Recall Stored Loop Slot 8"),
        AppAction::StoreCurrentLoopToSlot1 => Some("Store Current Loop To Slot 1"),
        AppAction::StoreCurrentLoopToSlot2 => Some("Store Current Loop To Slot 2"),
        AppAction::StoreCurrentLoopToSlot3 => Some("Store Current Loop To Slot 3"),
        AppAction::StoreCurrentLoopToSlot4 => Some("Store Current Loop To Slot 4"),
        AppAction::StoreCurrentLoopToSlot5 => Some("Store Current Loop To Slot 5"),
        AppAction::StoreCurrentLoopToSlot6 => Some("Store Current Loop To Slot 6"),
        AppAction::StoreCurrentLoopToSlot7 => Some("Store Current Loop To Slot 7"),
        AppAction::StoreCurrentLoopToSlot8 => Some("Store Current Loop To Slot 8"),
        AppAction::ClearStoredLoopSlot1 => Some("Clear Stored Loop Slot 1"),
        AppAction::ClearStoredLoopSlot2 => Some("Clear Stored Loop Slot 2"),
        AppAction::ClearStoredLoopSlot3 => Some("Clear Stored Loop Slot 3"),
        AppAction::ClearStoredLoopSlot4 => Some("Clear Stored Loop Slot 4"),
        AppAction::ClearStoredLoopSlot5 => Some("Clear Stored Loop Slot 5"),
        AppAction::ClearStoredLoopSlot6 => Some("Clear Stored Loop Slot 6"),
        AppAction::ClearStoredLoopSlot7 => Some("Clear Stored Loop Slot 7"),
        AppAction::ClearStoredLoopSlot8 => Some("Clear Stored Loop Slot 8"),
        AppAction::ToggleCurrentTrackArm => Some("Track Arm"),
        AppAction::ToggleCurrentTrackMute => Some("Track Mute"),
        AppAction::ToggleCurrentTrackSolo => Some("Track Solo"),
        AppAction::ToggleCurrentTrackPassthrough => Some("Passthrough"),
        AppAction::ToggleCurrentTrackRecordingView => Some("Recording View"),
        AppAction::SelectPreviousRecordingClip => Some("Select Previous Recording Clip"),
        AppAction::SelectNextRecordingClip => Some("Select Next Recording Clip"),
        AppAction::ToggleSelectedRecordingClipMute => Some("Recording Clip Mute"),
        AppAction::DeleteSelectedRecordingClip => Some("Delete Recording Clip"),
        AppAction::ToggleFocusedTrackView => Some("Focused Track View"),
        AppAction::ToggleLinkEnabled => Some("Link Enable"),
        AppAction::ToggleLinkStartStopSync => Some("Link Start/Stop"),
        _ => None,
    }
}

pub(super) fn direct_mapping_key_label(event: &Event) -> Option<String> {
    let Event::KeyDown {
        keycode: Some(keycode),
        keymod,
        repeat: false,
        ..
    } = event
    else {
        return None;
    };

    if matches!(
        keycode,
        Keycode::LShift
            | Keycode::RShift
            | Keycode::LCtrl
            | Keycode::RCtrl
            | Keycode::LAlt
            | Keycode::RAlt
            | Keycode::LGui
            | Keycode::RGui
            | Keycode::Mode
            | Keycode::Escape
            | Keycode::F8
    ) {
        return None;
    }

    let key_label = keycode_mapping_label(*keycode)?;
    Some(with_modifier_prefixes(key_label, *keymod))
}

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

fn with_modifier_prefixes(key_label: &str, keymod: Mod) -> String {
    let mut label = String::new();
    if keymod.intersects(Mod::LCTRLMOD | Mod::RCTRLMOD) {
        label.push_str("Ctrl+");
    }
    if keymod.intersects(Mod::LALTMOD | Mod::RALTMOD) {
        label.push_str("Alt+");
    }
    if keymod.intersects(Mod::LSHIFTMOD | Mod::RSHIFTMOD) {
        label.push_str("Shift+");
    }
    label.push_str(key_label);
    label
}

fn keycode_mapping_label(keycode: Keycode) -> Option<&'static str> {
    match keycode {
        Keycode::Space => Some("Space"),
        Keycode::Tab => Some("Tab"),
        Keycode::Return => Some("Enter"),
        Keycode::Delete => Some("Delete"),
        Keycode::Backspace => Some("Backspace"),
        Keycode::Home => Some("Home"),
        Keycode::Left => Some("Left"),
        Keycode::Right => Some("Right"),
        Keycode::Up => Some("Up"),
        Keycode::Down => Some("Down"),
        Keycode::LeftBracket => Some("["),
        Keycode::RightBracket => Some("]"),
        Keycode::Comma => Some(","),
        Keycode::Period => Some("."),
        Keycode::Minus => Some("-"),
        Keycode::Equals => Some("="),
        Keycode::Slash => Some("/"),
        Keycode::Backslash => Some("\\"),
        Keycode::F1 => Some("F1"),
        Keycode::F2 => Some("F2"),
        Keycode::F3 => Some("F3"),
        Keycode::F4 => Some("F4"),
        Keycode::F5 => Some("F5"),
        Keycode::F6 => Some("F6"),
        Keycode::_0 => Some("0"),
        Keycode::_1 => Some("1"),
        Keycode::_2 => Some("2"),
        Keycode::_3 => Some("3"),
        Keycode::_4 => Some("4"),
        Keycode::_5 => Some("5"),
        Keycode::_6 => Some("6"),
        Keycode::_7 => Some("7"),
        Keycode::_8 => Some("8"),
        Keycode::_9 => Some("9"),
        Keycode::Kp1 => Some("Numpad1"),
        Keycode::Kp2 => Some("Numpad2"),
        Keycode::Kp3 => Some("Numpad3"),
        Keycode::Kp4 => Some("Numpad4"),
        Keycode::Kp5 => Some("Numpad5"),
        Keycode::Kp6 => Some("Numpad6"),
        Keycode::Kp7 => Some("Numpad7"),
        Keycode::Kp8 => Some("Numpad8"),
        Keycode::A => Some("A"),
        Keycode::B => Some("B"),
        Keycode::C => Some("C"),
        Keycode::D => Some("D"),
        Keycode::E => Some("E"),
        Keycode::F => Some("F"),
        Keycode::G => Some("G"),
        Keycode::H => Some("H"),
        Keycode::I => Some("I"),
        Keycode::J => Some("J"),
        Keycode::K => Some("K"),
        Keycode::L => Some("L"),
        Keycode::M => Some("M"),
        Keycode::N => Some("N"),
        Keycode::O => Some("O"),
        Keycode::P => Some("P"),
        Keycode::Q => Some("Q"),
        Keycode::R => Some("R"),
        Keycode::S => Some("S"),
        Keycode::T => Some("T"),
        Keycode::U => Some("U"),
        Keycode::V => Some("V"),
        Keycode::W => Some("W"),
        Keycode::X => Some("X"),
        Keycode::Y => Some("Y"),
        Keycode::Z => Some("Z"),
        _ => None,
    }
}

pub(super) fn track_indicator_target(
    kind: crate::ui::TrackIndicatorKind,
    overlay_slot: Option<Rect>,
) -> Option<DiscoverabilityTarget> {
    match kind {
        crate::ui::TrackIndicatorKind::Armed => Some(DiscoverabilityTarget {
            action: AppAction::ToggleCurrentTrackArm,
            display_scope: Some("Active Track"),
            allowed_mapping_scopes: &["Active Track"],
            overlay_slot,
        }),
        crate::ui::TrackIndicatorKind::Recording => Some(DiscoverabilityTarget {
            action: AppAction::ToggleRecording,
            display_scope: Some("Armed/Active"),
            allowed_mapping_scopes: &["Armed/Active", "Active Track"],
            overlay_slot,
        }),
        crate::ui::TrackIndicatorKind::Muted => Some(DiscoverabilityTarget {
            action: AppAction::ToggleCurrentTrackMute,
            display_scope: Some("Active Track"),
            allowed_mapping_scopes: &["Active Track"],
            overlay_slot,
        }),
        crate::ui::TrackIndicatorKind::Solo => Some(DiscoverabilityTarget {
            action: AppAction::ToggleCurrentTrackSolo,
            display_scope: Some("Active Track"),
            allowed_mapping_scopes: &["Active Track"],
            overlay_slot,
        }),
    }
}

use super::*;

impl App {
    pub(super) fn direct_mapping_footer_content(
        &self,
    ) -> Option<(String, String, Vec<MappingBadge>)> {
        match self.direct_mapping_state.mode {
            DirectMappingMode::Inactive => {
                if self.status_state.hovered_target.is_some() {
                    None
                } else {
                    self.direct_mapping_state
                        .status_message
                        .as_ref()
                        .map(|message| ("Direct Map".to_string(), message.clone(), Vec::new()))
                }
            }
            DirectMappingMode::Targeting => Some((
                "Direct Map".to_string(),
                self.direct_mapping_state.status_message.clone().unwrap_or_else(|| {
                    "Select a highlighted control, then move the next MIDI control or key. Esc cancels."
                        .to_string()
                }),
                Vec::new(),
            )),
            DirectMappingMode::AwaitingInput(target) => {
                let title = match target.display_scope {
                    Some(scope) => format!("Direct Map: {} ({scope})", target.target_label),
                    None => format!("Direct Map: {}", target.target_label),
                };
                Some((
                    title,
                    "Move the next MIDI note, CC, or key now. Esc cancels.".to_string(),
                    self.summarize_direct_mapping_target(target).badges,
                ))
            }
        }
    }

    pub(super) fn summarize_direct_mapping_target(
        &self,
        target: DirectMappingTarget,
    ) -> ActionDiscoverabilitySummary {
        let mut summary = self.summarize_discoverability_target(DiscoverabilityTarget {
            action: target.action,
            display_scope: target.display_scope,
            allowed_mapping_scopes: &[],
            overlay_slot: None,
        });
        summary.badges.retain(|badge| {
            badge.built_in || self.direct_mapping_badge_matches_scope(badge, target)
        });
        summary.total_bindings = summary.badges.len();
        summary
    }

    pub(super) fn direct_mapping_badge_matches_scope(
        &self,
        badge: &MappingBadge,
        target: DirectMappingTarget,
    ) -> bool {
        self.mappings.iter().any(|entry| {
            !badge.built_in
                && entry.enabled
                && entry.scope_label == target.scope_label
                && entry.source_kind == badge.source_kind
                && entry.source_label == badge.text
                && mapping_entry_targets_action(entry, target.action)
        })
    }

    pub(super) fn summarize_discoverability_target(
        &self,
        target: DiscoverabilityTarget,
    ) -> ActionDiscoverabilitySummary {
        let mut badges = built_in_keyboard_binding_labels(target.action)
            .iter()
            .map(|label| MappingBadge {
                text: (*label).to_string(),
                source_kind: MappingSourceKind::Key,
                built_in: true,
            })
            .collect::<Vec<_>>();

        badges.extend(self.mappings.iter().filter_map(|entry| {
            if !mapping_entry_targets_action(entry, target.action) {
                return None;
            }
            if !target.allowed_mapping_scopes.is_empty()
                && !target
                    .allowed_mapping_scopes
                    .iter()
                    .any(|scope| *scope == entry.scope_label.as_str())
            {
                return None;
            }
            Some(MappingBadge {
                text: entry.source_label.clone(),
                source_kind: entry.source_kind,
                built_in: false,
            })
        }));

        badges.sort_by_key(|badge| {
            (
                mapping_source_sort_key(badge.source_kind),
                if badge.built_in { 0 } else { 1 },
                badge.text.clone(),
            )
        });
        badges.dedup_by(|left, right| {
            left.text == right.text
                && left.source_kind == right.source_kind
                && left.built_in == right.built_in
        });

        let title = match target.display_scope {
            Some(scope) => format!("{} ({scope})", action_label(target.action)),
            None => action_label(target.action).to_string(),
        };
        let total_bindings = badges.len();

        ActionDiscoverabilitySummary {
            title,
            badges,
            total_bindings,
        }
    }

    pub(super) fn draw_mapping_badges<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        bounds: Rect,
        badges: &[MappingBadge],
        total_bindings: usize,
        max_badges: usize,
        max_label_width: usize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut cursor_x = bounds.x;
        let visible = badges.len().min(max_badges);
        for badge in badges.iter().take(visible) {
            let label = compact_badge_text(&badge.text, max_label_width);
            let draw_label = format!("{} {}", badge_kind_prefix(badge.source_kind), label);
            let width = crate::ui::text_width(&draw_label, 1) + 10;
            if cursor_x + width as i32 > bounds.x + bounds.width() as i32 {
                break;
            }
            let chip = Rect::new(
                cursor_x,
                bounds.y + 2,
                width,
                bounds.height().saturating_sub(4),
            );
            let (fill, text) = mapping_badge_palette(badge);
            canvas.set_draw_color(fill);
            canvas.fill_rect(chip)?;
            crate::ui::draw_text_fitted(
                canvas,
                &draw_label,
                Rect::new(chip.x + 5, chip.y + 2, chip.width().saturating_sub(10), 8),
                1,
                text,
            )?;
            cursor_x += chip.width() as i32 + 4;
        }

        let remaining = total_bindings.saturating_sub(visible);
        if remaining > 0 {
            let draw_label = format!("+{remaining}");
            let width = crate::ui::text_width(&draw_label, 1) + 10;
            if cursor_x + width as i32 <= bounds.x + bounds.width() as i32 {
                let chip = Rect::new(
                    cursor_x,
                    bounds.y + 2,
                    width,
                    bounds.height().saturating_sub(4),
                );
                canvas.set_draw_color(Color::RGB(56, 64, 80));
                canvas.fill_rect(chip)?;
                crate::ui::draw_text_fitted(
                    canvas,
                    &draw_label,
                    Rect::new(chip.x + 5, chip.y + 2, chip.width().saturating_sub(10), 8),
                    1,
                    Color::RGB(228, 232, 238),
                )?;
            }
        }

        Ok(())
    }

    pub(super) fn draw_timeline_discoverability_overlay<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        content_bounds: Rect,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for (rect, target) in self.timeline_discoverability_targets(content_bounds) {
            self.draw_inline_discoverability_badges(canvas, rect, target)?;
        }
        Ok(())
    }

    pub(super) fn draw_routing_discoverability_overlay<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        content_bounds: Rect,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for (rect, target) in self.routing_discoverability_targets(content_bounds) {
            self.draw_inline_discoverability_badges(canvas, rect, target)?;
        }
        Ok(())
    }

    pub(super) fn draw_direct_mapping_targets<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        tabs_bounds: Rect,
        content_bounds: Rect,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if self.direct_mapping_state.mode == DirectMappingMode::Inactive {
            return Ok(());
        }

        for page in self.direct_mapping_tab_targets(tabs_bounds) {
            canvas.set_draw_color(Color::RGB(132, 84, 84));
            canvas.draw_rect(page.hit_rect)?;
        }

        for target in self.direct_mapping_targets(content_bounds) {
            canvas.set_draw_color(Color::RGB(176, 116, 72));
            canvas.draw_rect(Rect::new(
                target.hit_rect.x - 1,
                target.hit_rect.y - 1,
                target.hit_rect.width().saturating_add(2),
                target.hit_rect.height().saturating_add(2),
            ))?;
            if self.direct_mapping_state.mode == DirectMappingMode::AwaitingInput(target) {
                canvas.set_draw_color(Color::RGB(252, 146, 126));
                canvas.draw_rect(Rect::new(
                    target.hit_rect.x - 3,
                    target.hit_rect.y - 3,
                    target.hit_rect.width().saturating_add(6),
                    target.hit_rect.height().saturating_add(6),
                ))?;
            }
        }

        Ok(())
    }

    pub(super) fn draw_inline_discoverability_badges<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        anchor: Rect,
        target: DiscoverabilityTarget,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let summary = self.summarize_discoverability_target(target);
        if summary.badges.is_empty() {
            return Ok(());
        }

        if let Some(slot) = target.overlay_slot {
            return self.draw_compact_discoverability_slot(canvas, slot, &summary);
        }

        let max_badges = if anchor.width() <= 24 || anchor.height() <= 12 {
            1
        } else {
            2
        };
        let badge_height = 10_u32;
        let label_width = if max_badges == 1 { 4 } else { 6 };
        let y = if anchor.height() <= 12 {
            anchor.y - badge_height as i32 - 2
        } else {
            anchor.y + 2
        };
        let x = if anchor.width() >= 44 {
            anchor.x + anchor.width() as i32 - 32
        } else {
            anchor.x + anchor.width() as i32 + 3
        };
        let bounds = Rect::new(x, y, 72, badge_height + 4);
        self.draw_mapping_badges(
            canvas,
            bounds,
            &summary.badges,
            summary.total_bindings,
            max_badges,
            label_width,
        )
    }

    pub(super) fn draw_compact_discoverability_slot<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        slot: Rect,
        summary: &ActionDiscoverabilitySummary,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let built_in_count = summary.badges.iter().filter(|badge| badge.built_in).count();
        let user_count = summary
            .badges
            .iter()
            .filter(|badge| !badge.built_in)
            .count();

        if built_in_count > 0 && user_count > 0 {
            let left_width = (slot.width() / 2).max(1);
            let right_width = slot.width().saturating_sub(left_width);
            canvas.set_draw_color(Color::RGB(64, 84, 126));
            canvas.fill_rect(Rect::new(slot.x, slot.y, left_width, slot.height()))?;
            canvas.set_draw_color(Color::RGB(88, 128, 76));
            canvas.fill_rect(Rect::new(
                slot.x + left_width as i32,
                slot.y,
                right_width,
                slot.height(),
            ))?;
        } else if user_count > 0 {
            canvas.set_draw_color(Color::RGB(88, 128, 76));
            canvas.fill_rect(slot)?;
        } else {
            canvas.set_draw_color(Color::RGB(64, 84, 126));
            canvas.fill_rect(slot)?;
        }

        let count_text = if summary.total_bindings >= 10 {
            "+".to_string()
        } else {
            summary.total_bindings.to_string()
        };
        crate::ui::draw_text_fitted(
            canvas,
            &count_text,
            Rect::new(
                slot.x + 1,
                slot.y + 1,
                slot.width().saturating_sub(2),
                slot.height().saturating_sub(2),
            ),
            1,
            Color::RGB(244, 244, 236),
        )?;

        Ok(())
    }

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
        let Some(entry) = self
            .mappings
            .get_mut(self.page_state.selected_mapping_index)
        else {
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
        let Some(entry) = self
            .mappings
            .get_mut(self.page_state.selected_mapping_index)
        else {
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
