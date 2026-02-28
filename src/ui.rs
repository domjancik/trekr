use sdl3::rect::Rect;
use sdl3::{pixels::Color, render::{Canvas, RenderTarget}};

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

pub fn split_top_strip(rect: Rect, strip_height: u32, gap: i32) -> Result<(Rect, Rect), String> {
    let remaining_height = rect.height() as i32 - strip_height as i32 - gap;
    if remaining_height <= 0 {
        return Err("split_top_strip produced a non-positive content rectangle".to_string());
    }

    Ok((
        Rect::new(rect.x, rect.y, rect.width(), strip_height),
        Rect::new(
            rect.x,
            rect.y + strip_height as i32 + gap,
            rect.width(),
            remaining_height as u32,
        ),
    ))
}

pub fn equal_columns(bounds: Rect, count: usize, gap: i32) -> Vec<Rect> {
    if count == 0 {
        return Vec::new();
    }

    let total_gap = gap * count.saturating_sub(1) as i32;
    let column_width = ((bounds.width() as i32 - total_gap) / count as i32).max(8);
    let mut columns = Vec::with_capacity(count);

    for index in 0..count {
        let x = bounds.x + index as i32 * (column_width + gap);
        columns.push(Rect::new(x, bounds.y, column_width as u32, bounds.height()));
    }

    columns
}

pub fn stacked_rows(bounds: Rect, count: usize, gap: i32) -> Vec<Rect> {
    if count == 0 {
        return Vec::new();
    }

    let total_gap = gap * count.saturating_sub(1) as i32;
    let row_height = ((bounds.height() as i32 - total_gap) / count as i32).max(8);
    let mut rows = Vec::with_capacity(count);

    for index in 0..count {
        let y = bounds.y + index as i32 * (row_height + gap);
        rows.push(Rect::new(bounds.x, y, bounds.width(), row_height as u32));
    }

    rows
}

pub fn draw_text<T: RenderTarget>(
    canvas: &mut Canvas<T>,
    text: &str,
    x: i32,
    y: i32,
    scale: u32,
    color: Color,
) -> Result<(), String> {
    let scale = scale.max(1) as i32;
    canvas.set_draw_color(color);

    let mut cursor_x = x;
    for character in text.chars() {
        draw_glyph(canvas, character.to_ascii_uppercase(), cursor_x, y, scale)?;
        cursor_x += (glyph_width() + 1) * scale;
    }

    Ok(())
}

pub fn text_width(text: &str, scale: u32) -> u32 {
    let scale = scale.max(1);
    text.chars().count() as u32 * ((glyph_width() as u32 + 1) * scale)
}

pub fn truncate_text_to_width(text: &str, max_width: u32, scale: u32) -> String {
    if text_width(text, scale) <= max_width {
        return text.to_string();
    }

    let ellipsis = "...";
    if text_width(ellipsis, scale) > max_width {
        return String::new();
    }

    let mut fitted = String::new();
    for character in text.chars() {
        let mut candidate = fitted.clone();
        candidate.push(character);
        candidate.push_str(ellipsis);
        if text_width(&candidate, scale) > max_width {
            break;
        }
        fitted.push(character);
    }

    if fitted.is_empty() {
        ellipsis.to_string()
    } else {
        fitted.push_str(ellipsis);
        fitted
    }
}

pub fn draw_text_fitted<T: RenderTarget>(
    canvas: &mut Canvas<T>,
    text: &str,
    bounds: Rect,
    scale: u32,
    color: Color,
) -> Result<(), String> {
    let fitted = truncate_text_to_width(text, bounds.width(), scale);
    if fitted.is_empty() {
        return Ok(());
    }

    draw_text(canvas, &fitted, bounds.x, bounds.y, scale, color)
}

fn draw_glyph<T: RenderTarget>(
    canvas: &mut Canvas<T>,
    character: char,
    x: i32,
    y: i32,
    scale: i32,
) -> Result<(), String> {
    let glyph = glyph_rows(character);
    for (row_index, row) in glyph.iter().copied().enumerate() {
        for column in 0..glyph_width() {
            let bit = 1 << (glyph_width() - 1 - column);
            if row & bit != 0 {
                canvas.fill_rect(Rect::new(
                    x + column * scale,
                    y + row_index as i32 * scale,
                    scale as u32,
                    scale as u32,
                ))
                .map_err(|error| error.to_string())?;
            }
        }
    }
    Ok(())
}

const fn glyph_width() -> i32 {
    5
}

fn glyph_rows(character: char) -> [u8; 7] {
    match character {
        'A' => [0x0E, 0x11, 0x11, 0x1F, 0x11, 0x11, 0x11],
        'B' => [0x1E, 0x11, 0x11, 0x1E, 0x11, 0x11, 0x1E],
        'C' => [0x0E, 0x11, 0x10, 0x10, 0x10, 0x11, 0x0E],
        'D' => [0x1C, 0x12, 0x11, 0x11, 0x11, 0x12, 0x1C],
        'E' => [0x1F, 0x10, 0x10, 0x1E, 0x10, 0x10, 0x1F],
        'F' => [0x1F, 0x10, 0x10, 0x1E, 0x10, 0x10, 0x10],
        'G' => [0x0E, 0x11, 0x10, 0x17, 0x11, 0x11, 0x0E],
        'H' => [0x11, 0x11, 0x11, 0x1F, 0x11, 0x11, 0x11],
        'I' => [0x1F, 0x04, 0x04, 0x04, 0x04, 0x04, 0x1F],
        'J' => [0x01, 0x01, 0x01, 0x01, 0x11, 0x11, 0x0E],
        'K' => [0x11, 0x12, 0x14, 0x18, 0x14, 0x12, 0x11],
        'L' => [0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x1F],
        'M' => [0x11, 0x1B, 0x15, 0x15, 0x11, 0x11, 0x11],
        'N' => [0x11, 0x19, 0x15, 0x13, 0x11, 0x11, 0x11],
        'O' => [0x0E, 0x11, 0x11, 0x11, 0x11, 0x11, 0x0E],
        'P' => [0x1E, 0x11, 0x11, 0x1E, 0x10, 0x10, 0x10],
        'Q' => [0x0E, 0x11, 0x11, 0x11, 0x15, 0x12, 0x0D],
        'R' => [0x1E, 0x11, 0x11, 0x1E, 0x14, 0x12, 0x11],
        'S' => [0x0F, 0x10, 0x10, 0x0E, 0x01, 0x01, 0x1E],
        'T' => [0x1F, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04],
        'U' => [0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x0E],
        'V' => [0x11, 0x11, 0x11, 0x11, 0x11, 0x0A, 0x04],
        'W' => [0x11, 0x11, 0x11, 0x15, 0x15, 0x15, 0x0A],
        'X' => [0x11, 0x11, 0x0A, 0x04, 0x0A, 0x11, 0x11],
        'Y' => [0x11, 0x11, 0x0A, 0x04, 0x04, 0x04, 0x04],
        'Z' => [0x1F, 0x01, 0x02, 0x04, 0x08, 0x10, 0x1F],
        '0' => [0x0E, 0x11, 0x13, 0x15, 0x19, 0x11, 0x0E],
        '1' => [0x04, 0x0C, 0x14, 0x04, 0x04, 0x04, 0x1F],
        '2' => [0x0E, 0x11, 0x01, 0x02, 0x04, 0x08, 0x1F],
        '3' => [0x1E, 0x01, 0x01, 0x0E, 0x01, 0x01, 0x1E],
        '4' => [0x02, 0x06, 0x0A, 0x12, 0x1F, 0x02, 0x02],
        '5' => [0x1F, 0x10, 0x10, 0x1E, 0x01, 0x01, 0x1E],
        '6' => [0x06, 0x08, 0x10, 0x1E, 0x11, 0x11, 0x0E],
        '7' => [0x1F, 0x01, 0x02, 0x04, 0x08, 0x08, 0x08],
        '8' => [0x0E, 0x11, 0x11, 0x0E, 0x11, 0x11, 0x0E],
        '9' => [0x0E, 0x11, 0x11, 0x0F, 0x01, 0x02, 0x1C],
        ':' => [0x00, 0x04, 0x04, 0x00, 0x04, 0x04, 0x00],
        '/' => [0x01, 0x02, 0x02, 0x04, 0x08, 0x08, 0x10],
        '-' => [0x00, 0x00, 0x00, 0x1F, 0x00, 0x00, 0x00],
        '_' => [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x1F],
        '.' => [0x00, 0x00, 0x00, 0x00, 0x00, 0x0C, 0x0C],
        '(' => [0x02, 0x04, 0x08, 0x08, 0x08, 0x04, 0x02],
        ')' => [0x08, 0x04, 0x02, 0x02, 0x02, 0x04, 0x08],
        '[' => [0x0E, 0x08, 0x08, 0x08, 0x08, 0x08, 0x0E],
        ']' => [0x0E, 0x02, 0x02, 0x02, 0x02, 0x02, 0x0E],
        '+' => [0x00, 0x04, 0x04, 0x1F, 0x04, 0x04, 0x00],
        '=' => [0x00, 0x1F, 0x00, 0x1F, 0x00, 0x00, 0x00],
        ' ' => [0x00; 7],
        _ => [0x1F, 0x01, 0x02, 0x04, 0x00, 0x04, 0x00],
    }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NoteRect {
    pub rect: Rect,
    pub clipped: bool,
}

pub fn note_rects(
    lane: Rect,
    notes: &[crate::project::MidiNote],
    range: crate::timeline::LoopRegion,
    flow: TimelineFlow,
) -> Vec<NoteRect> {
    let mut rects = Vec::new();

    for note in notes.iter().copied().filter(|note| note.intersects(range)) {
        let note_start = note.start_ticks.max(range.start_ticks);
        let note_end = note.end_ticks().min(range.end_ticks());
        let clipped = note_start != note.start_ticks || note_end != note.end_ticks();

        let rect = match flow {
            TimelineFlow::DownwardColumns => {
                vertical_note_rect(lane, note, note_start, note_end, range)
            }
            TimelineFlow::AcrossRows => {
                horizontal_note_rect(lane, note, note_start, note_end, range)
            }
        };

        rects.push(NoteRect { rect, clipped });
    }

    rects
}

pub fn track_header_rect(lane: Rect, flow: TimelineFlow) -> Rect {
    match flow {
        TimelineFlow::DownwardColumns => Rect::new(lane.x, lane.y, lane.width(), 34),
        TimelineFlow::AcrossRows => Rect::new(lane.x, lane.y, 56, lane.height()),
    }
}

pub fn track_status_rect(lane: Rect, flow: TimelineFlow) -> Rect {
    match flow {
        TimelineFlow::DownwardColumns => Rect::new(lane.x, lane.y, lane.width(), 14),
        TimelineFlow::AcrossRows => Rect::new(lane.x, lane.y, 56, 14),
    }
}

pub fn track_label_rect(lane: Rect, flow: TimelineFlow) -> Rect {
    match flow {
        TimelineFlow::DownwardColumns => Rect::new(lane.x, lane.y + 14, lane.width(), 20),
        TimelineFlow::AcrossRows => Rect::new(lane.x, lane.y + 14, 56, lane.height().saturating_sub(14)),
    }
}

pub fn track_content_rect(lane: Rect, flow: TimelineFlow) -> Rect {
    match flow {
        TimelineFlow::DownwardColumns => Rect::new(
            lane.x,
            lane.y + 34,
            lane.width(),
            lane.height().saturating_sub(34),
        ),
        TimelineFlow::AcrossRows => Rect::new(
            lane.x + 56,
            lane.y,
            lane.width().saturating_sub(56),
            lane.height(),
        ),
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

pub fn range_highlight_rect(
    bounds: Rect,
    flow: TimelineFlow,
    view_start_ticks: u64,
    view_length_ticks: u64,
    range: crate::timeline::LoopRegion,
) -> Rect {
    let view_length_ticks = view_length_ticks.max(1);
    let start_ratio =
        range.start_ticks.saturating_sub(view_start_ticks) as f32 / view_length_ticks as f32;
    let end_ratio =
        range.end_ticks().saturating_sub(view_start_ticks) as f32 / view_length_ticks as f32;

    match flow {
        TimelineFlow::DownwardColumns => {
            let y = bounds.y + (bounds.height() as f32 * start_ratio.clamp(0.0, 1.0)) as i32;
            let end_y = bounds.y + (bounds.height() as f32 * end_ratio.clamp(0.0, 1.0)) as i32;
            Rect::new(bounds.x, y, bounds.width(), (end_y - y).max(2) as u32)
        }
        TimelineFlow::AcrossRows => {
            let x = bounds.x + (bounds.width() as f32 * start_ratio.clamp(0.0, 1.0)) as i32;
            let end_x = bounds.x + (bounds.width() as f32 * end_ratio.clamp(0.0, 1.0)) as i32;
            Rect::new(x, bounds.y, (end_x - x).max(2) as u32, bounds.height())
        }
    }
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

fn vertical_note_rect(
    lane: Rect,
    note: crate::project::MidiNote,
    note_start: u64,
    note_end: u64,
    range: crate::timeline::LoopRegion,
) -> Rect {
    let pitch_min = 36_i32;
    let pitch_span = 60_i32;
    let lane_inner_x = lane.x + 8;
    let lane_inner_width = (lane.width() as i32 - 16).max(8);
    let note_x_ratio =
        ((i32::from(note.pitch) - pitch_min).clamp(0, pitch_span - 1)) as f32 / pitch_span as f32;
    let note_width = (lane_inner_width / 10).max(4);
    let x = lane_inner_x + ((lane_inner_width - note_width) as f32 * note_x_ratio) as i32;

    let start_ratio =
        note_start.saturating_sub(range.start_ticks) as f32 / range.length_ticks.max(1) as f32;
    let end_ratio =
        note_end.saturating_sub(range.start_ticks) as f32 / range.length_ticks.max(1) as f32;
    let y = lane.y + (lane.height() as f32 * start_ratio) as i32;
    let height = ((lane.height() as f32 * (end_ratio - start_ratio)).max(4.0)) as u32;

    Rect::new(x, y, note_width as u32, height)
}

fn horizontal_note_rect(
    lane: Rect,
    note: crate::project::MidiNote,
    note_start: u64,
    note_end: u64,
    range: crate::timeline::LoopRegion,
) -> Rect {
    let pitch_min = 36_i32;
    let pitch_span = 60_i32;
    let lane_inner_y = lane.y + 8;
    let lane_inner_height = (lane.height() as i32 - 16).max(8);
    let note_y_ratio =
        ((i32::from(note.pitch) - pitch_min).clamp(0, pitch_span - 1)) as f32 / pitch_span as f32;
    let note_height = (lane_inner_height / 10).max(4);
    let y = lane_inner_y + ((lane_inner_height - note_height) as f32 * note_y_ratio) as i32;

    let start_ratio =
        note_start.saturating_sub(range.start_ticks) as f32 / range.length_ticks.max(1) as f32;
    let end_ratio =
        note_end.saturating_sub(range.start_ticks) as f32 / range.length_ticks.max(1) as f32;
    let x = lane.x + (lane.width() as f32 * start_ratio) as i32;
    let width = ((lane.width() as f32 * (end_ratio - start_ratio)).max(4.0)) as u32;

    Rect::new(x, y, width, note_height as u32)
}

#[cfg(test)]
mod tests {
    use super::{
        HeaderBadgeKind, TimelineFlow, detail_badge_rect, equal_columns, header_badges,
        note_rects, passthrough_rail_rect, playhead_rect_in_range, range_highlight_rect,
        split_top_strip, stacked_rows, surface_rect, text_width, timeline_guides,
        track_column_pairs, track_content_rect, track_header_rect, track_label_rect,
        track_status_rect, truncate_text_to_width,
    };
    use crate::project::MidiNote;
    use crate::timeline::LoopRegion;
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
    fn split_top_strip_preserves_remaining_content() {
        let (top, rest) = split_top_strip(Rect::new(0, 0, 320, 200), 24, 8).unwrap();

        assert_eq!(top.height(), 24);
        assert_eq!(rest.y, 32);
        assert_eq!(rest.height(), 168);
    }

    #[test]
    fn equal_columns_splits_bounds_evenly() {
        let columns = equal_columns(Rect::new(0, 0, 300, 100), 3, 6);

        assert_eq!(columns.len(), 3);
        assert!(columns[1].x > columns[0].x);
    }

    #[test]
    fn stacked_rows_splits_bounds_evenly() {
        let rows = stacked_rows(Rect::new(0, 0, 100, 240), 4, 4);

        assert_eq!(rows.len(), 4);
        assert!(rows[1].y > rows[0].y);
    }

    #[test]
    fn truncate_text_to_width_adds_ellipsis_when_needed() {
        let full = "MICROSOFT GS WAVETABLE SYNTH";
        let truncated = truncate_text_to_width(full, text_width("MICROSOFT...", 1), 1);

        assert!(truncated.ends_with("..."));
        assert!(text_width(&truncated, 1) <= text_width("MICROSOFT...", 1));
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

        assert_eq!(header.height(), 34);
        assert_eq!(header.width(), 80);
    }

    #[test]
    fn track_chrome_reserves_status_and_content_bands() {
        let lane = Rect::new(10, 20, 80, 240);
        let status = track_status_rect(lane, TimelineFlow::DownwardColumns);
        let label = track_label_rect(lane, TimelineFlow::DownwardColumns);
        let content = track_content_rect(lane, TimelineFlow::DownwardColumns);

        assert_eq!(status.height(), 14);
        assert_eq!(label.y, 34);
        assert_eq!(content.y, 54);
        assert_eq!(content.height(), 206);
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

    #[test]
    fn note_rects_use_pitch_for_horizontal_position_in_vertical_view() {
        let low = MidiNote::new(40, 0, 240, 100);
        let high = MidiNote::new(80, 0, 240, 100);
        let rects = note_rects(
            Rect::new(10, 10, 80, 240),
            &[low, high],
            LoopRegion::new(0, 960),
            TimelineFlow::DownwardColumns,
        );

        assert!(rects[1].rect.x > rects[0].rect.x);
    }

    #[test]
    fn note_rects_clip_to_loop_range() {
        let note = MidiNote::new(64, 0, 960, 100);
        let rects = note_rects(
            Rect::new(10, 10, 80, 240),
            &[note],
            LoopRegion::new(240, 240),
            TimelineFlow::DownwardColumns,
        );

        assert_eq!(rects.len(), 1);
        assert!(rects[0].clipped);
    }

    #[test]
    fn range_highlight_follows_time_axis() {
        let vertical = range_highlight_rect(
            Rect::new(0, 0, 80, 400),
            TimelineFlow::DownwardColumns,
            0,
            1600,
            LoopRegion::new(400, 400),
        );
        let horizontal = range_highlight_rect(
            Rect::new(0, 0, 80, 400),
            TimelineFlow::AcrossRows,
            0,
            1600,
            LoopRegion::new(400, 400),
        );

        assert_eq!(vertical.width(), 80);
        assert_eq!(horizontal.height(), 400);
    }
}
