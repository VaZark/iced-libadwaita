# MenuButton

GTK4: [`GtkMenuButton`](https://docs.gtk.org/gtk4/class.MenuButton.html)

## What it is

Shows a popover or `GMenuModel` menu when clicked; content is an icon, a label, or an arrow. If there is no room below, it opens above; it is pushed in if it would go off-screen.

## CSS nodes

```
menubutton
╰── button.toggle
    ╰── <content>
        ╰── [arrow]
```

## Style classes and states

`.image-button`, `.text-button`, `.arrow-button`, arrow direction `.none .up .down .left .right`, optional `.circular` on `menubutton`.

## Adwaita defers

In toolbars menu buttons with an arrow are flat automatically; supports `.suggested-action`, `.destructive-action`.

## iced mapping

`menu_button` / `primary_menu` (popover or full-page presentation); see [../widgets/menu-button.md](../widgets/menu-button.md).
