# OverlaySplitView

libadwaita: [`OverlaySplitView`](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/class.OverlaySplitView.html)

## What it is

Sidebar beside content, or as an overlay when collapsed (1.4).

## Anatomy and behavior

`show-sidebar`, `pin-sidebar`, `sidebar-position`, gestures; same width defaults (25% / 180 / 280). Collapsed uses `max-sidebar-width` when possible.

## CSS nodes and classes

`overlay-split-view`; collapsed sidebar `.overlay-pane`.

## iced mapping

`OverlaySplitView::new(sidebar, content).collapsed().show_sidebar().on_close()`.
