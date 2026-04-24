use super::layout::{interlaced_color_at, loop_regions_intersect, rects_overlap};
use super::*;

fn is_high_contrast_light(theme: &Theme) -> bool {
    theme.preset == ThemePreset::HighContrastLight
}

fn is_high_contrast_dark(theme: &Theme) -> bool {
    theme.preset == ThemePreset::HighContrastDark
}

impl App {
    pub(crate) fn draw_track_subcolumn<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        bounds: Rect,
        accent: Color,
        view_start_ticks: u64,
        range_ticks: u64,
        playhead_ticks: u64,
        is_active: bool,
        detail: bool,
        track: &Track,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let theme = self.theme();
        let high_contrast = is_high_contrast_light(theme);
        let high_contrast_dark = is_high_contrast_dark(theme);
        let track_bg = if high_contrast {
            if track.state.muted {
                Color::RGB(228, 228, 228)
            } else {
                Color::RGB(250, 250, 250)
            }
        } else if high_contrast_dark {
            if track.state.muted {
                Color::RGB(12, 12, 12)
            } else {
                Color::RGB(4, 4, 4)
            }
        } else if track.state.muted {
            Color::RGB(16, 18, 24)
        } else {
            Color::RGB(20, 27, 40)
        };
        canvas.set_draw_color(track_bg);
        canvas.fill_rect(bounds)?;
        canvas.set_draw_color(if track.state.soloed {
            theme.transport.play_active
        } else if is_active {
            if high_contrast {
                theme.app_chrome.surface_border
            } else {
                Color::RGB(240, 222, 116)
            }
        } else {
            if high_contrast {
                Color::RGB(96, 96, 96)
            } else {
                Color::RGB(88, 96, 120)
            }
        });
        canvas.draw_rect(bounds)?;
        if track.state.passthrough {
            canvas.set_draw_color(if high_contrast {
                theme.app_chrome.tab_accent_midi_io
            } else if high_contrast_dark {
                Color::RGB(255, 255, 255)
            } else {
                Color::RGB(74, 210, 214)
            });
            canvas.fill_rect(Rect::new(
                bounds.x + 1,
                bounds.y + 1,
                2,
                bounds.height().saturating_sub(2),
            ))?;
        }

        let label_rect = timeline_subcolumn_label_rect(bounds, self.timeline_flow);
        let content_rect = timeline_subcolumn_content_rect(bounds, self.timeline_flow);

        canvas.set_draw_color(accent);
        canvas.fill_rect(label_rect)?;

        if !detail && track.state.loop_enabled {
            let loop_highlight = crate::ui::range_highlight_rect(
                content_rect,
                self.timeline_flow,
                view_start_ticks,
                range_ticks.max(1),
                track.loop_region,
            );
            canvas.set_draw_color(if is_active {
                if high_contrast {
                    Color::RGB(232, 232, 232)
                } else if high_contrast_dark {
                    Color::RGB(36, 36, 36)
                } else {
                    Color::RGB(88, 72, 24)
                }
            } else {
                if high_contrast {
                    Color::RGB(240, 240, 240)
                } else if high_contrast_dark {
                    Color::RGB(18, 18, 18)
                } else {
                    Color::RGB(54, 48, 28)
                }
            });
            canvas.fill_rect(loop_highlight)?;
        }

        for guide in crate::ui::timeline_guides(content_rect, self.timeline_flow) {
            canvas.set_draw_color(if high_contrast {
                Color::RGB(196, 196, 196)
            } else if high_contrast_dark {
                Color::RGB(72, 72, 72)
            } else {
                Color::RGB(52, 62, 84)
            });
            canvas.fill_rect(guide)?;
        }
        let top_row_y = label_rect.y + 3;
        let bottom_row_y = label_rect.y + label_rect.height() as i32 - 10;
        let clip_controls = if !detail && track.selected_recording_clip().is_some() {
            Some(self.recording_clip_control_rects(label_rect))
        } else {
            None
        };
        let name_right = clip_controls
            .map(|(mute_rect, _)| mute_rect.x - 4)
            .unwrap_or(label_rect.x + label_rect.width() as i32 - 4);
        let label_left = if detail {
            let slot_rects = self.stored_loop_slot_rects(label_rect);
            let active_slot = track.active_stored_loop_slot();
            let queued_slot = track.queued_stored_loop_slot();
            for (slot_index, slot_rect) in &slot_rects {
                let filled = track.stored_loop_slot(*slot_index).is_some();
                let active = active_slot == Some(*slot_index);
                let queued = queued_slot == Some(*slot_index);
                let slot_fill = if active {
                    theme.transport.song_loop
                } else if queued {
                    theme.app_chrome.tab_accent_midi_io
                } else if filled {
                    stored_loop_slot_color(*slot_index, theme)
                } else {
                    if high_contrast {
                        Color::RGB(255, 255, 255)
                    } else if high_contrast_dark {
                        Color::RGB(12, 12, 12)
                    } else {
                        Color::RGB(72, 70, 68)
                    }
                };
                canvas.set_draw_color(slot_fill);
                canvas.fill_rect(*slot_rect)?;
                canvas.set_draw_color(if active || queued || filled {
                    theme.app_chrome.surface_border
                } else if queued {
                    theme.app_chrome.surface_border
                } else if high_contrast {
                    Color::RGB(128, 128, 128)
                } else if high_contrast_dark {
                    Color::RGB(112, 112, 112)
                } else {
                    Color::RGB(122, 120, 116)
                });
                canvas.draw_rect(*slot_rect)?;
                let slot_label = (slot_index + 1).to_string();
                crate::ui::draw_text_fitted(
                    canvas,
                    &slot_label,
                    crate::app::support::ui_helpers::horizontally_center_text_rect(
                        &slot_label,
                        crate::app::support::ui_helpers::compact_label_rect(*slot_rect),
                        1,
                    ),
                    1,
                    if active || queued || filled {
                        contrasting_text_color(slot_fill, theme)
                    } else if high_contrast {
                        Color::RGB(0, 0, 0)
                    } else if high_contrast_dark {
                        Color::RGB(255, 255, 255)
                    } else {
                        Color::RGB(180, 178, 172)
                    },
                )?;
            }
            if STORED_LOOP_SLOT_COUNT > slot_rects.len() {
                let overflow = format!("+{}", STORED_LOOP_SLOT_COUNT - slot_rects.len());
                if let Some((_, last_slot_rect)) = slot_rects.last() {
                    crate::ui::draw_text_fitted(
                        canvas,
                        &overflow,
                        Rect::new(
                            last_slot_rect.x + last_slot_rect.width() as i32 + 3,
                            last_slot_rect.y + 1,
                            14,
                            7,
                        ),
                        1,
                        if high_contrast {
                            Color::RGB(0, 0, 0)
                        } else if high_contrast_dark {
                            Color::RGB(255, 255, 255)
                        } else {
                            Color::RGB(210, 194, 160)
                        },
                    )?;
                }
            }
            slot_rects
                .last()
                .map(|(_, rect)| rect.x + rect.width() as i32 + 5)
                .unwrap_or(label_rect.x + 4)
        } else {
            let passthrough_button = self.track_passthrough_button_rect(label_rect);
            let passthrough_fill = if track.state.passthrough {
                if high_contrast {
                    theme.app_chrome.tab_accent_midi_io
                } else if high_contrast_dark {
                    Color::RGB(255, 255, 255)
                } else {
                    Color::RGB(74, 210, 214)
                }
            } else {
                if high_contrast {
                    Color::RGB(255, 255, 255)
                } else if high_contrast_dark {
                    Color::RGB(16, 16, 16)
                } else {
                    Color::RGB(44, 70, 94)
                }
            };
            canvas.set_draw_color(passthrough_fill);
            canvas.fill_rect(passthrough_button)?;
            canvas.set_draw_color(if track.state.passthrough {
                if high_contrast {
                    theme.app_chrome.surface_border
                } else {
                    Color::RGB(210, 246, 248)
                }
            } else {
                if high_contrast {
                    theme.app_chrome.surface_border
                } else {
                    Color::RGB(144, 170, 194)
                }
            });
            canvas.draw_rect(passthrough_button)?;
            crate::ui::draw_text_fitted(
                canvas,
                "THRU",
                crate::app::support::ui_helpers::horizontally_center_text_rect(
                    "THRU",
                    crate::app::support::ui_helpers::compact_label_rect(passthrough_button),
                    1,
                ),
                1,
                contrasting_text_color(passthrough_fill, theme),
            )?;
            passthrough_button.x + passthrough_button.width() as i32 + 4
        };
        crate::ui::draw_text_fitted(
            canvas,
            &track.name,
            Rect::new(
                label_left,
                top_row_y,
                (name_right - label_left).max(0) as u32,
                8,
            ),
            1,
            if high_contrast {
                Color::RGB(0, 0, 0)
            } else if high_contrast_dark {
                Color::RGB(255, 255, 255)
            } else {
                Color::RGB(244, 244, 236)
            },
        )?;

        let role_badge = if detail {
            crate::ui::detail_badge_rect(label_rect)
        } else {
            Rect::new(
                label_rect.x + 4,
                bottom_row_y - 2,
                label_rect.width().saturating_sub(8).min(28),
                11,
            )
        };
        let role_badge_fill = if detail {
            if track.state.loop_enabled && self.project.transport.loop_enabled {
                theme.transport.song_loop
            } else {
                if high_contrast {
                    Color::RGB(255, 255, 255)
                } else if high_contrast_dark {
                    Color::RGB(16, 16, 16)
                } else {
                    Color::RGB(88, 82, 76)
                }
            }
        } else {
            if high_contrast {
                Color::RGB(255, 255, 255)
            } else if high_contrast_dark {
                Color::RGB(12, 12, 12)
            } else {
                Color::RGB(38, 58, 90)
            }
        };
        canvas.set_draw_color(role_badge_fill);
        canvas.fill_rect(role_badge)?;
        canvas.set_draw_color(if high_contrast {
            theme.app_chrome.surface_border
        } else {
            if detail {
                Color::RGB(238, 214, 172)
            } else {
                Color::RGB(188, 204, 226)
            }
        });
        canvas.draw_rect(role_badge)?;
        crate::ui::draw_text_fitted(
            canvas,
            if detail { "LOOP" } else { "SONG" },
            crate::app::support::ui_helpers::horizontally_center_text_rect(
                if detail { "LOOP" } else { "SONG" },
                crate::app::support::ui_helpers::compact_label_rect(role_badge),
                1,
            ),
            1,
            contrasting_text_color(role_badge_fill, theme),
        )?;
        if !detail {
            self.draw_recording_view_controls(
                canvas,
                label_rect,
                content_rect,
                track,
                clip_controls,
            )?;
        }

        let note_range = crate::timeline::LoopRegion::new(view_start_ticks, range_ticks.max(1));
        let selected_note_indices = track.selected_note_indices();
        let focused_note_index = track.focused_note_index();
        let anchor_note_index = track.anchor_note_index();
        let preview_region = track.preview_region(
            self.project.transport,
            self.record_capture_ticks(track),
            self.record_context(track),
        );
        let preview_notes = track.preview_notes(
            self.project.transport,
            self.record_capture_ticks(track),
            self.record_context(track),
        );
        self.draw_track_recording_content(
            canvas,
            content_rect,
            track,
            note_range,
            is_active,
            detail,
            selected_note_indices.as_slice(),
            focused_note_index,
            anchor_note_index,
            preview_region,
            preview_notes.as_slice(),
        )?;

        if track.recording_view != RecordingView::Stacked {
            if let Some(preview_region) = preview_region {
                if preview_region.intersects(note_range) {
                    for region in crate::ui::region_rects(
                        content_rect,
                        &[preview_region],
                        note_range,
                        self.timeline_flow,
                    ) {
                        if detail {
                            canvas.set_draw_color(Color::RGBA(214, 72, 72, 124));
                            canvas.fill_rect(region.rect)?;
                        }
                        canvas.set_draw_color(Color::RGB(248, 122, 122));
                        canvas.draw_rect(region.rect)?;
                    }
                }
            }

            for note in crate::ui::note_rects(
                content_rect,
                preview_notes.as_slice(),
                note_range,
                self.timeline_flow,
            ) {
                canvas.set_draw_color(Color::RGBA(238, 108, 108, 176));
                canvas.fill_rect(note.rect)?;
                canvas.set_draw_color(Color::RGB(255, 176, 176));
                canvas.draw_rect(note.rect)?;
            }
        }

        self.draw_track_loop_markers(canvas, content_rect, note_range, track)?;

        let playhead = crate::ui::playhead_rect_in_range(
            content_rect,
            self.timeline_flow,
            view_start_ticks,
            range_ticks.max(1),
            playhead_ticks,
        )?;
        if !detail && track.recording_view == RecordingView::Stacked && is_active {
            canvas.set_draw_color(if self.project.transport.playing {
                if high_contrast {
                    theme.app_chrome.surface_border
                } else {
                    Color::RGB(248, 240, 132)
                }
            } else {
                if high_contrast {
                    Color::RGB(96, 96, 96)
                } else {
                    Color::RGB(140, 150, 162)
                }
            });
            canvas.fill_rect(playhead)?;
            self.draw_recording_clip_scrollbar(canvas, content_rect, track)?;
        } else {
            canvas.set_draw_color(if self.project.transport.playing {
                if high_contrast {
                    theme.app_chrome.surface_border
                } else {
                    Color::RGB(248, 240, 132)
                }
            } else {
                if high_contrast {
                    Color::RGB(96, 96, 96)
                } else {
                    Color::RGB(140, 150, 162)
                }
            });
            canvas.fill_rect(playhead)?;
        }
        for tick in crate::ui::timeline_ruler_ticks(content_rect, self.timeline_flow) {
            canvas.set_draw_color(if high_contrast {
                Color::RGB(96, 96, 96)
            } else {
                Color::RGB(166, 178, 198)
            });
            canvas.fill_rect(tick)?;
        }

        Ok(())
    }

    fn draw_track_loop_markers<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        content_rect: Rect,
        note_range: crate::timeline::LoopRegion,
        track: &Track,
    ) -> Result<(), Box<dyn std::error::Error>> {
        #[derive(Clone)]
        struct LoopMarker {
            range: crate::timeline::LoopRegion,
            label: String,
            color: Color,
            emphasized: bool,
            queued: bool,
        }
        #[derive(Clone, Copy)]
        struct MarkerSpan {
            color: Color,
            start: i32,
            end: i32,
        }

        let active_slot = track.active_stored_loop_slot();
        let queued_slot = track.queued_stored_loop_slot();
        let mut markers = Vec::new();
        for slot_index in 0..STORED_LOOP_SLOT_COUNT {
            let Some(stored_loop) = track.stored_loop_slot(slot_index) else {
                continue;
            };
            markers.push(LoopMarker {
                range: stored_loop.as_loop_region(),
                label: (slot_index + 1).to_string(),
                color: stored_loop_slot_color(slot_index, self.theme()),
                emphasized: active_slot == Some(slot_index),
                queued: queued_slot == Some(slot_index),
            });
        }

        if active_slot.is_none() {
            markers.push(LoopMarker {
                range: track.loop_region,
                label: "L".to_string(),
                color: if track.state.loop_enabled {
                    self.theme().transport.song_loop
                } else {
                    if self.theme().preset == ThemePreset::HighContrastLight {
                        Color::RGB(96, 96, 96)
                    } else if self.theme().preset == ThemePreset::HighContrastDark {
                        Color::RGB(160, 160, 160)
                    } else {
                        Color::RGB(128, 122, 112)
                    }
                },
                emphasized: true,
                queued: false,
            });
        }

        let mut spans = Vec::new();
        for marker in &markers {
            if !loop_regions_intersect(marker.range, note_range) {
                continue;
            }

            let span_rect = crate::ui::range_highlight_rect(
                content_rect,
                self.timeline_flow,
                note_range.start_ticks,
                note_range.length_ticks,
                marker.range,
            );
            let (start, end) = match self.timeline_flow {
                TimelineFlow::DownwardColumns => (
                    span_rect.y,
                    span_rect.y + span_rect.height().max(1) as i32 - 1,
                ),
                TimelineFlow::AcrossRows => (
                    span_rect.x,
                    span_rect.x + span_rect.width().max(1) as i32 - 1,
                ),
            };
            spans.push(MarkerSpan {
                color: marker.color,
                start,
                end,
            });
        }

        if spans.is_empty() {
            return Ok(());
        }

        let side_thickness = 4_i32;
        let theme = self.theme();
        let primary_tick = if theme.preset == ThemePreset::HighContrastLight {
            theme.app_chrome.surface_border
        } else if theme.preset == ThemePreset::HighContrastDark {
            Color::RGB(255, 255, 255)
        } else {
            Color::RGB(252, 238, 194)
        };
        let queued_tick = if theme.preset == ThemePreset::HighContrastLight {
            theme.app_chrome.tab_accent_midi_io
        } else if theme.preset == ThemePreset::HighContrastDark {
            Color::RGB(160, 160, 160)
        } else {
            Color::RGB(184, 226, 248)
        };
        let secondary_tick = if theme.preset == ThemePreset::HighContrastLight {
            Color::RGB(96, 96, 96)
        } else if theme.preset == ThemePreset::HighContrastDark {
            Color::RGB(160, 160, 160)
        } else {
            Color::RGB(218, 224, 232)
        };
        let side_major = side_thickness.max(1) as u32;
        let content_bg = if theme.preset == ThemePreset::HighContrastLight {
            if track.state.muted {
                Color::RGB(228, 228, 228)
            } else {
                Color::RGB(250, 250, 250)
            }
        } else if theme.preset == ThemePreset::HighContrastDark {
            if track.state.muted {
                Color::RGB(12, 12, 12)
            } else {
                Color::RGB(4, 4, 4)
            }
        } else if track.state.muted {
            Color::RGB(16, 18, 24)
        } else {
            Color::RGB(20, 27, 40)
        };

        match self.timeline_flow {
            TimelineFlow::DownwardColumns => {
                let x = content_rect.x + 1;
                let usable_width = (content_rect.x + content_rect.width() as i32 - x).max(1);
                let band_width = side_major.min(usable_width as u32);
                let start_y = content_rect.y;
                let end_y = content_rect.y + content_rect.height() as i32 - 2;
                if end_y < start_y {
                    return Ok(());
                }
                let mut placed_label_rects = Vec::new();
                let label_spacing = 9_i32;

                for y in start_y..=end_y {
                    let colors = spans
                        .iter()
                        .filter(|span| y >= span.start && y <= span.end)
                        .map(|span| span.color)
                        .collect::<Vec<_>>();
                    if colors.is_empty() {
                        continue;
                    }
                    if let Some(color) = interlaced_color_at(&colors, (y - start_y).max(0) as usize)
                    {
                        canvas.set_draw_color(color);
                        canvas.fill_rect(Rect::new(x, y, band_width, 1))?;
                    }
                }

                for marker in &markers {
                    if !loop_regions_intersect(marker.range, note_range) {
                        continue;
                    }
                    let span_rect = crate::ui::range_highlight_rect(
                        content_rect,
                        self.timeline_flow,
                        note_range.start_ticks,
                        note_range.length_ticks,
                        marker.range,
                    );
                    let line_h = span_rect.height().max(1);
                    let marker_start_y = span_rect.y.clamp(start_y, end_y);
                    let end_marker_y = (span_rect.y + line_h as i32 - 1).clamp(start_y, end_y);
                    if marker_start_y > end_marker_y {
                        continue;
                    }
                    canvas.set_draw_color(if marker.emphasized {
                        primary_tick
                    } else if marker.queued {
                        queued_tick
                    } else {
                        secondary_tick
                    });
                    canvas.fill_rect(Rect::new(x, marker_start_y, band_width.min(4), 1))?;
                    canvas.fill_rect(Rect::new(x, end_marker_y, band_width.min(4), 1))?;

                    let marker_mid_y = marker_start_y + (end_marker_y - marker_start_y) / 2;
                    let label_y = (marker_mid_y - 3).clamp(
                        content_rect.y,
                        content_rect.y + content_rect.height() as i32 - 7,
                    );
                    let mut label_rect = Rect::new(x + band_width as i32 + 3, label_y, 8, 7);
                    for offset_step in 0..8 {
                        let candidate = Rect::new(
                            x + band_width as i32 + 3 + offset_step * label_spacing,
                            label_y,
                            8,
                            7,
                        );
                        if !placed_label_rects
                            .iter()
                            .any(|existing| rects_overlap(*existing, candidate))
                        {
                            label_rect = candidate;
                            break;
                        }
                    }
                    placed_label_rects.push(label_rect);
                    let label_readback = readback_rect_rgba(canvas, label_rect, self.viewport_size);
                    crate::ui::draw_text_fitted_inverted(
                        canvas,
                        marker.label.as_str(),
                        label_rect,
                        1,
                        |px, py| readback_color_at(&label_readback, px, py).unwrap_or(content_bg),
                    )?;
                    if marker.emphasized {
                        draw_loop_label_underline(
                            canvas,
                            marker.label.as_str(),
                            label_rect,
                            content_rect,
                            self.viewport_size,
                            content_bg,
                        )?;
                    } else if marker.queued {
                        canvas.set_draw_color(queued_tick);
                        canvas.fill_rect(Rect::new(
                            label_rect.x,
                            (label_rect.y + label_rect.height() as i32 + 1)
                                .min(content_rect.y + content_rect.height() as i32 - 1),
                            label_rect.width().min(4),
                            1,
                        ))?;
                    }
                }
            }
            TimelineFlow::AcrossRows => {
                let y = content_rect.y + 1;
                let usable_height = (content_rect.y + content_rect.height() as i32 - y).max(1);
                let band_height = side_major.min(usable_height as u32);
                let start_x = content_rect.x;
                let end_x = content_rect.x + content_rect.width() as i32 - 1;
                let mut placed_label_rects = Vec::new();
                let label_spacing = 9_i32;

                for x in start_x..=end_x {
                    let colors = spans
                        .iter()
                        .filter(|span| x >= span.start && x <= span.end)
                        .map(|span| span.color)
                        .collect::<Vec<_>>();
                    if colors.is_empty() {
                        continue;
                    }
                    for pixel in 0..band_height as usize {
                        if let Some(color) = interlaced_color_at(&colors, pixel) {
                            canvas.set_draw_color(color);
                            canvas.fill_rect(Rect::new(x, y + pixel as i32, 1, 1))?;
                        }
                    }
                }

                for marker in &markers {
                    if !loop_regions_intersect(marker.range, note_range) {
                        continue;
                    }
                    let span_rect = crate::ui::range_highlight_rect(
                        content_rect,
                        self.timeline_flow,
                        note_range.start_ticks,
                        note_range.length_ticks,
                        marker.range,
                    );
                    let line_w = span_rect.width().max(1);
                    let end_marker_x = span_rect.x + line_w as i32 - 1;
                    canvas.set_draw_color(if marker.emphasized {
                        primary_tick
                    } else if marker.queued {
                        queued_tick
                    } else {
                        secondary_tick
                    });
                    canvas.fill_rect(Rect::new(span_rect.x, y, 1, band_height.min(4)))?;
                    canvas.fill_rect(Rect::new(end_marker_x, y, 1, band_height.min(4)))?;

                    let label_x = (span_rect.x + line_w as i32 / 2 - 3).clamp(
                        content_rect.x,
                        content_rect.x + content_rect.width() as i32 - 7,
                    );
                    let mut label_rect = Rect::new(label_x, y + band_height as i32 + 3, 7, 6);
                    for offset_step in 0..8 {
                        let candidate =
                            Rect::new(label_x + offset_step * label_spacing, label_rect.y, 7, 6);
                        if !placed_label_rects
                            .iter()
                            .any(|existing| rects_overlap(*existing, candidate))
                        {
                            label_rect = candidate;
                            break;
                        }
                    }
                    placed_label_rects.push(label_rect);
                    let label_readback = readback_rect_rgba(canvas, label_rect, self.viewport_size);
                    crate::ui::draw_text_fitted_inverted(
                        canvas,
                        marker.label.as_str(),
                        label_rect,
                        1,
                        |px, py| readback_color_at(&label_readback, px, py).unwrap_or(content_bg),
                    )?;
                    if marker.emphasized {
                        draw_loop_label_underline(
                            canvas,
                            marker.label.as_str(),
                            label_rect,
                            content_rect,
                            self.viewport_size,
                            content_bg,
                        )?;
                    } else if marker.queued {
                        canvas.set_draw_color(queued_tick);
                        canvas.fill_rect(Rect::new(
                            label_rect.x,
                            (label_rect.y + label_rect.height() as i32 + 1)
                                .min(content_rect.y + content_rect.height() as i32 - 1),
                            label_rect.width().min(4),
                            1,
                        ))?;
                    }
                }
            }
        }

        Ok(())
    }
}

fn draw_loop_label_underline<T: RenderTarget>(
    canvas: &mut Canvas<T>,
    label: &str,
    label_rect: Rect,
    content_rect: Rect,
    viewport_size: (u32, u32),
    fallback_bg: Color,
) -> Result<(), String> {
    let underline_width = crate::ui::text_width(label, 1)
        .min(label_rect.width())
        .max(1);
    let underline_x =
        (label_rect.x + ((label_rect.width() as i32 - underline_width as i32) / 2).max(0) - 1)
            .max(content_rect.x);
    let underline_y = (label_rect.y + label_rect.height() as i32 + 1)
        .min(content_rect.y + content_rect.height() as i32 - 1);
    let underline_rect = Rect::new(underline_x, underline_y, underline_width, 1);
    let underline_readback = readback_rect_rgba(canvas, underline_rect, viewport_size);
    for offset in 0..underline_width as i32 {
        let px = underline_x + offset;
        let bg = readback_color_at(&underline_readback, px, underline_y).unwrap_or(fallback_bg);
        canvas.set_draw_color(Color::RGB(255 - bg.r, 255 - bg.g, 255 - bg.b));
        canvas
            .fill_rect(Rect::new(px, underline_y, 1, 1))
            .map_err(|error| error.to_string())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timeline_thru_hit_rect_matches_rendered_subcolumn_header() {
        let app = App::new();
        let content_bounds = Rect::new(40, 40, 1200, 620);
        let (_, body_bounds) =
            crate::ui::split_top_strip(content_bounds, 28, 6).expect("timeline content");
        let (_, timeline_bounds) =
            crate::ui::split_top_strip(body_bounds, transport_strip_height(), 8)
                .expect("timeline body");
        let columns = crate::ui::track_column_pairs(timeline_bounds, app.project.tracks.len());
        let (full_bounds, detail_bounds) = columns[0];
        let (body_full_bounds, _) = app.track_column_body_bounds(full_bounds, detail_bounds);
        let label_rect = timeline_subcolumn_label_rect(body_full_bounds, app.timeline_flow);
        let content_rect = timeline_subcolumn_content_rect(body_full_bounds, app.timeline_flow);
        let thru_rect = app.track_passthrough_button_rect(label_rect);
        let intersects = |a: Rect, b: Rect| {
            a.x < b.x + b.width() as i32
                && a.x + a.width() as i32 > b.x
                && a.y < b.y + b.height() as i32
                && a.y + a.height() as i32 > b.y
        };

        assert!(rect_contains(
            label_rect,
            thru_rect.x + thru_rect.width() as i32 / 2,
            thru_rect.y + thru_rect.height() as i32 / 2,
        ));
        assert!(!intersects(thru_rect, content_rect));
    }

    #[test]
    fn click_below_thru_does_not_toggle_passthrough() {
        let mut app = App::new();
        let content_bounds = Rect::new(40, 40, 1200, 620);
        let (_, body_bounds) =
            crate::ui::split_top_strip(content_bounds, 28, 6).expect("timeline content");
        let (_, timeline_bounds) =
            crate::ui::split_top_strip(body_bounds, transport_strip_height(), 8)
                .expect("timeline body");
        let columns = crate::ui::track_column_pairs(timeline_bounds, app.project.tracks.len());
        let (full_bounds, detail_bounds) = columns[0];
        let (body_full_bounds, _) = app.track_column_body_bounds(full_bounds, detail_bounds);
        let label_rect = timeline_subcolumn_label_rect(body_full_bounds, app.timeline_flow);
        let thru_rect = app.track_passthrough_button_rect(label_rect);
        let before = app.project.tracks[0].state.passthrough;
        let below_y = thru_rect.y + thru_rect.height() as i32 + 2;

        let control = app.handle_timeline_pointer(
            content_bounds,
            thru_rect.x + thru_rect.width() as i32 / 2,
            below_y,
            ActionSource::Pointer,
        );

        assert!(matches!(control, None | Some(AppControl::Continue)));
        assert_eq!(app.project.tracks[0].state.passthrough, before);
    }
}
