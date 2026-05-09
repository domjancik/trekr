use super::*;

fn is_high_contrast_light(theme: &Theme) -> bool {
    theme.preset == ThemePreset::HighContrastLight
}

fn is_high_contrast_dark(theme: &Theme) -> bool {
    theme.preset == ThemePreset::HighContrastDark
}

pub(crate) fn indexed_notes(
    track: &Track,
    recording_clip_id: Option<u64>,
) -> Vec<(usize, crate::project::MidiNote)> {
    track
        .midi_notes
        .iter()
        .copied()
        .enumerate()
        .filter(|(_, note)| match recording_clip_id {
            Some(clip_id) => note.recording_clip_id == Some(clip_id),
            None => note.recording_clip_id.is_none(),
        })
        .collect()
}

pub(crate) fn indexed_all_notes(track: &Track) -> Vec<(usize, crate::project::MidiNote)> {
    track.midi_notes.iter().copied().enumerate().collect()
}

impl App {
    pub(crate) fn record_head_ticks(&self, track: &Track) -> u64 {
        if track.state.loop_enabled {
            self.effective_track_playhead(track)
        } else {
            self.playhead_ticks
        }
    }

    pub(crate) fn record_capture_ticks(&self, track: &Track) -> u64 {
        if self.record_context(track).is_some() {
            self.transport_ticks
        } else {
            self.record_head_ticks(track)
        }
    }

    pub(crate) fn live_input_event_ticks(&self, track: &Track) -> u64 {
        if self.project.transport.playing {
            self.record_capture_ticks(track)
        } else {
            self.live_fx_ticks
        }
    }

    pub(crate) fn record_context(&self, track: &Track) -> Option<crate::project::RecordContext> {
        if track.state.loop_enabled {
            Some(crate::project::RecordContext {
                range: track.loop_region,
                wrap_basis_ticks: 0,
                extend_clip_on_wrap: self.project.transport.loop_recording_extends_clip,
            })
        } else if self.project.transport.loop_enabled {
            Some(crate::project::RecordContext {
                range: self.project.loop_region,
                wrap_basis_ticks: self.project.loop_region.start_ticks,
                extend_clip_on_wrap: self.project.transport.loop_recording_extends_clip,
            })
        } else {
            None
        }
    }

    pub(crate) fn detail_loop_range(&self, track: &Track) -> crate::timeline::LoopRegion {
        self.record_context(track)
            .map(|context| context.range)
            .unwrap_or(track.loop_region)
    }

    pub(crate) fn begin_recording(&mut self) {
        let target_indices = self.record_target_indices();
        if target_indices.is_empty() {
            return;
        }

        for index in target_indices {
            let pressed_at = self
                .project
                .tracks
                .get(index)
                .map(|track| self.record_capture_ticks(track))
                .unwrap_or(self.playhead_ticks);
            if let Some(track) = self.project.tracks.get_mut(index) {
                track.clear_queued_stored_loop_recall();
                track.begin_recording(pressed_at);
            }
        }
        self.project.transport.recording = true;
        self.project.transport.playing = true;
        self.mark_midi_runtime_dirty();
        self.sync_midi_runtime_state_if_needed();
    }

    pub(crate) fn finish_recording(&mut self) {
        if self.midi_runtime.is_enabled() {
            self.sync_midi_runtime_state_if_needed();
            let snapshot = self.midi_runtime.capture_snapshot();
            self.merge_runtime_recording_takes(&snapshot);
        }
        let transport = self.project.transport;
        let track_count = self.project.tracks.len();

        for index in 0..track_count {
            let release_ticks = self
                .project
                .tracks
                .get(index)
                .map(|track| self.record_capture_ticks(track))
                .unwrap_or(self.playhead_ticks);
            let record_context = self
                .project
                .tracks
                .get(index)
                .and_then(|track| self.record_context(track));
            if let Some(track) = self.project.tracks.get_mut(index) {
                if track.active_take.is_some() {
                    track.finish_recording(transport, release_ticks, record_context);
                }
            }
        }

        self.project.transport.recording = false;
        self.mark_midi_runtime_dirty();
        self.sync_midi_runtime_state_if_needed();
        if self.midi_runtime.is_enabled() {
            let snapshot = self.midi_runtime.capture_snapshot();
            self.merge_runtime_recording_takes(&snapshot);
        }
        self.sync_active_track_recording_clip_scroll();
    }

    pub(crate) fn record_target_indices(&self) -> Vec<usize> {
        let armed: Vec<usize> = self
            .project
            .tracks
            .iter()
            .enumerate()
            .filter_map(|(index, track)| track.state.armed.then_some(index))
            .collect();
        if armed.is_empty() {
            vec![self.project.active_track_index]
        } else {
            armed
        }
    }

    pub(crate) fn draw_recording_view_controls<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        label_rect: Rect,
        _content_rect: Rect,
        track: &Track,
        clip_controls: Option<(Rect, Rect, Rect)>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let theme = self.theme();
        let high_contrast = is_high_contrast_light(theme);
        let high_contrast_dark = is_high_contrast_dark(theme);
        if track.recording_view == RecordingView::Stacked {
            let can_scroll_left = self.can_select_previous_recording_clip(track);
            let can_scroll_right = self.can_select_next_recording_clip(track);
            let (left_rect, right_rect) = self.recording_view_scroll_control_rects(label_rect);
            canvas.set_draw_color(if can_scroll_left {
                if high_contrast {
                    Color::RGB(255, 255, 255)
                } else if high_contrast_dark {
                    Color::RGB(24, 24, 24)
                } else {
                    Color::RGB(74, 82, 98)
                }
            } else {
                if high_contrast {
                    Color::RGB(232, 232, 232)
                } else if high_contrast_dark {
                    Color::RGB(8, 8, 8)
                } else {
                    Color::RGB(48, 54, 68)
                }
            });
            canvas.fill_rect(left_rect)?;
            canvas.set_draw_color(if can_scroll_left {
                if high_contrast {
                    Color::RGB(0, 0, 0)
                } else if high_contrast_dark {
                    Color::RGB(255, 255, 255)
                } else {
                    Color::RGB(202, 212, 224)
                }
            } else {
                if high_contrast {
                    Color::RGB(128, 128, 128)
                } else if high_contrast_dark {
                    Color::RGB(96, 96, 96)
                } else {
                    Color::RGB(112, 118, 130)
                }
            });
            canvas.draw_rect(left_rect)?;
            crate::ui::draw_text_fitted(
                canvas,
                "<",
                Rect::new(
                    left_rect.x + 6,
                    left_rect.y + 1,
                    left_rect.width().saturating_sub(12),
                    8,
                ),
                1,
                if can_scroll_left {
                    if high_contrast {
                        Color::RGB(0, 0, 0)
                    } else if high_contrast_dark {
                        Color::RGB(255, 255, 255)
                    } else {
                        Color::RGB(244, 244, 236)
                    }
                } else {
                    if high_contrast {
                        Color::RGB(96, 96, 96)
                    } else if high_contrast_dark {
                        Color::RGB(128, 128, 128)
                    } else {
                        Color::RGB(144, 150, 160)
                    }
                },
            )?;
            canvas.set_draw_color(if can_scroll_right {
                if high_contrast {
                    Color::RGB(255, 255, 255)
                } else if high_contrast_dark {
                    Color::RGB(24, 24, 24)
                } else {
                    Color::RGB(74, 82, 98)
                }
            } else {
                if high_contrast {
                    Color::RGB(232, 232, 232)
                } else if high_contrast_dark {
                    Color::RGB(8, 8, 8)
                } else {
                    Color::RGB(48, 54, 68)
                }
            });
            canvas.fill_rect(right_rect)?;
            canvas.set_draw_color(if can_scroll_right {
                if high_contrast {
                    Color::RGB(0, 0, 0)
                } else if high_contrast_dark {
                    Color::RGB(255, 255, 255)
                } else {
                    Color::RGB(202, 212, 224)
                }
            } else {
                if high_contrast {
                    Color::RGB(128, 128, 128)
                } else if high_contrast_dark {
                    Color::RGB(96, 96, 96)
                } else {
                    Color::RGB(112, 118, 130)
                }
            });
            canvas.draw_rect(right_rect)?;
            crate::ui::draw_text_fitted(
                canvas,
                ">",
                Rect::new(
                    right_rect.x + 6,
                    right_rect.y + 1,
                    right_rect.width().saturating_sub(12),
                    8,
                ),
                1,
                if can_scroll_right {
                    if high_contrast {
                        Color::RGB(0, 0, 0)
                    } else if high_contrast_dark {
                        Color::RGB(255, 255, 255)
                    } else {
                        Color::RGB(244, 244, 236)
                    }
                } else {
                    if high_contrast {
                        Color::RGB(96, 96, 96)
                    } else if high_contrast_dark {
                        Color::RGB(128, 128, 128)
                    } else {
                        Color::RGB(144, 150, 160)
                    }
                },
            )?;
        }
        let view_rect = self.recording_view_chip_rect(label_rect);
        let view_fill = match track.recording_view {
            RecordingView::Overlay => {
                if high_contrast {
                    theme.app_chrome.tab_accent_timeline
                } else if high_contrast_dark {
                    Color::RGB(255, 255, 255)
                } else {
                    Color::RGB(50, 84, 126)
                }
            }
            RecordingView::Stacked => {
                if high_contrast {
                    theme.transport.song_loop
                } else if high_contrast_dark {
                    Color::RGB(160, 160, 160)
                } else {
                    Color::RGB(124, 98, 48)
                }
            }
        };
        canvas.set_draw_color(view_fill);
        canvas.fill_rect(view_rect)?;
        canvas.set_draw_color(if high_contrast {
            Color::RGB(0, 0, 0)
        } else {
            Color::RGB(232, 228, 208)
        });
        canvas.draw_rect(view_rect)?;
        crate::ui::draw_text_fitted(
            canvas,
            match track.recording_view {
                RecordingView::Overlay => "OVR",
                RecordingView::Stacked => "STK",
            },
            crate::app::support::ui_helpers::horizontally_center_text_rect(
                match track.recording_view {
                    RecordingView::Overlay => "OVR",
                    RecordingView::Stacked => "STK",
                },
                crate::app::support::ui_helpers::compact_label_rect(view_rect),
                1,
            ),
            1,
            contrasting_text_color(view_fill, theme),
        )?;

        if let (Some(selected_clip), Some((align_rect, mute_rect, delete_rect))) =
            (track.selected_recording_clip(), clip_controls)
        {
            let align_fill = if high_contrast {
                theme.transport.song_loop
            } else if high_contrast_dark {
                Color::RGB(255, 255, 255)
            } else {
                Color::RGB(88, 110, 74)
            };
            canvas.set_draw_color(align_fill);
            canvas.fill_rect(align_rect)?;
            canvas.set_draw_color(if high_contrast {
                Color::RGB(0, 0, 0)
            } else {
                Color::RGB(228, 236, 214)
            });
            canvas.draw_rect(align_rect)?;
            crate::ui::draw_text_fitted(
                canvas,
                "A",
                crate::app::support::ui_helpers::horizontally_center_text_rect(
                    "A",
                    crate::app::support::ui_helpers::compact_label_rect(align_rect),
                    1,
                ),
                1,
                contrasting_text_color(align_fill, theme),
            )?;

            let mute_fill = if selected_clip.muted {
                if high_contrast {
                    Color::RGB(232, 232, 232)
                } else if high_contrast_dark {
                    Color::RGB(72, 72, 72)
                } else {
                    Color::RGB(120, 118, 112)
                }
            } else {
                if high_contrast {
                    theme.transport.play_active
                } else if high_contrast_dark {
                    Color::RGB(255, 255, 255)
                } else {
                    Color::RGB(84, 122, 92)
                }
            };
            canvas.set_draw_color(mute_fill);
            canvas.fill_rect(mute_rect)?;
            canvas.set_draw_color(if high_contrast {
                Color::RGB(0, 0, 0)
            } else {
                Color::RGB(228, 232, 216)
            });
            canvas.draw_rect(mute_rect)?;
            crate::ui::draw_text_fitted(
                canvas,
                if selected_clip.muted { "ON" } else { "M" },
                crate::app::support::ui_helpers::horizontally_center_text_rect(
                    if selected_clip.muted { "ON" } else { "M" },
                    crate::app::support::ui_helpers::compact_label_rect(mute_rect),
                    1,
                ),
                1,
                contrasting_text_color(mute_fill, theme),
            )?;

            let delete_fill = if high_contrast {
                theme.transport.record_active
            } else if high_contrast_dark {
                Color::RGB(255, 255, 255)
            } else {
                Color::RGB(132, 74, 70)
            };
            canvas.set_draw_color(delete_fill);
            canvas.fill_rect(delete_rect)?;
            canvas.set_draw_color(if high_contrast {
                Color::RGB(0, 0, 0)
            } else {
                Color::RGB(240, 220, 210)
            });
            canvas.draw_rect(delete_rect)?;
            crate::ui::draw_text_fitted(
                canvas,
                "X",
                crate::app::support::ui_helpers::horizontally_center_text_rect(
                    "X",
                    crate::app::support::ui_helpers::compact_label_rect(delete_rect),
                    1,
                ),
                1,
                contrasting_text_color(delete_fill, theme),
            )?;
        }

        Ok(())
    }

    pub(crate) fn draw_track_recording_content<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        content_rect: Rect,
        track: &Track,
        note_range: crate::timeline::LoopRegion,
        is_active: bool,
        detail: bool,
        selected_note_indices: &[usize],
        focused_note_index: Option<usize>,
        anchor_note_index: Option<usize>,
        preview_region: Option<crate::timeline::Region>,
        preview_notes: &[crate::project::MidiNote],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let theme = self.theme();
        let high_contrast = is_high_contrast_light(theme);
        let high_contrast_dark = is_high_contrast_dark(theme);
        if track.recording_view == RecordingView::Stacked
            && (!track.recording_clips().is_empty() || preview_region.is_some())
        {
            let unowned_regions: Vec<_> = track
                .regions
                .iter()
                .copied()
                .filter(|region| region.recording_clip_id.is_none())
                .collect();
            let unowned_notes = indexed_notes(track, None);
            self.draw_region_entries(
                canvas,
                content_rect,
                &unowned_regions,
                note_range,
                track,
                is_active,
                track.state.muted,
            )?;
            self.draw_note_entries(
                canvas,
                content_rect,
                &unowned_notes,
                note_range,
                track,
                detail,
                track.state.muted,
                selected_note_indices,
                focused_note_index,
                anchor_note_index,
            )?;

            for lane in self.recording_lane_layouts(content_rect, track) {
                canvas.set_draw_color(if lane.preview {
                    if high_contrast {
                        Color::RGB(250, 236, 236)
                    } else if high_contrast_dark {
                        Color::RGB(20, 20, 20)
                    } else {
                        Color::RGB(54, 32, 36)
                    }
                } else if lane.selected {
                    if high_contrast {
                        Color::RGB(236, 236, 236)
                    } else if high_contrast_dark {
                        Color::RGB(28, 28, 28)
                    } else {
                        Color::RGB(46, 62, 94)
                    }
                } else {
                    if high_contrast {
                        Color::RGB(244, 244, 244)
                    } else if high_contrast_dark {
                        Color::RGB(8, 8, 8)
                    } else {
                        Color::RGB(26, 34, 48)
                    }
                });
                canvas.fill_rect(lane.rect)?;
                canvas.set_draw_color(if lane.preview {
                    if high_contrast {
                        theme.transport.record_active
                    } else if high_contrast_dark {
                        Color::RGB(255, 255, 255)
                    } else {
                        Color::RGB(248, 122, 122)
                    }
                } else if lane.selected {
                    if high_contrast {
                        Color::RGB(0, 0, 0)
                    } else if high_contrast_dark {
                        Color::RGB(255, 255, 255)
                    } else {
                        Color::RGB(248, 226, 134)
                    }
                } else {
                    if high_contrast {
                        Color::RGB(128, 128, 128)
                    } else if high_contrast_dark {
                        Color::RGB(96, 96, 96)
                    } else {
                        Color::RGB(76, 92, 118)
                    }
                });
                canvas.draw_rect(lane.rect)?;

                if lane.preview {
                    if let Some(preview_region) =
                        preview_region.filter(|region| region.intersects(note_range))
                    {
                        for region in crate::ui::region_rects(
                            lane.rect,
                            &[preview_region],
                            note_range,
                            self.timeline_flow,
                        ) {
                            if detail {
                                canvas.set_draw_color(if high_contrast {
                                    Color::RGBA(196, 40, 40, 72)
                                } else {
                                    Color::RGBA(214, 72, 72, 124)
                                });
                                canvas.fill_rect(region.rect)?;
                            }
                            canvas.set_draw_color(if high_contrast {
                                theme.transport.record_active
                            } else {
                                Color::RGB(248, 122, 122)
                            });
                            canvas.draw_rect(region.rect)?;
                        }
                    }

                    for note in crate::ui::note_rects(
                        lane.rect,
                        preview_notes,
                        note_range,
                        self.timeline_flow,
                    ) {
                        canvas.set_draw_color(if high_contrast {
                            Color::RGBA(196, 40, 40, 136)
                        } else {
                            Color::RGBA(238, 108, 108, 176)
                        });
                        canvas.fill_rect(note.rect)?;
                        canvas.set_draw_color(if high_contrast {
                            theme.transport.record_active
                        } else {
                            Color::RGB(255, 176, 176)
                        });
                        canvas.draw_rect(note.rect)?;
                    }
                } else if let Some(clip_id) = lane.clip_id {
                    let lane_muted = track.state.muted || lane.muted;
                    let lane_regions: Vec<_> = track
                        .regions
                        .iter()
                        .copied()
                        .filter(|region| region.recording_clip_id == Some(clip_id))
                        .collect();
                    let lane_notes = indexed_notes(track, Some(clip_id));
                    self.draw_region_entries(
                        canvas,
                        lane.rect,
                        &lane_regions,
                        note_range,
                        track,
                        is_active,
                        lane_muted,
                    )?;
                    self.draw_note_entries(
                        canvas,
                        lane.rect,
                        &lane_notes,
                        note_range,
                        track,
                        detail,
                        lane_muted,
                        selected_note_indices,
                        focused_note_index,
                        anchor_note_index,
                    )?;
                }
            }

            return Ok(());
        }

        self.draw_region_entries(
            canvas,
            content_rect,
            &track.regions,
            note_range,
            track,
            is_active,
            track.state.muted,
        )?;
        self.draw_note_entries(
            canvas,
            content_rect,
            &indexed_all_notes(track),
            note_range,
            track,
            detail,
            track.state.muted,
            selected_note_indices,
            focused_note_index,
            anchor_note_index,
        )?;
        Ok(())
    }

    fn draw_region_entries<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        lane_rect: Rect,
        regions: &[crate::timeline::Region],
        note_range: crate::timeline::LoopRegion,
        track: &Track,
        is_active: bool,
        muted_override: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let high_contrast = self.theme().preset == ThemePreset::HighContrastLight;
        let high_contrast_dark = self.theme().preset == ThemePreset::HighContrastDark;
        for source_region in regions.iter().copied() {
            let region_muted =
                muted_override || track.recording_clip_is_muted(source_region.recording_clip_id);
            let Some(region) = crate::ui::region_rects(
                lane_rect,
                &[source_region],
                note_range,
                self.timeline_flow,
            )
            .into_iter()
            .next() else {
                continue;
            };
            canvas.set_draw_color(if region.clipped {
                if high_contrast {
                    Color::RGB(224, 216, 196)
                } else if high_contrast_dark {
                    Color::RGB(56, 56, 56)
                } else {
                    Color::RGB(108, 88, 56)
                }
            } else if region_muted {
                if high_contrast {
                    Color::RGB(228, 228, 228)
                } else if high_contrast_dark {
                    Color::RGB(18, 18, 18)
                } else {
                    Color::RGB(42, 46, 56)
                }
            } else {
                if high_contrast {
                    Color::RGB(238, 238, 238)
                } else if high_contrast_dark {
                    Color::RGB(12, 12, 12)
                } else {
                    Color::RGB(44, 54, 76)
                }
            });
            canvas.fill_rect(region.rect)?;
            canvas.set_draw_color(if is_active {
                if high_contrast {
                    Color::RGB(0, 0, 0)
                } else if high_contrast_dark {
                    Color::RGB(255, 255, 255)
                } else {
                    Color::RGB(212, 196, 122)
                }
            } else {
                if high_contrast {
                    Color::RGB(128, 128, 128)
                } else if high_contrast_dark {
                    Color::RGB(96, 96, 96)
                } else {
                    Color::RGB(96, 106, 126)
                }
            });
            canvas.draw_rect(region.rect)?;
        }

        Ok(())
    }

    fn draw_note_entries<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        lane_rect: Rect,
        note_entries: &[(usize, crate::project::MidiNote)],
        note_range: crate::timeline::LoopRegion,
        track: &Track,
        detail: bool,
        muted_override: bool,
        selected_note_indices: &[usize],
        focused_note_index: Option<usize>,
        anchor_note_index: Option<usize>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let high_contrast = self.theme().preset == ThemePreset::HighContrastLight;
        let high_contrast_dark = self.theme().preset == ThemePreset::HighContrastDark;
        let notes: Vec<_> = note_entries.iter().map(|(_, note)| *note).collect();
        for note in crate::ui::note_rects(lane_rect, &notes, note_range, self.timeline_flow) {
            let absolute_index = note_entries[note.source_index].0;
            let note_muted = muted_override
                || track
                    .recording_clip_is_muted(note_entries[note.source_index].1.recording_clip_id);
            let selected = selected_note_indices.contains(&absolute_index);
            let focused = focused_note_index == Some(absolute_index);
            let anchored = anchor_note_index == Some(absolute_index);
            canvas.set_draw_color(if selected && detail {
                if high_contrast {
                    Color::RGB(0, 92, 160)
                } else if high_contrast_dark {
                    Color::RGB(255, 255, 255)
                } else {
                    Color::RGB(112, 174, 228)
                }
            } else if selected {
                if high_contrast {
                    Color::RGB(0, 0, 0)
                } else if high_contrast_dark {
                    Color::RGB(196, 196, 196)
                } else {
                    Color::RGB(88, 136, 194)
                }
            } else if note_muted {
                if high_contrast {
                    Color::RGB(144, 144, 144)
                } else if high_contrast_dark {
                    Color::RGB(72, 72, 72)
                } else {
                    Color::RGB(92, 100, 112)
                }
            } else if note.clipped {
                if high_contrast {
                    Color::RGB(194, 138, 0)
                } else if high_contrast_dark {
                    Color::RGB(160, 160, 160)
                } else {
                    Color::RGB(244, 204, 132)
                }
            } else {
                if high_contrast {
                    Color::RGB(32, 32, 32)
                } else if high_contrast_dark {
                    Color::RGB(255, 255, 255)
                } else {
                    Color::RGB(210, 222, 236)
                }
            });
            canvas.fill_rect(note.rect)?;
            canvas.set_draw_color(if focused {
                if high_contrast {
                    Color::RGB(194, 138, 0)
                } else if high_contrast_dark {
                    Color::RGB(0, 0, 0)
                } else {
                    Color::RGB(252, 246, 158)
                }
            } else if anchored {
                if high_contrast {
                    Color::RGB(0, 128, 96)
                } else if high_contrast_dark {
                    Color::RGB(0, 0, 0)
                } else {
                    Color::RGB(180, 226, 176)
                }
            } else if selected {
                if high_contrast {
                    Color::RGB(255, 255, 255)
                } else if high_contrast_dark {
                    Color::RGB(0, 0, 0)
                } else {
                    Color::RGB(224, 238, 248)
                }
            } else if note_muted {
                if high_contrast {
                    Color::RGB(96, 96, 96)
                } else if high_contrast_dark {
                    Color::RGB(128, 128, 128)
                } else {
                    Color::RGB(128, 134, 144)
                }
            } else {
                if high_contrast {
                    Color::RGB(255, 255, 255)
                } else if high_contrast_dark {
                    Color::RGB(0, 0, 0)
                } else {
                    Color::RGB(245, 247, 250)
                }
            });
            canvas.draw_rect(note.rect)?;
            if focused {
                let inner = Rect::new(
                    note.rect.x + 1,
                    note.rect.y + 1,
                    note.rect.width().saturating_sub(2).max(1),
                    note.rect.height().saturating_sub(2).max(1),
                );
                canvas.set_draw_color(if high_contrast {
                    Color::RGB(194, 138, 0)
                } else if high_contrast_dark {
                    Color::RGB(0, 0, 0)
                } else {
                    Color::RGB(252, 208, 88)
                });
                canvas.draw_rect(inner)?;
            }
        }

        Ok(())
    }

    pub(crate) fn recording_lane_layouts(
        &self,
        content_rect: Rect,
        track: &Track,
    ) -> Vec<RecordingLaneLayout> {
        let gap = 2;
        let window = self.recording_lane_window(track, self.recording_lane_capacity(content_rect));
        let visible_clips = &track.recording_clips()[window.committed_start..window.committed_end];
        let lane_count = window.visible_committed + usize::from(window.show_preview);
        let lane_rects = match self.timeline_flow {
            TimelineFlow::DownwardColumns => {
                crate::ui::equal_columns(content_rect, lane_count, gap)
            }
            TimelineFlow::AcrossRows => crate::ui::stacked_rows(content_rect, lane_count, gap),
        };

        let mut layouts: Vec<_> = visible_clips
            .iter()
            .zip(lane_rects.iter().copied())
            .map(|(clip, rect)| RecordingLaneLayout {
                clip_id: Some(clip.id),
                rect,
                selected: track.selected_recording_clip_id == Some(clip.id),
                muted: clip.muted,
                preview: false,
            })
            .collect();

        if window.show_preview {
            if let Some(rect) = lane_rects.get(window.visible_committed).copied() {
                layouts.push(RecordingLaneLayout {
                    clip_id: None,
                    rect,
                    selected: false,
                    muted: false,
                    preview: true,
                });
            }
        }

        layouts
    }

    pub(crate) fn recording_lane_hit_clip(
        &self,
        content_rect: Rect,
        track: &Track,
        x: i32,
        y: i32,
    ) -> Option<u64> {
        self.recording_lane_layouts(content_rect, track)
            .into_iter()
            .find_map(|lane| {
                rect_contains(lane.rect, x, y)
                    .then_some(lane.clip_id)
                    .flatten()
            })
    }

    pub(crate) fn recording_lane_capacity(&self, content_rect: Rect) -> usize {
        match self.timeline_flow {
            TimelineFlow::DownwardColumns => {
                let min_lane_width = 15_i32;
                let gap = 2_i32;
                (((content_rect.width() as i32 + gap) / (min_lane_width + gap)).max(1)) as usize
            }
            TimelineFlow::AcrossRows => {
                let min_lane_height = 26_i32;
                let gap = 2_i32;
                (((content_rect.height() as i32 + gap) / (min_lane_height + gap)).max(1)) as usize
            }
        }
    }

    pub(crate) fn recording_view_chip_rect(&self, label_rect: Rect) -> Rect {
        let top_y = label_rect.y + label_rect.height() as i32 - 13;
        let right = label_rect.x + label_rect.width() as i32 - 4;
        Rect::new(right - 26, top_y, 26, 11)
    }

    pub(crate) fn track_passthrough_button_rect(&self, label_rect: Rect) -> Rect {
        Rect::new(
            label_rect.x + 4,
            label_rect.y + 1,
            label_rect.width().saturating_sub(8).min(30),
            11,
        )
    }

    fn stored_loop_visible_slot_count(&self, label_rect: Rect) -> usize {
        let slot_w = 10_i32;
        let gap = 2_i32;
        let side_padding = 8_i32;
        let min_name_space = 24_i32;
        let available = label_rect.width() as i32 - side_padding - min_name_space;
        if available < slot_w {
            return 0;
        }
        (((available + gap) / (slot_w + gap)).max(0) as usize).min(STORED_LOOP_SLOT_COUNT)
    }

    pub(crate) fn stored_loop_slot_rects(&self, label_rect: Rect) -> Vec<(usize, Rect)> {
        let visible_slots = self
            .stored_loop_visible_slot_count(label_rect)
            .min(STORED_LOOP_SLOT_COUNT);
        let slot_w = 10_u32;
        let slot_h = 11_u32;
        let gap = 2_i32;
        let mut rects = Vec::with_capacity(visible_slots);
        for slot_index in 0..visible_slots {
            rects.push((
                slot_index,
                Rect::new(
                    label_rect.x + 4 + slot_index as i32 * (slot_w as i32 + gap),
                    label_rect.y,
                    slot_w,
                    slot_h,
                ),
            ));
        }
        rects
    }

    pub(crate) fn recording_view_scroll_control_rects(&self, label_rect: Rect) -> (Rect, Rect) {
        let top_y = label_rect.y + label_rect.height() as i32 - 13;
        let view_rect = self.recording_view_chip_rect(label_rect);
        let right_rect = Rect::new(view_rect.x - 16, top_y, 12, 11);
        let left_rect = Rect::new(right_rect.x - 14, top_y, 12, 11);
        (left_rect, right_rect)
    }

    fn selected_recording_clip_index(&self, track: &Track) -> Option<usize> {
        track.selected_recording_clip_id.and_then(|selected_id| {
            track
                .recording_clips()
                .iter()
                .position(|clip| clip.id == selected_id)
        })
    }

    fn can_select_previous_recording_clip(&self, track: &Track) -> bool {
        self.selected_recording_clip_index(track)
            .map(|index| index > 0)
            .unwrap_or(false)
    }

    fn can_select_next_recording_clip(&self, track: &Track) -> bool {
        self.selected_recording_clip_index(track)
            .map(|index| index + 1 < track.recording_clips().len())
            .unwrap_or(false)
    }

    pub(crate) fn sync_active_track_recording_clip_scroll(&mut self) {
        let Some(full_bounds) = self.active_track_full_bounds() else {
            return;
        };
        let content_rect =
            crate::ui::track_content_rect(full_bounds, self.timeline_flow, self.ui_metrics());
        let total_capacity = self.recording_lane_capacity(content_rect).max(1);
        let Some(track) = self.project.active_track_mut() else {
            return;
        };
        let total_lanes = track.recording_clips.len() + usize::from(track.active_take.is_some());
        if total_lanes == 0 {
            track.recording_clip_scroll = 0;
            return;
        }

        let visible_lanes = total_capacity.min(total_lanes);
        let max_start = total_lanes.saturating_sub(visible_lanes);
        track.recording_clip_scroll = track.recording_clip_scroll.min(max_start);
        if track.active_take.is_some() {
            track.recording_clip_scroll = track.recording_clip_scroll.max(max_start);
        }
        let Some(selected_id) = track.selected_recording_clip_id else {
            return;
        };
        if track.active_take.is_some() {
            return;
        }
        let Some(selected_index) = track
            .recording_clips
            .iter()
            .position(|clip| clip.id == selected_id)
        else {
            return;
        };
        if selected_index < track.recording_clip_scroll {
            track.recording_clip_scroll = selected_index;
        } else if selected_index >= track.recording_clip_scroll + visible_lanes {
            track.recording_clip_scroll = selected_index + 1 - visible_lanes;
        }
    }

    pub(crate) fn recording_clip_scroll_control_hit(
        &self,
        label_rect: Rect,
        track: &Track,
        x: i32,
        y: i32,
    ) -> Option<AppAction> {
        if track.recording_view != RecordingView::Stacked {
            return None;
        }
        let (left_rect, right_rect) = self.recording_view_scroll_control_rects(label_rect);
        if rect_contains(left_rect, x, y) && self.can_select_previous_recording_clip(track) {
            return Some(AppAction::SelectPreviousRecordingClip);
        }
        if rect_contains(right_rect, x, y) && self.can_select_next_recording_clip(track) {
            return Some(AppAction::SelectNextRecordingClip);
        }
        None
    }

    pub(crate) fn recording_clip_control_rects(&self, label_rect: Rect) -> (Rect, Rect, Rect) {
        let top_y = label_rect.y + 1;
        let right = label_rect.x + label_rect.width() as i32 - 4;
        (
            Rect::new(right - 44, top_y, 12, 11),
            Rect::new(right - 28, top_y, 12, 11),
            Rect::new(right - 12, top_y, 12, 11),
        )
    }

    pub(crate) fn recording_clip_scrollbar_rects(
        &self,
        content_rect: Rect,
        track: &Track,
    ) -> Option<(Rect, Rect)> {
        if track.recording_view != RecordingView::Stacked {
            return None;
        }

        let total_lanes = track.recording_clips.len() + usize::from(track.active_take.is_some());
        if total_lanes == 0 {
            return None;
        }

        let window = self.recording_lane_window(track, self.recording_lane_capacity(content_rect));
        let visible_lanes = window.visible_total.clamp(1, total_lanes);
        let start = window.start;
        let rail = Rect::new(
            content_rect.x + 4,
            content_rect.y,
            content_rect.width().saturating_sub(8),
            2,
        );
        if rail.width() == 0 {
            return None;
        }

        let thumb_width = ((rail.width() as usize * visible_lanes) / total_lanes)
            .max(6)
            .min(rail.width() as usize) as u32;
        let max_offset = rail.width().saturating_sub(thumb_width) as i32;
        let max_start = total_lanes.saturating_sub(visible_lanes);
        let thumb_x = if max_start == 0 {
            rail.x
        } else {
            rail.x + (max_offset as i64 * start as i64 / max_start as i64) as i32
        };
        let thumb = Rect::new(thumb_x, rail.y, thumb_width, 2);
        Some((rail, thumb))
    }

    pub(crate) fn draw_recording_clip_scrollbar<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        content_rect: Rect,
        track: &Track,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let Some((rail, thumb)) = self.recording_clip_scrollbar_rects(content_rect, track) else {
            return Ok(());
        };
        canvas.set_draw_color(if self.theme().preset == ThemePreset::HighContrastLight {
            Color::RGB(160, 160, 160)
        } else if self.theme().preset == ThemePreset::HighContrastDark {
            Color::RGB(96, 96, 96)
        } else {
            Color::RGB(92, 100, 120)
        });
        canvas.fill_rect(rail)?;
        canvas.set_draw_color(if self.theme().preset == ThemePreset::HighContrastLight {
            Color::RGB(0, 0, 0)
        } else if self.theme().preset == ThemePreset::HighContrastDark {
            Color::RGB(255, 255, 255)
        } else {
            Color::RGB(244, 214, 118)
        });
        canvas.fill_rect(thumb)?;
        Ok(())
    }

    fn recording_lane_window(&self, track: &Track, total_capacity: usize) -> RecordingLaneWindow {
        let total_capacity = total_capacity.max(1);
        let committed_len = track.recording_clips().len();
        let preview_index = track.active_take.as_ref().map(|_| committed_len);
        let total_lanes = committed_len + usize::from(preview_index.is_some());
        if total_lanes == 0 {
            return RecordingLaneWindow {
                start: 0,
                visible_total: 0,
                committed_start: 0,
                committed_end: 0,
                visible_committed: 0,
                show_preview: false,
            };
        }

        let visible_total = total_capacity.min(total_lanes);
        let max_start = total_lanes.saturating_sub(visible_total);
        let mut start = track.recording_clip_scroll.min(max_start);
        if let Some(preview_index) = preview_index {
            start = start.max(preview_index + 1 - visible_total);
        }
        let end = start + visible_total;
        let committed_start = start.min(committed_len);
        let committed_end = end.min(committed_len);
        let show_preview = preview_index.is_some_and(|preview_index| preview_index >= start);

        RecordingLaneWindow {
            start,
            visible_total,
            committed_start,
            committed_end,
            visible_committed: committed_end.saturating_sub(committed_start),
            show_preview,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn region_span(region: crate::timeline::Region) -> (u64, u64) {
        (region.start_ticks, region.length_ticks)
    }
    use crate::actions::AppAction;
    use crate::project::RecordingView;
    use crate::timeline::RecordingTake;

    #[test]
    fn recording_clip_actions_update_active_track_clip_state() {
        let mut app = App::new();
        let transport = app.project.transport;
        {
            let track = app.project.active_track_mut().unwrap();
            track.clear_content();
            track.commit_take(transport, RecordingTake::new(0).release(480), None);
            track.commit_take(transport, RecordingTake::new(960).release(1_440), None);
        }

        app.apply_action(AppAction::ToggleCurrentTrackRecordingView);
        assert_eq!(
            app.project.active_track().unwrap().recording_view,
            RecordingView::Stacked
        );

        app.apply_action(AppAction::SelectPreviousRecordingClip);
        let selected_before_delete = app
            .project
            .active_track()
            .unwrap()
            .selected_recording_clip_id
            .expect("selected clip");
        app.apply_action(AppAction::ToggleSelectedRecordingClipMute);
        assert!(
            app.project
                .active_track()
                .unwrap()
                .selected_recording_clip()
                .unwrap()
                .muted
        );

        app.apply_action(AppAction::DeleteSelectedRecordingClip);
        let active = app.project.active_track().unwrap();
        assert_eq!(active.recording_clips.len(), 1);
        assert_ne!(active.recording_clips[0].id, selected_before_delete);
    }

    #[test]
    fn stacked_all_track_layout_shows_at_least_three_recording_lanes() {
        let app = App::new();
        let timeline_bounds = Rect::new(0, 0, 1000, 420);
        let (_, full_bounds, _) = app.visible_track_columns(timeline_bounds)[0];
        let content_rect =
            crate::ui::track_content_rect(full_bounds, app.timeline_flow, app.ui_metrics());

        assert!(app.recording_lane_capacity(content_rect) >= 3);
    }

    #[test]
    fn stacked_view_shows_preview_lane_while_recording() {
        let mut app = App::new();
        let transport = app.project.transport;
        {
            let track = app.project.active_track_mut().unwrap();
            track.clear_content();
            track.recording_view = RecordingView::Stacked;
            track.commit_take(transport, RecordingTake::new(0).release(480), None);
        }

        app.transport_ticks = 960;
        app.playhead_ticks = 960;
        app.apply_action(AppAction::ToggleRecording);

        let timeline_bounds = Rect::new(0, 0, 1000, 420);
        let (_, full_bounds, _) = app.visible_track_columns(timeline_bounds)[0];
        let content_rect =
            crate::ui::track_content_rect(full_bounds, app.timeline_flow, app.ui_metrics());
        let layouts = app.recording_lane_layouts(content_rect, app.project.active_track().unwrap());

        assert_eq!(layouts.len(), 2);
        assert!(layouts.iter().any(|lane| lane.preview));
    }

    #[test]
    fn stacked_view_preview_lane_shifts_visible_window_as_committed() {
        let mut app = App::new();
        let transport = app.project.transport;
        let trailing_clip_ids = {
            let track = app.project.active_track_mut().unwrap();
            track.clear_content();
            track.recording_view = RecordingView::Stacked;
            for index in 0..5 {
                let start = index * 480;
                track.commit_take(
                    transport,
                    RecordingTake::new(start).release(start + 240),
                    None,
                );
            }
            let trailing = vec![track.recording_clips[3].id, track.recording_clips[4].id];
            track.recording_clip_scroll = 2;
            track.active_take = Some(RecordingTake::new(2400));
            trailing
        };

        let content_rect = Rect::new(0, 0, 49, 200);
        assert_eq!(app.recording_lane_capacity(content_rect), 3);
        let layouts = app.recording_lane_layouts(content_rect, app.project.active_track().unwrap());

        assert_eq!(layouts.len(), 3);
        assert_eq!(layouts[0].clip_id, Some(trailing_clip_ids[0]));
        assert_eq!(layouts[1].clip_id, Some(trailing_clip_ids[1]));
        assert!(layouts[2].preview);
    }

    #[test]
    fn stacked_scrollbar_thumb_tracks_clip_window_position() {
        let mut app = App::new();
        let transport = app.project.transport;
        {
            let track = app.project.active_track_mut().unwrap();
            track.clear_content();
            track.recording_view = RecordingView::Stacked;
            for index in 0..5 {
                let start = index * 480;
                track.commit_take(
                    transport,
                    RecordingTake::new(start).release(start + 240),
                    None,
                );
            }
            track.recording_clip_scroll = 0;
        }

        let timeline_bounds = Rect::new(0, 0, 1000, 420);
        let (_, full_bounds, _) = app.visible_track_columns(timeline_bounds)[0];
        let content_rect =
            crate::ui::track_content_rect(full_bounds, app.timeline_flow, app.ui_metrics());
        let (_, thumb_before) = app
            .recording_clip_scrollbar_rects(content_rect, app.project.active_track().unwrap())
            .expect("scrollbar");

        app.project
            .active_track_mut()
            .unwrap()
            .recording_clip_scroll = 2;
        let (_, thumb_after) = app
            .recording_clip_scrollbar_rects(content_rect, app.project.active_track().unwrap())
            .expect("scrollbar");

        assert!(thumb_after.x > thumb_before.x);
    }

    #[test]
    fn recording_targets_armed_tracks_before_active_track() {
        let mut app = App::new();
        app.project.clear_all_track_content();
        app.project.select_track(0);
        app.project.tracks[2].state.armed = true;

        app.apply_action(AppAction::ToggleRecording);
        assert!(app.project.tracks[2].active_take.is_some());
        assert!(app.project.tracks[0].active_take.is_none());

        app.transport_ticks = 960;
        app.playhead_ticks = 960;
        app.apply_action(AppAction::ToggleRecording);

        assert!(!app.project.tracks[2].regions.is_empty());
        assert!(app.project.tracks[0].regions.is_empty());
    }

    #[test]
    fn stopping_playback_commits_active_recording() {
        let mut app = App::new();
        app.project.active_track_mut().unwrap().clear_content();

        app.apply_action(AppAction::ToggleRecording);
        app.transport_ticks = 960;
        app.playhead_ticks = 960;
        app.apply_action(AppAction::TogglePlayback);

        assert!(!app.project.transport.recording);
        assert!(!app.project.transport.playing);
        assert!(!app.project.active_track().unwrap().regions.is_empty());
        assert!(app.project.active_track().unwrap().active_take.is_none());
    }

    #[test]
    fn stopping_recording_syncs_cleared_take_back_to_runtime() {
        let mut app = App::new();
        app.project.active_track_mut().unwrap().clear_content();
        app.project.active_track_mut().unwrap().routing.input_port =
            TrackPortSelection::named(MidiPortRef::new("Test Input"));

        app.apply_action(AppAction::ToggleRecording);
        let input_port = app
            .project
            .active_track()
            .and_then(|track| track.routing.input_port.as_named_port().cloned())
            .expect("test track should have explicit input port");
        app.inject_midi_input_event(MidiInputEvent {
            port: input_port.clone(),
            channel: 1,
            message: MidiInputMessage::NoteOn {
                pitch: 64,
                velocity: 100,
            },
            received_at: std::time::Instant::now(),
            backend_timestamp_micros: None,
            sequence: 0,
        });
        app.transport_ticks = 960;
        app.playhead_ticks = 960;
        app.inject_midi_input_event(MidiInputEvent {
            port: input_port,
            channel: 1,
            message: MidiInputMessage::NoteOff { pitch: 64 },
            received_at: std::time::Instant::now(),
            backend_timestamp_micros: None,
            sequence: 0,
        });

        app.apply_action(AppAction::ToggleRecording);
        app.update_timing_from_runtime();

        assert!(!app.project.transport.recording);
        assert!(app.project.active_track().unwrap().active_take.is_none());
        assert!(!app.project.active_track().unwrap().regions.is_empty());
    }

    #[test]
    fn looped_track_recording_commits_inside_track_loop() {
        let mut app = App::new();
        let track = app.project.active_track_mut().unwrap();
        track.clear_content();
        track.state.loop_enabled = true;
        track.loop_region = crate::timeline::LoopRegion::new(960, 960);
        app.project.transport.quantize = crate::transport::QuantizeMode::Off;
        app.project.transport.loop_enabled = false;
        app.project.transport.loop_recording_extends_clip = false;
        app.transport_ticks = 1_680;
        app.playhead_ticks = 1_680;

        app.apply_action(AppAction::ToggleRecording);
        app.transport_ticks = 2_160;
        app.playhead_ticks = 1_200;
        app.apply_action(AppAction::ToggleRecording);

        let regions = &app.project.active_track().unwrap().regions;
        assert_eq!(regions.len(), 1);
        assert_eq!(region_span(regions[0]), (1_680, 240));
    }

    #[test]
    fn looped_track_recording_can_extend_clip_after_wrap() {
        let mut app = App::new();
        let track = app.project.active_track_mut().unwrap();
        track.clear_content();
        track.state.loop_enabled = true;
        track.loop_region = crate::timeline::LoopRegion::new(960, 960);
        app.project.transport.quantize = crate::transport::QuantizeMode::Off;
        app.project.transport.loop_enabled = false;
        app.project.transport.loop_recording_extends_clip = true;
        app.transport_ticks = 1_680;
        app.playhead_ticks = 1_680;

        app.apply_action(AppAction::ToggleRecording);
        app.transport_ticks = 2_160;
        app.playhead_ticks = 1_200;
        app.apply_action(AppAction::ToggleRecording);

        let regions = &app.project.active_track().unwrap().regions;
        assert_eq!(regions.len(), 1);
        assert_eq!(region_span(regions[0]), (960, 960));
    }

    #[test]
    fn looped_track_recording_preview_rebases_to_loop_start_after_wrap() {
        let mut app = App::new();
        let track = app.project.active_track_mut().unwrap();
        track.clear_content();
        track.state.loop_enabled = true;
        track.loop_region = crate::timeline::LoopRegion::new(960, 960);
        app.project.transport.quantize = crate::transport::QuantizeMode::Off;
        app.project.transport.loop_enabled = false;
        app.project.transport.loop_recording_extends_clip = true;
        app.transport_ticks = 1_680;
        app.playhead_ticks = 1_680;

        app.apply_action(AppAction::ToggleRecording);
        app.transport_ticks = 2_160;
        app.playhead_ticks = 1_200;

        let active_track = app.project.active_track().unwrap();
        let preview = active_track.preview_region(
            app.project.transport,
            app.record_capture_ticks(active_track),
            app.record_context(active_track),
        );

        assert_eq!(preview.map(region_span), Some((960, 960)));
    }

    #[test]
    fn detail_loop_range_uses_global_loop_when_track_loop_is_disabled() {
        let mut app = App::new();
        app.project.loop_region = crate::timeline::LoopRegion::new(960, 960);
        app.project.transport.loop_enabled = true;
        let track = app.project.active_track_mut().unwrap();
        track.state.loop_enabled = false;
        track.loop_region = crate::timeline::LoopRegion::new(0, 3_840);

        let detail_range = app.detail_loop_range(app.project.active_track().unwrap());

        assert_eq!(detail_range, crate::timeline::LoopRegion::new(960, 960));
    }

    #[test]
    fn record_context_prefers_track_loop_over_global_loop_when_both_are_enabled() {
        let mut app = App::new();
        app.project.loop_region = crate::timeline::LoopRegion::new(960, 960);
        app.project.transport.loop_enabled = true;
        let track = app.project.active_track_mut().unwrap();
        track.state.loop_enabled = true;
        track.loop_region = crate::timeline::LoopRegion::new(0, 3_840);

        let record_context = app
            .record_context(app.project.active_track().unwrap())
            .unwrap();

        assert_eq!(
            record_context.range,
            crate::timeline::LoopRegion::new(0, 3_840)
        );
        assert_eq!(record_context.wrap_basis_ticks, 0);
    }

    #[test]
    fn record_context_uses_transport_phase_for_track_loops_with_offset_start() {
        let mut app = App::new();
        app.project.transport.loop_enabled = false;
        let track = app.project.active_track_mut().unwrap();
        track.state.loop_enabled = true;
        track.loop_region = crate::timeline::LoopRegion::new(240, 960);

        let record_context = app
            .record_context(app.project.active_track().unwrap())
            .unwrap();

        assert_eq!(
            record_context.range,
            crate::timeline::LoopRegion::new(240, 960)
        );
        assert_eq!(record_context.wrap_basis_ticks, 0);
    }

    #[test]
    fn track_loop_recording_with_offset_start_commits_at_audible_playhead() {
        let mut app = App::new();
        app.project.transport.quantize = crate::transport::QuantizeMode::Off;
        app.project.transport.loop_enabled = false;
        let track = app.project.active_track_mut().unwrap();
        track.clear_content();
        track.state.loop_enabled = true;
        track.loop_region = crate::timeline::LoopRegion::new(240, 960);

        app.transport_ticks = 1_140;
        app.playhead_ticks = app.effective_track_playhead(app.project.active_track().unwrap());
        app.apply_action(AppAction::ToggleRecording);
        app.transport_ticks = 1_260;
        app.playhead_ticks = app.effective_track_playhead(app.project.active_track().unwrap());
        app.apply_action(AppAction::ToggleRecording);

        let committed = app
            .project
            .active_track()
            .unwrap()
            .regions
            .last()
            .copied()
            .unwrap();
        assert_eq!(region_span(committed), (420, 120));
    }

    #[test]
    fn looped_track_preview_clamps_to_loop_end_when_extension_is_off() {
        let mut app = App::new();
        let track = app.project.active_track_mut().unwrap();
        track.clear_content();
        track.state.loop_enabled = true;
        track.loop_region = crate::timeline::LoopRegion::new(2_880, 1_920);
        app.project.transport.quantize = crate::transport::QuantizeMode::Off;
        app.project.transport.loop_enabled = false;
        app.project.transport.loop_recording_extends_clip = false;
        app.transport_ticks = 5_600;
        app.playhead_ticks = 5_600;

        app.apply_action(AppAction::ToggleRecording);
        app.transport_ticks = 5_900;
        app.playhead_ticks = 2_980;

        let active_track = app.project.active_track().unwrap();
        let preview = active_track.preview_region(
            app.project.transport,
            app.record_capture_ticks(active_track),
            app.record_context(active_track),
        );

        assert_eq!(preview, Some(crate::timeline::Region::new(4_640, 160)));
    }
}
