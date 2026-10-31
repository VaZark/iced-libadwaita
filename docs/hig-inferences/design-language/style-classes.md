# Style classes

Source: [style classes](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/style-classes.html). All of these have a style
function in `iced_libadwaita::style` (see mapping column).

| Class | Meaning | Combines with | iced |
|---|---|---|---|
| `.suggested-action` | Accent-colored important button | circular, pill; MenuButton, SplitButton, ButtonRow | `style::button::suggested` |
| `.destructive-action` | Destructive colors | flat, circular, pill | `style::button::destructive` |
| `.flat` | Looks like a label/icon until hovered; toolbar default | circular, pill | `style::button::flat` |
| `.raised` | Regular look inside toolbars | circular, pill | `style::button::raised` |
| `.circular` | Round; 1–2 chars or icon | suggested, destructive, flat, raised, osd | `circular`, `circular_flat` |
| `.pill` | Pill-shaped standalone button (status pages) | same as circular | `pill`, `suggested_pill` |
| `.linked` | Group controls with no spacing, shared corners | not flat/suggested | `button::linked(pos)` |
| `.toolbar` | Bars: 6px margins and spacing; icon-only, icon+label, menu, split and dropdown buttons turn flat, text-only buttons stay raised | | `toolbar_view`, `HeaderBar` |
| `.spacer` | Invisible separator in toolbars | | `space()` |
| `.dimmed` | `--dim-opacity` | | `style::text::dim` |
| `.title-N`, `.heading`, `.caption*`, `.body`, `.monospace`, `.numeric`, `.document` | Typography | | `widget::typography` |
| `.accent` `.success` `.warning` `.error` | Colors (also retint accent) | any widget, entries for validation | `style::text::*`, `text_input::error` |
| `.boxed-list` | ListBox (no selection) as one card with separators | | `boxed_list` |
| `.boxed-list-separate` | Each row its own card | | `PreferencesGroup::separate_rows` |
| `.card` | Card appearance; with `.activatable` gets hover/active | | `style::container::card`, `button::card` |
| `.navigation-sidebar` | Rounded, padded items; **neutral** selection color; no list background | | `navigation_sidebar`, `sidebar_row` |
| `.osd` | Dark translucent, white accent; overlay buttons larger; floating toolbars padded/rounded; progress bars thinner, no trough | circular, pill | `style::container::osd`, `button::osd` |
| `.background` / `.view` / `.frame` | window / view colors / `1px solid --border-color` | | `container::{window,view}` |
| `.compact` | Smaller `AdwStatusPage` (sidebars/popovers) | | `StatusPage::compact` |
| `.menu` | Popover with list → menu look | | `dropdown` popover |
| `.inline` | Neutral (non-headerbar) look for SearchBar/TabBar/TextView | | `tab_bar` |
| `.property` | ActionRow/ExpanderRow: de-emphasize title, emphasize subtitle (read-only value) | monospace (subtitle only) | `ActionRow::property` |
| `.selection-mode` | Larger round check button | | n/a |
| `.lowres-icon` / `.icon-dropshadow` | Icon shadows | | n/a |
| `.undershoot-*` | Scroll shadows (prefer ToolbarView) | | n/a |
| `.flat` + `.round` on ToggleGroup | Flat group of buttons / rounded group | | `ToggleGroup::flat/round` |

Deprecated: `.content`, `.sidebar`, `.app-notification`, `.large-title`,
`.opaque`, `.dim-label`, flat HeaderBar.
