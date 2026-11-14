//! The gradient token type: a CSS-like `linear-gradient(<angle>deg, <color> <pos>%, ...)`.
//!
//! ```text
//! linear-gradient(90deg, #0a246a 0%, #a6caf0 100%)      left to right, navy to light blue
//! linear-gradient(180deg, #ffffff, #d4d0c8)              top to bottom, positions spread evenly
//! ```
//!
//! The angle follows CSS: `0deg` points up, `90deg` right, `180deg` down. Two to eight stops;
//! each is a color (any form a color token takes) with an optional `%` position. Positions that
//! are given must lie in `0..=100` and not decrease; a missing position is spread evenly between
//! its neighbours (the first defaults to 0%, the last to 100%).

use iced::{Background, Color};

/// The most stops iced draws.
pub const MAX_STOPS: usize = 8;

/// A validated linear gradient: an angle and two to eight stops, positions in `0..=1`.
#[derive(Debug, Clone, PartialEq)]
pub struct Gradient {
    /// CSS angle in degrees, normalized to `0..360`.
    pub angle: f32,
    /// `(position in 0..=1, color)`, positions non-decreasing.
    pub stops: Vec<(f32, Color)>,
}

impl Gradient {
    /// Parses `linear-gradient(<angle>deg, <color> [<pos>%], ...)`. `None` when the text is not
    /// exactly that: a bad angle, fewer than two or more than eight stops, a position outside
    /// `0..=100` or out of order.
    pub fn parse(raw: &str) -> Option<Self> {
        let raw = raw.trim();
        let inner = raw
            .strip_prefix("linear-gradient(")
            .or_else(|| raw.strip_prefix("LINEAR-GRADIENT("))?
            .strip_suffix(')')?;
        let mut parts = split_top_level(inner).into_iter();
        let angle = parts.next()?.trim().strip_suffix("deg")?.trim().parse::<f32>().ok()?;
        if !angle.is_finite() || !(-360.0..=360.0).contains(&angle) {
            return None;
        }
        let angle = angle.rem_euclid(360.0);
        let mut raw_stops: Vec<(Color, Option<f32>)> = Vec::new();
        for part in parts {
            let part = part.trim();
            let (color_text, pos) = match part.rsplit_once(char::is_whitespace) {
                Some((c, p)) if p.ends_with('%') => {
                    let n = p.trim_end_matches('%').trim().parse::<f32>().ok()?;
                    if !n.is_finite() || !(0.0..=100.0).contains(&n) {
                        return None;
                    }
                    (c.trim(), Some(n / 100.0))
                }
                _ => (part, None),
            };
            let color = super::variable::parse_color(color_text)?;
            raw_stops.push((color, pos));
        }
        if raw_stops.len() < 2 || raw_stops.len() > MAX_STOPS {
            return None;
        }
        // Fill the missing positions evenly between the ones given.
        let last = raw_stops.len() - 1;
        raw_stops[0].1.get_or_insert(0.0);
        raw_stops[last].1.get_or_insert(1.0);
        let mut i = 0;
        while i < raw_stops.len() {
            if raw_stops[i].1.is_some() {
                i += 1;
                continue;
            }
            let start = i - 1;
            let mut end = i;
            while raw_stops[end].1.is_none() {
                end += 1;
            }
            let (a, b) = (raw_stops[start].1?, raw_stops[end].1?);
            for k in start + 1..end {
                raw_stops[k].1 = Some(a + (b - a) * (k - start) as f32 / (end - start) as f32);
            }
            i = end + 1;
        }
        let stops: Vec<(f32, Color)> = raw_stops.into_iter().map(|(c, p)| (p.unwrap(), c)).collect();
        if stops.windows(2).any(|w| w[1].0 < w[0].0) {
            return None;
        }
        Some(Self { angle, stops })
    }

    /// The text this gradient is written as (parses back to an equal gradient).
    pub fn to_text(&self) -> String {
        let stops: Vec<String> = self
            .stops
            .iter()
            .map(|(p, c)| format!("{} {}%", crate::color::to_hex(*c), trim_float(p * 100.0)))
            .collect();
        format!("linear-gradient({}deg, {})", trim_float(self.angle), stops.join(", "))
    }

    /// The iced gradient. iced's angle is CSS's already: `0` points up and grows clockwise, so
    /// `180deg` is top to bottom.
    pub fn to_iced(&self) -> iced::Gradient {
        let radians = self.angle.to_radians();
        let mut g = iced::gradient::Linear::new(iced::Radians(radians));
        for (p, c) in &self.stops {
            g = g.add_stop(*p, *c);
        }
        iced::Gradient::Linear(g)
    }

    /// As a widget background.
    pub fn background(&self) -> Background {
        Background::Gradient(self.to_iced())
    }

    /// The gradient with `f` applied to each stop's color (derived hover and pressed fills).
    pub fn map(&self, f: impl Fn(Color) -> Color) -> Self {
        Self {
            angle: self.angle,
            stops: self.stops.iter().map(|(p, c)| (*p, f(*c))).collect(),
        }
    }

    /// The gradient with each color's alpha scaled by `k` (a disabled or translucent fill).
    pub fn faded(&self, k: f32) -> Self {
        Self {
            angle: self.angle,
            stops: self
                .stops
                .iter()
                .map(|(p, c)| (*p, Color { a: c.a * k, ..*c }))
                .collect(),
        }
    }

    /// The gradient with each color mixed `t` of the way towards `with` (hover and pressed
    /// fills derived from a base gradient).
    pub fn mixed(&self, with: Color, t: f32) -> Self {
        let t = t.clamp(0.0, 1.0);
        Self {
            angle: self.angle,
            stops: self
                .stops
                .iter()
                .map(|(p, c)| {
                    (
                        *p,
                        Color {
                            r: c.r + (with.r - c.r) * t,
                            g: c.g + (with.g - c.g) * t,
                            b: c.b + (with.b - c.b) * t,
                            a: c.a,
                        },
                    )
                })
                .collect(),
        }
    }
}

fn trim_float(v: f32) -> String {
    let s = format!("{:.2}", v);
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}

/// Splits on commas that are not inside parentheses.
fn split_top_level(text: &str) -> Vec<&str> {
    let (mut depth, mut start, mut out) = (0i32, 0, Vec::new());
    for (i, c) in text.char_indices() {
        match c {
            '(' => depth += 1,
            ')' => depth -= 1,
            ',' if depth == 0 => {
                out.push(&text[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    out.push(&text[start..]);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_stops_angles_and_spreads_missing_positions() {
        let g = Gradient::parse("linear-gradient(90deg, #0a246a 0%, #a6caf0 100%)").unwrap();
        assert_eq!(g.angle, 90.0);
        assert_eq!(g.stops.len(), 2);
        assert_eq!(g.stops[1].0, 1.0);
        let g = Gradient::parse("linear-gradient(180deg, #000, #888, #fff)").unwrap();
        assert_eq!(g.stops.iter().map(|s| s.0).collect::<Vec<_>>(), [0.0, 0.5, 1.0]);
        let g = Gradient::parse("linear-gradient(-90deg, #000 10%, #111, #fff 90%)").unwrap();
        assert_eq!(g.angle, 270.0);
        assert!((g.stops[1].0 - 0.5).abs() < 1e-6);
        // rgba() colors contain commas: they are not stop separators.
        assert!(Gradient::parse("linear-gradient(0deg, rgba(0, 0, 0, 0.5) 0%, #fff 100%)").is_some());
    }

    #[test]
    fn round_trips() {
        for text in [
            "linear-gradient(90deg, #0a246a 0%, #a6caf0 100%)",
            "linear-gradient(135deg, #ff000080 0%, #00ff00 33.5%, #0000ff 100%)",
        ] {
            let g = Gradient::parse(text).unwrap();
            assert_eq!(Gradient::parse(&g.to_text()), Some(g));
        }
    }

    #[test]
    fn rejects_what_is_not_valid() {
        for bad in [
            "",
            "#fff",
            "linear-gradient(90deg, #fff)",                       // one stop
            "linear-gradient(90, #000, #fff)",                    // no unit
            "linear-gradient(400deg, #000, #fff)",                // angle out of range
            "linear-gradient(nandeg, #000, #fff)",
            "linear-gradient(90deg, #000 60%, #fff 40%)",         // out of order
            "linear-gradient(90deg, #000 -5%, #fff 100%)",        // position out of range
            "linear-gradient(90deg, #000 0%, #fff 120%)",
            "linear-gradient(90deg, nothex 0%, #fff 100%)",
            "linear-gradient(90deg, #000, #111, #222, #333, #444, #555, #666, #777, #888)", // 9 stops
            "radial-gradient(90deg, #000, #fff)",
        ] {
            assert!(Gradient::parse(bad).is_none(), "{bad}");
        }
    }

    #[test]
    fn the_angle_maps_css_to_iced() {
        // iced's angle is CSS's: 0 points up, 90deg right, 180deg down (clockwise).
        for (css, rad) in [(0.0, 0.0), (90.0, std::f32::consts::FRAC_PI_2), (180.0, std::f32::consts::PI)] {
            let g = Gradient::parse(&format!("linear-gradient({css}deg, #000, #fff)")).unwrap();
            let iced::Gradient::Linear(l) = g.to_iced();
            assert!((l.angle.0 - rad).abs() < 1e-5, "{css}deg");
        }
        // And iced's own geometry agrees: 180deg runs from the top edge to the bottom edge.
        let g = Gradient::parse("linear-gradient(180deg, #000, #fff)").unwrap();
        let iced::Gradient::Linear(l) = g.to_iced();
        let (a, b) = l.angle.to_distance(&iced::Rectangle::new(iced::Point::ORIGIN, iced::Size::new(100.0, 40.0)));
        assert!(a.y < b.y && (a.x - b.x).abs() < 1e-3);
    }

    #[test]
    fn faded_and_mixed_keep_positions() {
        let g = Gradient::parse("linear-gradient(0deg, #ffffff, #000000)").unwrap();
        assert_eq!(g.faded(0.5).stops[0].1.a, 0.5);
        let m = g.mixed(Color::BLACK, 0.5);
        assert!((m.stops[0].1.r - 0.5).abs() < 1e-6);
        assert_eq!(m.stops[1].0, 1.0);
    }
}
