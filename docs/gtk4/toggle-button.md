# ToggleButton

GTK4: [`GtkToggleButton`](https://docs.gtk.org/gtk4/class.ToggleButton.html)

## What it is

A button that stays pressed until clicked again; can be grouped (radio-like, one active).

## CSS nodes

```
button.toggle
```

## Style classes and states

`.toggle` distinguishes it from a plain button.

## Adwaita defers

`AdwToggleGroup` replaces hand-grouped toggle buttons; the active toggle uses `--active-toggle-bg-color`.

## iced mapping

`style::button::toggle(active, pos)`, `ToggleGroup`.
