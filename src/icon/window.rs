//! Bundled window-control artwork.
use super::{IconSource, IconView};

/// Standard close, minimize, maximize and restore symbols.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowIcon {
    Close,
    Minimize,
    Maximize,
    Restore,
}
impl WindowIcon {
    pub const ALL: &[Self] = &[Self::Close, Self::Minimize, Self::Maximize, Self::Restore];
    pub fn source(self) -> &'static str {
        match self {
            Self::Close => include_str!("window/close.svg"),
            Self::Minimize => include_str!("window/minimize.svg"),
            Self::Maximize => include_str!("window/maximize.svg"),
            Self::Restore => include_str!("window/restore.svg"),
        }
    }
    pub fn view<'a>(self, size: f32) -> IconView<'a> {
        IconSource::from(self).view(size)
    }
}
impl From<WindowIcon> for IconSource {
    fn from(icon: WindowIcon) -> Self {
        Self::svg(icon.source())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bundled_window_artwork_renders_at_control_sizes() {
        for icon in WindowIcon::ALL {
            let tree = resvg::usvg::Tree::from_str(icon.source(), &Default::default()).unwrap();
            let bounds = tree.root().abs_stroke_bounding_box();
            assert!(
                bounds.left() >= 0.0
                    && bounds.top() >= 0.0
                    && bounds.right() <= 16.0
                    && bounds.bottom() <= 16.0
            );
            for size in [12, 16, 24, 32] {
                let mut pixels = resvg::tiny_skia::Pixmap::new(size, size).unwrap();
                resvg::render(
                    &tree,
                    resvg::tiny_skia::Transform::from_scale(size as f32 / 16.0, size as f32 / 16.0),
                    &mut pixels.as_mut(),
                );
                assert!(pixels.data().chunks_exact(4).any(|p| p[3] > 0));
            }
        }
        for icon in [
            crate::Icon::WindowClose,
            crate::Icon::WindowMinimize,
            crate::Icon::WindowMaximize,
            crate::Icon::WindowRestore,
        ] {
            assert!(matches!(IconSource::from(icon), IconSource::Svg(_)));
        }
    }
}
