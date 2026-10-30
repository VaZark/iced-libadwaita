//! Small color helpers mirroring the Sass/CSS functions libadwaita relies on
//! (`alpha()`, `mix()`, `rgba()`).

use iced::Color;

/// Builds an opaque color from a `0xRRGGBB` literal.
pub fn hex(rgb: u32) -> Color {
    Color::from_rgb8((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8)
}

/// Returns `color` with its alpha replaced by `a` (like GTK's `alpha()`).
pub fn alpha(color: Color, a: f32) -> Color {
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
