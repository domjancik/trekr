#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LauncherPage {
    Launch,
    Branches,
    Installs,
    Settings,
}

impl LauncherPage {
    pub const ALL: [Self; 4] = [Self::Launch, Self::Branches, Self::Installs, Self::Settings];

    pub fn label(self) -> &'static str {
        match self {
            Self::Launch => "Launch",
            Self::Branches => "Branches",
            Self::Installs => "Installs",
            Self::Settings => "Settings",
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::Launch => Self::Branches,
            Self::Branches => Self::Installs,
            Self::Installs => Self::Settings,
            Self::Settings => Self::Launch,
        }
    }

    pub fn previous(self) -> Self {
        match self {
            Self::Launch => Self::Settings,
            Self::Branches => Self::Launch,
            Self::Installs => Self::Branches,
            Self::Settings => Self::Installs,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsRow {
    RepoUrl,
    WindowMode,
    StateMode,
    ProjectPath,
}

impl SettingsRow {
    pub const ALL: [Self; 4] = [
        Self::RepoUrl,
        Self::WindowMode,
        Self::StateMode,
        Self::ProjectPath,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::RepoUrl => "Repo URL",
            Self::WindowMode => "Window Mode",
            Self::StateMode => "State Mode",
            Self::ProjectPath => "Project/State File",
        }
    }
}

#[derive(Debug, Clone)]
pub struct LauncherUiState {
    pub page: LauncherPage,
    pub selected_launch_index: usize,
    pub selected_branch_index: usize,
    pub selected_install_index: usize,
    pub selected_settings_row: usize,
}

impl Default for LauncherUiState {
    fn default() -> Self {
        Self {
            page: LauncherPage::Launch,
            selected_launch_index: 0,
            selected_branch_index: 0,
            selected_install_index: 0,
            selected_settings_row: 0,
        }
    }
}
