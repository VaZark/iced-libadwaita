# NavigationSplitView

libadwaita: [`NavigationSplitView`](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/class.NavigationSplitView.html)

## What it is

Sidebar + content side by side, or a navigation view when collapsed (1.4).

## Anatomy and behavior

- `collapsed` (typically `max-width: 400sp`), `show-content` picks the pane when collapsed.
- Sidebar width: fraction 25%, min 180sp, max 280sp; `sidebar-position`.
- Header bars hide inner window buttons; collapsed content bar shows a back button.

## CSS nodes and classes

`navigation-split-view` > `widget.sidebar-pane`, `widget.content-pane` (collapsed: `navigation-view`).

## Spec

Sidebar bg = sidebar color with a sidebar-border hairline.

## iced mapping

`NavigationSplitView::new(sidebar, content).adapt(width).collapsed_sidebar(CollapsedSidebar::Drawer | FirstPage).show_sidebar(open).on_close_drawer(msg).window_radius(r)`. Collapsed (below 768px or when the content would be under 520px; libadwaita uses `max-width: 400`), the sidebar is a **drawer** by default: it slides over the content from the start edge with a scrim, uses the max sidebar width (280) but at most 85% of the window so it never takes the whole screen, and is opened by an app-owned toggle (`show_sidebar`). `FirstPage` is libadwaita's own collapsed behavior (the sidebar is the first page of a navigation stack, in the content color; `show_content` picks the pane).

## Deviations / gaps

Drawer as the default for the navigation split view is a project choice (libadwaita's collapsed navigation split view is page-based; its _overlay_ split view is the drawer). No swipe gesture and no spring animation yet.
