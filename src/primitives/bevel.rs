//! Bevel geometry: which edge of a frame is which tone, with no drawing.
//!
//! A bevel is one or two 1px rings around a rectangle. Each ring has a light edge (top and
//! left) and a dark edge (bottom and right), and a style says which of four tones each edge is:
//!
//! ```text
//! raised, 2 rings       sunken, 2 rings
//! L L L L L L L D       d d d d d d d L
//! L l l l l l d D       d D D D D D l L
//! L l         d D       d D         l L
//! L l         d D       d D         l L
//! L l d d d d d D       d D l l l l l L
//! D D D D D D D D       L L L L L L L L
//!
//! L light (border-light)        l light-inner (border-light-inner)
//! D dark (border-dark)          d dark-inner (border-dark-inner)
//! ```
//!
//! [`tones`] has the table for every style. With one ring only the outer one is drawn.

use iced::{Rectangle, Size};

/// A drawn border style (`border-style` other than `solid`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BevelStyle {
    /// Sticks out: light top-left, dark bottom-right (a button).
    Raised,
    /// Sinks in: the opposite (a text field).
    Sunken,
    /// A groove: a dark line with a light one inside, as a group box.
    Etched,
    /// A ridge: the groove reversed.
    Bump,
    /// A window frame: a light outer ring and a bright inner one.
    Window,
}

impl BevelStyle {
    /// The style a `border-style` value names; `None` for `solid` (the toolkit's own border) and
    /// for anything unknown.
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "raised" => Some(Self::Raised),
            "sunken" => Some(Self::Sunken),
            "etched" => Some(Self::Etched),
            "bump" => Some(Self::Bump),
            "window" => Some(Self::Window),
            _ => None,
        }
    }

    /// The style while pressed: a raised bevel sinks and a sunken one rises; others are as is.
    pub fn pressed(self) -> Self {
        match self {
            Self::Raised | Self::Window => Self::Sunken,
            Self::Sunken => Self::Raised,
            other => other,
        }
    }
}

/// The color roles of a bevel, the theme's `border-*` keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tone {
    /// `border-light`.
    Light,
    /// `border-dark`.
    Dark,
    /// `border-light-inner`.
    LightInner,
    /// `border-dark-inner`.
    DarkInner,
}

/// One drawn edge: a rectangle (relative to the frame's top-left) and its tone.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Edge {
    pub rect: Rectangle,
    pub tone: Tone,
}

/// How many 1px rings a `border-width` draws: 1 below 1.5px, else 2.
pub fn rings(border_width: f32) -> u8 {
    if border_width < 1.5 { 1 } else { 2 }
}

/// The `(top-left, bottom-right)` tones of ring `ring` (0 outer, 1 inner) of `style`.
pub fn tones(style: BevelStyle, ring: u8) -> (Tone, Tone) {
    use Tone::*;
    match (style, ring) {
        (BevelStyle::Raised, 0) => (Light, Dark),
        (BevelStyle::Raised, _) => (LightInner, DarkInner),
        (BevelStyle::Sunken, 0) => (DarkInner, Light),
        (BevelStyle::Sunken, _) => (Dark, LightInner),
        (BevelStyle::Etched, 0) => (DarkInner, Light),
        (BevelStyle::Etched, _) => (Light, DarkInner),
        (BevelStyle::Bump, 0) => (Light, DarkInner),
        (BevelStyle::Bump, _) => (DarkInner, Light),
        (BevelStyle::Window, 0) => (LightInner, Dark),
        (BevelStyle::Window, _) => (Light, DarkInner),
    }
}

/// The edges of a frame of `size`: per ring, a top and a left line in the top-left tone and a
/// bottom and a right line in the bottom-right tone. The bottom and right lines run the full
/// length, so the top-right and bottom-left corner pixels are the bottom-right tone, as in the
/// classic frames.
pub fn edges(style: BevelStyle, rings: u8, size: Size) -> Vec<Edge> {
    let (w, h) = (size.width, size.height);
    let mut out = Vec::new();
    for ring in 0..rings.min(2) {
        let k = ring as f32;
        let (tl, br) = tones(style, ring);
        let (iw, ih) = (w - 2.0 * k, h - 2.0 * k);
        if iw <= 1.0 || ih <= 1.0 {
            break;
        }
        let rect = |x: f32, y: f32, w: f32, h: f32| Rectangle::new(iced::Point::new(x, y), Size::new(w, h));
        out.push(Edge { rect: rect(k, k, iw - 1.0, 1.0), tone: tl });
        out.push(Edge { rect: rect(k, k, 1.0, ih - 1.0), tone: tl });
        out.push(Edge { rect: rect(k, h - k - 1.0, iw, 1.0), tone: br });
        out.push(Edge { rect: rect(w - k - 1.0, k, 1.0, ih), tone: br });
    }
    out
}

/// The tone of the pixel at `(x, y)` in a frame of `size`, or `None` inside the frame. For tests
/// and hit checks.
pub fn tone_at(style: BevelStyle, rings: u8, size: Size, x: f32, y: f32) -> Option<Tone> {
    // Later rings are inside earlier ones, so the first match is the pixel's.
    edges(style, rings, size)
        .into_iter()
        .find(|e| e.rect.contains(iced::Point::new(x + 0.5, y + 0.5)))
        .map(|e| e.tone)
}

#[cfg(test)]
mod tests {
    use super::*;

    const S: Size = Size { width: 10.0, height: 8.0 };

    #[test]
    fn parse_names_the_drawn_styles_only() {
        assert_eq!(BevelStyle::parse("raised"), Some(BevelStyle::Raised));
        assert_eq!(BevelStyle::parse("solid"), None);
        assert_eq!(BevelStyle::parse("groove"), None);
    }

    #[test]
    fn one_ring_is_light_top_left_and_dark_bottom_right_when_raised() {
        let t = |x, y| tone_at(BevelStyle::Raised, 1, S, x, y);
        assert_eq!(t(0.0, 0.0), Some(Tone::Light));
        assert_eq!(t(4.0, 0.0), Some(Tone::Light));
        assert_eq!(t(0.0, 4.0), Some(Tone::Light));
        // The full-length bottom and right lines own the other two corners.
        assert_eq!(t(9.0, 0.0), Some(Tone::Dark));
        assert_eq!(t(0.0, 7.0), Some(Tone::Dark));
        assert_eq!(t(9.0, 7.0), Some(Tone::Dark));
        assert_eq!(t(5.0, 7.0), Some(Tone::Dark));
        // Inside the ring there is nothing at width 1.
        assert_eq!(t(1.0, 1.0), None);
    }

    #[test]
    fn two_rings_have_an_inner_pair_of_tones() {
        let t = |style, x, y| tone_at(style, 2, S, x, y);
        assert_eq!(t(BevelStyle::Raised, 1.0, 1.0), Some(Tone::LightInner));
        assert_eq!(t(BevelStyle::Raised, 8.0, 6.0), Some(Tone::DarkInner));
        assert_eq!(t(BevelStyle::Raised, 0.0, 0.0), Some(Tone::Light));
        assert_eq!(t(BevelStyle::Sunken, 0.0, 0.0), Some(Tone::DarkInner));
        assert_eq!(t(BevelStyle::Sunken, 1.0, 1.0), Some(Tone::Dark));
        assert_eq!(t(BevelStyle::Sunken, 9.0, 7.0), Some(Tone::Light));
        assert_eq!(t(BevelStyle::Sunken, 8.0, 6.0), Some(Tone::LightInner));
        // Nothing inside two rings.
        assert_eq!(t(BevelStyle::Raised, 2.0, 2.0), None);
    }

    #[test]
    fn etched_and_bump_are_each_others_reverse() {
        let at = |style, x, y| tone_at(style, 2, S, x, y);
        assert_eq!(at(BevelStyle::Etched, 0.0, 0.0), Some(Tone::DarkInner));
        assert_eq!(at(BevelStyle::Etched, 1.0, 1.0), Some(Tone::Light));
        assert_eq!(at(BevelStyle::Bump, 0.0, 0.0), Some(Tone::Light));
        assert_eq!(at(BevelStyle::Bump, 1.0, 1.0), Some(Tone::DarkInner));
    }

    #[test]
    fn a_window_frame_is_a_light_gray_outer_ring_and_a_white_inner_one() {
        assert_eq!(tone_at(BevelStyle::Window, 2, S, 0.0, 0.0), Some(Tone::LightInner));
        assert_eq!(tone_at(BevelStyle::Window, 2, S, 1.0, 1.0), Some(Tone::Light));
        assert_eq!(tone_at(BevelStyle::Window, 2, S, 9.0, 7.0), Some(Tone::Dark));
    }

    #[test]
    fn pressing_swaps_raised_and_sunken_and_a_ring_count_follows_the_width() {
        assert_eq!(BevelStyle::Raised.pressed(), BevelStyle::Sunken);
        assert_eq!(BevelStyle::Sunken.pressed(), BevelStyle::Raised);
        assert_eq!(BevelStyle::Etched.pressed(), BevelStyle::Etched);
        assert_eq!((rings(0.5), rings(1.0), rings(2.0), rings(5.0)), (1, 1, 2, 2));
    }

    #[test]
    fn a_frame_too_small_for_a_ring_draws_nothing_for_it() {
        assert!(edges(BevelStyle::Raised, 2, Size::new(3.0, 3.0)).len() <= 4);
        assert!(edges(BevelStyle::Raised, 1, Size::new(1.0, 1.0)).is_empty());
    }
}
