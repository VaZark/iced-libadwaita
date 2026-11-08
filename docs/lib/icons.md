# System icons

General-purpose icons come from installed XDG themes, following the freedesktop [Icon Theme Specification](https://specifications.freedesktop.org/icon-theme/) and [Icon Naming Specification](https://specifications.freedesktop.org/icon-naming/latest/).

Select an installed theme in the shared desktop configuration:

```ini
[icons]
theme=Adwaita
```

`system` follows `gtk-icon-theme-name` from shared GTK settings files, with `hicolor` as the fallback when no preference is available. There is no forced Adwaita fallback. Reinstalling configuration with a different theme clears lookup and render caches on their next use; redraw the application to display it. Use `utils::detect_theme()` at startup and `utils::follow_theme()` as a subscription to follow desktop changes. GTK preference files are checked every second; changing their selected theme invalidates icon caches on the next draw. Edits to artwork inside an unchanged theme are not watched.

Lookup uses `$HOME/.icons`, `$XDG_DATA_HOME/icons` (default `$HOME/.local/share/icons`), `$XDG_DATA_DIRS/icons` (default `/usr/local/share/icons:/usr/share/icons`), and `/usr/share/pixmaps`. Theme metadata controls directories, inheritance, size and scale selection. Parents are searched recursively in order, followed by `hicolor`, then unthemed files. SVG, PNG and XPM are supported. No recursive filesystem guessing is used.

## Rendering and migration

```rust
use iced_adaptive_design::{Icon, IconSource};

let predefined = Icon::Search.view(16.0);
let symbolic = IconSource::symbolic("document-open").view(24.0).scale(2);
let regular = IconSource::named("folder").view(32.0);
let resolved = iced_adaptive_design::icon::resolve("document-open", 24, 2, true);
```

Symbolic requests try `<name>-symbolic` before `<name>`. Missing or unreadable icons return `None` from resolution and render as empty widgets with the requested dimensions. Themes need not provide every predefined icon. Applications should retain tooltips and accessible text for icon-only actions.

`view` and `colored` now return `IconView`, replacing concrete `Svg<Theme>` annotations. Existing width, height, content-fit, rotation, opacity, class and style builders remain available. `.weight(extra_pixels)` slightly thickens a system SVG without replacing its artwork. Set output scale with `.scale(integer_scale)` when the application knows its display scale; the default is 1. The resolver accepts logical size and scale explicitly.

Symbolic SVGs follow the inherited foreground, including button state, selection, headerbar foreground, and disabled opacity. Semantic `success`, `warning`, and `error` classes use the current theme's corresponding colors. A `.colored(size, color)` call or a style returning `color: Some(color)` overrides the base foreground. Symbolic raster icons use a foreground-colored alpha mask. A monochrome regular-name fallback for a symbolic request uses the foreground palette; full-color fallbacks retain their colors so interior details remain visible. Explicit regular requests (`IconSource::named`) retain the asset’s original colors. Application-supplied SVG handles retain monochrome tint behavior.

Surface foregrounds are contrast-adjusted when automatic contrast is enabled, including after tint changes. Explicit component foreground overrides remain authoritative. Disabled opacity is retained rather than preblended against a presumed background.

`source` and `handle` remain deprecated SVG-only adapters: they return installed SVG data or empty output when the asset is missing or raster. Use `resolve` for bytes and path, or `view` for all supported formats. Explicit application SVGs still work with `IconSource::svg` or `From<svg::Handle>`; the library supplies none.

## Predefined identifier mappings

Every existing enum variant is preserved. Names outside the standardized list are conventional theme extensions; their availability depends on the installed theme unless bundled defaults are provided below. Symbolic requests append `-symbolic` automatically.

| Variant | Base identifier |
| --- | --- |
| `GoNext` | `go-next` |
| `GoPrevious` | `go-previous` |
| `GoDown` | `go-down` |
| `GoUp` | `go-up` |
| `GoUpDown` | `view-sort-ascending` |
| `GoFirst` | `go-first` |
| `GoLast` | `go-last` |
| `Check` | `object-select` |
| `WindowClose` | `window-close` |
| `WindowMinimize` | `window-minimize` |
| `WindowMaximize` | `window-maximize` |
| `WindowRestore` | `window-restore` |
| `Window` | `window-new` |
| `Sidebar` | `sidebar-show` |
| `OpenMenu` | `open-menu` |
| `ListAdd` | `list-add` |
| `ListRemove` | `list-remove` |
| `Search` | `edit-find` |
| `ViewMore` | `view-more` |
| `DialogInformation` | `dialog-information` |
| `HelpAbout` | `help-about` |
| `DialogWarning` | `dialog-warning` |
| `DialogError` | `dialog-error` |
| `Edit` | `document-edit` |
| `Delete` | `edit-delete` |
| `Reset` | `edit-undo` |
| `Settings` | `preferences-system` |
| `Home` | `go-home` |
| `Calendar` | `x-office-calendar` |
| `Filter` | `view-filter` |
| `Folder` | `folder` |
| `Document` | `text-x-generic` |
| `DragHandle` | `list-drag-handle` |
| `Play` | `media-playback-start` |
| `Pause` | `media-playback-pause` |
| `SkipBackward` | `media-skip-backward` |
| `SkipForward` | `media-skip-forward` |
| `Dice` | `dice` |
| `Shuffle` | `media-playlist-shuffle` |
| `Repeat` | `media-playlist-repeat` |
| `Queue` | `view-list` |
| `MusicNote` | `audio-x-generic` |
| `Heart` | `emblem-favorite` |
| `Moon` | `weather-clear-night` |
| `Bolt` | `thunderbolt` |
| `Globe` | `web-browser` |
| `Tag` | `tag` |
| `Star` | `starred` |
| `Lyrics` | `media-view-subtitles` |
| `Compass` | `find-location` |
| `Sparkle` | `star-new` |
| `Clock` | `preferences-system-time` |
| `Target` | `mark-location` |
| `Smile` | `face-smile` |
| `Volume` | `audio-volume-high` |
| `VolumeMute` | `audio-volume-muted` |
| `Fullscreen` | `view-fullscreen` |
| `FullscreenExit` | `view-restore` |
| `Compact` | `view-compact` |

## Entry controls

`EntryIcon` provides bundled 16×16 SVG artwork for entry controls, so calendar,
step buttons and selection chevrons have consistent proportions on every system.
Use `EntryIcon::Search.view(16.0)` for a prefix, or pass an `EntryIcon` to
`section_icon`, `section_button` or `ActionEntry`. The set also includes text,
number, switch, multiselect, chips, multiline, clear, navigation and drag icons.
The gallery's Icons page previews the full set. Icons inherit the current text
color and scale from their square viewbox.

`AppearanceIcon::{System, Light, Dark}` supplies bundled monitor, sun and moon
SVGs for `ThemeToggle`. `Icon::Search` also uses the bundled search SVG, so the
header search button has the same proportions as entry search prefixes.

`ChevronIcon` supplies bundled left, right, up, down, up/down, first and last
SVGs. The shared `IconSource` conversion applies them to `Icon::GoNext`,
`GoPrevious`, `GoUp`, `GoDown`, `GoUpDown`, `GoFirst` and `GoLast`, so navigation,
entry rows, disclosure controls and sorting use consistent artwork. Explicitly
named system icons still follow the installed theme.

The standard `widgets::ThemeToggle` is the gallery's top-bar appearance switcher:

```rust,ignore
use iced_adaptive_design::widgets::{ThemeMode, ThemeToggle};

ThemeToggle::new(ThemeMode::from(iced_adaptive_design::utils::scheme_override()))
    .on_toggle(Message::Appearance)
    .metrics(metrics)
```

Handle the message with `utils::set_scheme_override(mode.scheme_override())`.
The headless `primitives::appearance::ThemeMode` owns the System → Light → Dark
cycle and override conversion. The widget supplies artwork, tooltip and disabled
behavior. The previous `components` exports remain available.

Window controls use bundled `WindowIcon::{Close, Minimize, Maximize, Restore}`
artwork through the shared `IconSource` conversion. `EntryIcon::Clear` shares the
close SVG. Override individual header-bar symbols with custom SVGs or system names:

```rust,ignore
use iced_adaptive_design::{IconSource, widgets::{HeaderBar, WindowControlIcons}};

HeaderBar::new().window_control_icons(WindowControlIcons {
    close: IconSource::svg(include_str!("my-close.svg")),
    minimize: IconSource::symbolic("window-minimize"),
    ..Default::default()
})
```

Unspecified symbols retain the bundled defaults; button layout and actions follow
`WindowControls` as before.

Set application-wide replacements for any predefined `Icon` before building views:

```rust,ignore
use iced_adaptive_design::{Icon, IconSource, icon::set_icon_override};

set_icon_override(Icon::WindowClose, Some(IconSource::svg(include_str!("close.svg"))));
set_icon_override(Icon::WindowMinimize, Some(IconSource::symbolic("window-minimize")));
set_icon_override(Icon::WindowClose, None); // restore this default
iced_adaptive_design::icon::clear_icon_overrides(); // restore all defaults
```

Overrides apply when constructing views, including default header-bar controls.
Explicit SVGs, named system icons and per-header custom artwork stay explicit.

Bundled common artwork is available through existing `Icon` variants: `ListAdd`
(plus), `Delete` (trash), `DialogInformation` and `HelpAbout` (circled i), `Edit`
(pencil), `ViewMore` (three dots), `Play`, `Pause`, `Stop`, `SkipForward`,
`SkipBackward`, `Heart`, `Volume`, `VolumeMute`, `Fullscreen`, and `FullscreenExit`
(return to a window). `WindowRestore` retains its bundled overlapping-window SVG.
These defaults use the same 16px viewbox and follow the parent's foreground.
`icon::set_icon_override` replaces any of them globally; explicitly named system
icons still use the installed theme. `Icon::bundled_source()` exposes the original
SVG when one exists.
