//! Desktop preferences and desktop-neutral XDG icon lookup.
//!
//! Window layout comes from the config (`desktop.conf` and the theme's `[windows]` tokens), never
//! from a desktop's settings store. The only system input is the color scheme the settings portal
//! reports; appearance detection and updates are shared through [`crate::utils`].

use crate::Scheme;
use std::sync::atomic::{AtomicU8, Ordering};

/// A `Scheme` that can be set and cleared from any thread.
pub(crate) struct AtomicScheme(AtomicU8);
impl AtomicScheme {
    pub(crate) const fn new() -> Self {
        Self(AtomicU8::new(0))
    }
    pub(crate) fn load(&self) -> Option<Scheme> {
        match self.0.load(Ordering::Relaxed) {
            1 => Some(Scheme::Light),
            2 => Some(Scheme::Dark),
            _ => None,
        }
    }
    pub(crate) fn store(&self, scheme: Option<Scheme>) {
        let v = match scheme {
            None => 0,
            Some(Scheme::Light) => 1,
            Some(Scheme::Dark) => 2,
        };
        self.0.store(v, Ordering::Relaxed);
    }
}

/// A window button that can appear in a header bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WindowButton {
    Minimize,
    Maximize,
    Close,
}

/// Which window buttons are shown, and on which side
/// (`button-layout`, e.g. `appmenu:close` or `close,minimize,maximize:`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecorationLayout {
    /// Buttons on the start side (left in LTR), in order.
    pub start: Vec<WindowButton>,
    /// Buttons on the end side (right in LTR), in order.
    pub end: Vec<WindowButton>,
}

impl DecorationLayout {
    /// GNOME's default: only a close button, at the end (`appmenu:close`).
    pub fn gnome_default() -> Self {
        Self::parse("appmenu:close")
    }

    /// Parses a GTK `button-layout` string. `:` separates start from end; a
    /// string without `:` puts everything at the start. `appmenu`, `icon`,
    /// `spacer` and unknown names are ignored here.
    pub fn parse(layout: &str) -> Self {
        let layout = layout.trim().trim_matches('\'').trim_matches('"');
        let (start, end) = match layout.split_once(':') {
            Some((a, b)) => (a, b),
            None => (layout, ""),
        };
        let buttons = |s: &str| {
            s.split(',')
                .filter_map(|t| match t.trim() {
                    "minimize" => Some(WindowButton::Minimize),
                    "maximize" => Some(WindowButton::Maximize),
                    "close" => Some(WindowButton::Close),
                    _ => None,
                })
                .collect::<Vec<_>>()
        };
        Self {
            start: buttons(start),
            end: buttons(end),
        }
    }

    /// The layout in effect: the desktop config's fixed `button-layout` if set, else the theme's
    /// `[windows] button-layout` token, else [`gnome_default`](Self::gnome_default).
    pub fn detect() -> Self {
        if let crate::config::Pick::Fixed(s) = &crate::config::current().button_layout {
            return Self::parse(s);
        }
        match crate::theme_config::current().text("windows", "button-layout") {
            Some(s) => Self::parse(s),
            None => Self::gnome_default(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.start.is_empty() && self.end.is_empty()
    }
}

/// The color scheme the desktop reported through the **settings portal**
/// (`org.freedesktop.appearance color-scheme`); unset until it has reported.
static PORTAL_SCHEME: AtomicScheme = AtomicScheme::new();

/// Maps iced's system theme mode (read from the settings portal) to a scheme. "No preference" is
/// not an answer: iced reports it too when a window opens (a Wayland window has no theme of its
/// own), and it must not turn a dark desktop light, so it maps to `None`.
fn scheme_from_mode(mode: iced::theme::Mode) -> Option<crate::Scheme> {
    match mode {
        iced::theme::Mode::Dark => Some(crate::Scheme::Dark),
        iced::theme::Mode::Light => Some(crate::Scheme::Light),
        iced::theme::Mode::None => None,
    }
}

/// What a reported mode means now: the scheme it names (remembered), else what was known before.
fn resolve_scheme(mode: iced::theme::Mode) -> crate::Scheme {
    match scheme_from_mode(mode) {
        Some(scheme) => {
            remember_scheme(scheme);
            scheme
        }
        None => color_scheme().unwrap_or(crate::Scheme::Light),
    }
}

fn remember_scheme(scheme: crate::Scheme) {
    PORTAL_SCHEME.store(Some(scheme));
}

/// The desktop color scheme. It comes from the settings portal once [`follow_color_scheme`] has
/// reported it (that is the only source inside a sandbox such as Flatpak, and it is not a Wayland
/// protocol). `None` until it has reported.
pub fn color_scheme() -> Option<crate::Scheme> {
    PORTAL_SCHEME.load()
}

/// Reads the desktop color scheme from the settings portal now, and from then on
/// [`color_scheme`] returns it. Run it once at start-up (`Task::perform`-style: map the result
/// to a message that redraws).
pub fn detect_color_scheme() -> iced::Task<crate::Scheme> {
    iced::system::theme().map(resolve_scheme)
}

/// Emits when the user switches between light and dark, and keeps [`color_scheme`] current.
/// Subscribe to it and map it to any message that redraws, so an app with
/// `[appearance] color-scheme=system` follows the desktop live.
pub fn follow_color_scheme() -> iced::Subscription<crate::Scheme> {
    iced::system::theme_changes().map(resolve_scheme)
}

/// The configured icon theme (`[appearance] icon-theme`), `hicolor` when not set.
pub fn icon_theme() -> String {
    match &crate::config::current().icon_theme {
        crate::config::Pick::Fixed(name) => name.clone(),
        _ => "hicolor".into(),
    }
}

/// Compatibility lookup at 16 logical pixels, scale 1.
pub fn find_icon(name: &str) -> Option<Vec<u8>> {
    crate::icon::resolve(name, 16, 1, name.ends_with("-symbolic"))
        .map(|icon| icon.bytes.as_ref().to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;
    use WindowButton::*;

    #[test]
    fn parses_gtk_button_layouts() {
        let l = DecorationLayout::parse("'appmenu:close'");
        assert_eq!((l.start, l.end), (vec![], vec![Close]));
        let l = DecorationLayout::parse("close,minimize,maximize:");
        assert_eq!((l.start, l.end), (vec![Close, Minimize, Maximize], vec![]));
        let l = DecorationLayout::parse("icon,close:minimize,maximize,spacer");
        assert_eq!((l.start, l.end), (vec![Close], vec![Minimize, Maximize]));
        // No colon: everything at the start.
        let l = DecorationLayout::parse("close");
        assert_eq!((l.start, l.end), (vec![Close], vec![]));
        assert!(DecorationLayout::parse("appmenu:").is_empty());
    }

    #[test]
    fn the_portal_mode_decides_the_scheme() {
        use iced::theme::Mode;
        assert_eq!(scheme_from_mode(Mode::Dark), Some(crate::Scheme::Dark));
        assert_eq!(scheme_from_mode(Mode::Light), Some(crate::Scheme::Light));
        assert_eq!(
            scheme_from_mode(Mode::None),
            None,
            "no preference is not a scheme"
        );
    }

    #[test]
    fn gnome_default_is_close_only_at_the_end() {
        let d = DecorationLayout::gnome_default();
        assert!(d.start.is_empty());
        assert_eq!(d.end, vec![Close]);
    }
}
