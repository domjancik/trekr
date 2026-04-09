use crate::launcher::build_run_options;
use crate::launcher::catalog;
use crate::launcher::installs;
use crate::launcher::models::LauncherState;
use crate::launcher::process;
use crate::launcher::state;
use crate::launcher::ui::actions::{LauncherUiAction, resolve_keyboard};
use crate::launcher::ui::pages::{LauncherPage, LauncherUiState, SettingsRow};
use crate::launcher::{resolve_repo_url, upsert_install};
use sdl3::event::Event;
use sdl3::pixels::Color;
use sdl3::rect::Rect;
use sdl3::render::{Canvas, RenderTarget};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver};
use std::time::Duration;

pub fn run_ui(state_path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let mut state = state::load(&state_path).unwrap_or_default();
    let repo_url = resolve_repo_url(None, &state);
    if state.repo_url.is_none() {
        state.repo_url = Some(repo_url);
        state::save(&state_path, &state)?;
    }

    let mut app = LauncherUiApp::new(state_path, state);
    app.refresh_branches();
    app.run()
}

struct LauncherUiApp {
    state_path: PathBuf,
    state: LauncherState,
    ui_state: LauncherUiState,
    remote_branches: Vec<String>,
    status_line: String,
    install_job: Option<InstallJob>,
    state_file_input: Option<String>,
}

struct InstallJob {
    branch: String,
    receiver: Receiver<InstallJobMessage>,
}

enum InstallJobMessage {
    Progress(String),
    Finished(Result<crate::launcher::models::InstalledBuild, String>),
}

impl LauncherUiApp {
    fn new(state_path: PathBuf, state: LauncherState) -> Self {
        Self {
            state_path,
            state,
            ui_state: LauncherUiState::default(),
            remote_branches: Vec::new(),
            status_line: "Ready".to_string(),
            install_job: None,
            state_file_input: None,
        }
    }

    fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let sdl_context = sdl3::init()?;
        let video = sdl_context.video()?;
        let window = video
            .window("trekr launcher", 1280, 720)
            .position_centered()
            .resizable()
            .high_pixel_density()
            .build()
            .map_err(|err| err.to_string())?;
        let mut canvas = window.into_canvas();
        let mut event_pump = sdl_context.event_pump()?;

        'running: loop {
            for event in event_pump.poll_iter() {
                if self.handle_state_file_text_event(&event)? {
                    continue;
                }
                if self.handle_pointer_event(&event, &canvas)? {
                    continue;
                }
                if let Some(action) = resolve_keyboard(&event) {
                    if matches!(action, LauncherUiAction::Quit) {
                        break 'running;
                    }
                    if let Err(error) = self.apply_action(action) {
                        self.status_line = format!("{error}");
                    }
                }
            }

            self.poll_install_job()?;
            self.draw(&mut canvas)?;
            std::thread::sleep(Duration::from_millis(16));
        }

        self.persist_state()?;
        Ok(())
    }

    fn draw<T: RenderTarget>(
        &mut self,
        canvas: &mut Canvas<T>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let size = canvas.output_size().map_err(|err| err.to_string())?;
        let surface = crate::ui::surface_rect(size.0, size.1);
        let tabs_height = 24_u32;
        let footer_height = 24_u32;
        let content_height = surface
            .height()
            .saturating_sub(tabs_height + footer_height + 8);
        let tabs_rect = Rect::new(surface.x, surface.y, surface.width(), tabs_height);
        let content_rect = Rect::new(
            surface.x,
            surface.y + tabs_height as i32 + 4,
            surface.width(),
            content_height,
        );
        let footer_rect = Rect::new(
            surface.x,
            content_rect.y + content_rect.height() as i32 + 4,
            surface.width(),
            footer_height,
        );

        canvas.set_draw_color(Color::RGB(14, 18, 24));
        canvas.clear();

        self.draw_tabs(canvas, tabs_rect)?;
        self.draw_page(canvas, content_rect)?;
        self.draw_footer(canvas, footer_rect)?;
        canvas.present();
        Ok(())
    }

    fn draw_tabs<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        bounds: Rect,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let columns = crate::ui::equal_columns(bounds, LauncherPage::ALL.len(), 4);
        for (index, page) in LauncherPage::ALL.iter().copied().enumerate() {
            let rect = columns[index];
            let active = page == self.ui_state.page;
            canvas.set_draw_color(if active {
                Color::RGB(48, 68, 96)
            } else {
                Color::RGB(28, 36, 48)
            });
            canvas.fill_rect(rect)?;
            crate::ui::draw_text_fitted(
                canvas,
                page.label(),
                Rect::new(rect.x + 6, rect.y + 8, rect.width().saturating_sub(12), 8),
                1,
                if active {
                    Color::RGB(236, 242, 255)
                } else {
                    Color::RGB(172, 184, 202)
                },
            )?;
        }
        Ok(())
    }

    fn draw_page<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        bounds: Rect,
    ) -> Result<(), Box<dyn std::error::Error>> {
        canvas.set_draw_color(Color::RGB(22, 28, 38));
        canvas.fill_rect(bounds)?;
        match self.ui_state.page {
            LauncherPage::Launch => self.draw_launch_page(canvas, bounds),
            LauncherPage::Branches => self.draw_branches_page(canvas, bounds),
            LauncherPage::Installs => self.draw_installs_page(canvas, bounds),
            LauncherPage::Settings => self.draw_settings_page(canvas, bounds),
        }
    }

    fn draw_launch_page<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        bounds: Rect,
    ) -> Result<(), Box<dyn std::error::Error>> {
        crate::ui::draw_text(
            canvas,
            "LAUNCH INSTALLED BRANCH",
            bounds.x + 8,
            bounds.y + 8,
            1,
            Color::RGB(206, 222, 244),
        )?;
        let branches = self.launch_branches();
        self.draw_rows(
            canvas,
            Rect::new(
                bounds.x + 8,
                bounds.y + 24,
                bounds.width().saturating_sub(16),
                bounds.height().saturating_sub(74),
            ),
            &branches
                .iter()
                .map(|branch| {
                    let installed = self
                        .state
                        .installs
                        .iter()
                        .find(|entry| entry.branch == *branch);
                    if let Some(build) = installed {
                        format!("{branch}  |  {}  |  ready", build.commit)
                    } else {
                        format!("{branch}  |  not installed")
                    }
                })
                .collect::<Vec<_>>(),
            self.ui_state.selected_launch_index,
        )?;
        crate::ui::draw_text_fitted(
            canvas,
            &format!(
                "Args: window={}, state={}, project={}",
                self.state.default_window_mode,
                self.state.default_state_mode,
                self.state
                    .default_project_path
                    .as_ref()
                    .map(|path| path.display().to_string())
                    .unwrap_or_else(|| "(none)".to_string())
            ),
            Rect::new(
                bounds.x + 8,
                bounds.y + bounds.height() as i32 - 18,
                bounds.width().saturating_sub(16),
                8,
            ),
            1,
            Color::RGB(156, 170, 188),
        )?;
        Ok(())
    }

    fn draw_branches_page<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        bounds: Rect,
    ) -> Result<(), Box<dyn std::error::Error>> {
        crate::ui::draw_text(
            canvas,
            "REMOTE BRANCHES  (ENTER: TRACK/UNTRACK, R: REFRESH)",
            bounds.x + 8,
            bounds.y + 8,
            1,
            Color::RGB(206, 222, 244),
        )?;
        let rows = self
            .remote_branches
            .iter()
            .map(|branch| {
                let tracked = self
                    .state
                    .tracked_branches
                    .iter()
                    .any(|entry| entry == branch);
                if tracked {
                    format!("* {branch}")
                } else {
                    format!("  {branch}")
                }
            })
            .collect::<Vec<_>>();
        self.draw_rows(
            canvas,
            Rect::new(
                bounds.x + 8,
                bounds.y + 24,
                bounds.width().saturating_sub(16),
                bounds.height().saturating_sub(34),
            ),
            &rows,
            self.ui_state.selected_branch_index,
        )
    }

    fn draw_installs_page<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        bounds: Rect,
    ) -> Result<(), Box<dyn std::error::Error>> {
        crate::ui::draw_text(
            canvas,
            "TRACKED BRANCH INSTALLS  (ENTER: INSTALL/UPDATE)",
            bounds.x + 8,
            bounds.y + 8,
            1,
            Color::RGB(206, 222, 244),
        )?;
        let rows = self
            .state
            .tracked_branches
            .iter()
            .map(|branch| {
                if let Some(entry) = self
                    .state
                    .installs
                    .iter()
                    .find(|item| item.branch == *branch)
                {
                    format!("{branch}  |  {}", entry.commit)
                } else {
                    format!("{branch}  |  not installed")
                }
            })
            .collect::<Vec<_>>();
        self.draw_rows(
            canvas,
            Rect::new(
                bounds.x + 8,
                bounds.y + 24,
                bounds.width().saturating_sub(16),
                bounds.height().saturating_sub(34),
            ),
            &rows,
            self.ui_state.selected_install_index,
        )
    }

    fn draw_settings_page<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        bounds: Rect,
    ) -> Result<(), Box<dyn std::error::Error>> {
        crate::ui::draw_text(
            canvas,
            "SETTINGS  (Q/E ADJUST, ENTER APPLY/REFRESH)",
            bounds.x + 8,
            bounds.y + 8,
            1,
            Color::RGB(206, 222, 244),
        )?;

        let repo_value = self
            .state
            .repo_url
            .clone()
            .unwrap_or_else(|| "(none)".to_string());
        let project_value = self
            .state_file_input
            .as_ref()
            .map(|input| format!("{}\\{}", default_user_state_dir().display(), input))
            .or_else(|| {
                self.state
                    .default_project_path
                    .as_ref()
                    .map(|path| path.display().to_string())
            })
            .unwrap_or_else(|| "(none)".to_string());
        let install_dir_value = self
            .state
            .install_directory
            .as_ref()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| "(default user folder)".to_string());
        let ui_scale_value = self
            .state
            .default_ui_scale
            .map(|value| value.to_string())
            .unwrap_or_else(|| "Auto".to_string());
        let rows = vec![
            format!("{}: {}", SettingsRow::RepoUrl.label(), repo_value),
            format!(
                "{}: {}",
                SettingsRow::WindowMode.label(),
                self.state.default_window_mode
            ),
            format!(
                "{}: {}",
                SettingsRow::StateMode.label(),
                self.state.default_state_mode
            ),
            format!("{}: {}", SettingsRow::UiScale.label(), ui_scale_value),
            format!("{}: {}", SettingsRow::StateFile.label(), project_value),
            format!(
                "{}: {}",
                SettingsRow::InstallDirectory.label(),
                install_dir_value
            ),
            format!(
                "{}: {}",
                SettingsRow::SourceFallback.label(),
                if self.state.allow_source_build_fallback {
                    "On"
                } else {
                    "Off"
                }
            ),
        ];
        self.draw_rows(
            canvas,
            Rect::new(
                bounds.x + 8,
                bounds.y + 24,
                bounds.width().saturating_sub(16),
                bounds.height().saturating_sub(34),
            ),
            &rows,
            self.ui_state.selected_settings_row,
        )
    }

    fn draw_rows<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        bounds: Rect,
        rows: &[String],
        selected: usize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let row_height = 18_i32;
        let max_rows = (bounds.height() as i32 / row_height).max(0) as usize;
        for (index, row) in rows.iter().take(max_rows).enumerate() {
            let row_rect = Rect::new(
                bounds.x,
                bounds.y + index as i32 * row_height,
                bounds.width(),
                row_height as u32,
            );
            let is_selected = index == selected;
            canvas.set_draw_color(if is_selected {
                Color::RGB(56, 78, 108)
            } else {
                Color::RGB(26, 34, 46)
            });
            canvas.fill_rect(row_rect)?;
            crate::ui::draw_text_fitted(
                canvas,
                row,
                Rect::new(
                    row_rect.x + 6,
                    row_rect.y + 5,
                    row_rect.width().saturating_sub(12),
                    8,
                ),
                1,
                if is_selected {
                    Color::RGB(238, 244, 255)
                } else {
                    Color::RGB(178, 190, 210)
                },
            )?;
        }
        Ok(())
    }

    fn draw_footer<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        bounds: Rect,
    ) -> Result<(), Box<dyn std::error::Error>> {
        canvas.set_draw_color(Color::RGB(18, 24, 34));
        canvas.fill_rect(bounds)?;
        crate::ui::draw_text_fitted(
            canvas,
            &self.status_line,
            Rect::new(
                bounds.x + 6,
                bounds.y + 8,
                bounds.width().saturating_sub(12),
                8,
            ),
            1,
            Color::RGB(172, 188, 210),
        )?;
        Ok(())
    }

    fn handle_state_file_text_event(
        &mut self,
        event: &Event,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        if self.state_file_input.is_none() {
            return Ok(false);
        }

        match event {
            Event::TextInput { text, .. } => {
                if let Some(input) = &mut self.state_file_input {
                    input.push_str(text);
                    self.status_line =
                        format!("State filename: {}  (Enter save, Esc cancel)", input);
                }
                Ok(true)
            }
            Event::KeyDown {
                keycode: Some(sdl3::keyboard::Keycode::Backspace),
                repeat: false,
                ..
            } => {
                if let Some(input) = &mut self.state_file_input {
                    input.pop();
                    self.status_line =
                        format!("State filename: {}  (Enter save, Esc cancel)", input);
                }
                Ok(true)
            }
            Event::KeyDown {
                keycode: Some(sdl3::keyboard::Keycode::Return),
                repeat: false,
                ..
            } => {
                self.commit_state_file_input()?;
                Ok(true)
            }
            Event::KeyDown {
                keycode: Some(sdl3::keyboard::Keycode::Escape),
                repeat: false,
                ..
            } => {
                self.state_file_input = None;
                self.status_line = "State filename edit canceled".to_string();
                Ok(true)
            }
            _ => Ok(true),
        }
    }

    fn start_state_file_input(&mut self) {
        let initial = self
            .state
            .default_project_path
            .as_ref()
            .and_then(|path| path.file_name())
            .and_then(|name| name.to_str())
            .map(|name| name.to_string())
            .unwrap_or_else(|| "project.json".to_string());
        self.state_file_input = Some(initial.clone());
        self.status_line = format!(
            "State filename: {}  (base: {})",
            initial,
            default_user_state_dir().display()
        );
    }

    fn commit_state_file_input(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let Some(raw_input) = self.state_file_input.clone() else {
            return Ok(());
        };
        let trimmed = raw_input.trim();
        if trimmed.is_empty() {
            self.status_line =
                "State filename cannot be empty (Enter save, Esc cancel)".to_string();
            return Ok(());
        }

        let mut filename = std::path::Path::new(trimmed)
            .file_name()
            .and_then(|name| name.to_str())
            .map(|name| name.to_string())
            .ok_or_else(|| "Invalid state filename".to_string())?;
        if !filename.to_ascii_lowercase().ends_with(".json") {
            filename.push_str(".json");
        }

        let base_dir = default_user_state_dir();
        fs::create_dir_all(&base_dir)?;
        let final_path = base_dir.join(&filename);
        self.state.default_project_path = Some(final_path.clone());
        self.state_file_input = None;
        self.persist_state()?;
        self.status_line = format!("State file set to {}", final_path.display());
        Ok(())
    }

    fn handle_pointer_event<T: RenderTarget>(
        &mut self,
        event: &Event,
        canvas: &Canvas<T>,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        match event {
            Event::MouseButtonDown { x, y, .. } => {
                self.handle_pointer_position(
                    *x as i32,
                    *y as i32,
                    canvas.output_size()?.0,
                    canvas.output_size()?.1,
                );
                Ok(true)
            }
            Event::FingerDown { x, y, .. } => {
                let size = canvas.output_size()?;
                let px = (*x * size.0 as f32) as i32;
                let py = (*y * size.1 as f32) as i32;
                self.handle_pointer_position(px, py, size.0, size.1);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn handle_pointer_position(&mut self, x: i32, y: i32, width: u32, height: u32) {
        let surface = crate::ui::surface_rect(width, height);
        let tabs_height = 24_i32;
        let footer_height = 24_i32;
        let tabs_rect = Rect::new(surface.x, surface.y, surface.width(), tabs_height as u32);
        let content_rect = Rect::new(
            surface.x,
            surface.y + tabs_height + 4,
            surface.width(),
            surface
                .height()
                .saturating_sub((tabs_height + footer_height + 8) as u32),
        );

        if point_in_rect(x, y, tabs_rect) {
            let tabs = crate::ui::equal_columns(tabs_rect, LauncherPage::ALL.len(), 4);
            if let Some((index, _)) = tabs
                .iter()
                .enumerate()
                .find(|(_, rect)| point_in_rect(x, y, **rect))
            {
                self.ui_state.page = LauncherPage::ALL[index];
            }
            return;
        }

        if !point_in_rect(x, y, content_rect) {
            return;
        }

        let row_height = 18_i32;
        let row_index = ((y - (content_rect.y + 24)) / row_height).max(0) as usize;
        match self.ui_state.page {
            LauncherPage::Launch => self.ui_state.selected_launch_index = row_index,
            LauncherPage::Branches => self.ui_state.selected_branch_index = row_index,
            LauncherPage::Installs => self.ui_state.selected_install_index = row_index,
            LauncherPage::Settings => self.ui_state.selected_settings_row = row_index,
        }
    }

    fn apply_action(&mut self, action: LauncherUiAction) -> Result<(), Box<dyn std::error::Error>> {
        match action {
            LauncherUiAction::Quit => {}
            LauncherUiAction::ShowPage(page) => self.ui_state.page = page,
            LauncherUiAction::ShowNextPage => self.ui_state.page = self.ui_state.page.next(),
            LauncherUiAction::ShowPreviousPage => {
                self.ui_state.page = self.ui_state.page.previous()
            }
            LauncherUiAction::SelectPreviousItem => self.step_selection(-1),
            LauncherUiAction::SelectNextItem => self.step_selection(1),
            LauncherUiAction::AdjustBackward => self.adjust_setting(-1)?,
            LauncherUiAction::AdjustForward => self.adjust_setting(1)?,
            LauncherUiAction::ActivateItem => self.activate_current_item()?,
            LauncherUiAction::RefreshBranches => self.refresh_branches(),
        }
        Ok(())
    }

    fn step_selection(&mut self, delta: i32) {
        let len = match self.ui_state.page {
            LauncherPage::Launch => self.launch_branches().len(),
            LauncherPage::Branches => self.remote_branches.len(),
            LauncherPage::Installs => self.state.tracked_branches.len(),
            LauncherPage::Settings => SettingsRow::ALL.len(),
        };
        if len == 0 {
            return;
        }
        let step =
            |current: usize| -> usize { (current as i32 + delta).rem_euclid(len as i32) as usize };
        match self.ui_state.page {
            LauncherPage::Launch => {
                self.ui_state.selected_launch_index = step(self.ui_state.selected_launch_index)
            }
            LauncherPage::Branches => {
                self.ui_state.selected_branch_index = step(self.ui_state.selected_branch_index)
            }
            LauncherPage::Installs => {
                self.ui_state.selected_install_index = step(self.ui_state.selected_install_index)
            }
            LauncherPage::Settings => {
                self.ui_state.selected_settings_row = step(self.ui_state.selected_settings_row)
            }
        }
    }

    fn adjust_setting(&mut self, delta: i32) -> Result<(), Box<dyn std::error::Error>> {
        if self.ui_state.page != LauncherPage::Settings {
            return Ok(());
        }
        match SettingsRow::ALL
            .get(self.ui_state.selected_settings_row)
            .copied()
            .unwrap_or(SettingsRow::RepoUrl)
        {
            SettingsRow::RepoUrl => {}
            SettingsRow::WindowMode => {
                let values = ["windowed", "fullscreen", "kmsdrm-console"];
                self.state.default_window_mode =
                    cycle_label(&values, &self.state.default_window_mode, delta).to_string();
            }
            SettingsRow::StateMode => {
                let values = ["persisted", "demo", "empty"];
                self.state.default_state_mode =
                    cycle_label(&values, &self.state.default_state_mode, delta).to_string();
            }
            SettingsRow::UiScale => {
                let values = [None, Some(1.0_f32), Some(1.25), Some(1.5), Some(2.0)];
                let current_index = values
                    .iter()
                    .position(|candidate| *candidate == self.state.default_ui_scale)
                    .unwrap_or(0) as i32;
                self.state.default_ui_scale =
                    values[(current_index + delta).rem_euclid(values.len() as i32) as usize];
            }
            SettingsRow::StateFile => {
                let choices = collect_state_file_choices(self.state.default_project_path.clone());
                if !choices.is_empty() {
                    let current_index = choices
                        .iter()
                        .position(|candidate| {
                            Some(candidate.clone()) == self.state.default_project_path
                        })
                        .unwrap_or(0) as i32;
                    let next = (current_index + delta).rem_euclid(choices.len() as i32) as usize;
                    self.state.default_project_path = Some(choices[next].clone());
                }
            }
            SettingsRow::InstallDirectory => {
                let choices =
                    collect_install_directory_choices(self.state.install_directory.clone());
                if !choices.is_empty() {
                    let current_index = choices
                        .iter()
                        .position(|candidate| *candidate == self.state.install_directory)
                        .unwrap_or(0) as i32;
                    let next = (current_index + delta).rem_euclid(choices.len() as i32) as usize;
                    self.state.install_directory = choices[next].clone();
                }
            }
            SettingsRow::SourceFallback => {
                self.state.allow_source_build_fallback = delta > 0;
            }
        }
        self.persist_state()
    }

    fn activate_current_item(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        match self.ui_state.page {
            LauncherPage::Launch => self.activate_launch(),
            LauncherPage::Branches => self.activate_branches(),
            LauncherPage::Installs => self.activate_installs(),
            LauncherPage::Settings => self.activate_settings(),
        }
    }

    fn activate_launch(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let branches = self.launch_branches();
        let Some(branch) = branches.get(self.ui_state.selected_launch_index).cloned() else {
            self.status_line = "No branch selected".to_string();
            return Ok(());
        };
        let Some(install) = self
            .state
            .installs
            .iter()
            .find(|entry| entry.branch == branch)
            .cloned()
        else {
            self.status_line = format!("Branch '{branch}' is not installed");
            return Ok(());
        };
        let options = build_run_options(
            branch.clone(),
            self.state.default_project_path.clone(),
            Some(self.state.default_state_mode.clone()),
            Some(self.state.default_window_mode.clone()),
            self.state.default_ui_scale,
            Vec::new(),
        );
        let exit_code = process::run_installed(&install, &options)?;
        self.status_line = format!("Launched '{branch}' (exit code {exit_code})");
        self.state.last_selected_branch = Some(branch);
        self.persist_state()
    }

    fn activate_branches(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let Some(branch) = self
            .remote_branches
            .get(self.ui_state.selected_branch_index)
            .cloned()
        else {
            self.status_line = "No branch selected".to_string();
            return Ok(());
        };
        if self
            .state
            .tracked_branches
            .iter()
            .any(|entry| entry == &branch)
        {
            self.state.tracked_branches.retain(|entry| entry != &branch);
            self.status_line = format!("Untracked '{branch}'");
        } else {
            self.state.tracked_branches.push(branch.clone());
            self.state.tracked_branches.sort();
            self.status_line = format!("Tracked '{branch}'");
        }
        self.persist_state()
    }

    fn activate_installs(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.install_job.is_some() {
            self.status_line = "Install already running".to_string();
            return Ok(());
        }
        let Some(branch) = self
            .state
            .tracked_branches
            .get(self.ui_state.selected_install_index)
            .cloned()
        else {
            self.status_line = "No tracked branch selected".to_string();
            return Ok(());
        };
        let repo_url = resolve_repo_url(None, &self.state);
        let (tx, rx) = mpsc::channel::<InstallJobMessage>();
        let worker_branch = branch.clone();
        let allow_source_fallback = self.state.allow_source_build_fallback;
        let install_directory = self.state.install_directory.clone();
        std::thread::spawn(move || {
            let _ = tx.send(InstallJobMessage::Progress(format!(
                "Starting install for '{worker_branch}'"
            )));
            let result = installs::install_branch_with_progress(
                &repo_url,
                &worker_branch,
                false,
                allow_source_fallback,
                install_directory.as_deref(),
                |step| {
                    let _ = tx.send(InstallJobMessage::Progress(step.to_string()));
                },
            )
            .map_err(|error| error.to_string());
            let _ = tx.send(InstallJobMessage::Finished(result));
        });

        self.install_job = Some(InstallJob {
            branch: branch.clone(),
            receiver: rx,
        });
        self.status_line = format!("Installing '{branch}'...");
        Ok(())
    }

    fn activate_settings(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        match SettingsRow::ALL
            .get(self.ui_state.selected_settings_row)
            .copied()
            .unwrap_or(SettingsRow::RepoUrl)
        {
            SettingsRow::RepoUrl => self.refresh_branches(),
            SettingsRow::StateFile => {
                self.start_state_file_input();
            }
            SettingsRow::WindowMode
            | SettingsRow::StateMode
            | SettingsRow::UiScale
            | SettingsRow::InstallDirectory
            | SettingsRow::SourceFallback => {
                self.persist_state()?;
                self.status_line = "Settings saved".to_string();
            }
        }
        Ok(())
    }

    fn refresh_branches(&mut self) {
        let repo_url = resolve_repo_url(None, &self.state);
        match catalog::list_remote_branches(&repo_url) {
            Ok(branches) => {
                self.remote_branches = branches;
                self.status_line = format!("Loaded {} branches", self.remote_branches.len());
                if self.ui_state.selected_branch_index >= self.remote_branches.len() {
                    self.ui_state.selected_branch_index = 0;
                }
            }
            Err(error) => {
                self.status_line = format!("Branch refresh failed: {error}");
            }
        }
    }

    fn launch_branches(&self) -> Vec<String> {
        if self.state.tracked_branches.is_empty() {
            vec!["main".to_string()]
        } else {
            self.state.tracked_branches.clone()
        }
    }

    fn persist_state(&self) -> Result<(), Box<dyn std::error::Error>> {
        state::save(Path::new(&self.state_path), &self.state)
    }

    fn poll_install_job(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let Some(job) = self.install_job.take() else {
            return Ok(());
        };
        let mut keep_job = true;
        while let Ok(message) = job.receiver.try_recv() {
            match message {
                InstallJobMessage::Progress(message) => {
                    self.status_line = format!("{}: {message}", job.branch);
                }
                InstallJobMessage::Finished(result) => {
                    match result {
                        Ok(install) => {
                            upsert_install(&mut self.state.installs, install);
                            self.state.last_selected_branch = Some(job.branch.clone());
                            self.status_line = format!("Installed/updated '{}'", job.branch);
                            self.persist_state()?;
                        }
                        Err(error) => {
                            self.status_line =
                                format!("Install failed for '{}': {error}", job.branch);
                        }
                    }
                    keep_job = false;
                }
            }
        }

        if keep_job {
            self.install_job = Some(job);
        }
        Ok(())
    }
}

fn point_in_rect(x: i32, y: i32, rect: Rect) -> bool {
    x >= rect.x
        && y >= rect.y
        && x < rect.x + rect.width() as i32
        && y < rect.y + rect.height() as i32
}

fn cycle_label<'a>(options: &'a [&'a str], current: &str, delta: i32) -> &'a str {
    let current_index = options
        .iter()
        .position(|candidate| *candidate == current)
        .unwrap_or(0) as i32;
    options[(current_index + delta).rem_euclid(options.len() as i32) as usize]
}

fn collect_state_file_choices(current: Option<PathBuf>) -> Vec<PathBuf> {
    let mut choices = Vec::new();
    let state_dir = default_user_state_dir();
    if state_dir.exists() {
        if let Ok(entries) = fs::read_dir(&state_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
                {
                    choices.push(path);
                }
            }
        }
    }

    if let Some(current_path) = current {
        if !choices.iter().any(|candidate| candidate == &current_path) {
            choices.push(current_path);
        }
    }

    if choices.is_empty() {
        choices.push(state_dir.join("launcher-default.json"));
    } else {
        choices.sort();
    }
    choices
}

fn collect_install_directory_choices(current: Option<PathBuf>) -> Vec<Option<PathBuf>> {
    let mut choices = vec![None, Some(default_user_install_dir())];
    if let Some(path) = current {
        if !choices
            .iter()
            .any(|candidate| candidate.as_ref() == Some(&path))
        {
            choices.push(Some(path));
        }
    }
    choices
}

fn default_user_state_dir() -> PathBuf {
    if cfg!(windows) {
        if let Some(profile) = std::env::var_os("USERPROFILE") {
            return PathBuf::from(profile)
                .join("Documents")
                .join("trekr")
                .join("artifacts")
                .join("state");
        }
    }
    if let Some(home) = std::env::var_os("HOME") {
        return PathBuf::from(home)
            .join("Documents")
            .join("trekr")
            .join("artifacts")
            .join("state");
    }
    PathBuf::from("artifacts/state")
}

fn default_user_install_dir() -> PathBuf {
    if cfg!(windows) {
        if let Some(profile) = std::env::var_os("USERPROFILE") {
            return PathBuf::from(profile)
                .join("Documents")
                .join("trekr")
                .join("launcher-installs");
        }
    }
    if let Some(home) = std::env::var_os("HOME") {
        return PathBuf::from(home)
            .join("Documents")
            .join("trekr")
            .join("launcher-installs");
    }
    PathBuf::from("artifacts/launcher")
}
