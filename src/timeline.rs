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

    pub fn set_start_preserving_end(&mut self, start_ticks: u64) {
        let end_ticks = self.end_ticks().max(start_ticks + 1);
        self.start_ticks = start_ticks.min(end_ticks - 1);
        self.length_ticks = end_ticks - self.start_ticks;
    }

    pub fn set_end(&mut self, end_ticks: u64) {
        let clamped_end = end_ticks.max(self.start_ticks + 1);
        self.length_ticks = clamped_end - self.start_ticks;
    }

    pub fn shift_by(&mut self, delta_ticks: i64) {
        let shifted_start = if delta_ticks.is_negative() {
            self.start_ticks.saturating_sub(delta_ticks.unsigned_abs())
        } else {
            self.start_ticks.saturating_add(delta_ticks as u64)
        };
        self.start_ticks = shifted_start;
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

#[cfg(test)]
mod tests {
    use super::LoopRegion;

    #[test]
    fn loop_region_can_move_start_without_collapsing() {
        let mut loop_region = LoopRegion::new(100, 80);
        loop_region.set_start_preserving_end(140);

        assert_eq!(loop_region.start_ticks, 140);
        assert_eq!(loop_region.end_ticks(), 180);
    }

    #[test]
    fn loop_region_end_is_clamped_after_start() {
        let mut loop_region = LoopRegion::new(100, 80);
        loop_region.set_end(90);

        assert_eq!(loop_region.end_ticks(), 101);
    }

    #[test]
    fn loop_region_can_shift_both_directions() {
        let mut loop_region = LoopRegion::new(100, 80);
        loop_region.shift_by(40);
        assert_eq!(loop_region.start_ticks, 140);

        loop_region.shift_by(-200);
        assert_eq!(loop_region.start_ticks, 0);
    }
}
