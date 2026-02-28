use crate::routing::TrackRouting;
use crate::timeline::{LoopRegion, Region};
use crate::transport::Transport;

#[derive(Debug, Clone)]
pub struct Project {
    pub name: String,
    pub transport: Transport,
    pub loop_region: LoopRegion,
    pub tracks: Vec<Track>,
}

impl Project {
    pub fn demo() -> Self {
        Self {
            name: "Untitled".to_string(),
            transport: Transport::default(),
            loop_region: LoopRegion::new(0, 16 * 960),
            tracks: vec![
                Track::new("Track 1", TrackKind::Midi),
                Track::new("Track 2", TrackKind::Midi),
            ],
        }
    }

    pub fn full_song_range(&self) -> LoopRegion {
        let end_ticks = self
            .tracks
            .iter()
            .flat_map(|track| track.regions.iter())
            .map(|region| region.start_ticks + region.length_ticks)
            .max()
            .unwrap_or(self.loop_region.end_ticks());

        LoopRegion::new(0, end_ticks.max(self.loop_region.end_ticks()))
    }
}

#[derive(Debug, Clone)]
pub struct Track {
    pub name: String,
    pub kind: TrackKind,
    pub state: TrackState,
    pub routing: TrackRouting,
    pub regions: Vec<Region>,
}

impl Track {
    pub fn new(name: &str, kind: TrackKind) -> Self {
        Self {
            name: name.to_string(),
            kind,
            state: TrackState::default(),
            routing: TrackRouting::default(),
            regions: Vec::new(),
        }
    }

    /// V1 is MIDI-first, so record commit currently appends a MIDI region.
    pub fn commit_recording(&mut self, transport: Transport, pressed_at: u64, released_at: u64) {
        let start_ticks = transport.quantize_to_nearest(pressed_at);
        let end_ticks = transport.quantize_to_nearest(released_at);
        let length_ticks = end_ticks.saturating_sub(start_ticks);

        if length_ticks == 0 {
            return;
        }

        self.regions.push(Region::new(start_ticks, length_ticks));
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrackKind {
    Midi,
    Audio,
    Hybrid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TrackState {
    pub armed: bool,
    pub muted: bool,
    pub soloed: bool,
    pub passthrough: bool,
}

#[cfg(test)]
mod tests {
    use super::{Project, Track, TrackKind};
    use crate::timeline::Region;
    use crate::transport::{QuantizeMode, Transport};

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

        track.commit_recording(transport, 220, 721);

        assert_eq!(track.regions, vec![Region::new(240, 480)]);
    }
}
