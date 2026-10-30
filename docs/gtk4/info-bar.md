# InfoBar

GTK4: [`GtkInfoBar`](https://docs.gtk.org/gtk4/class.InfoBar.html)

## What it is

Message bar with an action area; message types `.info .warning .error .question`. **Deprecated since 4.10** (no GTK replacement).

## CSS nodes

```
infobar
```

## Style classes and states

`.info`, `.warning`, `.error`, `.question`, `.close`.

## Adwaita defers

Use `AdwBanner` (accent bar, optional gray/suggested button, `revealed`).

## iced mapping

`adw::widget::banner`.
