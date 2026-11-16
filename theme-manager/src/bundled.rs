//! The default set of themes the app ships, embedded in the binary.
//!
//! DECISION (user, 2026-11-16): the set is the six general themes below. The toy reference themes
//! (`win95`, `winxp`, `win7`, `mate`, `gnome2`) belong to `theme-toys` and are not bundled here.
//!
//! A bundled theme is never edited in place: opening one gives an unsaved copy, and applying one
//! as the global theme copies it to the user's theme directory first.

use iced_adaptive_design as iad;

/// `(key, file text)`, in the order they are listed.
pub(crate) const BUNDLED: [(&str, &str); 6] = [
    ("libadwaita", include_str!("../../themes/libadwaita.ron")),
    ("cosmic", include_str!("../../themes/cosmic.ron")),
    ("bubblegum", include_str!("../../themes/bubblegum.ron")),
    ("solarized", include_str!("../../themes/solarized.ron")),
    ("synthwave", include_str!("../../themes/synthwave.ron")),
    ("terminal", include_str!("../../themes/terminal.ron")),
];

/// The text of the bundled theme `key`.
pub(crate) fn text(key: &str) -> Option<&'static str> {
    BUNDLED.iter().find(|(k, _)| *k == key).map(|(_, t)| *t)
}

/// A theme text's display name: its `[theme] name`, else `fallback`.
pub(crate) fn name_of(text: &str, fallback: &str) -> String {
    iad::theme_config::Document::parse(text)
        .ok()
        .and_then(|d| d.text("theme", "name").filter(|n| !n.is_empty()).map(String::from))
        .unwrap_or_else(|| fallback.to_string())
}
