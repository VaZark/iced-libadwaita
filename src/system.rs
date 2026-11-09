//! Desktop preferences and desktop-neutral XDG icon lookup.
//!
//! Window layout and other existing desktop preferences remain best effort.
//! Appearance detection and updates are shared through [`crate::utils`]. Icons follow
//! desktop toolkit settings, without desktop-specific icon integrations.

use crate::Scheme;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

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

    /// The layout in effect: the desktop config's fixed `button-layout` if set, else the
    /// user's GNOME `button-layout` (cached), else [`gnome_default`](Self::gnome_default).
    pub fn detect() -> Self {
        if let crate::config::Pick::Fixed(s) = &crate::config::current().button_layout {
            return Self::parse(s);
        }
        static CACHE: OnceLock<DecorationLayout> = OnceLock::new();
        CACHE
            .get_or_init(|| {
                gsettings("org.gnome.desktop.wm.preferences", "button-layout")
                    .map(|s| Self::parse(&s))
                    .unwrap_or_else(Self::gnome_default)
            })
            .clone()
    }

    pub fn is_empty(&self) -> bool {
        self.start.is_empty() && self.end.is_empty()
    }
}

fn gsettings(schema: &str, key: &str) -> Option<String> {
    let out = Command::new("gsettings")
        .args(["get", schema, key])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8(out.stdout).ok()?;
    Some(s.trim().trim_matches('\'').to_string())
}

/// Whether the desktop wants animations (`org.gnome.desktop.interface enable-animations`,
/// GTK's `gtk-enable-animations`); default `true`. Cached.
pub fn animations_enabled() -> bool {
    static CACHE: OnceLock<bool> = OnceLock::new();
    *CACHE.get_or_init(|| {
        gsettings("org.gnome.desktop.interface", "enable-animations")
            .map(|v| v != "false")
            .unwrap_or(true)
    })
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
/// protocol), else from shared GTK settings files. `None` when neither is available.
pub fn color_scheme() -> Option<crate::Scheme> {
    if let Some(scheme) = PORTAL_SCHEME.load() {
        return Some(scheme);
    }
    desktop_preferences().scheme
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

/// The configured icon theme. `System` follows shared toolkit settings, falling back to `hicolor`.
pub fn icon_theme() -> String {
    match &crate::config::current().icon_theme {
        crate::config::Pick::Fixed(name) => name.clone(),
        _ => desktop_preferences()
            .icon_theme
            .unwrap_or_else(|| "hicolor".into()),
    }
}

/// Compatibility lookup at 16 logical pixels, scale 1.
pub fn find_icon(name: &str) -> Option<Vec<u8>> {
    crate::icon::resolve(name, 16, 1, name.ends_with("-symbolic"))
        .map(|icon| icon.bytes.as_ref().to_vec())
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct DesktopPreferences {
    pub icon_theme: Option<String>,
    pub scheme: Option<Scheme>,
}

fn settings_paths() -> Vec<PathBuf> {
    let mut roots: Vec<PathBuf> = std::env::var("XDG_CONFIG_DIRS")
        .unwrap_or_else(|_| "/etc/xdg".into())
        .split(':')
        .filter(|p| Path::new(p).is_absolute())
        .map(PathBuf::from)
        .collect();
    roots.reverse();
    roots.insert(0, PathBuf::from("/etc"));
    if let Some(home) = std::env::var_os("XDG_CONFIG_HOME").filter(|p| Path::new(p).is_absolute()) {
        roots.push(home.into());
    } else if let Some(home) = std::env::var_os("HOME") {
        roots.push(PathBuf::from(home).join(".config"));
    }
    roots
        .into_iter()
        .flat_map(|root| {
            [
                root.join("gtk-3.0/settings.ini"),
                root.join("gtk-4.0/settings.ini"),
            ]
        })
        .collect()
}

fn read_preferences(paths: &[PathBuf]) -> DesktopPreferences {
    let mut preferences = DesktopPreferences::default();
    for path in paths {
        if let Ok(text) = std::fs::read_to_string(path) {
            merge_preferences(&mut preferences, &text);
        }
    }
    preferences
}

fn merge_preferences(preferences: &mut DesktopPreferences, text: &str) {
    let mut settings = false;
    for line in text
        .lines()
        .map(str::trim)
        .filter(|l| !l.starts_with('#') && !l.starts_with(';'))
    {
        if line.starts_with('[') {
            settings = line == "[Settings]";
            continue;
        }
        if !settings {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let value = value.trim().trim_matches('"').trim_matches('\'');
        match key.trim() {
            "gtk-icon-theme-name"
                if !value.is_empty()
                    && value != "."
                    && value != ".."
                    && !value.contains(['/', '\\']) =>
            {
                preferences.icon_theme = Some(value.into())
            }
            "gtk-application-prefer-dark-theme" => match value {
                "true" | "1" => preferences.scheme = Some(Scheme::Dark),
                "false" | "0" => preferences.scheme = Some(Scheme::Light),
                _ => (),
            },
            _ => (),
        }
    }
}

/// Shared file snapshot, refreshed at most once per second rather than once per icon draw.
pub(crate) fn desktop_preferences() -> DesktopPreferences {
    static CACHE: OnceLock<Mutex<(Instant, DesktopPreferences)>> = OnceLock::new();
    let mut cache = CACHE
        .get_or_init(|| Mutex::new((Instant::now(), read_preferences(&settings_paths()))))
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    if cache.0.elapsed() >= Duration::from_secs(1) {
        *cache = (Instant::now(), read_preferences(&settings_paths()));
    }
    cache.1.clone()
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

    #[test]
    fn toolkit_preferences_are_section_aware_and_overlay_lower_priority_settings() {
        let mut settings = DesktopPreferences::default();
        merge_preferences(
            &mut settings,
            "[Settings]\ngtk-icon-theme-name=breeze-dark\ngtk-application-prefer-dark-theme=true\n",
        );
        assert_eq!(settings.icon_theme.as_deref(), Some("breeze-dark"));
        assert_eq!(settings.scheme, Some(Scheme::Dark));
        merge_preferences(
            &mut settings,
            "[Other]\ngtk-icon-theme-name=ignored\n[Settings]\ngtk-icon-theme-name=Papirus\ngtk-application-prefer-dark-theme=false\n",
        );
        assert_eq!(settings.icon_theme.as_deref(), Some("Papirus"));
        assert_eq!(settings.scheme, Some(Scheme::Light));
        merge_preferences(
            &mut settings,
            "[Settings]\ngtk-icon-theme-name=../bad\ngtk-application-prefer-dark-theme=invalid\n",
        );
        assert_eq!(settings.icon_theme.as_deref(), Some("Papirus"));
        assert_eq!(settings.scheme, Some(Scheme::Light));
    }
    #[test]
    fn absent_settings_do_not_invent_a_desktop_theme() {
        assert_eq!(read_preferences(&[]), DesktopPreferences::default());
    }
}
