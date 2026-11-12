//! Bundled defaults for predefined icons. Named system icons remain explicit.
use super::{ChevronIcon, EntryIcon, Icon, WindowIcon};

impl Icon {
    /// Bundled SVG artwork, when provided. Global overrides apply through `IconSource`.
    pub fn bundled_source(self) -> Option<&'static str> {
        Some(match self {
            Self::ListAdd => iad_icons::common::ADD,
            Self::OpenMenu => iad_icons::common::OPEN_MENU,
            // The night-sky glyph the appearance switch already draws for "dark".
            Self::Moon => iad_icons::appearance::DARK,
            Self::Delete => iad_icons::common::DELETE,
            Self::DialogInformation => iad_icons::common::INFORMATION,
            Self::HelpAbout => iad_icons::common::INFORMATION,
            Self::Edit => iad_icons::common::EDIT,
            Self::ViewMore => iad_icons::common::VIEW_MORE,
            Self::Play => iad_icons::common::PLAY,
            Self::Pause => iad_icons::common::PAUSE,
            Self::Stop => iad_icons::common::STOP,
            Self::SkipForward => iad_icons::common::SKIP_FORWARD,
            Self::SkipBackward => iad_icons::common::SKIP_BACKWARD,
            Self::Heart => iad_icons::common::HEART,
            Self::HeartFilled => iad_icons::common::HEART_FILLED,
            Self::Filter => iad_icons::common::FILTER,
            Self::Settings => iad_icons::common::SETTINGS,
            Self::Sort => iad_icons::common::SORT,
            Self::Volume => iad_icons::common::VOLUME,
            Self::VolumeMute => iad_icons::common::VOLUME_MUTE,
            Self::Fullscreen => iad_icons::common::FULLSCREEN,
            Self::FullscreenExit => iad_icons::common::FULLSCREEN_EXIT,
            Self::GoNext => ChevronIcon::Right.source(),
            Self::GoPrevious => ChevronIcon::Left.source(),
            Self::GoDown => ChevronIcon::Down.source(),
            Self::GoUp => ChevronIcon::Up.source(),
            Self::GoUpDown => ChevronIcon::UpDown.source(),
            Self::GoFirst => ChevronIcon::First.source(),
            Self::GoLast => ChevronIcon::Last.source(),
            Self::WindowClose => WindowIcon::Close.source(),
            Self::WindowMinimize => WindowIcon::Minimize.source(),
            Self::WindowMaximize => WindowIcon::Maximize.source(),
            Self::WindowRestore => WindowIcon::Restore.source(),
            Self::Search => EntryIcon::Search.source(),
            Self::DragHandle => EntryIcon::DragHandle.source(),
            Self::Folder => iad_icons::app::FOLDER,
            Self::Calendar => iad_icons::app::CALENDAR,
            Self::Pin => iad_icons::app::PIN,
            Self::Archive => iad_icons::app::ARCHIVE,
            Self::Palette => iad_icons::app::PALETTE,
            Self::Eye => iad_icons::app::EYE,
            Self::Note => iad_icons::app::NOTE,
            Self::CheckCircle => iad_icons::app::CHECK_CIRCLE,
            Self::Kanban => iad_icons::app::KANBAN,
            Self::FlatList => iad_icons::app::FLAT_LIST,
            Self::CompactRows => iad_icons::app::COMPACT_ROWS,
            Self::CardGrid => iad_icons::app::CARD_GRID,
            Self::Tags => iad_icons::app::TAGS,
            _ => return None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::IconSource;
    #[test]
    fn bundled_defaults_fit_and_render_at_control_sizes() {
        for icon in Icon::ALL {
            let Some(source) = icon.bundled_source() else {
                continue;
            };
            assert!(
                matches!(IconSource::from(*icon), IconSource::Svg(_)),
                "{icon:?}"
            );
            let tree = resvg::usvg::Tree::from_str(source, &Default::default()).unwrap();
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
