//! Pure screen-geometry helpers for the floating widget, kept free of Tauri
//! types so they can be unit-tested without a window system.

/// A rectangle in physical (device) pixels, origin at the top-left.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl Rect {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Self { x, y, w, h }
    }

    /// Whether the point lies inside the rectangle (right/bottom edges exclusive).
    pub fn contains(&self, px: f64, py: f64) -> bool {
        px >= f64::from(self.x)
            && px < f64::from(self.x + self.w)
            && py >= f64::from(self.y)
            && py < f64::from(self.y + self.h)
    }
}

/// How far the window must move left and up so a card of `size`, whose
/// top-left rests at `anchor`, stays inside `area` with `margin` to spare.
///
/// The shift never pushes the card past the top-left edge of the area, so a
/// card larger than the area simply pins to that corner.
pub fn expansion_shift(anchor: (i32, i32), size: (i32, i32), area: Rect, margin: i32) -> (i32, i32) {
    let overflow_x = (anchor.0 + size.0 - (area.x + area.w - margin)).max(0);
    let overflow_y = (anchor.1 + size.1 - (area.y + area.h - margin)).max(0);
    let room_x = (anchor.0 - (area.x + margin)).max(0);
    let room_y = (anchor.1 - (area.y + margin)).max(0);
    (overflow_x.min(room_x), overflow_y.min(room_y))
}

/// Whether a remembered anchor still lands on a connected display.
pub fn anchor_on_screen(anchor: (i32, i32), areas: &[Rect]) -> bool {
    areas
        .iter()
        .any(|a| a.contains(f64::from(anchor.0), f64::from(anchor.1)))
}

/// First-run placement: top-right corner of `area`, `inset` from the edges.
pub fn default_anchor(area: Rect, size: (i32, i32), inset: i32) -> (i32, i32) {
    (area.x + area.w - size.0 - inset, area.y + inset)
}

/// Debounces pointer enter/leave into expand/collapse decisions: entering
/// acts at once, leaving must hold for `grace_ticks` consecutive polls so a
/// pointer grazing the edge does not make the widget flicker.
#[derive(Debug)]
pub struct HoverTracker {
    grace_ticks: u8,
    outside_ticks: u8,
}

impl HoverTracker {
    pub fn new(grace_ticks: u8) -> Self {
        Self {
            grace_ticks,
            outside_ticks: 0,
        }
    }

    /// Feed one poll; returns `Some(expanded)` when the widget should change state.
    pub fn observe(&mut self, pointer_inside: bool, expanded: bool) -> Option<bool> {
        match (pointer_inside, expanded) {
            (true, false) => {
                self.outside_ticks = 0;
                Some(true)
            }
            (false, true) => {
                self.outside_ticks = self.outside_ticks.saturating_add(1);
                if self.outside_ticks >= self.grace_ticks {
                    self.outside_ticks = 0;
                    Some(false)
                } else {
                    None
                }
            }
            _ => {
                self.outside_ticks = 0;
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SCREEN: Rect = Rect {
        x: 0,
        y: 0,
        w: 2880,
        h: 1800,
    };

    #[test]
    fn contains_is_half_open() {
        let r = Rect::new(10, 20, 100, 50);
        assert!(r.contains(10.0, 20.0));
        assert!(r.contains(109.9, 69.9));
        assert!(!r.contains(110.0, 30.0));
        assert!(!r.contains(50.0, 70.0));
        assert!(!r.contains(9.9, 30.0));
    }

    #[test]
    fn no_shift_when_the_card_fits() {
        assert_eq!(expansion_shift((200, 200), (672, 448), SCREEN, 24), (0, 0));
    }

    #[test]
    fn shifts_left_and_up_by_the_overflow() {
        // Anchor 100px from the right edge; card is 672 wide, so it overflows by 672-100+24.
        let anchor = (SCREEN.w - 100, 100);
        assert_eq!(expansion_shift(anchor, (672, 448), SCREEN, 24), (596, 0));
        // Near the bottom edge it shifts up.
        let anchor = (200, SCREEN.h - 100);
        assert_eq!(expansion_shift(anchor, (672, 448), SCREEN, 24), (0, 372));
    }

    #[test]
    fn shift_never_pushes_past_the_top_left() {
        let tiny = Rect::new(0, 0, 400, 300);
        // Card bigger than the area: pins to the margin instead of going off-screen.
        assert_eq!(expansion_shift((50, 40), (672, 448), tiny, 24), (26, 16));
    }

    #[test]
    fn respects_a_secondary_monitor_origin() {
        let left_of_primary = Rect::new(-1920, 0, 1920, 1080);
        let anchor = (-60, 10);
        // Right edge of that monitor is x=0; card needs 336+24 clear.
        assert_eq!(expansion_shift(anchor, (336, 224), left_of_primary, 12), (288, 0));
    }

    #[test]
    fn saved_anchor_must_be_on_a_connected_display() {
        let areas = [SCREEN, Rect::new(2880, 0, 1920, 1080)];
        assert!(anchor_on_screen((100, 100), &areas));
        assert!(anchor_on_screen((3000, 500), &areas));
        // A monitor that was unplugged leaves the anchor stranded.
        assert!(!anchor_on_screen((6000, 100), &areas));
        assert!(!anchor_on_screen((100, 100), &[]));
    }

    #[test]
    fn default_anchor_hugs_the_top_right() {
        assert_eq!(default_anchor(SCREEN, (424, 128), 32), (2424, 32));
    }

    #[test]
    fn hover_expands_immediately_and_collapses_after_the_grace_period() {
        let mut t = HoverTracker::new(3);
        assert_eq!(t.observe(false, false), None);
        assert_eq!(t.observe(true, false), Some(true));
        assert_eq!(t.observe(true, true), None);
        // Pointer leaves: two quiet polls, collapse on the third.
        assert_eq!(t.observe(false, true), None);
        assert_eq!(t.observe(false, true), None);
        assert_eq!(t.observe(false, true), Some(false));
    }

    #[test]
    fn a_brief_exit_does_not_collapse() {
        let mut t = HoverTracker::new(3);
        assert_eq!(t.observe(false, true), None);
        assert_eq!(t.observe(false, true), None);
        // Pointer returns before the grace period ends: the countdown restarts.
        assert_eq!(t.observe(true, true), None);
        assert_eq!(t.observe(false, true), None);
        assert_eq!(t.observe(false, true), None);
        assert_eq!(t.observe(false, true), Some(false));
    }
}
