use crate::present::window_present_plan;

use super::layout::{page_tabs_layout, preferred_branding_width};
use super::*;

pub(crate) struct TransportChipSpec {
    pub(crate) label: String,
    pub(crate) compact_label: Option<String>,
    pub(crate) sublabel: Option<String>,
    pub(crate) compact_sublabel: Option<String>,
    pub(crate) action: Option<AppAction>,
    pub(crate) fill: Color,
}

impl TransportChipSpec {
    fn button(
        label: impl Into<String>,
        sublabel: impl Into<String>,
        action: Option<AppAction>,
        fill: Color,
    ) -> Self {
        Self {
            label: label.into(),
            compact_label: None,
            sublabel: Some(sublabel.into()),
            compact_sublabel: None,
            action,
            fill,
        }
    }

    fn with_compact_labels(
        mut self,
        compact_label: impl Into<String>,
        compact_sublabel: impl Into<String>,
    ) -> Self {
        self.compact_label = Some(compact_label.into());
        self.compact_sublabel = Some(compact_sublabel.into());
        self
    }
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
        let metrics = self.ui_metrics();
        let (tabs_bounds, page_area_bounds) =
            crate::ui::split_top_strip(inset, metrics.tabs_height_px, metrics.tabs_gap_px)?;
        let footer_height = metrics.footer_height_px;
        let footer_gap = metrics.footer_gap_px;
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
        let metrics = self.ui_metrics();
        let surface = crate::ui::surface_rect(width, height, metrics);
        let inset =
            crate::ui::inset_rect(surface, metrics.frame_inset_x_px, metrics.frame_inset_y_px)?;
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

        let tabs = crate::ui::equal_columns(
            tabs_bounds,
            AppPage::ALL.len(),
            self.ui_metrics().tabs_column_gap_px,
        );
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
            crate::ui::draw_text_fitted(
                canvas,
                page.label(),
                Rect::new(tab.x + 30, tab.y + 8, tab.width().saturating_sub(36), 8),
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
        let text_color = contrasting_text_color(spec.fill, theme);
        let compact = chip.width() <= 58;
        let label = if compact {
            spec.compact_label.as_deref().unwrap_or(&spec.label)
        } else {
            &spec.label
        };
        let sublabel = if compact {
            spec.compact_sublabel
                .as_deref()
                .or(spec.sublabel.as_deref())
        } else {
            spec.sublabel.as_deref()
        };
        if let Some(sublabel) = sublabel {
            let top_rect = Rect::new(chip.x + 4, chip.y + 5, chip.width().saturating_sub(8), 8);
            let bottom_rect = Rect::new(
                chip.x + 4,
                chip.y + chip.height() as i32 - 13,
                chip.width().saturating_sub(8),
                8,
            );
            let top_label_rect =
                crate::app::support::ui_helpers::horizontally_center_text_rect(label, top_rect, 1);
            let bottom_label_rect = crate::app::support::ui_helpers::horizontally_center_text_rect(
                sublabel,
                bottom_rect,
                1,
            );
            crate::ui::draw_text_fitted(canvas, label, top_label_rect, 1, text_color)?;
            crate::ui::draw_text_fitted(canvas, sublabel, bottom_label_rect, 1, text_color)?;
        } else {
            let label_rect = crate::app::support::ui_helpers::horizontally_center_text_rect(
                label,
                crate::app::support::ui_helpers::chrome_compact_text_rect(chip),
                1,
            );
            crate::ui::draw_text_fitted(canvas, label, label_rect, 1, text_color)?;
        }
        Ok(())
    }

    pub(crate) fn transport_left_button_specs(&self) -> Vec<TransportChipSpec> {
        let theme = self.theme();
        vec![
            TransportChipSpec::button(
                "Play",
                on_off(self.project.transport.playing),
                Some(AppAction::TogglePlayback),
                if self.project.transport.playing {
                    theme.transport.play_active
                } else {
                    theme.transport.play_idle
                },
            )
            .with_compact_labels("Ply", on_off(self.project.transport.playing)),
            TransportChipSpec::button(
                "Rec",
                on_off(self.project.transport.recording),
                Some(AppAction::ToggleRecording),
                if self.project.transport.recording {
                    theme.transport.record_active
                } else {
                    theme.transport.record_idle
                },
            )
            .with_compact_labels("Rec", on_off(self.project.transport.recording)),
            TransportChipSpec::button(
                "Rec Mode",
                short_record_mode_label(self.project.transport.record_mode),
                Some(AppAction::CycleRecordMode),
                theme.transport.record_mode,
            )
            .with_compact_labels(
                "Mode",
                compact_record_mode_label(self.project.transport.record_mode),
            ),
            TransportChipSpec::button(
                "Rec Wrap",
                if self.project.transport.loop_recording_extends_clip {
                    "Ext"
                } else {
                    "Clamp"
                },
                Some(AppAction::ToggleLoopRecordingExtension),
                if self.project.transport.loop_recording_extends_clip {
                    theme.transport.loop_wrap_extend
                } else {
                    theme.transport.loop_wrap_clamp
                },
            )
            .with_compact_labels(
                "Wrap",
                if self.project.transport.loop_recording_extends_clip {
                    "Ext"
                } else {
                    "Clp"
                },
            ),
            TransportChipSpec::button(
                "Song Loop",
                on_off(self.project.transport.loop_enabled),
                Some(AppAction::ToggleGlobalLoop),
                theme.transport.song_loop,
            )
            .with_compact_labels("Loop", on_off(self.project.transport.loop_enabled)),
            TransportChipSpec::button(
                "Harmony",
                note_name(self.project.global_harmony.root),
                Some(AppAction::CycleGlobalHarmonyRoot),
                theme.transport.harmony,
            )
            .with_compact_labels("Harm", note_name(self.project.global_harmony.root)),
            TransportChipSpec::button(
                "Launch Q",
                on_off(self.project.transport.stored_loop_recall_quantized),
                Some(AppAction::ToggleStoredLoopRecallQuantize),
                if self.project.transport.stored_loop_recall_quantized {
                    theme.transport.launch_quantize_enabled
                } else {
                    theme.transport.launch_quantize_disabled
                },
            )
            .with_compact_labels(
                "Lnch Q",
                on_off(self.project.transport.stored_loop_recall_quantized),
            ),
            TransportChipSpec::button(
                "Launch",
                launch_quantize_label(self.project.transport.stored_loop_launch_quantize),
                Some(AppAction::CycleStoredLoopLaunchQuantize),
                theme.transport.launch_quantize_mode,
            )
            .with_compact_labels(
                "Lnch",
                launch_quantize_label(self.project.transport.stored_loop_launch_quantize),
            ),
        ]
    }

    pub(crate) fn transport_right_button_specs(&self) -> Vec<TransportChipSpec> {
        let theme = self.theme();
        vec![
            TransportChipSpec::button(
                "Link",
                on_off(self.project.transport.link_enabled),
                Some(AppAction::ToggleLinkEnabled),
                if self.project.transport.link_enabled {
                    theme.transport.link_active
                } else {
                    theme.transport.link_idle
                },
            )
            .with_compact_labels("Link", on_off(self.project.transport.link_enabled)),
            TransportChipSpec::button(
                "Link Sync",
                on_off(self.project.transport.link_start_stop_sync),
                Some(AppAction::ToggleLinkStartStopSync),
                theme.transport.link_start_stop,
            )
            .with_compact_labels("Sync", on_off(self.project.transport.link_start_stop_sync)),
        ]
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
            let label_rect = crate::app::support::ui_helpers::horizontally_center_text_rect(
                label,
                crate::app::support::ui_helpers::chrome_compact_text_rect(chip),
                1,
            );
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

        if let Some((title, detail)) = self.clip_align_footer_content() {
            let label_width = crate::ui::text_width(&title, 1) + 4;
            let label_rect = Rect::new(bounds.x + 8, bounds.y + 7, label_width, 8);
            crate::ui::draw_text_fitted(
                canvas,
                &title,
                label_rect,
                1,
                theme.app_chrome.footer_title_direct,
            )?;
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
                theme.app_chrome.footer_detail_direct,
            )?;
        } else if let Some((title, detail, badges)) = self.direct_mapping_footer_content() {
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

pub(crate) fn transport_strip_height(metrics: &crate::ui_density::UiMetrics) -> u32 {
    metrics.transport_strip_height_px
}

fn short_record_mode_label(mode: crate::transport::RecordMode) -> &'static str {
    match mode {
        crate::transport::RecordMode::Overdub => "Ovrdub",
        crate::transport::RecordMode::Replace => "Replace",
    }
}

fn compact_record_mode_label(mode: crate::transport::RecordMode) -> &'static str {
    match mode {
        crate::transport::RecordMode::Overdub => "Ovd",
        crate::transport::RecordMode::Replace => "Repl",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_frame_layout_matches_draw_content_height_contract() {
        let app = App::new();
        let surface = crate::ui::surface_rect(1280, 720, app.ui_metrics());
        let inset = crate::ui::inset_rect(
            surface,
            app.ui_metrics().frame_inset_x_px,
            app.ui_metrics().frame_inset_y_px,
        )
        .expect("inset");
        let (_, content_bounds, footer_bounds) = app.page_frame_layout(inset).expect("layout");
        let (_, page_area_bounds) = crate::ui::split_top_strip(
            inset,
            app.ui_metrics().tabs_height_px,
            app.ui_metrics().tabs_gap_px,
        )
        .expect("page split");

        assert_eq!(content_bounds.y, page_area_bounds.y);
        assert_eq!(
            footer_bounds.y + footer_bounds.height() as i32,
            page_area_bounds.y + page_area_bounds.height() as i32
        );
        assert_eq!(
            content_bounds.height()
                + app.ui_metrics().footer_height_px
                + app.ui_metrics().footer_gap_px as u32,
            page_area_bounds.height()
        );
    }

    #[test]
    fn cycle_global_harmony_root_updates_transport_chip_label() {
        let mut app = App::new();
        app.apply_action(AppAction::CycleGlobalHarmonyRoot);
        let labels = app
            .transport_left_button_specs()
            .into_iter()
            .map(|chip| (chip.label, chip.sublabel.unwrap_or_default()))
            .collect::<Vec<_>>();
        assert!(
            labels
                .iter()
                .any(|(label, value)| label == "Harmony" && value == "C#")
        );
    }

    #[test]
    fn transport_button_specs_include_compact_labels_for_tight_layouts() {
        let app = App::new();
        let mut specs = app.transport_left_button_specs();
        specs.extend(app.transport_right_button_specs());
        let expected_song_loop_state = on_off(app.project.transport.loop_enabled);
        assert!(specs.iter().any(|chip| {
            chip.label == "Song Loop"
                && chip.compact_label.as_deref() == Some("Loop")
                && chip.compact_sublabel.as_deref() == Some(expected_song_loop_state)
        }));
        assert!(specs.iter().any(|chip| {
            chip.label == "Link Sync" && chip.compact_label.as_deref() == Some("Sync")
        }));
        assert!(specs.iter().any(|chip| {
            chip.label == "Rec Mode"
                && chip.compact_label.as_deref() == Some("Mode")
                && chip.compact_sublabel.as_deref() == Some("Ovd")
        }));
    }
}
