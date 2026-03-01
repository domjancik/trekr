use crate::routing::TrackRouting;
use crate::timeline::{LoopRegion, RecordedMidiNote, RecordingTake, Region};
use crate::transport::{RecordMode, Transport};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecordContext {
    pub range: LoopRegion,
    pub wrap_basis_ticks: u64,
    pub extend_clip_on_wrap: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    pub transport: Transport,
    pub loop_region: LoopRegion,
    pub active_track_index: usize,
    pub tracks: Vec<Track>,
}

impl Project {
    pub fn demo() -> Self {
        Self {
            name: "Untitled".to_string(),
            transport: Transport::default(),
            loop_region: LoopRegion::new(0, 16 * 960),
            active_track_index: 0,
            tracks: (1..=6)
                .map(|index| Track::new(&format!("Track {}", index), TrackKind::Midi))
                .collect(),
        }
    }

    pub fn empty() -> Self {
        Self {
            name: "Untitled".to_string(),
            transport: Transport::default(),
            loop_region: LoopRegion::new(0, 16 * 960),
            active_track_index: 0,
            tracks: (1..=6)
                .map(|index| Track::new_empty(&format!("Track {}", index), TrackKind::Midi))
                .collect(),
        }
    }

    pub fn full_song_range(&self) -> LoopRegion {
        let end_ticks = self
            .tracks
            .iter()
            .map(Track::content_end_ticks)
            .max()
            .unwrap_or(self.loop_region.end_ticks());

        LoopRegion::new(0, end_ticks.max(self.loop_region.end_ticks()))
    }

    pub fn select_track(&mut self, index: usize) {
        if self.tracks.is_empty() {
            self.active_track_index = 0;
            return;
        }

        self.active_track_index = index.min(self.tracks.len() - 1);
    }

    pub fn select_next_track(&mut self) {
        if self.tracks.is_empty() {
            return;
        }

        self.active_track_index = (self.active_track_index + 1) % self.tracks.len();
    }

    pub fn select_previous_track(&mut self) {
        if self.tracks.is_empty() {
            return;
        }

        let track_count = self.tracks.len();
        self.active_track_index = (self.active_track_index + track_count - 1) % track_count;
    }

    pub fn active_track(&self) -> Option<&Track> {
        self.tracks.get(self.active_track_index)
    }

    pub fn active_track_mut(&mut self) -> Option<&mut Track> {
        self.tracks.get_mut(self.active_track_index)
    }

    pub fn clear_all_track_content(&mut self) {
        for track in &mut self.tracks {
            track.clear_content();
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Track {
    pub name: String,
    pub kind: TrackKind,
    pub state: TrackState,
    pub routing: TrackRouting,
    pub loop_region: LoopRegion,
    pub active_take: Option<RecordingTake>,
    pub midi_notes: Vec<MidiNote>,
    pub regions: Vec<Region>,
}

impl Track {
    pub fn new(name: &str, kind: TrackKind) -> Self {
        let mut track = Self {
            name: name.to_string(),
            kind,
            state: TrackState::default(),
            routing: TrackRouting::default(),
            loop_region: LoopRegion::new(0, 4 * 960),
            active_take: None,
            midi_notes: Vec::new(),
            regions: Vec::new(),
        };
        track.seed_demo_notes();
        track
    }

    pub fn new_empty(name: &str, kind: TrackKind) -> Self {
        Self {
            name: name.to_string(),
            kind,
            state: TrackState::default(),
            routing: TrackRouting::default(),
            loop_region: LoopRegion::new(0, 4 * 960),
            active_take: None,
            midi_notes: Vec::new(),
            regions: Vec::new(),
        }
    }

    pub fn begin_recording(&mut self, pressed_at: u64) {
        self.active_take = Some(RecordingTake::new(pressed_at));
    }

    pub fn record_note_on(&mut self, pitch: u8, velocity: u8, started_at: u64) {
        if let Some(take) = self.active_take.as_mut() {
            take.note_on(pitch, velocity, started_at);
        }
    }

    pub fn record_note_off(&mut self, pitch: u8, ended_at: u64) {
        if let Some(take) = self.active_take.as_mut() {
            take.note_off(pitch, ended_at);
        }
    }

    /// V1 is MIDI-first, so record commit currently appends a MIDI region.
    pub fn finish_recording(
        &mut self,
        transport: Transport,
        released_at: u64,
        record_context: Option<RecordContext>,
    ) {
        let Some(take) = self.active_take.take() else {
            return;
        };

        self.commit_take(transport, take.release(released_at), record_context);
    }

    pub fn commit_take(
        &mut self,
        transport: Transport,
        take: RecordingTake,
        record_context: Option<RecordContext>,
    ) {
        let Some(released_at) = take.released_at_ticks else {
            return;
        };

        let (start_ticks, end_ticks) = normalized_region_span(
            transport,
            take.pressed_at_ticks,
            released_at,
            record_context,
            take.pressed_at_ticks,
            released_at,
        );
        let length_ticks = end_ticks.saturating_sub(start_ticks);

        if length_ticks == 0 {
            return;
        }

        let committed_region = Region::new(start_ticks, length_ticks);
        if transport.record_mode == RecordMode::Replace {
            self.remove_content_in_range(LoopRegion::new(
                committed_region.start_ticks,
                committed_region.length_ticks,
            ));
        }

        self.regions.push(committed_region);
        self.append_recorded_notes(
            transport,
            &take.recorded_notes,
            record_context,
            take.pressed_at_ticks,
            released_at,
            start_ticks,
            end_ticks,
        );
    }

    pub fn preview_region(
        &self,
        transport: Transport,
        current_ticks: u64,
        record_context: Option<RecordContext>,
    ) -> Option<Region> {
        self.active_take.as_ref().and_then(|take| {
            let (start_ticks, end_ticks) = normalized_region_span(
                transport,
                take.pressed_at_ticks,
                current_ticks,
                record_context,
                take.pressed_at_ticks,
                current_ticks,
            );
            let length_ticks = end_ticks.saturating_sub(start_ticks);
            (length_ticks > 0).then_some(Region::new(start_ticks, length_ticks))
        })
    }

    pub fn preview_notes(
        &self,
        transport: Transport,
        current_ticks: u64,
        record_context: Option<RecordContext>,
    ) -> Vec<MidiNote> {
        let Some(take) = self.active_take.as_ref() else {
            return Vec::new();
        };

        let mut notes = Vec::new();
        for recorded_note in &take.recorded_notes {
            if let Some(note) = preview_midi_note(
                transport,
                *recorded_note,
                record_context,
                take.pressed_at_ticks,
                current_ticks,
                current_ticks,
            ) {
                notes.push(note);
            }
        }

        for pending_note in &take.pending_notes {
            let recorded_note = RecordedMidiNote {
                pitch: pending_note.pitch,
                velocity: pending_note.velocity,
                started_at_ticks: pending_note.started_at_ticks,
                ended_at_ticks: current_ticks.max(pending_note.started_at_ticks),
            };
            if let Some(note) = preview_midi_note(
                transport,
                recorded_note,
                record_context,
                take.pressed_at_ticks,
                current_ticks,
                current_ticks,
            ) {
                notes.push(note);
            }
        }

        notes
    }

    pub fn content_end_ticks(&self) -> u64 {
        let notes_end = self
            .midi_notes
            .iter()
            .map(|note| note.end_ticks())
            .max()
            .unwrap_or(0);
        let regions_end = self
            .regions
            .iter()
            .map(|region| region.start_ticks + region.length_ticks)
            .max()
            .unwrap_or(0);

        notes_end.max(regions_end).max(self.loop_region.end_ticks())
    }

    fn seed_demo_notes(&mut self) {
        let base_pitch = 48 + ((self.name.len() as u8) % 12);
        let motif = [0_u8, 4, 7, 11, 7, 4, 2, 5];

        self.midi_notes = motif
            .iter()
            .enumerate()
            .map(|(index, interval)| {
                let step_ticks = 480_u64;
                let start_ticks = index as u64 * step_ticks;
                let duration = if index % 3 == 0 { 360 } else { 240 };
                MidiNote::new(
                    base_pitch.saturating_add(*interval),
                    start_ticks,
                    duration,
                    96_u8.saturating_sub(index as u8 * 4),
                )
            })
            .collect();
    }

    pub fn clear_content(&mut self) {
        self.active_take = None;
        self.midi_notes.clear();
        self.regions.clear();
    }

    fn remove_content_in_range(&mut self, range: LoopRegion) {
        self.midi_notes.retain(|note| !note.intersects(range));
        self.regions.retain(|region| !region.intersects(range));
    }

    fn append_recorded_notes(
        &mut self,
        transport: Transport,
        recorded_notes: &[RecordedMidiNote],
        record_context: Option<RecordContext>,
        take_pressed_at_ticks: u64,
        take_released_at_ticks: u64,
        start_ticks: u64,
        end_ticks: u64,
    ) {
        for recorded_note in recorded_notes {
            let (note_start, note_end) = normalized_note_span(
                transport,
                recorded_note.started_at_ticks,
                recorded_note.ended_at_ticks,
                record_context,
                take_pressed_at_ticks,
                take_released_at_ticks,
            );
            let note_start = note_start.clamp(start_ticks, end_ticks.saturating_sub(1));
            let note_end = note_end.clamp(note_start.saturating_add(1), end_ticks);
            let note_length = note_end.saturating_sub(note_start);
            if note_length == 0 {
                continue;
            }
            self.midi_notes.push(MidiNote::new(
                recorded_note.pitch,
                note_start,
                note_length,
                recorded_note.velocity,
            ));
        }
    }
}

fn normalized_region_span(
    transport: Transport,
    pressed_at_ticks: u64,
    released_at_ticks: u64,
    record_context: Option<RecordContext>,
    take_pressed_at_ticks: u64,
    take_end_ticks: u64,
) -> (u64, u64) {
    let start_ticks = transport.quantize_to_nearest(pressed_at_ticks);
    let end_ticks = transport.quantize_to_nearest(released_at_ticks.max(pressed_at_ticks));

    let Some(record_context) = record_context else {
        return (start_ticks, end_ticks);
    };

    let range_start = record_context.range.start_ticks;
    let range_end = record_context.range.end_ticks();
    let wrapped = loop_cycle(
        transport.quantize_to_nearest(take_end_ticks.max(take_pressed_at_ticks)),
        record_context,
    ) > loop_cycle(take_pressed_at_ticks, record_context);

    if record_context.extend_clip_on_wrap && wrapped {
        return (range_start, range_end);
    }

    let projected_start = projected_loop_ticks(start_ticks, record_context);
    let projected_end = projected_loop_ticks(end_ticks, record_context);
    let start_ticks = projected_start.clamp(range_start, range_end.saturating_sub(1));
    let mut end_ticks = projected_end.clamp(range_start, range_end);
    if wrapped || end_ticks < start_ticks {
        end_ticks = range_end;
    }

    (start_ticks, end_ticks)
}

fn normalized_note_span(
    transport: Transport,
    pressed_at_ticks: u64,
    released_at_ticks: u64,
    record_context: Option<RecordContext>,
    take_pressed_at_ticks: u64,
    take_end_ticks: u64,
) -> (u64, u64) {
    let start_ticks = transport.quantize_to_nearest(pressed_at_ticks);
    let end_ticks = transport.quantize_to_nearest(released_at_ticks.max(pressed_at_ticks));

    let Some(record_context) = record_context else {
        return (start_ticks, end_ticks);
    };

    let range_start = record_context.range.start_ticks;
    let range_end = record_context.range.end_ticks();
    let projected_start = projected_loop_ticks(start_ticks, record_context);
    let projected_end = projected_loop_ticks(end_ticks, record_context);
    let take_wrapped = loop_cycle(
        transport.quantize_to_nearest(take_end_ticks.max(take_pressed_at_ticks)),
        record_context,
    ) > loop_cycle(take_pressed_at_ticks, record_context);

    if record_context.extend_clip_on_wrap && take_wrapped {
        let start_ticks = projected_start.clamp(range_start, range_end.saturating_sub(1));
        let mut end_ticks = projected_end.clamp(range_start, range_end);
        if end_ticks < start_ticks {
            end_ticks = range_end;
        }
        return (start_ticks, end_ticks);
    }

    let start_ticks = projected_start.clamp(range_start, range_end.saturating_sub(1));
    let mut end_ticks = projected_end.clamp(range_start, range_end);
    if take_wrapped || end_ticks < start_ticks {
        end_ticks = range_end;
    }

    (start_ticks, end_ticks)
}

fn preview_midi_note(
    transport: Transport,
    recorded_note: RecordedMidiNote,
    record_context: Option<RecordContext>,
    take_pressed_at_ticks: u64,
    take_end_ticks: u64,
    current_ticks: u64,
) -> Option<MidiNote> {
    let (note_start, note_end) = normalized_note_span(
        transport,
        recorded_note.started_at_ticks,
        recorded_note
            .ended_at_ticks
            .min(current_ticks.max(recorded_note.started_at_ticks)),
        record_context,
        take_pressed_at_ticks,
        take_end_ticks,
    );
    let note_length = note_end.saturating_sub(note_start);
    (note_length > 0).then_some(MidiNote::new(
        recorded_note.pitch,
        note_start,
        note_length,
        recorded_note.velocity,
    ))
}

fn loop_cycle(ticks: u64, record_context: RecordContext) -> u64 {
    if record_context.range.length_ticks == 0 {
        return 0;
    }

    ticks.saturating_sub(record_context.wrap_basis_ticks) / record_context.range.length_ticks
}

fn projected_loop_ticks(ticks: u64, record_context: RecordContext) -> u64 {
    if record_context.range.length_ticks == 0 {
        return record_context.range.start_ticks;
    }

    let relative = ticks.saturating_sub(record_context.wrap_basis_ticks);
    record_context.range.start_ticks + (relative % record_context.range.length_ticks)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrackKind {
    Midi,
    Audio,
    Hybrid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MidiNote {
    pub pitch: u8,
    pub start_ticks: u64,
    pub length_ticks: u64,
    pub velocity: u8,
}

impl MidiNote {
    pub fn new(pitch: u8, start_ticks: u64, length_ticks: u64, velocity: u8) -> Self {
        Self {
            pitch,
            start_ticks,
            length_ticks,
            velocity,
        }
    }

    pub fn end_ticks(self) -> u64 {
        self.start_ticks + self.length_ticks
    }

    pub fn intersects(self, range: LoopRegion) -> bool {
        self.start_ticks < range.end_ticks() && self.end_ticks() > range.start_ticks
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct TrackState {
    pub armed: bool,
    pub loop_enabled: bool,
    pub muted: bool,
    pub soloed: bool,
    pub passthrough: bool,
}

#[cfg(test)]
mod tests {
    use super::{MidiNote, Project, RecordContext, Track, TrackKind};
    use crate::timeline::{LoopRegion, RecordingTake, Region};
    use crate::transport::{QuantizeMode, RecordMode, Transport};

    #[test]
    fn full_song_range_uses_loop_region_when_no_regions_exist() {
        let project = Project::demo();

        assert_eq!(
            project.full_song_range().length_ticks,
            project.loop_region.length_ticks
        );
    }

    #[test]
    fn full_song_range_expands_to_cover_latest_region_end() {
        let mut project = Project::demo();
        project.tracks[0].regions.push(Region::new(0, 960));
        project.tracks[1].regions.push(Region::new(3_840, 1_920));

        assert_eq!(project.full_song_range().end_ticks(), 15_360);
    }

    #[test]
    fn commit_recording_quantizes_release_to_nearest_boundary() {
        let transport = Transport {
            quantize: QuantizeMode::Sixteenth,
            ..Transport::default()
        };
        let mut track = Track::new_empty("Track 1", TrackKind::Midi);

        track.commit_take(transport, RecordingTake::new(220).release(721), None);

        assert_eq!(track.regions, vec![Region::new(240, 480)]);
    }

    #[test]
    fn finish_recording_consumes_active_take() {
        let transport = Transport::default();
        let mut track = Track::new_empty("Track 1", TrackKind::Midi);
        track.begin_recording(960);

        track.finish_recording(transport, 1_920, None);

        assert!(track.active_take.is_none());
        assert_eq!(track.regions, vec![Region::new(960, 960)]);
    }

    #[test]
    fn preview_region_tracks_active_take_span() {
        let transport = Transport {
            quantize: QuantizeMode::Quarter,
            ..Transport::default()
        };
        let mut track = Track::new_empty("Track 1", TrackKind::Midi);
        track.begin_recording(110);

        assert_eq!(
            track.preview_region(transport, 1_120, None),
            Some(Region::new(0, 960))
        );
    }

    #[test]
    fn preview_notes_include_pending_recorded_note_shapes() {
        let transport = Transport {
            quantize: QuantizeMode::Off,
            ..Transport::default()
        };
        let mut track = Track::new("Track 1", TrackKind::Midi);
        track.begin_recording(100);
        track.record_note_on(64, 96, 120);

        assert_eq!(
            track.preview_notes(transport, 360, None),
            vec![MidiNote::new(64, 120, 240, 96)]
        );
    }

    #[test]
    fn track_selection_supports_relative_and_absolute_moves() {
        let mut project = Project::demo();
        assert_eq!(project.active_track_index, 0);

        project.select_track(3);
        assert_eq!(project.active_track_index, 3);

        project.select_next_track();
        assert_eq!(project.active_track_index, 4);

        project.select_previous_track();
        assert_eq!(project.active_track_index, 3);
    }

    #[test]
    fn full_song_range_includes_note_content() {
        let mut project = Project::demo();
        project.tracks[0].midi_notes = vec![MidiNote::new(60, 0, 960, 100)];
        project.tracks[1].midi_notes = vec![MidiNote::new(72, 7_680, 960, 100)];

        assert_eq!(project.full_song_range().end_ticks(), 15_360);
    }

    #[test]
    fn midi_note_reports_range_intersection() {
        let note = MidiNote::new(60, 960, 480, 100);
        assert!(note.intersects(LoopRegion::new(1_200, 960)));
        assert!(!note.intersects(LoopRegion::new(2_000, 120)));
    }

    #[test]
    fn clear_content_resets_take_notes_and_regions() {
        let mut track = Track::new("Track 1", TrackKind::Midi);
        track.begin_recording(100);
        track.regions.push(Region::new(0, 240));

        track.clear_content();

        assert!(track.active_take.is_none());
        assert!(track.midi_notes.is_empty());
        assert!(track.regions.is_empty());
    }

    #[test]
    fn replace_record_mode_removes_overlapping_content() {
        let transport = Transport {
            quantize: QuantizeMode::Quarter,
            record_mode: RecordMode::Replace,
            ..Transport::default()
        };
        let mut track = Track::new("Track 1", TrackKind::Midi);
        track.midi_notes = vec![
            MidiNote::new(60, 0, 960, 100),
            MidiNote::new(64, 1_920, 960, 100),
        ];
        track.regions = vec![Region::new(0, 960), Region::new(1_920, 960)];
        let mut take = RecordingTake::new(0);
        take.note_on(67, 110, 120);
        take.note_off(67, 480);

        track.commit_take(transport, take.release(1_000), None);

        assert_eq!(
            track.regions,
            vec![Region::new(1_920, 960), Region::new(0, 960)]
        );
        assert!(
            track
                .midi_notes
                .iter()
                .any(|note| note.pitch == 64 && note.start_ticks == 1_920)
        );
        assert!(
            !track
                .midi_notes
                .iter()
                .any(|note| note.pitch == 60 && note.start_ticks == 0)
        );
        assert!(
            track
                .midi_notes
                .iter()
                .any(|note| note.pitch == 67 && note.start_ticks == 0)
        );
    }

    #[test]
    fn loop_recording_clamps_wrapped_release_to_loop_end() {
        let transport = Transport {
            quantize: QuantizeMode::Off,
            ..Transport::default()
        };
        let mut track = Track::new("Track 1", TrackKind::Midi);
        let record_context = RecordContext {
            range: LoopRegion::new(960, 960),
            wrap_basis_ticks: 0,
            extend_clip_on_wrap: false,
        };

        track.commit_take(
            transport,
            RecordingTake::new(1_680).release(2_160),
            Some(record_context),
        );

        assert_eq!(track.regions.last().copied(), Some(Region::new(1_680, 240)));
    }

    #[test]
    fn loop_recording_extension_keeps_notes_in_loop_positions() {
        let transport = Transport {
            quantize: QuantizeMode::Off,
            ..Transport::default()
        };
        let mut track = Track::new_empty("Track 1", TrackKind::Midi);
        let record_context = RecordContext {
            range: LoopRegion::new(960, 960),
            wrap_basis_ticks: 0,
            extend_clip_on_wrap: true,
        };
        let mut take = RecordingTake::new(1_680);
        take.note_on(64, 100, 1_700);
        take.note_off(64, 1_820);
        take.note_on(67, 100, 2_040);
        take.note_off(67, 2_160);

        track.commit_take(transport, take.release(2_160), Some(record_context));

        assert_eq!(track.regions.last().copied(), Some(Region::new(960, 960)));
        assert_eq!(
            track.midi_notes,
            vec![
                MidiNote::new(64, 1_700, 120, 100),
                MidiNote::new(67, 1_080, 120, 100),
            ]
        );
    }
}
