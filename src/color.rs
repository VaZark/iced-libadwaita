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

/// Tints `base` towards the color of `toward` by `t`, keeping the **alpha of `base`**: only the
/// hue moves, so a translucent surface stays as translucent after a hover or press tint (a
/// plain [`mix`] would drag its alpha towards the other color's).
pub fn tint(base: Color, toward: Color, t: f32) -> Color {
    Color {
        a: base.a,
        ..mix(base, toward, t)
    }
}

/// Tints `color` towards white (its alpha is kept).
pub fn lighten(color: Color, t: f32) -> Color {
    tint(color, Color::WHITE, t)
}

/// Tints `color` towards black (its alpha is kept).
pub fn darken(color: Color, t: f32) -> Color {
    tint(color, Color::BLACK, t)
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

/// Parses `#rgb`, `#rrggbb` or `#rrggbbaa` (the `#` is optional) into a color.
pub fn parse(text: &str) -> Option<Color> {
    let t = text.trim().trim_start_matches('#');
    let nibble = |i: usize| u8::from_str_radix(&t[i..=i], 16).ok().map(|v| v * 17);
    let byte = |i: usize| u8::from_str_radix(&t[i..i + 2], 16).ok();
    if !t.is_ascii() {
        return None;
    }
    let (r, g, b, a) = match t.len() {
        3 => (nibble(0)?, nibble(1)?, nibble(2)?, 255),
        6 => (byte(0)?, byte(2)?, byte(4)?, 255),
        8 => (byte(0)?, byte(2)?, byte(4)?, byte(6)?),
        _ => return None,
    };
    Some(Color::from_rgba8(r, g, b, a as f32 / 255.0))
}

/// The color as `#rrggbb`, or `#rrggbbaa` when it is not opaque.
pub fn to_hex(c: Color) -> String {
    let b = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    if c.a >= 1.0 {
        format!("#{:02x}{:02x}{:02x}", b(c.r), b(c.g), b(c.b))
    } else {
        format!("#{:02x}{:02x}{:02x}{:02x}", b(c.r), b(c.g), b(c.b), b(c.a))
    }
}

/// A color from the GNOME avatar palette for `index` (wraps). Stable: the same index is the
/// same color, so a list keeps its colors from one run to the next.
pub fn from_index(index: usize) -> Color {
    hex(AVATAR_COLORS[index % AVATAR_COLORS.len()])
}

/// A stable, theme-friendly color for any text: the same seed is always the same color, and
/// seeds spread over the avatar palette (a playlist's name, a user's id).
pub fn from_seed(seed: impl AsRef<str>) -> Color {
    let hash = seed
        .as_ref()
        .bytes()
        .fold(5381u32, |h, b| h.wrapping_mul(33).wrapping_add(b as u32));
    from_index(hash as usize)
}

/// The palette [`from_index`] and [`from_seed`] draw from.
const AVATAR_COLORS: [u32; 12] = [
    0x62a0ea, 0x57e389, 0xf8e45c, 0xffa348, 0xff7b63, 0xdc8add, 0xc061cb, 0x9a9996, 0x5e5c64,
    0x99c1f1, 0x8ff0a4, 0xf66151,
];

/// `fg` moved towards white or black, whichever the `bg` calls for, until it has at least
/// `min_ratio` of contrast against `bg` (4.5 is the WCAG minimum for text). Colors that already
/// pass are returned unchanged, so a light tag color stays itself on a light page and only a color
/// too close to its surface is pulled away from it.
pub fn ensure_contrast(fg: Color, bg: Color, min_ratio: f32) -> Color {
    if contrast_ratio(fg, bg) >= min_ratio {
        return fg;
    }
    let target = if relative_luminance(bg) > 0.18 {
        Color::BLACK
    } else {
        Color::WHITE
    };
    for step in 1..=20 {
        let c = Color {
            a: fg.a,
            ..mix(fg, target, step as f32 / 20.0)
        };
        if contrast_ratio(c, bg) >= min_ratio {
            return c;
        }
    }
    Color { a: fg.a, ..target }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colors_parse_and_print_as_hex() {
        assert_eq!(to_hex(parse("#3584e4").unwrap()), "#3584e4");
        assert_eq!(to_hex(parse("3584E4").unwrap()), "#3584e4");
        assert_eq!(to_hex(parse("#fff").unwrap()), "#ffffff");
        assert_eq!(to_hex(parse("#3584e480").unwrap()), "#3584e480");
        assert_eq!(parse("#12"), None);
        assert_eq!(parse("not a color"), None);
        assert_eq!(parse("#gg0000"), None);
    }

    #[test]
    fn a_seed_is_always_the_same_color() {
        assert_eq!(from_seed("Evening drive"), from_seed("Evening drive"));
        assert_eq!(from_index(1), from_index(13));
        assert_ne!(from_seed("a"), from_seed("b"));
    }

    #[test]
    fn contrast_is_raised_only_when_it_is_missing() {
        let dark_bg = hex(0x242424);
        let navy = hex(0x3b4a7a);
        assert!(contrast_ratio(navy, dark_bg) < 4.5);
        let fixed = ensure_contrast(navy, dark_bg, 4.5);
        assert!(contrast_ratio(fixed, dark_bg) >= 4.5);
        // Already readable: untouched.
        let white = Color::WHITE;
        assert_eq!(ensure_contrast(white, dark_bg, 4.5), white);
        // On a light page a pale color is darkened instead.
        let pale = hex(0xf8e45c);
        let light_bg = hex(0xfafafa);
        assert!(contrast_ratio(ensure_contrast(pale, light_bg, 4.5), light_bg) >= 4.5);
    }

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

    #[test]
    fn a_tint_keeps_the_alpha_of_the_color_it_tints() {
        let glass = Color::from_rgba(0.2, 0.4, 0.8, 0.5);
        for c in [
            lighten(glass, 0.3),
            darken(glass, 0.3),
            tint(glass, hex(0xff0000), 0.4),
        ] {
            assert_eq!(c.a, 0.5);
        }
        assert!(lighten(glass, 0.3).r > glass.r, "still moves towards white");
        // A mix is the CSS color-mix: alpha moves too.
        assert!(mix(glass, Color::WHITE, 0.5).a > 0.5);
    }
}
