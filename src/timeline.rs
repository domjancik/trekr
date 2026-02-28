#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Region {
    pub start_ticks: u64,
    pub length_ticks: u64,
}

impl Region {
    pub fn new(start_ticks: u64, length_ticks: u64) -> Self {
        Self {
            start_ticks,
            length_ticks,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LoopRegion {
    pub start_ticks: u64,
    pub length_ticks: u64,
}

impl LoopRegion {
    pub fn new(start_ticks: u64, length_ticks: u64) -> Self {
        Self {
            start_ticks,
            length_ticks,
        }
    }

    pub fn end_ticks(self) -> u64 {
        self.start_ticks + self.length_ticks
    }

    pub fn contains(self, ticks: u64) -> bool {
        ticks >= self.start_ticks && ticks <= self.end_ticks()
    }
}

/// Hold-to-record captures a press/release span before it is committed as a region.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecordingTake {
    pub pressed_at_ticks: u64,
    pub released_at_ticks: Option<u64>,
}

impl RecordingTake {
    pub fn new(pressed_at_ticks: u64) -> Self {
        Self {
            pressed_at_ticks,
            released_at_ticks: None,
        }
    }

    pub fn release(mut self, released_at_ticks: u64) -> Self {
        self.released_at_ticks = Some(released_at_ticks);
        self
    }
}
