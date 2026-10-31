# WindowControls

GTK4: [`GtkWindowControls`](https://docs.gtk.org/gtk4/class.WindowControls.html)

## What it is

Shows window frame controls (icon, minimize, maximize, close) for one side (`side`) according to `decoration-layout`; `.empty` when nothing to show.

## CSS nodes

```
windowcontrols
├── image.icon
├── button.minimize
├── button.maximize
╰── button.close
```

## Style classes and states

`.empty`, `.start`/`.end`.

## Adwaita defers

Adwaita has no SSD; these live inside `AdwHeaderBar`. In dialogs only close is shown. Buttons and side follow `button-layout`; icons come from the icon theme (see ../design-language/windows-and-csd.md).

## iced mapping

`HeaderBar::window_controls(WindowControls{..})` + `DecorationLayout::detect()`; icons via `Icon::{WindowClose,WindowMinimize,WindowMaximize,WindowRestore}` resolved through the icon theme.
