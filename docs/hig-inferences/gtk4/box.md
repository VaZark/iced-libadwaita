# Box

GTK4: [`GtkBox`](https://docs.gtk.org/gtk4/class.Box.html)

## What it is

Lays out children in a row/column with `spacing` **between** children, optional `homogeneous`, baseline alignment.

## CSS nodes

```
box
```

## Style classes and states

`.linked` (Adwaita/GTK theme) merges children visually.

## iced mapping

iced `row!`/`column!`; `.linked` → `style::button::linked(pos)`.
