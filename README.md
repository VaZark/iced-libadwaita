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

## Theming

Three layers, all overridable:

| Layer | Type | What it is |
|---|---|---|
| Named colors | `Palette` | libadwaita's `@define-color` names (`accent_bg_color`, `headerbar_bg_color`, `card_fg_color`, …) for light and dark |
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
* **Radii are capped at the GNOME HIG / Adwaita values** (4px small, 6px
  controls, 12px cards/dialogs; pill is separate). `Theme::with_metrics` clamps to
  `Metrics::MAX_RADIUS_*`; you can go smaller, not larger.
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
| ActionRow, SwitchRow, ComboRow, ExpanderRow, EntryRow, PasswordEntryRow, SpinRow, ButtonRow | `ActionRow`, `SwitchRow`, `ComboRow`, `ExpanderRow`, `EntryRow`, `password_entry_row`, `SpinRow`, `ButtonRow` | `boxed_list` for bare lists |
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
