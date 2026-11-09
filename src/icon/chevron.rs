//! Bundled navigation and disclosure chevrons shared by all controls.
use super::{IconSource, IconView};

/// Theme-independent directional artwork for navigation, disclosure and sorting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChevronIcon {
    Right,
    Left,
    Down,
    Up,
    UpDown,
    First,
    Last,
}
impl ChevronIcon {
    pub const ALL: &[Self] = &[
        Self::Right,
        Self::Left,
        Self::Down,
        Self::Up,
        Self::UpDown,
        Self::First,
        Self::Last,
    ];
    pub fn source(self) -> &'static str {
        match self {
            Self::Right => include_str!("chevron/right.svg"),
            Self::Left => include_str!("chevron/left.svg"),
            Self::Down => include_str!("chevron/down.svg"),
            Self::Up => include_str!("chevron/up.svg"),
            Self::UpDown => include_str!("chevron/up-down.svg"),
            Self::First => include_str!("chevron/first.svg"),
            Self::Last => include_str!("chevron/last.svg"),
        }
    }
    pub fn view<'a>(self, size: f32) -> IconView<'a> {
        IconSource::from(self).view(size)
    }
}
impl From<ChevronIcon> for IconSource {
    fn from(icon: ChevronIcon) -> Self {
        Self::svg(icon.source())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chevrons_render_within_their_viewbox() {
        for icon in ChevronIcon::ALL {
            let tree = resvg::usvg::Tree::from_str(icon.source(), &Default::default()).unwrap();
            let bounds = tree.root().abs_stroke_bounding_box();
            assert!(bounds.left() >= 0.0 && bounds.top() >= 0.0);
            assert!(bounds.right() <= 16.0 && bounds.bottom() <= 16.0);
            let mut pixels = resvg::tiny_skia::Pixmap::new(16, 16).unwrap();
            resvg::render(&tree, Default::default(), &mut pixels.as_mut());
            assert!(pixels.data().chunks_exact(4).any(|p| p[3] > 0));
        }
        for icon in [
            crate::Icon::GoNext,
            crate::Icon::GoPrevious,
            crate::Icon::GoUp,
            crate::Icon::GoDown,
            crate::Icon::GoUpDown,
            crate::Icon::GoFirst,
            crate::Icon::GoLast,
        ] {
            assert!(matches!(IconSource::from(icon), IconSource::Svg(_)));
        }
    }
}
