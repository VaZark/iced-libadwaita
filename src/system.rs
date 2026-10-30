//! Reading the desktop's configuration (GNOME/GSettings and the icon theme).
//!
//! Adwaita apps follow the user's setup: which window buttons exist and on which
//! side (`org.gnome.desktop.wm.preferences button-layout`), and which icon theme
//! draws them (`org.gnome.desktop.interface icon-theme`). Everything here is
//! best effort and falls back to GNOME defaults. Results are cached; nothing is
//! re-read per frame.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Mutex, OnceLock};

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
        Self { start: buttons(start), end: buttons(end) }
    }

    /// The user's configured layout (cached), or [`gnome_default`](Self::gnome_default).
    pub fn detect() -> Self {
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
    let out = Command::new("gsettings").args(["get", schema, key]).output().ok()?;
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

/// The current icon theme name (`icon-theme`), default `Adwaita`.
pub fn icon_theme() -> String {
    static CACHE: OnceLock<String> = OnceLock::new();
    CACHE
        .get_or_init(|| {
            gsettings("org.gnome.desktop.interface", "icon-theme")
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| "Adwaita".to_string())
        })
        .clone()
}

fn data_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Some(h) = std::env::var_os("XDG_DATA_HOME") {
        dirs.push(PathBuf::from(h).join("icons"));
    } else if let Some(home) = std::env::var_os("HOME") {
        dirs.push(PathBuf::from(&home).join(".local/share/icons"));
    }
    if let Some(home) = std::env::var_os("HOME") {
        dirs.push(PathBuf::from(home).join(".icons"));
    }
    let xdg = std::env::var("XDG_DATA_DIRS").unwrap_or_else(|_| "/usr/local/share:/usr/share".into());
    dirs.extend(xdg.split(':').filter(|s| !s.is_empty()).map(|d| Path::new(d).join("icons")));
    dirs.push(PathBuf::from("/usr/share/icons"));
    dirs
}

/// `Inherits=` of a theme's `index.theme`.
fn inherits(theme_dir: &Path) -> Vec<String> {
    let Ok(text) = std::fs::read_to_string(theme_dir.join("index.theme")) else {
        return Vec::new();
    };
    text.lines()
        .find_map(|l| l.strip_prefix("Inherits="))
        .map(|v| v.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect())
        .unwrap_or_default()
}

/// Finds `<name>.svg` under `dir` (depth-limited), preferring symbolic and
/// 16px variants, which is how themes lay out their window icons.
fn find_svg(dir: &Path, name: &str, depth: u32, best: &mut Option<(u32, PathBuf)>) {
    if depth == 0 {
        return;
    }
    let Ok(rd) = std::fs::read_dir(dir) else { return };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            find_svg(&p, name, depth - 1, best);
        } else if p.file_name().and_then(|n| n.to_str()) == Some(&format!("{name}.svg")) {
            let s = p.to_string_lossy();
            let score = u32::from(s.contains("symbolic")) * 4
                + u32::from(s.contains("/16/") || s.contains("16x16")) * 2
                + u32::from(s.contains("/ui/") || s.contains("/actions/"));
            if best.as_ref().is_none_or(|(b, _)| score > *b) {
                *best = Some((score, p));
            }
        }
    }
}

/// Looks `name` (e.g. `window-close-symbolic`) up in the current icon theme and
/// its `Inherits` chain, falling back to Adwaita and hicolor. Returns the SVG
/// bytes, or `None` when no installed theme has it.
pub fn find_icon(name: &str) -> Option<Vec<u8>> {
    static CACHE: OnceLock<Mutex<HashMap<String, Option<Vec<u8>>>>> = OnceLock::new();
    let cache = CACHE.get_or_init(Default::default);
    if let Some(hit) = cache.lock().ok()?.get(name) {
        return hit.clone();
    }

    let dirs = data_dirs();
    let mut chain = vec![icon_theme()];
    let mut i = 0;
    while i < chain.len() && chain.len() < 12 {
        for base in &dirs {
            for parent in inherits(&base.join(&chain[i])) {
                if !chain.contains(&parent) {
                    chain.push(parent);
                }
            }
        }
        i += 1;
    }
    for fallback in ["Adwaita", "hicolor"] {
        if !chain.iter().any(|c| c == fallback) {
            chain.push(fallback.to_string());
        }
    }

    let mut found = None;
    'themes: for theme in &chain {
        for base in &dirs {
            let dir = base.join(theme);
            if !dir.is_dir() {
                continue;
            }
            let mut best = None;
            find_svg(&dir, name, 5, &mut best);
            if let Some((_, path)) = best
                && let Ok(bytes) = std::fs::read(path)
            {
                found = Some(bytes);
                break 'themes;
            }
        }
    }
    cache.lock().ok()?.insert(name.to_string(), found.clone());
    found
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
    fn gnome_default_is_close_only_at_the_end() {
        let d = DecorationLayout::gnome_default();
        assert!(d.start.is_empty());
        assert_eq!(d.end, vec![Close]);
    }
}
