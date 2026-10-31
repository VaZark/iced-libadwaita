# Overlay

GTK4: [`GtkOverlay`](https://docs.gtk.org/gtk4/class.Overlay.html)

## What it is

Places overlay widgets over one main child by `halign`/`valign` + margins; only the main child affects sizing.

## CSS nodes

```
overlay
```

## Style classes and states

`.left/.right/.top/.bottom` on edge-positioned overlays.

## iced mapping

iced `stack!` (used by dialogs, toasts, bottom sheet, window frame).
