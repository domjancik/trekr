use super::*;

pub(crate) fn centered_text_rect(rect: Rect) -> Rect {
    Rect::new(
        rect.x,
        rect.y + ((rect.height() as i32 - 8) / 2).max(0),
        rect.width(),
        8,
    )
}

pub(crate) fn contrasting_text_color(fill: Color) -> Color {
    let brightness = u32::from(fill.r) * 299 + u32::from(fill.g) * 587 + u32::from(fill.b) * 114;
    if brightness / 1000 < 140 {
        Color::RGB(244, 244, 236)
    } else {
        Color::RGB(24, 28, 36)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contrasting_text_color_tracks_fill_luminance() {
        assert_eq!(
            contrasting_text_color(Color::RGB(40, 44, 52)),
            Color::RGB(244, 244, 236)
        );
        assert_eq!(
            contrasting_text_color(Color::RGB(240, 240, 240)),
            Color::RGB(24, 28, 36)
        );
    }
}
