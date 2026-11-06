//! Shared application utilities for global configuration and desktop appearance.
//!
//! Use [`theme`] to build from the installed configuration, run [`detect_theme`] at
//! startup, and subscribe to [`follow_theme`] to redraw when desktop preferences change.
//! Explicit configuration values take precedence over detected preferences.

use crate::{Scheme, Theme, config};
use iced::{Subscription, Task};
use std::{
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
    time::{Duration, Instant},
};

// Per-process user choice; never writes desktop settings or the shared config file.
static SCHEME_OVERRIDE: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(0);

/// Override the system/configured scheme for this application process.
/// `None` resumes following global settings. Returns false if app overrides are forbidden.
pub fn set_scheme_override(scheme: Option<Scheme>) -> bool {
    if scheme.is_some() && !config::current().allow_app_overrides {
        return false;
    }
    SCHEME_OVERRIDE.store(
        match scheme {
            None => 0,
            Some(Scheme::Light) => 1,
            Some(Scheme::Dark) => 2,
        },
        std::sync::atomic::Ordering::Relaxed,
    );
    true
}

/// The application's requested override, or `None` when following global configuration.
pub fn scheme_override() -> Option<Scheme> {
    match SCHEME_OVERRIDE.load(std::sync::atomic::Ordering::Relaxed) {
        1 => Some(Scheme::Light),
        2 => Some(Scheme::Dark),
        _ => None,
    }
}

pub(crate) fn effective_scheme(config: &config::Config, detected: Option<Scheme>) -> Scheme {
    scheme_with_override(config, detected, scheme_override())
}
fn scheme_with_override(
    config: &config::Config,
    detected: Option<Scheme>,
    app: Option<Scheme>,
) -> Scheme {
    if config.allow_app_overrides {
        if let Some(scheme) = app {
            return scheme;
        }
    }
    match config.color_scheme {
        config::Pick::Fixed(scheme) => scheme,
        config::Pick::System => detected.unwrap_or(Scheme::Light),
    }
}

/// Effective appearance after applying the installed global configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThemeSettings {
    pub scheme: Scheme,
    pub icon_theme: String,
}

/// Build the standard application theme from global configuration and detected preferences.
pub fn theme() -> Theme {
    Theme::from_config(&config::current())
}

/// Current effective appearance. Desktop settings are shared across the process.
pub fn theme_settings() -> ThemeSettings {
    let config = config::current();
    effective_settings(
        &config,
        crate::system::color_scheme(),
        desktop_preferences().icon_theme,
    )
}

fn effective_settings(
    config: &config::Config,
    scheme: Option<Scheme>,
    icons: Option<String>,
) -> ThemeSettings {
    ThemeSettings {
        scheme: effective_scheme(config, scheme),
        icon_theme: match &config.icon_theme {
            config::Pick::Fixed(value) => value.clone(),
            config::Pick::System => icons.unwrap_or_else(|| "hicolor".into()),
        },
    }
}

/// Detect the initial desktop scheme using iced's native settings/portal integration.
/// Reading the result also initializes shared toolkit icon-theme preferences.
pub fn detect_theme() -> Task<ThemeSettings> {
    crate::system::detect_color_scheme().map(|_| theme_settings())
}

/// Follow native color-scheme events and changes to desktop-neutral GTK settings files.
/// File preferences are checked every second; identical snapshots are not emitted repeatedly.
/// Map this subscription to an application message so the new theme is drawn.
pub fn follow_theme() -> Subscription<ThemeSettings> {
    Subscription::batch([
        crate::system::follow_color_scheme().map(|_| theme_settings()),
        Subscription::run(theme_file_changes),
    ])
}

fn theme_file_changes() -> impl iced::futures::Stream<Item = ThemeSettings> {
    iced::futures::stream::unfold(None, |mut previous| async move {
        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;
            let next = theme_settings();
            if previous.as_ref() != Some(&next) {
                previous = Some(next.clone());
                return Some((next, previous));
            }
        }
    })
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
    #[test]
    fn app_scheme_override_wins_until_cleared_without_changing_system_settings() {
        let mut config = config::Config::default();
        assert_eq!(
            scheme_with_override(&config, Some(Scheme::Dark), Some(Scheme::Light)),
            Scheme::Light
        );
        assert_eq!(
            scheme_with_override(&config, Some(Scheme::Dark), None),
            Scheme::Dark
        );
        config.color_scheme = config::Pick::Fixed(Scheme::Dark);
        assert_eq!(
            scheme_with_override(&config, Some(Scheme::Dark), Some(Scheme::Light)),
            Scheme::Light
        );
        config.allow_app_overrides = false;
        assert_eq!(
            scheme_with_override(&config, Some(Scheme::Dark), Some(Scheme::Light)),
            Scheme::Dark
        );
        assert_eq!(config.color_scheme, config::Pick::Fixed(Scheme::Dark));
    }

    #[test]
    fn explicit_global_configuration_wins_over_detected_theme_changes() {
        let mut config = config::Config::default();
        let system = effective_settings(&config, Some(Scheme::Dark), Some("breeze-dark".into()));
        assert_eq!(
            system,
            ThemeSettings {
                scheme: Scheme::Dark,
                icon_theme: "breeze-dark".into()
            }
        );
        let changed = effective_settings(&config, Some(Scheme::Light), Some("Papirus".into()));
        assert_ne!(system, changed);
        config.color_scheme = config::Pick::Fixed(Scheme::Light);
        config.icon_theme = config::Pick::Fixed("custom".into());
        assert_eq!(
            effective_settings(&config, Some(Scheme::Dark), Some("breeze-dark".into())),
            ThemeSettings {
                scheme: Scheme::Light,
                icon_theme: "custom".into()
            }
        );
        assert_eq!(
            effective_settings(&config::Config::default(), None, None),
            ThemeSettings {
                scheme: Scheme::Light,
                icon_theme: "hicolor".into()
            }
        );
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
