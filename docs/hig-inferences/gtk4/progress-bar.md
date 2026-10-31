# ProgressBar

GTK4: [`GtkProgressBar`](https://docs.gtk.org/gtk4/class.ProgressBar.html)

## What it is

Progress of a long operation: percentage (`fraction`) or activity (`pulse`, `pulse-step`); optional text; `inverted`.

## CSS nodes

```
progressbar[.osd]
├── [text]
╰── trough[.empty][.full]
    ╰── progress[.pulse]
```

## Style classes and states

`.osd`; `progress` gets `.left/.right/.top/.bottom` at ends.

## Spec

Thickness ≈8px ‡; fully rounded.

## Adwaita defers

`.osd` makes the bar thinner with no visible trough (window-top overlay).

## iced mapping

`adw::widget::progress_bar` (8px), `style::progress_bar::{default,success,warning,error}`.
