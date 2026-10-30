# iced-libadwaita

libadwaita-style theming and widgets for [iced](https://iced.rs) 0.14, aiming to
expose the same elements as [libadwaita's widget gallery](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/widget-gallery.html)
so that porting a GTK/libadwaita UI is mostly a rename.

> **Disclosure.** This is primarily for the author's personal project and is
> **managed primarily by AI** (Claude). Treat it as a logical fork of the
> libadwaita *design*, not an official or GNOME-endorsed project. It reimplements
> the look and widget vocabulary in Rust/iced; it contains no libadwaita code
> and is not affiliated with the GNOME project. Expect rough edges, API churn,
> and values that are *approximated* from libadwaita's stylesheet rather than
> copied. Review before depending on it.

## Quick start

```rust
use iced_libadwaita as adw;
use adw::widget as w;

fn view(&self) -> iced::Element<'_, Message, adw::Theme> {
    w::toolbar_view(
        w::PreferencesPage::new().push(
            w::PreferencesGroup::new()
                .title("General")
                .push(w::SwitchRow::new("Dark Style", self.dark, Message::Dark))
                .push(w::ActionRow::new("About").chevron(true).on_activate(Message::About)),
        ),
    )
    .top_bar(w::header_bar().title("My App"))
    .into()
}

// iced::application(boot, update, view).theme(|s| adw::Theme::dark()).run()
```

Use `adw::Theme` as the application theme: it implements iced's `theme::Base`
and every widget `Catalog`, so stock iced widgets (`button`, `text_input`,
`toggler`, `pick_list`, `scrollable`, …) are restyled too. Run
`cargo run --example gallery` for a tour (`-- --screenshot DIR` renders every
page in light and dark).

## Design reference

`docs/` holds the design language and one file per libadwaita element,
distilled from the official docs (colors, style classes, adaptive rules, CSS
nodes) with an *iced mapping* section recording how each is built and where we
deviate. Start at [docs/README.md](docs/README.md). Sizes the docs don't give
are marked ‡ (from memory of the stylesheet, unverified).

`docs/gtk4/` covers the underlying GTK4 widgets (scrollbars, entries, popovers…);
where GTK4 and libadwaita disagree, libadwaita wins.

## Windows: no server-side decorations

Adwaita dislikes SSD, so it is **off by default**. Use the window helpers and
draw your own header bar:

```rust
iced::application(boot, update, view)
    .window(adw::window::settings())            // decorations: false, min 360x200
    .run()

// view: header bar carries title + controls + drag; frame adds rounded corners and resize grips.
// `state` is maximized/fullscreen (track it with adw::window::changes() + refresh()):
// floating windows get 15px corners, maximized/fullscreen go edge to edge.
let corner = state.radius(&adw::Metrics::default());
adw::window::frame(
    w::toolbar_view(content).top_bar(
        w::header_bar()
            .title("My App")
            .window_radius(corner)                    // round the header's top corners
            .on_drag(Msg::Drag)                       // iced::window::drag
            .on_double_click(Msg::ToggleMaximize)
            .window_controls(w::WindowControls { on_minimize: Some(Msg::Min), on_maximize: Some(Msg::Max), on_close: Some(Msg::Close) }),
    ),
    state,
    Msg::Resize,                                      // iced::window::drag_resize
)
```

`adw::window::settings_with_ssd(true)` opts back in (also call
`Theme::with_transparent_window(false)`). The theme is transparent behind the root by
default so rounded corners show; use `navigation_sidebar_rounded` for a sidebar that
touches the bottom-left corner.

## Window buttons and menus

* **Window buttons follow the desktop.** Which buttons appear and on which side comes
  from GNOME's `button-layout` (default `appmenu:close`: a lone close button at the
  end), and their icons from the current icon theme (`window-close-symbolic`, …,
  falling back to built-in drawings). In **mobile mode** (`window::is_mobile`, width
  ≤ 450) they are hidden completely; corners still follow the real window state
  (rounded while floating, edge to edge only when maximized or fullscreen). Dialog headers
  show only a close button and are flat (no color of their own), so they flow into
  the dialog body like Nautilus's dialogs.
* **Primary menu:** `w::primary_menu(Menu::new("Main Menu").section([..]))` gives the
  HIG menu button. Present it as a popover or a full page:
  `.presentation(MenuPresentation::Dropdown | FullPage)`; `MenuPresentation::for_width(w)`
  picks full page at ≤ 450. See `docs/design-language/menus.md` for the conventions
  (GNOME HIG, Material, `AdwDialog`) behind this.
* **Continuous header bars** (Nautilus-style, no header color of their own):
  `Theme::with_flat_headerbars(true)`, or `HeaderBar::flat(true)` per bar; dialog
  headers are always flat. Titles are replaceable (`title_widget`, e.g. `w::path_bar`);
  only the window buttons are fixed.
* **Rounded windows stay contained:** every corner-touching surface and full-window layer
  takes the window radius (`NavigationSplitView::window_radius`, `dialog_in_window`,
  `bottom_sheet_in_window`, `HeaderBar::window_corners`), and the outline is drawn above the
  content. Maximized/fullscreen goes edge to edge.
* In a split view give each pane its own header bar; only the pane at a window edge
  shows the window buttons.

## Responsiveness

Follows libadwaita's rules (see `docs/design-language/adaptive-layouts.md`):
split views collapse at `max-width: 400` (`NavigationSplitView::adapt(width)`),
sidebars are 25% clamped to 180–280, the minimum window is 360×200, button and
header labels never wrap, row text wraps without ever clipping, and the header
bar keeps window controls reachable at any width, and every row flex-wraps its
suffix control below the title when the two can't share a line
(`Metrics::row_wrap_basis`, `ActionRow::wrap_suffix(false)` to opt out). When it wraps
the control goes 100% width (`CollapseBehavior::Stacked`, default) or keeps its width
(`Flex`); set it globally via `Metrics::row_collapse` or per row with `.collapse(..)`.
The building block is `adw::widget::FlexRow`. iced cannot ellipsize text, so
overflowing text is clipped at its container.

## Theming

Three layers, all overridable:

| Layer | Type | What it is |
|---|---|---|
| Named colors | `Palette` | libadwaita's CSS variable names (`accent_bg_color`, `headerbar_bg_color`, `card_fg_color`, …), values from the 1.11 docs, light and dark |
| Semantic | `Semantic` | Roles widgets actually ask for: `window`/`view`/`card`/`dialog`/… `Surface`s, `accent`/`destructive`/`success`/`warning`/`error` `Swatch`es, `border`, `opacity` (control/hover/disabled/dim alphas) |
| Metrics | `Metrics`, `Typography` | Padding, spacing, radii, min heights, row/headerbar/sidebar sizes, text scale |

```rust
let theme = adw::Theme::dark()
    .accent(adw::AccentColor::Purple)                 // system accent
    .with_palette(|p| p.window_bg_color = my_bg)      // named color; semantic re-derived
    .with_semantic(|s| s.card.bg = my_card)           // semantic override (apply after palette)
    .with_metrics(adw::Metrics::compact());           // density
```

* **Roles** (`Role::{Accent, Destructive, Success, Warning, Error}`) drive
  `style::button::role`, `style::container::{role, tinted}`, row coloring, etc.
* **Radii are capped at the GNOME HIG / Adwaita values** (4px small, 9px
  controls and menus, 12px cards, 15px popovers/windows/dialogs; pill is separate). `Theme::with_metrics` clamps to
  `Metrics::MAX_RADIUS_*`; you can go smaller, not larger.
* **Shadows are settings** (`Theme::with_shadows`): `Shadows::none()`, `.with_scale(..)`,
  `.with_intensity(..)`, or replace any `Elevation`. Defaults are libadwaita's
  layered shadows (card, popover, floating dialog, OSD); see `docs/design-language/shadows.md`.
  iced draws one shadow per quad, so stacked layers are merged (1px ring as the
  border + one blurred shadow).
* **Scrollbars** follow Adwaita/GTK4 overlay scrolling: use `adw::widget::scrolled_window(..)`
  (thin floating scrollbar, slider 20/40/60% by state, 1px outline).
* **Style classes** live in `adw::style::{button, container, text, text_input, …}`
  as plain `fn(&Theme[, Status]) -> Style`: `suggested`, `destructive`, `flat`,
  `pill`, `circular`, `card`, `osd`, `link`, `list_row`, `sidebar_row`, `toggle`,
  `container::{card, boxed_list, dialog, popover, osd, toast, banner}`,
  `text::{dim, accent, success, warning, error}`.

### Known limitation: build-time metrics

iced resolves styles at draw time but layout (padding, spacing, min heights) at
view-build time, and widget builders do not see the theme. Colors, radii,
opacities and borders therefore follow the active `Theme` automatically, while
padding/size tokens use `Metrics::default()` unless you pass `.metrics(m)` to the
widget (the gallery does this). Typography is the same: use
`Typography::title_1(..)` etc. for non-default sizes.

## Widget coverage

Mirrors the libadwaita gallery. Widgets are stateless builders; the app owns
state (expanded, current page, open sheet…) and handles the messages.

| libadwaita | here | Notes |
|---|---|---|
| AdwStatusPage, AdwBanner, AdwAvatar, AdwSpinner, AdwToastOverlay | `status_page`, `banner`, `avatar`, `spinner`, `toast_overlay` + `Toast` | Spinner needs a frame-driven `rotation`; toasts are cleared by the app |
| AdwShortcutLabel | `shortcut_label` | |
| AdwToggleGroup | `toggle_group` / `ToggleItem` | |
| AdwDialog, AdwAlertDialog, AdwAboutDialog, AdwShortcutsDialog | `dialog`, `AlertDialog`, `AboutDialog`, `shortcuts_dialog` | Modal over a `stack`, backdrop click optional |
| ActionRow, SwitchRow, ComboRow, ExpanderRow, EntryRow, PasswordEntryRow, SpinRow, ButtonRow | `ActionRow`, `SwitchRow`, `ComboRow` (custom popover `Dropdown`), `ExpanderRow`, `EntryRow`, `password_entry_row`, `SpinRow`, `ButtonRow` | `boxed_list` for bare lists; `Switch` is a 46×26 custom widget |
| AdwPreferencesGroup / Page / Dialog | `PreferencesGroup`, `PreferencesPage`, `preferences_dialog` | |
| AdwNavigationView, NavigationSplitView, OverlaySplitView | `navigation_view`, `NavigationSplitView`, `OverlaySplitView` | Push/pop is an app-owned stack |
| AdwSidebar, ViewSwitcherSidebar | `sidebar` / `navigation_sidebar` + `sidebar_row` | |
| AdwViewSwitcher, ViewSwitcherBar, InlineViewSwitcher | `view_switcher`, `view_switcher_bar`, `inline_view_switcher` | |
| AdwCarousel, indicators | `carousel`, `carousel_indicator_dots/lines` | Indicator-driven; no swipe gesture |
| AdwTabBar, TabButton | `tab_bar`, `tab_button` | No drag-reorder |
| AdwBottomSheet | `bottom_sheet` | No drag gesture |
| AdwBreakpointBin, Breakpoint | `breakpoint_bin`, `Breakpoint` | Width conditions only |
| AdwClamp, WrapBox, Bin | `clamp`, `wrap_box`, `bin` | |
| AdwToolbarView, WindowTitle, HeaderBar, Window, SplitButton, ButtonContent | `toolbar_view`, `window_title`, `HeaderBar`, `window`, `split_button`, `ButtonContent` | `HeaderBar` has `on_drag`/`on_double_click`/`WindowControls` for client-side decorations |
| AdwTabOverview | — | not implemented |

Not covered: dynamic breakpoint *setters*, animations (`AdwAnimation`),
`AdwStyleManager` system color-scheme/accent tracking, symbolic icon theme
lookup (a small built-in SVG set, `Icon`, is provided), markdown/table catalogs.

## Status

* Builds against iced 0.14 with the `advanced`, `svg`, `canvas` features.
* Verified by rendering the gallery (`examples/gallery.rs`) in light and dark and
  inspecting the output; `cargo test` covers theme token behavior.
* `iced::window::screenshot` does not capture `text_editor` text (reproduced on
  stock iced with no code from this crate); the editor is styled but cannot be
  confirmed by screenshot.
* No license has been chosen yet. Decide before publishing.
