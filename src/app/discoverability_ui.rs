use super::*;
use crate::theme::discoverability as discoverability_theme;

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

impl App {
    pub(super) fn discoverability_target_at(&self, x: i32, y: i32) -> Option<DiscoverabilityTarget> {
        if self.overlay_state.active == Some(AppOverlay::MappingsQuickView) {
            return None;
        }
        let surface = crate::ui::surface_rect(self.viewport_size.0, self.viewport_size.1);
        let inset = crate::ui::inset_rect(surface, 24, 24).ok()?;
        let (_, content_bounds, _) = self.page_frame_layout(inset).ok()?;

        let targets =
            page_discoverability_targets(self.page_state.current_page, self, content_bounds);

        targets
            .into_iter()
            .find_map(|(rect, target)| rect_contains(rect, x, y).then_some(target))
    }

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
                canvas.set_draw_color(discoverability_theme::BADGE_OVERFLOW_FILL);
                canvas.fill_rect(chip)?;
                crate::ui::draw_text_fitted(
                    canvas,
                    &draw_label,
                    Rect::new(chip.x + 5, chip.y + 2, chip.width().saturating_sub(10), 8),
                    1,
                    discoverability_theme::BADGE_OVERFLOW_TEXT,
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
            canvas.set_draw_color(discoverability_theme::DIRECT_TAB_TARGET);
            canvas.draw_rect(page.hit_rect)?;
        }

        for target in self.direct_mapping_targets(content_bounds) {
            canvas.set_draw_color(discoverability_theme::DIRECT_TARGET_BORDER);
            canvas.draw_rect(Rect::new(
                target.hit_rect.x - 1,
                target.hit_rect.y - 1,
                target.hit_rect.width().saturating_add(2),
                target.hit_rect.height().saturating_add(2),
            ))?;
            if self.direct_mapping_state.mode == DirectMappingMode::AwaitingInput(target) {
                canvas.set_draw_color(discoverability_theme::DIRECT_TARGET_ACTIVE_BORDER);
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
        let user_count = summary.badges.iter().filter(|badge| !badge.built_in).count();

        if built_in_count > 0 && user_count > 0 {
            let left_width = (slot.width() / 2).max(1);
            let right_width = slot.width().saturating_sub(left_width);
            canvas.set_draw_color(discoverability_theme::SLOT_BUILT_IN_FILL);
            canvas.fill_rect(Rect::new(slot.x, slot.y, left_width, slot.height()))?;
            canvas.set_draw_color(discoverability_theme::SLOT_USER_FILL);
            canvas.fill_rect(Rect::new(
                slot.x + left_width as i32,
                slot.y,
                right_width,
                slot.height(),
            ))?;
        } else if user_count > 0 {
            canvas.set_draw_color(discoverability_theme::SLOT_USER_FILL);
            canvas.fill_rect(slot)?;
        } else {
            canvas.set_draw_color(discoverability_theme::SLOT_BUILT_IN_FILL);
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
            discoverability_theme::SLOT_COUNT_TEXT,
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discoverability_summary_hides_disabled_and_absolute_track_mappings() {
        let mut app = App::new();
        app.mappings = vec![
            MappingEntry {
                source_kind: MappingSourceKind::Midi,
                source_device_label: "Any MIDI".to_string(),
                source_label: "CC20".to_string(),
                target_label: "Track Arm".to_string(),
                scope_label: "Active Track".to_string(),
                enabled: true,
            },
            MappingEntry {
                source_kind: MappingSourceKind::Midi,
                source_device_label: "Any MIDI".to_string(),
                source_label: "CC21".to_string(),
                target_label: "Track Arm".to_string(),
                scope_label: "Track 3".to_string(),
                enabled: true,
            },
            MappingEntry {
                source_kind: MappingSourceKind::Osc,
                source_device_label: default_mapping_source_device(),
                source_label: "/track/active/arm".to_string(),
                target_label: "Track Arm".to_string(),
                scope_label: "Active Track".to_string(),
                enabled: false,
            },
        ];

        let summary = app.summarize_discoverability_target(DiscoverabilityTarget {
            action: AppAction::ToggleCurrentTrackArm,
            display_scope: Some("Active Track"),
            allowed_mapping_scopes: &["Active Track"],
            overlay_slot: None,
        });

        assert!(summary.badges.iter().any(|badge| badge.text == "A"));
        assert!(summary.badges.iter().any(|badge| badge.text == "CC20"));
        assert!(!summary.badges.iter().any(|badge| badge.text == "CC21"));
        assert!(!summary
            .badges
            .iter()
            .any(|badge| badge.text == "/track/active/arm"));
    }

    #[test]
    fn summarize_discoverability_target_includes_note_edit_shortcuts() {
        let app = App::new();

        let summary = app.summarize_discoverability_target(DiscoverabilityTarget {
            action: AppAction::SelectNotesAtPlayhead,
            display_scope: Some("Active Track"),
            allowed_mapping_scopes: &["Active Track"],
            overlay_slot: None,
        });

        assert!(summary.badges.iter().any(|badge| badge.text == "T"));
    }

    #[test]
    fn discoverability_overlay_toggles_separately_from_quick_overlay() {
        let mut app = App::new();

        app.apply_action(AppAction::ToggleDiscoverabilityOverlay);
        assert_eq!(app.overlay_state.active, Some(AppOverlay::Discoverability));

        app.apply_action(AppAction::ToggleMappingsOverlay);
        assert_eq!(
            app.overlay_state.active,
            Some(AppOverlay::MappingsQuickView)
        );
    }
}
