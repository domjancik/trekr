use crate::present::window_present_plan;

use super::layout::{page_tabs_layout, preferred_branding_width};
use super::*;

pub(crate) struct TransportChipSpec {
    pub(crate) label: String,
    pub(crate) action: Option<AppAction>,
    pub(crate) fill: Color,
}

impl App {
    pub(crate) fn draw_frame_surface(
        &self,
        pixel_format: PixelFormat,
    ) -> Result<sdl3::surface::Surface<'static>, Box<dyn std::error::Error>> {
        let width = self.viewport_size.0.max(1);
        let height = self.viewport_size.1.max(1);
        let surface = sdl3::surface::Surface::new(width, height, pixel_format)?;
        let mut canvas = surface.into_canvas()?;
        self.draw(&mut canvas)?;
        Ok(canvas.into_surface())
    }

    pub(crate) fn draw<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.draw_scene(canvas)?;
        canvas.present();
        Ok(())
    }

    pub(crate) fn page_frame_layout(
        &self,
        inset: Rect,
    ) -> Result<(Rect, Rect, Rect), Box<dyn std::error::Error>> {
        let (tabs_bounds, page_area_bounds) = crate::ui::split_top_strip(inset, 28, 12)?;
        let footer_height = 22_u32;
        let footer_gap = 8_i32;
        let footer_bounds = Rect::new(
            page_area_bounds.x,
            page_area_bounds.y + page_area_bounds.height() as i32 - footer_height as i32,
            page_area_bounds.width(),
            footer_height,
        );
        let content_bounds = Rect::new(
            page_area_bounds.x,
            page_area_bounds.y,
            page_area_bounds.width(),
            page_area_bounds
                .height()
                .saturating_sub(footer_height)
                .saturating_sub(footer_gap as u32),
        );
        Ok((tabs_bounds, content_bounds, footer_bounds))
    }

    fn draw_scene<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (width, height) = active_draw_size(canvas.output_size()?, self.viewport_size);
        let surface = crate::ui::surface_rect(width, height);
        let inset = crate::ui::inset_rect(surface, 24, 24)?;
        let (tabs_bounds, content_bounds, footer_bounds) = self.page_frame_layout(inset)?;
        let theme = self.theme();

        canvas.set_draw_color(theme.app_chrome.window_clear);
        canvas.clear();

        canvas.set_draw_color(theme.app_chrome.surface_fill);
        canvas.fill_rect(surface)?;
        canvas.set_draw_color(theme.app_chrome.surface_border);
        canvas.draw_rect(surface)?;

        if preferred_branding_width(tabs_bounds.width()) == 0 {
            self.draw_frame_brand_fallback(canvas, surface)?;
        }
        self.draw_page_tabs(canvas, tabs_bounds)?;

        render_page(self.page_state.current_page, self, canvas, content_bounds)?;

        self.draw_direct_mapping_targets(canvas, tabs_bounds, content_bounds)?;
        self.draw_overlay(canvas, inset)?;
        self.draw_footer(canvas, footer_bounds)?;
        Ok(())
    }

    fn draw_frame_brand_fallback<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        surface: Rect,
    ) -> Result<(), Box<dyn std::error::Error>> {
        branding::draw_frame_brand_fallback(canvas, surface, self.theme())
    }

    pub(crate) fn configure_window_canvas(
        &mut self,
        canvas: &mut Canvas<sdl3::video::Window>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let scale = effective_ui_scale(canvas.window().display_scale(), self.ui_scale_override);
        let output_size = canvas.output_size()?;
        self.viewport_size = logical_viewport_size(output_size, scale);
        canvas.set_scale(scale, scale)?;
        Ok(())
    }

    pub(crate) fn draw_window(
        &self,
        canvas: &mut Canvas<sdl3::video::Window>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (scale_x, scale_y) = canvas.scale();
        if !should_interpolate_window_scale(self.ui_scaling_mode, scale_x, scale_y) {
            return self.draw(canvas);
        }

        let output_size = canvas.output_size()?;
        let present_plan =
            window_present_plan(output_size, true, self.theme().app_chrome.window_clear);
        let logical_size = self.viewport_size;
        let texture_creator = canvas.texture_creator();
        let mut frame = texture_creator.create_texture_target(
            Some(texture_creator.default_pixel_format()),
            logical_size.0.max(1),
            logical_size.1.max(1),
        )?;
        frame.set_scale_mode(sdl3::render::ScaleMode::Linear);

        let mut draw_result: Result<(), Box<dyn std::error::Error>> = Ok(());
        canvas.with_texture_canvas(&mut frame, |texture_canvas| {
            draw_result = (|| -> Result<(), Box<dyn std::error::Error>> {
                texture_canvas.set_scale(1.0, 1.0)?;
                self.draw_scene(texture_canvas)
            })();
        })?;
        draw_result?;

        canvas.set_scale(1.0, 1.0)?;
        canvas.set_draw_color(present_plan.clear_color);
        canvas.clear();
        canvas.copy(&frame, None, present_plan.destination)?;
        canvas.present();
        canvas.set_scale(scale_x, scale_y)?;
        Ok(())
    }

    fn draw_overlay<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        bounds: Rect,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match self.overlay_state.active {
            Some(AppOverlay::MappingsQuickView) => self.draw_mappings_overlay(canvas, bounds),
            Some(AppOverlay::Discoverability) | None => Ok(()),
        }
    }

    fn draw_page_tabs<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        bounds: Rect,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (branding_bounds, tabs_bounds) = page_tabs_layout(bounds);
        self.draw_branding(canvas, branding_bounds)?;
        let theme = self.theme();

        let tabs = crate::ui::equal_columns(tabs_bounds, AppPage::ALL.len(), 10);
        for (index, page) in AppPage::ALL.iter().copied().enumerate() {
            let tab = tabs[index];
            let active = page == self.page_state.current_page;
            canvas.set_draw_color(if active {
                theme.app_chrome.tab_active_fill
            } else {
                theme.app_chrome.tab_inactive_fill
            });
            canvas.fill_rect(tab)?;
            canvas.set_draw_color(if active {
                theme.app_chrome.tab_active_border
            } else {
                theme.app_chrome.tab_inactive_border
            });
            canvas.draw_rect(tab)?;

            let accent = Rect::new(tab.x + 6, tab.y + 6, 18, tab.height().saturating_sub(12));
            let color = match page {
                AppPage::Timeline => theme.app_chrome.tab_accent_timeline,
                AppPage::Mappings => theme.app_chrome.tab_accent_mappings,
                AppPage::MidiIo => theme.app_chrome.tab_accent_midi_io,
                AppPage::Routing => theme.app_chrome.tab_accent_routing,
            };
            canvas.set_draw_color(color);
            canvas.fill_rect(accent)?;
            let label_rect = crate::app::support::ui_helpers::chrome_text_rect(Rect::new(
                tab.x + 29,
                tab.y + 1,
                tab.width().saturating_sub(31),
                tab.height().saturating_sub(2),
            ));
            crate::ui::draw_text_fitted(
                canvas,
                page.label(),
                label_rect,
                1,
                if active {
                    theme.app_chrome.tab_text_active
                } else {
                    theme.app_chrome.tab_text_inactive
                },
            )?;
        }

        Ok(())
    }

    fn draw_branding<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        bounds: Rect,
    ) -> Result<(), Box<dyn std::error::Error>> {
        branding::draw_branding(
            canvas,
            bounds,
            self.startup_started_at.elapsed(),
            self.theme(),
        )
    }

    pub(crate) fn draw_transport_chip<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        chip: Rect,
        spec: &TransportChipSpec,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let theme = self.theme();
        canvas.set_draw_color(spec.fill);
        canvas.fill_rect(chip)?;
        canvas.set_draw_color(theme.app_chrome.surface_border);
        canvas.draw_rect(chip)?;
        let label_rect = crate::app::support::ui_helpers::chrome_text_rect(chip);
        crate::ui::draw_text_fitted(
            canvas,
            &spec.label,
            label_rect,
            1,
            contrasting_text_color(spec.fill, theme),
        )?;
        Ok(())
    }

    pub(crate) fn transport_top_chip_specs(&self) -> Vec<TransportChipSpec> {
        let theme = self.theme();
        vec![
            TransportChipSpec {
                label: format!("Play {}", on_off(self.project.transport.playing)),
                action: Some(AppAction::TogglePlayback),
                fill: if self.project.transport.playing {
                    theme.transport.play_active
                } else {
                    theme.transport.play_idle
                },
            },
            TransportChipSpec {
                label: format!("Record {}", on_off(self.project.transport.recording)),
                action: Some(AppAction::ToggleRecording),
                fill: if self.project.transport.recording {
                    theme.transport.record_active
                } else {
                    theme.transport.record_idle
                },
            },
            TransportChipSpec {
                label: format!("Mode {}", self.project.transport.record_mode.label()),
                action: Some(AppAction::CycleRecordMode),
                fill: theme.transport.record_mode,
            },
        ]
    }

    pub(crate) fn transport_bottom_chip_specs(&self) -> Vec<TransportChipSpec> {
        let theme = self.theme();
        vec![
            TransportChipSpec {
                label: format!(
                    "Wrap {}",
                    if self.project.transport.loop_recording_extends_clip {
                        "Extend"
                    } else {
                        "Clamp"
                    }
                ),
                action: Some(AppAction::ToggleLoopRecordingExtension),
                fill: if self.project.transport.loop_recording_extends_clip {
                    theme.transport.loop_wrap_extend
                } else {
                    theme.transport.loop_wrap_clamp
                },
            },
            TransportChipSpec {
                label: format!("Song Loop {}", on_off(self.project.transport.loop_enabled)),
                action: Some(AppAction::ToggleGlobalLoop),
                fill: theme.transport.song_loop,
            },
            TransportChipSpec {
                label: format!("Tempo {}", self.project.transport.tempo_bpm),
                action: None,
                fill: theme.transport.tempo,
            },
            TransportChipSpec {
                label: format!("Harmony {}", note_name(self.project.global_harmony.root)),
                action: Some(AppAction::CycleGlobalHarmonyRoot),
                fill: theme.transport.harmony,
            },
            TransportChipSpec {
                label: format!("NoteAdd {}", on_off(self.note_additive_select_held)),
                action: None,
                fill: if self.note_additive_select_held {
                    theme.transport.note_add_held
                } else {
                    theme.transport.note_add_idle
                },
            },
        ]
    }

    pub(crate) fn transport_link_chip_specs(&self) -> Vec<TransportChipSpec> {
        let theme = self.theme();
        vec![
            TransportChipSpec {
                label: format!("Link {}", on_off(self.project.transport.link_enabled)),
                action: Some(AppAction::ToggleLinkEnabled),
                fill: if self.project.transport.link_enabled {
                    theme.transport.link_active
                } else {
                    theme.transport.link_idle
                },
            },
            TransportChipSpec {
                label: format!(
                    "Start/Stop {}",
                    on_off(self.project.transport.link_start_stop_sync)
                ),
                action: Some(AppAction::ToggleLinkStartStopSync),
                fill: theme.transport.link_start_stop,
            },
        ]
    }

    pub(crate) fn transport_status_chip_specs(&self) -> Vec<TransportChipSpec> {
        let theme = self.theme();
        vec![
            TransportChipSpec {
                label: format!(
                    "LaunchQ {}",
                    on_off(self.project.transport.stored_loop_recall_quantized)
                ),
                action: Some(AppAction::ToggleStoredLoopRecallQuantize),
                fill: if self.project.transport.stored_loop_recall_quantized {
                    theme.transport.launch_quantize_enabled
                } else {
                    theme.transport.launch_quantize_disabled
                },
            },
            TransportChipSpec {
                label: format!(
                    "Launch {}",
                    launch_quantize_label(self.project.transport.stored_loop_launch_quantize)
                ),
                action: Some(AppAction::CycleStoredLoopLaunchQuantize),
                fill: theme.transport.launch_quantize_mode,
            },
            TransportChipSpec {
                label: format!("Quant {}", quantize_label(self.project.transport.quantize)),
                action: None,
                fill: theme.transport.quantize,
            },
            TransportChipSpec {
                label: format!("Peers {}", self.link_snapshot.peers),
                action: None,
                fill: theme.transport.peers,
            },
        ]
    }

    pub(crate) fn transport_right_panel_width(&self, bounds: Rect) -> u32 {
        let top_row = chip_row_width(&self.transport_link_chip_specs())
            .saturating_add(96)
            .saturating_add(12);
        let bottom_row = chip_row_width(&self.transport_status_chip_specs()).saturating_add(12);
        let desired = top_row.max(bottom_row).max(236);
        let max_allowed = bounds.width().saturating_sub(220).max(236);
        desired.min(max_allowed)
    }

    fn draw_footer<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        bounds: Rect,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let theme = self.theme();
        canvas.set_draw_color(theme.app_chrome.footer_bg);
        canvas.fill_rect(bounds)?;
        canvas.set_draw_color(theme.app_chrome.surface_border);
        canvas.draw_rect(bounds)?;

        let overlay_chips = [
            (
                "F5 Mappings",
                self.overlay_state.active == Some(AppOverlay::MappingsQuickView),
                theme.app_chrome.footer_chip_mappings,
            ),
            (
                "F7 Discover",
                self.overlay_state.active == Some(AppOverlay::Discoverability),
                theme.app_chrome.footer_chip_discover,
            ),
            (
                "F8 Direct",
                self.direct_mapping_state.mode != DirectMappingMode::Inactive,
                theme.app_chrome.footer_chip_direct,
            ),
        ];
        let mut right_edge = bounds.x + bounds.width() as i32 - 6;
        for (label, active, color) in overlay_chips.into_iter().rev() {
            let width = crate::ui::text_width(label, 1) + 10;
            let chip = Rect::new(
                right_edge - width as i32,
                bounds.y + 5,
                width,
                bounds.height().saturating_sub(10),
            );
            canvas.set_draw_color(if active {
                color
            } else {
                theme.app_chrome.footer_chip_inactive
            });
            canvas.fill_rect(chip)?;
            let label_rect = crate::app::support::ui_helpers::chrome_text_rect(chip);
            crate::ui::draw_text_fitted(
                canvas,
                label,
                label_rect,
                1,
                if active {
                    theme.app_chrome.footer_text_active
                } else {
                    theme.app_chrome.footer_text_inactive
                },
            )?;
            right_edge = chip.x - 6;
        }

        if let Some((title, detail, badges)) = self.direct_mapping_footer_content() {
            let label_width = crate::ui::text_width(&title, 1) + 4;
            let label_rect = Rect::new(bounds.x + 8, bounds.y + 7, label_width, 8);
            crate::ui::draw_text_fitted(
                canvas,
                &title,
                label_rect,
                1,
                theme.app_chrome.footer_title_direct,
            )?;
            let detail_left = label_rect.x + label_rect.width() as i32 + 8;
            let detail_width = (right_edge - detail_left).max(0) as u32;
            if !badges.is_empty() {
                self.draw_mapping_badges(
                    canvas,
                    Rect::new(
                        detail_left,
                        bounds.y + 3,
                        detail_width,
                        bounds.height().saturating_sub(6),
                    ),
                    &badges,
                    badges.len(),
                    4,
                    10,
                )?;
            } else {
                crate::ui::draw_text_fitted(
                    canvas,
                    &detail,
                    Rect::new(detail_left, bounds.y + 7, detail_width, 8),
                    1,
                    theme.app_chrome.footer_detail_direct,
                )?;
            }
        } else if let Some(target) = self.status_state.hovered_target {
            let summary = self.summarize_discoverability_target(target);
            let label_width = crate::ui::text_width(&summary.title, 1) + 4;
            let label_rect = Rect::new(bounds.x + 8, bounds.y + 7, label_width, 8);
            crate::ui::draw_text_fitted(
                canvas,
                &summary.title,
                label_rect,
                1,
                theme.app_chrome.footer_text_inactive,
            )?;
            let badges_left = label_rect.x + label_rect.width() as i32 + 8;
            let badges_width = (right_edge - badges_left).max(0) as u32;
            if summary.badges.is_empty() {
                crate::ui::draw_text_fitted(
                    canvas,
                    "No mappings",
                    Rect::new(badges_left, bounds.y + 7, badges_width, 8),
                    1,
                    theme.app_chrome.footer_empty_mapping,
                )?;
            } else {
                self.draw_mapping_badges(
                    canvas,
                    Rect::new(
                        badges_left,
                        bounds.y + 3,
                        badges_width,
                        bounds.height().saturating_sub(6),
                    ),
                    &summary.badges,
                    summary.total_bindings,
                    4,
                    10,
                )?;
            }
        } else if let Some((title, detail)) = self.timeline_fx_footer_content() {
            let label_width = crate::ui::text_width(&title, 1) + 4;
            let label_rect = Rect::new(bounds.x + 8, bounds.y + 7, label_width, 8);
            crate::ui::draw_text_fitted(canvas, &title, label_rect, 1, theme.mappings.page_title)?;
            crate::ui::draw_text_fitted(
                canvas,
                &detail,
                Rect::new(
                    label_rect.x + label_rect.width() as i32 + 8,
                    bounds.y + 7,
                    (right_edge - label_rect.x - label_rect.width() as i32 - 12).max(0) as u32,
                    8,
                ),
                1,
                theme.app_chrome.detail_text,
            )?;
        } else {
            let last_action = self
                .status_state
                .history_message
                .clone()
                .or_else(|| {
                    self.status_state.last_action.map(|status| {
                        format!(
                            "Last Action: {} via {}",
                            action_label(status.action),
                            action_source_label(status.source)
                        )
                    })
                })
                .unwrap_or_else(|| "Last Action: Ready".to_string());
            crate::ui::draw_text_fitted(
                canvas,
                &last_action,
                Rect::new(
                    bounds.x + 8,
                    bounds.y + 7,
                    (right_edge - bounds.x - 12).max(0) as u32,
                    8,
                ),
                1,
                theme.app_chrome.detail_text,
            )?;
        }

        Ok(())
    }
}

pub(crate) fn transport_strip_height() -> u32 {
    34
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_frame_layout_matches_draw_content_height_contract() {
        let app = App::new();
        let surface = crate::ui::surface_rect(1280, 720);
        let inset = crate::ui::inset_rect(surface, 24, 24).expect("inset");
        let (_, content_bounds, footer_bounds) = app.page_frame_layout(inset).expect("layout");
        let (_, page_area_bounds) = crate::ui::split_top_strip(inset, 28, 12).expect("page split");

        assert_eq!(content_bounds.y, page_area_bounds.y);
        assert_eq!(
            footer_bounds.y + footer_bounds.height() as i32,
            page_area_bounds.y + page_area_bounds.height() as i32
        );
        assert_eq!(content_bounds.height() + 22 + 8, page_area_bounds.height());
    }

    #[test]
    fn cycle_global_harmony_root_updates_transport_chip_label() {
        let mut app = App::new();
        app.apply_action(AppAction::CycleGlobalHarmonyRoot);
        let labels = app
            .transport_bottom_chip_specs()
            .into_iter()
            .map(|chip| chip.label)
            .collect::<Vec<_>>();
        assert!(labels.iter().any(|label| label == "Harmony C#"));
    }
}
