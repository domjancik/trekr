use sdl3::pixels::Color;

pub const fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::RGB(r, g, b)
}

pub mod app_chrome {
    use super::rgb;
    use sdl3::pixels::Color;

    pub const WINDOW_CLEAR: Color = rgb(18, 24, 38);
    pub const SURFACE_FILL: Color = rgb(28, 34, 50);
    pub const SURFACE_BORDER: Color = rgb(88, 96, 120);
    pub const TAB_ACTIVE_FILL: Color = rgb(72, 96, 142);
    pub const TAB_INACTIVE_FILL: Color = rgb(34, 44, 64);
    pub const TAB_ACTIVE_BORDER: Color = rgb(248, 236, 162);
    pub const TAB_INACTIVE_BORDER: Color = rgb(92, 100, 120);
    pub const TAB_TEXT_ACTIVE: Color = rgb(248, 244, 212);
    pub const TAB_TEXT_INACTIVE: Color = rgb(188, 194, 206);
    pub const FOOTER_BG: Color = rgb(20, 26, 38);
    pub const FOOTER_CHIP_INACTIVE: Color = rgb(56, 66, 84);
    pub const FOOTER_TEXT_ACTIVE: Color = rgb(248, 244, 214);
    pub const FOOTER_TEXT_INACTIVE: Color = rgb(180, 190, 204);
    pub const DETAIL_TEXT: Color = rgb(188, 198, 212);
    pub const ACTION_TEXT: Color = rgb(244, 244, 236);
    pub const BRAND_FALLBACK: Color = rgb(244, 232, 146);
    pub const TAB_ACCENT_TIMELINE: Color = rgb(84, 144, 220);
    pub const TAB_ACCENT_MAPPINGS: Color = rgb(212, 168, 84);
    pub const TAB_ACCENT_MIDI_IO: Color = rgb(96, 200, 164);
    pub const TAB_ACCENT_ROUTING: Color = rgb(224, 112, 112);
    pub const FOOTER_CHIP_MAPPINGS: Color = rgb(156, 122, 68);
    pub const FOOTER_CHIP_DISCOVER: Color = rgb(72, 136, 166);
    pub const FOOTER_CHIP_DIRECT: Color = rgb(188, 82, 82);
    pub const FOOTER_TITLE_DIRECT: Color = rgb(248, 228, 208);
    pub const FOOTER_DETAIL_DIRECT: Color = rgb(214, 200, 188);
    pub const FOOTER_EMPTY_MAPPING: Color = rgb(168, 178, 194);
    pub const OVERLAY_BACKDROP: Color = Color::RGBA(10, 14, 24, 220);
    pub const OVERLAY_PANEL_FILL: Color = rgb(24, 30, 44);
    pub const OVERLAY_HEADER_TEXT: Color = rgb(150, 162, 180);
    pub const OVERLAY_ROW_SELECTED_FILL: Color = rgb(58, 72, 102);
    pub const OVERLAY_ROW_IDLE_FILL: Color = rgb(34, 42, 60);
    pub const OVERLAY_ROW_IDLE_BORDER: Color = rgb(82, 92, 114);
    pub const OVERLAY_TARGET_TEXT: Color = rgb(208, 220, 236);
    pub const OVERLAY_SCOPE_TEXT: Color = rgb(182, 192, 210);
    pub const OVERLAY_META_TEXT: Color = rgb(160, 170, 184);
}

pub mod mappings {
    use super::rgb;
    use sdl3::pixels::Color;

    pub const PAGE_BG: Color = rgb(22, 28, 42);
    pub const PAGE_BORDER: Color = rgb(88, 96, 120);
    pub const PAGE_TITLE: Color = rgb(244, 232, 146);
    pub const WRITE_MODE_ACTIVE: Color = rgb(74, 96, 138);
    pub const WRITE_MODE_INACTIVE: Color = rgb(50, 62, 88);
    pub const LEARN_ARMED: Color = rgb(146, 62, 62);
    pub const LEARN_IDLE: Color = rgb(44, 56, 78);
    pub const DIRECT_IDLE_FILL: Color = rgb(84, 58, 58);
    pub const DIRECT_ARMED_FILL: Color = rgb(140, 74, 74);
    pub const DIRECT_IDLE_BORDER: Color = rgb(108, 118, 138);
    pub const DIRECT_ARMED_BORDER: Color = rgb(252, 214, 194);
    pub const ROW_SELECTED_FILL: Color = rgb(52, 64, 92);
    pub const ROW_IDLE_FILL: Color = rgb(30, 36, 52);
    pub const ROW_SELECTED_BORDER: Color = rgb(244, 232, 146);
    pub const ROW_IDLE_BORDER: Color = rgb(78, 88, 110);
    pub const OVERVIEW_TEXT: Color = rgb(236, 242, 248);
    pub const LEARN_SELECTED_BORDER: Color = rgb(252, 232, 146);
    pub const LEARN_IDLE_BORDER: Color = rgb(96, 108, 132);
    pub const LEARN_TEXT: Color = rgb(236, 240, 246);
    pub const DIRECT_BADGE_IDLE_FILL: Color = rgb(54, 62, 82);
    pub const DIRECT_TEXT: Color = rgb(242, 238, 234);
    pub const META_TEXT: Color = rgb(154, 166, 182);
    pub const FIELD_FILL_SELECTED: Color = rgb(66, 80, 112);
    pub const FIELD_FILL_IDLE: Color = rgb(42, 50, 70);
    pub const TARGET_FILL_ENABLED: Color = rgb(182, 194, 212);
    pub const TARGET_FILL_DISABLED: Color = rgb(104, 112, 124);
    pub const SCOPE_FILL: Color = rgb(66, 74, 88);
    pub const ENABLED_FILL_ON: Color = rgb(132, 220, 120);
    pub const ENABLED_FILL_OFF: Color = rgb(92, 96, 102);
    pub const SOURCE_KIND_KEY: Color = rgb(98, 148, 232);
    pub const SOURCE_KIND_MIDI: Color = rgb(96, 202, 146);
    pub const SOURCE_KIND_OSC: Color = rgb(220, 154, 88);
    pub const WRITE_FIELD_ACTIVE: Color = rgb(92, 98, 64);
    pub const WRITE_FIELD_LEARN: Color = rgb(120, 42, 42);
    pub const DEVICE_TEXT_ACTIVE: Color = rgb(226, 234, 244);
    pub const DEVICE_TEXT_INACTIVE: Color = rgb(124, 132, 146);
    pub const TARGET_TEXT: Color = rgb(24, 28, 36);
    pub const SCOPE_TEXT: Color = rgb(236, 238, 242);
    pub const WRITE_FIELD_BORDER: Color = rgb(252, 232, 146);
    pub const WRITE_FIELD_BORDER_LEARN: Color = rgb(252, 126, 126);
    pub const TAP_BADGE_FILL: Color = rgb(86, 98, 124);
    pub const FOOTER_BG: Color = rgb(26, 32, 46);
    pub const FOOTER_TOKEN_ROW: Color = rgb(62, 78, 106);
    pub const FOOTER_TOKEN_FIELD: Color = rgb(74, 88, 118);
    pub const FOOTER_TOKEN_ACT: Color = rgb(82, 100, 136);
    pub const FOOTER_TOKEN_WRITE: Color = rgb(96, 82, 52);
    pub const FOOTER_TOKEN_DIRECT: Color = rgb(128, 78, 78);
    pub const FOOTER_TOKEN_NEW: Color = rgb(66, 96, 84);
    pub const FOOTER_TOKEN_REMOVE: Color = rgb(110, 74, 74);
}

pub mod io_pages {
    use super::rgb;
    use sdl3::pixels::Color;

    pub const PAGE_BG: Color = rgb(22, 28, 42);
    pub const PAGE_BORDER: Color = rgb(88, 96, 120);
    pub const PAGE_TITLE: Color = rgb(244, 232, 146);
    pub const SUBTITLE: Color = rgb(184, 194, 206);
    pub const PANEL_BG: Color = rgb(28, 34, 50);
    pub const ROW_IDLE_BG: Color = rgb(28, 34, 50);
    pub const ROW_SELECTED_BG: Color = rgb(56, 70, 100);
    pub const ROW_IDLE_BORDER: Color = rgb(70, 80, 102);
    pub const ROW_SELECTED_BORDER: Color = rgb(244, 232, 146);
    pub const FOCUS_BORDER: Color = rgb(242, 232, 150);
    pub const LABEL_TEXT: Color = rgb(230, 236, 244);
}

pub mod transport {
    use super::rgb;
    use sdl3::pixels::Color;

    pub const PLAY_ACTIVE: Color = rgb(96, 162, 122);
    pub const PLAY_IDLE: Color = rgb(74, 84, 102);
    pub const RECORD_ACTIVE: Color = rgb(180, 76, 76);
    pub const RECORD_IDLE: Color = rgb(88, 78, 82);
    pub const RECORD_MODE: Color = rgb(76, 94, 136);
    pub const LOOP_WRAP_EXTEND: Color = rgb(126, 106, 60);
    pub const LOOP_WRAP_CLAMP: Color = rgb(96, 82, 70);
    pub const SONG_LOOP: Color = rgb(116, 96, 54);
    pub const TEMPO: Color = rgb(70, 100, 120);
    pub const HARMONY: Color = rgb(88, 82, 124);
    pub const NOTE_ADD_HELD: Color = rgb(88, 130, 176);
    pub const NOTE_ADD_IDLE: Color = rgb(62, 76, 94);
    pub const LINK_ACTIVE: Color = rgb(74, 122, 144);
    pub const LINK_IDLE: Color = rgb(68, 76, 92);
    pub const LINK_START_STOP: Color = rgb(82, 98, 130);
    pub const LAUNCH_QUANTIZE_ENABLED: Color = rgb(102, 124, 86);
    pub const LAUNCH_QUANTIZE_DISABLED: Color = rgb(72, 88, 110);
    pub const LAUNCH_QUANTIZE_MODE: Color = rgb(78, 96, 122);
    pub const QUANTIZE: Color = rgb(70, 86, 108);
    pub const PEERS: Color = rgb(66, 80, 102);
}

pub mod discoverability {
    use super::rgb;
    use sdl3::pixels::Color;

    pub const BADGE_OVERFLOW_FILL: Color = rgb(56, 64, 80);
    pub const BADGE_OVERFLOW_TEXT: Color = rgb(228, 232, 238);
    pub const DIRECT_TAB_TARGET: Color = rgb(132, 84, 84);
    pub const DIRECT_TARGET_BORDER: Color = rgb(176, 116, 72);
    pub const DIRECT_TARGET_ACTIVE_BORDER: Color = rgb(252, 146, 126);
    pub const SLOT_BUILT_IN_FILL: Color = rgb(64, 84, 126);
    pub const SLOT_USER_FILL: Color = rgb(88, 128, 76);
    pub const SLOT_COUNT_TEXT: Color = rgb(244, 244, 236);
}
