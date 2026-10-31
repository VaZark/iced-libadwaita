# ScrolledWindow

GTK4: [`GtkScrolledWindow`](https://docs.gtk.org/gtk4/class.ScrolledWindow.html)

## What it is

Container that scrolls a child; a non-scrollable child is wrapped in a `GtkViewport`.

## Behavior

- Policies per axis (`hscrollbar-policy`, `vscrollbar-policy`); no internal scrollbars when both are _never_/_external_.
- `overlay-scrolling`, `kinetic-scrolling` (touch), `window-placement`.
- `min-content-width/height`, `max-content-width/height`, `propagate-natural-width/height` control sizing.
- Signals `edge-overshot`, `edge-reached`.

## CSS nodes

```
scrolledwindow[.frame]
├── (child)
├── scrollbar (see scrollbar.md)
├── junction  (when both scrollbars show)
├── overshoot.top|bottom|left|right
╰── undershoot.top|bottom|left|right
```

## Style classes and states

`.frame` when `has-frame`; scrollbars get `.left/.right/.top/.bottom`, `.overlay-indicator`, `.dragging`, `.hovering`. (The GTK page notes the over/undershoot nodes are subnodes with positional classes, not `.undershoot-*` classes; libadwaita's `.undershoot-*` style classes are a separate, Adwaita-level convenience.)

## Adwaita defers

`AdwToolbarView` provides the scroll undershoot between content and bars; `AdwClampScrollable` and `AdwClamp` constrain scrolled content width.

## iced mapping

`scrolled_window(content)` (vertical, overlay Adwaita scrollbar), `.horizontal()` and `.both()`. `PreferencesPage`, the gallery pages and popover menus use it.
