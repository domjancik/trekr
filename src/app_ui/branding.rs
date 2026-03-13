use std::time::Duration;

pub const BRAND_NAME: &str = "trekr";
pub const BRAND_SITE: &str = "domj.net";
pub const BRAND_HASH: &str = match option_env!("TREKR_BUILD_HASH") {
    Some(value) => value,
    None => "dev",
};
const BRAND_DATE: Option<&str> = option_env!("TREKR_BUILD_DATE");

pub fn brand_build_date() -> Option<&'static str> {
    BRAND_DATE
        .filter(|value| !value.is_empty() && !value.eq_ignore_ascii_case("unknown"))
        .map(compact_build_date)
}

pub fn brand_fallback_badge() -> String {
    brand_build_date()
        .map(|value| format!("{value} {BRAND_HASH}"))
        .unwrap_or_else(|| BRAND_HASH.to_string())
}

pub fn startup_logo_intensity(elapsed: Duration, index: usize) -> f32 {
    let phase1_lead_ms = 640_u64;
    let phase1_step_ms = 180_u64;
    let phase1_trail_ms = 420_u64;
    let phase1_start = Duration::from_millis(phase1_lead_ms);
    let phase1_end = phase1_start + Duration::from_millis((phase1_step_ms * 4) + phase1_trail_ms);
    if elapsed < phase1_start {
        return 0.0;
    }
    if elapsed < phase1_end {
        let local_start = phase1_start + Duration::from_millis(phase1_step_ms * index as u64);
        let local_end = local_start + Duration::from_millis(phase1_trail_ms);
        if elapsed < local_start || elapsed > local_end {
            return 0.0;
        }
        let t = (elapsed - local_start).as_secs_f32()
            / Duration::from_millis(phase1_trail_ms).as_secs_f32();
        // phase 1 trail: 1 -> middle -> 0 with overlap (no fully dark gaps)
        return (1.0 - t).clamp(0.0, 1.0);
    }

    let phase2_gap_ms = 100_u64;
    let phase2_start = phase1_end + Duration::from_millis(phase2_gap_ms);
    let phase2_step_ms = 220_u64;
    let phase2_ramp_ms = 300_u64;
    let reveal_index = startup_logo_reveal_step(index);
    if elapsed < phase2_start {
        return 0.0;
    }

    let ring_start = phase2_start + Duration::from_millis(phase2_step_ms * reveal_index);
    if elapsed <= ring_start {
        return 0.0;
    }
    let ring_end = ring_start + Duration::from_millis(phase2_ramp_ms);
    if elapsed >= ring_end {
        return 1.0;
    }

    let t =
        (elapsed - ring_start).as_secs_f32() / Duration::from_millis(phase2_ramp_ms).as_secs_f32();
    // phase 2 reveal: 0 -> middle -> 1
    t.clamp(0.0, 1.0)
}

pub fn startup_logo_animation_duration() -> Duration {
    // phase1 lead + overlap trail-through + phase2 gap + center-out reveal + white hold
    Duration::from_millis(640 + (180 * 4) + 420 + 100 + (220 * 2) + 300 + 320)
}

fn compact_build_date(value: &str) -> &str {
    if value.len() >= 10 {
        &value[..10]
    } else {
        value
    }
}

fn startup_logo_reveal_step(index: usize) -> u64 {
    match index {
        2 => 0,     // E first
        1 | 3 => 1, // inner pair
        0 | 4 => 2, // outer pair
        _ => 2,
    }
}
