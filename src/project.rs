use crate::routing::TrackRouting;
use crate::timeline::{LoopRegion, RecordingTake, Region};
use crate::transport::{RecordMode, Transport};

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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

    pub fn begin_recording(&mut self, pressed_at: u64) {
        self.active_take = Some(RecordingTake::new(pressed_at));
    }

    /// V1 is MIDI-first, so record commit currently appends a MIDI region.
    pub fn finish_recording(&mut self, transport: Transport, released_at: u64) {
        let Some(take) = self.active_take.take() else {
            return;
        };

        self.commit_take(transport, take.release(released_at));
    }

    pub fn commit_take(&mut self, transport: Transport, take: RecordingTake) {
        let Some(released_at) = take.released_at_ticks else {
            return;
        };

        let start_ticks = transport.quantize_to_nearest(take.pressed_at_ticks);
        let end_ticks = transport.quantize_to_nearest(released_at);
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
        self.append_demo_recorded_notes(start_ticks, length_ticks);
    }

    pub fn preview_region(&self, transport: Transport, current_ticks: u64) -> Option<Region> {
        self.active_take.and_then(|take| {
            take.preview_region(current_ticks, |ticks| transport.quantize_to_nearest(ticks))
        })
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

    fn append_demo_recorded_notes(&mut self, start_ticks: u64, length_ticks: u64) {
        let step = (length_ticks / 4).max(120);
        let note_len = (step / 2).max(60);
        let base_pitch = 60 + ((self.name.len() as u8) % 12);

        for index in 0..4 {
            let note_start = start_ticks + step * index;
            if note_start >= start_ticks + length_ticks {
                break;
            }
            self.midi_notes.push(MidiNote::new(
                base_pitch.saturating_add((index as u8) * 2),
                note_start,
                note_len.min(length_ticks),
                104,
            ));
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrackKind {
    Midi,
    Audio,
    Hybrid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TrackState {
    pub armed: bool,
    pub loop_enabled: bool,
    pub muted: bool,
    pub soloed: bool,
    pub passthrough: bool,
}

#[cfg(test)]
mod tests {
    use super::{MidiNote, Project, Track, TrackKind};
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
        let mut track = Track::new("Track 1", TrackKind::Midi);

        track.commit_take(transport, RecordingTake::new(220).release(721));

        assert_eq!(track.regions, vec![Region::new(240, 480)]);
    }

    #[test]
    fn finish_recording_consumes_active_take() {
        let transport = Transport::default();
        let mut track = Track::new("Track 1", TrackKind::Midi);
        track.begin_recording(960);

        track.finish_recording(transport, 1_920);

        assert!(track.active_take.is_none());
        assert_eq!(track.regions, vec![Region::new(960, 960)]);
    }

    #[test]
    fn preview_region_tracks_active_take_span() {
        let transport = Transport {
            quantize: QuantizeMode::Quarter,
            ..Transport::default()
        };
        let mut track = Track::new("Track 1", TrackKind::Midi);
        track.begin_recording(110);

        assert_eq!(
            track.preview_region(transport, 1_120),
            Some(Region::new(0, 960))
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

        track.commit_take(transport, RecordingTake::new(0).release(1_000));

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
        assert!(track.midi_notes.iter().any(|note| note.start_ticks < 960));
        assert!(
            !track
                .midi_notes
                .iter()
                .any(|note| note.pitch == 60 && note.start_ticks == 0)
        );
    }
}
