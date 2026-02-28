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

pub fn surface_rect(width: u32, height: u32) -> Rect {
    let gutter = 18_i32;
    let width = width as i32;
    let height = height as i32;
    Rect::new(
        gutter,
        gutter,
        (width - gutter * 2) as u32,
        (height - gutter * 2) as u32,
    )
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

pub fn track_column_pairs(bounds: Rect, track_count: usize) -> Vec<(Rect, Rect)> {
    if track_count == 0 {
        return Vec::new();
    }

    let pair_gap = 14_i32;
    let inner_gap = 6_i32;
    let total_pair_gap = pair_gap * (track_count.saturating_sub(1) as i32);
    let pair_width = ((bounds.width() as i32 - total_pair_gap) / track_count as i32).max(20);
    let sub_width = ((pair_width - inner_gap) / 2).max(8);
    let mut pairs = Vec::with_capacity(track_count);

    for index in 0..track_count {
        let pair_x = bounds.x + index as i32 * (pair_width + pair_gap);
        let full = Rect::new(pair_x, bounds.y, sub_width as u32, bounds.height());
        let detail = Rect::new(
            pair_x + sub_width + inner_gap,
            bounds.y,
            sub_width as u32,
            bounds.height(),
        );
        pairs.push((full, detail));
    }

    pairs
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

pub fn detail_badge_rect(header: Rect) -> Rect {
    Rect::new(
        header.x + (header.width() as i32 / 2),
        header.y + 4,
        (header.width() / 2).max(10),
        (header.height() - 8).max(6),
    )
}

pub fn passthrough_rail_rect(bounds: Rect) -> Rect {
    Rect::new(
        bounds.x + 2,
        bounds.y + 2,
        4,
        bounds.height().saturating_sub(4),
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeaderBadgeKind {
    TrackIndex,
    Armed,
    Muted,
    Solo,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HeaderBadge {
    pub kind: HeaderBadgeKind,
    pub rect: Rect,
}

pub fn header_badges(header: Rect) -> [HeaderBadge; 4] {
    let size = (header.height().saturating_sub(8)).max(6) as i32;
    let y = header.y + 4;
    [
        HeaderBadge {
            kind: HeaderBadgeKind::TrackIndex,
            rect: Rect::new(header.x + 4, y, size as u32, size as u32),
        },
        HeaderBadge {
            kind: HeaderBadgeKind::Armed,
            rect: Rect::new(header.x + 8 + size, y, size as u32, size as u32),
        },
        HeaderBadge {
            kind: HeaderBadgeKind::Muted,
            rect: Rect::new(header.x + 12 + size * 2, y, size as u32, size as u32),
        },
        HeaderBadge {
            kind: HeaderBadgeKind::Solo,
            rect: Rect::new(header.x + 16 + size * 3, y, size as u32, size as u32),
        },
    ]
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

pub fn playhead_rect_in_range(
    bounds: Rect,
    flow: TimelineFlow,
    view_start_ticks: u64,
    view_length_ticks: u64,
    position_ticks: u64,
) -> Result<Rect, String> {
    let view_length_ticks = view_length_ticks.max(1);
    let clamped = position_ticks.clamp(view_start_ticks, view_start_ticks + view_length_ticks);
    let relative_ticks = clamped.saturating_sub(view_start_ticks);
    let phase = relative_ticks as f32 / view_length_ticks as f32;

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

#[cfg(test)]
mod tests {
    use super::{
        HeaderBadgeKind, TimelineFlow, detail_badge_rect, header_badges, passthrough_rail_rect,
        playhead_rect_in_range, surface_rect, timeline_guides, track_column_pairs,
        track_header_rect,
    };
    use sdl3::rect::Rect;

    #[test]
    fn surface_rect_fills_window_with_margin() {
        let surface = surface_rect(1280, 720);

        assert_eq!(surface.width(), 1244);
        assert_eq!(surface.height(), 684);
    }

    #[test]
    fn track_column_pairs_build_full_detail_pairs() {
        let pairs = track_column_pairs(Rect::new(0, 0, 1000, 400), 4);
        let (full, detail) = pairs[0];

        assert!(detail.x > full.x);
        assert_eq!(full.height(), 400);
        assert_eq!(detail.height(), 400);
    }

    #[test]
    fn playhead_uses_major_axis_for_orientation() {
        let vertical = playhead_rect_in_range(
            Rect::new(10, 10, 300, 200),
            TimelineFlow::DownwardColumns,
            0,
            1000,
            500,
        )
        .expect("vertical playhead");
        let horizontal = playhead_rect_in_range(
            Rect::new(10, 10, 300, 200),
            TimelineFlow::AcrossRows,
            0,
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

    #[test]
    fn detail_badge_uses_header_space() {
        let badge = detail_badge_rect(Rect::new(20, 10, 40, 20));
        assert!(badge.x > 20);
        assert!(badge.width() > 0);
    }

    #[test]
    fn header_badges_include_track_and_state_markers() {
        let badges = header_badges(Rect::new(10, 10, 80, 20));
        assert_eq!(badges[0].kind, HeaderBadgeKind::TrackIndex);
        assert_eq!(badges[3].kind, HeaderBadgeKind::Solo);
    }

    #[test]
    fn passthrough_rail_stays_thin() {
        let rail = passthrough_rail_rect(Rect::new(10, 20, 80, 240));
        assert_eq!(rail.width(), 4);
        assert_eq!(rail.height(), 236);
    }
}
