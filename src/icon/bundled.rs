//! Bundled defaults for predefined icons. Named system icons remain explicit.
use super::{ChevronIcon, EntryIcon, Icon, WindowIcon};

impl Icon {
    /// Bundled SVG artwork, when provided. Global overrides apply through `IconSource`.
    pub fn bundled_source(self) -> Option<&'static str> {
        Some(match self {
            Self::ListAdd => include_str!("common/add.svg"),
            Self::Delete => include_str!("common/delete.svg"),
            Self::DialogInformation => include_str!("common/information.svg"),
            Self::HelpAbout => include_str!("common/information.svg"),
            Self::Edit => include_str!("common/edit.svg"),
            Self::ViewMore => include_str!("common/view-more.svg"),
            Self::Play => include_str!("common/play.svg"),
            Self::Pause => include_str!("common/pause.svg"),
            Self::Stop => include_str!("common/stop.svg"),
            Self::SkipForward => include_str!("common/skip-forward.svg"),
            Self::SkipBackward => include_str!("common/skip-backward.svg"),
            Self::Heart => include_str!("common/heart.svg"),
            Self::HeartFilled => include_str!("common/heart-filled.svg"),
            Self::Filter => include_str!("common/filter.svg"),
            Self::Settings => include_str!("common/settings.svg"),
            Self::Sort => include_str!("common/sort.svg"),
            Self::Volume => include_str!("common/volume.svg"),
            Self::VolumeMute => include_str!("common/volume-mute.svg"),
            Self::Fullscreen => include_str!("common/fullscreen.svg"),
            Self::FullscreenExit => include_str!("common/fullscreen-exit.svg"),
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
