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
}
