# Splitter

GTK4: [`GtkPaned`](https://docs.gtk.org/gtk4/class.Paned.html). After Mantine's [Splitter](https://mantine.dev/core/splitter/).

## What it is

Panes along one axis with a draggable handle between each pair. libadwaita prefers the adaptive `AdwNavigationSplitView` and `AdwOverlaySplitView` for a sidebar; a splitter is for panes the user sizes: an editor and a terminal, a list and a preview.

## Anatomy and behavior

- **Sizes** ([`primitives::splitter`](../../lib/primitives.md)). A pane is fixed (`SplitterPane::fixed(px)`: it keeps its size when the container changes and changes only when its own handle is dragged) or flexible (`SplitterPane::weight(w)`: flexible panes share what the fixed ones leave). If the fixed panes exceed the container they shrink in proportion.
- **Limits.** `min` / `max` per pane, in pixels or as a fraction of the container.
- **Dragging.** A handle moves space between its two neighbours. `Redistribute::Nearest` takes from the next pane beyond when the neighbour is at its limit.
- **Collapsing.** A collapsible pane dragged past half its minimum collapses to nothing and comes back at the size it had when dragged out past half its minimum; `collapse`, `expand` and `toggle` do it from code.
- **Handle.** A hairline with a small grip; `with_handle(false)` hides the grip (the line still drags), `line_size(px)` sets the thickness. The handle is highlighted in the accent color while hovered or dragged.
- Splitters nest: put one in a pane. Orientation is horizontal (side by side) or vertical (stacked).

## iced mapping

`primitives::splitter` is pure and unit tested (resolution of fixed and flexible panes, limits, redistribution, collapsing, write-back in each pane's own unit). `widget::Splitter` is controlled: the app owns one `SplitterState`, the widget speaks one message (`SplitterEvent`) and the app applies it with `state.apply(event)`. A drag publishes the distance between where the handle is and where the pointer holds it, so it stays true when a limit stops the handle. Not included: moving a handle with the keyboard (iced has no focus model).
