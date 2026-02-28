const LINK_QUANTUM_BEATS: f64 = 4.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LinkSnapshot {
    pub enabled: bool,
    pub start_stop_sync: bool,
    pub peers: usize,
    pub tempo_bpm: f64,
    pub beat: f64,
    pub phase: f64,
    pub is_playing: bool,
    pub micros: i64,
}

impl Default for LinkSnapshot {
    fn default() -> Self {
        Self {
            enabled: false,
            start_stop_sync: false,
            peers: 0,
            tempo_bpm: 120.0,
            beat: 0.0,
            phase: 0.0,
            is_playing: false,
            micros: 0,
        }
    }
}

#[cfg(not(windows))]
mod backend {
    use super::{LINK_QUANTUM_BEATS, LinkSnapshot};
    use ableton_link::Link;

    pub struct LinkRuntime {
        link: Link,
        snapshot: LinkSnapshot,
    }

    impl LinkRuntime {
        pub fn new(initial_tempo_bpm: f64) -> Self {
            let mut runtime = Self {
                link: Link::new(initial_tempo_bpm),
                snapshot: LinkSnapshot {
                    tempo_bpm: initial_tempo_bpm,
                    ..LinkSnapshot::default()
                },
            };
            runtime.refresh();
            runtime
        }

        pub fn snapshot(&self) -> LinkSnapshot {
            self.snapshot
        }

        pub fn set_enabled(&mut self, enabled: bool) {
            if self.link.is_enabled() != enabled {
                self.link.enable(enabled);
            }
            self.refresh();
        }

        pub fn set_start_stop_sync(&mut self, enabled: bool) {
            if self.link.is_start_stop_sync_enabled() != enabled {
                self.link.enable_start_stop_sync(enabled);
            }
            self.refresh();
        }

        pub fn commit_tempo(&mut self, bpm: f64) {
            let micros = self.link.clock().micros();
            let link_ptr: *mut Link = &mut self.link;
            self.link.with_app_session_state(move |mut session| {
                session.set_tempo(bpm, micros);
                unsafe {
                    (*link_ptr).commit_app_session_state(session);
                }
            });
            self.refresh();
        }

        pub fn commit_playing(&mut self, playing: bool, beat: f64) {
            let micros = self.link.clock().micros();
            let link_ptr: *mut Link = &mut self.link;
            self.link.with_app_session_state(move |mut session| {
                session.set_is_playing_and_request_beat_at_time(
                    playing,
                    micros,
                    beat,
                    LINK_QUANTUM_BEATS,
                );
                unsafe {
                    (*link_ptr).commit_app_session_state(session);
                }
            });
            self.refresh();
        }

        pub fn refresh(&mut self) -> LinkSnapshot {
            let micros = self.link.clock().micros();
            let enabled = self.link.is_enabled();
            let start_stop_sync = self.link.is_start_stop_sync_enabled();
            let peers = self.link.num_peers();
            let mut snapshot = LinkSnapshot {
                enabled,
                start_stop_sync,
                peers,
                micros,
                ..self.snapshot
            };
            self.link.with_app_session_state(|session| {
                snapshot.tempo_bpm = session.tempo();
                snapshot.beat = session.beat_at_time(micros, LINK_QUANTUM_BEATS);
                snapshot.phase = session.phase_at_time(micros, LINK_QUANTUM_BEATS);
                snapshot.is_playing = session.is_playing();
            });
            self.snapshot = snapshot;
            snapshot
        }
    }
}

#[cfg(windows)]
mod backend {
    use super::LinkSnapshot;
    use std::time::Instant;

    pub struct LinkRuntime {
        snapshot: LinkSnapshot,
        started_at: Instant,
    }

    impl LinkRuntime {
        pub fn new(initial_tempo_bpm: f64) -> Self {
            Self {
                snapshot: LinkSnapshot {
                    tempo_bpm: initial_tempo_bpm,
                    ..LinkSnapshot::default()
                },
                started_at: Instant::now(),
            }
        }

        pub fn snapshot(&self) -> LinkSnapshot {
            self.snapshot
        }

        pub fn set_enabled(&mut self, enabled: bool) {
            self.snapshot.enabled = enabled;
        }

        pub fn set_start_stop_sync(&mut self, enabled: bool) {
            self.snapshot.start_stop_sync = enabled;
        }

        pub fn commit_tempo(&mut self, bpm: f64) {
            self.snapshot.tempo_bpm = bpm.max(20.0);
        }

        pub fn commit_playing(&mut self, playing: bool, _beat: f64) {
            self.snapshot.is_playing = playing;
        }

        pub fn refresh(&mut self) -> LinkSnapshot {
            self.snapshot.micros = self.started_at.elapsed().as_micros() as i64;
            if self.snapshot.is_playing {
                let beats_per_second = self.snapshot.tempo_bpm / 60.0;
                self.snapshot.beat = (self.snapshot.micros as f64 / 1_000_000.0) * beats_per_second;
                self.snapshot.phase = self.snapshot.beat.rem_euclid(super::LINK_QUANTUM_BEATS);
            }
            self.snapshot
        }
    }
}

pub use backend::LinkRuntime;

#[cfg(test)]
mod tests {
    use super::LinkRuntime;

    #[test]
    fn link_runtime_tracks_enable_flags() {
        let mut runtime = LinkRuntime::new(120.0);
        runtime.set_enabled(true);
        runtime.set_start_stop_sync(true);
        let snapshot = runtime.refresh();
        assert!(snapshot.enabled);
        assert!(snapshot.start_stop_sync);
    }
}
