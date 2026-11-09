//! Shared application utilities for global configuration and desktop appearance.
//!
//! Use [`theme`] to build from the installed configuration, run [`detect_theme`] at
//! startup, and subscribe to [`follow_theme`] to redraw when desktop preferences change.
//! Explicit configuration values take precedence over detected preferences.

use crate::{Scheme, Theme, config};
use iced::{Subscription, Task};
use std::time::Duration;

// Per-process user choice; never writes desktop settings or the shared config file.
static SCHEME_OVERRIDE: crate::system::AtomicScheme = crate::system::AtomicScheme::new();

/// Override the system/configured scheme for this application process.
/// `None` resumes following global settings. Returns false if app overrides are forbidden.
pub fn set_scheme_override(scheme: Option<Scheme>) -> bool {
    if scheme.is_some() && !config::current().allow_app_overrides {
        return false;
    }
    SCHEME_OVERRIDE.store(scheme);
    true
}

/// The application's requested override, or `None` when following global configuration.
pub fn scheme_override() -> Option<Scheme> {
    SCHEME_OVERRIDE.load()
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
        crate::system::desktop_preferences().icon_theme,
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
}
