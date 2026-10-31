# Popover

GTK4: [`GtkPopover`](https://docs.gtk.org/gtk4/class.Popover.html)

## What it is

Bubble popup attached to a parent; positioned by `position`, `pointing-to`, `offset`; modal by default (`autohide`: click outside or Esc closes); optional arrow; cascade popdown.

## CSS nodes

```
popover.background[.menu]
├── arrow
╰── contents
    ╰── <child>
```

## Style classes and states

`.background` always; `.menu` for menus.

## Adwaita defers

Surface is `--popover-bg-color`, radius 12px, shaded border (`--popover-shade-color`). Adwaita popovers have **no arrow** by default in menus.

## iced mapping

`Dropdown`'s overlay (click-outside/Esc to close). Position: below the trigger, flips above if no room.
