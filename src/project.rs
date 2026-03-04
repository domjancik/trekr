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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum RecordingView {
    #[default]
    Overlay,
    Stacked,
}

impl RecordingView {
    pub fn toggle(self) -> Self {
        match self {
            Self::Overlay => Self::Stacked,
            Self::Stacked => Self::Overlay,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordingClip {
    pub id: u64,
    pub region: Region,
    #[serde(default)]
    pub muted: bool,
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
    #[serde(default)]
    pub recording_clips: Vec<RecordingClip>,
    #[serde(default)]
    pub recording_view: RecordingView,
    #[serde(default)]
    pub selected_recording_clip_id: Option<u64>,
    #[serde(default)]
    pub recording_clip_scroll: usize,
    #[serde(default = "default_next_recording_clip_id")]
    pub next_recording_clip_id: u64,
    #[serde(default)]
    pub note_selection: NoteSelection,
}

fn default_next_recording_clip_id() -> u64 {
    1
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
            recording_clips: Vec::new(),
            recording_view: RecordingView::default(),
            selected_recording_clip_id: None,
            recording_clip_scroll: 0,
            next_recording_clip_id: default_next_recording_clip_id(),
            note_selection: NoteSelection::default(),
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
            recording_clips: Vec::new(),
            recording_view: RecordingView::default(),
            selected_recording_clip_id: None,
            recording_clip_scroll: 0,
            next_recording_clip_id: default_next_recording_clip_id(),
            note_selection: NoteSelection::default(),
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

        let recording_clip_id = self.allocate_recording_clip_id();
        let committed_region = Region::new_recorded(start_ticks, length_ticks, recording_clip_id);
        if transport.record_mode == RecordMode::Replace {
            self.remove_content_in_range(LoopRegion::new(
                committed_region.start_ticks,
                committed_region.length_ticks,
            ));
        }

        self.recording_clips.push(RecordingClip {
            id: recording_clip_id,
            region: committed_region,
            muted: false,
        });
        self.selected_recording_clip_id = Some(recording_clip_id);
        self.regions.push(committed_region);
        self.append_recorded_notes(
            transport,
            &take.recorded_notes,
            record_context,
            take.pressed_at_ticks,
            released_at,
            start_ticks,
            end_ticks,
            recording_clip_id,
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

    pub fn recording_clips(&self) -> &[RecordingClip] {
        &self.recording_clips
    }

    pub fn recording_clip(&self, clip_id: u64) -> Option<&RecordingClip> {
        self.recording_clips.iter().find(|clip| clip.id == clip_id)
    }

    pub fn selected_recording_clip(&self) -> Option<&RecordingClip> {
        self.selected_recording_clip_id
            .and_then(|clip_id| self.recording_clip(clip_id))
    }

    pub fn toggle_recording_view(&mut self) {
        self.recording_view = self.recording_view.toggle();
        if self.recording_view == RecordingView::Stacked
            && self.selected_recording_clip_id.is_none()
        {
            self.selected_recording_clip_id = self.recording_clips.last().map(|clip| clip.id);
        }
        self.clear_note_selection();
    }

    pub fn select_recording_clip(&mut self, clip_id: u64) -> bool {
        if self.recording_clip(clip_id).is_none() {
            return false;
        }

        self.selected_recording_clip_id = Some(clip_id);
        self.clear_note_selection();
        true
    }

    pub fn select_next_recording_clip(&mut self) -> bool {
        if self.recording_clips.is_empty() {
            self.selected_recording_clip_id = None;
            return false;
        }

        let selected_index = self.selected_recording_clip_id.and_then(|clip_id| {
            self.recording_clips
                .iter()
                .position(|clip| clip.id == clip_id)
        });
        let next_index = selected_index
            .map(|index| (index + 1) % self.recording_clips.len())
            .unwrap_or(0);
        self.selected_recording_clip_id = Some(self.recording_clips[next_index].id);
        self.clear_note_selection();
        true
    }

    pub fn select_previous_recording_clip(&mut self) -> bool {
        if self.recording_clips.is_empty() {
            self.selected_recording_clip_id = None;
            return false;
        }

        let previous_index = self
            .selected_recording_clip_id
            .and_then(|clip_id| {
                self.recording_clips
                    .iter()
                    .position(|clip| clip.id == clip_id)
            })
            .map(|index| {
                if index == 0 {
                    self.recording_clips.len() - 1
                } else {
                    index - 1
                }
            })
            .unwrap_or(self.recording_clips.len() - 1);
        self.selected_recording_clip_id = Some(self.recording_clips[previous_index].id);
        self.clear_note_selection();
        true
    }

    pub fn toggle_selected_recording_clip_mute(&mut self) -> bool {
        let Some(selected_id) = self.selected_recording_clip_id else {
            return false;
        };
        let Some(clip) = self
            .recording_clips
            .iter_mut()
            .find(|clip| clip.id == selected_id)
        else {
            self.selected_recording_clip_id = None;
            return false;
        };

        clip.muted = !clip.muted;
        true
    }

    pub fn delete_selected_recording_clip(&mut self) -> bool {
        let Some(selected_id) = self.selected_recording_clip_id else {
            return false;
        };

        self.remove_recording_clips_by_ids(&[selected_id]);
        true
    }

    pub fn recording_clip_is_muted(&self, recording_clip_id: Option<u64>) -> bool {
        recording_clip_id
            .and_then(|clip_id| self.recording_clip(clip_id))
            .map(|clip| clip.muted)
            .unwrap_or(false)
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
        self.recording_clips.clear();
        self.selected_recording_clip_id = None;
        self.recording_clip_scroll = 0;
        self.clear_note_selection();
    }

    pub fn has_note_selection(&self) -> bool {
        !self.selected_note_indices().is_empty()
    }

    pub fn selected_note_indices(&self) -> Vec<usize> {
        self.valid_selection_indices()
    }

    pub fn focused_note_index(&self) -> Option<usize> {
        let selected = self.valid_selection_indices();
        let focus = self.note_selection.focus_note_index?;
        selected.contains(&focus).then_some(focus)
    }

    pub fn anchor_note_index(&self) -> Option<usize> {
        let selected = self.valid_selection_indices();
        let anchor = self.note_selection.anchor_note_index?;
        selected.contains(&anchor).then_some(anchor)
    }

    pub fn clear_note_selection(&mut self) {
        self.note_selection = NoteSelection::default();
    }

    pub fn select_notes_at_playhead(&mut self, playhead_ticks: u64, additive: bool) -> bool {
        let hit_indices = self.playhead_hit_indices(playhead_ticks);
        if hit_indices.is_empty() {
            return false;
        }

        let mut selected = if additive {
            self.valid_selection_indices()
        } else {
            Vec::new()
        };
        selected.extend(hit_indices.iter().copied());
        self.update_note_selection(selected, hit_indices.last().copied());
        true
    }

    pub fn select_next_note(&mut self, playhead_ticks: u64, additive: bool) -> bool {
        let ordered = self.ordered_note_indices();
        let Some(candidate) = self.next_note_candidate(&ordered, playhead_ticks) else {
            return false;
        };
        self.apply_note_focus(candidate, additive);
        true
    }

    pub fn select_previous_note(&mut self, playhead_ticks: u64, additive: bool) -> bool {
        let ordered = self.ordered_note_indices();
        let Some(candidate) = self.previous_note_candidate(&ordered, playhead_ticks) else {
            return false;
        };
        self.apply_note_focus(candidate, additive);
        true
    }

    pub fn focus_first_selected_note(&mut self) -> bool {
        let selected = self.ordered_selected_note_indices();
        let Some(&focus) = selected.first() else {
            return false;
        };
        self.update_note_selection(selected, Some(focus));
        true
    }

    pub fn focus_last_selected_note(&mut self) -> bool {
        let selected = self.ordered_selected_note_indices();
        let Some(&focus) = selected.last() else {
            return false;
        };
        self.update_note_selection(selected, Some(focus));
        true
    }

    pub fn extend_note_selection_forward(&mut self, playhead_ticks: u64) -> bool {
        if !self.has_note_selection() {
            return self.select_notes_at_playhead(playhead_ticks, false);
        }

        let ordered = self.ordered_note_indices();
        let selected = self.ordered_selected_note_indices();
        let Some(&trailing) = selected.last() else {
            return false;
        };
        let Some(position) = ordered.iter().position(|&index| index == trailing) else {
            return false;
        };
        let Some(&candidate) = ordered.get(position + 1) else {
            return false;
        };

        let mut next_selected = selected;
        next_selected.push(candidate);
        self.update_note_selection(next_selected, Some(candidate));
        true
    }

    pub fn extend_note_selection_backward(&mut self, playhead_ticks: u64) -> bool {
        if !self.has_note_selection() {
            return self.select_notes_at_playhead(playhead_ticks, false);
        }

        let ordered = self.ordered_note_indices();
        let selected = self.ordered_selected_note_indices();
        let Some(&leading) = selected.first() else {
            return false;
        };
        let Some(position) = ordered.iter().position(|&index| index == leading) else {
            return false;
        };
        if position == 0 {
            return false;
        }
        let candidate = ordered[position - 1];

        let mut next_selected = selected;
        next_selected.push(candidate);
        self.update_note_selection(next_selected, Some(candidate));
        true
    }

    pub fn extend_note_selection_both(&mut self, playhead_ticks: u64) -> bool {
        if !self.has_note_selection() {
            return self.select_notes_at_playhead(playhead_ticks, false);
        }

        let ordered = self.ordered_note_indices();
        let selected = self.ordered_selected_note_indices();
        let (Some(&leading), Some(&trailing)) = (selected.first(), selected.last()) else {
            return false;
        };
        let Some(leading_position) = ordered.iter().position(|&index| index == leading) else {
            return false;
        };
        let Some(trailing_position) = ordered.iter().position(|&index| index == trailing) else {
            return false;
        };

        let mut next_selected = selected.clone();
        let mut changed = false;
        if leading_position > 0 {
            next_selected.push(ordered[leading_position - 1]);
            changed = true;
        }
        if trailing_position + 1 < ordered.len() {
            next_selected.push(ordered[trailing_position + 1]);
            changed = true;
        }
        if !changed {
            return false;
        }

        let focus = self
            .focused_note_index()
            .or_else(|| selected.last().copied());
        self.update_note_selection(next_selected, focus);
        true
    }

    pub fn contract_note_selection(&mut self) -> bool {
        let mut selected = self.ordered_selected_note_indices();
        if selected.len() <= 1 {
            return false;
        }

        let focus = self
            .focused_note_index()
            .or_else(|| selected.last().copied())
            .unwrap_or(selected[0]);
        if Some(&focus) == selected.first() {
            selected.remove(0);
            self.update_note_selection(selected.clone(), selected.first().copied());
        } else {
            selected.pop();
            self.update_note_selection(selected.clone(), selected.last().copied());
        }
        true
    }

    pub fn nudge_selected_notes_time(&mut self, delta_ticks: i64) -> bool {
        let selected = self.valid_selection_indices();
        if selected.is_empty() {
            return false;
        }

        for index in selected {
            let note = &mut self.midi_notes[index];
            note.start_ticks = if delta_ticks.is_negative() {
                note.start_ticks.saturating_sub(delta_ticks.unsigned_abs())
            } else {
                note.start_ticks.saturating_add(delta_ticks as u64)
            };
        }
        true
    }

    pub fn nudge_selected_notes_pitch(&mut self, delta: i16) -> bool {
        let selected = self.valid_selection_indices();
        if selected.is_empty() {
            return false;
        }

        let mut changed = false;
        for index in selected {
            let note = &mut self.midi_notes[index];
            let next_pitch = (i16::from(note.pitch) + delta).clamp(0, 127) as u8;
            changed |= next_pitch != note.pitch;
            note.pitch = next_pitch;
        }
        changed
    }

    fn remove_content_in_range(&mut self, range: LoopRegion) {
        self.clear_note_selection();
        let mut owned_clip_ids: Vec<u64> = self
            .midi_notes
            .iter()
            .filter(|note| note.intersects(range))
            .filter_map(|note| note.recording_clip_id)
            .collect();
        owned_clip_ids.extend(
            self.regions
                .iter()
                .filter(|region| region.intersects(range))
                .filter_map(|region| region.recording_clip_id),
        );
        owned_clip_ids.sort_unstable();
        owned_clip_ids.dedup();

        if !owned_clip_ids.is_empty() {
            self.remove_recording_clips_by_ids(&owned_clip_ids);
        }

        self.midi_notes
            .retain(|note| note.recording_clip_id.is_some() || !note.intersects(range));
        self.regions
            .retain(|region| region.recording_clip_id.is_some() || !region.intersects(range));
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
        recording_clip_id: u64,
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
            self.midi_notes.push(MidiNote::new_recorded(
                recorded_note.pitch,
                note_start,
                note_length,
                recorded_note.velocity,
                recording_clip_id,
            ));
        }
    }

    fn allocate_recording_clip_id(&mut self) -> u64 {
        let clip_id = self.next_recording_clip_id.max(1);
        self.next_recording_clip_id = clip_id.saturating_add(1);
        clip_id
    }

    fn remove_recording_clips_by_ids(&mut self, clip_ids: &[u64]) {
        if clip_ids.is_empty() {
            return;
        }

        let next_selection = self.next_recording_selection_after_delete(clip_ids);
        self.clear_note_selection();
        self.midi_notes.retain(|note| {
            !note
                .recording_clip_id
                .is_some_and(|id| clip_ids.contains(&id))
        });
        self.regions.retain(|region| {
            !region
                .recording_clip_id
                .is_some_and(|id| clip_ids.contains(&id))
        });
        self.recording_clips
            .retain(|clip| !clip_ids.contains(&clip.id));
        self.selected_recording_clip_id =
            next_selection.filter(|clip_id| self.recording_clip(*clip_id).is_some());
    }

    fn next_recording_selection_after_delete(&self, deleted_ids: &[u64]) -> Option<u64> {
        let Some(selected_id) = self.selected_recording_clip_id else {
            return None;
        };
        if !deleted_ids.contains(&selected_id) {
            return Some(selected_id);
        }

        let Some(selected_index) = self
            .recording_clips
            .iter()
            .position(|clip| clip.id == selected_id)
        else {
            return None;
        };

        self.recording_clips
            .iter()
            .skip(selected_index + 1)
            .find(|clip| !deleted_ids.contains(&clip.id))
            .or_else(|| {
                self.recording_clips
                    .iter()
                    .take(selected_index)
                    .rev()
                    .find(|clip| !deleted_ids.contains(&clip.id))
            })
            .map(|clip| clip.id)
    }

    fn valid_selection_indices(&self) -> Vec<usize> {
        let mut indices = self.note_selection.selected_note_indices.clone();
        indices.retain(|index| {
            *index < self.midi_notes.len() && self.note_matches_selection_scope(*index)
        });
        indices.sort_unstable();
        indices.dedup();
        indices
    }

    fn ordered_note_indices(&self) -> Vec<usize> {
        let mut ordered: Vec<usize> = (0..self.midi_notes.len()).collect();
        ordered.retain(|index| self.note_matches_selection_scope(*index));
        ordered.sort_by_key(|&index| {
            let note = self.midi_notes[index];
            (note.start_ticks, note.pitch, index)
        });
        ordered
    }

    fn ordered_selected_note_indices(&self) -> Vec<usize> {
        let selected = self.valid_selection_indices();
        self.ordered_note_indices()
            .into_iter()
            .filter(|index| selected.contains(index))
            .collect()
    }

    fn playhead_hit_indices(&self, playhead_ticks: u64) -> Vec<usize> {
        self.ordered_note_indices()
            .into_iter()
            .filter(|&index| {
                let note = self.midi_notes[index];
                note.start_ticks <= playhead_ticks && note.end_ticks() > playhead_ticks
            })
            .collect()
    }

    fn next_note_candidate(&self, ordered: &[usize], playhead_ticks: u64) -> Option<usize> {
        if let Some(focus) = self.focused_note_index() {
            let position = ordered.iter().position(|&index| index == focus)?;
            return ordered.get(position + 1).copied();
        }

        ordered
            .iter()
            .copied()
            .find(|&index| self.midi_notes[index].start_ticks >= playhead_ticks)
    }

    fn previous_note_candidate(&self, ordered: &[usize], playhead_ticks: u64) -> Option<usize> {
        if let Some(focus) = self.focused_note_index() {
            let position = ordered.iter().position(|&index| index == focus)?;
            return position
                .checked_sub(1)
                .and_then(|index| ordered.get(index).copied());
        }

        ordered
            .iter()
            .rev()
            .copied()
            .find(|&index| self.midi_notes[index].start_ticks < playhead_ticks)
    }

    fn apply_note_focus(&mut self, candidate: usize, additive: bool) {
        let mut selected = if additive {
            self.valid_selection_indices()
        } else {
            Vec::new()
        };
        selected.push(candidate);
        self.update_note_selection(selected, Some(candidate));
    }

    fn update_note_selection(&mut self, selected: Vec<usize>, focus: Option<usize>) {
        let mut selected = selected;
        selected.retain(|index| {
            *index < self.midi_notes.len() && self.note_matches_selection_scope(*index)
        });
        selected.sort_unstable();
        selected.dedup();

        if selected.is_empty() {
            self.note_selection = NoteSelection::default();
            return;
        }

        let ordered_selected: Vec<usize> = self
            .ordered_note_indices()
            .into_iter()
            .filter(|index| selected.contains(index))
            .collect();
        let focus = focus
            .filter(|index| ordered_selected.contains(index))
            .unwrap_or_else(|| *ordered_selected.last().unwrap_or(&ordered_selected[0]));
        let anchor = if ordered_selected.len() == 1 {
            Some(focus)
        } else if Some(&focus) == ordered_selected.first() {
            ordered_selected.last().copied()
        } else {
            ordered_selected.first().copied()
        };

        self.note_selection = NoteSelection {
            selected_note_indices: selected,
            focus_note_index: Some(focus),
            anchor_note_index: anchor,
        };
    }

    fn note_matches_selection_scope(&self, index: usize) -> bool {
        let Some(note) = self.midi_notes.get(index) else {
            return false;
        };

        if self.recording_view == RecordingView::Stacked {
            if let Some(selected_clip_id) = self.selected_recording_clip_id {
                return note.recording_clip_id == Some(selected_clip_id);
            }
        }

        true
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
    #[serde(default)]
    pub recording_clip_id: Option<u64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NoteSelection {
    pub selected_note_indices: Vec<usize>,
    pub focus_note_index: Option<usize>,
    pub anchor_note_index: Option<usize>,
}

impl MidiNote {
    pub fn new(pitch: u8, start_ticks: u64, length_ticks: u64, velocity: u8) -> Self {
        Self {
            pitch,
            start_ticks,
            length_ticks,
            velocity,
            recording_clip_id: None,
        }
    }

    pub fn new_recorded(
        pitch: u8,
        start_ticks: u64,
        length_ticks: u64,
        velocity: u8,
        recording_clip_id: u64,
    ) -> Self {
        Self {
            pitch,
            start_ticks,
            length_ticks,
            velocity,
            recording_clip_id: Some(recording_clip_id),
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
    use super::{MidiNote, Project, RecordContext, RecordingClip, RecordingView, Track, TrackKind};
    use crate::timeline::{LoopRegion, RecordingTake, Region};
    use crate::transport::{QuantizeMode, RecordMode, Transport};

    fn region_span(region: Region) -> (u64, u64) {
        (region.start_ticks, region.length_ticks)
    }

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

        assert_eq!(track.regions.len(), 1);
        assert_eq!(region_span(track.regions[0]), (240, 480));
        assert!(track.regions[0].recording_clip_id.is_some());
    }

    #[test]
    fn finish_recording_consumes_active_take() {
        let transport = Transport::default();
        let mut track = Track::new_empty("Track 1", TrackKind::Midi);
        track.begin_recording(960);

        track.finish_recording(transport, 1_920, None);

        assert!(track.active_take.is_none());
        assert_eq!(track.regions.len(), 1);
        assert_eq!(region_span(track.regions[0]), (960, 960));
        assert!(track.regions[0].recording_clip_id.is_some());
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
            track
                .preview_region(transport, 1_120, None)
                .map(region_span),
            Some((0, 960))
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
    fn select_notes_at_playhead_captures_overlapping_hits() {
        let mut track = Track::new_empty("Track 1", TrackKind::Midi);
        track.midi_notes = vec![
            MidiNote::new(60, 0, 480, 100),
            MidiNote::new(64, 120, 480, 100),
            MidiNote::new(67, 720, 240, 100),
        ];

        assert!(track.select_notes_at_playhead(240, false));
        assert_eq!(track.selected_note_indices(), vec![0, 1]);
        assert_eq!(track.focused_note_index(), Some(1));
        assert_eq!(track.anchor_note_index(), Some(0));
    }

    #[test]
    fn extend_and_contract_note_selection_follow_edges() {
        let mut track = Track::new_empty("Track 1", TrackKind::Midi);
        track.midi_notes = vec![
            MidiNote::new(60, 0, 120, 100),
            MidiNote::new(62, 240, 120, 100),
            MidiNote::new(64, 480, 120, 100),
        ];

        assert!(track.select_notes_at_playhead(0, false));
        assert!(track.extend_note_selection_forward(0));
        assert_eq!(track.selected_note_indices(), vec![0, 1]);

        assert!(track.contract_note_selection());
        assert_eq!(track.selected_note_indices(), vec![0]);
    }

    #[test]
    fn nudging_selected_notes_preserves_membership() {
        let mut track = Track::new_empty("Track 1", TrackKind::Midi);
        track.midi_notes = vec![
            MidiNote::new(60, 0, 240, 100),
            MidiNote::new(64, 480, 240, 100),
        ];

        track.select_notes_at_playhead(0, false);
        assert!(track.nudge_selected_notes_time(120));
        assert!(track.nudge_selected_notes_pitch(2));

        assert_eq!(track.selected_note_indices(), vec![0]);
        assert_eq!(track.midi_notes[0], MidiNote::new(62, 120, 240, 100));
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

        assert_eq!(track.regions.len(), 2);
        assert_eq!(region_span(track.regions[0]), (1_920, 960));
        assert_eq!(region_span(track.regions[1]), (0, 960));
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

        assert_eq!(
            track.regions.last().copied().map(region_span),
            Some((1_680, 240))
        );
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

        assert_eq!(
            track.regions.last().copied().map(region_span),
            Some((960, 960))
        );
        assert_eq!(
            track.midi_notes,
            vec![
                MidiNote::new_recorded(
                    64,
                    1_700,
                    120,
                    100,
                    track.regions[0].recording_clip_id.unwrap()
                ),
                MidiNote::new_recorded(
                    67,
                    1_080,
                    120,
                    100,
                    track.regions[0].recording_clip_id.unwrap()
                ),
            ]
        );
    }

    #[test]
    fn recording_clip_muting_and_delete_follow_clip_selection() {
        let transport = Transport::default();
        let mut track = Track::new_empty("Track 1", TrackKind::Midi);

        track.commit_take(transport, RecordingTake::new(0).release(480), None);
        track.commit_take(transport, RecordingTake::new(960).release(1_440), None);

        assert_eq!(track.recording_clips.len(), 2);
        let first_id = track.recording_clips[0].id;
        let second_id = track.recording_clips[1].id;
        assert_eq!(track.selected_recording_clip_id, Some(second_id));

        assert!(track.select_previous_recording_clip());
        assert_eq!(track.selected_recording_clip_id, Some(first_id));
        assert!(track.toggle_selected_recording_clip_mute());
        assert!(track.recording_clip_is_muted(Some(first_id)));

        assert!(track.delete_selected_recording_clip());
        assert_eq!(track.recording_clips.len(), 1);
        assert_eq!(track.recording_clips[0].id, second_id);
        assert_eq!(track.selected_recording_clip_id, Some(second_id));
    }

    #[test]
    fn stacked_view_note_selection_is_scoped_to_selected_recording() {
        let mut track = Track::new_empty("Track 1", TrackKind::Midi);
        track.midi_notes = vec![
            MidiNote::new_recorded(60, 0, 240, 100, 1),
            MidiNote::new_recorded(64, 0, 240, 100, 2),
        ];
        track.recording_clips = vec![
            RecordingClip {
                id: 1,
                region: Region::new_recorded(0, 240, 1),
                muted: false,
            },
            RecordingClip {
                id: 2,
                region: Region::new_recorded(0, 240, 2),
                muted: false,
            },
        ];
        track.recording_view = RecordingView::Stacked;
        track.selected_recording_clip_id = Some(2);

        assert!(track.select_notes_at_playhead(0, false));
        assert_eq!(track.selected_note_indices(), vec![1]);

        assert!(track.select_previous_recording_clip());
        assert!(track.select_notes_at_playhead(0, false));
        assert_eq!(track.selected_note_indices(), vec![0]);
    }
}
