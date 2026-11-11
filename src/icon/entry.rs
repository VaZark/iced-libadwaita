//! Bundled 16px SVG artwork for entry controls, independent of the system icon theme.
use super::{IconSource, IconView};

/// Consistent artwork for entry prefixes, suffixes and control actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryIcon {
    Text,
    Number,
    Calendar,
    Switch,
    Action,
    Select,
    Multiselect,
    Chips,
    Multiline,
    Search,
    Clear,
    Increment,
    Decrement,
    Navigation,
    DragHandle,
}
impl EntryIcon {
    pub const ALL: &[Self] = &[
        Self::Text,
        Self::Number,
        Self::Calendar,
        Self::Switch,
        Self::Action,
        Self::Select,
        Self::Multiselect,
        Self::Chips,
        Self::Multiline,
        Self::Search,
        Self::Clear,
        Self::Increment,
        Self::Decrement,
        Self::Navigation,
        Self::DragHandle,
    ];
    pub fn source(self) -> &'static str {
        match self {
            Self::Text => iad_icons::entry::TEXT,
            Self::Number => iad_icons::entry::NUMBER,
            Self::Calendar => iad_icons::entry::CALENDAR,
            Self::Switch => iad_icons::entry::SWITCH,
            Self::Action => iad_icons::entry::ACTION,
            Self::Select => iad_icons::entry::SELECT,
            Self::Multiselect => iad_icons::entry::MULTISELECT,
            Self::Chips => iad_icons::entry::CHIPS,
            Self::Multiline => iad_icons::entry::MULTILINE,
            Self::Search => iad_icons::entry::SEARCH,
            Self::Clear => super::WindowIcon::Close.source(),
            Self::Increment => iad_icons::entry::INCREMENT,
            Self::Decrement => iad_icons::entry::DECREMENT,
            Self::Navigation => super::ChevronIcon::Right.source(),
            Self::DragHandle => iad_icons::entry::DRAGHANDLE,
        }
    }
    pub fn view<'a>(self, size: f32) -> IconView<'a> {
        IconSource::from(self).view(size)
    }
}
impl From<EntryIcon> for IconSource {
    fn from(icon: EntryIcon) -> Self {
        Self::svg(icon.source())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_icon_fits_its_viewbox_and_renders_at_control_sizes() {
        for icon in EntryIcon::ALL {
            let tree = resvg::usvg::Tree::from_str(icon.source(), &Default::default()).unwrap();
            assert_eq!(tree.size().width(), 16.0);
            assert_eq!(tree.size().height(), 16.0);
            let bounds = tree.root().abs_stroke_bounding_box();
            assert!(
                bounds.left() >= 0.0
                    && bounds.top() >= 0.0
                    && bounds.right() <= 16.0
                    && bounds.bottom() <= 16.0,
                "{icon:?}: {bounds:?}"
            );
            for size in [12, 16, 24, 32] {
                let mut pixels = resvg::tiny_skia::Pixmap::new(size, size).unwrap();
                resvg::render(
                    &tree,
                    resvg::tiny_skia::Transform::from_scale(size as f32 / 16.0, size as f32 / 16.0),
                    &mut pixels.as_mut(),
                );
                assert!(
                    pixels.data().chunks_exact(4).any(|p| p[3] > 0),
                    "{icon:?} at {size}px"
                );
            }
        }
    }
}
