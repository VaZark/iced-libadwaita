# Expander

GTK4: [`GtkExpander`](https://docs.gtk.org/gtk4/class.Expander.html)

## What it is

Reveals or hides a child; arrow like tree views; `expanded`, label or label widget.

## CSS nodes

```
expander-widget
╰── box
    ├── title
    │   ├── expander
    │   ╰── <label widget>
    ╰── <child>
```

## Style classes and states

`:checked` on the arrow while open.

## Adwaita defers

Use `AdwExpanderRow` in boxed lists.

## iced mapping

`ExpanderRow`.
