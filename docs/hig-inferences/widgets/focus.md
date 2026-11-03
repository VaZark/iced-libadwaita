# FocusScope and focus_return

No libadwaita widget: GTK has focus chains and `AdwDialog`'s focus handling built in. After Mantine's [`use-focus-trap`](https://mantine.dev/hooks/use-focus-trap/), [`use-focus-return`](https://mantine.dev/hooks/use-focus-return/) and [`use-focus-within`](https://mantine.dev/hooks/use-focus-within/); the rules are the `primitives::focus` primitive.

## What it is

Keyboard focus for a part of the window. iced has no focus model of its own beyond each widget's `Focusable` state, in tree order; these wrappers read and set which focusable widget inside them is focused, so they work for every widget that takes focus without ids.

## Anatomy and behavior

- **Trap.** `FocusScope::trap(true)` (default): Tab and Shift+Tab move through the scope and wrap at its ends instead of leaving it (`trap_step`).
- **Initial focus.** When the scope appears, the first focusable widget takes focus, or the one named by `initial_focus(id)`: the child's widget id (`entry(..).id(id)`), which the primitive resolves to a place in the scope (`index_of`); an id that is not found leaves the first. `auto_focus(false)` turns it off.
- **Within.** `on_focus_change(|inside| msg)` is sent when focus enters or leaves the scope (`Within`).
- **Return.** `focus_return(content, active)` wraps a base and its modal layer: when `active` turns off, focus goes back to what had it when it turned on (`Return`), once.
- **Components accept `initial_focus`.** `Overlay`, `Dock` and `Popover` take `initial_focus(id)` and pass it to the scope they make of their content; `ColorInput` declares its color code entry as its own. `Overlay` (so every dialog and docked sheet) is a trap with initial focus and a return. The gallery's search bar is a scope that does not trap, so the field takes focus when the search opens.

## iced mapping

`widget::FocusScope` and `focus_return` wrap their content and run a probe over it (counting focusables, reading their ids and which is focused) and a focusing operation, from `update`. Not included: focus rings for widgets iced does not make focusable (buttons).
