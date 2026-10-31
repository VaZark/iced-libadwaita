# Paned

GTK4: [`GtkPaned`](https://docs.gtk.org/gtk4/class.Paned.html)

## What it is

Two panes with a draggable handle; per-child `resize`/`shrink`; `wide-handle`.

## CSS nodes

```
paned
├── <child>
├── separator[.wide]
╰── <child>
```

## Style classes and states

`.wide`

## Adwaita defers

Prefer `AdwNavigationSplitView` / `AdwOverlaySplitView` (adaptive).

## iced mapping

`NavigationSplitView` / `OverlaySplitView` (fixed sidebar width; not draggable).
