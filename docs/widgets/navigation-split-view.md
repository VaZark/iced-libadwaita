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

`NavigationSplitView::new(sidebar, content).sidebar_width(f).collapsed(b).show_content(b)`.

## Deviations / gaps

Fixed sidebar width; fraction/min/max to be added.
