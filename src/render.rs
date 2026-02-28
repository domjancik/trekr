use crate::project::Project;
use crate::timeline::LoopRegion;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderDetail {
    Summary,
    LoopFocus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PaneRenderModel {
    pub detail: RenderDetail,
    pub range: LoopRegion,
    pub visible_tracks: usize,
}

impl PaneRenderModel {
    pub fn full_song(project: &Project) -> Self {
        Self {
            detail: RenderDetail::Summary,
            range: project.full_song_range(),
            visible_tracks: project.tracks.len(),
        }
    }

    pub fn loop_detail(project: &Project) -> Self {
        Self {
            detail: RenderDetail::LoopFocus,
            range: project.loop_region,
            visible_tracks: project.tracks.len(),
        }
    }
}
