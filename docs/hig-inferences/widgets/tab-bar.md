# TabBar, TabView, TabOverview, TabButton

libadwaita: [`TabBar`](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/class.TabBar.html)

## What it is

Tabs for an `AdwTabView`: scrollable, pinned tabs, autohide, `expand-tabs`, action widgets, drag-drop. `TabOverview` shows a thumbnail grid; `TabButton` shows the tab count.

## Anatomy and behavior

`.inline` removes the headerbar look. TabOverview: search, new-tab button, pinned cards.

## CSS nodes and classes

`tabbar` (`.inline`); `taboverview`; `tabbutton`.

## iced mapping

`tab_bar(tabs, active, on_select, on_close)`, `tab_button(count, msg)`.

## Deviations / gaps

No overview, drag, pinning, scrolling.
