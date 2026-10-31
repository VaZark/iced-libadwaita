# SpinButton

GTK4: [`GtkSpinButton`](https://docs.gtk.org/gtk4/class.SpinButton.html)

## What it is

Numeric entry with up/down buttons; adjustment (value, range, step, page), `digits`, `numeric`, `snap-to-ticks`, `wrap`, `update-policy`.

## CSS nodes

```
spinbutton.horizontal
├── text
│   ├── undershoot.left
│   ╰── undershoot.right
├── button.down
╰── button.up
(vertical: button.up, text, button.down)
```

## Style classes and states

`.horizontal`/`.vertical`, `.up`, `.down`

## Adwaita defers

`AdwSpinRow` embeds it; buttons are flat/linked.

## iced mapping

`SpinRow` (− value +).
