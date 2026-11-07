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
            Self::Text => include_str!("entry/text.svg"),
            Self::Number => include_str!("entry/number.svg"),
            Self::Calendar => include_str!("entry/calendar.svg"),
            Self::Switch => include_str!("entry/switch.svg"),
            Self::Action => include_str!("entry/action.svg"),
            Self::Select => include_str!("entry/select.svg"),
            Self::Multiselect => include_str!("entry/multiselect.svg"),
            Self::Chips => include_str!("entry/chips.svg"),
            Self::Multiline => include_str!("entry/multiline.svg"),
            Self::Search => include_str!("entry/search.svg"),
            Self::Clear => include_str!("entry/clear.svg"),
            Self::Increment => include_str!("entry/increment.svg"),
            Self::Decrement => include_str!("entry/decrement.svg"),
            Self::Navigation => include_str!("entry/navigation.svg"),
            Self::DragHandle => include_str!("entry/draghandle.svg"),
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
