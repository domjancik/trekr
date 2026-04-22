use sdl3::pixels::Color;
use sdl3::render::FRect;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FramePresentPlan {
    pub clear_color: Color,
    pub destination: FRect,
    pub interpolate: bool,
}

pub fn window_present_plan(
    output_size: (u32, u32),
    interpolate: bool,
    clear_color: Color,
) -> FramePresentPlan {
    FramePresentPlan {
        clear_color,
        destination: FRect::new(
            0.0,
            0.0,
            output_size.0.max(1) as f32,
            output_size.1.max(1) as f32,
        ),
        interpolate,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn present_plan_matches_output_size() {
        let plan = window_present_plan((1280, 720), true, Color::RGB(1, 2, 3));
        assert_eq!(plan.destination.w, 1280.0);
        assert_eq!(plan.destination.h, 720.0);
        assert!(plan.interpolate);
    }
}
