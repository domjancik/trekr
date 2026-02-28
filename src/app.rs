use crate::engine::EngineConfig;
use crate::project::Project;
use crate::ui::LayoutMode;

/// App is the top-level composition root for the first vertical slice.
pub struct App {
    project: Project,
    engine_config: EngineConfig,
    layout_mode: LayoutMode,
}

impl App {
    pub fn new() -> Self {
        Self {
            project: Project::demo(),
            engine_config: EngineConfig::default(),
            layout_mode: LayoutMode::FixedFit,
        }
    }

    pub fn bootstrap_summary(&self) -> String {
        format!(
            "trekr bootstrap: project='{}', tracks={}, layout={:?}, sample_rate={}",
            self.project.name,
            self.project.tracks.len(),
            self.layout_mode,
            self.engine_config.sample_rate_hz
        )
    }
}
