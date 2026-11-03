//! Headless logic of a color picker: one color as hue, saturation, value and alpha, its
//! conversions and its text forms. After Mantine's
//! [ColorPicker](https://mantine.dev/core/color-picker/) and `ColorInput`.
//!
//! The color is kept as [`Hsva`] rather than RGB because that is what the picker's controls edit:
//! the hue slider changes `h`, the square changes `s` and `v`, the alpha slider `a`. Keeping hue
//! apart means a gray or a black does not lose its hue while you drag across it.
//!
//! * [`Hsva`]: the value, with `from_rgb8` / `to_rgb8`, HSL, and the edits a control makes
//!   ([`with_hue`](Hsva::with_hue), [`at`](Hsva::at), [`with_alpha`](Hsva::with_alpha));
//! * [`Format`] and [`Hsva::format`]: hex, hex with alpha, rgb, rgba, hsl, hsla, hsv, hsva;
//! * [`Hsva::parse`]: any of those, plus the short hex forms, as typed in a field.
//!
//! Pure: no iced, no colors of a theme.

/// A color as hue (degrees, `0..360`), saturation, value and alpha (`0..=1`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hsva {
    pub h: f32,
    pub s: f32,
    pub v: f32,
    pub a: f32,
}

/// The text forms of a color.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Format {
    /// `#rrggbb`.
    #[default]
    Hex,
    /// `#rrggbbaa`.
    Hexa,
    /// `rgb(r, g, b)`.
    Rgb,
    /// `rgba(r, g, b, a)`.
    Rgba,
    /// `hsl(h, s%, l%)`.
    Hsl,
    /// `hsla(h, s%, l%, a)`.
    Hsla,
    /// `hsv(h, s%, v%)`.
    Hsv,
    /// `hsva(h, s%, v%, a)`.
    Hsva,
}

impl Format {
    pub const ALL: [Format; 8] = [
        Format::Hex,
        Format::Hexa,
        Format::Rgb,
        Format::Rgba,
        Format::Hsl,
        Format::Hsla,
        Format::Hsv,
        Format::Hsva,
    ];

    /// Whether the text form carries alpha.
    pub fn has_alpha(self) -> bool {
        matches!(self, Self::Hexa | Self::Rgba | Self::Hsla | Self::Hsva)
    }
}

fn wrap_hue(h: f32) -> f32 {
    let h = h % 360.0;
    if h < 0.0 { h + 360.0 } else { h }
}

fn unit(x: f32) -> f32 {
    x.clamp(0.0, 1.0)
}

fn byte(x: f32) -> u8 {
    (unit(x) * 255.0).round() as u8
}

impl Hsva {
    pub fn new(h: f32, s: f32, v: f32, a: f32) -> Self {
        Self {
            h: wrap_hue(h),
            s: unit(s),
            v: unit(v),
            a: unit(a),
        }
    }

    pub const BLACK: Self = Self {
        h: 0.0,
        s: 0.0,
        v: 0.0,
        a: 1.0,
    };

    // -- RGB

    /// From red, green and blue in `0..=1`.
    pub fn from_rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        let (r, g, b) = (unit(r), unit(g), unit(b));
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let d = max - min;
        let h = if d == 0.0 {
            0.0
        } else if max == r {
            60.0 * (((g - b) / d) % 6.0)
        } else if max == g {
            60.0 * ((b - r) / d + 2.0)
        } else {
            60.0 * ((r - g) / d + 4.0)
        };
        Self::new(h, if max == 0.0 { 0.0 } else { d / max }, max, a)
    }
    /// From 8-bit red, green and blue (opaque).
    pub fn from_rgb8(r: u8, g: u8, b: u8) -> Self {
        Self::from_rgba(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0)
    }
    /// Red, green, blue and alpha in `0..=1`.
    pub fn to_rgba(self) -> (f32, f32, f32, f32) {
        let c = self.v * self.s;
        let x = c * (1.0 - ((self.h / 60.0) % 2.0 - 1.0).abs());
        let m = self.v - c;
        let (r, g, b) = match (self.h / 60.0) as u32 {
            0 => (c, x, 0.0),
            1 => (x, c, 0.0),
            2 => (0.0, c, x),
            3 => (0.0, x, c),
            4 => (x, 0.0, c),
            _ => (c, 0.0, x),
        };
        (r + m, g + m, b + m, self.a)
    }
    /// 8-bit red, green and blue.
    pub fn to_rgb8(self) -> [u8; 3] {
        let (r, g, b, _) = self.to_rgba();
        [byte(r), byte(g), byte(b)]
    }

    // -- HSL

    /// Hue, saturation and lightness (`0..=1`) of the same color.
    pub fn to_hsl(self) -> (f32, f32, f32) {
        let l = self.v * (1.0 - self.s / 2.0);
        let s = if l == 0.0 || l == 1.0 {
            0.0
        } else {
            (self.v - l) / l.min(1.0 - l)
        };
        (self.h, s, l)
    }
    pub fn from_hsl(h: f32, s: f32, l: f32, a: f32) -> Self {
        let (s, l) = (unit(s), unit(l));
        let v = l + s * l.min(1.0 - l);
        let sv = if v == 0.0 { 0.0 } else { 2.0 * (1.0 - l / v) };
        Self::new(h, sv, v, a)
    }

    // -- the edits a control makes

    pub fn with_hue(self, h: f32) -> Self {
        Self::new(h, self.s, self.v, self.a)
    }
    pub fn with_alpha(self, a: f32) -> Self {
        Self::new(self.h, self.s, self.v, a)
    }
    /// The color at a point of the saturation / value square: `x` across (saturation) and `y`
    /// down (value falls from the top), both `0..=1`.
    pub fn at(self, x: f32, y: f32) -> Self {
        Self::new(self.h, unit(x), 1.0 - unit(y), self.a)
    }
    /// Where this color is in the square: `(x, y)` in `0..=1`.
    pub fn position(self) -> (f32, f32) {
        (self.s, 1.0 - self.v)
    }

    // -- text

    /// The color as text in `format`.
    pub fn format(self, format: Format) -> String {
        let [r, g, b] = self.to_rgb8();
        let a = round2(self.a);
        let (h, s, l) = self.to_hsl();
        let pct = |x: f32| (x * 100.0).round() as i32;
        let h = h.round() as i32 % 360;
        match format {
            Format::Hex => format!("#{r:02x}{g:02x}{b:02x}"),
            Format::Hexa => format!("#{r:02x}{g:02x}{b:02x}{:02x}", byte(self.a)),
            Format::Rgb => format!("rgb({r}, {g}, {b})"),
            Format::Rgba => format!("rgba({r}, {g}, {b}, {a})"),
            Format::Hsl => format!("hsl({h}, {}%, {}%)", pct(s), pct(l)),
            Format::Hsla => format!("hsla({h}, {}%, {}%, {a})", pct(s), pct(l)),
            Format::Hsv => format!("hsv({h}, {}%, {}%)", pct(self.s), pct(self.v)),
            Format::Hsva => format!("hsva({h}, {}%, {}%, {a})", pct(self.s), pct(self.v)),
        }
    }

    /// Reads a color typed in any of the forms [`format`](Self::format) writes, and the short
    /// hex forms `#rgb` and `#rgba`. The leading `#` of a hex color may be left out.
    pub fn parse(text: &str) -> Option<Self> {
        let t = text.trim().to_lowercase();
        if let Some(args) = function(&t, &["rgba", "rgb"]) {
            let (r, g, b) = (
                channel(args.first()?)?,
                channel(args.get(1)?)?,
                channel(args.get(2)?)?,
            );
            let a = args.get(3).map_or(Some(1.0), |a| alpha(a))?;
            return Some(Self::from_rgba(r, g, b, a));
        }
        if let Some(args) = function(&t, &["hsla", "hsl"]) {
            let (h, s, l) = (
                hue(args.first()?)?,
                percent(args.get(1)?)?,
                percent(args.get(2)?)?,
            );
            let a = args.get(3).map_or(Some(1.0), |a| alpha(a))?;
            return Some(Self::from_hsl(h, s, l, a));
        }
        if let Some(args) = function(&t, &["hsva", "hsv"]) {
            let (h, s, v) = (
                hue(args.first()?)?,
                percent(args.get(1)?)?,
                percent(args.get(2)?)?,
            );
            let a = args.get(3).map_or(Some(1.0), |a| alpha(a))?;
            return Some(Self::new(h, s, v, a));
        }
        hex(t.trim_start_matches('#'))
    }
}

fn round2(x: f32) -> f32 {
    (x * 100.0).round() / 100.0
}

/// The arguments of `name(a, b, c)` for any of `names` (longest first), if `t` is one.
fn function(t: &str, names: &[&str]) -> Option<Vec<String>> {
    names.iter().find_map(|n| {
        let rest = t.strip_prefix(n)?.trim_start().strip_prefix('(')?;
        let inner = rest.strip_suffix(')')?;
        Some(inner.split(',').map(|a| a.trim().to_string()).collect())
    })
}

/// An rgb channel: `0..=255`, or a percentage.
fn channel(s: &str) -> Option<f32> {
    match s.strip_suffix('%') {
        Some(p) => Some(unit(p.trim().parse::<f32>().ok()? / 100.0)),
        None => Some(unit(s.parse::<f32>().ok()? / 255.0)),
    }
}

fn alpha(s: &str) -> Option<f32> {
    match s.strip_suffix('%') {
        Some(p) => Some(unit(p.trim().parse::<f32>().ok()? / 100.0)),
        None => Some(unit(s.parse::<f32>().ok()?)),
    }
}

fn hue(s: &str) -> Option<f32> {
    Some(wrap_hue(
        s.trim_end_matches("deg").trim().parse::<f32>().ok()?,
    ))
}

fn percent(s: &str) -> Option<f32> {
    Some(unit(
        s.strip_suffix('%')?.trim().parse::<f32>().ok()? / 100.0,
    ))
}

fn hex(h: &str) -> Option<Hsva> {
    if !h.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let nib = |c: char| c.to_digit(16).map(|d| (d * 17) as u8);
    let pair = |s: &str| u8::from_str_radix(s, 16).ok();
    let (r, g, b, a) = match h.len() {
        3 | 4 => {
            let c: Vec<char> = h.chars().collect();
            (
                nib(c[0])?,
                nib(c[1])?,
                nib(c[2])?,
                c.get(3).map_or(Some(255), |&c| nib(c))?,
            )
        }
        6 | 8 => (
            pair(&h[0..2])?,
            pair(&h[2..4])?,
            pair(&h[4..6])?,
            h.get(6..8).map_or(Some(255), pair)?,
        ),
        _ => return None,
    };
    Some(Hsva::from_rgba(
        r as f32 / 255.0,
        g as f32 / 255.0,
        b as f32 / 255.0,
        a as f32 / 255.0,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn near(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-3
    }

    #[test]
    fn the_primary_colors_have_the_expected_hues() {
        assert_eq!(Hsva::from_rgb8(255, 0, 0).h, 0.0);
        assert!(near(Hsva::from_rgb8(0, 255, 0).h, 120.0));
        assert!(near(Hsva::from_rgb8(0, 0, 255).h, 240.0));
        assert!(near(Hsva::from_rgb8(255, 255, 0).h, 60.0));
        let white = Hsva::from_rgb8(255, 255, 255);
        assert!(near(white.s, 0.0) && near(white.v, 1.0));
        assert_eq!(Hsva::from_rgb8(0, 0, 0).v, 0.0);
    }

    #[test]
    fn every_8_bit_color_round_trips_through_hsv() {
        for r in (0..=255).step_by(17) {
            for g in (0..=255).step_by(17) {
                for b in (0..=255).step_by(17) {
                    let c = Hsva::from_rgb8(r as u8, g as u8, b as u8);
                    assert_eq!(c.to_rgb8(), [r as u8, g as u8, b as u8], "{r} {g} {b}");
                }
            }
        }
        // And a spread of ones that are not on the lattice.
        for (r, g, b) in [(12, 200, 77), (255, 1, 254), (3, 3, 4), (128, 64, 32)] {
            assert_eq!(Hsva::from_rgb8(r, g, b).to_rgb8(), [r, g, b]);
        }
    }

    #[test]
    fn hsl_agrees_with_hsv() {
        let (h, s, l) = Hsva::from_rgb8(255, 0, 0).to_hsl();
        assert!(near(h, 0.0) && near(s, 1.0) && near(l, 0.5));
        let (_, s, l) = Hsva::from_rgb8(255, 255, 255).to_hsl();
        assert!(near(s, 0.0) && near(l, 1.0));
        let back = Hsva::from_hsl(120.0, 1.0, 0.5, 1.0);
        assert_eq!(back.to_rgb8(), [0, 255, 0]);
        let c = Hsva::from_rgb8(200, 100, 50);
        let (h, s, l) = c.to_hsl();
        assert_eq!(Hsva::from_hsl(h, s, l, 1.0).to_rgb8(), [200, 100, 50]);
    }

    #[test]
    fn a_control_edits_one_channel_and_keeps_the_rest() {
        let c = Hsva::new(200.0, 0.5, 0.6, 0.8);
        assert_eq!(c.with_hue(30.0), Hsva::new(30.0, 0.5, 0.6, 0.8));
        assert_eq!(c.with_alpha(0.1).h, 200.0);
        let p = c.at(0.25, 0.1);
        assert!(near(p.s, 0.25) && near(p.v, 0.9) && p.h == 200.0 && p.a == 0.8);
        assert_eq!(c.at(5.0, -3.0), c.at(1.0, 0.0), "positions are clamped");
        let (x, y) = c.position();
        assert!(near(x, 0.5) && near(y, 0.4));
    }

    #[test]
    fn hue_is_kept_while_the_color_is_gray_or_black() {
        let c = Hsva::new(210.0, 0.7, 0.8, 1.0);
        assert_eq!(c.at(0.0, 0.5).h, 210.0, "no saturation: still the same hue");
        assert_eq!(c.at(0.5, 1.0).h, 210.0, "no value: still the same hue");
    }

    #[test]
    fn hues_wrap_into_a_turn() {
        assert!(near(Hsva::new(370.0, 1.0, 1.0, 1.0).h, 10.0));
        assert!(near(Hsva::new(-30.0, 1.0, 1.0, 1.0).h, 330.0));
    }

    #[test]
    fn text_forms_are_written_as_css() {
        let red = Hsva::from_rgb8(255, 0, 0);
        assert_eq!(red.format(Format::Hex), "#ff0000");
        assert_eq!(red.format(Format::Hexa), "#ff0000ff");
        assert_eq!(red.format(Format::Rgb), "rgb(255, 0, 0)");
        assert_eq!(
            red.with_alpha(0.5).format(Format::Rgba),
            "rgba(255, 0, 0, 0.5)"
        );
        assert_eq!(red.format(Format::Hsl), "hsl(0, 100%, 50%)");
        assert_eq!(red.format(Format::Hsv), "hsv(0, 100%, 100%)");
        assert_eq!(
            red.with_alpha(0.25).format(Format::Hsva),
            "hsva(0, 100%, 100%, 0.25)"
        );
        assert_eq!(red.with_alpha(0.0).format(Format::Hexa), "#ff000000");
        assert!(Format::Hexa.has_alpha() && !Format::Hex.has_alpha());
    }

    #[test]
    fn every_text_form_reads_back_to_the_same_color() {
        let c = Hsva::from_rgb8(30, 144, 255).with_alpha(0.5);
        for f in Format::ALL {
            let text = c.format(f);
            let back = Hsva::parse(&text).unwrap_or_else(|| panic!("{text} does not parse"));
            let (a, b) = (c.to_rgb8(), back.to_rgb8());
            let tolerance = if matches!(f, Format::Hsl | Format::Hsla | Format::Hsv | Format::Hsva)
            {
                3
            } else {
                0
            };
            for i in 0..3 {
                assert!(
                    (a[i] as i32 - b[i] as i32).abs() <= tolerance,
                    "{text}: {a:?} vs {b:?}"
                );
            }
            if f.has_alpha() {
                assert!((back.a - c.a).abs() < 0.01, "{text}");
            }
        }
    }

    #[test]
    fn typed_forms_are_read_liberally() {
        assert_eq!(Hsva::parse("#f00").unwrap().to_rgb8(), [255, 0, 0]);
        assert_eq!(Hsva::parse("F00").unwrap().to_rgb8(), [255, 0, 0]);
        assert_eq!(Hsva::parse("  #1E90FF ").unwrap().to_rgb8(), [30, 144, 255]);
        assert!(near(Hsva::parse("#f008").unwrap().a, 0x88 as f32 / 255.0));
        assert_eq!(Hsva::parse("rgb(0,255,0)").unwrap().to_rgb8(), [0, 255, 0]);
        assert_eq!(
            Hsva::parse("rgb(100%, 0%, 0%)").unwrap().to_rgb8(),
            [255, 0, 0]
        );
        assert!(near(Hsva::parse("rgba(0, 0, 0, 50%)").unwrap().a, 0.5));
        assert_eq!(
            Hsva::parse("hsl(240, 100%, 50%)").unwrap().to_rgb8(),
            [0, 0, 255]
        );
        assert!(near(Hsva::parse("hsv(90deg, 100%, 100%)").unwrap().h, 90.0));
    }

    #[test]
    fn nonsense_is_not_a_color() {
        for bad in [
            "",
            "#",
            "#12",
            "#12345",
            "#ggg",
            "rgb(1, 2)",
            "rgb(a, b, c)",
            "hsl(0, 1, 2)",
            "blue",
            "#ff00ff00ff",
        ] {
            assert!(Hsva::parse(bad).is_none(), "{bad:?} should not parse");
        }
    }
}
