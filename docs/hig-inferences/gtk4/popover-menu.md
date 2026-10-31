# PopoverMenu

GTK4: [`GtkPopoverMenu`](https://docs.gtk.org/gtk4/class.PopoverMenu.html)

## What it is

Popover that renders a `GMenuModel`: items, sections (with optional heading and horizontal-button display hints), submenus (nested or sliding), custom children; shows accelerators.

## CSS nodes

```
popover.menu
╰── …  button.model per item
```

## Style classes and states

`.menu`, `.model`, `.horizontal`, display hints (`horizontal-buttons`, `circular-buttons`, `inline-buttons`).

## Adwaita defers

Item rows are ≈32px high with 6px radius and a hover tint ‡; sections separated by a hairline.

## iced mapping

`menu_item` + `popover` helpers; no `GMenuModel`, sections or submenus yet.
