use super::*;
use crate::theme::{app_chrome, mappings as mappings_theme};
use crate::actions::AppAction;
use sdl3::event::Event;
use sdl3::keyboard::{Keycode, Mod};
 
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



impl App {

    pub(super) fn mapping_row_cells(&self, row: Rect) -> [Rect; 6] {
        let type_rect = Rect::new(row.x + 4, row.y + 3, 46, row.height().saturating_sub(6));
        let source_rect = Rect::new(
            type_rect.x + type_rect.width() as i32 + 6,
            row.y + 3,
            92,
            row.height().saturating_sub(6),
        );
        let device_rect = Rect::new(
            source_rect.x + source_rect.width() as i32 + 6,
            row.y + 3,
            98,
            row.height().saturating_sub(6),
        );
        let enabled_rect = Rect::new(
            row.x + row.width() as i32 - 34,
            row.y + 3,
            28,
            row.height().saturating_sub(6),
        );
        let scope_rect = Rect::new(
            enabled_rect.x - 80,
            row.y + 3,
            72,
            row.height().saturating_sub(6),
        );
        let target_rect = Rect::new(
            device_rect.x + device_rect.width() as i32 + 6,
            row.y + 3,
            (scope_rect.x - (device_rect.x + device_rect.width() as i32 + 12)).max(48) as u32,
            row.height().saturating_sub(6),
        );
        [
            type_rect,
            source_rect,
            device_rect,
            target_rect,
            scope_rect,
            enabled_rect,
        ]
    }

    pub(crate) fn draw_mappings_page<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        content_bounds: Rect,
    ) -> Result<(), Box<dyn std::error::Error>> {
        canvas.set_draw_color(mappings_theme::PAGE_BG);
        canvas.fill_rect(content_bounds)?;
        canvas.set_draw_color(app_chrome::SURFACE_BORDER);
        canvas.draw_rect(content_bounds)?;
        crate::ui::draw_text_fitted(
            canvas,
            "Mappings",
            Rect::new(content_bounds.x + 8, content_bounds.y + 8, 180, 14),
            2,
            mappings_theme::PAGE_TITLE,
        )?;
        let overview_badge = Rect::new(content_bounds.x + 200, content_bounds.y + 8, 188, 16);
        canvas.set_draw_color(if self.page_state.mapping_mode == MappingPageMode::Write {
            mappings_theme::WRITE_MODE_ACTIVE
        } else {
            mappings_theme::WRITE_MODE_INACTIVE
        });
        canvas.fill_rect(overview_badge)?;
        canvas.set_draw_color(mappings_theme::PAGE_TITLE);
        canvas.draw_rect(overview_badge)?;
        crate::ui::draw_text_fitted(
            canvas,
            &format!("Tap Mode: {}", self.page_state.mapping_mode.label()),
            Rect::new(content_bounds.x + 208, content_bounds.y + 12, 170, 8),
            1,
            mappings_theme::OVERVIEW_TEXT,
        )?;
        let learn_badge = Rect::new(content_bounds.x + 392, content_bounds.y + 8, 136, 16);
        canvas.set_draw_color(if self.page_state.mapping_midi_learn_armed {
            mappings_theme::LEARN_ARMED
        } else {
            mappings_theme::LEARN_IDLE
        });
        canvas.fill_rect(learn_badge)?;
        canvas.set_draw_color(
            if self.page_state.selected_mapping_field == MappingField::SourceValue
                && self.page_state.mapping_mode == MappingPageMode::Write
            {
                mappings_theme::LEARN_SELECTED_BORDER
            } else {
                mappings_theme::LEARN_IDLE_BORDER
            },
        );
        canvas.draw_rect(learn_badge)?;
        crate::ui::draw_text_fitted(
            canvas,
            if self.page_state.mapping_midi_learn_armed {
                "Tap Learn: waiting"
            } else {
                "Tap Learn: idle"
            },
            Rect::new(learn_badge.x + 8, learn_badge.y + 4, 120, 8),
            1,
            mappings_theme::LEARN_TEXT,
        )?;
        let direct_badge = Rect::new(content_bounds.x + 532, content_bounds.y + 8, 154, 16);
        canvas.set_draw_color(
            if self.direct_mapping_state.mode == DirectMappingMode::Inactive {
                mappings_theme::DIRECT_BADGE_IDLE_FILL
            } else {
                mappings_theme::DIRECT_ARMED_FILL
            },
        );
        canvas.fill_rect(direct_badge)?;
        canvas.set_draw_color(
            if self.direct_mapping_state.mode == DirectMappingMode::Inactive {
                mappings_theme::DIRECT_IDLE_BORDER
            } else {
                mappings_theme::DIRECT_ARMED_BORDER
            },
        );
        canvas.draw_rect(direct_badge)?;
        crate::ui::draw_text_fitted(
            canvas,
            if self.direct_mapping_state.mode == DirectMappingMode::Inactive {
                "Tap Direct Map"
            } else {
                "Tap Direct: armed"
            },
            Rect::new(direct_badge.x + 8, direct_badge.y + 4, 138, 8),
            1,
            mappings_theme::DIRECT_TEXT,
        )?;
        crate::ui::draw_text_fitted(
            canvas,
            &format!(
                "Rows {} / {}",
                self.page_state
                    .selected_mapping_index
                    .saturating_add(1)
                    .min(self.mappings.len()),
                self.mappings.len()
            ),
            Rect::new(
                content_bounds.x + content_bounds.width() as i32 - 100,
                content_bounds.y + 12,
                92,
                8,
            ),
            1,
            mappings_theme::META_TEXT,
        )?;

        let footer_bounds = Rect::new(
            content_bounds.x + 8,
            content_bounds.y + content_bounds.height() as i32 - 20,
            content_bounds.width().saturating_sub(16),
            12,
        );
        let list_bounds = Rect::new(
            content_bounds.x + 8,
            content_bounds.y + 44,
            content_bounds.width().saturating_sub(16),
            content_bounds.height().saturating_sub(68),
        );
        let header_row = Rect::new(
            list_bounds.x,
            content_bounds.y + 30,
            list_bounds.width(),
            10,
        );
        let header_cells = self.mapping_row_cells(Rect::new(
            header_row.x,
            header_row.y,
            header_row.width(),
            18,
        ));
        for (index, field) in MappingField::ALL.iter().enumerate() {
            crate::ui::draw_text_fitted(
                canvas,
                field.label(),
                Rect::new(
                    header_cells[index].x,
                    header_row.y,
                    header_cells[index].width(),
                    8,
                ),
                1,
                Color::RGB(154, 166, 182),
            )?;
        }
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

        for visible_index in 0..visible_rows {
            let index = start_index + visible_index;
            if index >= self.mappings.len() {
                break;
            }
            let row = Rect::new(
                list_bounds.x,
                list_bounds.y + visible_index as i32 * stride,
                list_bounds.width(),
                row_height as u32,
            );
            let entry = &self.mappings[index];
            let selected = index == self.page_state.selected_mapping_index;
            canvas.set_draw_color(if selected {
                mappings_theme::ROW_SELECTED_FILL
            } else {
                mappings_theme::ROW_IDLE_FILL
            });
            canvas.fill_rect(row)?;
            canvas.set_draw_color(if selected {
                mappings_theme::PAGE_TITLE
            } else {
                mappings_theme::ROW_IDLE_BORDER
            });
            canvas.draw_rect(row)?;

            let cells = self.mapping_row_cells(row);
            let source_rect = Rect::new(cells[0].x, cells[0].y, 14, cells[0].height());
            let source_color = match entry.source_kind {
                MappingSourceKind::Key => mappings_theme::SOURCE_KIND_KEY,
                MappingSourceKind::Midi => mappings_theme::SOURCE_KIND_MIDI,
                MappingSourceKind::Osc => mappings_theme::SOURCE_KIND_OSC,
            };
            canvas.set_draw_color(source_color);
            canvas.fill_rect(source_rect)?;

            let enabled_rect = Rect::new(cells[5].x + 6, cells[5].y, 14, cells[5].height());
            canvas.set_draw_color(if entry.enabled {
                mappings_theme::ENABLED_FILL_ON
            } else {
                mappings_theme::ENABLED_FILL_OFF
            });
            canvas.fill_rect(enabled_rect)?;

            let kind_rect = cells[0];
            let device_rect = cells[1];
            let trigger_rect = cells[2];
            let target_rect = cells[3];
            let scope_rect = cells[4];
            canvas.set_draw_color(if selected {
                mappings_theme::FIELD_FILL_SELECTED
            } else {
                mappings_theme::FIELD_FILL_IDLE
            });
            canvas.fill_rect(kind_rect)?;
            canvas.fill_rect(trigger_rect)?;
            canvas.fill_rect(device_rect)?;
            canvas.set_draw_color(if entry.enabled {
                mappings_theme::TARGET_FILL_ENABLED
            } else {
                mappings_theme::TARGET_FILL_DISABLED
            });
            canvas.fill_rect(target_rect)?;
            canvas.set_draw_color(mappings_theme::SCOPE_FILL);
            canvas.fill_rect(scope_rect)?;
            canvas.fill_rect(cells[5])?;
            if selected && self.page_state.mapping_mode == MappingPageMode::Write {
                let field_rect = cells[mapping_field_index(self.page_state.selected_mapping_field)];
                canvas.set_draw_color(
                    if self.page_state.mapping_midi_learn_armed
                        && self.page_state.selected_mapping_field == MappingField::SourceValue
                    {
                        mappings_theme::WRITE_FIELD_LEARN
                    } else {
                        mappings_theme::WRITE_FIELD_ACTIVE
                    },
                );
                canvas.fill_rect(field_rect)?;
            }
            crate::ui::draw_text_fitted(
                canvas,
                mapping_source_label(entry.source_kind),
                Rect::new(
                    kind_rect.x + 18,
                    row.y + 5,
                    kind_rect.width().saturating_sub(22),
                    8,
                ),
                1,
                app_chrome::ACTION_TEXT,
            )?;
            crate::ui::draw_text_fitted(
                canvas,
                &entry.source_label,
                Rect::new(
                    trigger_rect.x + 4,
                    row.y + 5,
                    trigger_rect.width().saturating_sub(8),
                    8,
                ),
                1,
                app_chrome::ACTION_TEXT,
            )?;
            let mapping_device_label = if entry.source_kind == MappingSourceKind::Midi {
                if entry.source_device_label != default_mapping_source_device()
                    && !self.input_port_is_available(&entry.source_device_label)
                {
                    format!("{} (offline)", entry.source_device_label)
                } else {
                    entry.source_device_label.clone()
                }
            } else {
                "--".to_string()
            };
            crate::ui::draw_text_fitted(
                canvas,
                &mapping_device_label,
                Rect::new(
                    device_rect.x + 4,
                    row.y + 5,
                    device_rect.width().saturating_sub(8),
                    8,
                ),
                1,
                if entry.source_kind == MappingSourceKind::Midi {
                    mappings_theme::DEVICE_TEXT_ACTIVE
                } else {
                    mappings_theme::DEVICE_TEXT_INACTIVE
                },
            )?;
            crate::ui::draw_text_fitted(
                canvas,
                &if selected
                    && self.page_state.mapping_mode == MappingPageMode::Write
                    && self.page_state.selected_mapping_field == MappingField::Target
                    && self.target_lookup_state.active.is_some()
                {
                    self.target_lookup_state
                        .active
                        .as_ref()
                        .map(|lookup| {
                            if lookup.query.is_empty() {
                                "Search target…".to_string()
                            } else {
                                format!("Search: {}", lookup.query)
                            }
                        })
                        .unwrap_or_else(|| entry.target_label.clone())
                } else {
                    entry.target_label.clone()
                },
                Rect::new(
                    target_rect.x + 4,
                    row.y + 5,
                    target_rect.width().saturating_sub(8),
                    8,
                ),
                1,
                mappings_theme::TARGET_TEXT,
            )?;
            crate::ui::draw_text_fitted(
                canvas,
                compact_scope_label(&entry.scope_label),
                Rect::new(
                    scope_rect.x + 4,
                    row.y + 5,
                    scope_rect.width().saturating_sub(8),
                    8,
                ),
                1,
                mappings_theme::SCOPE_TEXT,
            )?;
            crate::ui::draw_text_fitted(
                canvas,
                if entry.enabled { "On" } else { "Off" },
                Rect::new(
                    cells[5].x + 2,
                    row.y + 5,
                    cells[5].width().saturating_sub(4),
                    8,
                ),
                1,
                mappings_theme::SCOPE_TEXT,
            )?;

            if selected && self.page_state.mapping_mode == MappingPageMode::Write {
                let field_rect = cells[mapping_field_index(self.page_state.selected_mapping_field)];
                canvas.set_draw_color(
                    if self.page_state.mapping_midi_learn_armed
                        && self.page_state.selected_mapping_field == MappingField::SourceValue
                    {
                        mappings_theme::WRITE_FIELD_BORDER_LEARN
                    } else {
                        mappings_theme::WRITE_FIELD_BORDER
                    },
                );
                canvas.draw_rect(field_rect)?;
                let tap_tag = Rect::new(row.x + row.width() as i32 - 68, row.y + 3, 34, 12);
                canvas.set_draw_color(mappings_theme::TAP_BADGE_FILL);
                canvas.fill_rect(tap_tag)?;
                crate::ui::draw_text_fitted(
                    canvas,
                    "Tap",
                    Rect::new(
                        tap_tag.x + 6,
                        tap_tag.y + 2,
                        tap_tag.width().saturating_sub(12),
                        8,
                    ),
                    1,
                    app_chrome::ACTION_TEXT,
                )?;
            }
        }

        self.draw_mapping_target_lookup(canvas, content_bounds)?;

        canvas.set_draw_color(mappings_theme::FOOTER_BG);
        canvas.fill_rect(footer_bounds)?;
        let footer_tokens = [
            ("Tap row", mappings_theme::FOOTER_TOKEN_ROW),
            ("Tap field", mappings_theme::FOOTER_TOKEN_FIELD),
            ("Tap again act", mappings_theme::FOOTER_TOKEN_ACT),
            ("W Write", mappings_theme::FOOTER_TOKEN_WRITE),
            ("F8 Direct", mappings_theme::FOOTER_TOKEN_DIRECT),
            ("N New", mappings_theme::FOOTER_TOKEN_NEW),
            ("Del/Bsp Remove", mappings_theme::FOOTER_TOKEN_REMOVE),
        ];
        let mut footer_x = footer_bounds.x + 6;
        for (label, fill) in footer_tokens {
            let token = Rect::new(
                footer_x,
                footer_bounds.y + 1,
                crate::ui::text_width(label, 1) + 12,
                footer_bounds.height().saturating_sub(2),
            );
            canvas.set_draw_color(fill);
            canvas.fill_rect(token)?;
            crate::ui::draw_text_fitted(
                canvas,
                label,
                Rect::new(
                    token.x + 6,
                    token.y + 2,
                    token.width().saturating_sub(12),
                    8,
                ),
                1,
                app_chrome::ACTION_TEXT,
            )?;
            footer_x += token.width() as i32 + 6;
        }
        crate::ui::draw_text_fitted(
            canvas,
            if self.target_lookup_state.active.is_some() {
                "Type filter  Up/Down Select  Enter Commit  Esc Cancel  Tab stays in lookup"
            } else {
                "Shift+Left/Right Field  Q/E Adjust  Enter Learn/Toggle"
            },
            Rect::new(
                footer_x + 6,
                footer_bounds.y + 2,
                footer_bounds
                    .width()
                    .saturating_sub((footer_x - footer_bounds.x) as u32)
                    .saturating_sub(12),
                8,
            ),
            1,
            mappings_theme::META_TEXT,
        )?;

        Ok(())
    }

    pub(super) fn draw_mappings_overlay<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        bounds: Rect,
    ) -> Result<(), Box<dyn std::error::Error>> {
        canvas.set_draw_color(app_chrome::OVERLAY_BACKDROP);
        canvas.fill_rect(bounds)?;

        let panel = Rect::new(
            bounds.x + 84,
            bounds.y + 44,
            bounds.width() - 168,
            bounds.height() - 88,
        );
        canvas.set_draw_color(app_chrome::OVERLAY_PANEL_FILL);
        canvas.fill_rect(panel)?;
        canvas.set_draw_color(mappings_theme::PAGE_TITLE);
        canvas.draw_rect(panel)?;
        let title_bounds = Rect::new(panel.x + 12, panel.y + 12, 220, 14);
        crate::ui::draw_text_fitted(
            canvas,
            "Mappings Overlay",
            title_bounds,
            2,
            mappings_theme::PAGE_TITLE,
        )?;
        crate::ui::draw_text_fitted(
            canvas,
            "F5 Close",
            Rect::new(panel.x + 12, panel.y + 32, 58, 8),
            1,
            app_chrome::DETAIL_TEXT,
        )?;
        crate::ui::draw_text_fitted(
            canvas,
            "W Write",
            Rect::new(panel.x + 80, panel.y + 32, 52, 8),
            1,
            app_chrome::DETAIL_TEXT,
        )?;
        crate::ui::draw_text_fitted(
            canvas,
            "Trigger",
            Rect::new(panel.x + 12, panel.y + 46, 56, 8),
            1,
            app_chrome::OVERLAY_HEADER_TEXT,
        )?;
        crate::ui::draw_text_fitted(
            canvas,
            "Action",
            Rect::new(panel.x + 146, panel.y + 46, 48, 8),
            1,
            app_chrome::OVERLAY_HEADER_TEXT,
        )?;
        crate::ui::draw_text_fitted(
            canvas,
            "Scope",
            Rect::new(panel.x + panel.width() as i32 - 126, panel.y + 46, 44, 8),
            1,
            app_chrome::OVERLAY_HEADER_TEXT,
        )?;

        let list_bounds = crate::ui::inset_rect(panel, 12, 66)?;
        let row_height = 18_i32;
        let row_gap = 3_i32;
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

        for visible_index in 0..visible_rows {
            let index = start_index + visible_index;
            if index >= self.mappings.len() {
                break;
            }
            let row = Rect::new(
                list_bounds.x,
                list_bounds.y + visible_index as i32 * stride,
                list_bounds.width(),
                row_height as u32,
            );
            let entry = &self.mappings[index];
            let selected = index == self.page_state.selected_mapping_index;
            canvas.set_draw_color(if selected {
                app_chrome::OVERLAY_ROW_SELECTED_FILL
            } else {
                app_chrome::OVERLAY_ROW_IDLE_FILL
            });
            canvas.fill_rect(row)?;
            canvas.set_draw_color(if selected {
                mappings_theme::PAGE_TITLE
            } else {
                app_chrome::OVERLAY_ROW_IDLE_BORDER
            });
            canvas.draw_rect(row)?;

            crate::ui::draw_text_fitted(
                canvas,
                &entry.source_label,
                Rect::new(row.x + 8, row.y + 5, 126, 8),
                1,
                app_chrome::ACTION_TEXT,
            )?;
            crate::ui::draw_text_fitted(
                canvas,
                &entry.target_label,
                Rect::new(row.x + 146, row.y + 5, 210, 8),
                1,
                app_chrome::OVERLAY_TARGET_TEXT,
            )?;
            crate::ui::draw_text_fitted(
                canvas,
                compact_scope_label(&entry.scope_label),
                Rect::new(row.x + row.width() as i32 - 126, row.y + 5, 90, 8),
                1,
                app_chrome::OVERLAY_SCOPE_TEXT,
            )?;
        }

        crate::ui::draw_text_fitted(
            canvas,
            &format!(
                "Rows {}-{} / {}",
                start_index.saturating_add(1),
                (start_index + visible_rows).min(self.mappings.len()),
                self.mappings.len()
            ),
            Rect::new(panel.x + panel.width() as i32 - 116, panel.y + 34, 104, 8),
            1,
            app_chrome::OVERLAY_META_TEXT,
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mappings_overlay_toggles_on_and_off() {
        let mut app = App::new();
        assert!(app.overlay_state.active.is_none());

        app.apply_action(AppAction::ToggleMappingsOverlay);
        assert_eq!(
            app.overlay_state.active,
            Some(AppOverlay::MappingsQuickView)
        );

        app.apply_action(AppAction::ToggleMappingsOverlay);
        assert!(app.overlay_state.active.is_none());
    }
}
