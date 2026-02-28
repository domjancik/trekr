use crate::render::TrackCompaction;
use sdl3::rect::Rect;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutMode {
    FixedFit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimelineFlow {
    DownwardColumns,
    AcrossRows,
}

impl TimelineFlow {
    pub fn toggle(self) -> Self {
        match self {
            Self::DownwardColumns => Self::AcrossRows,
            Self::AcrossRows => Self::DownwardColumns,
        }
    }
}

pub fn split_panes(width: u32, height: u32) -> [Rect; 2] {
    let gutter = 18_i32;
    let width = width as i32;
    let height = height as i32;
    let pane_height = ((height - gutter * 3) / 2).max(100);

    [
        Rect::new(
            gutter,
            gutter,
            (width - gutter * 2) as u32,
            pane_height as u32,
        ),
        Rect::new(
            gutter,
            gutter * 2 + pane_height,
            (width - gutter * 2) as u32,
            pane_height as u32,
        ),
    ]
}

pub fn inset_rect(rect: Rect, inset_x: i32, inset_y: i32) -> Result<Rect, String> {
    let width = rect.width() as i32 - inset_x * 2;
    let height = rect.height() as i32 - inset_y * 2;

    if width <= 0 || height <= 0 {
        return Err("pane inset produced a non-positive rectangle".to_string());
    }

    Ok(Rect::new(
        rect.x + inset_x,
        rect.y + inset_y,
        width as u32,
        height as u32,
    ))
}

pub fn lane_rects(
    bounds: Rect,
    track_count: usize,
    flow: TimelineFlow,
    compaction: TrackCompaction,
) -> Vec<Rect> {
    if track_count == 0 {
        return Vec::new();
    }

    let weights: Vec<u32> = (0..track_count)
        .map(|index| lane_weight(track_count, index, compaction))
        .collect();
    let total_weight: u32 = weights.iter().sum();

    let mut offset = 0_i32;
    let mut lanes = Vec::with_capacity(track_count);

    for (index, weight) in weights.iter().enumerate() {
        match flow {
            TimelineFlow::DownwardColumns => {
                let is_last = index + 1 == track_count;
                let available = bounds.width() as i32;
                let lane_width = if is_last {
                    available - offset
                } else {
                    ((available as i64 * *weight as i64) / total_weight as i64) as i32
                }
                .max(6);
                let lane = Rect::new(
                    bounds.x + offset,
                    bounds.y,
                    lane_width as u32,
                    bounds.height(),
                );
                lanes.push(inset_rect(lane, 3, 3).unwrap_or(lane));
                offset += lane_width;
            }
            TimelineFlow::AcrossRows => {
                let is_last = index + 1 == track_count;
                let available = bounds.height() as i32;
                let lane_height = if is_last {
                    available - offset
                } else {
                    ((available as i64 * *weight as i64) / total_weight as i64) as i32
                }
                .max(6);
                let lane = Rect::new(
                    bounds.x,
                    bounds.y + offset,
                    bounds.width(),
                    lane_height as u32,
                );
                lanes.push(inset_rect(lane, 3, 3).unwrap_or(lane));
                offset += lane_height;
            }
        }
    }

    lanes
}

pub fn region_blocks(lane: Rect, seed: usize, flow: TimelineFlow) -> Vec<Rect> {
    let major = match flow {
        TimelineFlow::DownwardColumns => lane.height() as i32,
        TimelineFlow::AcrossRows => lane.width() as i32,
    };
    let block_count = 2 + (seed % 3);
    let mut blocks = Vec::with_capacity(block_count);

    for block_index in 0..block_count {
        let start_ratio = 0.12 + 0.22 * block_index as f32;
        let size_ratio = 0.12 + 0.03 * ((seed + block_index) % 4) as f32;
        let major_start = (major as f32 * start_ratio) as i32;
        let major_size = (major as f32 * size_ratio) as i32;

        let rect = match flow {
            TimelineFlow::DownwardColumns => Rect::new(
                lane.x + 10,
                lane.y + major_start,
                lane.width().saturating_sub(20),
                major_size.max(6) as u32,
            ),
            TimelineFlow::AcrossRows => Rect::new(
                lane.x + major_start,
                lane.y + 10,
                major_size.max(6) as u32,
                lane.height().saturating_sub(20),
            ),
        };
        blocks.push(rect);
    }

    blocks
}

pub fn track_header_rect(lane: Rect, flow: TimelineFlow) -> Rect {
    match flow {
        TimelineFlow::DownwardColumns => Rect::new(lane.x, lane.y, lane.width(), 20),
        TimelineFlow::AcrossRows => Rect::new(lane.x, lane.y, 56, lane.height()),
    }
}

pub fn timeline_guides(bounds: Rect, flow: TimelineFlow) -> Vec<Rect> {
    let guide_count: usize = 8;
    let mut guides = Vec::with_capacity(guide_count.saturating_sub(1));

    for step in 1..guide_count {
        let ratio = step as f32 / guide_count as f32;
        let guide = match flow {
            TimelineFlow::DownwardColumns => {
                let y = bounds.y + (bounds.height() as f32 * ratio) as i32;
                Rect::new(bounds.x, y, bounds.width(), 1)
            }
            TimelineFlow::AcrossRows => {
                let x = bounds.x + (bounds.width() as f32 * ratio) as i32;
                Rect::new(x, bounds.y, 1, bounds.height())
            }
        };
        guides.push(guide);
    }

    guides
}

pub fn playhead_rect(
    bounds: Rect,
    flow: TimelineFlow,
    range_ticks: u64,
    elapsed_ms: u64,
) -> Result<Rect, String> {
    let range_ticks = range_ticks.max(1);
    let phase = (elapsed_ms % range_ticks) as f32 / range_ticks as f32;

    match flow {
        TimelineFlow::DownwardColumns => {
            let y = bounds.y + (bounds.height() as f32 * phase) as i32;
            Ok(Rect::new(bounds.x, y, bounds.width(), 2))
        }
        TimelineFlow::AcrossRows => {
            let x = bounds.x + (bounds.width() as f32 * phase) as i32;
            Ok(Rect::new(x, bounds.y, 2, bounds.height()))
        }
    }
}

fn lane_weight(track_count: usize, index: usize, compaction: TrackCompaction) -> u32 {
    if track_count <= 2 {
        return 100;
    }

    let is_outer = index == 0 || index + 1 == track_count;

    match (compaction, is_outer) {
        (TrackCompaction::Comfortable, _) => 100,
        (TrackCompaction::CompactEdges, true) => 60,
        (TrackCompaction::CompactEdges, false) => 100,
        (TrackCompaction::Dense, true) => 45,
        (TrackCompaction::Dense, false) => 80,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        TimelineFlow, lane_rects, playhead_rect, split_panes, timeline_guides, track_header_rect,
    };
    use crate::render::TrackCompaction;
    use sdl3::rect::Rect;

    #[test]
    fn pane_split_stacks_two_full_width_views() {
        let [top, bottom] = split_panes(1280, 720);

        assert_eq!(top.width(), 1244);
        assert_eq!(bottom.width(), 1244);
        assert!(bottom.y > top.y);
    }

    #[test]
    fn compact_edges_shrinks_outer_columns() {
        let lanes = lane_rects(
            Rect::new(0, 0, 1000, 200),
            4,
            TimelineFlow::DownwardColumns,
            TrackCompaction::CompactEdges,
        );

        assert!(lanes[0].width() < lanes[1].width());
        assert!(lanes[3].width() < lanes[2].width());
    }

    #[test]
    fn playhead_uses_major_axis_for_orientation() {
        let vertical = playhead_rect(
            Rect::new(10, 10, 300, 200),
            TimelineFlow::DownwardColumns,
            1000,
            500,
        )
        .expect("vertical playhead");
        let horizontal = playhead_rect(
            Rect::new(10, 10, 300, 200),
            TimelineFlow::AcrossRows,
            1000,
            500,
        )
        .expect("horizontal playhead");

        assert_eq!(vertical.width(), 300);
        assert_eq!(horizontal.height(), 200);
    }

    #[test]
    fn downward_columns_use_top_headers() {
        let header = track_header_rect(Rect::new(10, 20, 80, 240), TimelineFlow::DownwardColumns);

        assert_eq!(header.height(), 20);
        assert_eq!(header.width(), 80);
    }

    #[test]
    fn timeline_guides_follow_time_axis() {
        let vertical = timeline_guides(Rect::new(0, 0, 200, 400), TimelineFlow::DownwardColumns);
        let horizontal = timeline_guides(Rect::new(0, 0, 200, 400), TimelineFlow::AcrossRows);

        assert_eq!(vertical[0].width(), 200);
        assert_eq!(horizontal[0].height(), 400);
    }
}
