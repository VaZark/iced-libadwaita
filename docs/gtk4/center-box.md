# CenterBox

GTK4: [`GtkCenterBox`](https://docs.gtk.org/gtk4/class.CenterBox.html)

## What it is

Three children (start, center, end) with the center kept centered "as well as possible"; `shrink-center-last` (4.12).

## Behavior

When start/end differ in size the center child stays centered while space allows. This is the model for `AdwHeaderBar`'s loose centering policy.

## CSS nodes

```
box
```

## iced mapping

`widget::slots::Slots` (internal) implements the same policy for the header bar.
