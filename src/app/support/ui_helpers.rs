use super::*;

pub(crate) fn centered_text_rect(rect: Rect) -> Rect {
    Rect::new(
        rect.x,
        rect.y + ((rect.height() as i32 - 8) / 2).max(0),
        rect.width(),
        8,
    )
}

pub(crate) fn inset_rect_1(rect: Rect) -> Rect {
    Rect::new(
        rect.x + 1,
        rect.y + 1,
        rect.width().saturating_sub(2),
        rect.height().saturating_sub(2),
    )
}

pub(crate) fn chrome_text_rect(rect: Rect) -> Rect {
    inset_rect_1(rect)
}

pub(crate) fn chrome_compact_text_rect(rect: Rect) -> Rect {
    Rect::new(
        rect.x + 2,
        rect.y + 2,
        rect.width().saturating_sub(4),
        7,
    )
}

pub(crate) fn contrasting_text_color(fill: Color, theme: &crate::theme::Theme) -> Color {
    theme.text_on_fill(fill)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contrasting_text_color_tracks_fill_luminance() {
        assert_eq!(
            contrasting_text_color(
                Color::RGB(40, 44, 52),
                crate::theme::theme(crate::theme::ThemePreset::DefaultDark)
            ),
            Color::RGB(244, 244, 236)
        );
        assert_eq!(
            contrasting_text_color(
                Color::RGB(240, 240, 240),
                crate::theme::theme(crate::theme::ThemePreset::DefaultDark)
            ),
            Color::RGB(24, 28, 36)
        );
    }
}
