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
    ("libadwaita", include_str!("../../themes/libadwaita.conf")),
    ("cosmic", include_str!("../../themes/cosmic.conf")),
    ("bubblegum", include_str!("../../themes/bubblegum.conf")),
    ("solarized", include_str!("../../themes/solarized.conf")),
    ("synthwave", include_str!("../../themes/synthwave.conf")),
    ("terminal", include_str!("../../themes/terminal.conf")),
];

/// The text of the bundled theme `key`.
pub(crate) fn text(key: &str) -> Option<&'static str> {
    BUNDLED.iter().find(|(k, _)| *k == key).map(|(_, t)| *t)
}

/// A theme text's display name: its `[theme] name`, else `fallback`.
pub(crate) fn name_of(text: &str, fallback: &str) -> String {
    iad::config::parse_keyfile(text)
        .get(&("theme".to_string(), "name".to_string()))
        .filter(|n| !n.is_empty())
        .cloned()
        .unwrap_or_else(|| fallback.to_string())
}
