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

Row height ≈36 ‡, list padding 6 ‡, row gap 2 ‡, radius 9 (`$menu_radius`) ✓, selected = neutral fg tint (not accent).

## iced mapping

`navigation_sidebar(rows)` / `sidebar_row(icon, label, selected, msg)`.

## Deviations / gaps

No sections/badges/page mode.
