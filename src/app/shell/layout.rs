use super::*;

pub(crate) fn page_tabs_layout(bounds: Rect) -> (Rect, Rect) {
    let branding_width = preferred_branding_width(bounds.width());
    if branding_width == 0 {
        return (Rect::new(bounds.x, bounds.y, 0, bounds.height()), bounds);
    }

    let gap = 14_i32;
    let tabs_width = bounds
        .width()
        .saturating_sub(branding_width)
        .saturating_sub(gap as u32);
    (
        Rect::new(bounds.x, bounds.y, branding_width, bounds.height()),
        Rect::new(
            bounds.x + branding_width as i32 + gap,
            bounds.y,
            tabs_width,
            bounds.height(),
        ),
    )
}

pub(crate) fn preferred_branding_width(bounds_width: u32) -> u32 {
    let desired = 220_u32;
    let minimum_tabs_width = 360_u32;
    if bounds_width <= desired + minimum_tabs_width {
        0
    } else {
        desired
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preferred_branding_width_respects_minimum_tabs_space() {
        assert_eq!(preferred_branding_width(580), 0);
        assert_eq!(preferred_branding_width(581), 220);
    }

    #[test]
    fn page_tabs_layout_returns_full_bounds_when_branding_is_hidden() {
        let bounds = Rect::new(10, 20, 560, 32);
        let (branding, tabs) = page_tabs_layout(bounds);
        assert_eq!(branding.x(), bounds.x());
        assert_eq!(branding.y(), bounds.y());
        assert_eq!(branding.height(), bounds.height());
        assert_eq!(tabs, bounds);
    }
}
