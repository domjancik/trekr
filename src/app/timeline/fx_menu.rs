use super::*;
use sdl3::event::Event;
use sdl3::keyboard::{Keycode, Mod};
use sdl3::mouse::MouseButton;

#[derive(Clone, Debug)]
pub(crate) struct FxMenu {
    track: usize,
    context: TimelineContext,
    slot: usize,
    parameter: Option<usize>,
    original: Option<MidiFxSlot>,
    title: String,
    choices: Vec<(String, Option<MidiFxSlot>)>,
    selected: usize,
    current: usize,
    scroll: usize,
    anchor: (i32, i32),
}

impl App {
    fn fx_menu_allowed(&self) -> bool {
        self.page_state.current_page == AppPage::Timeline
            && self.overlay_state.active.is_none()
            && self.direct_mapping_state.mode == DirectMappingMode::Inactive
            && self.clip_align_session.is_none()
    }

    fn fx_menu_content_bounds(&self) -> Option<Rect> {
        let metrics = self.ui_metrics();
        let surface = crate::ui::surface_rect(self.viewport_size.0, self.viewport_size.1, metrics);
        let inset =
            crate::ui::inset_rect(surface, metrics.frame_inset_x_px, metrics.frame_inset_y_px)
                .ok()?;
        self.page_frame_layout(inset)
            .ok()
            .map(|(_, content, _)| content)
    }

    pub(crate) fn open_focused_fx_menu(&mut self) {
        if !self.fx_menu_allowed() {
            return;
        }
        let Some(chain) = self.page_state.selected_timeline_context.chain_kind() else {
            return;
        };
        let context = self.page_state.selected_timeline_context;
        let slot = self.selected_timeline_fx_row(chain);
        let parameter = (self.page_state.selected_timeline_fx_field
            == TimelineFxField::ParamPrimary)
            .then(|| self.selected_timeline_fx_param_window(chain));
        let Some(content) = self.fx_menu_content_bounds() else {
            return;
        };
        let Ok((_, _, timeline)) = self.timeline_page_layout(content) else {
            return;
        };
        let Some(layout) = self
            .visible_timeline_track_layouts(timeline)
            .into_iter()
            .find(|r| r.track_index == self.project.active_track_index)
        else {
            return;
        };
        let band = layout.fx_rect(context);
        self.open_fx_menu(
            self.project.active_track_index,
            context,
            slot,
            parameter,
            (band.x, band.y),
        );
    }

    fn open_fx_menu(
        &mut self,
        track: usize,
        context: TimelineContext,
        slot: usize,
        parameter: Option<usize>,
        anchor: (i32, i32),
    ) {
        let Some(chain) = context.chain_kind() else {
            return;
        };
        let Some(track_ref) = self.project.tracks.get(track) else {
            return;
        };
        let Some(original) = self.fx_chain(track_ref, chain).get(slot).cloned() else {
            return;
        };
        let (title, choices) = if let Some(parameter) = parameter {
            let Some(original) = original.as_ref() else {
                return;
            };
            let params = original.effect.inline_parameters();
            let Some(param) = params.get(parameter) else {
                return;
            };
            let choices = original
                .effect
                .inline_parameter_choices(
                    parameter,
                    self.project.tracks.len(),
                    self.project.transport.ppqn,
                )
                .into_iter()
                .map(|effect| {
                    let value = effect.inline_parameters()[parameter].value.clone();
                    (
                        value,
                        Some(MidiFxSlot {
                            enabled: original.enabled,
                            effect,
                        }),
                    )
                })
                .collect::<Vec<_>>();
            (
                format!("{}: {}", original.effect.kind().label(), param.label),
                choices,
            )
        } else {
            let mut choices = crate::midi_fx::MidiFxKind::ALL
                .into_iter()
                .map(|kind| {
                    let value = original
                        .as_ref()
                        .filter(|slot| slot.effect.kind() == kind)
                        .cloned()
                        .unwrap_or(MidiFxSlot {
                            enabled: original.as_ref().is_none_or(|slot| slot.enabled),
                            effect: MidiFx::default_for_kind(kind),
                        });
                    (kind.label().to_string(), Some(value))
                })
                .collect::<Vec<_>>();
            choices.push(("Empty slot".to_string(), None));
            ("Effect type".to_string(), choices)
        };
        if choices.is_empty() {
            return;
        }
        let current = choices
            .iter()
            .position(|(_, choice)| *choice == original)
            .unwrap_or(0);
        self.project.active_track_index = track;
        self.page_state.selected_timeline_context = context;
        self.set_selected_timeline_fx_row(chain, slot);
        self.page_state.selected_timeline_fx_field = if parameter.is_some() {
            TimelineFxField::ParamPrimary
        } else {
            TimelineFxField::Kind
        };
        if let Some(parameter) = parameter {
            self.set_selected_timeline_fx_param_window(chain, parameter);
        }
        self.status_state.hovered_target = None;
        self.status_state.hovered_fx_detail = None;
        self.fx_menu = Some(FxMenu {
            track,
            context,
            slot,
            parameter,
            original,
            title,
            choices,
            selected: current,
            current,
            scroll: 0,
            anchor,
        });
        self.move_fx_menu_selection(0);
    }

    fn fx_menu_geometry(&self) -> Option<(Rect, u32, usize)> {
        let menu = self.fx_menu.as_ref()?;
        let row_height = if self.ui_density_preset == UiDensityPreset::Touch {
            26
        } else {
            20
        };
        let rows = ((self.viewport_size.1.saturating_sub(56) / row_height) as usize)
            .max(1)
            .min(menu.choices.len());
        let text_width = menu
            .choices
            .iter()
            .map(|(label, _)| crate::ui::text_width(label, 1))
            .chain(std::iter::once(crate::ui::text_width(&menu.title, 1)))
            .max()
            .unwrap_or(80);
        let width = (text_width + 44)
            .max(210)
            .min(self.viewport_size.0.saturating_sub(16).max(1));
        let height =
            (rows as u32 * row_height + 40).min(self.viewport_size.1.saturating_sub(16).max(1));
        let x = menu.anchor.0.clamp(
            8.min(self.viewport_size.0 as i32 - width as i32),
            (self.viewport_size.0 as i32 - width as i32 - 8).max(8),
        );
        let y = menu.anchor.1.clamp(
            8.min(self.viewport_size.1 as i32 - height as i32),
            (self.viewport_size.1 as i32 - height as i32 - 8).max(8),
        );
        Some((Rect::new(x, y, width, height), row_height, rows))
    }

    fn fx_menu_row_at(&self, x: i32, y: i32) -> Option<usize> {
        let (rect, height, rows) = self.fx_menu_geometry()?;
        if !rect_contains(rect, x, y) || y < rect.y + 22 {
            return None;
        }
        let row = (y - rect.y - 22) as usize / height as usize;
        let menu = self.fx_menu.as_ref()?;
        (row < rows && row + menu.scroll < menu.choices.len()).then_some(row + menu.scroll)
    }

    pub(crate) fn move_fx_menu_selection(&mut self, delta: i32) {
        let Some((_, _, rows)) = self.fx_menu_geometry() else {
            return;
        };
        let Some(menu) = self.fx_menu.as_mut() else {
            return;
        };
        menu.selected = (menu.selected as i64 + i64::from(delta))
            .clamp(0, menu.choices.len() as i64 - 1) as usize;
        if menu.selected < menu.scroll {
            menu.scroll = menu.selected;
        }
        if menu.selected >= menu.scroll + rows {
            menu.scroll = menu.selected + 1 - rows;
        }
        menu.scroll = menu.scroll.min(menu.choices.len().saturating_sub(rows));
    }

    pub(crate) fn commit_fx_menu_choice(&mut self) {
        let Some(menu) = self.fx_menu.take() else {
            return;
        };
        let Some((_, choice)) = menu.choices.get(menu.selected) else {
            return;
        };
        let Some(chain_kind) = menu.context.chain_kind() else {
            return;
        };
        let Some(track) = self.project.tracks.get_mut(menu.track) else {
            return;
        };
        let chain = match chain_kind {
            MidiFxChainKind::Input => &mut track.midi_fx.input_fx,
            MidiFxChainKind::Output => &mut track.midi_fx.output_fx,
        };
        let Some(slot) = chain.get_mut(menu.slot) else {
            return;
        };
        // Discard stale menus rather than overwriting a newer edit.
        if *slot != menu.original || *slot == *choice {
            return;
        }
        *slot = choice.clone();
        if menu.parameter.is_none() {
            self.set_selected_timeline_fx_param_window(chain_kind, 0);
        }
        self.handle_timeline_fx_configuration_changed();
    }

    pub(crate) fn handle_fx_menu_pointer_event(&mut self, event: &Event) -> Option<AppControl> {
        if self.fx_menu.is_some() {
            if let Some((x, y)) =
                super::super::input::pointer_hover_position(event, self.viewport_size)
            {
                if let Some(index) = self.fx_menu_row_at(x, y) {
                    self.fx_menu.as_mut().unwrap().selected = index;
                }
                return Some(AppControl::Continue);
            }
            if let Event::MouseWheel { y, .. } = event {
                self.move_fx_menu_selection(if *y > 0.0 {
                    -3
                } else if *y < 0.0 {
                    3
                } else {
                    0
                });
                return Some(AppControl::Continue);
            }
            if let Event::MouseButtonDown {
                mouse_btn: MouseButton::Right,
                ..
            } = event
            {
                self.fx_menu = None;
                return Some(AppControl::Continue);
            }
            if let Some((x, y, source)) =
                super::super::input::pointer_down_position(event, self.viewport_size)
            {
                if let Some(index) = self.fx_menu_row_at(x, y) {
                    self.fx_menu.as_mut().unwrap().selected = index;
                    return Some(
                        self.apply_action_with_source(AppAction::ActivatePageItem, source),
                    );
                }
                self.fx_menu = None;
                return Some(AppControl::Continue);
            }
            return None;
        }
        if let Event::MouseButtonDown {
            mouse_btn: MouseButton::Right,
            x,
            y,
            ..
        } = event
        {
            if self.fx_menu_allowed() {
                let content = self.fx_menu_content_bounds()?;
                let (_, _, timeline) = self.timeline_page_layout(content).ok()?;
                for layout in self.visible_timeline_track_layouts(timeline) {
                    for context in [TimelineContext::InputFx, TimelineContext::OutputFx] {
                        let Some(hit) = self.timeline_fx_hit(
                            context,
                            layout.fx_rect(context),
                            &self.project.tracks[layout.track_index],
                            *x as i32,
                            *y as i32,
                        ) else {
                            continue;
                        };
                        let parameter = hit
                            .layout
                            .parameters
                            .iter()
                            .position(|rect| rect_contains(*rect, *x as i32, *y as i32));
                        if parameter.is_some()
                            || rect_contains(hit.layout.row, *x as i32, *y as i32)
                        {
                            self.open_fx_menu(
                                layout.track_index,
                                context,
                                hit.row_index,
                                parameter,
                                (*x as i32, *y as i32),
                            );
                        }
                        return Some(AppControl::Continue);
                    }
                }
            }
            return Some(AppControl::Continue);
        }
        None
    }

    pub(crate) fn handle_fx_menu_keyboard_event(&mut self, event: &Event) -> Option<AppControl> {
        let Event::KeyDown {
            keycode: Some(key),
            keymod,
            repeat: false,
            ..
        } = event
        else {
            return None;
        };
        if self.fx_menu.is_some() {
            let action = match *key {
                Keycode::Escape => Some(AppAction::CancelCurrentMode),
                Keycode::Return => Some(AppAction::ActivatePageItem),
                Keycode::Up | Keycode::Left | Keycode::Q => Some(AppAction::SelectPreviousPageItem),
                Keycode::Down | Keycode::Right | Keycode::E => Some(AppAction::SelectNextPageItem),
                Keycode::Home => {
                    self.move_fx_menu_selection(i32::MIN);
                    return Some(AppControl::Continue);
                }
                Keycode::End => {
                    self.move_fx_menu_selection(i32::MAX);
                    return Some(AppControl::Continue);
                }
                _ => None,
            };
            if let Some(action) = action {
                return Some(self.apply_action_with_source(action, ActionSource::Keyboard));
            }
        }
        if matches!(*key, Keycode::F10 | Keycode::Return)
            && keymod.intersects(Mod::LSHIFTMOD | Mod::RSHIFTMOD)
            && !keymod.intersects(
                Mod::LCTRLMOD
                    | Mod::RCTRLMOD
                    | Mod::LALTMOD
                    | Mod::RALTMOD
                    | Mod::LGUIMOD
                    | Mod::RGUIMOD,
            )
            && self.fx_menu_allowed()
            && self
                .page_state
                .selected_timeline_context
                .chain_kind()
                .is_some()
        {
            return Some(
                self.apply_action_with_source(
                    AppAction::OpenTimelineFxMenu,
                    ActionSource::Keyboard,
                ),
            );
        }
        None
    }

    pub(crate) fn draw_fx_menu<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let Some(menu) = self.fx_menu.as_ref() else {
            return Ok(());
        };
        let Some((rect, row_height, rows)) = self.fx_menu_geometry() else {
            return Ok(());
        };
        let theme = self.theme();
        canvas.set_draw_color(theme.io_pages.panel_bg);
        canvas.fill_rect(rect)?;
        canvas.set_draw_color(theme.io_pages.focus_border);
        canvas.draw_rect(rect)?;
        crate::ui::draw_text_fitted(
            canvas,
            &menu.title,
            Rect::new(rect.x + 7, rect.y + 7, rect.width().saturating_sub(14), 7),
            1,
            theme.text_on_fill(theme.io_pages.panel_bg),
        )?;
        for (row, (label, choice)) in menu.choices.iter().skip(menu.scroll).take(rows).enumerate() {
            let index = row + menu.scroll;
            let cell = Rect::new(
                rect.x + 3,
                rect.y + 22 + row as i32 * row_height as i32,
                rect.width().saturating_sub(6),
                row_height,
            );
            let fill = if index == menu.selected {
                theme.io_pages.routing_affordance_selected_fill
            } else {
                theme.io_pages.panel_bg
            };
            canvas.set_draw_color(fill);
            canvas.fill_rect(cell)?;
            let color = theme.text_on_fill(fill);
            let text_x = if menu.parameter.is_none() {
                cell.x + 31
            } else {
                cell.x + 17
            };
            if menu.parameter.is_none() {
                if let Some(slot) = choice {
                    super::fx_ui::draw_fx_icon(
                        canvas,
                        Rect::new(cell.x + 13, cell.y + (row_height as i32 - 14) / 2, 14, 14),
                        slot,
                        color,
                    )?;
                }
            }
            let y = cell.y + (row_height as i32 - 7) / 2;
            if index == menu.current {
                crate::ui::draw_text_fitted(canvas, "*", Rect::new(cell.x + 3, y, 6, 7), 1, color)?;
            }
            crate::ui::draw_text_fitted(
                canvas,
                label,
                Rect::new(text_x, y, (cell.right() - text_x - 3).max(1) as u32, 7),
                1,
                color,
            )?;
        }
        let footer = if menu.choices.len() > rows {
            format!(
                "Wheel  {}/{}  Enter / Esc",
                menu.selected + 1,
                menu.choices.len()
            )
        } else {
            "Up/Down  Enter / Esc".to_string()
        };
        crate::ui::draw_text_fitted(
            canvas,
            &footer,
            Rect::new(
                rect.x + 7,
                rect.bottom() - 12,
                rect.width().saturating_sub(14),
                7,
            ),
            1,
            theme.text_on_fill(theme.io_pages.panel_bg),
        )?;
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::midi_fx::MidiFxKind;
    fn app_with(kind: MidiFxKind) -> App {
        let mut app = App::new();
        app.mappings.clear();
        app.page_state.selected_timeline_context = TimelineContext::OutputFx;
        app.project.tracks[0].midi_fx.output_fx[0] = Some(MidiFxSlot {
            enabled: true,
            effect: MidiFx::default_for_kind(kind),
        });
        app
    }
    fn key(app: &mut App, key: Keycode) {
        app.handle_keyboard_event(&Event::KeyDown {
            timestamp: 0,
            window_id: 0,
            which: 0,
            raw: 0,
            keycode: Some(key),
            scancode: None,
            keymod: Mod::NOMOD,
            repeat: false,
        });
    }
    #[test]
    fn fx_menu_choices_cover_every_effect_parameter_and_valid_bounds() {
        let expected = [
            vec![7, 4, 10],
            vec![128, 128, 129],
            vec![49],
            vec![31],
            vec![8],
            vec![12, 2],
            vec![12, 2],
            vec![12, 2],
            vec![8],
            vec![6],
        ];
        for (kind, counts) in MidiFxKind::ALL.into_iter().zip(expected) {
            let effect = MidiFx::default_for_kind(kind);
            for (p, count) in counts.into_iter().enumerate() {
                let choices = effect.inline_parameter_choices(p, 6, 960);
                assert_eq!(choices.len(), count, "{kind:?} param {p}");
                assert!(choices.contains(&effect));
                for choice in choices {
                    assert_eq!(choice.kind(), kind);
                    for (other, param) in effect.inline_parameters().iter().enumerate() {
                        if other != p {
                            assert_eq!(choice.inline_parameters()[other].value, param.value);
                        }
                    }
                }
            }
        }
        let filter = MidiFx::NoteFilter {
            low: 30,
            high: 40,
            enabled_notes: vec![],
        };
        assert_eq!(filter.inline_parameter_choices(0, 6, 960).len(), 41);
        assert_eq!(filter.inline_parameter_choices(1, 6, 960).len(), 98);
        for p in [0, 1] {
            for choice in filter.inline_parameter_choices(p, 6, 960) {
                if let MidiFx::NoteFilter { low, high, .. } = choice {
                    assert!(low <= high);
                }
            }
        }
    }
    #[test]
    fn fx_type_menu_has_icons_all_kinds_empty_choice_and_single_undo() {
        let mut app = app_with(MidiFxKind::Arp);
        let original = app.project.tracks[0].midi_fx.output_fx[0].clone();
        app.open_fx_menu(0, TimelineContext::OutputFx, 0, None, (1200, 700));
        let menu = app.fx_menu.as_ref().unwrap();
        assert_eq!(menu.choices.len(), 11);
        assert_eq!(menu.choices[menu.current].1, original);
        let chosen = menu
            .choices
            .iter()
            .position(|(_, s)| {
                s.as_ref()
                    .is_some_and(|s| s.effect.kind() == MidiFxKind::Delay)
            })
            .unwrap();
        app.fx_menu.as_mut().unwrap().selected = chosen;
        app.apply_action(AppAction::ActivatePageItem);
        assert!(app.fx_menu.is_none());
        assert_eq!(
            app.project.tracks[0].midi_fx.output_fx[0]
                .as_ref()
                .unwrap()
                .effect
                .kind(),
            MidiFxKind::Delay
        );
        app.apply_action(AppAction::UndoTimeline);
        assert_eq!(app.project.tracks[0].midi_fx.output_fx[0], original);
    }
    #[test]
    fn fx_parameter_menu_commits_exact_value_by_pointer_and_undo() {
        let mut app = app_with(MidiFxKind::Arp);
        app.open_fx_menu(0, TimelineContext::OutputFx, 0, Some(2), (1200, 700));
        let menu = app.fx_menu.as_ref().unwrap();
        let index = menu
            .choices
            .iter()
            .position(|(label, _)| label == "40%")
            .unwrap();
        app.fx_menu.as_mut().unwrap().selected = index;
        app.move_fx_menu_selection(0);
        let (rect, height, _) = app.fx_menu_geometry().unwrap();
        let row = index - app.fx_menu.as_ref().unwrap().scroll;
        app.handle_pointer_event(&Event::MouseButtonDown {
            timestamp: 0,
            window_id: 0,
            which: 0,
            mouse_btn: MouseButton::Left,
            clicks: 1,
            x: (rect.x + 30) as f32,
            y: (rect.y + 22 + row as i32 * height as i32 + 5) as f32,
        });
        assert_eq!(
            app.project.tracks[0].midi_fx.output_fx[0]
                .as_ref()
                .unwrap()
                .effect
                .inline_parameters()[2]
                .value,
            "40%"
        );
        assert!(app.fx_menu.is_none());
        app.apply_action(AppAction::UndoTimeline);
        assert_eq!(
            app.project.tracks[0].midi_fx.output_fx[0]
                .as_ref()
                .unwrap()
                .effect
                .inline_parameters()[2]
                .value,
            "100%"
        );
    }
    #[test]
    fn fx_menu_scrolls_to_numeric_extremes_and_stays_on_small_screens() {
        let mut app = app_with(MidiFxKind::NoteFilter);
        for (w, h) in [(320, 240), (800, 480), (1280, 720)] {
            app.viewport_size = (w, h);
            app.open_fx_menu(
                0,
                TimelineContext::OutputFx,
                0,
                Some(1),
                (w as i32 - 1, h as i32 - 1),
            );
            let (rect, _, _) = app.fx_menu_geometry().unwrap();
            assert!(
                rect.x >= 0 && rect.y >= 0 && rect.right() <= w as i32 && rect.bottom() <= h as i32
            );
            key(&mut app, Keycode::Home);
            assert_eq!(app.fx_menu.as_ref().unwrap().selected, 0);
            key(&mut app, Keycode::End);
            let menu = app.fx_menu.as_ref().unwrap();
            assert_eq!(menu.selected, 127);
            assert!(menu.scroll > 0);
            key(&mut app, Keycode::Escape);
            assert!(app.fx_menu.is_none());
            assert_eq!(
                app.project.tracks[0].midi_fx.output_fx[0]
                    .as_ref()
                    .unwrap()
                    .effect
                    .inline_parameters()[1]
                    .value,
                "127"
            );
        }
    }
    #[test]
    fn fx_menu_outside_click_is_consumed_and_external_edits_close_it() {
        let mut app = app_with(MidiFxKind::Arp);
        let before = app.project.tracks[0].midi_fx.output_fx.clone();
        app.open_fx_menu(0, TimelineContext::OutputFx, 0, None, (100, 100));
        app.handle_pointer_event(&Event::MouseButtonDown {
            timestamp: 0,
            window_id: 0,
            which: 0,
            mouse_btn: MouseButton::Left,
            clicks: 1,
            x: 1.0,
            y: 1.0,
        });
        assert!(app.fx_menu.is_none());
        assert_eq!(app.project.tracks[0].midi_fx.output_fx, before);
        app.open_fx_menu(0, TimelineContext::OutputFx, 0, None, (100, 100));
        app.apply_action_with_source(AppAction::ToggleSelectedTimelineFx, ActionSource::Midi);
        assert!(app.fx_menu.is_none());
        assert!(
            !app.project.tracks[0].midi_fx.output_fx[0]
                .as_ref()
                .unwrap()
                .enabled
        );
    }
    #[test]
    fn fx_menu_keyboard_and_mapped_navigation_choose_without_pointer() {
        let mut app = app_with(MidiFxKind::Arp);
        app.page_state.selected_timeline_fx_field = TimelineFxField::ParamPrimary;
        app.set_selected_timeline_fx_param_window(MidiFxChainKind::Output, 2);
        app.handle_keyboard_event(&Event::KeyDown {
            timestamp: 0,
            window_id: 0,
            which: 0,
            raw: 0,
            keycode: Some(Keycode::Return),
            scancode: None,
            keymod: Mod::LSHIFTMOD,
            repeat: false,
        });
        assert!(app.fx_menu.is_some());
        app.apply_action(AppAction::SelectPreviousPageItem);
        app.apply_action(AppAction::ActivatePageItem);
        assert_eq!(
            app.project.tracks[0].midi_fx.output_fx[0]
                .as_ref()
                .unwrap()
                .effect
                .inline_parameters()[2]
                .value,
            "90%"
        );
    }
    #[test]
    fn fx_menu_renderer_captures_kind_and_long_numeric_options() {
        let mut app = app_with(MidiFxKind::NoteFilter);
        for (parameter, name) in [(None, "fx-type-menu"), (Some(1), "fx-value-menu")] {
            app.open_fx_menu(0, TimelineContext::OutputFx, 0, parameter, (200, 630));
            let mut canvas =
                sdl3::surface::Surface::new(1280, 720, sdl3::pixels::PixelFormat::RGBA32)
                    .unwrap()
                    .into_canvas()
                    .unwrap();
            app.draw(&mut canvas).unwrap();
            if let Some(dir) = std::env::var_os("TREKR_FX_REVIEW_DIR") {
                let path = std::path::PathBuf::from(dir);
                std::fs::create_dir_all(&path).unwrap();
                app.capture_surface_to_png(canvas.surface(), &path.join(format!("{name}.png")))
                    .unwrap();
            }
        }
    }
}
