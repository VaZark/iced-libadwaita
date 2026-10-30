# HeaderBar

GTK4: [`GtkHeaderBar`](https://docs.gtk.org/gtk4/class.HeaderBar.html)

## What it is

Custom titlebar: like a `GtkCenterBox` with children packed at start/end, title centered relative to the box width, window controls per `decoration-layout`, `show-title-buttons`. Default title label: single line, ellipsized at end, `width-chars` 5, `.title`.

## CSS nodes

```
headerbar
╰── windowhandle
    ╰── box
        ├── box.start
        │   ├── windowcontrols.start
        │   ╰── [other children]
        ├── [title widget]
        ╰── box.end
            ├── [other children]
            ╰── windowcontrols.end
```

## Style classes and states

`.title` on the default title label.

## Adwaita defers

`AdwHeaderBar` adds a **loose** centering policy by default (strict via `centering-policy`), automatic back button, dialog close-only behavior and the window-handle drag. Adwaita wins: see [../widgets/header-bar.md](../widgets/header-bar.md).

## iced mapping

`HeaderBar` (custom three-slot layout, loose centering).
