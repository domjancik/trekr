use super::*;

pub(crate) fn logical_viewport_size(output_size: (u32, u32), display_scale: f32) -> (u32, u32) {
    let scale = display_scale.max(1.0);
    (
        ((output_size.0 as f32) / scale).round().max(1.0) as u32,
        ((output_size.1 as f32) / scale).round().max(1.0) as u32,
    )
}

pub(crate) fn active_draw_size(
    canvas_output_size: (u32, u32),
    viewport_size: (u32, u32),
) -> (u32, u32) {
    if viewport_size.0 > 0 && viewport_size.1 > 0 {
        viewport_size
    } else {
        canvas_output_size
    }
}

pub(crate) fn effective_ui_scale(display_scale: f32, override_scale: Option<f32>) -> f32 {
    override_scale.unwrap_or(display_scale).max(1.0)
}

pub(crate) fn should_interpolate_window_scale(
    mode: UiScalingMode,
    scale_x: f32,
    scale_y: f32,
) -> bool {
    match mode {
        UiScalingMode::Auto => {
            has_fractional_scale_component(scale_x) || has_fractional_scale_component(scale_y)
        }
        UiScalingMode::Nearest => false,
        UiScalingMode::Linear => true,
    }
}

fn has_fractional_scale_component(scale: f32) -> bool {
    let rounded = scale.round();
    (scale - rounded).abs() > 0.001
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn logical_viewport_size_respects_display_scale() {
        assert_eq!(logical_viewport_size((2560, 1440), 2.0), (1280, 720));
        assert_eq!(logical_viewport_size((1920, 1080), 1.5), (1280, 720));
    }

    #[test]
    fn active_draw_size_prefers_logical_viewport_over_output_pixels() {
        assert_eq!(active_draw_size((2560, 1440), (1280, 720)), (1280, 720));
        assert_eq!(active_draw_size((1280, 720), (0, 0)), (1280, 720));
    }

    #[test]
    fn ui_scale_override_wins_over_display_scale() {
        assert_eq!(effective_ui_scale(1.5, Some(2.0)), 2.0);
        assert_eq!(effective_ui_scale(1.5, None), 1.5);
        assert_eq!(effective_ui_scale(0.5, None), 1.0);
    }

    #[test]
    fn auto_window_scale_interpolation_only_enables_for_non_integer_values() {
        assert!(should_interpolate_window_scale(
            UiScalingMode::Auto,
            1.5,
            1.0
        ));
        assert!(should_interpolate_window_scale(
            UiScalingMode::Auto,
            1.0,
            1.25
        ));
        assert!(!should_interpolate_window_scale(
            UiScalingMode::Auto,
            2.0,
            1.0
        ));
        assert!(!should_interpolate_window_scale(
            UiScalingMode::Auto,
            2.0004,
            1.0
        ));
    }

    #[test]
    fn explicit_window_scale_modes_override_auto_behavior() {
        assert!(!should_interpolate_window_scale(
            UiScalingMode::Nearest,
            1.5,
            1.5
        ));
        assert!(should_interpolate_window_scale(
            UiScalingMode::Linear,
            1.0,
            1.0
        ));
    }
}
