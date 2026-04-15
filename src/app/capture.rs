use crate::app::AppOverlay;
use crate::pages::AppPage;
use crate::project::{MidiNote, RecordingClip, RecordingView, STORED_LOOP_SLOT_COUNT, Track};
use crate::timeline::Region;
use sdl3::pixels::{Color, PixelFormat};
use sdl3::rect::Rect;
use sdl3::render::{Canvas, RenderTarget};

use super::types::CaptureSpec;

pub(super) fn capture_specs() -> [CaptureSpec; 7] {
    [
        CaptureSpec {
            page: AppPage::Timeline,
            overlay: None,
            focused_track_view: false,
            open_clip_align: false,
            filename: "timeline.png",
        },
        CaptureSpec {
            page: AppPage::Timeline,
            overlay: None,
            focused_track_view: true,
            open_clip_align: false,
            filename: "timeline-focused.png",
        },
        CaptureSpec {
            page: AppPage::Timeline,
            overlay: None,
            focused_track_view: true,
            open_clip_align: true,
            filename: "timeline-clip-align.png",
        },
        CaptureSpec {
            page: AppPage::Mappings,
            overlay: None,
            focused_track_view: false,
            open_clip_align: false,
            filename: "mappings.png",
        },
        CaptureSpec {
            page: AppPage::Mappings,
            overlay: Some(AppOverlay::MappingsQuickView),
            focused_track_view: false,
            open_clip_align: false,
            filename: "mappings-overlay.png",
        },
        CaptureSpec {
            page: AppPage::MidiIo,
            overlay: None,
            focused_track_view: false,
            open_clip_align: false,
            filename: "midi-io.png",
        },
        CaptureSpec {
            page: AppPage::Routing,
            overlay: None,
            focused_track_view: false,
            open_clip_align: false,
            filename: "routing.png",
        },
    ]
}

#[derive(Debug, Clone)]
pub(super) struct RgbaReadback {
    logical_rect: Rect,
    output_rect: Rect,
    scale_x: f32,
    scale_y: f32,
    pitch: usize,
    pixels: Vec<u8>,
}

pub(super) fn readback_rect_rgba<T: RenderTarget>(
    canvas: &Canvas<T>,
    logical_rect: Rect,
    logical_viewport_size: (u32, u32),
) -> Option<RgbaReadback> {
    if logical_rect.width() == 0 || logical_rect.height() == 0 {
        return None;
    }
    let output_size = canvas.output_size().ok()?;
    let scale_x = if logical_viewport_size.0 > 0 {
        output_size.0 as f32 / logical_viewport_size.0 as f32
    } else {
        1.0
    };
    let scale_y = if logical_viewport_size.1 > 0 {
        output_size.1 as f32 / logical_viewport_size.1 as f32
    } else {
        1.0
    };
    let sx = scale_x.max(0.0001);
    let sy = scale_y.max(0.0001);
    let ox = (logical_rect.x as f32 * sx).floor() as i32;
    let oy = (logical_rect.y as f32 * sy).floor() as i32;
    let ow = (logical_rect.width() as f32 * sx).ceil().max(1.0) as u32;
    let oh = (logical_rect.height() as f32 * sy).ceil().max(1.0) as u32;
    let output_rect = Rect::new(ox, oy, ow, oh);
    let surface = canvas.read_pixels(output_rect).ok()?;
    let converted = surface.convert_format(PixelFormat::RGBA32).ok()?;
    let pitch = converted.pitch() as usize;
    let pixels = converted.with_lock(|src| src.to_vec());
    Some(RgbaReadback {
        logical_rect,
        output_rect,
        scale_x,
        scale_y,
        pitch,
        pixels,
    })
}

pub(super) fn readback_color_at(readback: &Option<RgbaReadback>, x: i32, y: i32) -> Option<Color> {
    let readback = readback.as_ref()?;
    if x < readback.logical_rect.x
        || y < readback.logical_rect.y
        || x >= readback.logical_rect.x + readback.logical_rect.width() as i32
        || y >= readback.logical_rect.y + readback.logical_rect.height() as i32
    {
        return None;
    }
    let local_logical_x = x - readback.logical_rect.x;
    let local_logical_y = y - readback.logical_rect.y;
    let local_x = (local_logical_x as f32 * readback.scale_x).floor() as usize;
    let local_y = (local_logical_y as f32 * readback.scale_y).floor() as usize;
    if local_x >= readback.output_rect.width() as usize
        || local_y >= readback.output_rect.height() as usize
    {
        return None;
    }
    let base = local_y
        .saturating_mul(readback.pitch)
        .saturating_add(local_x.saturating_mul(4));
    if base + 3 >= readback.pixels.len() {
        return None;
    }
    Some(Color::RGBA(
        readback.pixels[base],
        readback.pixels[base + 1],
        readback.pixels[base + 2],
        readback.pixels[base + 3],
    ))
}

pub(super) fn seed_capture_demo_track(track: &mut Track, track_index: usize) {
    let overlaps = [
        crate::timeline::LoopRegion::new(0, 3_840),
        crate::timeline::LoopRegion::new(480, 3_360),
        crate::timeline::LoopRegion::new(960, 2_880),
        crate::timeline::LoopRegion::new(1_440, 2_400),
        crate::timeline::LoopRegion::new(1_920, 1_920),
        crate::timeline::LoopRegion::new(2_400, 1_440),
        crate::timeline::LoopRegion::new(2_880, 960),
        crate::timeline::LoopRegion::new(3_360, 960),
    ];

    let recording_clip_id = 1;
    let clip_start_ticks = 240;
    let clip_length_ticks = 4_320;
    track.midi_notes = dense_capture_notes(track_index)
        .into_iter()
        .map(|note| {
            if note.start_ticks >= clip_start_ticks
                && note.start_ticks < clip_start_ticks + clip_length_ticks
            {
                MidiNote::new_recorded(
                    note.pitch,
                    note.start_ticks,
                    note.length_ticks,
                    note.velocity,
                    recording_clip_id,
                )
            } else {
                note
            }
        })
        .collect();
    let clip_region = Region::new_recorded(clip_start_ticks, clip_length_ticks, recording_clip_id);
    track.regions = vec![clip_region];
    track.recording_clips = vec![RecordingClip {
        id: recording_clip_id,
        region: clip_region,
        muted: false,
        native_start_ticks: clip_start_ticks,
        native_end_ticks: clip_start_ticks + clip_length_ticks,
        native_duration_ticks: clip_length_ticks,
        native_capture_tempo_bpm: 96,
    }];
    track.selected_recording_clip_id = Some(recording_clip_id);
    track.next_recording_clip_id = recording_clip_id + 1;
    track.recording_view = RecordingView::Stacked;
    track.loop_region = overlaps[0];
    track.state.loop_enabled = true;

    track.stored_loops = vec![None; STORED_LOOP_SLOT_COUNT];
    track.active_stored_loop_slot = None;
    for (slot_index, range) in overlaps.iter().copied().enumerate() {
        track.loop_region = range;
        track.store_current_loop_to_slot(slot_index);
    }
    track.recall_stored_loop_slot(2);
}

fn dense_capture_notes(track_index: usize) -> Vec<MidiNote> {
    let mut notes = Vec::with_capacity(80);
    let base_pitch = 42_u8.saturating_add((track_index as u8).saturating_mul(2));
    for step in 0..40_u64 {
        let start = step * 120;
        let primary_pitch = base_pitch.saturating_add((step % 12) as u8);
        let secondary_pitch = primary_pitch.saturating_add(7);
        let velocity = 72_u8.saturating_add(((step * 9) % 44) as u8);
        notes.push(MidiNote::new(primary_pitch, start, 180, velocity));
        if step % 2 == 0 {
            notes.push(MidiNote::new(
                secondary_pitch,
                start + 60,
                120,
                velocity.saturating_sub(10),
            ));
        }
    }
    notes
}
