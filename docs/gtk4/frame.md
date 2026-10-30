# Frame

GTK4: [`GtkFrame`](https://docs.gtk.org/gtk4/class.Frame.html)

## What it is

Single-child wrapper with a border and optional label; clips its child (can round corners).

## CSS nodes

```
frame
├── <label widget>
╰── <child>
```

## Style classes and states

(`.frame` is a libadwaita utility class: `1px solid --border-color`)

## iced mapping

`container` with `style::container::*` borders.
