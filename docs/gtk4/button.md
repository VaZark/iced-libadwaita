# Button

GTK4: [`GtkButton`](https://docs.gtk.org/gtk4/class.Button.html)

## What it is

Runs a callback when pressed and released; holds any child (usually a label). Role: button.

## Behavior

`has-frame`; `can-shrink` (4.12) lets it be smaller than its content. Use `clicked`, not `activate`.

## CSS nodes

```
button
```

## Style classes and states

`.image-button` / `.text-button` (content type), `.flat`, `.keyboard-activating`, and widely used `.suggested-action`, `.destructive-action`, `.circular`. Button-like widgets add `.toggle`, `.popup`, `.scale`, `.lock`, `.color`.

## Adwaita defers

Adwaita defines the appearance and classes (`.flat`, `.raised`, `.pill`, `.circular`, `.suggested-action`, `.destructive-action`, `.osd`, `.card`, `.linked`) and flattens toolbar buttons. See [../design-language/style-classes.md](../design-language/style-classes.md) and [../widgets/button.md](../widgets/button.md).

## iced mapping

`style::button::*`, `widget::*_button`.
