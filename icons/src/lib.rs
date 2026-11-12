//! # iad-icons
//!
//! The bundled artwork of `iced-adaptive-design`: 16×16 SVG glyphs drawn with a 1.5px round
//! stroke in `#000`, which a toolkit recolours to the current foreground. This crate is only
//! data (no GUI toolkit), so anything that wants the same glyphs can use it: another toolkit,
//! a documentation build, a tool that exports icon sheets.
//!
//! ```
//! assert!(iad_icons::common::ADD.starts_with("<svg"));
//! assert!(iad_icons::all().any(|(name, _)| name == "app/pin"));
//! ```

/// General-purpose glyphs: add, delete, edit, filter, media, volume, favourites.
pub mod common {
    pub const ADD: &str = include_str!("../svg/common/add.svg");
    pub const DELETE: &str = include_str!("../svg/common/delete.svg");
    pub const EDIT: &str = include_str!("../svg/common/edit.svg");
    pub const FILTER: &str = include_str!("../svg/common/filter.svg");
    pub const FULLSCREEN_EXIT: &str = include_str!("../svg/common/fullscreen-exit.svg");
    pub const FULLSCREEN: &str = include_str!("../svg/common/fullscreen.svg");
    pub const HEART_FILLED: &str = include_str!("../svg/common/heart-filled.svg");
    pub const HEART: &str = include_str!("../svg/common/heart.svg");
    pub const INFORMATION: &str = include_str!("../svg/common/information.svg");
    pub const OPEN_MENU: &str = include_str!("../svg/common/open-menu.svg");
    pub const PAUSE: &str = include_str!("../svg/common/pause.svg");
    pub const PLAY: &str = include_str!("../svg/common/play.svg");
    pub const SETTINGS: &str = include_str!("../svg/common/settings.svg");
    /// A window with its sidebar marked: show or hide the sidebar or drawer.
    pub const SIDEBAR: &str = include_str!("../svg/common/sidebar.svg");
    pub const SKIP_BACKWARD: &str = include_str!("../svg/common/skip-backward.svg");
    pub const SKIP_FORWARD: &str = include_str!("../svg/common/skip-forward.svg");
    pub const SORT: &str = include_str!("../svg/common/sort.svg");
    pub const STOP: &str = include_str!("../svg/common/stop.svg");
    pub const VIEW_MORE: &str = include_str!("../svg/common/view-more.svg");
    pub const VOLUME_MUTE: &str = include_str!("../svg/common/volume-mute.svg");
    pub const VOLUME: &str = include_str!("../svg/common/volume.svg");
}

/// Navigation and disclosure chevrons.
pub mod chevron {
    pub const DOWN: &str = include_str!("../svg/chevron/down.svg");
    pub const FIRST: &str = include_str!("../svg/chevron/first.svg");
    pub const LAST: &str = include_str!("../svg/chevron/last.svg");
    pub const LEFT: &str = include_str!("../svg/chevron/left.svg");
    pub const RIGHT: &str = include_str!("../svg/chevron/right.svg");
    pub const UP_DOWN: &str = include_str!("../svg/chevron/up-down.svg");
    pub const UP: &str = include_str!("../svg/chevron/up.svg");
}

/// Window controls.
pub mod window {
    pub const CLOSE: &str = include_str!("../svg/window/close.svg");
    pub const MAXIMIZE: &str = include_str!("../svg/window/maximize.svg");
    pub const MINIMIZE: &str = include_str!("../svg/window/minimize.svg");
    pub const RESTORE: &str = include_str!("../svg/window/restore.svg");
}

/// Artwork for entry controls: prefixes, suffixes and actions.
pub mod entry {
    pub const ACTION: &str = include_str!("../svg/entry/action.svg");
    pub const CALENDAR: &str = include_str!("../svg/entry/calendar.svg");
    pub const CHIPS: &str = include_str!("../svg/entry/chips.svg");
    pub const DECREMENT: &str = include_str!("../svg/entry/decrement.svg");
    pub const DRAGHANDLE: &str = include_str!("../svg/entry/draghandle.svg");
    pub const INCREMENT: &str = include_str!("../svg/entry/increment.svg");
    pub const MULTILINE: &str = include_str!("../svg/entry/multiline.svg");
    pub const MULTISELECT: &str = include_str!("../svg/entry/multiselect.svg");
    pub const NUMBER: &str = include_str!("../svg/entry/number.svg");
    pub const SEARCH: &str = include_str!("../svg/entry/search.svg");
    pub const SELECT: &str = include_str!("../svg/entry/select.svg");
    pub const SWITCH: &str = include_str!("../svg/entry/switch.svg");
    pub const TEXT: &str = include_str!("../svg/entry/text.svg");
}

/// System, light and dark appearance.
pub mod appearance {
    pub const DARK: &str = include_str!("../svg/appearance/dark.svg");
    pub const LIGHT: &str = include_str!("../svg/appearance/light.svg");
    pub const SYSTEM: &str = include_str!("../svg/appearance/system.svg");
}

/// Glyphs for apps built on the toolkit: pins, archives, palettes, boards, lists.
pub mod app {
    /// A solid folder (the one filled glyph: it takes a colour of its own well).
    pub const FOLDER: &str = include_str!("../svg/app/folder.svg");
    /// A calendar page with today marked (filled).
    pub const CALENDAR: &str = include_str!("../svg/app/calendar.svg");
    pub const ARCHIVE: &str = include_str!("../svg/app/archive.svg");
    pub const CARD_GRID: &str = include_str!("../svg/app/card-grid.svg");
    pub const CHECK_CIRCLE: &str = include_str!("../svg/app/check-circle.svg");
    pub const COMPACT_ROWS: &str = include_str!("../svg/app/compact-rows.svg");
    pub const EYE: &str = include_str!("../svg/app/eye.svg");
    pub const FLAT_LIST: &str = include_str!("../svg/app/flat-list.svg");
    pub const KANBAN: &str = include_str!("../svg/app/kanban.svg");
    pub const NOTE: &str = include_str!("../svg/app/note.svg");
    pub const PALETTE: &str = include_str!("../svg/app/palette.svg");
    pub const PIN: &str = include_str!("../svg/app/pin.svg");
    pub const TAGS: &str = include_str!("../svg/app/tags.svg");
}

/// Every glyph as `("group/name", svg)`, in a stable order.
pub fn all() -> impl Iterator<Item = (&'static str, &'static str)> {
    [
        ("common/add", common::ADD),
        ("common/delete", common::DELETE),
        ("common/edit", common::EDIT),
        ("common/filter", common::FILTER),
        ("common/fullscreen-exit", common::FULLSCREEN_EXIT),
        ("common/fullscreen", common::FULLSCREEN),
        ("common/heart-filled", common::HEART_FILLED),
        ("common/heart", common::HEART),
        ("common/information", common::INFORMATION),
        ("common/open-menu", common::OPEN_MENU),
        ("common/pause", common::PAUSE),
        ("common/play", common::PLAY),
        ("common/settings", common::SETTINGS),
        ("common/sidebar", common::SIDEBAR),
        ("common/skip-backward", common::SKIP_BACKWARD),
        ("common/skip-forward", common::SKIP_FORWARD),
        ("common/sort", common::SORT),
        ("common/stop", common::STOP),
        ("common/view-more", common::VIEW_MORE),
        ("common/volume-mute", common::VOLUME_MUTE),
        ("common/volume", common::VOLUME),
        ("chevron/down", chevron::DOWN),
        ("chevron/first", chevron::FIRST),
        ("chevron/last", chevron::LAST),
        ("chevron/left", chevron::LEFT),
        ("chevron/right", chevron::RIGHT),
        ("chevron/up-down", chevron::UP_DOWN),
        ("chevron/up", chevron::UP),
        ("window/close", window::CLOSE),
        ("window/maximize", window::MAXIMIZE),
        ("window/minimize", window::MINIMIZE),
        ("window/restore", window::RESTORE),
        ("entry/action", entry::ACTION),
        ("entry/calendar", entry::CALENDAR),
        ("entry/chips", entry::CHIPS),
        ("entry/decrement", entry::DECREMENT),
        ("entry/draghandle", entry::DRAGHANDLE),
        ("entry/increment", entry::INCREMENT),
        ("entry/multiline", entry::MULTILINE),
        ("entry/multiselect", entry::MULTISELECT),
        ("entry/number", entry::NUMBER),
        ("entry/search", entry::SEARCH),
        ("entry/select", entry::SELECT),
        ("entry/switch", entry::SWITCH),
        ("entry/text", entry::TEXT),
        ("appearance/dark", appearance::DARK),
        ("appearance/light", appearance::LIGHT),
        ("appearance/system", appearance::SYSTEM),
        ("app/archive", app::ARCHIVE),
        ("app/calendar", app::CALENDAR),
        ("app/folder", app::FOLDER),
        ("app/card-grid", app::CARD_GRID),
        ("app/check-circle", app::CHECK_CIRCLE),
        ("app/compact-rows", app::COMPACT_ROWS),
        ("app/eye", app::EYE),
        ("app/flat-list", app::FLAT_LIST),
        ("app/kanban", app::KANBAN),
        ("app/note", app::NOTE),
        ("app/palette", app::PALETTE),
        ("app/pin", app::PIN),
        ("app/tags", app::TAGS),
    ]
    .into_iter()
}

/// The SVG for a `"group/name"` as listed by [`all`].
pub fn get(name: &str) -> Option<&'static str> {
    all().find(|(n, _)| *n == name).map(|(_, svg)| svg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_glyph_is_a_16_by_16_stroked_svg_that_a_toolkit_can_recolour() {
        let mut seen = std::collections::HashSet::new();
        for (name, svg) in all() {
            assert!(seen.insert(name), "{name} listed twice");
            assert!(svg.starts_with("<svg ") && svg.trim_end().ends_with("</svg>"), "{name}");
            assert!(svg.contains(r#"viewBox="0 0 16 16""#), "{name}: not 16x16");
            assert!(!svg.contains("currentColor"), "{name}: fixed colour is #000, recoloured by the toolkit");
        }
        assert!(seen.len() >= 50, "found only {}", seen.len());
    }

    #[test]
    fn lookup_by_name() {
        assert_eq!(get("app/pin"), Some(app::PIN));
        assert_eq!(get("app/nope"), None);
    }
}
