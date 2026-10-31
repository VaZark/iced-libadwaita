# CheckButton

GTK4: [`GtkCheckButton`](https://docs.gtk.org/gtk4/class.CheckButton.html)

## What it is

Indicator plus label. Grouped check buttons become radio buttons. Supports an inconsistent (mixed) state. Space/Enter activates.

## CSS nodes

```
checkbutton[.text-button][.grouped]
├── check   (or `radio` when grouped)
╰── [label]
```

## Style classes and states

`.text-button`, `.grouped`; `libadwaita` adds `.selection-mode` (larger round check for selecting list items).

## Adwaita defers

Checked fill is the accent (`--accent-bg-color`) with `--accent-fg-color` mark; unchecked is a translucent fg tint with a border.

## iced mapping

`style::checkbox`, `style::radio` (stock iced `checkbox`/`radio`). No inconsistent state.
