//! Shadow tokens. The bare baseline has none; `themes/libadwaita.ron` models libadwaita's.
//!
//! Adwaita builds elevation from *stacked* CSS box-shadows (a 1px ring plus one
//! or two blurred layers). iced draws one shadow per quad, so each [`Elevation`]
//! keeps the original layers and is *resolved* into an iced `Shadow` (the
//! blurred layers merged) plus an optional 1px ring (drawn as the border).
//!
//! Everything is a setting: turn shadows off, scale their size, change their
//! intensity, or replace any elevation.
//!
//! ```
//! # use iced_adaptive_design::{Theme, Shadows};
//! let flat = Theme::light().with_shadows(Shadows::none());
//! let soft = Theme::light().with_shadows(Shadows::default().with_intensity(0.5));
//! ```
//!
//! Layer values (light) come from libadwaita `_lists.scss`, `_popovers.scss`,
//! `_dialogs.scss` and `_toolbars.scss`; see `docs/hig-inferences/design-language/shadows.md`.

use iced::{Color, Shadow, Vector};

/// One CSS box-shadow layer: `x y blur spread color`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Layer {
    pub x: f32,
    pub y: f32,
    pub blur: f32,
    pub spread: f32,
    pub color: Color,
}

impl Layer {
    pub const fn new(x: f32, y: f32, blur: f32, spread: f32, color: Color) -> Self {
        Self {
            x,
            y,
            blur,
            spread,
            color,
        }
    }

    /// A pure 1px outline (`0 0 0 1px color`), which we draw as a border.
    pub fn is_ring(&self) -> bool {
        self.x == 0.0 && self.y == 0.0 && self.blur == 0.0 && self.spread > 0.0
    }
}

/// A stack of layers describing one level of elevation.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Elevation {
    pub layers: Vec<Layer>,
}

impl Elevation {
    pub fn none() -> Self {
        Self::default()
    }

    pub fn new(layers: impl IntoIterator<Item = Layer>) -> Self {
        Self {
            layers: layers.into_iter().collect(),
        }
    }
}

impl Elevation {
    /// Parses CSS box-shadow syntax: `x y blur spread color, ...` (lengths in `px`, `none` for
    /// no layers). Returns `None` when any layer is malformed.
    pub fn parse(text: &str) -> Option<Self> {
        let text = text.trim();
        if text.eq_ignore_ascii_case("none") || text.is_empty() {
            return Some(Self::none());
        }
        let mut layers = Vec::new();
        for part in split_top_level(text) {
            let part = part.trim();
            let mut n: Vec<f32> = Vec::new();
            let mut rest = part;
            while let Some((tok, tail)) = rest.split_once(char::is_whitespace) {
                match tok.trim_end_matches("px").parse::<f32>() {
                    Ok(v) if n.len() < 4 => n.push(v),
                    _ => break,
                }
                rest = tail.trim_start();
            }
            let (color, n) = (rest, n);
            let color = match crate::theme_config::VariableType::Color.parse(color)? {
                crate::theme_config::Value::Color(c) => c,
                _ => return None,
            };
            let [x, y] = <[f32; 2]>::try_from(n.get(..2)?).ok()?;
            layers.push(Layer::new(
                x,
                y,
                n.get(2).copied().unwrap_or(0.0),
                n.get(3).copied().unwrap_or(0.0),
                color,
            ));
        }
        Some(Self { layers })
    }

    /// The elevation in CSS box-shadow syntax (the inverse of [`parse`](Self::parse)).
    pub fn to_css(&self) -> String {
        if self.layers.is_empty() {
            return "none".into();
        }
        let b = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
        self.layers
            .iter()
            .map(|l| {
                let c = l.color;
                format!(
                    "{}px {}px {}px {}px rgba({}, {}, {}, {})",
                    l.x,
                    l.y,
                    l.blur,
                    l.spread,
                    b(c.r),
                    b(c.g),
                    b(c.b),
                    c.a
                )
            })
            .collect::<Vec<_>>()
            .join(", ")
    }
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

/// An elevation resolved for drawing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Resolved {
    /// The blurred layers merged into one iced shadow.
    pub shadow: Shadow,
    /// The 1px ring, to be used as the border color (width 1).
    pub ring: Option<Color>,
}

impl Resolved {
    /// The border to draw for the ring (zero width when there is none).
    pub fn border(&self, radius: impl Into<iced::border::Radius>) -> iced::Border {
        iced::Border {
            color: self.ring.unwrap_or(Color::TRANSPARENT),
            width: if self.ring.is_some() { 1.0 } else { 0.0 },
            radius: radius.into(),
        }
    }
}

/// All shadow settings of a [`Theme`](crate::Theme).
#[derive(Debug, Clone, PartialEq)]
pub struct Shadows {
    /// Master switch.
    pub enabled: bool,
    /// Multiplies offsets and blur radii (1.0 = as themed).
    pub scale: f32,
    /// Multiplies layer alpha (1.0 = as themed).
    pub intensity: f32,
    /// `.card` / `.boxed-list`.
    pub card: Elevation,
    /// Popovers and menus.
    pub popover: Elevation,
    /// Floating dialogs (`floating-sheet > sheet`).
    pub dialog: Elevation,
    /// `.osd` surfaces (toasts, overlay controls).
    pub osd: Elevation,
}

impl Shadows {
    /// The bare baseline: shadows on, but no elevation has any layers (a theme adds them).
    pub fn bare() -> Self {
        Self {
            enabled: true,
            scale: 1.0,
            intensity: 1.0,
            card: Elevation::none(),
            popover: Elevation::none(),
            dialog: Elevation::none(),
            osd: Elevation::none(),
        }
    }

    /// No shadows or rings anywhere.
    pub fn none() -> Self {
        Self {
            enabled: false,
            ..Self::bare()
        }
    }

    pub fn with_scale(mut self, scale: f32) -> Self {
        self.scale = scale.max(0.0);
        self
    }

    pub fn with_intensity(mut self, intensity: f32) -> Self {
        self.intensity = intensity.max(0.0);
        self
    }

    /// Merges an elevation into one iced shadow plus a ring.
    ///
    /// Blurred layers are combined by alpha-weighting their offsets and taking
    /// the largest extent (`blur + spread`); alpha is summed, capped at 0.5.
    pub fn resolve(&self, e: &Elevation) -> Resolved {
        if !self.enabled {
            return Resolved {
                shadow: Shadow::default(),
                ring: None,
            };
        }
        let k = self.intensity;
        let ring = e.layers.iter().find(|l| l.is_ring()).map(|l| Color {
            a: (l.color.a * k).min(1.0),
            ..l.color
        });

        let soft: Vec<&Layer> = e.layers.iter().filter(|l| !l.is_ring()).collect();
        let total: f32 = soft.iter().map(|l| l.color.a).sum();
        if soft.is_empty() || total <= 0.0 {
            return Resolved {
                shadow: Shadow::default(),
                ring,
            };
        }
        let (mut x, mut y, mut extent) = (0.0, 0.0, 0.0f32);
        for l in &soft {
            let w = l.color.a / total;
            x += l.x * w;
            y += l.y * w;
            extent = extent.max(l.blur + l.spread);
        }
        let base = soft[0].color;
        Resolved {
            shadow: Shadow {
                color: Color {
                    a: (total * k).min(0.5),
                    ..base
                },
                offset: Vector::new(x * self.scale, y * self.scale),
                blur_radius: extent * self.scale,
            },
            ring,
        }
    }

    pub fn card(&self) -> Resolved {
        self.resolve(&self.card)
    }
    pub fn popover(&self) -> Resolved {
        self.resolve(&self.popover)
    }
    pub fn dialog(&self) -> Resolved {
        self.resolve(&self.dialog)
    }
    pub fn osd(&self) -> Resolved {
        self.resolve(&self.osd)
    }
}

impl Default for Shadows {
    fn default() -> Self {
        Self::bare()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A themed set of shadows (the bare baseline has no layers).
    fn themed() -> Shadows {
        let e = |css| Elevation::parse(css).unwrap();
        Shadows {
            card: e("0 0 0 1px rgba(0, 0, 6, 0.03), 0 1px 3px 1px rgba(0, 0, 6, 0.07)"),
            dialog: e("0 0 14px 2px rgba(0, 0, 6, 0.03), 0 0 5px 2px rgba(0, 0, 6, 0.1)"),
            ..Shadows::bare()
        }
    }

    #[test]
    fn css_round_trips_and_rejects_garbage() {
        let css = "0px 1px 3px 1px rgba(0, 0, 6, 0.07), 0px 0px 0px 1px rgba(255, 255, 255, 0.1)";
        assert_eq!(Elevation::parse(css).unwrap().to_css(), css);
        assert_eq!(Elevation::parse("none"), Some(Elevation::none()));
        assert!(Elevation::parse("0 0 blur red").is_none());
    }

    #[test]
    fn card_has_ring_and_soft_shadow() {
        let r = themed().card();
        assert!(r.ring.is_some());
        assert!(r.shadow.blur_radius > 0.0 && r.shadow.color.a > 0.0);
        assert_eq!(r.border(12.0).width, 1.0);
    }

    #[test]
    fn disabled_removes_everything() {
        let r = Shadows::none().popover();
        assert!(r.ring.is_none());
        assert_eq!(r.shadow.color.a, 0.0);
        assert_eq!(r.border(12.0).width, 0.0);
    }

    #[test]
    fn scale_and_intensity_apply() {
        let a = themed().dialog();
        let b = themed().with_scale(2.0).with_intensity(0.5).dialog();
        assert_eq!(b.shadow.blur_radius, a.shadow.blur_radius * 2.0);
        assert!(b.shadow.color.a < a.shadow.color.a);
    }
}
