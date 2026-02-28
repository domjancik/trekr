#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderDetail {
    Summary,
    LoopFocus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PaneRenderModel {
    pub detail: RenderDetail,
    pub visible_tracks: usize,
}
