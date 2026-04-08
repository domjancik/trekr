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
use std::path::{Path, PathBuf};
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
}

impl LauncherUiApp {
    fn new(state_path: PathBuf, state: LauncherState) -> Self {
        Self {
            state_path,
            state,
            ui_state: LauncherUiState::default(),
            remote_branches: Vec::new(),
            status_line: "Ready".to_string(),
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
            .state
            .default_project_path
            .as_ref()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| "(none)".to_string());
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
            format!("{}: {}", SettingsRow::ProjectPath.label(), project_value),
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
            SettingsRow::ProjectPath => {
                let value = self
                    .state
                    .default_project_path
                    .as_ref()
                    .map(|path| path.display().to_string())
                    .unwrap_or_default();
                self.state.default_project_path =
                    if value.eq_ignore_ascii_case("state-fixtures/ui-looped.json") || delta < 0 {
                        None
                    } else {
                        Some(PathBuf::from("state-fixtures/ui-looped.json"))
                    };
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
            None,
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
        let install = installs::install_branch(&repo_url, &branch, false)?;
        upsert_install(&mut self.state.installs, install);
        self.state.last_selected_branch = Some(branch.clone());
        self.status_line = format!("Installed/updated '{branch}'");
        self.persist_state()
    }

    fn activate_settings(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        match SettingsRow::ALL
            .get(self.ui_state.selected_settings_row)
            .copied()
            .unwrap_or(SettingsRow::RepoUrl)
        {
            SettingsRow::RepoUrl => self.refresh_branches(),
            SettingsRow::WindowMode | SettingsRow::StateMode | SettingsRow::ProjectPath => {
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
