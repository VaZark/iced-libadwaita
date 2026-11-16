//! Bundled defaults for predefined icons. Named system icons remain explicit.
use super::{ChevronIcon, EntryIcon, Icon, WindowIcon};

impl Icon {
    /// The bundled SVG artwork of every predefined icon (the match is exhaustive, so a new
    /// variant cannot be added without drawing it). Global overrides apply through `IconSource`.
    pub fn bundled_source(self) -> &'static str {
        match self {
            Self::ListAdd => iad_icons::common::ADD,
            Self::OpenMenu => iad_icons::common::OPEN_MENU,
            // The night-sky glyph the appearance switch already draws for "dark".
            Self::Moon => iad_icons::appearance::DARK,
            Self::Delete => iad_icons::common::DELETE,
            Self::DialogInformation => iad_icons::common::INFORMATION,
            Self::HelpAbout => iad_icons::common::INFORMATION,
            Self::Edit => iad_icons::common::EDIT,
            Self::Save => iad_icons::common::SAVE,
            Self::SaveAs => iad_icons::common::SAVE_AS,
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
            Self::Sidebar => iad_icons::common::SIDEBAR,
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
            Self::FolderOpen => iad_icons::app::FOLDER_OPEN,
            Self::Sync => iad_icons::app::SYNC,
            Self::FileAdd => iad_icons::app::FILE_ADD,
            Self::FileShow => iad_icons::app::FILE_SHOW,
            Self::Link => iad_icons::app::LINK,
            Self::Home => iad_icons::common::HOME,
            Self::Check => iad_icons::common::CHECK,
            Self::ListRemove => iad_icons::common::REMOVE,
            Self::Window => iad_icons::common::WINDOW,
            Self::DialogWarning => iad_icons::common::WARNING,
            Self::DialogError => iad_icons::common::ERROR,
            Self::Reset => iad_icons::common::RESET,
            Self::Document => iad_icons::common::DOCUMENT,
            Self::Dice => iad_icons::common::DICE,
            Self::Bolt => iad_icons::common::BOLT,
            Self::Globe => iad_icons::common::GLOBE,
            Self::Tag => iad_icons::common::TAG,
            Self::Star => iad_icons::common::STAR,
            Self::Clock => iad_icons::common::CLOCK,
            Self::Target => iad_icons::common::TARGET,
            Self::Smile => iad_icons::common::SMILE,
            Self::Compass => iad_icons::common::COMPASS,
            Self::Sparkle => iad_icons::common::SPARKLE,
            Self::Compact => iad_icons::common::COMPACT,
            Self::Shuffle => iad_icons::media::SHUFFLE,
            Self::Repeat => iad_icons::media::REPEAT,
            Self::Queue => iad_icons::media::QUEUE,
            Self::MusicNote => iad_icons::media::MUSIC_NOTE,
            Self::Lyrics => iad_icons::media::LYRICS,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::IconSource;
    #[test]
    fn save_icons_are_registered_with_distinct_bundled_artwork() {
        for (icon, name, source) in [
            (Icon::Save, "document-save", iad_icons::common::SAVE),
            (Icon::SaveAs, "document-save-as", iad_icons::common::SAVE_AS),
        ] {
            assert_eq!(Icon::ALL.iter().filter(|&&value| value == icon).count(), 1);
            assert_eq!(icon.name(), name);
            assert_eq!(icon.system_name().unwrap(), format!("{name}-symbolic"));
            assert_eq!(icon.bundled_source(), source);
            assert!(matches!(IconSource::from(icon), IconSource::Svg(handle)
                if handle == iced::widget::svg::Handle::from_memory(source.as_bytes())));
            let _view = icon.view(16.0);
        }
        assert_ne!(Icon::Save.bundled_source(), Icon::SaveAs.bundled_source());
    }

    #[test]
    fn bundled_defaults_fit_and_render_at_control_sizes() {
        for icon in Icon::ALL {
            let source = icon.bundled_source();
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
