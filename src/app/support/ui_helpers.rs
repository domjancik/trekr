use super::*;

pub(crate) fn centered_text_rect(rect: Rect) -> Rect {
    Rect::new(
        rect.x,
        rect.y + ((rect.height() as i32 - 8) / 2).max(0),
        rect.width(),
        8,
    )
}

pub(crate) fn chrome_compact_text_rect(rect: Rect) -> Rect {
    Rect::new(
        rect.x + 2,
        rect.y + ((rect.height() as i32 - 8) / 2).max(0) + 1,
        rect.width().saturating_sub(4),
        8,
    )
}

pub(crate) fn compact_label_rect(rect: Rect) -> Rect {
    Rect::new(
        rect.x + 2,
        rect.y + ((rect.height() as i32 - 8) / 2).max(0) + 1,
        rect.width().saturating_sub(4),
        8,
    )
}

pub(crate) fn contrasting_text_color(fill: Color, theme: &crate::theme::Theme) -> Color {
    theme.text_on_fill(fill)
}

pub(crate) fn horizontally_center_text_rect(text: &str, rect: Rect, scale: u32) -> Rect {
    let fitted = crate::ui::truncate_text_to_width(text, rect.width(), scale);
    if fitted.is_empty() {
        return rect;
    }
    let text_width = crate::ui::text_width(&fitted, scale).min(rect.width());
    let x = rect.x + ((rect.width() as i32 - text_width as i32) / 2).max(0);
    Rect::new(x, rect.y, rect.width(), rect.height())
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
