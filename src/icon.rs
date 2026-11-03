//! A small set of built-in symbolic icons (16x16, stroke-based SVG).
//!
//! iced has no icon-theme lookup, so common symbolic icons are embedded.
//! They are tinted with the theme foreground by default.

use crate::Theme;
use iced::widget::svg::{self, Svg};
use iced::{Color, Length};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Icon {
    GoNext,
    GoPrevious,
    GoDown,
    GoUp,
    /// Two chevrons, one up and one down: a column that can be sorted but is not.
    GoUpDown,
    /// A chevron against a bar, pointing left: the first page.
    GoFirst,
    /// A chevron against a bar, pointing right: the last page.
    GoLast,
    Check,
    WindowClose,
    WindowMinimize,
    WindowMaximize,
    WindowRestore,
    /// A generic window glyph (not the themed window-control icons).
    Window,
    /// Toggles a sidebar (`sidebar-show-symbolic`).
    Sidebar,
    OpenMenu,
    ListAdd,
    ListRemove,
    Search,
    ViewMore,
    DialogInformation,
    DialogWarning,
    DialogError,
    Edit,
    Delete,
    Settings,
    Home,
    Calendar,
    /// A funnel: filter a list (`funnel-symbolic` / `view-filter`).
    Filter,
    /// A folder (`folder-symbolic`).
    Folder,
    /// A document (`text-x-generic-symbolic`).
    Document,
    /// Three bars: the grip of a draggable row (`list-drag-handle-symbolic`).
    DragHandle,
    /// A filled triangle pointing right (`media-playback-start-symbolic`).
    Play,
    /// Two bars (`media-playback-pause-symbolic`).
    Pause,
    /// A bar and a triangle pointing left (`media-skip-backward-symbolic`).
    SkipBackward,
    /// A bar and a triangle pointing right (`media-skip-forward-symbolic`).
    SkipForward,
    /// Two crossing arrows (`media-playlist-shuffle-symbolic`).
    Shuffle,
    /// Two arrows in a loop (`media-playlist-repeat-symbolic`).
    Repeat,
    /// Lines with a play mark: what plays next.
    Queue,
    /// A beamed note (`audio-x-generic-symbolic`).
    MusicNote,
    Heart,
    Moon,
    /// A lightning bolt.
    Bolt,
    Globe,
    /// A price tag: tags and labels.
    Tag,
    Star,
    /// A speech box with lines: lyrics, captions.
    Lyrics,
    /// A circle with a needle: explore, discover.
    Compass,
    /// A four-pointed star: new, magic, suggestions.
    Sparkle,
    Clock,
    /// A bullseye.
    Target,
    Smile,
}

impl Icon {
    /// Every built-in icon, in declaration order (for galleries and pickers).
    pub const ALL: &'static [Icon] = &[
        Icon::GoNext,
        Icon::GoPrevious,
        Icon::GoDown,
        Icon::GoUp,
        Icon::GoUpDown,
        Icon::GoFirst,
        Icon::GoLast,
        Icon::Check,
        Icon::WindowClose,
        Icon::WindowMinimize,
        Icon::WindowMaximize,
        Icon::WindowRestore,
        Icon::Window,
        Icon::Sidebar,
        Icon::OpenMenu,
        Icon::ListAdd,
        Icon::ListRemove,
        Icon::Search,
        Icon::ViewMore,
        Icon::DialogInformation,
        Icon::DialogWarning,
        Icon::DialogError,
        Icon::Edit,
        Icon::Delete,
        Icon::Settings,
        Icon::Home,
        Icon::Calendar,
        Icon::Filter,
        Icon::Folder,
        Icon::Document,
        Icon::DragHandle,
        Icon::Play,
        Icon::Pause,
        Icon::SkipBackward,
        Icon::SkipForward,
        Icon::Shuffle,
        Icon::Repeat,
        Icon::Queue,
        Icon::MusicNote,
        Icon::Heart,
        Icon::Moon,
        Icon::Bolt,
        Icon::Globe,
        Icon::Tag,
        Icon::Star,
        Icon::Lyrics,
        Icon::Compass,
        Icon::Sparkle,
        Icon::Clock,
        Icon::Target,
        Icon::Smile,
    ];
}

const HEAD: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16" fill="none" stroke="black" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">"#;

impl Icon {
    fn body(self) -> &'static str {
        match self {
            Icon::GoNext => r#"<path d="M6 3l5 5-5 5"/>"#,
            Icon::GoPrevious => r#"<path d="M10 3L5 8l5 5"/>"#,
            Icon::GoDown => r#"<path d="M3 6l5 5 5-5"/>"#,
            Icon::GoUp => r#"<path d="M3 10l5-5 5 5"/>"#,
            Icon::GoUpDown => r#"<path d="M4 6.5l4-4 4 4M4 9.5l4 4 4-4"/>"#,
            Icon::GoFirst => r#"<path d="M3.5 3v10M11 3L6 8l5 5"/>"#,
            Icon::GoLast => r#"<path d="M12.5 3v10M5 3l5 5-5 5"/>"#,
            Icon::Check => r#"<path d="M3 8.5l3.5 3.5L13 4.5"/>"#,
            Icon::WindowClose => r#"<path d="M4 4l8 8M12 4l-8 8"/>"#,
            Icon::WindowMinimize => r#"<path d="M3 12h10"/>"#,
            Icon::WindowMaximize => r#"<rect x="3" y="3" width="10" height="10" rx="1"/>"#,
            Icon::Window => {
                r#"<rect x="2" y="3" width="12" height="10" rx="1.5"/><path d="M2 6.5h12"/>"#
            }
            Icon::Sidebar => {
                r#"<rect x="2" y="3" width="12" height="10" rx="1.5"/><path d="M6 3v10"/>"#
            }
            Icon::WindowRestore => {
                r#"<rect x="2.5" y="5.5" width="8" height="8" rx="1"/><path d="M5.5 5.5v-2a1 1 0 011-1h6a1 1 0 011 1v6a1 1 0 01-1 1h-2"/>"#
            }
            Icon::OpenMenu => r#"<path d="M2 4h12M2 8h12M2 12h12"/>"#,
            Icon::ListAdd => r#"<path d="M8 3v10M3 8h10"/>"#,
            Icon::ListRemove => r#"<path d="M3 8h10"/>"#,
            Icon::Search => r#"<circle cx="7" cy="7" r="4.2"/><path d="M10.2 10.2L14 14"/>"#,
            Icon::ViewMore => {
                r#"<circle cx="8" cy="3" r="1" fill="black"/><circle cx="8" cy="8" r="1" fill="black"/><circle cx="8" cy="13" r="1" fill="black"/>"#
            }
            Icon::DialogInformation => {
                r#"<circle cx="8" cy="8" r="6.2"/><path d="M8 7.2v4"/><circle cx="8" cy="4.8" r=".6" fill="black"/>"#
            }
            Icon::DialogWarning => {
                r#"<path d="M8 2L14.5 13.5h-13z"/><path d="M8 6.5v3.2"/><circle cx="8" cy="11.6" r=".6" fill="black"/>"#
            }
            Icon::DialogError => {
                r#"<circle cx="8" cy="8" r="6.2"/><path d="M5.5 5.5l5 5M10.5 5.5l-5 5"/>"#
            }
            Icon::Edit => r#"<path d="M2.5 13.5l.7-3L11 2.7a1.4 1.4 0 012 2L5.5 12.8z"/>"#,
            Icon::Delete => r#"<path d="M3 4.5h10M6.5 4.5V3h3v1.5M4.5 4.5l.6 8.5h5.8l.6-8.5"/>"#,
            Icon::Settings => {
                r#"<circle cx="8" cy="8" r="2.2"/><path d="M8 1.8v2M8 12.2v2M1.8 8h2M12.2 8h2M3.6 3.6L5 5M11 11l1.4 1.4M12.4 3.6L11 5M5 11l-1.4 1.4"/>"#
            }
            Icon::Filter => r#"<path d="M2 3h12l-4.5 5.5V13l-3-1.5V8.5z"/>"#,
            Icon::Folder => {
                r#"<path d="M2 4.5a1 1 0 011-1h3l1.5 1.7H13a1 1 0 011 1V12a1 1 0 01-1 1H3a1 1 0 01-1-1z"/>"#
            }
            Icon::Document => {
                r#"<path d="M4 2h5l3 3v8.2a.8.8 0 01-.8.8H4.8a.8.8 0 01-.8-.8V2.8A.8.8 0 014.8 2zM9 2v3h3"/>"#
            }
            Icon::DragHandle => r#"<path d="M3 4.5h10M3 8h10M3 11.5h10"/>"#,
            Icon::Calendar => {
                r#"<rect x="2.5" y="3.5" width="11" height="10" rx="1.5"/><path d="M2.5 6.5h11M5.5 2v3M10.5 2v3"/>"#
            }
            Icon::Home => r#"<path d="M2 8l6-5.5L14 8M3.5 7v6.5h3.2V10h2.6v3.5h3.2V7"/>"#,
            Icon::Play => {
                r#"<path d="M2.5 2.8v10.4c0 1.2 1.2 1.9 2.2 1.3l8.6-5.2c.9-.5.9-1.9 0-2.5L4.7 1.5c-1-.6-2.2.1-2.2 1.3z" fill="black" stroke="none"/>"#
            }
            Icon::Pause => r#"<path d="M5 3v10M11 3v10" stroke-width="2.6"/>"#,
            Icon::SkipBackward => {
                r#"<rect x="2" y="2.5" width="2" height="11" rx=".8" fill="black" stroke="none"/><path d="M14 3.4v9.2c0 .9-1 1.4-1.7.9L6.2 9.1c-.6-.5-.6-1.7 0-2.2l6.1-4.4c.7-.5 1.7 0 1.7.9z" fill="black" stroke="none"/>"#
            }
            Icon::SkipForward => {
                r#"<rect x="12" y="2.5" width="2" height="11" rx=".8" fill="black" stroke="none"/><path d="M2 3.4v9.2c0 .9 1 1.4 1.7.9l6.1-4.4c.6-.5.6-1.7 0-2.2L3.7 2.5C3 2 2 2.5 2 3.4z" fill="black" stroke="none"/>"#
            }
            Icon::Shuffle => {
                r#"<path d="M2 4.5h2.5c3 0 4 7 7 7H14M2 11.5h2.5c1 0 1.8-1 2.6-2.2M9 6.2c.7-1 1.4-1.7 2.5-1.7H14M12 2.5l2 2-2 2M12 9.5l2 2-2 2"/>"#
            }
            Icon::Repeat => {
                r#"<path d="M3 7V6a2 2 0 012-2h8M11 2l2 2-2 2M13 9v1a2 2 0 01-2 2H3M5 14l-2-2 2-2"/>"#
            }
            Icon::Queue => r#"<path d="M2 4h9M2 8h9M2 12h5M12 10v4l3-2z"/>"#,
            Icon::MusicNote => {
                r#"<path d="M6 11.5V3l7-1.5v8.5"/><circle cx="4.5" cy="11.5" r="1.8"/><circle cx="11.5" cy="10" r="1.8"/>"#
            }
            Icon::Heart => {
                r#"<path d="M8 13.5S2.3 10.2 2.3 6.2A3 3 0 018 4.6a3 3 0 015.7 1.6c0 4-5.7 7.3-5.7 7.3z"/>"#
            }
            Icon::Moon => r#"<path d="M12.7 9.6A5.2 5.2 0 016.4 3.3a5.2 5.2 0 106.3 6.3z"/>"#,
            Icon::Bolt => r#"<path d="M9.2 1.5L3.5 9h4.3l-1 5.5L12.5 7H8.2z"/>"#,
            Icon::Globe => {
                r#"<circle cx="8" cy="8" r="6"/><path d="M2 8h12M8 2c2.2 2.2 2.2 9.8 0 12M8 2c-2.2 2.2-2.2 9.8 0 12"/>"#
            }
            Icon::Tag => {
                r#"<path d="M2.5 2.5h5l6 6-5 5-6-6z"/><circle cx="5.3" cy="5.3" r=".8" fill="black"/>"#
            }
            Icon::Star => {
                r#"<path d="M8 2l1.8 3.7 4 .6-2.9 2.8.7 4L8 11.2 4.4 13l.7-4L2.2 6.3l4-.6z"/>"#
            }
            Icon::Lyrics => {
                r#"<path d="M2.5 3h11v7.5h-6.2L4.5 13v-2.5h-2z"/><path d="M5 6h6M5 8.2h4"/>"#
            }
            Icon::Compass => {
                r#"<circle cx="8" cy="8" r="6"/><path d="M10.8 5.2L9.2 9.2 5.2 10.8 6.8 6.8z"/>"#
            }
            Icon::Sparkle => {
                r#"<path d="M8 1.5l1.5 4.5 4.5 1.5-4.5 1.5L8 13.5 6.5 9 2 7.5 6.5 6z"/>"#
            }
            Icon::Clock => r#"<circle cx="8" cy="8" r="6"/><path d="M8 4.5V8l2.4 1.6"/>"#,
            Icon::Target => {
                r#"<circle cx="8" cy="8" r="6"/><circle cx="8" cy="8" r="3"/><circle cx="8" cy="8" r=".8" fill="black"/>"#
            }
            Icon::Smile => {
                r#"<circle cx="8" cy="8" r="6"/><path d="M5.4 9.6a3.2 3.2 0 005.2 0"/><circle cx="5.9" cy="6.6" r=".8" fill="black"/><circle cx="10.1" cy="6.6" r=".8" fill="black"/>"#
            }
        }
    }

    /// The SVG document for this icon.
    pub fn source(self) -> String {
        format!("{HEAD}{}</svg>", self.body())
    }

    /// The freedesktop symbolic icon name, for icons the desktop's icon theme
    /// should draw (window controls follow the user's theme, like other GNOME apps).
    pub fn system_name(self) -> Option<&'static str> {
        Some(match self {
            Icon::WindowClose => "window-close-symbolic",
            Icon::WindowMinimize => "window-minimize-symbolic",
            Icon::WindowMaximize => "window-maximize-symbolic",
            Icon::WindowRestore => "window-restore-symbolic",
            _ => return None,
        })
    }

    /// An [`svg::Handle`] for the icon: the current icon theme's version when it
    /// has one (see [`Icon::system_name`]), else the built-in drawing.
    pub fn handle(self) -> svg::Handle {
        if let Some(bytes) = self.system_name().and_then(crate::system::find_icon) {
            return svg::Handle::from_memory(bytes);
        }
        // The built-in drawing never changes, so its handle is made once: building the SVG text
        // and hashing it on every view of every icon is work a screen full of icons repeats at
        // every frame.
        static BUILTIN: OnceLock<Mutex<HashMap<Icon, svg::Handle>>> = OnceLock::new();
        let cache = BUILTIN.get_or_init(Default::default);
        let Ok(mut cache) = cache.lock() else {
            return svg::Handle::from_memory(self.source().into_bytes());
        };
        cache
            .entry(self)
            .or_insert_with(|| svg::Handle::from_memory(self.source().into_bytes()))
            .clone()
    }

    /// Renders the icon at the given size, tinted with the window foreground.
    pub fn view<'a>(self, size: f32) -> Svg<'a, Theme> {
        svg(self.handle())
            .width(Length::Fixed(size))
            .height(Length::Fixed(size))
    }

    /// Renders the icon at the given size with a fixed tint.
    pub fn colored<'a>(self, size: f32, color: Color) -> Svg<'a, Theme> {
        self.view(size)
            .style(move |_: &Theme, _| svg::Style { color: Some(color) })
    }
}

/// Where an icon comes from: one of the built-in [`Icon`]s, or an SVG of your own (a 16 px
/// symbolic drawing in `currentColor`-style black strokes, tinted like the built-ins). Everything
/// that takes an icon takes `impl Into<IconSource>`, so both work: a menu item, a menu button,
/// a chip.
#[derive(Debug, Clone)]
pub enum IconSource {
    Icon(Icon),
    Svg(svg::Handle),
}

impl From<Icon> for IconSource {
    fn from(i: Icon) -> Self {
        Self::Icon(i)
    }
}

impl From<svg::Handle> for IconSource {
    fn from(h: svg::Handle) -> Self {
        Self::Svg(h)
    }
}

impl IconSource {
    /// An icon from SVG text, e.g. `IconSource::svg(include_str!("star.svg"))`.
    pub fn svg(source: impl Into<String>) -> Self {
        Self::Svg(svg::Handle::from_memory(source.into().into_bytes()))
    }
    /// Renders the icon at `size`, tinted with the window foreground.
    pub fn view<'a>(&self, size: f32) -> Svg<'a, Theme> {
        match self {
            Self::Icon(i) => i.view(size),
            Self::Svg(h) => svg(h.clone())
                .width(Length::Fixed(size))
                .height(Length::Fixed(size)),
        }
    }
}

fn svg<'a>(handle: svg::Handle) -> Svg<'a, Theme> {
    iced::widget::svg(handle)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_window_controls_use_the_system_icon_theme() {
        assert_eq!(
            Icon::WindowClose.system_name(),
            Some("window-close-symbolic")
        );
        assert_eq!(
            Icon::WindowRestore.system_name(),
            Some("window-restore-symbolic")
        );
        assert_eq!(Icon::Window.system_name(), None);
        assert_eq!(Icon::OpenMenu.system_name(), None);
    }

    #[test]
    fn an_icon_source_is_a_builtin_or_your_own_svg() {
        let a: IconSource = Icon::Star.into();
        assert!(matches!(a, IconSource::Icon(Icon::Star)));
        assert!(matches!(IconSource::svg("<svg/>"), IconSource::Svg(_)));
    }

    #[test]
    fn all_lists_every_icon_once_and_each_draws() {
        let mut seen = std::collections::HashSet::new();
        for i in Icon::ALL {
            assert!(seen.insert(*i));
            assert!(i.source().contains("<svg"));
        }
        assert!(Icon::ALL.len() > 50);
    }

    #[test]
    fn every_icon_has_a_drawing() {
        for i in [
            Icon::WindowRestore,
            Icon::Window,
            Icon::WindowClose,
            Icon::Play,
            Icon::Clock,
            Icon::Sparkle,
            Icon::Smile,
        ] {
            assert!(i.source().contains("<svg") && i.source().ends_with("</svg>"));
        }
    }
}
