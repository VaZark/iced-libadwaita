# Separator

GTK4: [`GtkSeparator`](https://docs.gtk.org/gtk4/class.Separator.html)

## What it is

Draws a horizontal or vertical line.

## CSS nodes

```
separator
```

## Style classes and states

`.horizontal` / `.vertical`. (libadwaita adds `.spacer` to make it invisible whitespace in toolbars.)

## Adwaita defers

Color is the border color (`--border-color`, foreground at 15%), 1px.

## iced mapping

`style::rule::default`, `container::{card_separator,bar_separator,sidebar_separator}` (different shade colors per context).
