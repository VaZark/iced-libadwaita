//! Input sections (Mantine's `leftSection` / `rightSection`): the arithmetic that keeps an
//! input's text clear of the elements drawn inside its border.
//!
//! A section is a slot of `width` px sitting `inset` px from the input's edge; the text area
//! starts after the slot plus a `gap`. No section leaves the input's own padding untouched.

use iced::Padding;

/// The slot width: the explicit one when set, else `default`. Never negative.
pub fn slot_width(explicit: Option<f32>, default: f32) -> f32 {
    explicit.unwrap_or(default).max(0.0)
}

/// Text padding of an input whose left and right sections have the given slot widths
/// (`None`: no section on that side, padding stays `base`).
pub fn text_padding(
    base: Padding,
    inset: f32,
    gap: f32,
    left: Option<f32>,
    right: Option<f32>,
) -> Padding {
    Padding {
        left: left.map_or(base.left, |w| inset + w + gap),
        right: right.map_or(base.right, |w| inset + w + gap),
        ..base
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_section_keeps_the_padding() {
        let base = Padding::from([6.0, 9.0]);
        assert_eq!(text_padding(base, 4.0, 4.0, None, None), base);
    }

    #[test]
    fn a_section_pushes_the_text_past_its_slot() {
        let base = Padding::from([6.0, 9.0]);
        let p = text_padding(base, 4.0, 4.0, Some(24.0), None);
        assert_eq!((p.left, p.right, p.top), (32.0, 9.0, 6.0));
        let p = text_padding(base, 4.0, 4.0, Some(24.0), Some(40.0));
        assert_eq!((p.left, p.right), (32.0, 48.0));
    }

    #[test]
    fn explicit_width_wins_and_is_clamped() {
        assert_eq!(slot_width(None, 24.0), 24.0);
        assert_eq!(slot_width(Some(40.0), 24.0), 40.0);
        assert_eq!(slot_width(Some(-3.0), 24.0), 0.0);
    }
}
