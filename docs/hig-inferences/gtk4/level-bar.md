# LevelBar

GTK4: [`GtkLevelBar`](https://docs.gtk.org/gtk4/class.LevelBar.html)

## What it is

Level indicator (battery, strength). Default offsets low 0.25, high 0.75, full 1.0; continuous or discrete blocks.

## CSS nodes

```
levelbar[.discrete|.continuous]
╰── trough
    ├── block.filled.<level-name>
    ╰── block.empty
```

## Style classes and states

`.discrete/.continuous`, `.filled/.empty`, offset names (`low`, `high`, `full`).

## Adwaita defers

`--success-bg-color` is the high offset, `--warning-bg-color` the low offset.

## iced mapping

`style::progress_bar::{success,warning,error}` approximate it; no dedicated widget.
