//! System-theme icons and bundled SVG artwork for entry controls.
mod appearance;
mod bundled;
mod chevron;
mod entry;
mod window;
pub use appearance::AppearanceIcon;
pub use chevron::ChevronIcon;
pub use entry::EntryIcon;
pub use window::WindowIcon;
mod resolver;
mod widget;
use crate::Theme;
use iced::{Color, widget::svg};
pub use resolver::{ResolvedIcon, available_themes, resolve};
pub use widget::IconView;

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
    /// A generic window icon.
    Window,
    /// Toggles a sidebar (`sidebar-show-symbolic`).
    Sidebar,
    OpenMenu,
    ListAdd,
    ListRemove,
    Search,
    ViewMore,
    DialogInformation,
    /// Information/about action, often available as a monochrome toolbar glyph.
    HelpAbout,
    DialogWarning,
    DialogError,
    Edit,
    Delete,
    /// A counterclockwise arrow: reset controls to their initial state.
    Reset,
    Settings,
    Home,
    Calendar,
    /// A funnel: filter a list (`funnel-symbolic` / `view-filter`).
    Filter,
    /// Bars that shrink: sort order (`view-sort-descending-symbolic`).
    Sort,
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
    /// A square: stop playback.
    Stop,
    /// A bar and a triangle pointing left (`media-skip-backward-symbolic`).
    SkipBackward,
    /// A bar and a triangle pointing right (`media-skip-forward-symbolic`).
    SkipForward,
    /// A die with five pips: a random choice or surprise.
    Dice,
    /// Two crossing arrows (`media-playlist-shuffle-symbolic`).
    Shuffle,
    /// Two arrows in a loop (`media-playlist-repeat-symbolic`).
    Repeat,
    /// Lines with a play mark: what plays next.
    Queue,
    /// A beamed note (`audio-x-generic-symbolic`).
    MusicNote,
    Heart,
    /// The same heart, filled: something marked as a favourite (pair it with [`Icon::Heart`]).
    HeartFilled,
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
    /// A speaker with sound waves: volume.
    Volume,
    /// A speaker with a cross: muted.
    VolumeMute,
    /// Four corners pointing out: enter fullscreen.
    Fullscreen,
    /// Four corners pointing in: leave fullscreen.
    FullscreenExit,
    /// A window with its bottom strip marked: the compact, one-line view.
    Compact,
    /// A push-pin: keep something at the top.
    Pin,
    /// A box with a lid: put away, out of the way.
    Archive,
    /// A painter's palette: choose a colour.
    Palette,
    /// An eye: look at it as it reads (a preview) rather than edit it.
    Eye,
    /// A note with a folded corner.
    Note,
    /// A circled check: finished things.
    CheckCircle,
    /// Three columns of different heights: a kanban board.
    Kanban,
    /// Bulleted lines, nothing nested: one flat list.
    FlatList,
    /// One line per item: compact rows rather than cards.
    CompactRows,
    /// Cards of different heights in columns: a masonry grid.
    CardGrid,
    /// A tag with a second one behind it: manage tags.
    Tags,
}

impl Icon {
    /// Every predefined icon, in declaration order (for galleries and pickers).
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
        Icon::HelpAbout,
        Icon::DialogWarning,
        Icon::DialogError,
        Icon::Edit,
        Icon::Delete,
        Icon::Reset,
        Icon::Settings,
        Icon::Home,
        Icon::Calendar,
        Icon::Filter,
        Icon::Sort,
        Icon::Folder,
        Icon::Document,
        Icon::DragHandle,
        Icon::Play,
        Icon::Pause,
        Icon::Stop,
        Icon::SkipBackward,
        Icon::SkipForward,
        Icon::Dice,
        Icon::Shuffle,
        Icon::Repeat,
        Icon::Queue,
        Icon::MusicNote,
        Icon::Heart,
        Icon::HeartFilled,
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
        Icon::Volume,
        Icon::VolumeMute,
        Icon::Fullscreen,
        Icon::FullscreenExit,
        Icon::Compact,
        Icon::Pin,
        Icon::Archive,
        Icon::Palette,
        Icon::Eye,
        Icon::Note,
        Icon::CheckCircle,
        Icon::Kanban,
        Icon::FlatList,
        Icon::CompactRows,
        Icon::CardGrid,
        Icon::Tags,
    ];
}

impl Icon {
    /// Base XDG identifier (conventional extension names where no standard name exists).
    pub fn name(self) -> &'static str {
        match self {
            Self::GoNext => "go-next",
            Self::GoPrevious => "go-previous",
            Self::GoDown => "go-down",
            Self::GoUp => "go-up",
            Self::GoUpDown => "view-sort-ascending",
            Self::GoFirst => "go-first",
            Self::GoLast => "go-last",
            Self::Check => "object-select",
            Self::WindowClose => "window-close",
            Self::WindowMinimize => "window-minimize",
            Self::WindowMaximize => "window-maximize",
            Self::WindowRestore => "window-restore",
            Self::Window => "window-new",
            Self::Sidebar => "sidebar-show",
            Self::OpenMenu => "open-menu",
            Self::ListAdd => "list-add",
            Self::ListRemove => "list-remove",
            Self::Search => "edit-find",
            Self::ViewMore => "view-more",
            Self::DialogInformation => "dialog-information",
            Self::HelpAbout => "help-about",
            Self::DialogWarning => "dialog-warning",
            Self::DialogError => "dialog-error",
            Self::Edit => "document-edit",
            Self::Delete => "edit-delete",
            Self::Reset => "edit-undo",
            Self::Settings => "preferences-system",
            Self::Home => "go-home",
            Self::Calendar => "x-office-calendar",
            Self::Filter => "view-filter",
            Self::Sort => "view-sort-descending",
            Self::Folder => "folder",
            Self::Document => "text-x-generic",
            Self::DragHandle => "list-drag-handle",
            Self::Play => "media-playback-start",
            Self::Pause => "media-playback-pause",
            Self::Stop => "media-playback-stop",
            Self::SkipBackward => "media-skip-backward",
            Self::SkipForward => "media-skip-forward",
            Self::Dice => "dice",
            Self::Shuffle => "media-playlist-shuffle",
            Self::Repeat => "media-playlist-repeat",
            Self::Queue => "view-list",
            Self::MusicNote => "audio-x-generic",
            Self::Heart => "emblem-favorite",
            Self::HeartFilled => "emblem-favorite",
            Self::Moon => "weather-clear-night",
            Self::Bolt => "thunderbolt",
            Self::Globe => "web-browser",
            Self::Tag => "tag",
            Self::Star => "starred",
            Self::Lyrics => "media-view-subtitles",
            Self::Compass => "find-location",
            Self::Sparkle => "star-new",
            Self::Clock => "preferences-system-time",
            Self::Target => "mark-location",
            Self::Smile => "face-smile",
            Self::Volume => "audio-volume-high",
            Self::VolumeMute => "audio-volume-muted",
            Self::Fullscreen => "view-fullscreen",
            Self::FullscreenExit => "view-restore",
            Self::Compact => "view-compact",
            Self::Pin => "view-pin",
            Self::Archive => "archive",
            Self::Palette => "applications-graphics",
            Self::Eye => "view-reveal",
            Self::Note => "note",
            Self::CheckCircle => "emblem-ok",
            Self::Kanban => "view-kanban",
            Self::FlatList => "view-list-flat",
            Self::CompactRows => "view-list-compact",
            Self::CardGrid => "view-grid",
            Self::Tags => "tag-new",
        }
    }
    /// Symbolic system identifier, retained for compatibility.
    pub fn system_name(self) -> Option<&'static str> {
        Some(match self {
            Self::GoNext => "go-next-symbolic",
            Self::GoPrevious => "go-previous-symbolic",
            Self::GoDown => "go-down-symbolic",
            Self::GoUp => "go-up-symbolic",
            Self::GoUpDown => "view-sort-ascending-symbolic",
            Self::GoFirst => "go-first-symbolic",
            Self::GoLast => "go-last-symbolic",
            Self::Check => "object-select-symbolic",
            Self::WindowClose => "window-close-symbolic",
            Self::WindowMinimize => "window-minimize-symbolic",
            Self::WindowMaximize => "window-maximize-symbolic",
            Self::WindowRestore => "window-restore-symbolic",
            Self::Window => "window-new-symbolic",
            Self::Sidebar => "sidebar-show-symbolic",
            Self::OpenMenu => "open-menu-symbolic",
            Self::ListAdd => "list-add-symbolic",
            Self::ListRemove => "list-remove-symbolic",
            Self::Search => "edit-find-symbolic",
            Self::ViewMore => "view-more-symbolic",
            Self::DialogInformation => "dialog-information-symbolic",
            Self::HelpAbout => "help-about-symbolic",
            Self::DialogWarning => "dialog-warning-symbolic",
            Self::DialogError => "dialog-error-symbolic",
            Self::Edit => "document-edit-symbolic",
            Self::Delete => "edit-delete-symbolic",
            Self::Reset => "edit-undo-symbolic",
            Self::Settings => "preferences-system-symbolic",
            Self::Home => "go-home-symbolic",
            Self::Calendar => "x-office-calendar-symbolic",
            Self::Filter => "view-filter-symbolic",
            Self::Sort => "view-sort-descending-symbolic",
            Self::Folder => "folder-symbolic",
            Self::Document => "text-x-generic-symbolic",
            Self::DragHandle => "list-drag-handle-symbolic",
            Self::Play => "media-playback-start-symbolic",
            Self::Pause => "media-playback-pause-symbolic",
            Self::Stop => "media-playback-stop-symbolic",
            Self::SkipBackward => "media-skip-backward-symbolic",
            Self::SkipForward => "media-skip-forward-symbolic",
            Self::Dice => "dice-symbolic",
            Self::Shuffle => "media-playlist-shuffle-symbolic",
            Self::Repeat => "media-playlist-repeat-symbolic",
            Self::Queue => "view-list-symbolic",
            Self::MusicNote => "audio-x-generic-symbolic",
            Self::Heart => "emblem-favorite-symbolic",
            Self::HeartFilled => "emblem-favorite-symbolic",
            Self::Moon => "weather-clear-night-symbolic",
            Self::Bolt => "thunderbolt-symbolic",
            Self::Globe => "web-browser-symbolic",
            Self::Tag => "tag-symbolic",
            Self::Star => "starred-symbolic",
            Self::Lyrics => "media-view-subtitles-symbolic",
            Self::Compass => "find-location-symbolic",
            Self::Sparkle => "star-new-symbolic",
            Self::Clock => "preferences-system-time-symbolic",
            Self::Target => "mark-location-symbolic",
            Self::Smile => "face-smile-symbolic",
            Self::Volume => "audio-volume-high-symbolic",
            Self::VolumeMute => "audio-volume-muted-symbolic",
            Self::Fullscreen => "view-fullscreen-symbolic",
            Self::FullscreenExit => "view-restore-symbolic",
            Self::Compact => "view-compact-symbolic",
            Self::Pin => "view-pin-symbolic",
            Self::Archive => "archive-symbolic",
            Self::Palette => "applications-graphics-symbolic",
            Self::Eye => "view-reveal-symbolic",
            Self::Note => "note-symbolic",
            Self::CheckCircle => "emblem-ok-symbolic",
            Self::Kanban => "view-kanban-symbolic",
            Self::FlatList => "view-list-flat-symbolic",
            Self::CompactRows => "view-list-compact-symbolic",
            Self::CardGrid => "view-grid-symbolic",
            Self::Tags => "tag-new-symbolic",
        })
    }
    /// Resolve this symbolic icon at the requested logical size and scale.
    pub fn resolve(self, size: u32, scale: u32) -> Option<ResolvedIcon> {
        resolve(self.name(), size, scale, true)
    }
    /// Installed SVG source, or an empty string for missing or raster assets.
    #[deprecated(note = "use resolve; system icons may be raster images")]
    pub fn source(self) -> String {
        self.resolve(16, 1)
            .filter(|i| i.path.extension().is_some_and(|e| e == "svg"))
            .and_then(|i| String::from_utf8(i.bytes.to_vec()).ok())
            .unwrap_or_default()
    }
    /// Installed SVG handle. Missing and raster assets produce an empty handle.
    #[deprecated(note = "use view or resolve to support all system icon formats")]
    pub fn handle(self) -> svg::Handle {
        #[allow(deprecated)]
        svg::Handle::from_memory(self.source().into_bytes())
    }
    pub fn view<'a>(self, size: f32) -> IconView<'a> {
        IconView::new(self.into(), size)
    }
    pub fn colored<'a>(self, size: f32, color: Color) -> IconView<'a> {
        self.view(size)
            .style(move |_: &Theme, _| svg::Style { color: Some(color) })
    }
}

/// A predefined system icon, a named system icon, or an application-supplied SVG.
#[derive(Debug, Clone)]
pub enum IconSource {
    Icon(Icon),
    Svg(svg::Handle),
    Named { name: String, symbolic: bool },
}
static ICON_OVERRIDES: std::sync::OnceLock<
    std::sync::RwLock<std::collections::HashMap<Icon, IconSource>>,
> = std::sync::OnceLock::new();

fn icon_overrides() -> &'static std::sync::RwLock<std::collections::HashMap<Icon, IconSource>> {
    ICON_OVERRIDES.get_or_init(Default::default)
}

/// Globally replaces a predefined icon for subsequently built views.
/// Pass `None` to restore its default bundled or system artwork.
/// Explicit SVGs and named system icons are unaffected.
pub fn set_icon_override(icon: Icon, source: Option<IconSource>) {
    let mut overrides = icon_overrides().write().unwrap();
    if let Some(source) = source {
        overrides.insert(icon, source);
    } else {
        overrides.remove(&icon);
    }
}

/// Restores default artwork for every predefined icon in subsequently built views.
pub fn clear_icon_overrides() {
    icon_overrides().write().unwrap().clear();
}

impl From<Icon> for IconSource {
    fn from(icon: Icon) -> Self {
        if let Some(source) = icon_overrides().read().unwrap().get(&icon).cloned() {
            return source;
        }
        icon.bundled_source()
            .map(Self::svg)
            .unwrap_or(Self::Icon(icon))
    }
}
impl From<svg::Handle> for IconSource {
    fn from(handle: svg::Handle) -> Self {
        Self::Svg(handle)
    }
}
impl IconSource {
    /// A regular icon from the configured system theme.
    pub fn named(name: impl Into<String>) -> Self {
        Self::Named {
            name: name.into(),
            symbolic: false,
        }
    }
    /// A symbolic icon, falling back to its regular counterpart.
    pub fn symbolic(name: impl Into<String>) -> Self {
        Self::Named {
            name: name.into(),
            symbolic: true,
        }
    }
    pub fn svg(source: impl Into<String>) -> Self {
        Self::Svg(svg::Handle::from_memory(source.into().into_bytes()))
    }
    pub fn view<'a>(&self, size: f32) -> IconView<'a> {
        IconView::new(self.clone(), size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn global_override_can_be_replaced_and_reset() {
        let icon = Icon::Compact;
        set_icon_override(icon, Some(IconSource::symbolic("custom-compact")));
        assert!(
            matches!(IconSource::from(icon), IconSource::Named { name, symbolic: true } if name == "custom-compact")
        );
        let svg = svg::Handle::from_memory(WindowIcon::Close.source().as_bytes());
        set_icon_override(icon, Some(svg.clone().into()));
        assert!(matches!(IconSource::from(icon), IconSource::Svg(handle) if handle == svg));
        set_icon_override(icon, None);
        assert!(matches!(
            IconSource::from(icon),
            IconSource::Icon(Icon::Compact)
        ));
    }
    #[test]
    fn every_variant_has_a_unique_mapping() {
        let mut seen = std::collections::HashSet::new();
        for icon in Icon::ALL {
            assert!(seen.insert(*icon));
            assert!(!icon.name().is_empty());
            assert_eq!(
                icon.system_name().unwrap(),
                format!("{}-symbolic", icon.name())
            );
        }
        assert!(seen.len() > 50);
    }
}
