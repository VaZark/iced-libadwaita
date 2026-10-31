# ShortcutLabel

libadwaita: [`ShortcutLabel`](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/class.ShortcutLabel.html)

## What it is

Displays a keyboard shortcut from an accelerator string (1.8).

## Anatomy and behavior

Alternatives (space), ranges (`...`), simultaneous (`&`), sequences (`+`); `disabled-text` placeholder.

## CSS nodes and classes

`shortcut-label` with `.keycap` labels and `.dimmed` separators.

## Spec

Keycaps are small rounded translucent boxes.

## iced mapping

`shortcut_label("Ctrl+Shift+P")` renders keycaps; GTK accelerator syntax not parsed.
