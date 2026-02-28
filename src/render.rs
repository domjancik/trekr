use crate::project::Project;
use crate::timeline::LoopRegion;
use crate::ui::TrackOrientation;

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
    pub compaction: TrackCompaction,
}

impl PaneRenderModel {
    pub fn full_song(project: &Project) -> Self {
        Self {
            detail: RenderDetail::Summary,
            range: project.full_song_range(),
            visible_tracks: project.tracks.len(),
            compaction: TrackCompaction::from_track_count(project.tracks.len()),
        }
    }

    pub fn loop_detail(project: &Project) -> Self {
        Self {
            detail: RenderDetail::LoopFocus,
            range: project.loop_region,
            visible_tracks: project.tracks.len(),
            compaction: TrackCompaction::from_track_count(project.tracks.len()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrackCompaction {
    Comfortable,
    CompactEdges,
    Dense,
}

impl TrackCompaction {
    pub fn from_track_count(track_count: usize) -> Self {
        match track_count {
            0..=8 => Self::Comfortable,
            9..=16 => Self::CompactEdges,
            _ => Self::Dense,
        }
    }

    pub fn lane_hint(self, orientation: TrackOrientation) -> &'static str {
        match (self, orientation) {
            (Self::Comfortable, TrackOrientation::Rows) => "full-height lanes",
            (Self::Comfortable, TrackOrientation::Columns) => "full-width lanes",
            (Self::CompactEdges, TrackOrientation::Rows) => "smaller outer rows",
            (Self::CompactEdges, TrackOrientation::Columns) => "smaller outer columns",
            (Self::Dense, TrackOrientation::Rows) => "dense rows",
            (Self::Dense, TrackOrientation::Columns) => "dense columns",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::TrackCompaction;
    use crate::ui::TrackOrientation;

    #[test]
    fn compaction_scales_with_track_count() {
        assert_eq!(
            TrackCompaction::from_track_count(4),
            TrackCompaction::Comfortable
        );
        assert_eq!(
            TrackCompaction::from_track_count(12),
            TrackCompaction::CompactEdges
        );
        assert_eq!(
            TrackCompaction::from_track_count(20),
            TrackCompaction::Dense
        );
    }

    #[test]
    fn compaction_lane_hints_match_orientation() {
        assert_eq!(
            TrackCompaction::CompactEdges.lane_hint(TrackOrientation::Columns),
            "smaller outer columns"
        );
    }
}
