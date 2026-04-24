use std::ffi::c_void;
use std::ptr::NonNull;

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

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct NativeLinkSnapshot {
    enabled: u8,
    start_stop_sync: u8,
    is_playing: u8,
    reserved: u8,
    peers: usize,
    tempo_bpm: f64,
    beat: f64,
    phase: f64,
    micros: i64,
}

impl From<NativeLinkSnapshot> for LinkSnapshot {
    fn from(value: NativeLinkSnapshot) -> Self {
        Self {
            enabled: value.enabled != 0,
            start_stop_sync: value.start_stop_sync != 0,
            peers: value.peers,
            tempo_bpm: value.tempo_bpm,
            beat: value.beat,
            phase: value.phase,
            is_playing: value.is_playing != 0,
            micros: value.micros,
        }
    }
}

pub struct LinkRuntime {
    handle: NonNull<c_void>,
    snapshot: LinkSnapshot,
}

impl LinkRuntime {
    pub fn new(initial_tempo_bpm: f64) -> Self {
        let handle = unsafe { trekr_link_new(initial_tempo_bpm) };
        let handle = NonNull::new(handle).expect("link runtime should allocate");
        let mut runtime = Self {
            handle,
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
        unsafe { trekr_link_set_enabled(self.handle.as_ptr(), bool_to_u8(enabled)) };
        self.refresh();
    }

    pub fn set_start_stop_sync(&mut self, enabled: bool) {
        unsafe {
            trekr_link_set_start_stop_sync_enabled(self.handle.as_ptr(), bool_to_u8(enabled))
        };
        self.refresh();
    }

    pub fn commit_tempo(&mut self, bpm: f64) {
        unsafe { trekr_link_commit_tempo(self.handle.as_ptr(), bpm) };
        self.refresh();
    }

    pub fn commit_playing(&mut self, playing: bool, beat: f64) {
        unsafe {
            trekr_link_commit_playing(
                self.handle.as_ptr(),
                bool_to_u8(playing),
                beat,
                LINK_QUANTUM_BEATS,
            )
        };
        self.refresh();
    }

    pub fn refresh(&mut self) -> LinkSnapshot {
        let mut native = NativeLinkSnapshot {
            enabled: 0,
            start_stop_sync: 0,
            is_playing: 0,
            reserved: 0,
            peers: 0,
            tempo_bpm: self.snapshot.tempo_bpm,
            beat: 0.0,
            phase: 0.0,
            micros: 0,
        };
        unsafe {
            trekr_link_snapshot(self.handle.as_ptr(), LINK_QUANTUM_BEATS, &mut native);
        }
        self.snapshot = native.into();
        self.snapshot
    }
}

impl Drop for LinkRuntime {
    fn drop(&mut self) {
        unsafe { trekr_link_free(self.handle.as_ptr()) };
    }
}

fn bool_to_u8(value: bool) -> u8 {
    if value { 1 } else { 0 }
}

unsafe extern "C" {
    fn trekr_link_new(bpm: f64) -> *mut c_void;
    fn trekr_link_free(handle: *mut c_void);
    fn trekr_link_set_enabled(handle: *mut c_void, enabled: u8);
    fn trekr_link_set_start_stop_sync_enabled(handle: *mut c_void, enabled: u8);
    fn trekr_link_snapshot(handle: *mut c_void, quantum: f64, snapshot: *mut NativeLinkSnapshot);
    fn trekr_link_commit_tempo(handle: *mut c_void, bpm: f64);
    fn trekr_link_commit_playing(handle: *mut c_void, is_playing: u8, beat: f64, quantum: f64);
}

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
