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
    Color { a: color.a * a, ..color }
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
}
