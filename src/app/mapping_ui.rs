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
}

