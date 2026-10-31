//! Shadow tokens, modeled on libadwaita's stylesheet.
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
//! # use iced_libadwaita::{Theme, Shadows};
//! let flat = Theme::light().with_shadows(Shadows::none());
//! let soft = Theme::light().with_shadows(Shadows::default().with_intensity(0.5));
//! ```
//!
//! Layer values (light) come from libadwaita `_lists.scss`, `_popovers.scss`,
//! `_dialogs.scss` and `_toolbars.scss`; see `docs/hig-inferences/design-language/shadows.md`.

use crate::color::{black, ink};
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
    /// Multiplies offsets and blur radii (1.0 = Adwaita).
    pub scale: f32,
    /// Multiplies layer alpha (1.0 = Adwaita).
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
    /// libadwaita's values.
    pub fn adwaita() -> Self {
        Self {
            enabled: true,
            scale: 1.0,
            intensity: 1.0,
            // _lists.scss: 0 0 0 1px ink 3%, 0 1px 3px 1px ink 7%, 0 2px 6px 2px ink 3%
            card: Elevation::new([
                Layer::new(0.0, 0.0, 0.0, 1.0, ink(0.03)),
                Layer::new(0.0, 1.0, 3.0, 1.0, ink(0.07)),
                Layer::new(0.0, 2.0, 6.0, 2.0, ink(0.03)),
            ]),
            // _popovers.scss: 0 0 0 1px black 5%, 0 1px 5px 1px black 9%, 0 2px 14px 3px black 5%
            popover: Elevation::new([
                Layer::new(0.0, 0.0, 0.0, 1.0, black(0.05)),
                Layer::new(0.0, 1.0, 5.0, 1.0, black(0.09)),
                Layer::new(0.0, 2.0, 14.0, 3.0, black(0.05)),
            ]),
            // _dialogs.scss floating-sheet: 0 0 14px 2px ink 3%, 0 0 5px 2px ink 10%, 0 0 0 1px black 5%
            dialog: Elevation::new([
                Layer::new(0.0, 0.0, 14.0, 2.0, ink(0.03)),
                Layer::new(0.0, 0.0, 5.0, 2.0, ink(0.10)),
                Layer::new(0.0, 0.0, 0.0, 1.0, black(0.05)),
            ]),
            // _popovers.scss `.osd popover > contents`: 0 0 1px white 10%
            osd: Elevation::new([Layer::new(
                0.0,
                0.0,
                0.0,
                1.0,
                Color {
                    a: 0.10,
                    ..Color::WHITE
                },
            )]),
        }
    }

    /// No shadows or rings anywhere.
    pub fn none() -> Self {
        Self {
            enabled: false,
            ..Self::adwaita()
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
        Self::adwaita()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn card_has_ring_and_soft_shadow() {
        let r = Shadows::adwaita().card();
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
        let a = Shadows::adwaita().dialog();
        let b = Shadows::adwaita()
            .with_scale(2.0)
            .with_intensity(0.5)
            .dialog();
        assert_eq!(b.shadow.blur_radius, a.shadow.blur_radius * 2.0);
        assert!(b.shadow.color.a < a.shadow.color.a);
    }
}
