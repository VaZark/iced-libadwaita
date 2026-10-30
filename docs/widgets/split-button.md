# SplitButton

libadwaita: [`SplitButton`](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/class.SplitButton.html)

## What it is

Button plus dropdown arrow in one control; popover from `menu-model` or custom.

## Anatomy and behavior

`can-shrink`, `dropdown-tooltip`, `direction`. Supports `.suggested-action`, `.destructive-action`, `.flat`, `.raised`; **not** `.pill`.

## CSS nodes and classes

`splitbutton` > `button`, `separator`, `menubutton` > `button.toggle` > `arrow`. `.image-button`/`.text-button`.

## Spec

Linked halves with a 1px separator.

## iced mapping

`split_button(content, on_press, on_menu)`.
