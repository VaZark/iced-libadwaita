//! Small color helpers mirroring the Sass/CSS functions libadwaita relies on
//! (`alpha()`, `mix()`, `rgba()`).

use iced::Color;

/// Builds an opaque color from a `0xRRGGBB` literal.
pub fn hex(rgb: u32) -> Color {
    Color::from_rgb8((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8)
}

/// Scales the alpha of `color` by `a`, exactly like GTK's `alpha()` and CSS
/// `color-mix(in srgb, color a%, transparent)`.
///
/// This multiplies: libadwaita's foreground is `rgb(0 0 6 / 80%)` in the light
/// style, so `alpha(fg, 0.2)` is 16% effective, not 20%. Use [`with_alpha`] to
/// set an absolute alpha instead.
pub fn alpha(color: Color, a: f32) -> Color {
    Color {
        a: color.a * a,
        ..color
    }
}

/// Returns `color` with its alpha *replaced* by `a`.
pub fn with_alpha(color: Color, a: f32) -> Color {
    Color { a, ..color }
}

/// Multiplies the alpha of `color` by `factor`.
pub fn fade(color: Color, factor: f32) -> Color {
    Color {
        a: color.a * factor,
        ..color
    }
}

/// Linear mix of two colors; `t = 0.0` gives `a`, `t = 1.0` gives `b`
/// (like GTK's `mix(a, b, t)`).
pub fn mix(a: Color, b: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    Color {
        r: a.r + (b.r - a.r) * t,
        g: a.g + (b.g - a.g) * t,
        b: a.b + (b.b - a.b) * t,
        a: a.a + (b.a - a.a) * t,
    }
}

/// Mixes `color` towards white.
pub fn lighten(color: Color, t: f32) -> Color {
    mix(color, Color::WHITE, t)
}

/// Mixes `color` towards black.
pub fn darken(color: Color, t: f32) -> Color {
    mix(color, Color::BLACK, t)
}

/// Black with the given alpha (`rgba(0, 0, 0, a)`).
pub fn black(a: f32) -> Color {
    Color { a, ..Color::BLACK }
}

/// White with the given alpha (`rgba(255, 255, 255, a)`).
pub fn white(a: f32) -> Color {
    Color { a, ..Color::WHITE }
}

/// libadwaita's "ink": `rgb(0 0 6 / a)`, the near-black used for foregrounds
/// and shades in the light style.
pub fn ink(a: f32) -> Color {
    Color::from_rgba8(0, 0, 6, a)
}

/// WCAG 2 relative luminance of an sRGB color (alpha is ignored).
pub fn relative_luminance(c: Color) -> f32 {
    let lin = |v: f32| {
        if v <= 0.03928 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * lin(c.r) + 0.7152 * lin(c.g) + 0.0722 * lin(c.b)
}

/// WCAG 2 contrast ratio between two colors, `1.0 ..= 21.0`.
pub fn contrast_ratio(a: Color, b: Color) -> f32 {
    let (la, lb) = (relative_luminance(a), relative_luminance(b));
    (la.max(lb) + 0.05) / (la.min(lb) + 0.05)
}

/// CSS Color Module Level 5 `color-contrast(bg vs candidates…)`: the candidate with the
/// highest contrast ratio against `bg` (the first one on a tie). Falls back to `bg` itself
/// for an empty list.
pub fn color_contrast(bg: Color, candidates: &[Color]) -> Color {
    let mut best: Option<(Color, f32)> = None;
    for &c in candidates {
        let r = contrast_ratio(bg, c);
        if best.is_none_or(|(_, b)| r > b) {
            best = Some((c, r));
        }
    }
    best.map_or(bg, |(c, _)| c)
}

/// `color-contrast(bg vs candidates… to target)`: the first candidate whose contrast ratio
/// reaches `target` (4.5 is WCAG AA for body text, 7.0 is AAA, 3.0 for large text); when none
/// does, the one with the highest ratio.
pub fn color_contrast_to(bg: Color, candidates: &[Color], target: f32) -> Color {
    candidates
        .iter()
        .copied()
        .find(|&c| contrast_ratio(bg, c) >= target)
        .unwrap_or_else(|| color_contrast(bg, candidates))
}

/// The light text color used on dark fills.
pub const LIGHT_TEXT: Color = Color::WHITE;

/// The dark text color used on light fills (libadwaita's near-black ink).
pub const DARK_TEXT: Color = Color {
    r: 0.114,
    g: 0.114,
    b: 0.133,
    a: 1.0,
};

/// The default text color for `bg`: [`LIGHT_TEXT`] or [`DARK_TEXT`], whichever has the higher
/// contrast ratio (`color-contrast(bg vs white, ink)`). Components use this for text and icons
/// on a colored fill unless the caller overrides it.
pub fn readable_on(bg: Color) -> Color {
    color_contrast(bg, &[LIGHT_TEXT, DARK_TEXT])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alpha_multiplies_like_gtk() {
        let fg = ink(0.8);
        assert!((alpha(fg, 0.2).a - 0.16).abs() < 1e-6);
        assert_eq!(alpha(Color::WHITE, 0.5).a, 0.5);
        assert_eq!(with_alpha(fg, 0.2).a, 0.2);
    }

    #[test]
    fn contrast_ratio_matches_wcag() {
        assert!((contrast_ratio(Color::WHITE, Color::BLACK) - 21.0).abs() < 0.01);
        assert!((contrast_ratio(Color::WHITE, Color::WHITE) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn readable_on_picks_the_higher_contrast() {
        assert_eq!(
            readable_on(hex(0x1c71d8)),
            LIGHT_TEXT,
            "dark blue gets light text"
        );
        assert_eq!(
            readable_on(hex(0xf6d32d)),
            DARK_TEXT,
            "yellow gets dark text"
        );
        assert_eq!(readable_on(Color::WHITE), DARK_TEXT);
        assert_eq!(readable_on(Color::BLACK), LIGHT_TEXT);
    }

    #[test]
    fn color_contrast_to_takes_the_first_that_passes() {
        let bg = hex(0x808080);
        let list = [hex(0x909090), Color::BLACK, Color::WHITE];
        // The first candidate is too close; black is the first to reach AA.
        assert_eq!(color_contrast_to(bg, &list, 4.5), Color::BLACK);
        // Nothing reaches 30: fall back to the highest ratio.
        assert_eq!(
            color_contrast_to(bg, &list, 30.0),
            color_contrast(bg, &list)
        );
    }
}
