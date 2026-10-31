# Sidebar and ViewSwitcherSidebar

libadwaita: [`Sidebar`](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/class.Sidebar.html)

## What it is

Adaptive sidebar of sections and items (1.9); `ViewSwitcherSidebar` builds one from a ViewStack.

## Anatomy and behavior

- Zero or one selected item; `activated`. Mode **sidebar** (`.navigation-sidebar` list) or **page** (boxed-list preferences page) when collapsed in a NavigationSplitView.
- Items: icon, label, unread dot or badge; sections with titles; filter, placeholder, prefix/suffix, context menu, drag-and-drop.
- `.navigation-sidebar`: rounded padded items, neutral selection color, no list background.

## CSS nodes and classes

`sidebar` > list `.navigation-sidebar` (or `preferencespage`).

## Spec

Row height ≈36 ‡, row side margin 6 and bottom margin 2, radius 9 (`$menu_radius`) ✓, selected = neutral fg tint (not accent).

## iced mapping

`navigation_sidebar(rows)` / `sidebar_row(icon, label, selected, msg)`.

#### Padding levels

`SidebarPadding::{Sm, Md, Lg}` on `SidebarList::padding` (or one row with `SidebarItem::padding`;
`sidebar_row_padded` for the bare function). Each level steps the row height, the row's side
padding, the gap between rows and the horizontal list inset together, relative to the density's base:

| Level | Resembles | Height | Side | Gap | Horizontal inset |
|---|---|---|---|---|---|
| `Sm` | Mantine `NavLink` (dense) | 28 | 8 | 1 | 4 |
| `Md` (default) | Nautilus | 36 | 12 | 2 | 6 |
| `Lg` | GNOME Settings | text height + 24 (≈43–44) | 14 | 2 | 6 |

`Lg` is from the source (✓): gnome-control-center's `cc-panel-list.c` `row_data_new()` puts a
`GtkGrid` in each row with margins 12 top and bottom, 6 at the sides and 12 column spacing, inside
libadwaita's `.navigation-sidebar > row` (min-height 36, padding `0 8px`, margin `0 6px 2px`).
So the row is 24 + one line of text, approximately 43–44 tall (12 + the inherited font's line + 12); its content starts 14
from the row edge (8 + 6), and the icon-label gap is 12, which is already our base. Row gap 2 and
horizontal inset 6 are the base. The list also has 6px top padding and 4px bottom padding,
plus the last row's 2px bottom margin: an effective 6px inset on all sides.
See the [libadwaita sidebar stylesheet](https://github.com/GNOME/libadwaita/blob/main/src/stylesheet/widgets/_sidebars.scss).
`Lg` keeps the inherited interface font and 16px icon size, and grows with larger text;
it does not force a 44px height. Selection uses the neutral 10% foreground tint and 9px
radius; its border/outline does not reserve extra space.
`Sm` and `Md` are ‡ (from memory of Mantine and Nautilus, not measured).
