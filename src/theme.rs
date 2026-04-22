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
