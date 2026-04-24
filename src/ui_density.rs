#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiDensityPreset {
    Default,
    Compact,
    Touch,
    Tiny,
}

impl UiDensityPreset {
    pub fn from_name(value: &str) -> Option<Self> {
        if value.eq_ignore_ascii_case("default") {
            Some(Self::Default)
        } else if value.eq_ignore_ascii_case("compact") {
            Some(Self::Compact)
        } else if value.eq_ignore_ascii_case("touch") {
            Some(Self::Touch)
        } else if value.eq_ignore_ascii_case("tiny") {
            Some(Self::Tiny)
        } else {
            None
        }
    }

    pub fn from_env() -> Self {
        std::env::var("TREKR_UI_DENSITY")
            .ok()
            .as_deref()
            .and_then(Self::from_name)
            .unwrap_or(Self::Default)
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Compact => "compact",
            Self::Touch => "touch",
            Self::Tiny => "tiny",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiMetrics {
    pub surface_gutter_px: i32,
    pub frame_inset_x_px: i32,
    pub frame_inset_y_px: i32,
    pub tabs_height_px: u32,
    pub tabs_gap_px: i32,
    pub tabs_column_gap_px: i32,
    pub footer_height_px: u32,
    pub footer_gap_px: i32,
    pub page_header_height_px: u32,
    pub page_header_gap_px: i32,
    pub transport_strip_height_px: u32,
    pub panel_gap_px: i32,
    pub row_gap_px: i32,
    pub chip_min_height_px: u32,
    pub touch_target_min_px: u32,
    pub track_pair_gap_px: i32,
    pub track_inner_gap_px: i32,
    pub track_header_height_px: u32,
    pub track_label_height_px: u32,
    pub track_status_height_px: u32,
    pub track_status_y_offset_px: i32,
    pub track_cross_axis_header_px: u32,
    pub detail_badge_height_px: u32,
    pub detail_badge_inset_x_px: i32,
    pub detail_badge_bottom_offset_px: i32,
    pub mapping_badge_height_px: u32,
    pub mapping_header_y_px: i32,
    pub mapping_list_y_px: i32,
    pub mapping_footer_height_px: u32,
    pub mapping_row_height_px: u32,
    pub mapping_row_gap_px: i32,
    pub mappings_side_inset_px: i32,
    pub midi_header_height_px: u32,
    pub midi_header_gap_px: i32,
    pub midi_panel_header_height_px: u32,
    pub midi_list_inset_px: i32,
    pub midi_column_gap_px: i32,
    pub routing_inset_x_px: i32,
    pub routing_inset_y_px: i32,
    pub routing_header_height_px: u32,
    pub routing_header_gap_px: i32,
    pub routing_panel_gap_px: i32,
    pub routing_row_gap_px: i32,
}

const DEFAULT_METRICS: UiMetrics = UiMetrics {
    surface_gutter_px: 18,
    frame_inset_x_px: 24,
    frame_inset_y_px: 24,
    tabs_height_px: 28,
    tabs_gap_px: 12,
    tabs_column_gap_px: 10,
    footer_height_px: 22,
    footer_gap_px: 8,
    page_header_height_px: 28,
    page_header_gap_px: 6,
    transport_strip_height_px: 34,
    panel_gap_px: 8,
    row_gap_px: 8,
    chip_min_height_px: 16,
    touch_target_min_px: 18,
    track_pair_gap_px: 14,
    track_inner_gap_px: 6,
    track_header_height_px: 34,
    track_label_height_px: 24,
    track_status_height_px: 15,
    track_status_y_offset_px: -1,
    track_cross_axis_header_px: 56,
    detail_badge_height_px: 11,
    detail_badge_inset_x_px: 4,
    detail_badge_bottom_offset_px: 12,
    mapping_badge_height_px: 16,
    mapping_header_y_px: 30,
    mapping_list_y_px: 44,
    mapping_footer_height_px: 12,
    mapping_row_height_px: 18,
    mapping_row_gap_px: 3,
    mappings_side_inset_px: 8,
    midi_header_height_px: 28,
    midi_header_gap_px: 10,
    midi_panel_header_height_px: 22,
    midi_list_inset_px: 10,
    midi_column_gap_px: 14,
    routing_inset_x_px: 12,
    routing_inset_y_px: 32,
    routing_header_height_px: 48,
    routing_header_gap_px: 10,
    routing_panel_gap_px: 12,
    routing_row_gap_px: 10,
};

const COMPACT_METRICS: UiMetrics = UiMetrics {
    surface_gutter_px: 14,
    frame_inset_x_px: 18,
    frame_inset_y_px: 18,
    tabs_height_px: 24,
    tabs_gap_px: 8,
    tabs_column_gap_px: 8,
    footer_height_px: 20,
    footer_gap_px: 6,
    page_header_height_px: 24,
    page_header_gap_px: 4,
    transport_strip_height_px: 30,
    panel_gap_px: 6,
    row_gap_px: 6,
    chip_min_height_px: 14,
    touch_target_min_px: 16,
    track_pair_gap_px: 10,
    track_inner_gap_px: 4,
    track_header_height_px: 30,
    track_label_height_px: 20,
    track_status_height_px: 13,
    track_status_y_offset_px: -1,
    track_cross_axis_header_px: 48,
    detail_badge_height_px: 10,
    detail_badge_inset_x_px: 3,
    detail_badge_bottom_offset_px: 11,
    mapping_badge_height_px: 14,
    mapping_header_y_px: 26,
    mapping_list_y_px: 38,
    mapping_footer_height_px: 10,
    mapping_row_height_px: 16,
    mapping_row_gap_px: 2,
    mappings_side_inset_px: 6,
    midi_header_height_px: 24,
    midi_header_gap_px: 8,
    midi_panel_header_height_px: 18,
    midi_list_inset_px: 8,
    midi_column_gap_px: 10,
    routing_inset_x_px: 10,
    routing_inset_y_px: 24,
    routing_header_height_px: 40,
    routing_header_gap_px: 8,
    routing_panel_gap_px: 10,
    routing_row_gap_px: 8,
};

const TOUCH_METRICS: UiMetrics = UiMetrics {
    surface_gutter_px: 22,
    frame_inset_x_px: 28,
    frame_inset_y_px: 28,
    tabs_height_px: 34,
    tabs_gap_px: 14,
    tabs_column_gap_px: 12,
    footer_height_px: 26,
    footer_gap_px: 10,
    page_header_height_px: 34,
    page_header_gap_px: 8,
    transport_strip_height_px: 40,
    panel_gap_px: 10,
    row_gap_px: 10,
    chip_min_height_px: 20,
    touch_target_min_px: 24,
    track_pair_gap_px: 16,
    track_inner_gap_px: 8,
    track_header_height_px: 38,
    track_label_height_px: 26,
    track_status_height_px: 17,
    track_status_y_offset_px: -1,
    track_cross_axis_header_px: 64,
    detail_badge_height_px: 12,
    detail_badge_inset_x_px: 5,
    detail_badge_bottom_offset_px: 13,
    mapping_badge_height_px: 20,
    mapping_header_y_px: 34,
    mapping_list_y_px: 50,
    mapping_footer_height_px: 14,
    mapping_row_height_px: 22,
    mapping_row_gap_px: 4,
    mappings_side_inset_px: 10,
    midi_header_height_px: 34,
    midi_header_gap_px: 12,
    midi_panel_header_height_px: 26,
    midi_list_inset_px: 12,
    midi_column_gap_px: 16,
    routing_inset_x_px: 16,
    routing_inset_y_px: 36,
    routing_header_height_px: 56,
    routing_header_gap_px: 12,
    routing_panel_gap_px: 14,
    routing_row_gap_px: 12,
};

const TINY_METRICS: UiMetrics = UiMetrics {
    surface_gutter_px: 10,
    frame_inset_x_px: 12,
    frame_inset_y_px: 12,
    tabs_height_px: 22,
    tabs_gap_px: 6,
    tabs_column_gap_px: 6,
    footer_height_px: 18,
    footer_gap_px: 4,
    page_header_height_px: 22,
    page_header_gap_px: 4,
    transport_strip_height_px: 26,
    panel_gap_px: 4,
    row_gap_px: 4,
    chip_min_height_px: 12,
    touch_target_min_px: 14,
    track_pair_gap_px: 8,
    track_inner_gap_px: 4,
    track_header_height_px: 26,
    track_label_height_px: 16,
    track_status_height_px: 11,
    track_status_y_offset_px: -1,
    track_cross_axis_header_px: 42,
    detail_badge_height_px: 9,
    detail_badge_inset_x_px: 2,
    detail_badge_bottom_offset_px: 9,
    mapping_badge_height_px: 12,
    mapping_header_y_px: 24,
    mapping_list_y_px: 34,
    mapping_footer_height_px: 10,
    mapping_row_height_px: 14,
    mapping_row_gap_px: 2,
    mappings_side_inset_px: 6,
    midi_header_height_px: 22,
    midi_header_gap_px: 6,
    midi_panel_header_height_px: 18,
    midi_list_inset_px: 6,
    midi_column_gap_px: 8,
    routing_inset_x_px: 8,
    routing_inset_y_px: 18,
    routing_header_height_px: 34,
    routing_header_gap_px: 6,
    routing_panel_gap_px: 8,
    routing_row_gap_px: 6,
};

pub fn ui_metrics(preset: UiDensityPreset) -> &'static UiMetrics {
    match preset {
        UiDensityPreset::Default => &DEFAULT_METRICS,
        UiDensityPreset::Compact => &COMPACT_METRICS,
        UiDensityPreset::Touch => &TOUCH_METRICS,
        UiDensityPreset::Tiny => &TINY_METRICS,
    }
}

#[cfg(test)]
mod tests {
    use super::{UiDensityPreset, ui_metrics};

    #[test]
    fn density_aliases_parse_expected_names() {
        assert_eq!(
            UiDensityPreset::from_name("default"),
            Some(UiDensityPreset::Default)
        );
        assert_eq!(
            UiDensityPreset::from_name("compact"),
            Some(UiDensityPreset::Compact)
        );
        assert_eq!(
            UiDensityPreset::from_name("touch"),
            Some(UiDensityPreset::Touch)
        );
        assert_eq!(
            UiDensityPreset::from_name("tiny"),
            Some(UiDensityPreset::Tiny)
        );
    }

    #[test]
    fn touch_density_expands_targets_over_default() {
        assert!(
            ui_metrics(UiDensityPreset::Touch).touch_target_min_px
                > ui_metrics(UiDensityPreset::Default).touch_target_min_px
        );
    }

    #[test]
    fn tiny_density_is_tighter_than_default() {
        let tiny = ui_metrics(UiDensityPreset::Tiny);
        let default = ui_metrics(UiDensityPreset::Default);
        assert!(tiny.surface_gutter_px < default.surface_gutter_px);
        assert!(tiny.transport_strip_height_px < default.transport_strip_height_px);
    }
}
