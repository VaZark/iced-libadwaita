//! A small set of built-in symbolic icons (16x16, stroke-based SVG).
//!
//! iced has no icon-theme lookup, so common symbolic icons are embedded.
//! They are tinted with the theme foreground by default.

use crate::Theme;
use iced::widget::svg::{self, Svg};
use iced::{Color, Length};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Icon {
    GoNext,
    GoPrevious,
    GoDown,
    GoUp,
    Check,
    WindowClose,
    WindowMinimize,
    WindowMaximize,
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
}

const HEAD: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16" fill="none" stroke="black" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">"#;

impl Icon {
    fn body(self) -> &'static str {
        match self {
            Icon::GoNext => r#"<path d="M6 3l5 5-5 5"/>"#,
            Icon::GoPrevious => r#"<path d="M10 3L5 8l5 5"/>"#,
            Icon::GoDown => r#"<path d="M3 6l5 5 5-5"/>"#,
            Icon::GoUp => r#"<path d="M3 10l5-5 5 5"/>"#,
            Icon::Check => r#"<path d="M3 8.5l3.5 3.5L13 4.5"/>"#,
            Icon::WindowClose => r#"<path d="M4 4l8 8M12 4l-8 8"/>"#,
            Icon::WindowMinimize => r#"<path d="M3 12h10"/>"#,
            Icon::WindowMaximize => r#"<rect x="3" y="3" width="10" height="10" rx="1"/>"#,
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
            Icon::Home => r#"<path d="M2 8l6-5.5L14 8M3.5 7v6.5h3.2V10h2.6v3.5h3.2V7"/>"#,
        }
    }

    /// The SVG document for this icon.
    pub fn source(self) -> String {
        format!("{HEAD}{}</svg>", self.body())
    }

    /// An [`svg::Handle`] for the icon.
    pub fn handle(self) -> svg::Handle {
        svg::Handle::from_memory(self.source().into_bytes())
    }

    /// Renders the icon at the given size, tinted with the window foreground.
    pub fn view<'a>(self, size: f32) -> Svg<'a, Theme> {
        svg(self.handle()).width(Length::Fixed(size)).height(Length::Fixed(size))
    }

    /// Renders the icon at the given size with a fixed tint.
    pub fn colored<'a>(self, size: f32, color: Color) -> Svg<'a, Theme> {
        self.view(size).style(move |_: &Theme, _| svg::Style { color: Some(color) })
    }
}

fn svg<'a>(handle: svg::Handle) -> Svg<'a, Theme> {
    iced::widget::svg(handle)
}
