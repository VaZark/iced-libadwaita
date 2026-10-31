# Switch

GTK4: [`GtkSwitch`](https://docs.gtk.org/gtk4/class.Switch.html)

## What it is

On/off light switch; click the trough or drag the slider. `active` is the slider position; `state` is the underlying state shown by trough color and may lag.

## CSS nodes

```
switch
├── image
├── image
╰── slider
```

## Style classes and states

No style classes. Role: switch.

## Spec

Adwaita track ≈46×26 with a 20px knob ‡ (see [../widgets/switch.md](../widgets/switch.md)).

## Adwaita defers

Accent track when on, translucent foreground track when off, white knob with a faint shadow.

## iced mapping

`adw::widget::Switch` (custom widget, 46×26). `style::toggler` styles stock iced `toggler`.
