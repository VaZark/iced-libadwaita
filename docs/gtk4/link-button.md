# LinkButton

GTK4: [`GtkLinkButton`](https://docs.gtk.org/gtk4/class.LinkButton.html)

## What it is

A button holding a hyperlink; default action launches the URI; `visited` state; context menu with Copy.

## CSS nodes

```
button.link
```

## Style classes and states

`.link`

## Adwaita defers

Link color is the accent standalone color (`--accent-color`).

## iced mapping

`style::button::link` (no underline: iced buttons cannot underline children; no visited state).
